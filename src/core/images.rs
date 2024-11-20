use super::model::Model;
use miette::{miette, IntoDiagnostic, Result};
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

impl Model<Image> {
    pub async fn load_from_db(&mut self) {
        let result = query_as!(
            Image,
            r#"SELECT title as "title!", file_path as "path!", id as "id: i32" from images"#
        )
            .fetch_all(&mut self.db)
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
    Ok(query_as!(Image, r#"SELECT title as "title!", file_path as "path!", id as "id: i32" from images where id = ?"#, database_id).fetch_one(db).await.into_diagnostic()?)
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
