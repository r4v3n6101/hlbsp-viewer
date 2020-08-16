mod lumps;
mod miptex;

use bsp::{LumpType, RawMap};
use elapsed::measure_time;
use glium::{
    backend::Facade,
    implement_vertex,
    index::PrimitiveType,
    texture::{MipmapsOption, RawImage2d},
    IndexBuffer, Rect, Texture2d, VertexBuffer,
};
use itertools::Itertools;
use log::{debug, info};
use lumps::*;
use miptex::{MipTexture, MIP_NUM};
use std::{
    collections::{HashMap, HashSet},
    iter::Iterator,
};
use wad::Archive;

#[derive(Copy, Clone)]
pub struct GlVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}

implement_vertex!(GlVertex, position, tex_coords, normal);

pub struct MapRender {
    vbo: VertexBuffer<GlVertex>,
    textured_ibos: HashMap<String, IndexBuffer<u32>>, // lowercase
    textures: HashMap<String, Texture2d>,             // lowercase
}

#[inline]
fn calculate_uvs(vertex: &Vec3, texinfo: &TexInfo, texture: &MipTexture) -> [f32; 2] {
    let dot_product = |a: &Vec3, b: &Vec3| a.0 * b.0 + a.1 * b.1 + a.2 * b.2;
    [
        (dot_product(vertex, &texinfo.vs) + texinfo.ss) / (texture.main_width() as f32),
        (dot_product(vertex, &texinfo.vt) + texinfo.st) / (texture.main_height() as f32),
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

impl MapRender {
    pub fn new<F: ?Sized + Facade>(map: &RawMap, facade: &F) -> Self {
        let (elapsed, out) = measure_time(|| {
            let vertices = parse_vertices(map.lump_data(LumpType::Vertices)).unwrap();
            let edges = parse_edges(map.lump_data(LumpType::Edges)).unwrap();
            let surfedges = parse_surfedges(map.lump_data(LumpType::Surfegdes)).unwrap();
            let normals = parse_normals_from_planes(map.lump_data(LumpType::Planes)).unwrap();
            let faces = parse_faces(map.lump_data(LumpType::Faces)).unwrap();
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
                .map(|f| {
                    let n = &normals[f.plane_id];
                    let normal = if f.side {
                        [n.0, n.1, n.2]
                    } else {
                        [-n.0, -n.1, -n.2]
                    };

                    let texinfo = &texinfos[f.texinfo_id];
                    let texture = &textures[texinfo.texture_id];

                    let tex_name = texture.name().to_string();

                    if !texture.is_empty() && !loaded_textures.contains_key(&tex_name) {
                        loaded_textures
                            .insert(tex_name.clone(), Self::upload_miptex(facade, texture));
                        debug!("Load intern miptex: {}", &tex_name);
                    }

                    let begin = vbo_vertices.len();
                    let v = surfedges
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
                        .map(move |v| GlVertex {
                            position: [v.0 + origin.0, v.1 + origin.1, v.2 + origin.2],
                            tex_coords: calculate_uvs(&v, texinfo, texture),
                            normal,
                        })
                        .collect_vec();
                    vbo_vertices.extend(v);
                    let end = vbo_vertices.len();
                    let indices = triangulate((begin..end).collect_vec());

                    (tex_name, indices)
                })
                .into_group_map()
                .into_iter()
                .map(|(k, v)| {
                    let indices = v.into_iter().flatten().map(|x| x as u32).collect_vec();
                    debug!("{} triangles using `{}` miptex", indices.len() / 3, &k);
                    (
                        k,
                        IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices).unwrap(),
                    )
                })
                .collect();

            info!("Textured render groups: {}", textured_ibos.len());

            let vbo = VertexBuffer::new(facade, &vbo_vertices).unwrap();
            info!("Vertices: {}", vbo_vertices.len());

            Self {
                vbo,
                textured_ibos,
                textures: loaded_textures,
            }
        });

        info!("Map loading done in {}", elapsed);
        out
    }

    pub const fn vbo(&self) -> &VertexBuffer<GlVertex> {
        &self.vbo
    }

    pub fn textured_ibos(&self) -> impl Iterator<Item = (&Texture2d, &IndexBuffer<u32>)> {
        self.textured_ibos
            .iter()
            .filter_map(move |(key, value)| Some((self.textures.get(key)?, value)))
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
            let image = RawImage2d::from_raw_rgb(pixels, dims);
            miplevel.write(rect, image);
        }
        texture
    }

    pub fn load_from_archive<F: ?Sized + Facade>(&mut self, facade: &F, archive: &Archive) {
        let present: HashSet<_> = self.textures.keys().cloned().collect();
        let required: HashSet<_> = self.textured_ibos.keys().cloned().collect();
        let loaded = required.difference(&present).cloned().filter_map(|name| {
            let entry = archive.get_by_name(name.to_ascii_uppercase())?;
            let miptex = MipTexture::parse(entry.data()).ok()?;
            let tex2d = Self::upload_miptex(facade, &miptex);
            debug!("Load extern miptex: {}", &name);

            Some((name, tex2d))
        });
        self.textures.extend(loaded);
    }
}
