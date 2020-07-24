mod support;

use glium::{glutin, program, uniform, Surface};
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

    start_window_loop(&map, opt.mip_level);
}

fn substitute_wad_textures<'a>(map: &mut IndexedMap<'a>, archives: &'a [wad::Archive<'a>]) {
    archives.iter().for_each(|a| map.replace_empty_textures(a));
}

fn start_window_loop(map: &IndexedMap, mip_level: usize) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let cb = glutin::ContextBuilder::new();

    let mut camera = Camera::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let root_model = map.root_model();
    let vertices: Vec<GlVertex> = map
        .calculate_vertices(root_model)
        .into_iter()
        .map(GlVertex::from)
        .collect();
    let textured_ibos: Vec<_> = map
        .indices_triangulated(root_model)
        .into_iter()
        .filter(|&(tex, _)| tex.name() != "sky" && tex.name() != "aaatrigger")
        .group_by(|&(tex, _)| tex)
        .into_iter()
        .map(|(tex, group)| {
            let dims = (
                tex.width(mip_level).unwrap(),
                tex.height(mip_level).unwrap(),
            );
            let image =
                glium::texture::RawImage2d::from_raw_rgb(tex.pixels(mip_level).unwrap(), dims);
            let texture = glium::texture::Texture2d::new(&display, image).unwrap();

            let indices: Vec<_> = group
                .flat_map(|(_, indices)| indices.into_iter().rev().map(|x| x as u16))
                .collect();
            let ibo = glium::index::IndexBuffer::new(
                &display,
                glium::index::PrimitiveType::TrianglesList,
                &indices,
            )
            .unwrap();
            (texture, ibo)
        })
        .collect();

    let origin = root_model.origin;
    let origin = [origin.0, origin.1, origin.2];
    let vbo = glium::vertex::VertexBuffer::new(&display, &vertices).unwrap();
    let program = program!(&display,
         140 => {
             vertex: include_str!("../shaders/vert.glsl"),
             fragment: include_str!("../shaders/frag.glsl"),
         },
    )
    .unwrap();
    let draw_params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..glium::Depth::default()
        },
        ..glium::DrawParameters::default()
    };

    event_loop.run(move |event, _, control_flow| {
        if let glutin::event::Event::WindowEvent {
            window_id: _,
            event: wevent,
        } = event
        {
            *control_flow = process_window(&wevent, &mut camera)
        } else {
            let next_frame_time =
                std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

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

fn process_window(
    wevent: &glutin::event::WindowEvent,
    camera: &mut Camera,
) -> glutin::event_loop::ControlFlow {
    match wevent {
        glutin::event::WindowEvent::KeyboardInput { input, .. } => {
            let mut exit = false;
            if let Some(virt_keycode) = input.virtual_keycode {
                match virt_keycode {
                    glutin::event::VirtualKeyCode::W => camera.position.z += 0.01,
                    glutin::event::VirtualKeyCode::A => camera.position.x -= 0.01,
                    glutin::event::VirtualKeyCode::S => camera.position.z -= 0.01,
                    glutin::event::VirtualKeyCode::D => camera.position.x += 0.01,
                    glutin::event::VirtualKeyCode::Space => camera.position.y += 0.01,
                    glutin::event::VirtualKeyCode::LControl => camera.position.y -= 0.01,

                    glutin::event::VirtualKeyCode::Up => camera.rotate_by(1.0, 0.0, 0.0),
                    glutin::event::VirtualKeyCode::Down => camera.rotate_by(-1.0, 0.0, 0.0),
                    glutin::event::VirtualKeyCode::Left => camera.rotate_by(0.0, -1.0, 0.0), // TODO : glitch
                    glutin::event::VirtualKeyCode::Right => camera.rotate_by(0.0, 1.0, 0.0),

                    glutin::event::VirtualKeyCode::Q => exit = true,
                    _ => (),
                }
            }
            if exit {
                glutin::event_loop::ControlFlow::Exit
            } else {
                glutin::event_loop::ControlFlow::Poll
            }
        }
        glutin::event::WindowEvent::Resized(glutin::dpi::PhysicalSize { width, height }) => {
            camera.aspect_ratio = (*width as f32) / (*height as f32);
            glutin::event_loop::ControlFlow::Poll
        }
        glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
        _ => glutin::event_loop::ControlFlow::Poll,
    }
}
