use std::{error::Error, fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    Slide,
    core::{content::Content, service_items::ServiceItem},
};

use super::{
    images::Image, presentations::Presentation, songs::Song,
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

impl TryFrom<PathBuf> for ServiceItemKind {
    type Error = miette::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or(miette::miette!(
                "There isn't an extension on this file"
            ))?;
        match ext {
            "png" | "jpg" | "jpeg" => {
                Ok(Self::Image(Image::from(path)))
            }
            "mp4" | "mkv" | "webm" => {
                Ok(Self::Video(Video::from(path)))
            }
            _ => Err(miette::miette!("Unknown item")),
        }
    }
}

impl ServiceItemKind {
    pub fn title(&self) -> String {
        match self {
            ServiceItemKind::Song(song) => song.title.to_string(),
            ServiceItemKind::Video(video) => video.title.to_string(),
            ServiceItemKind::Image(image) => image.title.to_string(),
            ServiceItemKind::Presentation(presentation) => {
                presentation.title.to_string()
            }
            ServiceItemKind::Content(_slide) => todo!(),
        }
    }

    pub fn to_service_item(&self) -> ServiceItem {
        match self {
            ServiceItemKind::Song(song) => song.to_service_item(),
            ServiceItemKind::Video(video) => video.to_service_item(),
            ServiceItemKind::Image(image) => image.to_service_item(),
            ServiceItemKind::Presentation(presentation) => {
                presentation.to_service_item()
            }
            ServiceItemKind::Content(_slide) => {
                todo!()
            }
        }
    }
}

impl std::fmt::Display for ServiceItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Song(_) => "song".to_owned(),
            Self::Image(_) => "image".to_owned(),
            Self::Video(_) => "video".to_owned(),
            Self::Presentation(_) => "html".to_owned(),
            Self::Content(_) => "content".to_owned(),
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
    fn from(val: ServiceItemKind) -> Self {
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
            Self::UnknownType => {
                "The type does not exist. It needs to be one of 'song', 'video', 'image', 'presentation', or 'content'"
            }
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
