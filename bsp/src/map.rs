use crate::{
    io::{BspMapReader, LumpType},
    lump::{read_entity, read_unsized},
};
use bincode2::Result as BincodeResult;
use serde::Deserialize;
use std::{
    ffi::CString,
    io::{Error as IOError, Read, Seek},
};

const MAX_MAP_HULLS: usize = 4;

pub type Vec3 = (f32, f32, f32);
type Edge = (u16, u16);
type AABB = ([f32; 3], [f32; 3]);
type Surfedge = i32;
type Marksurface = u16;

#[derive(Debug)]
pub struct Map {
    pub entities: CString,
    pub vertices: Vec<Vec3>,
    pub edges: Vec<Edge>,
    pub faces: Vec<(usize, usize)>, // TODO : temporary
}

impl Map {
    pub fn new<R: Read + Seek>(reader: &BspMapReader<R>) -> BincodeResult<Map> {
        #[derive(Deserialize)]
        struct TexInfo {
            vs: Vec3,
            vs_shift: f32,
            vt: Vec3,
            vt_shift: f32,
            miptex_indes: u32,
            flags: u32,
        }
        #[derive(Clone, Deserialize)]
        struct Face {
            plane_index: u16,
            plane_side: u16,
            first_edge: u16,
            edges_num: u16,
            texinfo_index: u16,
            styles: [u8; 4],
            lightmap_offset: u32,
        }
        #[derive(Deserialize)]
        struct Model {
            bounding_box: AABB,
            origin: Vec3,
            headnodes: [u32; MAX_MAP_HULLS],
            vis_leaves: u32,
            first_face: u32,
            faces_num: u32,
        }

        let entities = reader.read_lump(LumpType::Entities)?;
        let entities = read_entity(entities).map_err(|e| IOError::from(e))?;

        let vertices = reader.read_lump(LumpType::Vertices)?;
        let vertices: Vec<Vec3> = read_unsized(vertices)?;

        let surfedges = reader.read_lump(LumpType::Surfegdes)?;
        let surfedges: Vec<Surfedge> = read_unsized(surfedges)?;

        // TODO: safety checks here: edges[i]
        let edges = reader.read_lump(LumpType::Edges)?;
        let edges: Vec<Edge> = read_unsized(edges)?;
        let edges = surfedges
            .into_iter()
            .map(|i| {
                if i > 0 {
                    edges[i as usize]
                } else {
                    let e = edges[(-i) as usize];
                    (e.1, e.0)
                }
            })
            .collect(); // TODO : this remove unused edges

        let marksurfaces = reader.read_lump(LumpType::Marksurfaces)?;
        let marksurfaces: Vec<Marksurface> = read_unsized(marksurfaces)?;

        let faces = reader.read_lump(LumpType::Faces)?;
        let faces: Vec<Face> = read_unsized(faces)?;
        let faces: Vec<Face> = marksurfaces
            .into_iter()
            .map(|v| v as usize)
            .map(|i| faces[i].clone())
            .collect();

        let faces = faces
            .into_iter()
            .map(|f| (f.first_edge as usize, f.edges_num as usize))
            .collect();

        Ok(Map {
            entities,
            vertices,
            edges,
            faces,
        })
    }
}
