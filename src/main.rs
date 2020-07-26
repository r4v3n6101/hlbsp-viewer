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

fn get_window_center(window: &glutin::window::Window) -> glutin::dpi::PhysicalPosition<f64> {
    let out_pos = window.outer_position().unwrap();
    let out_size = window.outer_size();
    glutin::dpi::PhysicalPosition {
        x: (out_pos.x + out_size.width as i32 / 2) as f64,
        y: (out_pos.y + out_size.height as i32 / 2) as f64,
    }
}

fn grab_cursor(window: &glutin::window::Window) {
    window.set_cursor_visible(false);
    window.set_cursor_grab(true).unwrap();
    window
        .set_cursor_position(get_window_center(window))
        .unwrap();
}

fn start_window_loop(map: &IndexedMap, mip_level: usize) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let cb = glutin::ContextBuilder::new();

    let mut camera = Camera::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    grab_cursor(display.gl_window().window());

    let root_model = map.root_model();
    let vertices: Vec<GlVertex> = map
        .calculate_vertices(root_model)
        .into_iter()
        .map(GlVertex::from)
        .collect();
    let mut indices_triangulated = map.indices_triangulated(root_model);
    // TODO : temporal until a new group by
    indices_triangulated.sort_by(|(a, _), (b, _)| a.name().partial_cmp(b.name()).unwrap());
    let textured_ibos: Vec<_> = indices_triangulated
        .into_iter()
        .filter(|&(tex, _)| tex.name() != "sky" && tex.name() != "aaatrigger")
        .group_by(|&(tex, _)| tex)
        .into_iter()
        .inspect(|(tex, _)| println!("Texture: {}", tex.name()))
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
        let gl_window = display.gl_window();
        let window = gl_window.window();
        match event {
            glutin::event::Event::WindowEvent {
                window_id: _,
                event: wevent,
            } => *control_flow = process_window(window, &wevent, &mut camera),
            glutin::event::Event::RedrawRequested(_) => {
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
            glutin::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {
                let next_frame_time =
                    std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
                *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
            }
        }
    });
}

fn process_window(
    window: &glutin::window::Window,
    wevent: &glutin::event::WindowEvent,
    camera: &mut Camera,
) -> glutin::event_loop::ControlFlow {
    match wevent {
        glutin::event::WindowEvent::KeyboardInput { input, .. } => {
            let mut exit = false;
            if let Some(virt_keycode) = input.virtual_keycode {
                match virt_keycode {
                    glutin::event::VirtualKeyCode::W => camera.move_forward(0.05),
                    glutin::event::VirtualKeyCode::S => camera.move_back(0.05),
                    glutin::event::VirtualKeyCode::A => camera.move_left(0.05),
                    glutin::event::VirtualKeyCode::D => camera.move_right(0.05),
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
        glutin::event::WindowEvent::CursorMoved {
            position: glutin::dpi::PhysicalPosition { x, y },
            ..
        } => {
            let mouse_pos = get_window_center(window);
            let (dx, dy) = (x - mouse_pos.x, y - mouse_pos.y);
            window
                .set_cursor_position(get_window_center(window))
                .unwrap();
            camera.rotate_by((-dy * 0.1) as f32, (dx * 0.1) as f32, 0.0);
            glutin::event_loop::ControlFlow::Poll
        }
        glutin::event::WindowEvent::Resized(glutin::dpi::PhysicalSize { width, height }) => {
            camera.aspect_ratio = (*width as f32) / (*height as f32);
            glutin::event_loop::ControlFlow::Poll
        }
        glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
        _ => glutin::event_loop::ControlFlow::Poll,
    }
}
