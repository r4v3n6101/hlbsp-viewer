use crate::name::deserialize_fixed_len_cstring;
use arraylib::Array;
use bincode::{deserialize_from, Result as BincodeResult};
use serde::Deserialize;
use std::{
    ffi::CString,
    io::{Cursor, Read, Seek, SeekFrom},
};

const MIP_TEXTURES: usize = 4;

pub struct MipTexture {
    pub name: CString,
    pub width: u32,
    pub height: u32,
    pub color_indices: [Vec<u8>; MIP_TEXTURES],
    pub color_table: Palette,
}

pub type Palette = [u8; 256 * 3];

impl MipTexture {
    pub fn new(data: Vec<u8>) -> BincodeResult<MipTexture> {
        #[derive(Debug, Deserialize)]
        struct MipTexHeader {
            #[serde(deserialize_with = "deserialize_fixed_len_cstring")]
            name: CString,
            width: u32,
            height: u32,
            offsets: [u32; 4],
        }
        let mut cursor = Cursor::new(data);
        let MipTexHeader {
            name,
            width,
            height,
            offsets,
        } = deserialize_from(&mut cursor)?;

        let mut color_indices = <[_; MIP_TEXTURES]>::from_fn(|_| vec![]);
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
