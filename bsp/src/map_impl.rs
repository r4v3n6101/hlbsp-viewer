use crate::{
    lumps::{
        parse_edges, parse_faces, parse_models, parse_normals_from_planes, parse_surfedges,
        parse_texinfos, parse_textures, parse_vertices, Face, Model, TexInfo, Vec3,
    },
    LumpType, RawMap,
};
use itertools::Itertools;
use miptex::MipTexture;
use std::iter::{empty, once, Iterator};

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

// TODO : divide by miptex width, height for normalization
fn calculate_uvs(vertex: &Vec3, texinfo: &TexInfo, texture: &MipTexture) -> UV {
    (
        (texinfo.ss + dot_product(vertex, &texinfo.vs)) / (texture.full_width() as f32),
        (texinfo.st + dot_product(vertex, &texinfo.vt)) / (texture.full_height() as f32),
    )
}

// Assuming all vertices are in clockwise
fn triangulated<'a>(
    mut vertices: impl Iterator<Item = usize> + 'a,
) -> Box<dyn Iterator<Item = usize> + 'a> {
    if let Some(base_point) = vertices.next() {
        Box::new(
            vertices
                .tuple_windows::<(_, _)>()
                .flat_map(move |(v1, v2)| once(base_point).chain(once(v1)).chain(once(v2))),
        )
    } else {
        Box::new(empty())
    }
}

/*
 * Map implementation without using bsp-tree data
*/
pub struct IndexedMap<'a> {
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

    pub fn indices(
        &'a self,
        model: &'a Model,
    ) -> impl Iterator<Item = impl Iterator<Item = usize>> + 'a {
        let mut i = 0;
        self.faces(model).map(move |f| {
            let vertices_num = f.surfedge_num;
            let indices = i..i + vertices_num;
            i += vertices_num;
            indices
        })
    }

    pub fn indices_triangulated(&'a self, model: &'a Model) -> Vec<usize> {
        self.indices(model).flat_map(triangulated).collect()
    }

    pub fn replace_empty_textures<F: FnMut(&mut MipTexture)>(&mut self, f: F) {
        self.textures.iter_mut().filter(|t| t.empty()).for_each(f);
    }
}
