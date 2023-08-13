mod camera;
mod render;

use glium::{
    glutin::{self, event::DeviceEvent},
    Surface,
};
use render::Level;
use std::path::{Path, PathBuf};
use clap::Parser;
use camera::Camera;

const MOVE_SPEED: f32 = 100.0;
const CAMERA_OFFSET: f32 = 64.0;
// Safe, because there's no multiple thread accessing this
static mut MOUSE_GRABBED: bool = true;

/// A program allows you to view hlbsp maps (bsp v30)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to bsp map
    #[arg(short, long)]
    bsp_path: PathBuf,
    /// Path to wad files which are required to load textures
    #[arg(short, long)]
    wad_path: Vec<PathBuf>,
    /// Path to directory stores skybox textures
    #[arg(short, long)]
    skybox_path: Option<PathBuf>,
}

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();

    start_window_loop(args.bsp_path, &args.wad_path, args.skybox_path);
}

fn grab_cursor(window: &glutin::window::Window) {
    window.set_cursor_visible(false);
    window
        .set_cursor_grab(glutin::window::CursorGrabMode::Locked)
        .unwrap();
}

fn ungrab_cursor(window: &glutin::window::Window) {
    window.set_cursor_visible(true);
    window
        .set_cursor_grab(glutin::window::CursorGrabMode::None)
        .unwrap();
}

fn start_window_loop<P: AsRef<Path>>(bsp_path: P, wad_path: &[P], skybox_path: Option<P>) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let cb = glutin::ContextBuilder::new();

    let mut camera = Camera::new(1024.0, 768.0, 90.0f32.to_radians(), 1.0, 8192.0);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    grab_cursor(display.gl_window().window());

    let level_render = Level::new(&display, bsp_path, wad_path, skybox_path);
    if let Some((x, y, z)) = level_render.start_point() {
        camera.set_position(x, y + CAMERA_OFFSET, z);
    }

    let draw_params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
        depth: glium::Depth {
            test: glium::DepthTest::IfLessOrEqual,
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
            glutin::event::Event::MainEventsCleared => window.request_redraw(),
            glutin::event::Event::RedrawRequested(_) => {
                let mut frame = display.draw();

                let projection = camera.perspective();
                let view = camera.view();

                frame.clear_color_and_depth((1.0, 1.0, 0.0, 1.0), 1.0);
                level_render.render(&mut frame, projection, view, &draw_params);
                frame.finish().unwrap();
            }
            glutin::event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => unsafe {
                if MOUSE_GRABBED {
                    camera.rotate_by((-delta.1 * 0.1) as f32, (delta.0 * 0.1) as f32, 0.0);
                }
            },
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
        glutin::event::WindowEvent::Resized(glutin::dpi::PhysicalSize { width, height }) => {
            camera.aspect = (*width as f32) / (*height as f32);
            glutin::event_loop::ControlFlow::Poll
        }
        glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
        _ => glutin::event_loop::ControlFlow::Poll,
    }
}
