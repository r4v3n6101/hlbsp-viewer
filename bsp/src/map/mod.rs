use crate::io::{BspMapReader, LumpType};
use bincode2::Result as BincodeResult;
use serde::Deserialize;
use std::{
    ffi::CString,
    io::{Read, Seek},
};
use wad::miptex::MipTexture;

mod internal;
use internal::*;

const MAX_MAP_HULLS: usize = 4;

pub type Vec3 = (f32, f32, f32);
type Edge = (u16, u16);
type AABB<T> = [T; 6];
type Surfedge = i32;
type Marksurface = u16;
type LightColor = Vec3;

#[derive(Deserialize)]
struct Plane {
    normal: Vec3,
    distance: f32,
    ptype: i32,
}

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
    bounding_box: AABB<f32>,
    origin: Vec3,
    headnodes: [u32; MAX_MAP_HULLS],
    vis_leaves: u32,
    first_face: u32,
    faces_num: u32,
}

#[derive(Deserialize)]
struct Clipnode {
    plane: i32,
    children: [i16; 2],
}

#[derive(Deserialize)]
struct Leaf {
    contents: i32,
    vis_offset: i32,
    bounding_box: AABB<i16>,
    first_marksurface: u16,
    marksurfaces_num: u16,
    ambient_levels: [u8; 4],
}

#[derive(Deserialize)]
struct Node {
    plane_index: u32,
    children: [i16; 2],
    bounding_box: AABB<i16>,
    first_face: u16,
    faces_num: u16,
}

pub struct Map {
    entities: CString,
    planes: Vec<Plane>,
    textures: Vec<MipTexture>,
    vertices: Vec<Vec3>,
    visibility: Vec<u8>,
    nodes: Vec<Node>,
    texinfo: Vec<TexInfo>,
    faces: Vec<Face>,
    lightmap: Vec<LightColor>,
    clipnodes: Vec<Clipnode>,
    leaves: Vec<Leaf>,
    marksurfaces: Vec<Marksurface>,
    edges: Vec<Edge>,
    surfedges: Vec<Surfedge>,
    models: Vec<Model>,
}

impl Map {
    pub fn new<R: Read + Seek>(reader: &BspMapReader<R>) -> BincodeResult<Map> {
        Ok(Map {
            entities: read_entities(reader)?,
            planes: read_unsized_lump(reader, LumpType::Planes)?,
            textures: read_miptexs(reader)?,
            vertices: read_unsized_lump(reader, LumpType::Vertices)?,
            visibility: read_unsized_lump(reader, LumpType::Visibility)?,
            nodes: read_unsized_lump(reader, LumpType::Nodes)?,
            texinfo: read_unsized_lump(reader, LumpType::TexInfo)?,
            faces: read_unsized_lump(reader, LumpType::Faces)?,
            lightmap: read_unsized_lump(reader, LumpType::Lighting)?,
            clipnodes: read_unsized_lump(reader, LumpType::Clipnodes)?,
            leaves: read_unsized_lump(reader, LumpType::Leaves)?,
            marksurfaces: read_unsized_lump(reader, LumpType::Marksurfaces)?,
            edges: read_unsized_lump(reader, LumpType::Edges)?,
            surfedges: read_unsized_lump(reader, LumpType::Surfegdes)?,
            models: read_unsized_lump(reader, LumpType::Models)?,
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
