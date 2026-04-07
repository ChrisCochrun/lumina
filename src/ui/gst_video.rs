// use iced_video_player::Video;

// fn video_player(video: &Video) -> Element<Message> {}

use iced_video_player::Video;
use miette::{IntoDiagnostic, Result};
use url::Url;

pub fn create_video(url: &Url, framerate: u16) -> Result<Video> {
    // Based on `iced_video_player::Video::new`,
    // but without a text sink so that the built-in subtitle functionality triggers.
    // and with some better gstreamer tweaks
    use gstreamer as gst;
    use gstreamer_app as gst_app;
    use gstreamer_app::prelude::*;

    gst::init().into_diagnostic()?;

    let pipeline = format!(
        r#"playbin uri="{}" video-sink="videoscale ! videoconvert ! videoflip method=automatic ! videorate ! appsink name=lumina_video drop=true caps=video/x-raw,format=NV12,framerate={framerate}/1,pixel-aspect-ratio=1/1""#,
        url.as_str()
    );

    let pipeline =
        gst::parse::launch(pipeline.as_ref()).into_diagnostic()?;
    let pipeline = pipeline
        .downcast::<gst::Pipeline>()
        .map_err(|_| iced_video_player::Error::Cast)
        .into_diagnostic()?;

    let video_sink: gst::Element = pipeline.property("video-sink");
    let pad = video_sink.pads().first().cloned().expect("first pad");
    let pad = pad
        .dynamic_cast::<gst::GhostPad>()
        .map_err(|_| iced_video_player::Error::Cast)
        .into_diagnostic()?;
    let bin = pad
        .parent_element()
        .ok_or_else(|| {
            iced_video_player::Error::AppSink(String::from(
                "Should have a parent element here",
            ))
        })
        .into_diagnostic()?
        .downcast::<gst::Bin>()
        .map_err(|_| iced_video_player::Error::Cast)
        .into_diagnostic()?;
    let video_sink = bin
        .by_name("lumina_video")
        .ok_or_else(|| {
            iced_video_player::Error::AppSink(String::from(
                "Can't find element lumina_video",
            ))
        })
        .into_diagnostic()?;
    let video_sink = video_sink
        .downcast::<gst_app::AppSink>()
        .map_err(|_| iced_video_player::Error::Cast)
        .into_diagnostic()?;
    let result = Video::from_gst_pipeline(pipeline, video_sink, None);
    result.into_diagnostic()
}
