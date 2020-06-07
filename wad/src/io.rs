const MAX_NAME: usize = 16;

use byteorder::{ReadBytesExt, LE};
use cstr::read_cstring;
use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    io::{Error as IOError, ErrorKind, Read, Result as IOResult, Seek, SeekFrom},
    slice::Iter,
};

pub struct WadEntry {
    pub name: CString,
    pub entry_type: u8,
    pub size: u32,
    file_pos: u32,
    disk_size: u32,
    compression: bool,
}

impl WadEntry {
    fn read<T: Read + Seek>(reader: &mut T) -> IOResult<WadEntry> {
        let file_pos = reader.read_u32::<LE>()?;
        let disk_size = reader.read_u32::<LE>()?;
        let size = reader.read_u32::<LE>()?;
        let entry_type = reader.read_u8()?;
        let compression = reader.read_u8()? != 0;
        let _dummy = reader.read_u16::<LE>()?;
        let name = read_cstring(reader, MAX_NAME)?;

        Ok(WadEntry {
            name,
            entry_type,
            size,
            file_pos,
            disk_size,
            compression,
        })
    }
}

pub struct WadReader<R: Read + Seek> {
    reader: RefCell<R>,
    entries: Vec<WadEntry>,
}

impl<R: Read + Seek> WadReader<R> {
    pub fn create(mut reader: R) -> IOResult<WadReader<R>> {
        reader.seek(SeekFrom::Start(0))?;
        let mut header = [0u8; 4];
        reader.read_exact(&mut header)?;
        if header == [b'W', b'A', b'D', b'3'] {
            let count = reader.read_u32::<LE>()?;
            let offset = reader.read_u32::<LE>()?;
            reader.seek(SeekFrom::Start(offset as u64))?;
            let entries: IOResult<Vec<WadEntry>> =
                (0..count).map(|_| WadEntry::read(&mut reader)).collect();
            Ok(WadReader {
                reader: RefCell::new(reader),
                entries: entries?,
            })
        } else {
            Err(IOError::new(
                ErrorKind::InvalidData,
                format!("Wrong header: {:?}", header), // TODO : remove debug
            ))
        }
    }

    pub fn entries(&self) -> Iter<WadEntry> {
        self.entries.iter()
    }

    pub fn find_entry(&self, name: &CStr) -> Option<&WadEntry> {
        self.entries().find(|&entry| entry.name.as_c_str() == name)
    }

    pub fn read_entry(&self, entry: &WadEntry) -> IOResult<Vec<u8>> {
        self.reader
            .borrow_mut()
            .seek(SeekFrom::Start(entry.file_pos as u64))?;
        let mut data = vec![0; entry.size as usize];
        self.reader.borrow_mut().read_exact(&mut data)?;
        Ok(data)
    }
}
