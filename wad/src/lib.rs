extern crate byteorder;

pub mod file;
pub mod miptex;

use std::{
    ffi::CString,
    io::{Error as IOError, ErrorKind, Read, Result as IOResult},
};

fn read_cstring<T: Read>(reader: &mut T, max_size: usize) -> IOResult<CString> {
    let mut vec = vec![0; max_size];
    reader.read_exact(&mut vec)?;
    Ok(match vec.iter().position(|&c| c == b'\0') {
        Some(nul_pos) => CString::new(vec[..nul_pos].to_vec()),
        None => CString::new(vec),
    }
    .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn wad_read() {
        let data = std::io::Cursor::new(std::fs::read(env!("WAD_TEST")).unwrap());
        let mut wad_reader = file::WadReader::create(data).unwrap();
        let entries = wad_reader.entries().unwrap();
        assert_eq!(entries.len(), 32);
        for e in &entries {
            println!("{:?}", e.name);
        }
    }

    #[test]
    fn read_wad_miptex() {
        let data = std::io::Cursor::new(std::fs::read(env!("WAD_TEST")).unwrap());
        let mut wad_reader = file::WadReader::create(data).unwrap();
        let entries = wad_reader.entries().unwrap();
        entries.iter().for_each(|data| {
            let data = wad_reader.read_entry(data);
            let miptex = miptex::MipTexture::new(data.unwrap()).unwrap();
            println!("MipTex name: {:?}", miptex.name);
        });
    }
}
