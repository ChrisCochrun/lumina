use super::model::Model;
use cosmic::{executor, iced::Executor};
use miette::{miette, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, SqliteConnection};
use std::path::PathBuf;
use tracing::error;

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

impl Model<Video> {
    pub async fn load_from_db(&mut self) {
        let result = query_as!(Video, r#"SELECT title as "title!", file_path as "path!", start_time as "start_time!: f32", end_time as "end_time!: f32", loop as "looping!", id as "id: i32" from videos"#).fetch_all(&mut self.db).await;
        match result {
            Ok(v) => {
                for video in v.into_iter() {
                    let _ = self.add_item(video);
                }
            }
            Err(e) => {
                error!("There was an error in converting videos: {e}")
            }
        };
    }
}

pub async fn get_video_from_db(
    database_id: i32,
    db: &mut SqliteConnection,
) -> Result<Video> {
    Ok(query_as!(Video, r#"SELECT title as "title!", file_path as "path!", start_time as "start_time!: f32", end_time as "end_time!: f32", loop as "looping!", id as "id: i32" from videos where id = ?"#, database_id).fetch_one(db).await.into_diagnostic()?)
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
            db: crate::core::model::get_db().await,
        };
        video_model.load_from_db().await;
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
            db: crate::core::model::get_db().await,
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
