use super::*;
use bincode2::{config, deserialize_from, LengthOption, Result as BincodeResult};
use serde::de::DeserializeOwned;
use std::{
    ffi::CString,
    io::{Cursor, Error as IOError},
    mem::size_of,
};

// TODO : switch to std (PR: #73139)
#[inline]
pub(crate) fn read_entities<R: Read + Seek>(reader: &BspMapReader<R>) -> BincodeResult<CString> {
    let mut data = reader.read_lump(LumpType::Entities)?;
    data.pop();
    Ok(CString::new(data).map_err(|e| IOError::from(e))?)
}

pub(crate) fn read_unsized_lump<R: Read + Seek, T: DeserializeOwned>(
    reader: &BspMapReader<R>,
    lump_type: LumpType,
) -> BincodeResult<Vec<T>> {
    let data = reader.read_lump(lump_type)?;
    let size = data.len() / size_of::<T>();
    let mut cursor = Cursor::new(data);
    let mut out: Vec<T> = Vec::with_capacity(size);
    for _ in 0..size {
        out.push(deserialize_from(&mut cursor)?);
    }
    Ok(out)
}

pub(crate) fn read_miptexs<R: Read + Seek>(
    reader: &BspMapReader<R>,
) -> BincodeResult<Vec<MipTexture>> {
    let data = reader.read_lump(LumpType::Textures)?;
    config()
        .array_length(LengthOption::U32)
        .deserialize::<Vec<u32>>(&data)?
        .into_iter()
        .map(|offset| MipTexture::new(&data[offset as usize..]))
        .collect()
}
