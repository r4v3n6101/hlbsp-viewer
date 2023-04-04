use bevy_app::{App, Plugin};
use bevy_asset::AddAsset;

pub use types::{Bsp, Wad};

mod loaders;
mod types;

pub struct GsFilesPlugin;

impl Plugin for GsFilesPlugin {
    fn build(&self, app: &mut App) {
        // TODO : needed for client only, yep?
        app.add_asset::<Wad>().add_asset_loader(loaders::WadLoader);
        app.add_asset::<Bsp>().add_asset_loader(loaders::BspLoader);
    }
}
