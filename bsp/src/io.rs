use bincode2::{deserialize_from, ErrorKind, Result as BincodeResult};
use serde::Deserialize;
use std::{
    cell::RefCell,
    io::{Read, Result as IOResult, Seek, SeekFrom},
};

const HL_BSP_VERSION: u32 = 30;
const MAX_LUMPS: usize = 15;

pub enum LumpType {
    Entities,
    Planes,
    Textures,
    Vertices,
    Visibility,
    Nodes,
    TexInfo,
    Faces,
    Lighting,
    Clipnodes,
    Leaves,
    Marksurfaces,
    Edges,
    Surfegdes,
    Models,
}

#[derive(Deserialize)]
struct Lump {
    offset: u32,
    length: u32,
}

pub struct BspMapReader<R: Read + Seek> {
    reader: RefCell<R>,
    lumps: [Lump; MAX_LUMPS],
}

impl<R: Read + Seek> BspMapReader<R> {
    pub fn create(mut reader: R) -> BincodeResult<BspMapReader<R>> {
        let (version, lumps) = deserialize_from(&mut reader)?;
        if let HL_BSP_VERSION = version {
            Ok(BspMapReader {
                reader: RefCell::new(reader),
                lumps,
            })
        } else {
            let msg = format!(
                "Wrong HL BSP version: found {}, expected {}",
                version, HL_BSP_VERSION
            );
            Err(ErrorKind::Custom(msg).into())
        }
    }

    pub fn read_lump(&self, index: LumpType) -> IOResult<Vec<u8>> {
        let lump = &self.lumps[index as usize];
        self.reader
            .borrow_mut()
            .seek(SeekFrom::Start(u64::from(lump.offset)))?;
        let mut data = vec![0_u8; lump.length as usize];
        self.reader.borrow_mut().read_exact(&mut data)?;
        Ok(data)
    }
}
