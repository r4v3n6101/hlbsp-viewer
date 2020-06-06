use byteorder::{ReadBytesExt, LE};
use cstr::read_cstring;
use std::{
    ffi::CString,
    io::{Cursor, Read, Result as IOResult, Seek, SeekFrom},
};

const MIP_TEXTURES: usize = 4;
const MAX_NAME: usize = 16;

pub struct MipTexture {
    pub name: CString,
    pub width: u32,
    pub height: u32,
    pub color_indices: [Vec<u8>; MIP_TEXTURES],
    pub color_table: [u8; 256 * 3],
}

impl MipTexture {
    pub fn new(data: Vec<u8>) -> IOResult<MipTexture> {
        let mut cursor = Cursor::new(data);
        let name: CString = read_cstring(&mut cursor, MAX_NAME)?;
        let width = cursor.read_u32::<LE>()?;
        let height = cursor.read_u32::<LE>()?;

        let mut offsets = [0u32; MIP_TEXTURES];
        for offset in &mut offsets {
            *offset = cursor.read_u32::<LE>()?;
        }

        let mut color_indices = [vec![], vec![], vec![], vec![]]; // TODO : simplify
        for i in 0..MIP_TEXTURES {
            let offset = offsets[i];
            cursor.seek(SeekFrom::Start(offset as u64))?;
            let size = width * height / (1 << (2 * i));
            let mut indices = vec![0u8; size as usize];
            cursor.read_exact(&mut indices)?;
            color_indices[i] = indices;
        }

        cursor.seek(SeekFrom::Current(2))?; // skip 2 dummy bytes after last mipdata
        let mut color_table = [0u8; 256 * 3];
        cursor.read_exact(&mut color_table)?;

        Ok(MipTexture {
            name,
            width,
            height,
            color_indices,
            color_table,
        })
    }
}
