extern crate hlbsp2obj;

use hlbsp2obj::{bsp::*, read_struct};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, stdin, Write};

fn main() {
    let mut in_name = String::new();
    stdin().read_line(&mut in_name).unwrap();
    in_name = in_name.replace('\n', "");
    let in_f = File::open(in_name).unwrap();
    let size = in_f.metadata().unwrap().len();
    let mut bsp: Vec<u8> = Vec::with_capacity(size as usize);
    let mut reader = BufReader::new(in_f);
    reader.read_to_end(&mut bsp).unwrap();

    let mut out_name = String::new();
    stdin().read_line(&mut out_name).unwrap();
    out_name = out_name.replace('\n', "");
    let out_f = File::create(out_name).unwrap();
    let writer = BufWriter::new(out_f);
    write_obj(writer, &bsp);
}

fn write_obj<W>(mut writer: W, bsp: &[u8])
    where W: Write
{
    let header: Header = read_struct(&bsp);
    let vertices: Vertices = header.lumps[LUMP_VERTICES].read_array(&bsp);
    let faces: Vec<Face> = header.lumps[LUMP_FACES].read_array(&bsp);
    let surfedges: Vec<i32> = header.lumps[LUMP_SURFEDGES].read_array(&bsp);
    let edges: Vec<Edge> = header.lumps[LUMP_EDGES].read_array(&bsp);

    vertices.into_iter().for_each(|vertex| {
        writeln!(writer, "v {} {} {}", vertex.0, vertex.1, vertex.2).unwrap();
    });
    write!(writer, "\n\n\n").unwrap();
    faces.into_iter().for_each(|face| {
        write!(writer, "f").unwrap();
        for i in 0..face.edges {
            let surfedge_i: u32 = face.first_edge + (i as u32);
            let surfedge: i32 = surfedges[surfedge_i as usize];
            let vert: u16;
            if surfedge > 0 {
                vert = edges[surfedge as usize].vertices[0];
            } else {
                vert = edges[-surfedge as usize].vertices[1];
            }
            write!(writer, " {}", vert + 1).unwrap();
        }
        writeln!(writer).unwrap();
    });
    // TODO : Add texture support;
}