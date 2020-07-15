use bsp::{lumps::*, LumpType, Map};
use glium::implement_vertex;
use std::iter::{once, Iterator};

type UV = (f32, f32);

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}

implement_vertex!(Vertex, position, tex_coords, normal);

fn dot_product(a: &Vec3, b: &Vec3) -> f32 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

fn calculate_uvs(vertex: &Vec3, texinfo: &TexInfo) -> UV {
    (
        texinfo.ss + dot_product(vertex, &texinfo.vs),
        texinfo.st + dot_product(vertex, &texinfo.vt),
    )
}

// Assuming all vertices are in clockwise
// TODO : use iterators?
fn triangulate(vertices: Vec<usize>) -> Vec<usize> {
    let n = vertices.len();
    match n {
        0..=2 => vec![],
        3 => vertices,
        _ => {
            let mut out = Vec::with_capacity((n - 2) * 3);
            let base_point = vertices[0];
            for i in 1..n - 1 {
                let p1 = vertices[i];
                let p2 = vertices[i + 1];
                out.push(base_point);
                out.push(p1);
                out.push(p2);
            }
            out
        }
    }
}

pub struct MapRender {
    vertices: Vec<Vec3>,
    edges: Vec<(usize, usize)>,
    surfedges: Vec<i16>,
    normals: Vec<Vec3>,
    texinfos: Vec<TexInfo>,
    faces: Vec<Face>,
    models: Vec<Model>,
}

impl MapRender {
    // TODO : remove unwraps
    pub fn new(map: &Map) -> MapRender {
        MapRender {
            vertices: parse_vertices(map.lump_data(LumpType::Vertices)).unwrap(),
            edges: parse_edges(map.lump_data(LumpType::Edges)).unwrap(),
            surfedges: parse_ledges(map.lump_data(LumpType::Surfegdes)).unwrap(),
            normals: parse_normals_from_planes(map.lump_data(LumpType::Planes)).unwrap(),
            texinfos: parse_texinfos(map.lump_data(LumpType::TexInfo)).unwrap(),
            faces: parse_faces(map.lump_data(LumpType::Faces)).unwrap(),
            models: parse_models(map.lump_data(LumpType::Models)).unwrap(),
        }
    }

    pub fn get_vertices(&self) -> Vec<Vertex> {
        let root_model = &self.models[0];
        self.faces
            .iter()
            .skip(root_model.face_id)
            .take(root_model.face_num)
            .flat_map(|f| self.face_to_vertices(f))
            .collect()
    }

    pub fn get_indices(&self) -> Vec<usize> {
        let root_model = &self.models[0];
        self.faces
            .iter()
            .skip(root_model.face_id)
            .take(root_model.face_num)
            .flat_map(|f| self.face_to_indices(f))
            .collect()
    }

    fn face_to_edges<'a>(&'a self, face: &'a Face) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.surfedges
            .iter()
            .skip(face.surfedge_id)
            .take(face.surfedge_num)
            .map(move |&i| {
                if i >= 0 {
                    self.edges[i as usize]
                } else {
                    let e = self.edges[-i as usize];
                    (e.1, e.0)
                }
            })
    }

    fn face_to_vertices<'a>(&'a self, face: &'a Face) -> impl Iterator<Item = Vertex> + 'a {
        let normal = if face.side {
            let n = &self.normals[face.plane_id];
            (-n.0, -n.1, -n.2)
        } else {
            self.normals[face.plane_id] // TODO : unsafe, may crash
        };

        let texinfo = &self.texinfos[face.texinfo_id];
        self.face_to_edges(face).flat_map(move |(v1, v2)| {
            let (v1, v2) = (self.vertices[v1], self.vertices[v2]);
            let n = normal;
            let (uv1, uv2) = (calculate_uvs(&v1, texinfo), calculate_uvs(&v2, texinfo));
            once(Vertex {
                position: [v1.0, v1.1, v1.2],
                tex_coords: [uv1.0, uv1.1],
                normal: [n.0, n.1, n.2],
            })
            .chain(once(Vertex {
                position: [v2.0, v2.1, v2.2],
                tex_coords: [uv2.0, uv2.1],
                normal: [n.0, n.1, n.2],
            }))
        })
    }

    fn face_to_indices<'a>(&'a self, face: &'a Face) -> impl Iterator<Item = usize> + 'a {
        triangulate(
            self.face_to_edges(face)
                .flat_map(|(v1, v2)| once(v1).chain(once(v2)))
                .collect(),
        )
        .into_iter()
    }
}
