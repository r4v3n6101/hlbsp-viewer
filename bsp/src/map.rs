use crate::{
    io::{BspMapReader, LumpType},
    lump::{read_entity, read_miptexs, read_unsized},
};
use bincode2::Result as BincodeResult;
use serde::Deserialize;
use std::{
    ffi::CString,
    io::{Error as IOError, Read, Seek},
};
use wad::miptex::MipTexture;

const MAX_MAP_HULLS: usize = 4;

pub type Vec3 = (f32, f32, f32);
type Edge = (u16, u16);
type AABB = ([f32; 3], [f32; 3]);
type Surfedge = i32;
type Marksurface = u16;
type LightColor = Vec3;

#[derive(Deserialize)]
struct TexInfo {
    vs: Vec3,
    vs_shift: f32,
    vt: Vec3,
    vt_shift: f32,
    miptex_indes: u32,
    flags: u32,
}

#[derive(Deserialize)]
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

pub struct Map {
    entities: CString,
    vertices: Vec<Vec3>,
    edges: Vec<Edge>,
    surfedges: Vec<Surfedge>,
    faces: Vec<Face>,
    marksurfaces: Vec<Marksurface>,
    models: Vec<Model>,
    lightmap: Vec<LightColor>,
    textures: Vec<MipTexture>,
}

impl Map {
    pub fn new<R: Read + Seek>(reader: &BspMapReader<R>) -> BincodeResult<Map> {
        let entities = reader.read_lump(LumpType::Entities)?;
        let entities = read_entity(entities).map_err(|e| IOError::from(e))?;

        let vertices = reader.read_lump(LumpType::Vertices)?;
        let vertices: Vec<Vec3> = read_unsized(vertices)?;

        let edges = reader.read_lump(LumpType::Edges)?;
        let edges: Vec<Edge> = read_unsized(edges)?;

        let surfedges = reader.read_lump(LumpType::Surfegdes)?;
        let surfedges: Vec<Surfedge> = read_unsized(surfedges)?;

        let faces = reader.read_lump(LumpType::Faces)?;
        let faces: Vec<Face> = read_unsized(faces)?;

        let marksurfaces = reader.read_lump(LumpType::Marksurfaces)?;
        let marksurfaces: Vec<Marksurface> = read_unsized(marksurfaces)?;

        let models = reader.read_lump(LumpType::Models)?;
        let models = read_unsized(models)?;

        let lightmap = reader.read_lump(LumpType::Lighting)?;
        let lightmap = read_unsized(lightmap)?;

        let textures = reader.read_lump(LumpType::Textures)?;
        let textures = read_miptexs(textures)?;

        Ok(Map {
            entities,
            vertices,
            edges,
            surfedges,
            faces,
            marksurfaces,
            models,
            lightmap,
            textures,
        })
    }

    pub fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    pub fn textures(&self) -> &[MipTexture] {
        &self.textures
    }

    pub fn replace_empty_textures<F: Fn(&mut MipTexture)>(&mut self, f: F) {
        self.textures
            .iter_mut()
            .filter(|tex| tex.color_indices.is_none())
            .for_each(|tex| f(tex));
    }
}
