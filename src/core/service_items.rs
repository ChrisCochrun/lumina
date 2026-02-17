use std::borrow::Cow;
use std::cmp::Ordering;
use std::ops::Deref;
use std::path::PathBuf;

use cosmic::iced::clipboard::mime::{AllowedMimeTypes, AsMimeTypes};
use crisp::types::{Keyword, Symbol, Value};
use miette::{IntoDiagnostic, Result, miette};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::Slide;

use super::images::Image;
use super::presentations::Presentation;
use super::songs::{Song, lisp_to_song};
use super::videos::Video;

use super::kinds::ServiceItemKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceItem {
    pub id: i32,
    pub title: String,
    pub database_id: i32,
    pub kind: ServiceItemKind,
    pub slides: Vec<Slide>,
    // pub item: Box<dyn ServiceTrait>,
}

impl Eq for ServiceItem {}

impl PartialOrd for ServiceItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ServiceItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl TryFrom<(Vec<u8>, String)> for ServiceItem {
    type Error = miette::Error;

    fn try_from(
        value: (Vec<u8>, String),
    ) -> std::result::Result<Self, Self::Error> {
        let (data, mime) = value;
        debug!(?mime);
        ron::de::from_bytes(&data).into_diagnostic()
    }
}

impl AllowedMimeTypes for ServiceItem {
    fn allowed() -> Cow<'static, [String]> {
        Cow::from(vec![
            "application/service-item".to_string(),
            "text/uri-list".to_string(),
            "x-special/gnome-copied-files".to_string(),
        ])
    }
}

impl AsMimeTypes for ServiceItem {
    fn available(&self) -> Cow<'static, [String]> {
        debug!(?self);
        Cow::from(vec!["application/service-item".to_string()])
    }

    fn as_bytes(
        &self,
        mime_type: &str,
    ) -> Option<std::borrow::Cow<'static, [u8]>> {
        debug!(?self);
        debug!(mime_type);
        let ron = ron::ser::to_string(self).ok()?;
        debug!(ron);
        Some(Cow::from(ron.into_bytes()))
    }
}

impl TryFrom<PathBuf> for ServiceItem {
    type Error = miette::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                miette::miette!(
                    "There isn't an extension on this file"
                )
            })?;
        match ext {
            "png" | "jpg" | "jpeg" => {
                Ok(Self::from(&Image::from(path)))
            }
            "mp4" | "mkv" | "webm" => {
                Ok(Self::from(&Video::from(path)))
            }
            _ => Err(miette!("Unkown service item")),
        }
    }
}

impl From<&ServiceItem> for Value {
    fn from(value: &ServiceItem) -> Self {
        match &value.kind {
            ServiceItemKind::Song(song) => Self::from(song),
            ServiceItemKind::Video(video) => Self::from(video),
            ServiceItemKind::Image(image) => Self::from(image),
            ServiceItemKind::Presentation(presentation) => {
                Self::from(presentation)
            }
            ServiceItemKind::Content(slide) => Self::from(slide),
        }
    }
}

impl ServiceItem {
    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn to_slides(&self) -> Result<Vec<Slide>> {
        match &self.kind {
            ServiceItemKind::Song(song) => song.to_slides(),
            ServiceItemKind::Video(video) => video.to_slides(),
            ServiceItemKind::Image(image) => image.to_slides(),
            ServiceItemKind::Presentation(presentation) => {
                presentation.to_slides()
            }
            ServiceItemKind::Content(slide) => {
                Ok(vec![slide.clone()])
            }
        }
    }
}

impl Default for ServiceItem {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::default(),
            database_id: 0,
            kind: ServiceItemKind::Content(Slide::default()),
            slides: vec![],
            // item: Box::new(Image::default()),
        }
    }
}

impl From<Value> for ServiceItem {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

#[allow(clippy::option_if_let_else)]
#[allow(clippy::match_like_matches_macro)]
impl From<&Value> for ServiceItem {
    fn from(value: &Value) -> Self {
        match value {
            Value::List(list) => match &list[0] {
                Value::Symbol(Symbol(s)) if s == "slide" => {
                    let background_pos = list
                        .iter()
                        .position(|v| match v {
                            Value::Keyword(Keyword(background))
                                if background == "background" =>
                            {
                                true
                            }
                            _ => false,
                        })
                        .map_or_else(|| 1, |pos| pos + 1);
                    if let Some(_content) =
                        list.iter().position(|v| match v {
                            Value::List(list)
                                if list.iter().next()
                                    == Some(&Value::Symbol(
                                        Symbol("text".into()),
                                    )) =>
                            {
                                list.iter().next().is_some()
                            }
                            _ => false,
                        })
                    {
                        let slide = Slide::from(value);
                        let title = slide.text();
                        Self {
                            id: 0,
                            title,
                            database_id: 0,
                            kind: ServiceItemKind::Content(
                                slide.clone(),
                            ),
                            slides: vec![slide],
                        }
                    } else if let Some(background) =
                        list.get(background_pos)
                    {
                        if let Value::List(item) = background {
                            match &item[0] {
                                Value::Symbol(Symbol(s))
                                    if s == "image" =>
                                {
                                    Self::from(&Image::from(
                                        background,
                                    ))
                                }
                                Value::Symbol(Symbol(s))
                                    if s == "video" =>
                                {
                                    Self::from(&Video::from(
                                        background,
                                    ))
                                }
                                Value::Symbol(Symbol(s))
                                    if s == "presentation" =>
                                {
                                    Self::from(&Presentation::from(
                                        background,
                                    ))
                                }
                                _ => todo!(),
                            }
                        } else {
                            error!(
                                "There is no background here: {:?}",
                                background
                            );
                            Self::default()
                        }
                    } else {
                        error!(
                            "There is no background here: {:?}",
                            background_pos
                        );
                        Self::default()
                    }
                }
                Value::Symbol(Symbol(s)) if s == "song" => {
                    let song = lisp_to_song(list.clone());
                    Self::from(&song)
                }
                _ => Self::default(),
            },
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Service {
    items: Vec<ServiceItem>,
}

impl Deref for Service {
    type Target = Vec<ServiceItem>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

// impl Iterator for ServiceItemModel {
//     type Item = ServiceItem;

//     fn next(&mut self) -> Option<Self::Item> {
//         *self.items.iter().next()
//     }
// }

impl From<Vec<ServiceItem>> for Service {
    fn from(items: Vec<ServiceItem>) -> Self {
        Self { items }
    }
}

impl From<&Song> for ServiceItem {
    fn from(song: &Song) -> Self {
        song.to_slides().map_or_else(
            |_| Self {
                kind: ServiceItemKind::Song(song.clone()),
                database_id: song.id,
                title: song.title.clone(),
                ..Default::default()
            },
            |slides| Self {
                kind: ServiceItemKind::Song(song.clone()),
                database_id: song.id,
                title: song.title.clone(),
                slides,
                ..Default::default()
            },
        )
    }
}

impl From<&Video> for ServiceItem {
    fn from(video: &Video) -> Self {
        video.to_slides().map_or_else(
            |_| Self {
                kind: ServiceItemKind::Video(video.clone()),
                database_id: video.id,
                title: video.title.clone(),
                ..Default::default()
            },
            |slides| Self {
                kind: ServiceItemKind::Video(video.clone()),
                database_id: video.id,
                title: video.title.clone(),
                slides,
                ..Default::default()
            },
        )
    }
}

impl From<&Image> for ServiceItem {
    fn from(image: &Image) -> Self {
        image.to_slides().map_or_else(
            |_| Self {
                kind: ServiceItemKind::Image(image.clone()),
                database_id: image.id,
                title: image.title.clone(),
                ..Default::default()
            },
            |slides| Self {
                kind: ServiceItemKind::Image(image.clone()),
                database_id: image.id,
                title: image.title.clone(),
                slides,
                ..Default::default()
            },
        )
    }
}

impl From<&Presentation> for ServiceItem {
    fn from(presentation: &Presentation) -> Self {
        match presentation.to_slides() {
            Ok(slides) => Self {
                kind: ServiceItemKind::Presentation(
                    presentation.clone(),
                ),
                database_id: presentation.id,
                title: presentation.title.clone(),
                slides,
                ..Default::default()
            },
            Err(e) => {
                error!(?e);
                Self {
                    kind: ServiceItemKind::Presentation(
                        presentation.clone(),
                    ),
                    database_id: presentation.id,
                    title: presentation.title.clone(),
                    ..Default::default()
                }
            }
        }
    }
}

#[allow(unused)]
impl Service {
    fn add_item(&mut self, item: impl Into<ServiceItem>) {
        let service_item: ServiceItem = item.into();
        self.items.push(service_item);
    }

    pub fn to_slides(&self) -> Result<Vec<Slide>> {
        let slides = self
            .items
            .iter()
            .filter_map(|item| {
                let slides = item.to_slides().ok();
                debug!(?slides);
                slides
            })
            .flatten()
            .collect::<Vec<Slide>>();
        let mut final_slides = vec![];
        for (index, mut slide) in slides.into_iter().enumerate() {
            slide.set_index(i32::try_from(index).into_diagnostic()?);
            final_slides.push(slide);
        }
        Ok(final_slides)
    }
}

pub trait ServiceTrait {
    fn title(&self) -> String;
    fn id(&self) -> i32;
    fn to_slides(&self) -> Result<Vec<Slide>>;
    fn box_clone(&self) -> Box<dyn ServiceTrait>;
}

impl Clone for Box<dyn ServiceTrait> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl std::fmt::Debug for Box<dyn ServiceTrait> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "{}: {}", self.id(), self.title())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::core::presentations::PresKind;

    use super::*;
    use pretty_assertions::assert_eq;

    fn test_song() -> Song {
        Song {
            title: "Death Was Arrested".to_string(),
            ..Default::default()
        }
    }

    fn test_presentation() -> Presentation {
        Presentation {
            id: 0,
            title: "20240327T133649--12-isaiah-and-jesus__lesson_project_tfc".into(),
            path: PathBuf::from(
                "~/docs/notes/lessons/20240327T133649--12-isaiah-and-jesus__lesson_project_tfc.html",
            ),
            kind: PresKind::Html,
        }
    }

    #[test]
    pub fn test_service_item() {
        let song = test_song();
        let service_item = ServiceItem::from(&song);
        let pres = test_presentation();
        let pres_item = ServiceItem::from(&pres);
        let mut service_model = Service::default();
        service_model.add_item(&song);
        assert_eq!(
            ServiceItemKind::Song(song),
            service_model.items[0].kind
        );
        assert_eq!(
            ServiceItemKind::Presentation(pres),
            pres_item.kind
        );
        assert_eq!(service_item, service_model.items[0]);
    }
}
