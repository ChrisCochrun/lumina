use crate::{Background, Slide, SlideBuilder, TextAlignment};

use super::{
    content::Content,
    kinds::ServiceItemKind,
    model::{LibraryKind, Model},
    service_items::ServiceTrait,
};
use crisp::types::{Keyword, Symbol, Value};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{
    Sqlite, SqliteConnection, SqlitePool, pool::PoolConnection,
    query, query_as,
};
use std::path::{Path, PathBuf};
use tracing::{debug, error};

#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct Image {
    pub id: i32,
    pub title: String,
    pub path: PathBuf,
}

impl From<PathBuf> for Image {
    fn from(value: PathBuf) -> Self {
        let title = value
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        Self {
            id: 0,
            title,
            path: value.canonicalize().unwrap_or(value),
        }
    }
}

impl From<&Path> for Image {
    fn from(value: &Path) -> Self {
        Self::from(value.to_owned())
    }
}

impl From<&Image> for Value {
    fn from(_value: &Image) -> Self {
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
        Background::try_from(self.path.clone()).ok()
    }

    fn subtext(&self) -> String {
        if self.path.exists() {
            self.path
                .file_name()
                .map_or("Missing image".into(), |f| {
                    f.to_string_lossy().to_string()
                })
        } else {
            "Missing image".into()
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
                        path.rsplit_once('/').unwrap_or_default().1;
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
    pub async fn new_image_model(db: &mut SqlitePool) -> Self {
        let mut model = Self {
            items: vec![],
            kind: LibraryKind::Image,
        };

        let mut db = db.acquire().await.expect("probs");

        model.load_from_db(&mut db).await;
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
                for image in v {
                    let _ = self.add_item(image);
                }
            }
            Err(e) => {
                error!(
                    "There was an error in converting images: {e}"
                );
            }
        }
    }
}

pub async fn remove_from_db(
    db: PoolConnection<Sqlite>,
    id: i32,
) -> Result<()> {
    query!("DELETE FROM images WHERE id = $1", id)
        .execute(&mut db.detach())
        .await
        .into_diagnostic()
        .map(|_| ())
}

pub async fn add_image_to_db(
    image: Image,
    db: PoolConnection<Sqlite>,
) -> Result<()> {
    let path = image
        .path
        .to_str()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let mut db = db.detach();
    query!(
        r#"INSERT INTO images (title, file_path) VALUES ($1, $2)"#,
        image.title,
        path,
    )
    .execute(&mut db)
    .await
    .into_diagnostic()?;
    Ok(())
}

pub async fn update_image_in_db(
    image: Image,
    db: PoolConnection<Sqlite>,
) -> Result<()> {
    let path = image
        .path
        .to_str()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let mut db = db.detach();
    debug!(?image, "should be been updated");
    let result = query!(
        r#"UPDATE images SET title = $2, file_path = $3 WHERE id = $1"#,
        image.id,
        image.title,
        path,
    )
        .execute(&mut db)
        .await.into_diagnostic();

    match result {
        Ok(_) => {
            debug!("should have been updated");
            Ok(())
        }
        Err(e) => {
            error! {?e};
            Err(e)
        }
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
            path: PathBuf::from(
                "/home/chris/pics/memes/no-i-dont-think.gif",
            ),
            ..Default::default()
        }
    }

    #[tokio::test]
    pub async fn test_db_and_model() {
        let mut image_model: Model<Image> = Model {
            items: vec![],
            kind: LibraryKind::Image,
        };
        let mut db = add_db().await.unwrap().acquire().await.unwrap();
        image_model.load_from_db(&mut db).await;
        if let Some(image) = image_model.find(|i| i.id == 23) {
            let test_image = test_image("no-i-dont-think.gif".into());
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
            kind: LibraryKind::Image,
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

    async fn add_db() -> Result<SqlitePool> {
        let db_url = String::from("sqlite://./test.db");
        SqlitePool::connect(&db_url).await.into_diagnostic()
    }
}
