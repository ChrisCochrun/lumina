use std::fmt::Display;
use std::num::NonZero;
use std::path::Path;
use std::time::Duration;

use cosmic::widget::image::Handle;
use iced_video_player::gst_app::prelude::*;
use iced_video_player::gst_app::{self};
use iced_video_player::{Position, Video, gst};
use image::{DynamicImage, ImageFormat, RgbaImage};
use url::Url;

#[derive(Debug, Default)]
pub struct VideoSettings {
    pub mute: bool,
    pub framerate: u16,
    pub appsink_name: String,
}

type Result<T> = std::result::Result<T, VideoError>;

pub fn create_video(url: &Url, settings: &VideoSettings) -> Result<Video> {
    // Based on `iced_video_player::Video::new`,
    // but without a text sink so that the built-in subtitle functionality triggers.
    // and with some better gstreamer tweaks

    gst::init().map_err(VideoError::GlibError)?;

    let pipeline = format!(
        r#"playbin uri="{0}" video-sink="videoscale ! videoconvert ! videoflip method=automatic ! videorate ! appsink name={1} drop=true caps=video/x-raw,format=NV12,framerate={2}/1,pixel-aspect-ratio=1/1{3}""#,
        url.as_str(),
        settings.appsink_name,
        settings.framerate,
        if settings.mute { ",mute=true" } else { "" },
    );

    let pipeline =
        gst::parse::launch(pipeline.as_ref()).map_err(VideoError::GlibError)?;
    let pipeline = pipeline
        .downcast::<gst::Pipeline>()
        .map_err(|_| VideoError::IcedVideoError(iced_video_player::Error::Cast))?;

    let video_sink: gst::Element = pipeline.property("video-sink");
    let pad = video_sink.pads().first().cloned().expect("first pad");
    let pad = pad
        .dynamic_cast::<gst::GhostPad>()
        .map_err(|_| VideoError::IcedVideoError(iced_video_player::Error::Cast))?;
    let bin = pad
        .parent_element()
        .ok_or_else(|| {
            VideoError::IcedVideoError(iced_video_player::Error::AppSink(String::from(
                "Should have a parent element here",
            )))
        })?
        .downcast::<gst::Bin>()
        .map_err(|_| VideoError::IcedVideoError(iced_video_player::Error::Cast))?;
    let video_sink = bin.by_name(&settings.appsink_name).ok_or_else(|| {
        VideoError::IcedVideoError(iced_video_player::Error::AppSink(format!(
            "Can't find element {}",
            settings.appsink_name
        )))
    })?;
    let video_sink = video_sink
        .downcast::<gst_app::AppSink>()
        .map_err(|_| VideoError::IcedVideoError(iced_video_player::Error::Cast))?;
    Video::from_gst_pipeline(pipeline, video_sink, None)
        .map_err(VideoError::IcedVideoError)
}

pub fn thumbnail(input: &Url, output: &Path) -> Result<Handle> {
    let thumbnails = {
        let mut video = create_video(
            input,
            &VideoSettings {
                mute: true,
                ..Default::default()
            },
        )?;

        let duration = video.duration();
        //TODO: how best to decide time?
        let position = if duration.as_secs_f64() < 20.0 {
            // If less than 20 seconds, divide duration by 2
            Position::Time(duration / 2)
        } else {
            // If more than 20 seconds, thumbnail at 10 seconds
            Position::Time(Duration::new(10, 0))
        };
        video
            .thumbnails([position], NonZero::new(1).expect("Not zero"))
            .map_err(VideoError::IcedVideoError)?
    };
    // TODO: do not require clone of pixels data
    if let Some(cosmic::widget::image::Handle::Rgba {
        id: _,
        width,
        height,
        pixels,
    }) = &thumbnails.first()
    {
        let image = RgbaImage::from_raw(*width, *height, pixels.to_vec())
            .map(DynamicImage::ImageRgba8)
            .ok_or_else(|| {
                VideoError::ThumbnailError(String::from("Cannot convert handle to image"))
            })?;

        image
            .save_with_format(output, ImageFormat::Png)
            .map_err(VideoError::ThumbnailImageError)?;
    } else {
        return Err(VideoError::ThumbnailError(String::from(
            "Unsupported handle format",
        )));
    }

    thumbnails
        .first()
        .cloned()
        .ok_or_else(|| VideoError::ThumbnailError(String::from("Error creating handles")))
}

#[derive(Debug)]
pub enum VideoError {
    ThumbnailError(String),
    IcedVideoError(iced_video_player::Error),
    GlibError(gst::glib::Error),
    ThumbnailImageError(image::ImageError),
}

impl std::error::Error for VideoError {}

impl Display for VideoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThumbnailError(message) => {
                write!(f, "ThumbnailError: {message}")
            }
            Self::IcedVideoError(error) => {
                write!(f, "IcedVideoError: {error}")
            }
            Self::GlibError(error) => {
                write!(f, "GlibError: {error}")
            }
            Self::ThumbnailImageError(error) => {
                write!(f, "ImageError: {error}")
            }
        }
    }
}
