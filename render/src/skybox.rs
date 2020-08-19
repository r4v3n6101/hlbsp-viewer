use glium::{
    backend::Facade,
    framebuffer::SimpleFrameBuffer,
    implement_vertex,
    index::{IndexBuffer, IndexBufferAny, PrimitiveType},
    program,
    texture::{CubeLayer, Cubemap, RawImage2d, Texture2d},
    uniform,
    uniforms::MagnifySamplerFilter,
    vertex::{VertexBuffer, VertexBufferAny},
    BlitTarget, Depth, DepthTest, DrawParameters, Program, Surface,
};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);

const CUBE_VERTICES: [Vertex; 24] = [
    Vertex {
        position: [-1.0, -1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
    },
];

const CUBE_INDICES: [u16; 36] = [
    0, 2, 1, 0, 3, 2, 4, 6, 5, 4, 7, 6, 8, 10, 9, 8, 11, 10, 12, 14, 13, 12, 15, 14, 16, 18, 17,
    16, 19, 18, 20, 22, 21, 20, 23, 22,
];

const CUBEMAP_SIDES: [CubeLayer; 6] = [
    CubeLayer::PositiveX,
    CubeLayer::NegativeX,
    CubeLayer::PositiveY,
    CubeLayer::NegativeY,
    CubeLayer::PositiveZ,
    CubeLayer::NegativeZ,
];

pub struct Skybox {
    vbo: VertexBufferAny,
    ibo: IndexBufferAny,
    cubemap: Cubemap,
    program: Program,
}

impl Skybox {
    pub fn new<F: ?Sized + Facade>(facade: &F, dimension: u32, textures: [Vec<u8>; 6]) -> Self {
        let vbo = VertexBuffer::new(facade, &CUBE_VERTICES).unwrap();
        let ibo = IndexBuffer::new(facade, PrimitiveType::TrianglesList, &CUBE_INDICES).unwrap();
        let program = program!(facade,
            140 => {
                vertex: include_str!("../shaders/skybox/vert.glsl"),
                fragment: include_str!("../shaders/skybox/frag.glsl"),
            }
        )
        .unwrap();

        let cubemap = Cubemap::empty(facade, dimension).unwrap();
        let blit_rect = BlitTarget {
            left: 0,
            bottom: 0,
            width: dimension as i32,
            height: dimension as i32,
        };

        for side in &CUBEMAP_SIDES {
            let i = side.get_layer_index();
            let image = RawImage2d::from_raw_rgba(textures[i].clone(), (dimension, dimension)); // TODO : clone
            let texture = Texture2d::new(facade, image).unwrap();
            let target = SimpleFrameBuffer::new(facade, cubemap.main_level().image(*side)).unwrap();
            texture.as_surface().blit_whole_color_to(
                &target,
                &blit_rect,
                MagnifySamplerFilter::Linear,
            );
        }
        Self {
            vbo: vbo.into(),
            ibo: ibo.into(),
            program,
            cubemap,
        }
    }

    pub fn render<S: Surface>(&self, surface: &mut S, mvp: [[f32; 4]; 4]) {
        let uniforms = uniform! {
            mvp: mvp,
            cubetex: self.cubemap.sampled().magnify_filter(MagnifySamplerFilter::Linear),
        };
        let draw_params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Depth::default()
            },
            ..DrawParameters::default()
        };

        surface
            .draw(&self.vbo, &self.ibo, &self.program, &uniforms, &draw_params)
            .unwrap();
    }
}
