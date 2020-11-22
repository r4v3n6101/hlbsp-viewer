use arraylib::Array;
use image::{ImageError, ImageResult};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};

const EXTENSION: &str = "tga";
const SIDES: [&str; 6] = ["rt", "lf", "up", "dn", "bk", "ft"];

pub struct Cubemap {
    dimension: u32,
    sides: [Vec<u8>; 6],
}

impl Cubemap {
    pub fn read<S: AsRef<str>, P: AsRef<Path>>(name: S, path: P) -> ImageResult<Self> {
        let mut dimension = 0;
        let sides: Vec<_> = SIDES
            .iter()
            .map(|postfix| {
                let file_name = format!("{}{}.{}", name.as_ref(), postfix, EXTENSION);
                let file_path = path.as_ref().join(file_name);
                let image = image::open(file_path)?.to_rgba8();
                if dimension == 0 {
                    dimension = image.width();
                } else if dimension != image.width() {
                    return Err(ImageError::IoError(IOError::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Different dimension of texture. Got {}, expected {}.",
                            image.width(),
                            dimension
                        ),
                    )));
                } else if image.width() != image.height() {
                    return Err(ImageError::IoError(IOError::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Not square texture. Width {}, height {}.",
                            image.width(),
                            image.height()
                        ),
                    )));
                }
                Ok(image.into_raw())
            })
            .collect::<ImageResult<_>>()?;
        let sides = <[Vec<u8>; 6]>::from_iter(sides.into_iter());

        Ok(Self { dimension, sides })
    }

    pub const fn dimension(&self) -> u32 {
        self.dimension
    }

    pub const fn sides(&self) -> &[Vec<u8>] {
        &self.sides
    }
}
