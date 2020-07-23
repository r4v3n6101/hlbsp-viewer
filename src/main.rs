mod support;

use glium::{
    glutin::{
        dpi::{LogicalSize, PhysicalSize},
        event::{Event, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    index::{IndexBuffer, PrimitiveType},
    program,
    texture::{RawImage2d, Texture2d},
    uniform,
    vertex::VertexBuffer,
    Display, Surface,
};
use itertools::Itertools;
use map_impl::IndexedMap;
use std::path::PathBuf;
use structopt::StructOpt;
use support::{Camera, GlVertex};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hlbsp_viewer",
    about = "A program allows you to view hlbsp maps (bsp v30)"
)]
struct Opt {
    #[structopt(short, long = "bsp", parse(from_os_str), help = "Path to bsp map")]
    bsp_path: PathBuf,
    #[structopt(
        short,
        long = "wad",
        parse(from_os_str),
        help = "Paths of wad files which are required to load textures"
    )]
    wad_path: Vec<PathBuf>,
    #[structopt(
        short,
        long = "mip",
        default_value = "0",
        help = "Mip level of textures to load"
    )]
    mip_level: usize,
}

fn main() {
    let opt = Opt::from_args();
    let file = std::fs::read(&opt.bsp_path).unwrap();
    let bsp_map = bsp::RawMap::parse(&file).unwrap();
    let mut map = IndexedMap::new(&bsp_map);
    let wad_files: Vec<_> = opt
        .wad_path
        .iter()
        .map(|path| std::fs::read(path).unwrap())
        .collect();
    let archives: Vec<_> = wad_files
        .iter()
        .map(|file| wad::Archive::parse(&file).unwrap())
        .collect();
    substitute_wad_textures(&mut map, &archives);

    start_window_loop(&map, 0); // TODO : tempory mip_level
}

fn substitute_wad_textures<'a>(map: &mut IndexedMap<'a>, archives: &'a [wad::Archive<'a>]) {
    archives.iter().for_each(|a| map.replace_empty_textures(a));
}

fn start_window_loop(map: &IndexedMap, mip_level: usize) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(LogicalSize::new(1024.0, 768.0));
    let cb = ContextBuilder::new();

    let mut camera = Camera::new();
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let root_model = map.root_model();
    let vertices: Vec<GlVertex> = map
        .calculate_vertices(root_model)
        .into_iter()
        .map(GlVertex::from)
        .collect();
    let textured_ibos: Vec<_> = map
        .indices_triangulated(root_model)
        .into_iter()
        .group_by(|&(tex, _)| tex)
        .into_iter()
        .map(|(tex, group)| {
            let dims = (
                tex.width(mip_level).unwrap(),
                tex.height(mip_level).unwrap(),
            );
            let image = RawImage2d::from_raw_rgb(tex.pixels(mip_level).unwrap(), dims);
            let texture = Texture2d::new(&display, image).unwrap();

            let indices: Vec<_> = group
                .flat_map(|(_, indices)| indices.into_iter().rev().map(|x| x as u16))
                .collect();
            let ibo = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &indices).unwrap();
            (texture, ibo)
        })
        .collect();

    let origin = root_model.origin;
    let origin = [origin.0, origin.1, origin.2];
    let vbo = VertexBuffer::new(&display, &vertices).unwrap();
    let program = program!(&display,
         140 => {
             vertex: "
                #version 140

                uniform mat4 proj;
                uniform mat4 view;
                uniform vec3 origin;

                in vec3 position;
                in vec2 tex_coords;
                
                out vec2 o_tex_coords;

                void main() {
                    o_tex_coords =  tex_coords;
                    vec3 pos = (position - origin) * 0.001;
                    gl_Position = proj * view * vec4(vec3(pos.x, pos.z, -pos.y), 1.0);
                }
            ",

             fragment: "
                #version 140
                in vec2 o_tex_coords;
                uniform sampler2D tex;

                void main() {
                    gl_FragColor = texture2D(tex, o_tex_coords);
                }
            "
         },
    )
    .unwrap();
    let draw_params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id: _,
            event: wevent,
        } => *control_flow = process_window(wevent, &mut camera),
        _ => {
            let next_frame_time =
                std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
            *control_flow = ControlFlow::WaitUntil(next_frame_time);

            let mut target = display.draw();
            target.clear_color_and_depth((1.0, 1.0, 0.0, 1.0), 1.0);
            let persp: [[_; 4]; 4] = camera.perspective().into();
            let view: [[_; 4]; 4] = camera.view().into();

            textured_ibos.iter().for_each(|(tex, ibo)| {
                let uniforms = uniform! {
                    proj: persp,
                    view: view,
                    origin: origin,
                    tex: tex,
                };
                target
                    .draw(&vbo, ibo, &program, &uniforms, &draw_params)
                    .unwrap();
            });
            target.finish().unwrap();
        }
    });
}

fn process_window(wevent: WindowEvent, camera: &mut Camera) -> ControlFlow {
    match wevent {
        WindowEvent::KeyboardInput { input, .. } => {
            let mut exit = false;
            if let Some(virt_keycode) = input.virtual_keycode {
                match virt_keycode {
                    VirtualKeyCode::W => camera.position.z += 0.01,
                    VirtualKeyCode::A => camera.position.x -= 0.01,
                    VirtualKeyCode::S => camera.position.z -= 0.01,
                    VirtualKeyCode::D => camera.position.x += 0.01,
                    VirtualKeyCode::Space => camera.position.y += 0.01,
                    VirtualKeyCode::LControl => camera.position.y -= 0.01,

                    VirtualKeyCode::Up => camera.rotate_by(1.0, 0.0, 0.0),
                    VirtualKeyCode::Down => camera.rotate_by(-1.0, 0.0, 0.0),
                    VirtualKeyCode::Left => camera.rotate_by(0.0, -1.0, 0.0), // TODO : glitch
                    VirtualKeyCode::Right => camera.rotate_by(0.0, 1.0, 0.0),

                    VirtualKeyCode::Q => exit = true,
                    _ => (),
                }
            }
            if exit {
                ControlFlow::Exit
            } else {
                ControlFlow::Poll
            }
        }
        WindowEvent::Resized(PhysicalSize { width, height }) => {
            camera.aspect_ratio = (width as f32) / (height as f32);
            ControlFlow::Poll
        }
        WindowEvent::CloseRequested => ControlFlow::Exit,
        _ => ControlFlow::Poll,
    }
}
