use cgmath::prelude;
use std::{
    ffi::{CStr, CString},
    io::{Error as IOError, ErrorKind, Read, Result as IOResult},
};

pub trait LumpReader {
    type Output;

    fn read(data: Vec<u8>) -> IOResult<Self::Output>;
}

pub struct Entity;

impl LumpReader for Entity {
    type Output = CString;

    fn read(data: Vec<u8>) -> IOResult<Self::Output> {
        CStr::from_bytes_with_nul(&data)
            .map_err(|e| IOError::new(ErrorKind::InvalidData, e))
            .map(|v| v.to_owned())
    }
}
