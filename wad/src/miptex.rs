use crate::read_cstring;
use byteorder::{ReadBytesExt, LE};
use std::{
    ffi::CString,
    io::{Cursor, Result as IOResult, Seek, SeekFrom},
};

const MIP_TEXTURES: usize = 4;
const MAX_NAME: usize = 16;

pub struct MipTexture {
    pub name: CString,
    pub width: u32,
    pub height: u32,
    pub textures: [Vec<u8>; MIP_TEXTURES],
}

impl MipTexture {
    pub fn new(data: Vec<u8>) -> IOResult<MipTexture> {
        let mut cursor = Cursor::new(data.as_slice());
        let name: CString = read_cstring(&mut cursor, MAX_NAME)?;
        let width = cursor.read_u32::<LE>()?;
        let height = cursor.read_u32::<LE>()?;
        let mut offsets = [0u32; MIP_TEXTURES];
        for i in 0..MIP_TEXTURES {
            offsets[i] = cursor.read_u32::<LE>()?;
        }

        cursor.seek(SeekFrom::Start(0))?;
        Ok(MipTexture {
            name,
            width,
            height,
            textures: [vec![], vec![], vec![], vec![]],
        })
    }
}
