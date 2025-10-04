use crate::{Background, SlideBuilder, TextAlignment};

use super::{
    content::Content,
    kinds::ServiceItemKind,
    model::{LibraryKind, Model},
    service_items::ServiceTrait,
    slide::Slide,
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
    Clone, Debug, Default, PartialEq, Serialize, Deserialize,
)]
pub struct Video {
    pub id: i32,
    pub title: String,
    pub path: PathBuf,
    pub start_time: Option<f32>,
    pub end_time: Option<f32>,
    pub looping: bool,
}

impl From<&Video> for Value {
    fn from(_value: &Video) -> Self {
        Self::List(vec![Self::Symbol(Symbol("video".into()))])
    }
}

impl From<PathBuf> for Video {
    fn from(value: PathBuf) -> Self {
        let title: String = value.file_name().map_or_else(
            || "Video".into(),
            |filename| filename.to_str().unwrap_or("Video").into(),
        );
        Self {
            title,
            path: value,
            ..Default::default()
        }
    }
}

impl From<&Path> for Video {
    fn from(value: &Path) -> Self {
        Self::from(value.to_owned())
    }
}

impl Content for Video {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn kind(&self) -> ServiceItemKind {
        ServiceItemKind::Video(self.clone())
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
                .map_or("Missing video".into(), |f| {
                    f.to_string_lossy().to_string()
                })
        } else {
            "Missing video".into()
        }
    }
}

impl From<Value> for Video {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

impl From<&Value> for Video {
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

                let start_time = if let Some(start_pos) =
                    list.iter().position(|v| {
                        v == &Value::Keyword(Keyword::from(
                            "start-time",
                        ))
                    }) {
                    let pos = start_pos + 1;
                    list.get(pos).map(|p| i32::from(p) as f32)
                } else {
                    None
                };

                let end_time = if let Some(end_pos) =
                    list.iter().position(|v| {
                        v == &Value::Keyword(Keyword::from(
                            "end-time",
                        ))
                    }) {
                    let pos = end_pos + 1;
                    list.get(pos).map(|p| i32::from(p) as f32)
                } else {
                    None
                };

                let looping = if let Some(loop_pos) =
                    list.iter().position(|v| {
                        v == &Value::Keyword(Keyword::from("loop"))
                    }) {
                    let pos = loop_pos + 1;
                    list.get(pos)
                        .is_some_and(|l| String::from(l) == *"true")
                } else {
                    false
                };

                Self {
                    title: title.unwrap_or_default(),
                    path: path.unwrap_or_default(),
                    start_time,
                    end_time,
                    looping,
                    ..Default::default()
                }
            }
            _ => todo!(),
        }
    }
}

impl ServiceTrait for Video {
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
            .video_loop(self.looping)
            .video_start_time(self.start_time.unwrap_or(0.0))
            .video_end_time(self.end_time.unwrap_or(0.0))
            .build()?;

        Ok(vec![slide])
    }

    fn box_clone(&self) -> Box<dyn ServiceTrait> {
        Box::new((*self).clone())
    }
}

impl Model<Video> {
    pub async fn new_video_model(db: &mut SqlitePool) -> Self {
        let mut model = Self {
            items: vec![],
            kind: LibraryKind::Video,
        };
        let mut db = db.acquire().await.expect("probs");

        model.load_from_db(&mut db).await;
        model
    }

    pub async fn load_from_db(&mut self, db: &mut SqliteConnection) {
        let result = query_as!(Video, r#"SELECT title as "title!", file_path as "path!", start_time as "start_time!: f32", end_time as "end_time!: f32", loop as "looping!", id as "id: i32" from videos"#).fetch_all(db).await;
        match result {
            Ok(v) => {
                for video in v {
                    let _ = self.add_item(video);
                }
            }
            Err(e) => {
                error!(
                    "There was an error in converting videos: {e}"
                );
            }
        }
    }
}

pub async fn remove_from_db(
    db: PoolConnection<Sqlite>,
    id: i32,
) -> Result<()> {
    query!("DELETE FROM videos WHERE id = $1", id)
        .execute(&mut db.detach())
        .await
        .into_diagnostic()
        .map(|_| ())
}

pub async fn add_video_to_db(
    video: Video,
    db: PoolConnection<Sqlite>,
) -> Result<()> {
    let path = video
        .path
        .to_str()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let mut db = db.detach();
    query!(
        r#"INSERT INTO videos (title, file_path, start_time, end_time, loop) VALUES ($1, $2, $3, $4, $5)"#,
        video.title,
        path,
        video.start_time,
        video.end_time,
        video.looping
    )
    .execute(&mut db)
    .await
    .into_diagnostic()?;
    Ok(())
}

pub async fn update_video_in_db(
    video: Video,
    db: PoolConnection<Sqlite>,
) -> Result<()> {
    let path = video
        .path
        .to_str()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let mut db = db.detach();
    debug!(?video, "should be been updated");
    let result = query!(
        r#"UPDATE videos SET title = $2, file_path = $3, start_time = $4, end_time = $5, loop = $6 WHERE id = $1"#,
        video.id,
        video.title,
        path,
        video.start_time,
        video.end_time,
        video.looping,
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

pub async fn get_video_from_db(
    database_id: i32,
    db: &mut SqliteConnection,
) -> Result<Video> {
    query_as!(Video, r#"SELECT title as "title!", file_path as "path!", start_time as "start_time!: f32", end_time as "end_time!: f32", loop as "looping!", id as "id: i32" from videos where id = ?"#, database_id).fetch_one(db).await.into_diagnostic()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    fn test_video(title: String) -> Video {
        Video {
            title,
            path: PathBuf::from("~/vids/camprules2024.mp4"),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_db_and_model() {
        let mut video_model: Model<Video> = Model {
            items: vec![],
            kind: LibraryKind::Video,
        };
        let mut db = crate::core::model::get_db().await;
        video_model.load_from_db(&mut db).await;
        if let Some(video) = video_model.find(|v| v.id == 73) {
            let test_video = test_video(
                "Getting started with Tokio. The ultimate starter guide to writing async Rust."
                    .into(),
            );
            assert_eq!(test_video.title, video.title);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn test_add_video() {
        let video = test_video("A new video".into());
        let mut video_model: Model<Video> = Model {
            items: vec![],
            kind: LibraryKind::Video,
        };
        let result = video_model.add_item(video.clone());
        let new_video = test_video("A newer video".into());
        match result {
            Ok(_) => {
                assert_eq!(
                    &video,
                    video_model.find(|v| v.id == 0).unwrap()
                );
                assert_ne!(
                    &new_video,
                    video_model.find(|v| v.id == 0).unwrap()
                );
            }
            Err(e) => assert!(
                false,
                "There was an error adding the video: {:?}",
                e
            ),
        }
    }
}
