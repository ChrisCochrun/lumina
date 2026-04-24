use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

use cosmic::widget::image::Handle;

use crate::core::slide::{BackgroundKind, Slide};

type Result<T> = std::result::Result<T, Error>;

pub fn load_images(mut slide: Slide) -> Result<Slide> {
    if matches!(slide.background().kind, BackgroundKind::Image) {
        let path = &slide.background().path;
        let image =
            image::open(path).map_err(|e| Error::ImageError(e))?;
        let (width, height, pixels) = (
            image.width(),
            image.height(),
            image.to_rgba8().to_vec(),
        );

        slide.background.image_handle =
            Some(Handle::from_rgba(width, height, pixels));
        Ok(slide)
    } else {
        Ok(slide)
    }
}

#[derive(Debug, Default)]
pub struct ImageLoader {
    decoded_images: HashMap<PathBuf, Handle>,
    decoding_images: HashSet<PathBuf>,
}

impl ImageLoader {
    pub fn load_image(&mut self, path: PathBuf) -> Result<Handle> {
        if self.decoded_images.contains_key(&path) {
            self.decoding_images.remove(&path);
            self.decoded_images
                .get(&path)
                .ok_or(Error::MissingImage)
                .map(Clone::clone)
        } else {
            self.decoding_images.insert(path.clone());
            let image = image::open(&path)
                .map_err(|e| Error::ImageError(e))?;
            let (width, height, pixels) =
                (image.width(), image.height(), image.into_bytes());
            self.decoding_images.remove(&path);
            Ok(Handle::from_rgba(width, height, pixels))
        }
    }

    pub fn get_image(&self, path: &PathBuf) -> Result<Handle> {
        self.decoded_images
            .get(path)
            .ok_or(Error::MissingImage)
            .map(Clone::clone)
    }
}

#[derive(Debug)]
pub enum Error {
    NonImage,
    LoadingError(io::Error),
    ImageError(image::ImageError),
    MissingImage,
}
