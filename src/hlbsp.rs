// TODO : If version is 30 then little. otherwise big endianness
use super::*;

const HEADER_LUMPS: usize = 15;
const HL_BSP_VERSION: i32 = 30;

const LUMP_ENTITIES: usize = 0;
const LUMP_PLANES: usize = 1;
const LUMP_TEXTURES: usize = 2;
const LUMP_VERTICES: usize = 3;
const LUMP_VISIBILITY: usize = 4;
const LUMP_NODES: usize = 5;
const LUMP_TEXINFO: usize = 6;
const LUMP_FACES: usize = 7;
const LUMP_LIGHTING: usize = 8;
const LUMP_CLIPNODES: usize = 9;
const LUMP_LEAVES: usize = 10;
const LUMP_MARKSURFACES: usize = 11;
const LUMP_EDGES: usize = 12;
const LUMP_SURFEDGES: usize = 13;
const LUMP_MODELS: usize = 14;

#[repr(C)]
struct Header { version: i32, lumps: [Lump; HEADER_LUMPS] }

#[repr(C)]
struct Lump { offset: i32, length: i32 }

#[repr(C)]
struct Face {
    plane: u16,
    plane_side: u16,
    first_edge: u32,
    edges: u16,
    texinfo: u16,
    styles: [u8; 4],
    lightmap_offset: u32,
}

#[repr(C)]
struct Edge { vertices: [u16; 2] }

#[repr(C)]
struct TexInfo {
    vs: Vec3,
    fs: f32,
    vt: Vec3,
    ft: f32,
    imip: u32,
    nflag: u32,
}

#[repr(C)]
struct TextureHeader { mip_textures: u32 } // TODO : Remove

#[repr(C)]
struct MipTex { name: [u8; 16], width: u32, height: u32, offsets: [u32; 4] } // TODO : Separate

type Vertices = Vec<Vec3>;

fn read_lump<S>(buf: &[u8], header: &Header, lump_index: usize) -> S {
    let lump = &header.lumps[lump_index];
    return read_struct(buf, lump.offset as usize);
}

fn read_lump_array<S>(buf: &[u8], header: &Header, lump_index: usize) -> Vec<S> {
    let lump = &header.lumps[lump_index];
    let off = lump.offset as usize;
    let len = lump.length as usize;

    return read_mul_struct(buf, off, len);
}

#[cfg(test)]
mod tests {
    #[test]
    fn convert_test() {
        // TODO : Rewrite this into separate program
        use super::*;
        use std::fs::File;
        use std::io::*;

        let name = "gasworks.bsp";
        let mut f: File = File::open(name).unwrap();
        let size = f.metadata().unwrap().len() as usize;

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        f.read_to_end(&mut buf).unwrap();

        let mut out = File::create("gasworks.obj").unwrap();

        let header: Header = read_struct(&buf, 0);

        let vertices: Vertices = read_lump_array(&buf, &header, LUMP_VERTICES);
        let faces: Vec<Face> = read_lump_array(&buf, &header, LUMP_FACES);
        let surfedges: Vec<i32> = read_lump_array(&buf, &header, LUMP_SURFEDGES);
        let edges: Vec<Edge> = read_lump_array(&buf, &header, LUMP_EDGES);
        vertices.into_iter().for_each(|vertex| {
            out.write(format!("v {} {} {}\n", vertex.0, vertex.1, vertex.2).as_bytes()).unwrap();
        });
        out.write("\n\n\n".as_bytes()).unwrap();
        faces.into_iter().for_each(|face| {
            let mut face_str = String::new();
            face_str += "f";
            for i in 0..face.edges {
                let surfedge_i: u32 = face.first_edge + (i as u32);
                let surfedge: i32 = surfedges[surfedge_i as usize];
                let vert: u16;
                if surfedge > 0 {
                    vert = edges[surfedge as usize].vertices[0];
                } else {
                    vert = edges[(-surfedge) as usize].vertices[1];
                }
                face_str += &format!(" {}", vert + 1); // OBJ uses 1 as first index instead of 0
            }
            face_str += "\n";
            out.write(face_str.as_bytes()).unwrap();
        });
    }

    #[test]
    fn texture_test() {
        use super::*;
        use std::fs::File;
        use std::io::*;

        let name = "gasworks.bsp";
        let mut f: File = File::open(name).unwrap();
        let size = f.metadata().unwrap().len() as usize;

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        f.read_to_end(&mut buf).unwrap();

        let header: Header = read_struct(&buf, 0);

        ///////////////////////////////////////////////////////////////////////////////////////////
        let lump = &header.lumps[LUMP_TEXTURES];
        let offset = lump.offset as usize;
        let tex_header: TextureHeader = read_struct(&buf, offset);
        let offsets: Vec<i32> = read_mul_struct(&buf, offset + 4, tex_header.mip_textures as usize * 4); // Or sizeof(i32)

        for i in offsets {
            let miptexoffset = offset + i as usize;
            let miptex: MipTex = read_struct(&buf, miptexoffset);
            // TODO : move out into a method
            println!("{}", String::from_utf8_lossy(&miptex.name));
        }
        ///////////////////////////////////////////////////////////////////////////////////////////
    }
}