mod entities;
mod map;
mod skybox;

use glam::Mat4;
use glium::{Display, DrawParameters, Frame};
use std::{fs, io::Cursor};
use tracing::{debug, error, info};

use crate::{cubemap::read_cubemap, Args};
use anyhow::Result;
use {
    entities::{find_info_player_start, get_skyname, get_start_point, Vec3},
    map::Map,
    skybox::Skybox,
};

pub struct Level {
    start_point: Option<Vec3>,
    map_render: Map,
    skybox: Option<Skybox>,
}

impl Level {
    pub fn new(display: &Display, args: &Args) -> Result<Self> {
        let bsp_file = fs::read(&args.bsp_path)?;
        let reader = Cursor::new(bsp_file);
        let bsp = goldsrc_rs::bsp(reader)?;
        let mut map_render = Map::new(display, &bsp);

        if !map_render.is_textures_loaded() {
            let file = fs::read(&args.wad_path)?;
            let reader = Cursor::new(file);
            let archive = goldsrc_rs::wad(reader)?;
            if let Some(file_name) = &args.wad_path.file_name() {
                debug!("Scanning {:?} for textures", file_name);
            }
            map_render.load_from_archive(display, &archive);
        }

        let info_player_start = find_info_player_start(&bsp.entities);
        let start_point = info_player_start.and_then(get_start_point);
        let skybox = get_skyname(&bsp.entities).and_then(|skyname| {
            args.skybox_path.clone().and_then(|skybox_path| {
                if let Ok(cubemap) = read_cubemap(&skyname, skybox_path) {
                    info!("Skybox loaded: {}", skyname);
                    Some(Skybox::new(display, &cubemap))
                } else {
                    error!("Error loading skybox: {}", skyname);
                    None
                }
            })
        });

        Ok(Self {
            start_point,
            map_render,
            skybox,
        })
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
        self.map_render.render(frame, projection, view, draw_params);
    }
}
