use crate::model::Model;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, SqliteConnection};
use std::path::PathBuf;
use tracing::error;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub id: i32,
    pub title: String,
    pub path: PathBuf,
}

impl Model<Image> {
    pub fn load_from_db(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = query_as!(Image, r#"SELECT title as "title!", filePath as "path!", id as "id: i32" from images"#).fetch_all(&mut self.db).await;
            match result {
                Ok(v) => {
                    for image in v.into_iter() {
                        let _ = self.add_item(image);
                    }
                }
                Err(e) => error!("There was an error in converting images: {e}"),
            }
        });
    }
}


pub async fn get_image_from_db(database_id: i32, db: &mut SqliteConnection) -> Result<Image> {
    Ok(query_as!(Image, r#"SELECT title as "title!", filePath as "path!", id as "id: i32" from images where id = ?"#, database_id).fetch_one(db).await?)
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

    #[test]
    pub fn test_db_and_model() {
        let mut image_model: Model<Image> = Model::default();
        image_model.load_from_db();
        if let Some(image) = image_model.find(|i| i.id == 3) {
            let test_image = test_image("nccq5".into());
            assert_eq!(test_image.title, image.title);
        } else {
            assert!(false);
        }
    }

    #[test]
    pub fn test_add_image() {
        let image = test_image("A new image".into());
        let mut image_model: Model<Image> = Model::default();
        let result = image_model.add_item(image.clone());
        let new_image = test_image("A newer image".into());
        match result {
            Ok(_) => {
                assert_eq!(&image, image_model.find(|i| i.id == 0).unwrap());
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
