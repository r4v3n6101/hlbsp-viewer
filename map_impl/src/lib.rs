use crate::lumps::{
    parse_edges, parse_entities_str, parse_faces, parse_models, parse_normals_from_planes,
    parse_surfedges, parse_texinfos, parse_textures, parse_vertices, Face, Model, TexInfo, Vec3,
};
use bsp::{LumpType, RawMap};
use miptex::MipTexture;
use std::iter::{once, Iterator};
use wad::Archive;

mod lumps;
pub mod miptex;

pub type UV = (f32, f32);

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

fn dot_product(a: &Vec3, b: &Vec3) -> f32 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

fn calculate_uvs(vertex: &Vec3, texinfo: &TexInfo, texture: &MipTexture) -> UV {
    (
        (dot_product(vertex, &texinfo.vs) /*+ texinfo.ss*/) / (texture.full_width() as f32),
        (dot_product(vertex, &texinfo.vt) /*+ texinfo.st*/) / (texture.full_height() as f32),
    )
}

// Assuming all vertices are in clockwise
fn triangulated(vertices: Vec<usize>) -> Vec<usize> {
    let n = vertices.len();
    match n {
        0..=2 => vec![],
        3 => vertices,
        _ => {
            let base_point = vertices[0];
            vertices[1..]
                .windows(2)
                .flat_map(|i| once(base_point).chain(i.iter().copied()))
                .collect()
        }
    }
}

/*
 * Map implementation without bsp-tree data
*/
pub struct IndexedMap<'a> {
    entities: &'a str,
    vertices: Vec<Vec3>,
    edges: Vec<(usize, usize)>,
    surfedges: Vec<i32>,
    normals: Vec<Vec3>,
    texinfos: Vec<TexInfo>,
    faces: Vec<Face>,
    textures: Vec<MipTexture<'a>>,
    models: Vec<Model>,
}

impl<'a> IndexedMap<'a> {
    // TODO : remove unwraps
    pub fn new(map: &'a RawMap) -> Self {
        Self {
            entities: parse_entities_str(map.lump_data(LumpType::Entities)).unwrap(),
            vertices: parse_vertices(map.lump_data(LumpType::Vertices)).unwrap(),
            edges: parse_edges(map.lump_data(LumpType::Edges)).unwrap(),
            surfedges: parse_surfedges(map.lump_data(LumpType::Surfegdes)).unwrap(),
            normals: parse_normals_from_planes(map.lump_data(LumpType::Planes)).unwrap(),
            texinfos: parse_texinfos(map.lump_data(LumpType::TexInfo)).unwrap(),
            faces: parse_faces(map.lump_data(LumpType::Faces)).unwrap(),
            textures: parse_textures(map.lump_data(LumpType::Textures)).unwrap(),
            models: parse_models(map.lump_data(LumpType::Models)).unwrap(),
        }
    }

    pub const fn entities(&self) -> &str {
        self.entities
    }

    pub fn textures(&self) -> &[MipTexture] {
        &self.textures
    }

    pub fn root_model(&self) -> &Model {
        &self.models[0]
    }

    fn faces(&'a self, model: &'a Model) -> impl Iterator<Item = &Face> + 'a {
        self.faces.iter().skip(model.face_id).take(model.face_num)
    }

    fn face_to_vertices(&'a self, face: &'a Face) -> impl Iterator<Item = Vec3> + 'a {
        self.surfedges
            .iter()
            .skip(face.surfedge_id)
            .take(face.surfedge_num)
            .map(move |&i| {
                let v = if i >= 0 {
                    self.edges[i as usize].0
                } else {
                    self.edges[-i as usize].1
                };
                self.vertices[v]
            })
    }

    pub fn calculate_vertices(&'a self, model: &'a Model) -> Vec<Vertex> {
        self.faces(model)
            .flat_map(|f| {
                let vertices = self.face_to_vertices(f);
                let normal = if f.side {
                    self.normals[f.plane_id]
                } else {
                    let n = self.normals[f.plane_id];
                    (-n.0, -n.1, -n.2)
                };

                let texinfo = &self.texinfos[f.texinfo_id];
                let texture = &self.textures[texinfo.texture_id];
                vertices.map(move |v| {
                    let uv = calculate_uvs(&v, texinfo, texture);
                    Vertex {
                        position: [v.0, v.1, v.2],
                        tex_coords: [uv.0, uv.1],
                        normal: [normal.0, normal.1, normal.2],
                    }
                })
            })
            .collect()
    }

    // TODO : comment why not iterators
    pub fn indices_with_texture(&'a self, model: &'a Model) -> Vec<(&'a MipTexture, Vec<usize>)> {
        let mut i = 0;
        self.faces(model)
            .map(move |f| {
                let vertices_num = f.surfedge_num;
                let indices = i..i + vertices_num;
                i += vertices_num;

                let texinfo = &self.texinfos[f.texinfo_id];
                let texture = &self.textures[texinfo.texture_id];
                (texture, indices.collect())
            })
            .collect()
    }

    pub fn indices_triangulated(&'a self, model: &'a Model) -> Vec<(&'a MipTexture, Vec<usize>)> {
        self.indices_with_texture(model)
            .into_iter()
            .map(|(t, indices)| (t, triangulated(indices)))
            .collect()
    }

    pub fn empty_textures(&self) -> Vec<&str> {
        self.textures
            .iter()
            .filter_map(|tex| if tex.empty() { Some(tex.name()) } else { None })
            .collect()
    }

    pub fn replace_empty_textures(&mut self, wad: &'a Archive<'a>) {
        self.textures
            .iter_mut()
            .filter(|t| t.empty())
            .for_each(|miptex| {
                let name = miptex.name().to_uppercase();
                if let Some(entry) = wad.get_by_name(&name) {
                    *miptex = MipTexture::parse(entry.data()).unwrap(); // TODO : same
                }
            });
    }
}
