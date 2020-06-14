use crate::io::{BspMapReader, LumpType};
use bincode2::Result as BincodeResult;
use std::{
    ffi::CString,
    io::{Read, Seek},
};
use wad::miptex::MipTexture;

mod lumps;
use lumps::*;
mod types;
use types::*;

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
