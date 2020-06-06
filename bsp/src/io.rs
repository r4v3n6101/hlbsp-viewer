use byteorder::{ReadBytesExt, LE};
use std::{
    ffi::CString,
    io::{BufRead, Error as IOError, ErrorKind, Result as IOResult, Seek},
};

const HL_BSP_VERSION: u32 = 30;
const MAX_LUMPS: usize = 15;

#[derive(Clone, Copy, Default)]
struct Lump {
    offset: u32,
    length: u32,
}

fn read_bsp<R: BufRead + Seek>(mut reader: R) -> IOResult<()> {
    let header = reader.read_u32::<LE>()?;
    if header != HL_BSP_VERSION {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            format!("Wrong bsp header {}, expected {}", header, HL_BSP_VERSION),
        ));
    }

    let mut lumps = [Lump::default(); MAX_LUMPS];
    for lump in &mut lumps {
        *lump = Lump {
            offset: reader.read_u32::<LE>()?,
            length: reader.read_u32::<LE>()?,
        }
    }

    todo!();
}
