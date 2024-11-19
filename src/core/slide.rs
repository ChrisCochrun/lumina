use crisp::types::{Keyword, Value};
use miette::{miette, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::core::lisp::Symbol;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
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
        if value == Value::Symbol("middle-center".into()) {
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

impl TryFrom<String> for Background {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim_start_matches("file://");
        let path = PathBuf::from(value);
        if !path.exists() {
            return Err(ParseError::DoesNotExist);
        }
        let extension = value.rsplit_once('.').unwrap_or_default();
        match extension.1 {
            "jpg" | "png" | "webp" | "html" => Ok(Self {
                path,
                kind: BackgroundKind::Image,
            }),
            "mp4" | "mkv" | "webm" => Ok(Self {
                path,
                kind: BackgroundKind::Video,
            }),
            _ => Err(ParseError::NonBackgroundFile),
        }
    }
}

impl TryFrom<PathBuf> for Background {
    type Error = ParseError;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let extension = value
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        match extension {
            "jpg" | "png" | "webp" | "html" => Ok(Self {
                path: value,
                kind: BackgroundKind::Image,
            }),
            "mp4" | "mkv" | "webm" => Ok(Self {
                path: value,
                kind: BackgroundKind::Video,
            }),
            _ => Err(ParseError::NonBackgroundFile),
        }
    }
}

impl TryFrom<&str> for Background {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self::try_from(String::from(value))?)
    }
}

impl TryFrom<&Path> for Background {
    type Error = ParseError;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Ok(Self::try_from(PathBuf::from(value))?)
    }
}

#[derive(Debug)]
pub enum ParseError {
    NonBackgroundFile,
    DoesNotExist,
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
        };
        write!(f, "Error: {message}")
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum BackgroundKind {
    #[default]
    Image,
    Video,
}

impl From<String> for BackgroundKind {
    fn from(value: String) -> Self {
        if value == "image" {
            BackgroundKind::Image
        } else {
            BackgroundKind::Video
        }
    }
}

#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Slide {
    id: i32,
    background: Background,
    text: String,
    font: String,
    font_size: i32,
    text_alignment: TextAlignment,
    video_loop: bool,
    video_start_time: f32,
    video_end_time: f32,
}

impl Slide {
    pub fn background(&self) -> &Background {
        &self.background
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn font_size(&self) -> i32 {
        self.font_size
    }

    pub fn font(&self) -> String {
        self.font.clone()
    }

    pub fn video_loop(&self) -> bool {
        self.video_loop
    }
}

impl From<Value> for Slide {
    fn from(value: Value) -> Self {
        match value {
            Value::List(list) => lisp_to_slide(list),
            _ => Slide::default(),
        }
    }
}

fn lisp_to_slide(lisp: Vec<Value>) -> Slide {
    let mut slide = SlideBuilder::new();
    let background_position = if let Some(background) =
        lisp.position(|v| Value::Keyword(Keyword::from("background")))
    {
        background + 1
    } else {
        0
    };

    if let Some(background) = lisp.get(background_position) {
        slide.background(lisp_to_background(background));
    } else {
        slide.background(Background::default());
    };
    match slide.build() {
        Ok(slide) => slide,
        Err(e) => {
            miette!("Shoot! Slide didn't build: {e}");
            Slide::default()
        }
    }
}

fn lisp_to_background(lisp: &Value) -> Background {
    match lisp {
        Value::List(list) => {
            if let Some(source) = list.iter().position(|v| {
                v == Value::Keyword(Keyword::from("source"))
            }) {
                let path = list[source + 1];
                let path = String::from(path);
                Background::from(path)
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
    text_alignment: Option<TextAlignment>,
    video_loop: Option<bool>,
    video_start_time: Option<f32>,
    video_end_time: Option<f32>,
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
        self.background.insert(background);
        self
    }

    pub(crate) fn text(mut self, text: impl Into<String>) -> Self {
        let _ = self.text.insert(text.into());
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
            video_loop,
            video_start_time,
            video_end_time,
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, Default)]
struct Image {
    pub source: String,
    pub fit: String,
    pub children: Vec<String>,
}
impl Image {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

fn build_image_bg(
    atom: &Value,
    image_map: &mut HashMap<String, String>,
    map_index: usize,
) {
    // This needs to be the cons that contains (image . ...)
    // the image is a symbol and the rest are keywords and other maps
    if atom.is_symbol() {
        // We shouldn't get a symbol
        return;
    }

    for atom in
        atom.list_iter().unwrap().map(|a| a.as_cons().unwrap())
    {
        if atom.car() == &Value::Symbol("image".into()) {
            build_image_bg(atom.cdr(), image_map, map_index);
        } else {
            let atom = atom.car();
            match atom {
                Value::Keyword(keyword) => {
                    image_map.insert(keyword.to_string(), "".into());
                    build_image_bg(atom, image_map, map_index);
                }
                Value::Symbol(symbol) => {
                    // let mut key;
                    // let image_map = image_map
                    //     .iter_mut()
                    //     .enumerate()
                    //     .filter(|(i, e)| i == &map_index)
                    //     .map(|(i, (k, v))| v.push_str(symbol))
                    //     .collect();
                    build_image_bg(atom, image_map, map_index);
                }
                Value::String(string) => {}
                _ => {}
            }
        }
    }
}

// fn build_slide(exp: Value) -> Result<Slide> {
//     let mut slide_builder = SlideBuilder::new();
//     let mut keyword = "idk";
//     for value in exp.as_cons().unwrap().to_vec().0 {
//         let mut vecs = vec![vec![]];
//         match value {
//             Value::Symbol(symbol) => {}
//             Value::Keyword(keyword) => {}
//             Value::String(string) => {}
//             Value::Number(num) => {}
//             Value::Cons(cons) => {
//                 vecs.push(get_lists(&value));
//             }
//             _ => {}
//         }
//     }

//     todo!()
// }

fn build_slides(
    cons: &lexpr::Cons,
    mut current_symbol: Symbol,
    mut slide_builder: SlideBuilder,
) -> SlideBuilder {
    for value in cons.list_iter() {
        dbg!(&current_symbol);
        match value {
            Value::Cons(v) => {
                slide_builder = build_slides(
                    &v,
                    current_symbol.clone(),
                    slide_builder,
                );
            }
            Value::Nil => {
                dbg!(Value::Nil);
            }
            Value::Bool(boolean) => {
                dbg!(boolean);
            }
            Value::Number(number) => {
                dbg!(number);
            }
            Value::String(string) => {
                dbg!(string);
            }
            Value::Symbol(symbol) => {
                dbg!(symbol);
                current_symbol =
                    Symbol::from_str(symbol).unwrap_or_default();
            }
            Value::Keyword(keyword) => {
                dbg!(keyword);
            }
            Value::Null => {
                dbg!("null");
            }
            Value::Char(c) => {
                dbg!(c);
            }
            Value::Bytes(b) => {
                dbg!(b);
            }
            Value::Vector(v) => {
                dbg!(v);
            }
        }
    }
    slide_builder
}

#[cfg(test)]
mod test {
    use lexpr::{parse::Options, Datum, Parser};
    use pretty_assertions::assert_eq;
    use serde_lexpr::from_str;
    use std::fs::read_to_string;
    use tracing::debug;

    use super::*;

    fn test_slide() -> Slide {
        Slide::default()
    }

    #[test]
    fn test_lexp_serialize() {
        let lisp =
            read_to_string("./test_presentation.lisp").expect("oops");
        println!("{lisp}");
        let mut parser =
            Parser::from_str_custom(&lisp, Options::elisp());
        for atom in parser.value_iter() {
            match atom {
                Ok(atom) => {
                    let symbol = Symbol::None;
                    let slide_builder = SlideBuilder::new();
                    atom.as_cons().map(|c| {
                        build_slides(c, symbol, slide_builder)
                    });
                }
                Err(e) => {
                    dbg!(e);
                }
            }
        }
        // parser.map(|atom| match atom {
        //     Ok(atom) => dbg!(atom),
        //     Err(e) => dbg!(e),
        // });
        // let lispy = from_str_elisp(&lisp).expect("oops");
        // if lispy.is_list() {
        //     for atom in lispy.list_iter().unwrap() {
        //         print_list(atom);
        //     }
        // }
        let slide: Slide = from_str(&lisp).expect("oops");
        let test_slide = test_slide();
        assert_eq!(slide, test_slide)
    }

    #[test]
    fn test_ron_deserialize() {
        let slide = read_to_string("./test_presentation.ron")
            .expect("Problem getting file read");
        match ron::from_str::<Vec<Slide>>(&slide) {
            Ok(s) => {
                dbg!(s);
                assert!(true)
            }
            Err(e) => {
                dbg!(e);
                assert!(false)
            }
        }
    }
}
