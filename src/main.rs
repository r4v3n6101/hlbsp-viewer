use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin};
use clap::Parser;
use gs_files::{Bsp, GsFilesPlugin, Wad};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "valve")]
    game: String,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            asset_folder: args.game,
            ..default()
        }))
        .add_plugin(GsFilesPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(load_map)
        .add_system(display_wads)
        .run();
}

fn load_map(asset_server: Res<AssetServer>, maps: Res<Assets<Bsp>>) {
    asset_server.load("maps/boot_camp.bsp").make_strong(&maps);
}

fn display_wads(
    wads: Res<Assets<Wad>>,
    asset_server: Res<AssetServer>,
    mut egui_ctx: ResMut<EguiContext>,
    mut cached_content: Local<HashMap<String, HashMap<String, egui::TextureId>>>,
) {
    for (handle_id, wad) in wads.iter() {
        let Some(asset_path) = asset_server.get_handle_path(handle_id) else {
            error!("Unknown path for wad");
            continue;
        };

        let wad_name = asset_path.path().to_string_lossy();
        if !cached_content.contains_key(wad_name.as_ref()) {
            cached_content.insert(
                wad_name.to_string(),
                wad.images
                    .iter()
                    .map(|(name, handle)| (name.to_string(), egui_ctx.add_image(handle.clone())))
                    .collect(),
            );
        }
    }
    egui::Window::new("Wad debug")
        .scroll2([true, true])
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            cached_content.iter().for_each(|(wad_name, images)| {
                ui.collapsing(wad_name, |ui| {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::LEFT).with_main_wrap(true),
                        |ui| {
                            images.iter().for_each(|(name, img_id)| {
                                ui.add(egui::Image::new(*img_id, [64.0, 64.0]))
                                    .on_hover_text(name);
                            });
                        },
                    );
                });
            });
        });
}
