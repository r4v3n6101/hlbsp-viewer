use std::{
    ffi::CString,
    io::{Read, Result as IOResult},
};

pub fn read_cstring<T: Read>(reader: &mut T, max_size: usize) -> IOResult<CString> {
    let mut vec = vec![0; max_size];
    reader.read_exact(&mut vec)?;
    Ok(match vec.iter().position(|&c| c == b'\0') {
        Some(nul_pos) => CString::new(vec[..nul_pos].to_vec()),
        None => CString::new(vec),
    }?)
}

#[cfg(test)]
mod tests {

    use super::read_cstring;
    use std::{ffi::CString, io::Cursor};

    fn bytes_to_cursor(bytes: Vec<u8>) -> Cursor<Vec<u8>> {
        Cursor::new(bytes)
    }

    #[test]
    fn read_empty_cstr() {
        assert_eq!(
            read_cstring(&mut bytes_to_cursor(vec![0u8]), 1).unwrap(),
            CString::new("").unwrap()
        );
    }

    #[test]
    fn read_without_nul_cstr() {
        assert_eq!(
            read_cstring(&mut bytes_to_cursor(vec![0x41, 0x56, 0u8]), 2).unwrap(),
            CString::new("AV").unwrap()
        );
    }

    #[test]
    fn read_double_nul_cstr() {
        assert_eq!(
            read_cstring(&mut bytes_to_cursor(vec![0x41, 0x56, 0u8, 0u8, 0x52]), 5).unwrap(),
            CString::new("AV").unwrap()
        );
    }
}
