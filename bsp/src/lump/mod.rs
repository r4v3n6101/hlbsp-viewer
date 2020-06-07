use byteorder::{ReadBytesExt, LE};
use cgmath::Point3;
use std::{
    ffi::CString,
    io::{Cursor, Result as IOResult},
    mem::size_of,
};

pub fn read_entities(data: Vec<u8>) -> IOResult<CString> {
    // TODO : wrap Vec<u8> directly to CString (waitin for issue solving)
    let mut data = data;
    data.pop();
    Ok(CString::new(data)?)
}

type Point = Point3<f32>;
pub fn read_vertices(data: Vec<u8>) -> IOResult<Vec<Point>> {
    let len = data.len() / size_of::<Point>();
    let mut cursor = Cursor::new(data);
    let mut vertices = Vec::with_capacity(len);
    for _ in 0..len {
        let x = cursor.read_f32::<LE>()?;
        let y = cursor.read_f32::<LE>()?;
        let z = cursor.read_f32::<LE>()?;
        vertices.push(Point::new(x, y, z));
    }
    Ok(vertices)
}

pub struct Face {
    plane: u16,
    plane_side: u16,
    first_edge: u16,
    edges: u16,
    texinfo: u16,
    styles: [u8; 4],
    lightmap_offset: u32,
}

pub fn read_faces(data: Vec<u8>) -> IOResult<Vec<Face>> {
    let len = data.len() / size_of::<Point>();
    let mut cursor = Cursor::new(data);
    let mut faces = Vec::with_capacity(len);
    for _ in 0..len {
        todo!()
    }
    Ok(faces)
}
