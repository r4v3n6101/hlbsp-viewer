use std::{collections::HashMap, path::PathBuf, str::FromStr};

use bevy::{asset::LoadState, prelude::*};
use bevy_egui::{
    egui::{load::SizedTexture, Align, Image as EImage, Layout, TextureId, Window},
    EguiContexts, EguiPlugin,
};
use clap::Parser;
use gs_files::{GsFilesPlugin, WadSourcePlugin};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "valve")]
    game: String,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins((
            WadSourcePlugin {
                root_path: PathBuf::from_str("./valve").unwrap(),
            },
            DefaultPlugins.set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                ..Default::default()
            }),
            GsFilesPlugin,
            EguiPlugin,
        ))
        .add_systems(Startup, load_wad)
        .add_systems(Update, display_wads)
        .run();
}

fn load_wad(asset_server: Res<AssetServer>, images: Res<Assets<Image>>) {}

fn display_wads(
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut egui_ctx: EguiContexts,
    mut cached_content: Local<HashMap<String, TextureId>>,
) {
    if !cached_content.contains_key("sky") {
        cached_content.insert(
            "sky".to_string(),
            egui_ctx.add_image(asset_server.load::<Image>("wad://+0c1a4_swtch3.mip")),
        );
    }
    Window::new("Wad debug")
        .scroll2([true, true])
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.with_layout(
                Layout::left_to_right(Align::LEFT).with_main_wrap(true),
                |ui| {
                    cached_content.iter().for_each(|(name, img_id)| {
                        ui.add(EImage::new(SizedTexture::new(*img_id, [64.0, 64.0])))
                            .on_hover_text(name);
                    });
                },
            );
        });
}
