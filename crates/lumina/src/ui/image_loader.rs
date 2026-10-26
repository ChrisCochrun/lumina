use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;

use cosmic::widget::image::Handle;
use tokio::task::JoinError;

type Result<T> = std::result::Result<T, Error>;

pub async fn load_images(path: PathBuf) -> Result<Handle> {
    tokio::task::spawn_blocking(move || {
        let image = image::open(&path).map_err(Error::ImageError)?;
        let (width, height, pixels) =
            (image.width(), image.height(), image.to_rgba8().to_vec());
        Ok(Handle::from_rgba(width, height, pixels))
    })
    .await
    .map_err(Error::AsyncError)
    .flatten()
}

#[derive(Debug, Default)]
pub struct ImageLoader {
    decoded_images: HashMap<PathBuf, Handle>,
    decoding_images: HashSet<PathBuf>,
}

impl ImageLoader {
    pub fn load_image(&mut self, path: &PathBuf) -> Result<Handle> {
        if self.decoded_images.contains_key(path) {
            self.decoding_images.remove(path);
            self.decoded_images
                .get(path)
                .ok_or(Error::MissingImage)
                .cloned()
        } else {
            self.decoding_images.insert(path.clone());
            let image = image::open(path).map_err(Error::ImageError)?;
            let (width, height, pixels) =
                (image.width(), image.height(), image.into_bytes());
            self.decoding_images.remove(path);
            Ok(Handle::from_rgba(width, height, pixels))
        }
    }

    pub fn get_image(&self, path: &PathBuf) -> Result<Handle> {
        self.decoded_images
            .get(path)
            .ok_or(Error::MissingImage)
            .cloned()
    }
}

#[derive(Debug)]
pub enum Error {
    NonImage,
    AsyncError(JoinError),
    LoadingError(io::Error),
    ImageError(image::ImageError),
    MissingImage,
}
