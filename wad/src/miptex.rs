use crate::name::deserialize_fixed_len_cstring;
use arraylib::{Array, ArrayMap};
use bincode::{deserialize_from, Result as BincodeResult};
use serde::Deserialize;
use std::{
    ffi::CString,
    io::{Cursor, Read, Seek, SeekFrom},
    num::NonZeroU32,
};

const MIP_TEXTURES: usize = 4;

pub struct MipTexture {
    pub name: CString,
    pub width: u32,
    pub height: u32,
    pub color_indices: [Option<Vec<u8>>; MIP_TEXTURES],
    pub color_table: Option<Palette>,
}

pub type Palette = [u8; 256 * 3];

impl MipTexture {
    pub fn new(data: Vec<u8>) -> BincodeResult<MipTexture> {
        #[derive(Deserialize)]
        struct MipTexHeader {
            #[serde(deserialize_with = "deserialize_fixed_len_cstring")]
            name: CString,
            width: u32,
            height: u32,
            offsets: [u32; MIP_TEXTURES],
        }

        let mut cursor = Cursor::new(data);
        let MipTexHeader {
            name,
            width,
            height,
            offsets,
        } = deserialize_from(&mut cursor)?;
        let offsets: [Option<NonZeroU32>; MIP_TEXTURES] = offsets.map(|x| NonZeroU32::new(x));

        // TODO : check that all offsets exist otherwise texture will be unloaded
        let color_table = if let Some(last_offset) = offsets[MIP_TEXTURES - 1] {
            cursor.seek(SeekFrom::Start(last_offset.get() as u64 + 2))?;
            let mut color_table = [0u8; 256 * 3];
            cursor.read_exact(&mut color_table)?;
            Some(color_table)
        } else {
            None
        };

        let mut color_indices = <[_; MIP_TEXTURES]>::from_fn(|_| None);
        for i in 0..MIP_TEXTURES {
            if let Some(offset) = offsets[i] {
                let offset = offset.get();
                cursor.seek(SeekFrom::Start(offset as u64))?;
                let size = width * height / (1 << (2 * i));
                let mut indices = vec![0u8; size as usize];
                cursor.read_exact(&mut indices)?;
                color_indices[i] = Some(indices);
            }
        }

        Ok(MipTexture {
            name,
            width,
            height,
            color_indices,
            color_table,
        })
    }
}
