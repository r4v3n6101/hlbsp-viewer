use crate::{
    io::{BspMapReader, LumpType},
    lump::{read_entity, read_unsized},
};
use bincode2::Result as BincodeResult;
use std::{
    ffi::CString,
    io::{Error as IOError, Read, Seek},
};

pub type Vec3 = (f32, f32, f32);

#[derive(Debug)]
pub struct Map {
    pub entities: CString,
    pub vertices: Vec<Vec3>,
}

impl Map {
    pub fn new<R: Read + Seek>(reader: &BspMapReader<R>) -> BincodeResult<Map> {
        let entities = reader.read_lump(LumpType::Entities)?;
        let entities = read_entity(entities).map_err(|e| IOError::from(e))?;
        let vertices = reader.read_lump(LumpType::Vertices)?;
        let vertices: Vec<Vec3> = read_unsized(vertices)?;
        Ok(Map { entities, vertices })
    }
}
