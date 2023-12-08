use std::io::{self, Cursor};

use bevy_asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use bevy_render::{
    render_resource::{TextureDimension, TextureFormat},
    texture::Image,
};
use bevy_utils::BoxedFuture;
use goldsrc_rs::texture::ColourData;

#[derive(Default)]
pub struct PicLoader;

impl AssetLoader for PicLoader {
    type Asset = Image;
    type Settings = ();
    type Error = io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a Self::Settings,
        _: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut buf = Cursor::new(Vec::new());
            reader.read_to_end(buf.get_mut()).await?;

            let pic = goldsrc_rs::pic(&mut buf)?;
            Ok(convert_to_image(pic.width, pic.height, &pic.data))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pic"]
    }
}

#[derive(Default)]
pub struct MipTexLoader;

impl AssetLoader for MipTexLoader {
    type Asset = Image;
    type Settings = ();
    type Error = io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a Self::Settings,
        _: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut buf = Cursor::new(Vec::new());
            reader.read_to_end(buf.get_mut()).await?;

            let miptex = goldsrc_rs::miptex(&mut buf)?;
            Ok(match miptex.data {
                Some(data) => convert_to_image(miptex.width, miptex.height, &data),
                None => Image::default(),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mip"]
    }
}

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
    type Asset = Image;
    type Settings = ();
    type Error = io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _: &'a Self::Settings,
        _: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut buf = Cursor::new(Vec::new());
            reader.read_to_end(buf.get_mut()).await?;

            let font = goldsrc_rs::font(&mut buf)?;
            Ok(convert_to_image(font.width, font.height, &font.data))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fnt"]
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
