#![allow(clippy::similar_names, unused)]
use cosmic::cosmic_theme::palette::Srgb;
use cosmic::iced::core::image::Allocation;
use cosmic::iced::font::{Stretch, Style, Weight};
use cosmic::iced::{Rectangle, Size, core as iced_core};
use cosmic::widget::image::Handle;
// use cosmic::dialog::ashpd::url::Url;
use crisp::types::{Keyword, Symbol, Value};
use fdsm_lumina::cosmic_text::{Attrs, FontSystem, Metrics};
use fdsm_lumina::style::{Color as FdsmColor, Glow, GlyphColoring, TextStyle};
use fdsm_lumina::{StyledText, StyledTextBuilder, TextRegion};
use fontdb::Family;
use iced_video_player::Video;
use iced_wgpu::core;
use image::EncodableLayout;
use miette::{Result, miette};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::ui::gst_video;
use crate::ui::text_svg::{Color, Font, Shadow, Stroke, TextSvg};

use super::songs::Song;

#[derive(Clone, Debug, Default)]
pub struct Slide {
    // This is not skipped so we can load back the slide_actions
    // that get mapped to it.
    pub(crate) id: SlideId,
    pub(crate) background: Background,
    pub(crate) thumbnail: Option<Allocation>,
    pub(crate) text: String,
    pub(crate) font: Option<Font>,
    pub(crate) font_size: i32,
    pub(crate) stroke: Option<Stroke>,
    pub(crate) shadow: Option<Shadow>,
    pub text_properties: Option<TextProperties>,
    pub(crate) text_alignment: TextAlignment,
    pub(crate) text_color: Option<Color>,
    pub(crate) audio: Option<PathBuf>,
    pub(crate) video_loop: bool,
    pub(crate) video_start_time: f32,
    pub(crate) video_end_time: f32,
    pub(crate) pdf_index: u32,
    pub(crate) text_svg: Option<TextSvg>,
    pub(crate) pdf_page: Option<Handle>,
    pub(crate) styled_text: Option<Arc<StyledText>>,
}

impl PartialEq for Slide {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.background == other.background
            && self.thumbnail == other.thumbnail
            && self.text == other.text
            && self.font == other.font
            && self.font_size == other.font_size
            && self.stroke == other.stroke
            && self.shadow == other.shadow
            && self.text_properties == other.text_properties
            && self.text_alignment == other.text_alignment
            && self.text_color == other.text_color
            && self.audio == other.audio
            && self.video_loop == other.video_loop
            && self.video_start_time == other.video_start_time
            && self.video_end_time == other.video_end_time
            && self.pdf_index == other.pdf_index
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TextProperties {
    pub text_alignment: TextAlignment,
    pub font: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<Weight>,
    pub font_style: Option<Style>,
    pub font_stretch: Option<Stretch>,
    pub text_color: Option<Srgb>,
    pub stroke_size: Option<f32>,
    pub stroke_color: Option<Srgb>,
    pub shadow_size: Option<f32>,
    pub shadow_offset: Option<(f32, f32)>,
    pub shadow_color: Option<Srgb>,
}

#[derive(
    Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash,
)]
pub struct SlideId(Uuid);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundKind {
    #[default]
    Image,
    Video,
    Pdf,
    Html,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TextAlignment {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    #[default]
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl From<TextAlignment> for fdsm_lumina::cosmic_text::Align {
    fn from(value: TextAlignment) -> Self {
        match value {
            TextAlignment::TopLeft => Self::Left,
            TextAlignment::TopCenter => Self::Center,
            TextAlignment::TopRight => Self::Right,
            TextAlignment::MiddleLeft => Self::Left,
            TextAlignment::MiddleCenter => Self::Center,
            TextAlignment::MiddleRight => Self::Right,
            TextAlignment::BottomLeft => Self::Left,
            TextAlignment::BottomCenter => Self::Center,
            TextAlignment::BottomRight => Self::Right,
        }
    }
}

impl From<Value> for TextAlignment {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

impl From<&Value> for TextAlignment {
    fn from(value: &Value) -> Self {
        if value == &Value::Symbol("center".into()) {
            Self::MiddleCenter
        } else {
            Self::TopCenter
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Background {
    pub path: PathBuf,
    pub kind: BackgroundKind,
    #[serde(skip)]
    pub image_handle: Option<Handle>,
    #[serde(skip)]
    pub image_allocation: Option<Allocation>,
}

impl TryFrom<&Background> for Video {
    type Error = ParseError;

    fn try_from(value: &Background) -> std::result::Result<Self, Self::Error> {
        Self::new(
            &url::Url::from_file_path(value.path.clone())
                .map_err(|()| ParseError::BackgroundNotVideo)?,
        )
        .map_err(|_| ParseError::BackgroundNotVideo)
    }
}

impl TryFrom<Background> for Video {
    type Error = ParseError;

    fn try_from(value: Background) -> std::result::Result<Self, Self::Error> {
        let url = &url::Url::from_file_path(value.path)
            .map_err(|()| ParseError::BackgroundNotVideo)?;

        let settings = gst_video::VideoSettings {
            mute: true,
            framerate: 30,
            appsink_name: "lumina_video".to_string(),
        };
        gst_video::create_video(url, &settings)
            .map_err(|_| ParseError::BackgroundNotVideo)
    }
}

impl TryFrom<String> for Background {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<PathBuf> for Background {
    type Error = ParseError;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let path = if path.starts_with("~") {
            let path = path.to_str().expect("Should have a string").to_string();
            let path = path.trim_start_matches("file://");
            let home = dirs::home_dir()
                .expect("We should have a home directory")
                .to_str()
                .expect("Gah")
                .to_string();
            let path = path.replace('~', &home);
            PathBuf::from(path)
        } else {
            path
        };

        match path.canonicalize() {
            Ok(value) => {
                let extension = value
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                match extension {
                    "jpeg" | "jpg" | "png" | "webp" => Ok(Self {
                        path: value.clone(),
                        kind: BackgroundKind::Image,
                        image_handle: Some(value.into()),
                        image_allocation: None,
                    }),
                    "mp4" | "mkv" | "webm" => Ok(Self {
                        path: value,
                        kind: BackgroundKind::Video,
                        image_handle: None,
                        image_allocation: None,
                    }),
                    "pdf" => Ok(Self {
                        path: value,
                        kind: BackgroundKind::Pdf,
                        image_handle: None,
                        image_allocation: None,
                    }),
                    "html" => Ok(Self {
                        path: value,
                        kind: BackgroundKind::Html,
                        image_handle: None,
                        image_allocation: None,
                    }),
                    _ => Err(ParseError::NonBackgroundFile),
                }
            }
            Err(_e) => {
                // error!("Couldn't canonicalize: {e} {:?}", path);
                Err(ParseError::CannotCanonicalize)
            }
        }
    }
}

impl TryFrom<&str> for Background {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.trim_start_matches("file://");
        if value.starts_with('~') {
            dirs::home_dir().map_or_else(
                || Self::try_from(PathBuf::from(value)),
                |home| {
                    home.to_str().map_or_else(
                        || Self::try_from(PathBuf::from(value)),
                        |home| {
                            let value = value.replace('~', home);
                            Self::try_from(PathBuf::from(value))
                        },
                    )
                },
            )
        } else if value.starts_with("./") {
            Err(ParseError::CannotCanonicalize)
        } else {
            Self::try_from(PathBuf::from(value))
        }
    }
}

impl TryFrom<&Path> for Background {
    type Error = ParseError;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::try_from(PathBuf::from(value))
    }
}

#[derive(Debug)]
pub enum ParseError {
    NonBackgroundFile,
    DoesNotExist,
    CannotCanonicalize,
    BackgroundNotVideo,
}

impl std::error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NonBackgroundFile => "The file is not a recognized image or video type",
            Self::DoesNotExist => "This file doesn't exist",
            Self::CannotCanonicalize => "Could not canonicalize this file",
            Self::BackgroundNotVideo => "This background isn't a video",
        };
        write!(f, "Error: {message}")
    }
}

impl From<String> for BackgroundKind {
    fn from(value: String) -> Self {
        if value == "image" {
            Self::Image
        } else {
            Self::Video
        }
    }
}

impl From<&Slide> for Value {
    fn from(_value: &Slide) -> Self {
        Self::List(vec![Self::Symbol(Symbol("slide".into()))])
    }
}

impl Slide {
    #[must_use]
    pub fn set_text(mut self, text: impl AsRef<str>) -> Self {
        self.text = text.as_ref().into();
        self
    }

    #[must_use]
    pub fn with_text_svg(mut self, text_svg: TextSvg) -> Self {
        self.text_svg = Some(text_svg);
        self
    }

    #[must_use]
    pub fn set_font(mut self, font: impl AsRef<str>) -> Self {
        self.font = Some(font.as_ref().into());
        self
    }

    #[must_use]
    pub const fn set_font_size(mut self, font_size: i32) -> Self {
        self.font_size = font_size;
        self
    }

    #[must_use]
    pub fn set_audio(mut self, audio: Option<PathBuf>) -> Self {
        self.audio = audio;
        self
    }

    #[must_use]
    pub const fn set_pdf_index(mut self, pdf_index: u32) -> Self {
        self.pdf_index = pdf_index;
        self
    }

    #[must_use]
    pub const fn set_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    #[must_use]
    pub const fn set_shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        self
    }

    #[must_use]
    pub const fn set_text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    #[must_use]
    pub const fn background(&self) -> &Background {
        &self.background
    }

    #[must_use]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    #[must_use]
    pub const fn text_alignment(&self) -> TextAlignment {
        self.text_alignment
    }

    #[must_use]
    pub const fn font_size(&self) -> i32 {
        self.font_size
    }

    #[must_use]
    pub fn font(&self) -> Option<Font> {
        self.font.clone()
    }

    #[must_use]
    pub const fn video_loop(&self) -> bool {
        self.video_loop
    }

    #[must_use]
    pub fn audio(&self) -> Option<PathBuf> {
        self.audio.clone()
    }

    #[must_use]
    pub fn pdf_page(&self) -> Option<Handle> {
        self.pdf_page.clone()
    }

    #[must_use]
    pub fn text_color(&self) -> Option<Color> {
        self.text_color.clone()
    }

    #[must_use]
    pub fn stroke(&self) -> Option<Stroke> {
        self.stroke.clone()
    }

    #[must_use]
    pub fn shadow(&self) -> Option<Shadow> {
        self.shadow.clone()
    }

    #[must_use]
    pub const fn pdf_index(&self) -> u32 {
        self.pdf_index
    }

    pub fn prepare_text(&mut self, font_system: &mut FontSystem) -> Result<()> {
        if let Some(text_properties) = &self.text_properties {
            let styled_text = build_text(font_system, &self.text, &text_properties);
            self.styled_text = Some(Arc::new(styled_text));
            Ok(())
        } else {
            Err(miette!("There is no text_properties on this slide"))
        }
    }

    // pub fn song_slides(song: &Song) -> Result<Vec<Self>> {
    //     let lyrics = song.get_lyrics()?;
    //     let slides: Vec<Self> = lyrics
    //         .iter()
    //         .map(|l| {
    //             let song = song.clone();
    //             SlideBuilder::new()
    //                 .background(song.background.unwrap_or_default())
    //                 .font(song.font.unwrap_or_default())
    //                 .font_size(song.font_size.unwrap_or_default())
    //                 .text_alignment(song.text_alignment.unwrap_or_default())
    //                 .audio(song.audio.unwrap_or_default())
    //                 .video_loop(true)
    //                 .video_start_time(0.0)
    //                 .video_end_time(0.0)
    //                 .text(l)
    //                 .build()
    //                 .unwrap_or_default()
    //         })
    //         .collect();

    //     Ok(slides)
    // }

    pub(crate) const fn set_id(&mut self, id: Uuid) {
        self.id = SlideId(id);
    }

    // pub fn slides_from_item(item: &ServiceItem) -> Result<Vec<Self>> {
    //     todo!()
    // }
}

// impl From<Value> for Slide {
//     fn from(value: Value) -> Self {
//         Self::from(&value)
//     }
// }

// impl From<&Value> for Slide {
//     fn from(value: &Value) -> Self {
//         match value {
//             Value::List(list) => lisp_to_slide(list),
//             _ => Self::default(),
//         }
//     }
// }

// #[allow(clippy::option_if_let_else)]
// fn lisp_to_slide(lisp: &[Value]) -> Slide {
//     const DEFAULT_BACKGROUND_LOCATION: usize = 1;
//     const DEFAULT_TEXT_LOCATION: usize = 0;

//     let mut slide = SlideBuilder::new();
//     let background_position = if let Some(background) = lisp
//         .iter()
//         .position(|v| v == &Value::Keyword(Keyword::from("background")))
//     {
//         background + 1
//     } else {
//         DEFAULT_BACKGROUND_LOCATION
//     };

//     if let Some(background) = lisp.get(background_position) {
//         slide = slide.background(lisp_to_background(background));
//     } else {
//         slide = slide.background(Background::default());
//     }

//     let text_position = lisp.iter().position(|v| match v {
//         Value::List(vec) => {
//             vec[DEFAULT_TEXT_LOCATION] == Value::Symbol(Symbol::from("text"))
//         }
//         _ => false,
//     });

//     if let Some(text_position) = text_position {
//         if let Some(text) = lisp.get(text_position) {
//             slide = slide.text(lisp_to_text(text));
//         } else {
//             slide = slide.text("");
//         }
//     } else {
//         slide = slide.text("");
//     }

//     if let Some(text_position) = text_position {
//         if let Some(text) = lisp.get(text_position) {
//             slide = slide.font_size(lisp_to_font_size(text));
//         } else {
//             slide = slide.font_size(0);
//         }
//     } else {
//         slide = slide.font_size(0);
//     }

//     slide = slide
//         .font("Quicksand")
//         .text_alignment(TextAlignment::MiddleCenter)
//         .video_loop(false)
//         .video_start_time(0.0)
//         .video_end_time(0.0);

//     match slide.build() {
//         Ok(slide) => slide,
//         Err(e) => {
//             error!("Shoot! Slide didn't build: {e}");
//             Slide::default()
//         }
//     }
// }

// #[allow(clippy::option_if_let_else)]
// fn lisp_to_font_size(lisp: &Value) -> i32 {
//     match lisp {
//         Value::List(list) => {
//             if let Some(font_size_position) = list
//                 .iter()
//                 .position(|v| v == &Value::Keyword(Keyword::from("font-size")))
//             {
//                 if let Some(font_size_value) = list.get(font_size_position + 1) {
//                     font_size_value.into()
//                 } else {
//                     50
//                 }
//             } else {
//                 50
//             }
//         }
//         _ => 50,
//     }
// }

// fn lisp_to_text(lisp: &Value) -> impl Into<String> {
//     match lisp {
//         Value::List(list) => list[1].clone(),
//         _ => "".into(),
//     }
// }

// // Need to return a Result here so that we can propogate
// // errors and then handle them appropriately
// #[allow(clippy::option_if_let_else)]
// pub fn lisp_to_background(lisp: &Value) -> Background {
//     match lisp {
//         Value::List(list) => {
//             let _kind = list[0].clone();
//             if let Some(source) = list
//                 .iter()
//                 .position(|v| v == &Value::Keyword(Keyword::from("source")))
//             {
//                 let source = &list[source + 1];
//                 match source {
//                     Value::String(s) => {
//                         if s.starts_with("./") {
//                             let Some(home) = dirs::home_dir() else {
//                                 panic!("Should always be there");
//                             };
//                             let Some(home) = home.to_str() else {
//                                 panic!("Should always be there");
//                             };
//                             let mut home = home.to_string();
//                             home.push('/');

//                             let s = s.replace("./", &home);
//                             match Background::try_from(s.as_str()) {
//                                 Ok(background) => background,
//                                 Err(e) => {
//                                     error!("Couldn't load background: {e}");
//                                     Background::default()
//                                 }
//                             }
//                         } else {
//                             match Background::try_from(s.as_str()) {
//                                 Ok(background) => background,
//                                 Err(e) => {
//                                     error!("Couldn't load background: {e}");
//                                     Background::default()
//                                 }
//                             }
//                         }
//                     }
//                     _ => Background::default(),
//                 }
//             } else {
//                 Background::default()
//             }
//         }
//         _ => Background::default(),
//     }
// }

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SlideBuilder {
    background: Option<Background>,
    text: Option<String>,
    font: Option<Font>,
    font_size: Option<i32>,
    audio: Option<PathBuf>,
    stroke: Option<Stroke>,
    shadow: Option<Shadow>,
    text_color: Option<Color>,
    text_alignment: Option<TextAlignment>,
    video_loop: Option<bool>,
    video_start_time: Option<f32>,
    video_end_time: Option<f32>,
    pdf_index: Option<u32>,
    #[serde(skip)]
    pdf_page: Option<Handle>,
    #[serde(skip)]
    text_svg: Option<TextSvg>,
}

impl SlideBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn background_path(
        mut self,
        background: PathBuf,
    ) -> Result<Self, ParseError> {
        let background = Background::try_from(background)?;
        let _ = self.background.insert(background);
        Ok(self)
    }

    pub(crate) fn background(mut self, background: Background) -> Self {
        let _ = self.background.insert(background);
        self
    }

    pub(crate) fn text(mut self, text: impl Into<String>) -> Self {
        let _ = self.text.insert(text.into());
        self
    }

    pub(crate) fn text_color(mut self, text_color: impl Into<Color>) -> Self {
        let _ = self.text_color.insert(text_color.into());
        self
    }

    pub(crate) fn audio(mut self, audio: impl Into<PathBuf>) -> Self {
        let _ = self.audio.insert(audio.into());
        self
    }

    pub(crate) fn font(mut self, font: impl Into<Font>) -> Self {
        let _ = self.font.insert(font.into());
        self
    }

    pub(crate) fn font_size(mut self, font_size: i32) -> Self {
        let _ = self.font_size.insert(font_size);
        self
    }

    pub(crate) fn color(mut self, color: impl Into<Color>) -> Self {
        let _ = self.text_color.insert(color.into());
        self
    }

    pub(crate) fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        let _ = self.stroke.insert(stroke.into());
        self
    }

    pub(crate) fn shadow(mut self, shadow: impl Into<Shadow>) -> Self {
        let _ = self.shadow.insert(shadow.into());
        self
    }

    pub(crate) fn text_alignment(mut self, text_alignment: TextAlignment) -> Self {
        let _ = self.text_alignment.insert(text_alignment);
        self
    }

    pub(crate) fn video_loop(mut self, video_loop: bool) -> Self {
        let _ = self.video_loop.insert(video_loop);
        self
    }

    pub(crate) fn video_start_time(mut self, video_start_time: f32) -> Self {
        let _ = self.video_start_time.insert(video_start_time);
        self
    }

    pub(crate) fn video_end_time(mut self, video_end_time: f32) -> Self {
        let _ = self.video_end_time.insert(video_end_time);
        self
    }

    pub(crate) fn text_svg(mut self, text_svg: impl Into<TextSvg>) -> Self {
        let _ = self.text_svg.insert(text_svg.into());
        self
    }

    pub(crate) fn pdf_page(mut self, pdf_page: Handle) -> Self {
        let _ = self.pdf_page.insert(pdf_page);
        self
    }

    pub(crate) fn pdf_index(mut self, pdf_index: impl Into<u32>) -> Self {
        let _ = self.pdf_index.insert(pdf_index.into());
        self
    }

    pub(crate) fn build(self) -> Result<Slide> {
        let Some(background) = self.background else {
            return Err(miette!("No background"));
        };
        let Some(text) = self.text else {
            return Err(miette!("No text"));
        };
        let Some(font_size) = self.font_size else {
            return Err(miette!("No font_size"));
        };
        let Some(text_alignment) = self.text_alignment else {
            return Err(miette!("No text_alignment"));
        };
        let Some(video_loop) = self.video_loop else {
            return Err(miette!("No video_loop"));
        };
        let Some(video_start_time) = self.video_start_time else {
            return Err(miette!("No video_start_time"));
        };
        let Some(video_end_time) = self.video_end_time else {
            return Err(miette!("No video_end_time"));
        };
        let text_properties = TextProperties {
            text_alignment,
            font: self
                .font
                .as_ref()
                .map(super::super::ui::text_svg::Font::get_name),
            font_size: self.font_size.map(|size| size as f32),
            font_weight: self
                .font
                .as_ref()
                .map(super::super::ui::text_svg::Font::get_weight),
            font_style: self
                .font
                .as_ref()
                .map(super::super::ui::text_svg::Font::get_style),
            font_stretch: None,
            text_color: self.text_color.as_ref().map(|color| color.0),
            stroke_size: self.stroke.as_ref().map(|stroke| f32::from(stroke.size)),
            stroke_color: self.stroke.as_ref().map(|stroke| stroke.color.0),
            shadow_size: self.shadow.as_ref().map(|shadow| f32::from(shadow.spread)),
            shadow_offset: self
                .shadow
                .as_ref()
                .map(|shadow| (f32::from(shadow.offset_x), f32::from(shadow.offset_y))),
            shadow_color: self.shadow.as_ref().map(|shadow| shadow.color.0),
        };
        let id = SlideId(Uuid::new_v4());
        Ok(Slide {
            id,
            background,
            text,
            text_properties: Some(text_properties),
            font: self.font,
            font_size,
            text_alignment,
            audio: self.audio,
            stroke: self.stroke,
            shadow: self.shadow,
            text_color: self.text_color,
            video_loop,
            video_start_time,
            video_end_time,
            text_svg: self.text_svg,
            pdf_index: self.pdf_index.unwrap_or_default(),
            pdf_page: self.pdf_page,
            ..Default::default()
        })
    }
}

const SLIDE_WIDTH: f32 = 1920.0;
const SLIDE_HEIGHT: f32 = 1080.0;

#[allow(clippy::too_many_lines)]
pub fn build_text(
    font_system: &mut FontSystem,
    text: impl AsRef<str>,
    text_props: &TextProperties,
) -> StyledText {
    let bounds = Rectangle::with_size(Size::new(SLIDE_WIDTH, SLIDE_HEIGHT));
    // Guard against font_size not being set. This prevents an assertion
    // panic in cosmic_text
    let font_size = text_props.font_size.unwrap_or(100.0);
    let font_size = if font_size == 0.0 { 100.0 } else { font_size };
    let metrics = Metrics::relative(font_size, 1.3);

    let mut attrs = Attrs::new()
        .family(
            text_props
                .font
                .as_ref()
                .map_or(Family::SansSerif, |font| Family::Name(font)),
        )
        .metrics(metrics);
    let default_attrs = attrs.clone();
    let mut text_builder = StyledTextBuilder::new(
        TextRegion::new(bounds.width, bounds.height),
        metrics,
        &default_attrs,
    );
    if let Some(style) = text_props.font_style {
        let style = match style {
            iced_core::font::Style::Normal => fontdb::Style::Normal,
            iced_core::font::Style::Italic => fontdb::Style::Italic,
            iced_core::font::Style::Oblique => fontdb::Style::Oblique,
        };
        attrs = attrs.style(style);
    }

    if let Some(weight) = text_props.font_weight {
        let weight = match weight {
            iced_core::font::Weight::Thin => fontdb::Weight::THIN,
            iced_core::font::Weight::ExtraLight => fontdb::Weight::EXTRA_LIGHT,
            iced_core::font::Weight::Light => fontdb::Weight::LIGHT,
            iced_core::font::Weight::Normal => fontdb::Weight::NORMAL,
            iced_core::font::Weight::Medium => fontdb::Weight::MEDIUM,
            iced_core::font::Weight::Semibold => fontdb::Weight::SEMIBOLD,
            iced_core::font::Weight::Bold => fontdb::Weight::BOLD,
            iced_core::font::Weight::ExtraBold => fontdb::Weight::EXTRA_BOLD,
            iced_core::font::Weight::Black => fontdb::Weight::BLACK,
        };
        attrs = attrs.weight(weight);
    }

    if let Some(stretch) = text_props.font_stretch {
        let stretch = match stretch {
            iced_core::font::Stretch::UltraCondensed => fontdb::Stretch::UltraCondensed,
            iced_core::font::Stretch::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
            iced_core::font::Stretch::Condensed => fontdb::Stretch::Condensed,
            iced_core::font::Stretch::SemiCondensed => fontdb::Stretch::SemiCondensed,
            iced_core::font::Stretch::Normal => fontdb::Stretch::Normal,
            iced_core::font::Stretch::SemiExpanded => fontdb::Stretch::SemiExpanded,
            iced_core::font::Stretch::Expanded => fontdb::Stretch::Expanded,
            iced_core::font::Stretch::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
            iced_core::font::Stretch::UltraExpanded => fontdb::Stretch::UltraExpanded,
        };
        attrs = attrs.stretch(stretch);
    }

    let color = text_props.text_color.unwrap_or_default();
    let color = FdsmColor::rgb(color.red, color.green, color.blue);
    let stroke_color = text_props.stroke_color.unwrap_or_default();
    let stroke_color =
        FdsmColor::rgb(stroke_color.red, stroke_color.green, stroke_color.blue);
    let stroke_width = text_props.stroke_size.unwrap_or_default();
    let shadow_color = text_props.shadow_color.unwrap_or_default();
    let shadow_color = GlyphColoring::Solid(FdsmColor::rgb(
        shadow_color.red,
        shadow_color.green,
        shadow_color.blue,
    ));
    let shadow_direction = text_props.shadow_offset.unwrap_or_default();
    let shadow_size = text_props.shadow_size.unwrap_or_default();

    let style = TextStyle {
        color: GlyphColoring::Solid(color),
        stroke_out: fdsm_lumina::style::Outline {
            color: GlyphColoring::Solid(stroke_color),
            width: stroke_width * 2.0,
            roundness: 1.0,
        },
        stroke_in: fdsm_lumina::style::Outline {
            color: GlyphColoring::Solid(FdsmColor::TRANSPARENT),
            width: 0.0,
            roundness: 1.0,
        },
        glow_out: Glow::default(),
        glow_in: Glow::default(),
        shadow: fdsm_lumina::style::Shadow {
            color: shadow_color,
            direction: shadow_direction.into(),
            width: shadow_size * 2.0,
            spread: 0.1,
            softness: shadow_size * 2.0,
            roundness: 1.0,
        },
    };
    // let attrs = Attrs::new();
    text_builder.set_align(Some(text_props.text_alignment.into()));
    text_builder.push_text(text.as_ref(), &attrs, style);
    // tracing::warn!("RUNNING");
    let mut text = text_builder.finish(font_system);
    let text_height = text.height();

    let offset_y = match text_props.text_alignment {
        TextAlignment::TopLeft | TextAlignment::TopCenter | TextAlignment::TopRight => {
            0.0
        }

        TextAlignment::MiddleLeft
        | TextAlignment::MiddleCenter
        | TextAlignment::MiddleRight => (bounds.height - text_height) / 2.0,

        TextAlignment::BottomLeft
        | TextAlignment::BottomCenter
        | TextAlignment::BottomRight => bounds.height - text_height,
    };

    text.set_vertical_offset(offset_y.max(0.0));

    text
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::fs::read_to_string;

    use super::*;

    fn test_slide() -> Slide {
        Slide {
            text: "This is frodo".to_string(),
            background: Background::try_from("~/pics/frodo.jpg").expect(""),
            font: Some("Quicksand".to_string().into()),
            font_size: 140,
            ..Default::default()
        }
    }

    fn test_second_slide() -> Slide {
        Slide {
            text: String::new(),
            background: Background::try_from("~/vids/test/camprules2024.mp4").expect(""),
            font: Some("Quicksand".to_string().into()),
            ..Default::default()
        }
    }

    #[test]
    fn ron_deserialize() {
        let slide =
            read_to_string("./test_presentation.ron").expect("Problem getting file read");
        if let Err(e) = ron::from_str::<Vec<Slide>>(&slide) {
            panic!("{e:?}")
        }
    }
}
