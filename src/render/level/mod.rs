mod entities;
mod map;
mod skybox;

use file::{
    bsp::{LumpType, RawMap},
    cubemap::Cubemap,
    wad::Archive,
};
use glam::Mat4;
use glium::{backend::Facade, DrawParameters, Frame};
use tracing::{debug, error, info};
use std::{fs::read as read_file, path::Path};
use {
    entities::{find_info_player_start, get_skyname, get_start_point, parse_entities, Vec3},
    map::Map,
    skybox::Skybox,
};

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

        for path in wad_paths {
            if map_render.is_textures_loaded() {
                break;
            }
            let file = read_file(path).unwrap();
            let archive = Archive::parse(&file).unwrap();
            if let Some(file_name) = path.as_ref().file_name() {
                debug!("Scanning {:?} for textures", file_name);
            }
            map_render.load_from_archive(facade, &archive);
        }

        let entities = parse_entities(raw_map.lump_data(LumpType::Entities)).unwrap();
        let info_player_start = find_info_player_start(&entities);
        let start_point = info_player_start.and_then(get_start_point);
        let skybox = get_skyname(&entities).and_then(|skyname| {
            skybox_path.and_then(|skybox_path| {
                if let Ok(cubemap) = Cubemap::read(&skyname, skybox_path) {
                    info!("Skybox loaded: {}", skyname);
                    Some(Skybox::new(facade, &cubemap))
                } else {
                    error!("Error loading skybox: {}", skyname);
                    None
                }
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

    pub fn render(
        &self,
        frame: &mut Frame,
        projection: Mat4,
        view: Mat4,
        draw_params: &DrawParameters,
    ) {
        if let Some(skybox) = &self.skybox {
            skybox.render(frame, projection, view, draw_params);
        }
        self.map_render
            .render(frame, projection, view, draw_params);
    }
}
