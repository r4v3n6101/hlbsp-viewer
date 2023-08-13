use std::{
    io::{self, ErrorKind},
    path::Path,
};

use image::{ImageError, ImageResult};

const SIDES: [&str; 6] = ["rt", "lf", "up", "dn", "bk", "ft"];

#[derive(Debug)]
pub struct Cubemap {
    pub dimension: u32,
    pub sides: [Vec<u8>; 6],
}

#[inline]
pub fn read_cubemap<P: AsRef<Path>>(name: &str, path: P) -> ImageResult<Cubemap> {
    let mut dimension = 0;
    let mut sides = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];

    for (i, postfix) in SIDES.iter().enumerate() {
        let file_name = format!("{}{}", name, postfix);
        let file_path = path.as_ref().join(file_name).with_extension("tga");
        let image = image::open(file_path)?.to_rgba8();

        if dimension == 0 {
            dimension = image.width();
        } else if dimension != image.width() || image.width() != image.height() {
            return Err(ImageError::IoError(io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Invalid texture dimensions. Side {}: Width {}, Height {}.",
                    i,
                    image.width(),
                    image.height()
                ),
            )));
        }

        sides[i] = image.into_raw();
    }

    Ok(Cubemap { dimension, sides })
}
