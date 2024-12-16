use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::Slide;

use super::{
    images::Image,
    presentations::Presentation,
    songs::Song,
    videos::Video,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceItemKind {
    Song(Song),
    Video(Video),
    Image(Image),
    Presentation(Presentation),
    Content(Slide),
}

impl std::fmt::Display for ServiceItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Song(s) => "song".to_owned(),
            Self::Image(i) => "image".to_owned(),
            Self::Video(v) => "video".to_owned(),
            Self::Presentation(p) => "html".to_owned(),
            Self::Content(s) => "content".to_owned(),
        };
        write!(f, "{s}")
    }
}

// impl TryFrom<String> for ServiceItemKind {
//     type Error = ParseError;
//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         match value.as_str() {
//             "song" => Ok(Self::Song),
//             "image" => Ok(Self::Image),
//             "video" => Ok(Self::Video),
//             "presentation" => {
//                 Ok(Self::Presentation(PresKind::Generic))
//             }
//             "html" => Ok(Self::Presentation(PresKind::Html)),
//             "pdf" => Ok(Self::Presentation(PresKind::Pdf)),
//             "content" => Ok(Self::Content),
//             _ => Err(ParseError::UnknownType),
//         }
//     }
// }

impl From<ServiceItemKind> for String {
    fn from(val: ServiceItemKind) -> String {
        match val {
            ServiceItemKind::Song(_) => "song".to_owned(),
            ServiceItemKind::Video(_) => "video".to_owned(),
            ServiceItemKind::Image(_) => "image".to_owned(),
            ServiceItemKind::Presentation(_) => {
                "presentation".to_owned()
            }
            ServiceItemKind::Content(_) => "content".to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnknownType,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let message = match self {
            Self::UnknownType => "The type does not exist. It needs to be one of 'song', 'video', 'image', 'presentation', or 'content'",
        };
        write!(f, "Error: {message}")
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_kinds() {
        assert_eq!(true, true)
    }
}
