use std::path::PathBuf;

use bevy_app::{App, Plugin};
use bevy_asset::{
    io::{AssetSource, AssetSourceId},
    AssetApp,
};
use io::wad::WadAssetReader;
use loaders::wad::{FontLoader, MipTexLoader, PicLoader};

mod io;
mod loaders;

pub struct WadSourcePlugin {
    pub root_path: PathBuf,
}

impl Plugin for WadSourcePlugin {
    fn build(&self, app: &mut App) {
        let root_path = self.root_path.clone();
        app.register_asset_source(
            AssetSourceId::new(Some("wad")),
            AssetSource::build().with_reader(move || Box::new(WadAssetReader::new(&root_path))),
        );
    }
}

pub struct GsFilesPlugin;

impl Plugin for GsFilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<PicLoader>()
            .init_asset_loader::<MipTexLoader>()
            .init_asset_loader::<FontLoader>();
    }
}
