use bincode2::{deserialize_from, Result as BincodeResult};
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    ffi::{CString, NulError},
    io::Cursor,
    mem::size_of,
};

pub fn read_entity(data: Vec<u8>) -> Result<CString, NulError> {
    let mut data = data; // TODO : excess copy, will be removed
    data.pop();
    CString::new(data)
}

pub type Vec3 = (f32, f32, f32);

pub fn read_unsized<T: DeserializeOwned>(data: Vec<u8>) -> BincodeResult<Vec<T>> {
    let size = data.len() / size_of::<T>();
    let mut cursor = Cursor::new(data);
    let mut out: Vec<T> = vec![]; // TODO : dont allocate memory as we have it, just transmute and changes vec sizes
    for _ in 0..size {
        out.push(deserialize_from(&mut cursor)?);
    }
    Ok(out)
}

pub fn read_miptexs(data: Vec<u8>) -> BincodeResult<()> {
    let mut cursor = Cursor::new(data);
    let offsets: Vec<u32> = deserialize_from(&mut cursor)?;

    Ok(())
}
