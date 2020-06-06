use byteorder::{ReadBytesExt, LE};
use std::io::{Error as IOError, ErrorKind, Read, Result as IOResult, Seek, SeekFrom};

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

#[derive(Clone, Copy, Default)]
struct Lump {
    offset: u32,
    length: u32,
}

pub struct BspMapReader<R: Read + Seek> {
    reader: R,
    lumps: [Lump; MAX_LUMPS],
}

impl<R: Read + Seek> BspMapReader<R> {
    pub fn create(mut reader: R) -> IOResult<BspMapReader<R>> {
        let header = reader.read_u32::<LE>()?;
        if header != HL_BSP_VERSION {
            return Err(IOError::new(
                ErrorKind::InvalidData,
                format!("Wrong bsp header {}, expected {}", header, HL_BSP_VERSION),
            ));
        }

        let mut lumps = [Lump::default(); MAX_LUMPS];
        for lump in &mut lumps {
            *lump = Lump {
                offset: reader.read_u32::<LE>()?,
                length: reader.read_u32::<LE>()?,
            }
        }

        Ok(BspMapReader { reader, lumps })
    }

    pub fn read_lump(&mut self, index: LumpType) -> IOResult<Vec<u8>> {
        let lump = &self.lumps[index as usize];
        self.reader.seek(SeekFrom::Start(lump.offset as u64))?;
        let mut data = vec![0u8; lump.length as usize];
        self.reader.read_exact(&mut data)?;
        Ok(data)
    }
}
