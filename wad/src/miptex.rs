use crate::name::deserialize_fixed_len_cstring;
use arraylib::Array;
use bincode2::{deserialize, Result as BincodeResult};
use serde::Deserialize;
use std::ffi::CString;

const MIP_TEXTURES: usize = 4;
const PALETTE_SIZE: usize = 256 * 3;

pub struct MipTexture {
    pub name: CString,
    pub width: u32,
    pub height: u32,
    pub color_indices: Option<[Vec<u8>; MIP_TEXTURES]>,
    pub color_table: Option<Vec<u8>>,
}

impl MipTexture {
    pub fn new(data: &[u8]) -> BincodeResult<MipTexture> {
        #[derive(Deserialize)]
        struct MipTexHeader {
            #[serde(deserialize_with = "deserialize_fixed_len_cstring")]
            name: CString,
            width: u32,
            height: u32,
            offsets: [u32; MIP_TEXTURES],
        }
        let MipTexHeader {
            name,
            width,
            height,
            offsets,
        } = deserialize(data)?;

        let tex_presense = offsets.iter().all(|&v| v != 0);
        let color_indices = if tex_presense {
            Some(<[_; MIP_TEXTURES]>::from_fn(|i| {
                let offset = offsets[i] as usize;
                let size = (width * height / (1 << (2 * i))) as usize;
                let color_indices = &data[offset..offset + size];
                color_indices.to_vec()
            }))
        } else {
            None
        };

        let color_table = if tex_presense {
            let col_table_begin = (offsets[MIP_TEXTURES - 1] + 2) as usize;
            let col_table = &data[col_table_begin..col_table_begin + 256 * 3]; // TODO : add notice that data must be validated
            let mut out = Vec::with_capacity(PALETTE_SIZE);
            out.extend_from_slice(col_table);
            Some(out)
        } else {
            None
        };

        Ok(MipTexture {
            name,
            width,
            height,
            color_indices,
            color_table,
        })
    }
}
