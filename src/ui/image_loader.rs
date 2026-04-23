use std::io;

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

pub enum Error {
    NonImage,
    LoadingError(io::Error),
    ImageError(image::ImageError),
}
