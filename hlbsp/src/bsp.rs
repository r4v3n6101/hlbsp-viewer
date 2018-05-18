use {read_mul_structs, read_struct};
pub use texture::MipTex;
pub use Vec3;

const HEADER_LUMPS: usize = 15;
pub const HL_BSP_VERSION: u32 = 30;

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
pub struct Header { pub version: u32, pub lumps: [Lump; HEADER_LUMPS] }

#[repr(C)]
pub struct Lump { pub offset: u32, pub length: u32 }

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

#[repr(C)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
    pub plane_type: u32,
}

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
pub struct TexInfo {
    pub vs: Vec3,
    pub fs: f32,
    pub vt: Vec3,
    pub ft: f32,
    pub imip: u32,
    pub nflag: u32,
}

pub type Vertex = Vec3;
pub type Edge = (u16, u16);
pub type Surfedge = i32;
pub type BoundingBox = (Vec3, Vec3);
pub type Marksurface = u16;

#[repr(C)]
pub struct Model {
    pub bb: BoundingBox,
    pub origin: Vec3,
    pub nodes: [u32; 4],
    pub numleaves: u32,
    pub first_face: u32,
    pub faces: u32,
}

/// General struct representing bsp file
pub struct BspMap {
    pub entities: String,
    pub planes: Vec<Plane>,
    pub miptexs: Vec<MipTex>,
    pub vertices: Vec<Vertex>,
    // vislist,
    // nodes,
    pub texinfos: Vec<TexInfo>,
    pub faces: Vec<Face>,
    // lightmaps,
    // clipnodes,
    // leaves,
    pub marksurfaces: Vec<Marksurface>,
    pub edges: Vec<Edge>,
    pub surfedges: Vec<Surfedge>,
    pub models: Vec<Model>,
}

impl BspMap {
    pub fn new(buf: &[u8]) -> Result<BspMap, &str> {
        let header: Header = read_struct(buf);
        if header.version != HL_BSP_VERSION {
            return Err("Mismatched bsp file's version");
        }

        let lumps = header.lumps;
        let ent_bytes: Vec<u8> = lumps[LUMP_ENTITIES].read_array(buf);
        let entities = String::from_utf8(ent_bytes).map_err(|_| "Error while parsing entities")?;
        let vertices: Vec<Vertex> = lumps[LUMP_VERTICES].read_array(buf);
        let faces: Vec<Face> = lumps[LUMP_FACES].read_array(buf);
        let planes: Vec<Plane> = lumps[LUMP_PLANES].read_array(buf);
        let surfedges: Vec<Surfedge> = lumps[LUMP_SURFEDGES].read_array(buf);
        let marksurfaces: Vec<Marksurface> = lumps[LUMP_MARKSURFACES].read_array(buf);
        let models: Vec<Model> = lumps[LUMP_MODELS].read_array(buf);
        let edges: Vec<Edge> = lumps[LUMP_EDGES].read_array(buf);
        let texinfos: Vec<TexInfo> = lumps[LUMP_TEXINFO].read_array(buf);

        let texs_off: usize = lumps[LUMP_TEXTURES].offset as usize;
        let tex = &buf[texs_off..];
        let mip_num = read_struct::<u32>(tex) as usize;

        let mut miptexs: Vec<MipTex> = Vec::with_capacity(mip_num);
        for i in 0..mip_num {
            let miptex_off: u32 = read_struct(&tex[4 + i * 4..]);
            let miptex: MipTex = read_struct(&tex[miptex_off as usize..]);
            miptexs.push(miptex);
        }

        Ok(BspMap {
            entities,
            planes,
            miptexs,
            vertices,
            texinfos,
            faces,
            marksurfaces,
            edges,
            surfedges,
            models,
        })
    }
}