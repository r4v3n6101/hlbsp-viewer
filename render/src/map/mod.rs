mod lumps;
mod miptex;

use elapsed::measure_time;
use file::{
    bsp::{LumpType, RawMap},
    wad::Archive,
};
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
    vertex::{VertexBuffer, VertexBufferAny},
    DrawParameters, Program, Rect, Surface,
};
use itertools::Itertools;
use log::debug;
use lumps::*;
use miptex::{MipTexture, MIP_NUM};
use std::{
    collections::{HashMap, HashSet},
    iter::Iterator,
};

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
fn calculate_uvs(vertex: &Vec3, texinfo: &TexInfo) -> [f32; 2] {
    let dot_product = |a: &Vec3, b: &Vec3| a.0 * b.0 + a.1 * b.1 + a.2 * b.2;
    [
        dot_product(vertex, &texinfo.vs) + texinfo.ss,
        dot_product(vertex, &texinfo.vt) + texinfo.st,
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
    vbo: VertexBufferAny,
    textured_ibos: HashMap<String, IndexBufferAny>, // lowercase
    textures: HashMap<String, Texture2d>,           // lowercase
    lightmap: BufferTexture<[u8; 4]>,
    program: Program,
}

impl Map {
    pub fn new<F: ?Sized + Facade>(map: &RawMap, facade: &F) -> Self {
        let vertices = parse_vertices(map.lump_data(LumpType::Vertices)).unwrap();
        let edges = parse_edges(map.lump_data(LumpType::Edges)).unwrap();
        let surfedges = parse_surfedges(map.lump_data(LumpType::Surfegdes)).unwrap();
        let normals = parse_normals_from_planes(map.lump_data(LumpType::Planes)).unwrap();
        let faces = parse_faces(map.lump_data(LumpType::Faces)).unwrap();
        let lightmap = map.lump_data(LumpType::Lighting);
        let texinfos = parse_texinfos(map.lump_data(LumpType::TexInfo)).unwrap();
        let textures = parse_textures(map.lump_data(LumpType::Textures)).unwrap();
        let models = parse_models(map.lump_data(LumpType::Models)).unwrap();

        let root_model = &models[0];

        let origin = root_model.origin;
        let vbo_size = faces
            .iter()
            .skip(root_model.face_id)
            .take(root_model.face_num)
            .map(|f| f.surfedge_num)
            .sum();
        let mut vbo_vertices = Vec::with_capacity(vbo_size);
        let mut loaded_textures = HashMap::new();

        let textured_ibos: HashMap<_, _> = faces
            .iter()
            .skip(root_model.face_id)
            .take(root_model.face_num)
            .filter_map(|f| {
                let texinfo = &texinfos[f.texinfo_id];
                let texture = &textures[texinfo.texture_id];
                let tex_name = texture.name().to_string();

                if TRANSPARENT_TEXTURES
                    .iter()
                    .any(|x| tex_name.eq_ignore_ascii_case(x))
                {
                    return None;
                }

                if !texture.is_empty() && !loaded_textures.contains_key(&tex_name) {
                    let (elapsed, ()) = measure_time(|| {
                        loaded_textures
                            .insert(tex_name.clone(), Self::upload_miptex(facade, texture));
                    });
                    debug!("Load intern miptex `{}` in {}", &tex_name, elapsed);
                }

                let n = &normals[f.plane_id];
                let normal = if f.side {
                    [n.0, n.1, n.2]
                } else {
                    [-n.0, -n.1, -n.2]
                };

                let begin = vbo_vertices.len();
                let lightmap_offset = f.lightmap;
                let mut verts = surfedges
                    .iter()
                    .skip(f.surfedge_id)
                    .take(f.surfedge_num)
                    .map(|&s| {
                        let i = if s < 0 {
                            edges[-s as usize].1
                        } else {
                            edges[s as usize].0
                        } as usize;
                        &vertices[i]
                    })
                    .map(move |v| Vertex {
                        position: [v.0 + origin.0, v.1 + origin.1, v.2 + origin.2],
                        tex_coords: calculate_uvs(&v, texinfo),
                        light_tex_coords: [0.0, 0.0],
                        lightmap_offset: (lightmap_offset / 3) as u32,
                        lightmap_size: [0, 0],
                        normal,
                    })
                    .collect_vec();

                let ([mut min_u, mut min_v], [mut max_u, mut max_v]) =
                    (verts[0].tex_coords, verts[0].tex_coords);

                min_u = min_u.floor();
                min_v = min_v.floor();
                max_u = max_u.floor();
                max_v = max_v.floor();

                for vert in &verts {
                    let [u, v] = vert.tex_coords;
                    min_u = u.floor().min(min_u);
                    max_u = u.floor().max(max_u);

                    min_v = v.floor().min(min_v);
                    max_v = v.floor().max(max_v);
                }

                let lightmap_size = [
                    ((max_u / 16.0).ceil() - (min_u / 16.0).floor() + 1.0) as u32,
                    ((max_v / 16.0).ceil() - (min_v / 16.0).floor() + 1.0) as u32,
                ];

                verts.iter_mut().for_each(|v| {
                    let [s, t] = v.tex_coords;

                    v.lightmap_size = lightmap_size;
                    v.light_tex_coords = [(s - min_u) / 16.0, (t - min_v) / 16.0];
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
                    vertex: include_str!("../../shaders/map/vert.glsl"),
                    fragment: include_str!("../../shaders/map/frag.glsl"),
                },
            )
            .unwrap()
        });
        debug!("Map shader was loaded in {}", elapsed);

        let (elapsed, lightmap) = measure_time(|| {
            let lightmap = lightmap
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

        debug!(
            "Map summary: [Vertices={}, Texture groups={}, Lightmap texels={}]",
            vbo_vertices.len(),
            textured_ibos.len(),
            lightmap.len()
        );

        Self {
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
            MipmapsOption::EmptyMipmapsMax((MIP_NUM - 1) as u32),
            miptex.main_width(),
            miptex.main_height(),
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
            let pixels = miptex.pixels(i as usize).unwrap();
            let image = RawImage2d::from_raw_rgba(pixels, dims);
            miplevel.write(rect, image);
        }
        texture
    }

    pub fn load_from_archive<F: ?Sized + Facade>(&mut self, facade: &F, archive: &Archive) {
        let present: HashSet<_> = self.textures.keys().cloned().collect();
        let required: HashSet<_> = self.textured_ibos.keys().cloned().collect();
        let loaded = required.difference(&present).cloned().filter_map(|name| {
            let (elapsed, tex2d) = measure_time(|| {
                let entry = archive
                    .get_by_name(name.to_ascii_uppercase())
                    .or_else(|| archive.get_by_name(name.to_ascii_lowercase()))?;
                let miptex = MipTexture::parse(entry.data()).ok()?;
                Some(Self::upload_miptex(facade, &miptex))
            });
            debug!("Load extern miptex `{}` in {}", &name, elapsed);

            Some((name, tex2d?))
        });
        self.textures.extend(loaded);
    }

    pub fn render<S: Surface>(
        &self,
        surface: &mut S,
        mvp: [[f32; 4]; 4],
        draw_params: &DrawParameters,
    ) {
        let lightmap = &self.lightmap;
        self.textured_ibos.iter().for_each(|(tex, ibo)| {
            if let Some(colormap) = self.textures.get(tex) {
                let uniforms = uniform! {
                    mvp: mvp,
                    colormap: colormap,
                    lightmap: lightmap,
                };
                surface
                    .draw(&self.vbo, ibo, &self.program, &uniforms, &draw_params)
                    .unwrap();
            }
        });
    }
}
