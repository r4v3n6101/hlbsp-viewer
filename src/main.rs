mod camera;

use glium::{
    glutin::{
        dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
        event::{Event, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    index::{IndexBuffer, PrimitiveType},
    program,
    program::Program,
    uniform,
    vertex::VertexBuffer,
    Display, Surface,
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hlbsp_viewer",
    about = "A program allows you to view hlbsp maps (bsp v30)"
)]
struct Opt {
    #[structopt(short, long = "bsp", parse(from_os_str))]
    bsp_path: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let file = std::fs::read(&opt.bsp_path).unwrap();
    let map_render = render::map::MapRender::new(&bsp::Map::parse(&file).unwrap());
    let vertices = map_render.get_vertices();
    let indices = map_render
        .get_indices()
        .into_iter()
        .map(|x| x as u32)
        .collect::<Vec<u32>>(); // TODO : temporary workaround, convert inside `render` module
    start_window_loop(map_render.root_model().origin, &vertices, &indices);
}

// TODO : indices are actually u16
fn start_window_loop(origin: (f32, f32, f32), vertices: &[render::map::Vertex], indices: &[u32]) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(LogicalSize::new(1024.0, 768.0));
    let cb = ContextBuilder::new();

    let mut camera = camera::Camera::new();
    let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
    let display = Display::new(wb, cb, &event_loop).unwrap();
    let vbo = VertexBuffer::new(&display, vertices).unwrap();
    let ibo = IndexBuffer::new(&display, PrimitiveType::TrianglesList, indices).unwrap();
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
                out vec4 f_color;

                void main() {
                    vec2 tt = fract(o_tex_coords);
                    if(tt.x > 0.5) {
                        f_color = vec4(1, 0, 0, 1.0);
                    }else{
                        f_color = vec4(0, 1, 0, 1.0);
                    }
                }
            "
         },
    )
    .unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id: _,
            event: wevent,
        } => *control_flow = process_window(wevent, &mut mouse_pos, &mut camera),
        _ => {
            let next_frame_time =
                std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            draw(&display, &vbo, &ibo, &program, origin, &camera);
        }
    });
}

fn draw(
    display: &Display,
    vbo: &VertexBuffer<render::map::Vertex>,
    ibo: &IndexBuffer<u32>,
    program: &Program,
    origin: (f32, f32, f32),
    camera: &camera::Camera,
) {
    let persp: [[_; 4]; 4] = camera.perspective().into();
    let view: [[_; 4]; 4] = camera.view().into();
    let uniforms = uniform! {
        proj: persp,
        view: view,
        origin: [origin.0, origin.1, origin.2],
    };

    let draw_parameters = glium::DrawParameters {
        backface_culling: glium::BackfaceCullingMode::CullCounterClockwise,
        depth: glium::Depth {
            test: glium::DepthTest::IfMoreOrEqual,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut target = display.draw();
    target.clear_color(1.0, 1.0, 0.0, 1.0);
    target
        .draw(vbo, ibo, program, &uniforms, &draw_parameters)
        .unwrap();
    target.finish().unwrap();
}

fn process_window(
    wevent: WindowEvent,
    mouse_pos: &mut PhysicalPosition<f64>,
    camera: &mut camera::Camera,
) -> ControlFlow {
    match wevent {
        WindowEvent::KeyboardInput { input, .. } => {
            let mut exit = false;
            if let Some(virt_keycode) = input.virtual_keycode {
                match virt_keycode {
                    VirtualKeyCode::W => camera.position.z += 0.01,
                    VirtualKeyCode::A => camera.position.x -= 0.01,
                    VirtualKeyCode::S => camera.position.z -= 0.01,
                    VirtualKeyCode::D => camera.position.x += 0.01,
                    VirtualKeyCode::Up => camera.position.y += 0.01,
                    VirtualKeyCode::Down => camera.position.y -= 0.01,
                    VirtualKeyCode::Left => camera.rotate_by(0.0, -1.0, 0.0),
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
        WindowEvent::CursorMoved { position, .. } => {
            let dx = (position.x - mouse_pos.x) as f32;
            let dy = (position.y - mouse_pos.y) as f32;
            *mouse_pos = position;

            // TODO : camera.rotate_by(dx * 0.1, -dy * 0.1, 0.0);
            ControlFlow::Poll
        }
        WindowEvent::Resized(PhysicalSize { width, height }) => {
            camera.aspect_ratio = (width as f32) / (height as f32);
            ControlFlow::Poll
        }
        WindowEvent::CloseRequested => ControlFlow::Exit,
        _ => ControlFlow::Poll,
    }
}
