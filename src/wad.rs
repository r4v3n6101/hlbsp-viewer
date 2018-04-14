use super::*;

const MAX_NAME: usize = 16;

#[repr(C)]
struct Header {
    magic: [u8; 4],
    num_dir: i32,
    dir_offset: i32,
}

#[repr(C)]
pub struct DirEntry {
    pub file_pos: i32,
    pub disk_size: i32,
    pub size: i32,
    pub entry_type: i8,
    pub compression: bool,
    dummy: i16,
    pub name: [u8; MAX_NAME],
}

pub fn entries(buf: &[u8]) -> Vec<DirEntry> {
    use std::mem::size_of;

    let header: Header = read_struct(&buf);
    let off = header.dir_offset as usize;
    let len = header.num_dir as usize * size_of::<DirEntry>();
    return read_mul_structs(&buf[off..off + len]);
}