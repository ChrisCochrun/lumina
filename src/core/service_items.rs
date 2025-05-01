use std::borrow::Cow;
use std::ops::Deref;

use cosmic::iced::clipboard::mime::{AllowedMimeTypes, AsMimeTypes};
use crisp::types::{Keyword, Symbol, Value};
use miette::Result;
use tracing::{debug, error};

use crate::Slide;

use super::images::Image;
use super::presentations::Presentation;
use super::songs::{lisp_to_song, Song};
use super::videos::Video;

use super::kinds::ServiceItemKind;

#[derive(Debug, PartialEq, Clone)]
pub struct ServiceItem {
    pub id: i32,
    pub title: String,
    pub database_id: i32,
    pub kind: ServiceItemKind,
    // pub item: Box<dyn ServiceTrait>,
}

impl TryFrom<(Vec<u8>, String)> for ServiceItem {
    type Error = miette::Error;

    fn try_from(
        value: (Vec<u8>, String),
    ) -> std::result::Result<Self, Self::Error> {
        let sto = value.0.to_owned();
        let song = Song {
            title: "Death Was Arrested".to_string(),
            ..Default::default()
        };
        debug!(?value);
        Ok(Self::from(&song))
    }
}

impl AllowedMimeTypes for ServiceItem {
    fn allowed() -> Cow<'static, [String]> {
        Cow::from(vec!["application/service-item".to_string()])
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
        let val = Value::from(self);
        let val = String::from(val);
        Some(Cow::from(val.into_bytes()))
    }
}

impl From<&ServiceItem> for Value {
    fn from(value: &ServiceItem) -> Self {
        match &value.kind {
            ServiceItemKind::Song(song) => Value::from(song),
            ServiceItemKind::Video(video) => Value::from(video),
            ServiceItemKind::Image(image) => Value::from(image),
            ServiceItemKind::Presentation(presentation) => {
                Value::from(presentation)
            }
            ServiceItemKind::Content(slide) => Value::from(slide),
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
            // item: Box::new(Image::default()),
        }
    }
}

impl From<Value> for ServiceItem {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

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
                    if let Some(content) =
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
                            kind: ServiceItemKind::Content(slide),
                        }
                    } else if let Some(background) =
                        list.get(background_pos)
                    {
                        match background {
                            Value::List(item) => match &item[0] {
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
                                    Self::from(
                                        &Presentation::from(
                                            background,
                                        ),
                                    )
                                }
                                _ => todo!(),
                            },
                            _ => {
                                error!(
                                    "There is no background here: {:?}",
                                    background
                                );
                                ServiceItem::default()
                            }
                        }
                    } else {
                        error!(
                            "There is no background here: {:?}",
                            background_pos
                        );
                        ServiceItem::default()
                    }
                }
                Value::Symbol(Symbol(s)) if s == "song" => {
                    let song = lisp_to_song(list.clone());
                    Self::from(&song)
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ServiceItemModel {
    items: Vec<ServiceItem>,
}

impl Deref for ServiceItemModel {
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

impl From<Vec<ServiceItem>> for ServiceItemModel {
    fn from(items: Vec<ServiceItem>) -> Self {
        Self { items }
    }
}

impl From<&Song> for ServiceItem {
    fn from(song: &Song) -> Self {
        Self {
            kind: ServiceItemKind::Song(song.clone()),
            database_id: song.id,
            title: song.title.clone(),
            ..Default::default()
        }
    }
}

impl From<&Video> for ServiceItem {
    fn from(video: &Video) -> Self {
        Self {
            kind: ServiceItemKind::Video(video.clone()),
            database_id: video.id,
            title: video.title.clone(),
            ..Default::default()
        }
    }
}

impl From<&Image> for ServiceItem {
    fn from(image: &Image) -> Self {
        Self {
            kind: ServiceItemKind::Image(image.clone()),
            database_id: image.id,
            title: image.title.clone(),
            ..Default::default()
        }
    }
}

impl From<&Presentation> for ServiceItem {
    fn from(presentation: &Presentation) -> Self {
        Self {
            kind: ServiceItemKind::Presentation(presentation.clone()),
            database_id: presentation.id,
            title: presentation.title.clone(),
            ..Default::default()
        }
    }
}

impl ServiceItemModel {
    fn add_item(
        &mut self,
        item: impl Into<ServiceItem>,
    ) -> Result<()> {
        let service_item: ServiceItem = item.into();
        self.items.push(service_item);
        Ok(())
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
            slide.set_index(index as i32);
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
        let mut service_model = ServiceItemModel::default();
        match service_model.add_item(&song) {
            Ok(_) => {
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
            Err(e) => panic!("Problem adding item: {:?}", e),
        }
    }
}
