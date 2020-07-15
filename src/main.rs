use glium::{
    glutin::{
        dpi::{LogicalSize, PhysicalSize},
        event::{Event, WindowEvent},
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
    about = "A programm allows you to view hlbsp maps (bsp v30)"
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
    start_window_loop(&vertices, &indices);
}

// TODO : indices are actually u16
fn start_window_loop(vertices: &[render::map::Vertex], indices: &[u32]) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(LogicalSize::new(1024.0, 768.0));
    let cb = ContextBuilder::new();

    let mut camera = render::camera::Camera::new();
    let display = Display::new(wb, cb, &event_loop).unwrap();
    let vbo = VertexBuffer::new(&display, vertices).unwrap();
    let ibo = IndexBuffer::new(&display, PrimitiveType::TrianglesList, indices).unwrap();
    let program = program!(&display,
         140 => {
             vertex: "
                #version 140

                uniform mat4 proj;
                uniform mat4 view;

                in vec3 position;

                void main() {
                    gl_Position = proj * view * vec4(position, 1.0);
                }
            ",

             fragment: "
                #version 140
                out vec4 f_color;

                void main() {
                    f_color = vec4(1, 0, 0, 1.0);
                }
            "
         },
    )
    .unwrap();

    draw(&display, &vbo, &ibo, &program, &camera);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id: _,
            event: wevent,
        } => *control_flow = process_window(wevent, &mut camera),
        _ => *control_flow = ControlFlow::Poll,
    });
}

fn draw(
    display: &Display,
    vbo: &VertexBuffer<render::map::Vertex>,
    ibo: &IndexBuffer<u32>,
    program: &Program,
    camera: &render::camera::Camera,
) {
    let persp: [[_; 4]; 4] = camera.perspective().into();
    let view: [[_; 4]; 4] = camera.view().into();
    let uniforms = uniform! {
        proj: persp,
        view: view,
    };
    let mut target = display.draw();
    target.clear_color(1.0, 1.0, 0.0, 1.0);
    target
        .draw(vbo, ibo, program, &uniforms, &Default::default())
        .unwrap();
    target.finish().unwrap();
}

fn process_window(wevent: WindowEvent, camera: &mut render::camera::Camera) -> ControlFlow {
    match wevent {
        WindowEvent::Resized(PhysicalSize { width, height }) => {
            camera.aspect_ratio = (width as f32) / (height as f32);
            ControlFlow::Poll
        }
        WindowEvent::CloseRequested => ControlFlow::Exit,
        _ => ControlFlow::Poll,
    }
}
