use bincode2::{config, deserialize_from, LengthOption, Result as BincodeResult};
use serde::de::DeserializeOwned;
use std::{
    ffi::{CString, NulError},
    io::Cursor,
    mem::size_of,
};
use wad::miptex::MipTexture;

pub fn read_entity(data: Vec<u8>) -> Result<CString, NulError> {
    let mut data = data; // TODO : excess copy, will be removed
    data.pop();
    CString::new(data)
}

pub fn read_unsized<T: DeserializeOwned>(data: Vec<u8>) -> BincodeResult<Vec<T>> {
    let size = data.len() / size_of::<T>();
    let mut cursor = Cursor::new(data);
    let mut out: Vec<T> = Vec::with_capacity(size);
    for _ in 0..size {
        out.push(deserialize_from(&mut cursor)?);
    }
    Ok(out)
}

pub fn read_miptexs(data: Vec<u8>) -> BincodeResult<Vec<MipTexture>> {
    config()
        .array_length(LengthOption::U32)
        .deserialize::<Vec<u32>>(&data)?
        .into_iter()
        .map(|offset| MipTexture::new(&data[offset as usize..]))
        .collect()
}
