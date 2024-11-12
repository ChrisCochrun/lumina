use miette::Result;

use super::images::Image;
use super::presentations::Presentation;
use super::songs::Song;
use super::videos::Video;

use super::kinds::ServiceItemKind;

#[derive(Debug, Default, PartialEq)]
pub struct ServiceItem {
    pub id: i32,
    pub database_id: i32,
    pub kind: ServiceItemKind,
}

#[derive(Debug, Default, PartialEq)]
pub struct ServiceItemModel {
    items: Vec<ServiceItem>,
}

impl From<&Song> for ServiceItem {
    fn from(song: &Song) -> Self {
        Self {
            kind: ServiceItemKind::Song,
            database_id: song.id,
            ..Default::default()
        }
    }
}

impl From<&Video> for ServiceItem {
    fn from(video: &Video) -> Self {
        Self {
            kind: ServiceItemKind::Video,
            database_id: video.id,
            ..Default::default()
        }
    }
}

impl From<&Image> for ServiceItem {
    fn from(image: &Image) -> Self {
        Self {
            kind: ServiceItemKind::Image,
            database_id: image.id,
            ..Default::default()
        }
    }
}

impl From<&Presentation> for ServiceItem {
    fn from(presentation: &Presentation) -> Self {
        Self {
            kind: ServiceItemKind::Presentation(presentation.kind.clone()),
            database_id: presentation.id,
            ..Default::default()
        }
    }
}

impl ServiceItemModel {
    fn add_item(&mut self, item: impl Into<ServiceItem>) -> Result<()> {
        let service_item: ServiceItem = item.into();
        self.items.push(service_item);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::core::presentations::PresKind;

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

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
                assert_eq!(ServiceItemKind::Song, service_model.items[0].kind);
                assert_eq!(
                    ServiceItemKind::Presentation(PresKind::Html),
                    pres_item.kind
                );
                assert_eq!(service_item, service_model.items[0]);
            }
            Err(e) => panic!("Problem adding item: {:?}", e),
        }
    }
}
