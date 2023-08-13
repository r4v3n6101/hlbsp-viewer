use elapsed::measure_time;
use glam::Mat4;
use glium::{
    backend::Facade,
    implement_vertex,
    index::{IndexBuffer, IndexBufferAny, PrimitiveType},
    program,
    texture::{
        buffer_texture::{BufferTexture, BufferTextureType},
        MipmapsOption, RawImage2d, Texture2d,
    },
    uniform,
    uniforms::MinifySamplerFilter,
    vertex::{VertexBuffer, VertexBufferAny},
    DrawParameters, Frame, Program, Rect, Surface,
};
use goldsrc_rs::{
    bsp::{Level, TextureInfo},
    texture::MipTexture,
    wad::{Archive, Content},
    SmolStr,
};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    iter::{Iterator, once},
};
use tracing::{debug, info};

const TRANSPARENT_TEXTURES: [&str; 2] = ["sky", "aaatrigger"];

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    light_tex_coords: [f32; 2],
    lightmap_offset: u32,
    lightmap_size: [u32; 2],
    normal: [f32; 3],
}

implement_vertex!(
    Vertex,
    position,
    tex_coords,
    light_tex_coords,
    lightmap_offset,
    lightmap_size,
    normal
);

#[inline]
fn calculate_uvs(vertex: &[f32; 3], texinfo: &TextureInfo) -> [f32; 2] {
    let dot_product = |a: &[f32; 3], b: &[f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    [
        dot_product(vertex, &texinfo.s) + texinfo.s_shift,
        dot_product(vertex, &texinfo.t) + texinfo.t_shift,
    ]
}

fn triangulate(vertices: Vec<usize>) -> Vec<usize> {
    let n = vertices.len();
    match n {
        0..=2 => vec![],
        3 => vertices,
        _ => vertices[1..]
            .windows(2)
            .flat_map(|i| std::iter::once(vertices[0]).chain(i.iter().copied()))
            .collect(),
    }
}

pub struct Map {
    origin: [f32; 3],
    vbo: VertexBufferAny,
    textured_ibos: HashMap<SmolStr, IndexBufferAny>, // lowercase
    textures: HashMap<SmolStr, Texture2d>,           // lowercase
    lightmap: BufferTexture<[u8; 4]>,
    program: Program,
}

impl Map {
    pub fn new<F: ?Sized + Facade>(facade: &F, bsp: &Level) -> Self {
        let root_model = &bsp.models[0];

        let origin = root_model.origin;

        let vbo_size = bsp
            .faces
            .iter()
            .skip(root_model.first_face_id as usize)
            .take(root_model.faces_num as usize)
            .map(|f| f.surfedges_num as usize)
            .sum();
        let mut vbo_vertices = Vec::with_capacity(vbo_size);
        let mut loaded_textures = HashMap::new();

        let textured_ibos: HashMap<_, _> = bsp
            .faces
            .iter()
            .skip(root_model.first_face_id as usize)
            .take(root_model.faces_num as usize)
            .filter_map(|f| {
                let texinfo = &bsp.texture_infos[f.texture_info_id as usize];
                let texture = &bsp.textures[texinfo.texture_id as usize];
                let tex_name = texture.name.to_owned();

                if TRANSPARENT_TEXTURES
                    .iter()
                    .any(|x| tex_name.eq_ignore_ascii_case(x))
                {
                    return None;
                }

                if texture.data.is_some() && !loaded_textures.contains_key(&tex_name) {
                    let (elapsed, ()) = measure_time(|| {
                        loaded_textures
                            .insert(tex_name.clone(), Self::upload_miptex(facade, texture));
                    });
                    debug!("Load intern miptex `{}` in {}", &tex_name, elapsed);
                }

                let n = &bsp.planes[f.plane_id as usize].normal;
                let normal = if f.plane_side != 0 {
                    *n
                } else {
                    [-n[0], -n[1], -n[2]]
                };

                let begin = vbo_vertices.len();
                let lightmap_offset = f.lightmap_offset;
                let mut verts = bsp
                    .surfedges
                    .iter()
                    .skip(f.first_surfedge_id as usize)
                    .take(f.surfedges_num as usize)
                    .map(|&s| {
                        let i = if s < 0 {
                            bsp.edges[-s as usize].1
                        } else {
                            bsp.edges[s as usize].0
                        } as usize;
                        &bsp.vertices[i]
                    })
                    .map(move |v| Vertex {
                        position: *v,
                        tex_coords: calculate_uvs(&v, texinfo),
                        light_tex_coords: [0.0, 0.0],
                        lightmap_offset: (lightmap_offset / 3) as u32,
                        lightmap_size: [0, 0],
                        normal,
                    })
                    .collect_vec();

                let ([mut min_u, mut min_v], [mut max_u, mut max_v]) =
                    (verts[0].tex_coords, verts[0].tex_coords);

                for vert in &verts {
                    let [u, v] = vert.tex_coords;
                    min_u = u.min(min_u);
                    max_u = u.max(max_u);

                    min_v = v.min(min_v);
                    max_v = v.max(max_v);
                }

                let lightmap_size = [
                    ((max_u / 16.0).ceil() - (min_u / 16.0).floor() + 1.0) as u32,
                    ((max_v / 16.0).ceil() - (min_v / 16.0).floor() + 1.0) as u32,
                ];

                verts.iter_mut().for_each(|v| {
                    let [s, t] = v.tex_coords;

                    v.lightmap_size = lightmap_size;
                    v.light_tex_coords = [
                        (s.ceil() - min_u.floor()) / 16.0,
                        (t.ceil() - min_v.floor()) / 16.0,
                    ];
                });

                vbo_vertices.extend(verts);
                let end = vbo_vertices.len();
                let indices = triangulate((begin..end).collect_vec());

                Some((tex_name, indices))
            })
            .into_group_map()
            .into_iter()
            .map(|(k, v)| {
                let indices = v.into_iter().flatten().map(|x| x as u32).collect_vec();
                debug!("{} triangles using `{}` miptex", indices.len() / 3, &k);
                (
                    k,
                    IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices)
                        .unwrap()
                        .into(),
                )
            })
            .collect();

        let vbo = VertexBuffer::new(facade, &vbo_vertices).unwrap().into();

        let (elapsed, program) = measure_time(|| {
            program!(facade,
                140 => {
                    vertex: include_str!("../shaders/map/vert.glsl"),
                    fragment: include_str!("../shaders/map/frag.glsl"),
                },
            )
            .unwrap()
        });
        debug!("Map shader was loaded in {}", elapsed);

        let (elapsed, lightmap) = measure_time(|| {
            let lightmap = bsp
                .lighting
                .chunks(3)
                .filter_map(|rgb| {
                    if let [r, g, b] = rgb {
                        Some([*r, *g, *b, 255])
                    } else {
                        None
                    }
                })
                .collect_vec();
            BufferTexture::persistent(facade, &lightmap, BufferTextureType::Float).unwrap()
        });
        debug!("Lightmap was loaded in {}", elapsed);

        info!(
            "Map summary: [Vertices={}, Texture groups={}, Lightmap texels={}]",
            vbo_vertices.len(),
            textured_ibos.len(),
            lightmap.len()
        );

        Self {
            origin,
            vbo,
            textured_ibos,
            textures: loaded_textures,
            lightmap,
            program,
        }
    }

    fn upload_miptex<F: ?Sized + Facade>(facade: &F, miptex: &MipTexture) -> Texture2d {
        let texture = Texture2d::empty_with_mipmaps(
            facade,
            MipmapsOption::EmptyMipmapsMax(3 as u32),
            miptex.width,
            miptex.height,
        )
        .unwrap();

        for i in 0..texture.get_mipmap_levels() {
            let miplevel = texture.mipmap(i).unwrap();
            let dims = (miplevel.width(), miplevel.height());
            let rect = Rect {
                left: 0,
                bottom: 0,
                width: dims.0,
                height: dims.1,
            };
            let pixels = miptex_pixels(&miptex, i as usize).unwrap();
            let image = RawImage2d::from_raw_rgba_reversed(&pixels, dims);
            miplevel.write(rect, image);
        }
        texture
    }

    pub fn is_textures_loaded(&self) -> bool {
        self.textured_ibos.len() == self.textures.len()
    }

    pub fn load_from_archive<F: ?Sized + Facade>(&mut self, facade: &F, archive: &Archive) {
        let present: HashSet<_> = self.textures.keys().cloned().collect();
        let required: HashSet<_> = self.textured_ibos.keys().cloned().collect();
        let loaded = required.difference(&present).cloned().filter_map(|name| {
            let (elapsed, tex2d) = measure_time(|| {
                let entry = archive
                    .get(&SmolStr::new_inline(&name.to_ascii_uppercase()))
                    .or_else(|| archive.get(&SmolStr::new_inline(&name.to_ascii_lowercase())))?;
                if let &Content::MipTexture(mip_texture) = &entry {
                    Some(Self::upload_miptex(facade, &mip_texture))
                } else {
                    None
                }
            });
            if tex2d.is_some() {
                debug!("Load extern miptex `{}` in {}", &name, elapsed);
            }

            Some((name, tex2d?))
        });
        self.textures.extend(loaded);
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        projection: Mat4,
        view: Mat4,
        draw_params: &DrawParameters,
    ) {
        let lightmap = &self.lightmap;
        let mvp = projection * view;
        let mvp = mvp.to_cols_array_2d();
        self.textured_ibos.iter().for_each(|(tex, ibo)| {
            if let Some(colormap) = self.textures.get(tex) {
                let uniforms = uniform! {
                    mvp: mvp,
                    origin: self.origin,
                    colormap: colormap.sampled().minify_filter(MinifySamplerFilter::LinearMipmapNearest),
                    lightmap: lightmap,
                };
                frame
                    .draw(&self.vbo, ibo, &self.program, &uniforms, draw_params)
                    .unwrap();
            }
        });
    }
}

fn miptex_pixels(miptex: &MipTexture, mip_level: usize) -> Option<Vec<u8>> {
    let color_table = &miptex.data.as_ref().unwrap().palette;
    Some(
        miptex.data.as_ref().unwrap().indices[mip_level]
            .iter()
            .map(|&i| i as usize)
            .flat_map(|i| {
                let r = color_table[i][0];
                let g = color_table[i][1];
                let b = color_table[i][2];
                let a = if r < 30 && g < 30 && b > 125 { 0 } else { 255 };
                once(r).chain(once(g)).chain(once(b)).chain(once(a))
            })
            .collect(),
    )
}
