use crate::{
    entities::{find_info_player_start, get_skyname, get_start_point, parse_entities, Vec3},
    map::Map,
    skybox::Skybox,
};
use cgmath::Matrix4;
use file::{
    bsp::{LumpType, RawMap},
    cubemap::Cubemap,
    wad::Archive,
};
use glium::{backend::Facade, DrawParameters, Surface};
use log::{debug, info};
use std::{fs::read as read_file, path::Path};

pub struct Level {
    start_point: Option<Vec3>,
    map_render: Map,
    skybox: Option<Skybox>,
}

impl Level {
    pub fn new<F: ?Sized + Facade, P: AsRef<Path>>(
        facade: &F,
        bsp_path: P,
        wad_paths: &[P],
        skybox_path: Option<P>,
    ) -> Self {
        // TODO : remove unwraps
        let bsp_file = read_file(bsp_path).unwrap();
        let raw_map = RawMap::parse(&bsp_file).unwrap();
        let mut map_render = Map::new(facade, &raw_map);

        wad_paths.iter().for_each(|path| {
            let file = read_file(path).unwrap();
            let archive = Archive::parse(&file).unwrap();
            if let Some(file_name) = path.as_ref().file_name() {
                debug!("Scanning {:?} for textures", file_name);
            }
            map_render.load_from_archive(facade, &archive);
        });

        let entities = parse_entities(raw_map.lump_data(LumpType::Entities)).unwrap();
        let info_player_start = find_info_player_start(&entities);
        let start_point = info_player_start.and_then(get_start_point);
        let skybox = get_skyname(&entities).and_then(|skyname| {
            info!("Map's skyname: {}", skyname);
            skybox_path.map(|skybox_path| {
                let cubemap = Cubemap::read(&skyname, skybox_path).unwrap();
                Skybox::new(facade, &cubemap)
            })
        });

        Self {
            start_point,
            map_render,
            skybox,
        }
    }

    pub const fn start_point(&self) -> Option<Vec3> {
        self.start_point
    }

    pub fn render<S: Surface>(
        &self,
        surface: &mut S,
        projection: Matrix4<f32>,
        view: Matrix4<f32>,
        draw_params: &DrawParameters,
    ) {
        if let Some(skybox) = &self.skybox {
            skybox.render(surface, projection, view, draw_params);
        }
        self.map_render
            .render(surface, projection, view, draw_params);
    }
}
