use std::{collections::HashMap, io::Cursor};

use bevy_asset::{AssetLoader, Error, LoadContext, LoadedAsset};
use bevy_log::warn;
use bevy_render::{
    render_resource::{TextureDimension, TextureFormat},
    texture::Image,
};
use goldsrc_rs::{
    texture::{ColourData, Font, MipTexture, Picture},
    wad::Content,
};

use crate::types::Wad;

pub struct WadLoader;

impl AssetLoader for WadLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> bevy_asset::BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let contents = goldsrc_rs::wad(Cursor::new(bytes))?;

            let mut images = HashMap::with_capacity(contents.len());
            for (name, content) in contents {
                match content {
                    Content::Picture(Picture {
                        width,
                        height,
                        data,
                    }) => {
                        // TODO : temporary solution
                        images.insert(
                            name.to_string(),
                            load_context.set_labeled_asset(
                                &name,
                                LoadedAsset::new(convert_to_image(width, height, &data)),
                            ),
                        );
                    }
                    Content::MipTexture(MipTexture {
                        width,
                        height,
                        data,
                        name,
                    }) => {
                        let Some(data) = data else {
                            warn!(?name, "Empty MipTexture");
                            continue;
                        };

                        images.insert(
                            name.to_string(),
                            load_context.set_labeled_asset(
                                &name,
                                LoadedAsset::new(convert_to_image(width, height, &data)),
                            ),
                        );
                    }
                    Content::Font(Font { .. }) => {
                        // TODO : load font as font, not picture
                        warn!("Fonts no supported yet");
                        continue;
                    }
                    Content::Other { ty, .. } => {
                        warn!(ty, "Unsupported content type");
                        continue;
                    }
                    _ => unreachable!(),
                };
            }

            load_context.set_default_asset(LoadedAsset::new(Wad { images }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["wad"]
    }
}

fn convert_to_image<const N: usize>(width: u32, height: u32, color_data: &ColourData<N>) -> Image {
    let data = color_data
        .indices
        .iter()
        .flatten()
        .map(|idx| color_data.palette[*idx as usize])
        .flat_map(|[r, g, b]| {
            if r == 0 && g == 0 && b == 255 {
                [0u8; 4]
            } else {
                [r, g, b, 255]
            }
        })
        .collect();
    let mut image = Image {
        data,
        ..Default::default()
    };
    image.texture_descriptor.dimension = TextureDimension::D2;
    image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
    image.texture_descriptor.size.width = width;
    image.texture_descriptor.size.height = height;
    image.texture_descriptor.mip_level_count = N as u32;

    image
}
