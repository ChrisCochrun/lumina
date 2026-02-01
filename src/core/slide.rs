use cosmic::{
    cosmic_theme::palette::rgb::Rgba, widget::image::Handle,
};
// use cosmic::dialog::ashpd::url::Url;
use crisp::types::{Keyword, Symbol, Value};
use iced_video_player::Video;
use miette::{Result, miette};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use tracing::error;

use crate::ui::text_svg::TextSvg;

use super::songs::Song;

#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Slide {
    id: i32,
    pub(crate) background: Background,
    text: String,
    font: String,
    font_size: i32,
    stroke_size: i32,
    stroke_color: Rgba,
    text_alignment: TextAlignment,
    audio: Option<PathBuf>,
    video_loop: bool,
    video_start_time: f32,
    video_end_time: f32,
    pdf_index: u32,
    #[serde(skip)]
    pdf_page: Option<Handle>,
    #[serde(skip)]
    pub text_svg: Option<TextSvg>,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum BackgroundKind {
    #[default]
    Image,
    Video,
    Pdf,
    Html,
}

#[derive(Debug, Clone, Default)]
struct Image {
    pub source: String,
    pub fit: String,
    pub children: Vec<String>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Hash,
)]
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

#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct Background {
    pub path: PathBuf,
    pub kind: BackgroundKind,
}

impl TryFrom<&Background> for Video {
    type Error = ParseError;

    fn try_from(
        value: &Background,
    ) -> std::result::Result<Self, Self::Error> {
        Self::new(
            &url::Url::from_file_path(value.path.clone())
                .map_err(|()| ParseError::BackgroundNotVideo)?,
        )
        .map_err(|_| ParseError::BackgroundNotVideo)
    }
}

impl TryFrom<Background> for Video {
    type Error = ParseError;

    fn try_from(
        value: Background,
    ) -> std::result::Result<Self, Self::Error> {
        Self::new(
            &url::Url::from_file_path(value.path)
                .map_err(|()| ParseError::BackgroundNotVideo)?,
        )
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
            let path = path.to_str().unwrap().to_string();
            let path = path.trim_start_matches("file://");
            let home = dirs::home_dir()
                .unwrap()
                .to_str()
                .unwrap()
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
                        path: value,
                        kind: BackgroundKind::Image,
                    }),
                    "mp4" | "mkv" | "webm" => Ok(Self {
                        path: value,
                        kind: BackgroundKind::Video,
                    }),
                    "pdf" => Ok(Self {
                        path: value,
                        kind: BackgroundKind::Pdf,
                    }),
                    "html" => Ok(Self {
                        path: value,
                        kind: BackgroundKind::Html,
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
            if let Some(home) = dirs::home_dir() {
                if let Some(home) = home.to_str() {
                    let value = value.replace('~', home);
                    Self::try_from(PathBuf::from(value))
                } else {
                    Self::try_from(PathBuf::from(value))
                }
            } else {
                Self::try_from(PathBuf::from(value))
            }
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
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let message = match self {
            Self::NonBackgroundFile => {
                "The file is not a recognized image or video type"
            }
            Self::DoesNotExist => "This file doesn't exist",
            Self::CannotCanonicalize => {
                "Could not canonicalize this file"
            }
            Self::BackgroundNotVideo => {
                "This background isn't a video"
            }
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
    pub fn set_text(mut self, text: impl AsRef<str>) -> Self {
        self.text = text.as_ref().into();
        self
    }

    pub fn with_text_svg(mut self, text_svg: TextSvg) -> Self {
        self.text_svg = Some(text_svg);
        self
    }

    pub fn set_font(mut self, font: impl AsRef<str>) -> Self {
        self.font = font.as_ref().into();
        self
    }

    pub const fn set_font_size(mut self, font_size: i32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn set_audio(mut self, audio: Option<PathBuf>) -> Self {
        self.audio = audio;
        self
    }

    pub const fn set_pdf_index(mut self, pdf_index: u32) -> Self {
        self.pdf_index = pdf_index;
        self
    }

    pub const fn background(&self) -> &Background {
        &self.background
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub const fn text_alignment(&self) -> TextAlignment {
        self.text_alignment
    }

    pub const fn font_size(&self) -> i32 {
        self.font_size
    }

    pub fn font(&self) -> String {
        self.font.clone()
    }

    pub const fn video_loop(&self) -> bool {
        self.video_loop
    }

    pub fn audio(&self) -> Option<PathBuf> {
        self.audio.clone()
    }

    pub fn pdf_page(&self) -> Option<Handle> {
        self.pdf_page.clone()
    }

    pub const fn pdf_index(&self) -> u32 {
        self.pdf_index
    }

    pub fn song_slides(song: &Song) -> Result<Vec<Self>> {
        let lyrics = song.get_lyrics()?;
        let slides: Vec<Self> = lyrics
            .iter()
            .map(|l| {
                let song = song.clone();
                SlideBuilder::new()
                    .background(song.background.unwrap_or_default())
                    .font(song.font.unwrap_or_default())
                    .font_size(song.font_size.unwrap_or_default())
                    .text_alignment(
                        song.text_alignment.unwrap_or_default(),
                    )
                    .audio(song.audio.unwrap_or_default())
                    .video_loop(true)
                    .video_start_time(0.0)
                    .video_end_time(0.0)
                    .text(l)
                    .build()
                    .unwrap_or_default()
            })
            .collect();

        Ok(slides)
    }

    pub(crate) const fn set_index(&mut self, index: i32) {
        self.id = index;
    }

    pub(crate) fn text_to_image(&self) {
        todo!()
    }

    // pub fn slides_from_item(item: &ServiceItem) -> Result<Vec<Self>> {
    //     todo!()
    // }
}

impl From<Value> for Slide {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

impl From<&Value> for Slide {
    fn from(value: &Value) -> Self {
        match value {
            Value::List(list) => lisp_to_slide(list),
            _ => Self::default(),
        }
    }
}

fn lisp_to_slide(lisp: &Vec<Value>) -> Slide {
    const DEFAULT_BACKGROUND_LOCATION: usize = 1;
    const DEFAULT_TEXT_LOCATION: usize = 0;

    let mut slide = SlideBuilder::new();
    let background_position = if let Some(background) =
        lisp.iter().position(|v| {
            v == &Value::Keyword(Keyword::from("background"))
        }) {
        background + 1
    } else {
        DEFAULT_BACKGROUND_LOCATION
    };

    if let Some(background) = lisp.get(background_position) {
        slide = slide.background(lisp_to_background(background));
    } else {
        slide = slide.background(Background::default());
    }

    let text_position = lisp.iter().position(|v| match v {
        Value::List(vec) => {
            vec[DEFAULT_TEXT_LOCATION]
                == Value::Symbol(Symbol::from("text"))
        }
        _ => false,
    });

    if let Some(text_position) = text_position {
        if let Some(text) = lisp.get(text_position) {
            slide = slide.text(lisp_to_text(text));
        } else {
            slide = slide.text("");
        }
    } else {
        slide = slide.text("");
    }

    if let Some(text_position) = text_position {
        if let Some(text) = lisp.get(text_position) {
            slide = slide.font_size(lisp_to_font_size(text));
        } else {
            slide = slide.font_size(0);
        }
    } else {
        slide = slide.font_size(0);
    }

    slide = slide
        .font("Quicksand")
        .text_alignment(TextAlignment::MiddleCenter)
        .video_loop(false)
        .video_start_time(0.0)
        .video_end_time(0.0);

    match slide.build() {
        Ok(slide) => slide,
        Err(e) => {
            error!("Shoot! Slide didn't build: {e}");
            Slide::default()
        }
    }
}

fn lisp_to_font_size(lisp: &Value) -> i32 {
    match lisp {
        Value::List(list) => {
            if let Some(font_size_position) =
                list.iter().position(|v| {
                    v == &Value::Keyword(Keyword::from("font-size"))
                })
            {
                if let Some(font_size_value) =
                    list.get(font_size_position + 1)
                {
                    font_size_value.into()
                } else {
                    50
                }
            } else {
                50
            }
        }
        _ => 50,
    }
}

fn lisp_to_text(lisp: &Value) -> impl Into<String> {
    match lisp {
        Value::List(list) => list[1].clone(),
        _ => "".into(),
    }
}

// Need to return a Result here so that we can propogate
// errors and then handle them appropriately
pub fn lisp_to_background(lisp: &Value) -> Background {
    match lisp {
        Value::List(list) => {
            let _kind = list[0].clone();
            if let Some(source) = list.iter().position(|v| {
                v == &Value::Keyword(Keyword::from("source"))
            }) {
                let source = &list[source + 1];
                match source {
                    Value::String(s) => {
                        if s.starts_with("./") {
                            let Some(home) = dirs::home_dir() else {
                                panic!("Should always be there");
                            };
                            let Some(home) = home.to_str() else {
                                panic!("Should always be there");
                            };
                            let mut home = home.to_string();
                            home.push('/');

                            let s = s.replace("./", &home);
                            match Background::try_from(s.as_str()) {
                                Ok(background) => background,
                                Err(e) => {
                                    error!(
                                        "Couldn't load background: {e}"
                                    );
                                    Background::default()
                                }
                            }
                        } else {
                            match Background::try_from(s.as_str()) {
                                Ok(background) => background,
                                Err(e) => {
                                    error!(
                                        "Couldn't load background: {e}"
                                    );
                                    Background::default()
                                }
                            }
                        }
                    }
                    _ => Background::default(),
                }
            } else {
                Background::default()
            }
        }
        _ => Background::default(),
    }
}

#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize,
)]
pub struct SlideBuilder {
    background: Option<Background>,
    text: Option<String>,
    font: Option<String>,
    font_size: Option<i32>,
    audio: Option<PathBuf>,
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

    pub(crate) fn background(
        mut self,
        background: Background,
    ) -> Self {
        let _ = self.background.insert(background);
        self
    }

    pub(crate) fn text(mut self, text: impl Into<String>) -> Self {
        let _ = self.text.insert(text.into());
        self
    }

    pub(crate) fn audio(mut self, audio: impl Into<PathBuf>) -> Self {
        let _ = self.audio.insert(audio.into());
        self
    }

    pub(crate) fn font(mut self, font: impl Into<String>) -> Self {
        let _ = self.font.insert(font.into());
        self
    }

    pub(crate) fn font_size(mut self, font_size: i32) -> Self {
        let _ = self.font_size.insert(font_size);
        self
    }

    pub(crate) fn text_alignment(
        mut self,
        text_alignment: TextAlignment,
    ) -> Self {
        let _ = self.text_alignment.insert(text_alignment);
        self
    }

    pub(crate) fn video_loop(mut self, video_loop: bool) -> Self {
        let _ = self.video_loop.insert(video_loop);
        self
    }

    pub(crate) fn video_start_time(
        mut self,
        video_start_time: f32,
    ) -> Self {
        let _ = self.video_start_time.insert(video_start_time);
        self
    }

    pub(crate) fn video_end_time(
        mut self,
        video_end_time: f32,
    ) -> Self {
        let _ = self.video_end_time.insert(video_end_time);
        self
    }

    pub(crate) fn text_svg(
        mut self,
        text_svg: impl Into<TextSvg>,
    ) -> Self {
        let _ = self.text_svg.insert(text_svg.into());
        self
    }

    pub(crate) fn pdf_page(mut self, pdf_page: Handle) -> Self {
        let _ = self.pdf_page.insert(pdf_page);
        self
    }

    pub(crate) fn pdf_index(
        mut self,
        pdf_index: impl Into<u32>,
    ) -> Self {
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
        let Some(font) = self.font else {
            return Err(miette!("No font"));
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
        Ok(Slide {
            background,
            text,
            font,
            font_size,
            text_alignment,
            audio: self.audio,
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

impl Image {
    fn new() -> Self {
        Default::default()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::fs::read_to_string;

    use super::*;

    fn test_slide() -> Slide {
        Slide {
            text: "This is frodo".to_string(),
            background: Background::try_from("~/pics/frodo.jpg")
                .unwrap(),
            font: "Quicksand".to_string(),
            font_size: 140,
            ..Default::default()
        }
    }

    fn test_second_slide() -> Slide {
        Slide {
            text: "".to_string(),
            background: Background::try_from(
                "~/vids/test/camprules2024.mp4",
            )
            .unwrap(),
            font: "Quicksand".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_lisp_serialize() {
        let lisp =
            read_to_string("./test_presentation.lisp").expect("oops");
        let lisp_value = crisp::reader::read(&lisp);
        match lisp_value {
            Value::List(value) => {
                let slide = Slide::from(value[0].clone());
                let test_slide = test_slide();
                assert_eq!(slide, test_slide);

                let second_slide = Slide::from(value[1].clone());
                let second_test_slide = test_second_slide();
                assert_eq!(second_slide, second_test_slide)
            }
            _ => panic!("this should be a lisp"),
        }
    }

    #[test]
    fn test_ron_deserialize() {
        let slide = read_to_string("./test_presentation.ron")
            .expect("Problem getting file read");
        match ron::from_str::<Vec<Slide>>(&slide) {
            Ok(_s) => {
                assert!(true)
            }
            Err(_e) => {
                assert!(false)
            }
        }
    }
}
