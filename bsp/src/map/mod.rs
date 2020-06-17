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

// TODO : hide inner fields
pub struct Map {
    pub entities: CString,
    pub planes: Vec<Plane>,
    pub textures: Vec<MipTexture>,
    pub vertices: Vec<Vec3>,
    pub visibility: Vec<u8>,
    pub nodes: Vec<Node>,
    pub texinfo: Vec<TexInfo>,
    pub faces: Vec<Face>,
    pub lightmap: Vec<LightColor>,
    pub clipnodes: Vec<Clipnode>,
    pub leaves: Vec<Leaf>,
    pub marksurfaces: Vec<Marksurface>,
    pub edges: Vec<Edge>,
    pub surfedges: Vec<Surfedge>,
    pub models: Vec<Model>,
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

    // TODO : make impls for acquiring vertices/textures/and so on

    pub fn replace_empty_textures<F: Fn(&mut MipTexture)>(&mut self, f: F) {
        self.textures
            .iter_mut()
            .filter(|tex| tex.color_indices.is_none())
            .for_each(|tex| f(tex));
    }
}
