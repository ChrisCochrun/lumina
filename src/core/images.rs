use crate::{Background, Slide, SlideBuilder, TextAlignment};

use super::{
    content::Content,
    kinds::ServiceItemKind,
    model::{get_db, LibraryKind, Model},
    service_items::ServiceTrait,
};
use crisp::types::{Keyword, Symbol, Value};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, SqliteConnection};
use std::path::PathBuf;
use tracing::error;

#[derive(
    Clone, Debug, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Image {
    pub id: i32,
    pub title: String,
    pub path: PathBuf,
}

impl From<&Image> for Value {
    fn from(value: &Image) -> Self {
        Self::List(vec![Self::Symbol(Symbol("image".into()))])
    }
}

impl Content for Image {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn kind(&self) -> ServiceItemKind {
        ServiceItemKind::Image(self.clone())
    }

    fn to_service_item(&self) -> super::service_items::ServiceItem {
        self.into()
    }

    fn background(&self) -> Option<Background> {
        if let Ok(background) =
            Background::try_from(self.path.clone())
        {
            Some(background)
        } else {
            None
        }
    }
}

impl From<Value> for Image {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

impl From<&Value> for Image {
    fn from(value: &Value) -> Self {
        match value {
            Value::List(list) => {
                let path = if let Some(path_pos) =
                    list.iter().position(|v| {
                        v == &Value::Keyword(Keyword::from("source"))
                    }) {
                    let pos = path_pos + 1;
                    list.get(pos)
                        .map(|p| PathBuf::from(String::from(p)))
                } else {
                    None
                };

                let title = path.clone().map(|p| {
                    let path =
                        p.to_str().unwrap_or_default().to_string();
                    let title =
                        path.rsplit_once("/").unwrap_or_default().1;
                    title.to_string()
                });
                Self {
                    title: title.unwrap_or_default(),
                    path: path.unwrap_or_default(),
                    ..Default::default()
                }
            }
            _ => todo!(),
        }
    }
}

impl ServiceTrait for Image {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn to_slides(&self) -> Result<Vec<Slide>> {
        let slide = SlideBuilder::new()
            .background(
                Background::try_from(self.path.clone())
                    .into_diagnostic()?,
            )
            .text("")
            .audio("")
            .font("")
            .font_size(50)
            .text_alignment(TextAlignment::MiddleCenter)
            .video_loop(false)
            .video_start_time(0.0)
            .video_end_time(0.0)
            .build()?;

        Ok(vec![slide])
    }

    fn box_clone(&self) -> Box<dyn ServiceTrait> {
        Box::new((*self).clone())
    }
}

impl Model<Image> {
    pub async fn new_image_model(db: &mut SqliteConnection) -> Self {
        let mut model = Self {
            items: vec![],
            kind: LibraryKind::Image,
        };

        model.load_from_db(db).await;
        model
    }

    pub async fn load_from_db(&mut self, db: &mut SqliteConnection) {
        let result = query_as!(
            Image,
            r#"SELECT title as "title!", file_path as "path!", id as "id: i32" from images"#
        )
            .fetch_all(db)
            .await;
        match result {
            Ok(v) => {
                for image in v.into_iter() {
                    let _ = self.add_item(image);
                }
            }
            Err(e) => {
                error!("There was an error in converting images: {e}")
            }
        };
    }
}

pub async fn get_image_from_db(
    database_id: i32,
    db: &mut SqliteConnection,
) -> Result<Image> {
    query_as!(Image, r#"SELECT title as "title!", file_path as "path!", id as "id: i32" from images where id = ?"#, database_id).fetch_one(db).await.into_diagnostic()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    fn test_image(title: String) -> Image {
        Image {
            title,
            path: PathBuf::from("~/pics/camprules2024.mp4"),
            ..Default::default()
        }
    }

    #[tokio::test]
    pub async fn test_db_and_model() {
        let mut image_model: Model<Image> = Model {
            items: vec![],
            db: crate::core::model::get_db().await,
        };
        image_model.load_from_db().await;
        if let Some(image) = image_model.find(|i| i.id == 3) {
            let test_image = test_image("nccq5".into());
            assert_eq!(test_image.title, image.title);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    pub async fn test_add_image() {
        let image = test_image("A new image".into());
        let mut image_model: Model<Image> = Model {
            items: vec![],
            db: crate::core::model::get_db().await,
        };
        let result = image_model.add_item(image.clone());
        let new_image = test_image("A newer image".into());
        match result {
            Ok(_) => {
                assert_eq!(
                    &image,
                    image_model.find(|i| i.id == 0).unwrap()
                );
                assert_ne!(
                    &new_image,
                    image_model.find(|i| i.id == 0).unwrap()
                );
            }
            Err(e) => assert!(
                false,
                "There was an error adding the image: {:?}",
                e
            ),
        }
    }
}
