use std::fmt::Display;
use std::num::NonZero;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use cosmic::Task;
use cosmic::widget::image::Handle;
use iced_video_player::gst_app::prelude::*;
use iced_video_player::gst_app::{self};
use iced_video_player::{Position, Video, gst};
use image::{DynamicImage, ImageFormat, RgbaImage};
use tracing::{debug, error};
use url::Url;

use crate::core::slide::SlideId;
use crate::ui::presenter;

#[derive(Debug)]
pub struct VideoSettings {
    pub mute: bool,
    pub framerate: u16,
    pub appsink_name: String,
}

#[derive(Debug, Default)]
pub struct VideoManager {
    pub previous_video: Option<VideoPair>,
    pub current_video: Option<VideoPair>,
    pub next_video: Option<VideoPair>,

    pub paused: bool,
    pub muted: bool,
    pub position: f64,
}

#[derive(Debug)]
pub struct VideoPair {
    pub preview: Video,
    pub presentation: Video,
    pub path: PathBuf,
    pub slide_id: SlideId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoLocation {
    Current,
    Next,
    Previous,
}

impl VideoManager {
    pub fn activate(&mut self, looping: bool, paused: Option<bool>, mute: Option<bool>) {
        if let Some(pair) = &mut self.current_video {
            pair.preview.set_looping(looping);
            pair.presentation.set_looping(looping);
            pair.preview.set_muted(true);
            pair.presentation.set_muted(mute.unwrap_or(false));
            pair.preview.set_paused(paused.unwrap_or(false));
            pair.presentation.set_paused(paused.unwrap_or(false));
        }
    }

    pub fn park(&mut self) {
        if let Some(pair) = &mut self.current_video {
            pair.preview.set_paused(true);
            pair.presentation.set_paused(true);
            pair.preview.set_muted(true);
            pair.presentation.set_muted(true);
        }
    }

    pub fn mute(&mut self, mute: bool) {
        if let Some(pair) = &mut self.current_video {
            pair.presentation.set_muted(mute);
            self.muted = mute;
        }
    }

    pub fn pause(&mut self, pause: bool) {
        if let Some(pair) = &mut self.current_video {
            pair.presentation.set_paused(pause);
            pair.preview.set_paused(pause);
            self.paused = pause;
        }
    }

    pub fn seek(&mut self, position: f64) {
        if let Some(pair) = &mut self.current_video {
            let position = Position::Time(std::time::Duration::from_secs_f64(position));
            if let Err(e) = pair.preview.seek(position, false) {
                error!("Problem changing video position: {e}");
            }
            if let Err(e) = pair.presentation.seek(position, false) {
                error!("Problem changing video position: {e}");
            }
        }
    }

    pub fn update_position(&mut self) {
        if let Some(pair) = &self.current_video
            && self.position > 0.0
            && pair.preview.position().as_secs_f64() != 0.0
        {
            self.position = pair.preview.position().as_secs_f64();
        }
    }

    pub(crate) fn preload(
        &mut self,
        path: PathBuf,
        slide_id: SlideId,
        video_location: VideoLocation,
    ) -> Task<presenter::Message> {
        Task::perform(
            async move {
                tokio::task::spawn_blocking(|| load_video(path, slide_id))
                    .await
                    .map_err(|_| {
                        VideoError::IOError(std::io::Error::other(
                            String::from("Error in spawning tokio task"),
                        ))
                    })
                    .and_then(|result| result)
            },
            |result| match result {
                Ok(videos) => presenter::Message::VideoPipelineFinished((
                    video_location,
                    Arc::new(videos),
                )),
                Err(e) => {
                    tracing::error!(?e);
                    presenter::Message::None
                }
            },
        )
    }

    pub fn set_new_video(
        &mut self,
        video_location: &VideoLocation,
        video_pair: Arc<VideoPair>,
    ) {
        match video_location {
            VideoLocation::Current => {
                self.current_video = Some(Arc::into_inner(video_pair).expect("Woops"));
                if let Some(pair) = &mut self.current_video {
                    pair.preview.set_muted(true);
                    pair.preview.set_paused(false);
                    pair.presentation.set_muted(false);
                    pair.presentation.set_paused(false);
                }
            }
            VideoLocation::Next => {
                self.next_video = Some(Arc::into_inner(video_pair).expect("Woops next"));
                if let Some(pair) = &mut self.next_video {
                    pair.preview.set_muted(true);
                    pair.preview.set_paused(true);
                    pair.presentation.set_muted(true);
                    pair.presentation.set_paused(true);
                }
            }
            VideoLocation::Previous => {
                self.previous_video =
                    Some(Arc::into_inner(video_pair).expect("Woops previous"));
                if let Some(pair) = &mut self.previous_video {
                    pair.preview.set_muted(true);
                    pair.preview.set_paused(true);
                    pair.presentation.set_muted(true);
                    pair.presentation.set_paused(true);
                }
            }
        }
    }
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            mute: true,
            framerate: 60,
            appsink_name: String::from("lumina_video"),
        }
    }
}

type Result<T> = std::result::Result<T, VideoError>;

fn load_video(path: PathBuf, slide_id: SlideId) -> Result<VideoPair> {
    let url = Url::from_file_path(&path).map_err(|()| {
        VideoError::IOError(std::io::Error::other(
            String::from("Couldn't convert video path to url"),
        ))
    })?;
    let main_settings = VideoSettings::default();
    let preview_settings = VideoSettings {
        mute: true,
        framerate: 15,
        appsink_name: String::from("lumina_preview"),
    };

    let main_video = create_video(&url, &main_settings)?;
    let preview_video = create_video(&url, &preview_settings)?;
    Ok(VideoPair {
        preview: preview_video,
        presentation: main_video,
        path,
        slide_id,
    })
}

pub fn create_video(url: &Url, settings: &VideoSettings) -> Result<Video> {
    // Based on `iced_video_player::Video::new`,
    // but without a text sink so that the built-in subtitle functionality triggers.
    // and with some better gstreamer tweaks

    gst::init().map_err(VideoError::GlibError)?;

    let pipeline = format!(
        r#"playbin uri="{0}" mute="{3}" video-sink="videoscale ! videoconvert ! videoflip method=automatic ! videorate ! appsink name={1} drop=true caps=video/x-raw,format=NV12,framerate={2}/1,pixel-aspect-ratio=1/1""#,
        url.as_str(),
        settings.appsink_name,
        settings.framerate,
        settings.mute,
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

pub fn thumbnail(input: &Url, output: &mut PathBuf) -> Result<Handle> {
    output.set_extension("png");
    if output.exists() {
        let image = image::open(&output).map_err(VideoError::ThumbnailImageError)?;
        let (width, height, pixels) =
            (image.width(), image.height(), image.to_rgba8().to_vec());
        return Ok(Handle::from_rgba(width, height, pixels));
    }
    debug!(?output);

    let thumbnails = {
        let mut video = create_video(input, &VideoSettings::default())?;

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

        if !output.exists() {
            output.set_extension("png");
            debug!(?output);
        }

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
    IOError(std::io::Error),
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
            Self::IOError(error) => {
                write!(f, "IOError: {error}")
            }
        }
    }
}
