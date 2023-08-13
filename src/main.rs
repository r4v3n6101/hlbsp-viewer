mod render;
mod cubemap;
mod state;

use clap::Parser;
use glium::{
    glutin::{self, event::DeviceEvent},
    Surface,
};
use state::State;
use std::path::PathBuf;
use anyhow::Result;

// Safe, because there's no multiple thread accessing this
static mut MOUSE_GRABBED: bool = true;

/// A program allows you to view hlbsp maps (bsp v30)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to bsp map
    #[arg(short, long)]
    pub bsp_path: PathBuf,
    /// Path to wad files which are required to load textures
    #[arg(short, long)]
    pub wad_paths: Vec<PathBuf>,
    /// Path to directory stores skybox textures
    #[arg(short, long)]
    pub skybox_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();

    start_window_loop(&args)
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

fn start_window_loop(args: &Args) -> Result<()> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
    let cb = glutin::ContextBuilder::new();

    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    grab_cursor(display.gl_window().window());

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

    let mut state = State::new(&display, args)?;

    event_loop.run(move |event, _, control_flow| {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        match event {
            glutin::event::Event::WindowEvent {
                window_id,
                ref event,
            } if window.id() == window_id => {
                state.camera.process_events(event);
                match event {
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == glutin::event::ElementState::Pressed {
                            if let Some(virt_keycode) = input.virtual_keycode {
                                match virt_keycode {
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
                                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        *control_flow = glutin::event_loop::ControlFlow::Poll;
                    }
                    glutin::event::WindowEvent::Resized(glutin::dpi::PhysicalSize {
                        width,
                        height,
                    }) => {
                        state.camera.aspect = (*width as f32) / (*height as f32);
                        *control_flow = glutin::event_loop::ControlFlow::Poll;
                    }
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit
                    }
                    _ => *control_flow = glutin::event_loop::ControlFlow::Poll,
                }
            }
            glutin::event::Event::MainEventsCleared => window.request_redraw(),
            glutin::event::Event::RedrawRequested(window_id) if window.id() == window_id => {
                let mut frame = display.draw();

                frame.clear_color_and_depth((1.0, 1.0, 0.0, 1.0), 1.0);
                state.render(&mut frame, &draw_params);
                frame.finish().unwrap();
            }
            glutin::event::Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => unsafe {
                if MOUSE_GRABBED {
                    state
                        .camera
                        .rotate_by((-delta.1 * 0.1) as f32, (delta.0 * 0.1) as f32, 0.0);
                }
            },
            _ => {
                let next_frame_time =
                    std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
                *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
            }
        }
    })
}
