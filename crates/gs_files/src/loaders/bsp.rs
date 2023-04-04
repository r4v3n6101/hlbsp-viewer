use std::{io::Cursor, path::Path};

use bevy_asset::{AssetLoader, AssetPath, BoxedFuture, Error, LoadContext, LoadedAsset};
use goldsrc_rs::{bsp::Level, map::Entity};

use crate::types::Bsp;

pub struct BspLoader;

impl AssetLoader for BspLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let level = goldsrc_rs::bsp(Cursor::new(bytes))?;

            let worldspawn = find_worldspawn(&level);
            let skyname = worldspawn.and_then(|w| w.get("skyname")).cloned();
            let wads_deps = worldspawn
                .and_then(|w| w.get("wad"))
                .map(|w| wads_as_dependencies(w))
                .unwrap_or_default();
            let wads = wads_deps
                .iter()
                .map(|path| load_context.get_handle(path.get_id()))
                .collect();

            load_context.set_default_asset(
                LoadedAsset::new(Bsp { skyname, wads }).with_dependencies(wads_deps),
            );

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bsp"]
    }
}

fn find_worldspawn(level: &Level) -> Option<&Entity> {
    level
        .entities
        .iter()
        .find(|e| e.get("classname").map(|s| s.as_str()) == Some("worldspawn"))
}

fn wads_as_dependencies(s: &str) -> Vec<AssetPath<'static>> {
    s.split(';')
        .map(|wad| Path::new(wad.rsplit('\\').next().unwrap_or(wad)))
        .map(|p| AssetPath::new(p.to_path_buf(), None))
        .collect()
}
