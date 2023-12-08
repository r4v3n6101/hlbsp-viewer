use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use bevy_asset::{
    io::{AssetReader, AssetReaderError, PathStream, Reader},
    BoxedFuture,
};
use bevy_utils::tracing::{error, info};
use futures::{
    future,
    io::{self, AllowStdIo},
    stream,
};
use goldsrc_rs::{wad::Entry, wad_entries, CStr16};

// TODO : generalize for valve asset reader?
pub struct WadAssetReader {
    entries: HashMap<CStr16, Entry>,
}

impl WadAssetReader {
    pub fn new(root_path: &Path) -> Self {
        let mut entries = HashMap::new();

        match fs::read_dir(root_path) {
            Ok(dir_entries) => {
                for entry in dir_entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => {
                            error!(?err, "error reading entry");
                            continue;
                        }
                    };
                    let path = entry.path();
                    if path.extension() == Some(OsStr::new("wad"))
                        || path.extension() == Some(OsStr::new("WAD"))
                    {
                        match File::open(&path)
                            .map(BufReader::new)
                            .and_then(|reader| wad_entries(reader, true))
                        {
                            Ok(x) => {
                                info!(entries = x.len(), ?path, "new wad detected");
                                entries.extend(x.into_iter());
                            }
                            Err(err) => {
                                error!(?err, "error reading wad");
                            }
                        }
                    }
                }
            }
            Err(err) => {
                error!(?err, "error reading directory");
            }
        };

        Self { entries }
    }
}

impl AssetReader for WadAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async {
            let name = path.file_stem().and_then(|d| d.to_str()).ok_or_else(|| {
                AssetReaderError::Io(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid filename passed",
                ))
            })?;
            let name = name.to_lowercase();
            let entry = self
                .entries
                .get(name.as_str())
                .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))?;

            Ok(Box::new(AllowStdIo::new(entry.reader())) as Box<Reader<'a>>)
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(future::err(AssetReaderError::NotFound(path.to_path_buf())))
    }

    fn read_directory<'a>(
        &'a self,
        _: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(future::ok(Box::new(stream::empty()) as Box<PathStream>))
    }

    fn is_directory<'a>(&'a self, _: &'a Path) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        Box::pin(future::ok(false))
    }
}
