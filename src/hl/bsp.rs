// TODO : If version is 30 then little. otherwise big endianness
use super::*;

const HEADER_LUMPS: usize = 15;
pub const HL_BSP_VERSION: i32 = 30;

pub const LUMP_ENTITIES: usize = 0;
pub const LUMP_PLANES: usize = 1;
pub const LUMP_TEXTURES: usize = 2;
pub const LUMP_VERTICES: usize = 3;
pub const LUMP_VISIBILITY: usize = 4;
pub const LUMP_NODES: usize = 5;
pub const LUMP_TEXINFO: usize = 6;
pub const LUMP_FACES: usize = 7;
pub const LUMP_LIGHTING: usize = 8;
pub const LUMP_CLIPNODES: usize = 9;
pub const LUMP_LEAVES: usize = 10;
pub const LUMP_MARKSURFACES: usize = 11;
pub const LUMP_EDGES: usize = 12;
pub const LUMP_SURFEDGES: usize = 13;
pub const LUMP_MODELS: usize = 14;

#[repr(C)]
pub struct Header { pub version: i32, pub  lumps: [Lump; HEADER_LUMPS] }

#[repr(C)]
pub struct Lump { pub offset: i32, pub length: i32 }

#[repr(C)]
pub struct Face {
    pub plane: u16,
    pub plane_side: u16,
    pub first_edge: u32,
    pub edges: u16,
    pub texinfo: u16,
    pub styles: [u8; 4],
    pub lightmap_offset: u32,
}

#[repr(C)]
pub struct Edge { pub vertices: [u16; 2] }

#[repr(C)]
pub struct TexInfo {
    pub vs: Vec3,
    pub fs: f32,
    pub vt: Vec3,
    pub ft: f32,
    pub imip: u32,
    pub nflag: u32,
}

#[repr(C)]
pub struct TextureHeader { pub mip_textures: u32 } // TODO : Remove

pub type Vertices = Vec<Vec3>;

impl Lump {
    pub fn read_struct<T>(&self, buf: &[u8]) -> T {
        return read_struct(&buf[self.offset as usize..]);
    }

    pub fn read_array<T>(&self, buf: &[u8]) -> Vec<T> {
        let off = self.offset as usize;
        let len = self.length as usize;
        return read_mul_structs(&buf[off..off + len]);
    }
}