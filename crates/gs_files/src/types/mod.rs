use std::collections::HashMap;

use bevy_asset::Handle;
use bevy_reflect::TypeUuid;
use bevy_render::texture::Image;

#[derive(TypeUuid)]
#[uuid = "fa7ac9dc-e81a-40bd-b1ad-e75003f7cb4a"]
pub struct Wad {
    pub images: HashMap<String, Handle<Image>>,
    // TODO : pub fonts: HashMap<String, Handle<Font>>,
}

#[derive(TypeUuid)]
#[uuid = "750434c7-1c96-430c-8fb2-e2172e4eb8e2"]
pub struct Bsp {
    // TODO : handles
    pub skyname: Option<String>,
    pub wads: Vec<Handle<Wad>>,
    // TOOD : pub textured_meshes: HashMap<String, Vec<Mesh>>,
}
