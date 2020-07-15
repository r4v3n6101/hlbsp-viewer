use glium::{
    glutin::{
        dpi::LogicalSize,
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

fn start_window_loop(vertices: &[render::map::Vertex], indices: &[u32]) {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("hlbsp viewer")
        .with_inner_size(LogicalSize::new(1024.0, 768.0));
    let cb = ContextBuilder::new();

    let display = Display::new(wb, cb, &event_loop).unwrap();
    let vbo = VertexBuffer::new(&display, vertices).unwrap();
    let ibo = IndexBuffer::new(&display, PrimitiveType::TrianglesList, indices).unwrap();
    let program = program!(&display,
         140 => {
             vertex: "
                #version 140

                uniform mat4 matrix;

                in vec3 position;

                out vec3 vColor;

                void main() {
                    gl_Position = vec4(position, 1.0) * matrix;
                    vColor = vec3(1,0,1);
                }
            ",

             fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;

                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
         },
    )
    .unwrap();

    draw(&display, &vbo, &ibo, &program);

    event_loop.run(|event, _, control_flow| match event {
        Event::WindowEvent {
            window_id: _,
            event: wevent,
        } => *control_flow = process_window(wevent),
        _ => *control_flow = ControlFlow::Poll,
    });
}

fn draw(
    display: &Display,
    vbo: &VertexBuffer<render::map::Vertex>,
    ibo: &IndexBuffer<u32>,
    program: &Program,
) {
    let uniforms = uniform! {
        matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
    };
    let mut target = display.draw();
    target.clear_color(1.0, 1.0, 0.0, 1.0);
    target
        .draw(vbo, ibo, program, &uniforms, &Default::default())
        .unwrap();
    target.finish().unwrap();
}

fn process_window(wevent: WindowEvent) -> ControlFlow {
    match wevent {
        WindowEvent::Resized(_) => ControlFlow::Poll,
        WindowEvent::CloseRequested => ControlFlow::Exit,
        _ => ControlFlow::Poll,
    }
}
