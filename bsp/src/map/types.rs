use serde::Deserialize;

const MAX_MAP_HULLS: usize = 4;

pub type Vec3 = (f32, f32, f32);
pub(crate) type Edge = (u16, u16);
pub(crate) type AABB<T> = [T; 6];
pub(crate) type Surfedge = i32;
pub(crate) type Marksurface = u16;
pub(crate) type LightColor = Vec3;

#[derive(Deserialize)]
pub(crate) struct Plane {
    pub(crate) normal: Vec3,
    pub(crate) distance: f32,
    pub(crate) ptype: i32,
}

#[derive(Deserialize)]
pub(crate) struct TexInfo {
    pub(crate) vs: Vec3,
    pub(crate) vs_shift: f32,
    pub(crate) vt: Vec3,
    pub(crate) vt_shift: f32,
    pub(crate) miptex_indes: u32,
    pub(crate) flags: u32,
}

#[derive(Deserialize)]
pub(crate) struct Face {
    pub(crate) plane_index: u16,
    pub(crate) plane_side: u16,
    pub(crate) first_edge: u16,
    pub(crate) edges_num: u16,
    pub(crate) texinfo_index: u16,
    pub(crate) styles: [u8; 4],
    pub(crate) lightmap_offset: u32,
}

#[derive(Deserialize)]
pub(crate) struct Model {
    pub(crate) bounding_box: AABB<f32>,
    pub(crate) origin: Vec3,
    pub(crate) headnodes: [u32; MAX_MAP_HULLS],
    pub(crate) vis_leaves: u32,
    pub(crate) first_face: u32,
    pub(crate) faces_num: u32,
}

#[derive(Deserialize)]
pub(crate) struct Clipnode {
    pub(crate) plane: i32,
    pub(crate) children: [i16; 2],
}

#[derive(Deserialize)]
pub(crate) struct Leaf {
    pub(crate) contents: i32,
    pub(crate) vis_offset: i32,
    pub(crate) bounding_box: AABB<i16>,
    pub(crate) first_marksurface: u16,
    pub(crate) marksurfaces_num: u16,
    pub(crate) ambient_levels: [u8; 4],
}

#[derive(Deserialize)]
pub(crate) struct Node {
    pub(crate) plane_index: u32,
    pub(crate) children: [i16; 2],
    pub(crate) bounding_box: AABB<i16>,
    pub(crate) first_face: u16,
    pub(crate) faces_num: u16,
}
