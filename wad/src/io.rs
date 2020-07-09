const WAD3_MAGIC: [u8; 4] = [b'W', b'A', b'D', b'3'];

use crate::name::deserialize_fixed_len_cstring;
use bincode2::{deserialize_from, ErrorKind, Result as BincodeResult};
use serde::Deserialize;
use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    io::{Read, Result as IOResult, Seek, SeekFrom},
};

#[derive(Debug, Deserialize)]
pub struct WadEntry {
    file_pos: u32,
    disk_size: u32,
    size: u32,
    entry_type: u8,
    compression: bool,
    _dummy: u16,
    #[serde(deserialize_with = "deserialize_fixed_len_cstring")]
    name: CString,
}

impl WadEntry {
    pub fn name(&self) -> &CStr {
        self.name.as_c_str()
    }

    pub const fn size(&self) -> u32 {
        self.size
    }

    pub const fn entry_type(&self) -> u8 {
        self.entry_type
    }
}

pub struct WadReader<R: Read + Seek> {
    reader: RefCell<R>,
    entries: Vec<WadEntry>,
}

impl<R: Read + Seek> WadReader<R> {
    pub fn create(mut reader: R) -> BincodeResult<WadReader<R>> {
        #[derive(Deserialize)]
        struct Header([u8; 4], u32, u32);
        let Header(magic, count, offset) = deserialize_from(&mut reader).unwrap();
        if let WAD3_MAGIC = magic {
            reader.seek(SeekFrom::Start(u64::from(offset)))?;
            let entries: BincodeResult<Vec<WadEntry>> =
                (0..count).map(|_| deserialize_from(&mut reader)).collect();
            Ok(WadReader {
                reader: RefCell::new(reader),
                entries: entries?,
            })
        } else {
            let msg = format!(
                "Wrong WAD magic: found `{:?}`, expected `{:?}`",
                magic, WAD3_MAGIC
            );
            Err(ErrorKind::Custom(msg).into())
        }
    }

    pub fn entries(&self) -> &[WadEntry] {
        self.entries.as_slice()
    }

    pub fn find_entry(&self, name: &CStr) -> Option<&WadEntry> {
        // TODO : inefficient due to O(N) search, consider using HashSet instead of Vec
        self.entries().iter().find(|&entry| entry.name() == name)
    }

    pub fn read_entry(&self, entry: &WadEntry) -> IOResult<Vec<u8>> {
        self.reader
            .borrow_mut()
            .seek(SeekFrom::Start(u64::from(entry.file_pos)))?;
        let mut data = vec![0; entry.size as usize];
        self.reader.borrow_mut().read_exact(&mut data)?;
        Ok(data)
    }
}
