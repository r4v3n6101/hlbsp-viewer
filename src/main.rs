mod support;

use file::{bsp::RawMap, wad::Archive};
use glium::{glutin, Surface};
use render::map::MapRender;
use std::path::PathBuf;
use structopt::StructOpt;
use support::{init_logger, Camera};

const MOVE_SPEED: f32 = 0.01;
// Safe, because there's no multiple thread accessing this
static mut MOUSE_GRABBED: bool = true;

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
}

fn main() {
    init_logger().unwrap();
    let opt = Opt::from_args();
    let file = std::fs::read(&opt.bsp_path).unwrap();
    let map = RawMap::parse(&file).unwrap();
    start_window_loop(&map, &opt.wad_path);
}

fn get_window_center(window: &glutin::window::Window) -> glutin::dpi::PhysicalPosition<f64> {
    let out_pos = window.outer_position().unwrap();
    let out_size = window.outer_size();
    glutin::dpi::PhysicalPosition {
        x: f64::from(out_pos.x + out_size.width as i32 / 2),
        y: f64::from(out_pos.y + out_size.height as i32 / 2),
    }
}

fn grab_cursor(window: &glutin::window::Window) {
    window.set_cursor_visible(false);
    window.set_cursor_grab(true).unwrap();
    window
        .set_cursor_position(get_window_center(window))
        .unwrap();
}

fn ungrab_cursor(window: &glutin::window::Window) {
    window.set_cursor_visible(true);
    window.set_cursor_grab(false).unwrap();
}

fn start_window_loop(map: &RawMap, wad_path: &[PathBuf]) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let cb = glutin::ContextBuilder::new();

    let mut camera = Camera::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    grab_cursor(display.gl_window().window());

    let mut map_render = MapRender::new(map, &display);
    wad_path
        .iter()
        .map(|path| std::fs::read(path).unwrap())
        .for_each(|file| {
            let archive = Archive::parse(&file).unwrap();
            map_render.load_from_archive(&display, &archive)
        });

    let draw_params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

    event_loop.run(move |event, _, control_flow| {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        match event {
            glutin::event::Event::WindowEvent {
                window_id: _,
                event: wevent,
            } => *control_flow = process_window(window, &wevent, &mut camera),
            glutin::event::Event::MainEventsCleared => window.request_redraw(),
            glutin::event::Event::RedrawRequested(_) => {
                let mut target = display.draw();
                let mvp = camera.perspective() * camera.view();

                target.clear_color_and_depth((1.0, 1.0, 0.0, 1.0), 1.0);
                map_render.render(&mut target, mvp.into(), &draw_params);
                target.finish().unwrap();
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
            if input.state == glutin::event::ElementState::Pressed {
                if let Some(virt_keycode) = input.virtual_keycode {
                    match virt_keycode {
                        glutin::event::VirtualKeyCode::W => camera.move_forward(MOVE_SPEED),
                        glutin::event::VirtualKeyCode::S => camera.move_back(MOVE_SPEED),
                        glutin::event::VirtualKeyCode::A => camera.move_left(MOVE_SPEED),
                        glutin::event::VirtualKeyCode::D => camera.move_right(MOVE_SPEED),
                        glutin::event::VirtualKeyCode::G => unsafe {
                            if MOUSE_GRABBED {
                                ungrab_cursor(window);
                                MOUSE_GRABBED = false;
                            } else {
                                grab_cursor(window);
                                MOUSE_GRABBED = true;
                            }
                        },
                        glutin::event::VirtualKeyCode::Q => {
                            return glutin::event_loop::ControlFlow::Exit
                        }
                        _ => (),
                    }
                }
            }
            glutin::event_loop::ControlFlow::Poll
        }
        glutin::event::WindowEvent::CursorMoved {
            position: glutin::dpi::PhysicalPosition { x, y },
            ..
        } => {
            unsafe {
                if MOUSE_GRABBED {
                    let mouse_pos = get_window_center(window);
                    let (dx, dy) = (x - mouse_pos.x, y - mouse_pos.y);
                    window
                        .set_cursor_position(get_window_center(window))
                        .unwrap();
                    camera.rotate_by((-dy * 0.1) as f32, (dx * 0.1) as f32, 0.0);
                }
            }
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
