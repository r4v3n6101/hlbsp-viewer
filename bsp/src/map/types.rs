use serde::Deserialize;

const MAX_MAP_HULLS: usize = 4;

pub type Vec3 = (f32, f32, f32);
pub type UV = (f32, f32);
pub type Edge = (u16, u16);
pub type AABB<T> = [T; 6];
pub type Surfedge = i32;
pub type Marksurface = u16;
pub type LightColor = Vec3;

#[derive(Deserialize)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
    pub ptype: i32,
}

#[derive(Deserialize)]
pub struct TexInfo {
    pub vs: Vec3,
    pub vs_shift: f32,
    pub vt: Vec3,
    pub vt_shift: f32,
    pub miptex_index: u32,
    pub animated: u32,
}

#[derive(Deserialize)]
pub struct Face {
    pub plane_index: u16,
    pub plane_side: u16,
    pub first_edge: u16,
    pub edges_num: u16,
    pub texinfo_index: u16,
    pub styles: [u8; 4],
    pub lightmap_offset: i32,
}

#[derive(Deserialize)]
pub struct Model {
    pub bounding_box: AABB<f32>,
    pub origin: Vec3,
    pub headnodes: [u32; MAX_MAP_HULLS],
    pub vis_leaves: u32,
    pub first_face: u32,
    pub faces_num: u32,
}

#[derive(Deserialize)]
pub struct Clipnode {
    pub plane: i32,
    pub children: [i16; 2],
}

#[derive(Deserialize)]
pub struct Leaf {
    pub contents: i32,
    pub vis_offset: i32,
    pub bounding_box: AABB<i16>,
    pub first_marksurface: u16,
    pub marksurfaces_num: u16,
    pub ambient_levels: [u8; 4],
}

#[derive(Deserialize)]
pub struct Node {
    pub plane_index: u32,
    pub children: [i16; 2],
    pub bounding_box: AABB<i16>,
    pub first_face: u16,
    pub faces_num: u16,
}

pub fn dot_product(a: &Vec3, b: &Vec3) -> f32 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}
