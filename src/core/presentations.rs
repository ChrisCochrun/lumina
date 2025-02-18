use crisp::types::{Keyword, Value};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow, query, sqlite::SqliteRow, Row, SqliteConnection,
};
use std::path::PathBuf;
use tracing::error;

use crate::{Background, Slide, SlideBuilder, TextAlignment};

use super::{
    content::Content,
    kinds::ServiceItemKind,
    model::{get_db, LibraryKind, Model},
    service_items::ServiceTrait,
};

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum PresKind {
    Html,
    #[default]
    Pdf,
    Generic,
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct Presentation {
    pub id: i32,
    pub title: String,
    pub path: PathBuf,
    pub kind: PresKind,
}

impl Content for Presentation {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn kind(&self) -> ServiceItemKind {
        ServiceItemKind::Presentation(self.clone())
    }

    fn to_service_item(&self) -> super::service_items::ServiceItem {
        self.into()
    }
}

impl From<Value> for Presentation {
    fn from(value: Value) -> Self {
        Self::from(&value)
    }
}

impl From<&Value> for Presentation {
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
                    p.to_str().unwrap_or_default().to_string()
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

impl ServiceTrait for Presentation {
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

impl Presentation {
    pub fn new() -> Self {
        Self {
            title: "".to_string(),
            ..Default::default()
        }
    }

    pub fn get_kind(&self) -> &PresKind {
        &self.kind
    }
}

impl FromRow<'_, SqliteRow> for Presentation {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            path: PathBuf::from({
                let string: String = row.try_get(2)?;
                string
            }),
            kind: if row.try_get(3)? {
                PresKind::Html
            } else {
                PresKind::Pdf
            },
        })
    }
}

impl Model<Presentation> {
    pub async fn new_presentation_model(
        db: &mut SqliteConnection,
    ) -> Self {
        let mut model = Self {
            items: vec![],
            kind: LibraryKind::Presentation,
        };

        model.load_from_db(db).await;
        model
    }

    pub async fn load_from_db(&mut self, db: &mut SqliteConnection) {
        let result = query!(
            r#"SELECT id as "id: i32", title, file_path as "path", html from presentations"#
        )
            .fetch_all(db)
            .await;
        match result {
            Ok(v) => {
                for presentation in v.into_iter() {
                    let _ = self.add_item(Presentation {
                        id: presentation.id,
                        title: presentation.title,
                        path: presentation.path.into(),
                        kind: if presentation.html {
                            PresKind::Html
                        } else {
                            PresKind::Pdf
                        },
                    });
                }
            }
            Err(e) => error!(
                "There was an error in converting presentations: {e}"
            ),
        }
    }
}

pub async fn get_presentation_from_db(
    database_id: i32,
    db: &mut SqliteConnection,
) -> Result<Presentation> {
    let row = query(r#"SELECT id as "id: i32", title, file_path as "path", html from presentations where id = $1"#).bind(database_id).fetch_one(db).await.into_diagnostic()?;
    Presentation::from_row(&row).into_diagnostic()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    fn test_presentation() -> Presentation {
        Presentation {
            id: 54,
            title: "20240327T133649--12-isaiah-and-jesus__lesson_project_tfc".into(),
            path: PathBuf::from(
                "file:///home/chris/docs/notes/lessons/20240327T133649--12-isaiah-and-jesus__lesson_project_tfc.html",
            ),
            kind: PresKind::Html,
        }
    }

    #[test]
    pub fn test_pres() {
        let pres = Presentation::new();
        assert_eq!(pres.get_kind(), &PresKind::Pdf)
    }

    #[tokio::test]
    async fn test_db_and_model() {
        let mut presentation_model: Model<Presentation> = Model {
            items: vec![],
            db: crate::core::model::get_db().await,
        };
        presentation_model.load_from_db().await;
        if let Some(presentation) =
            presentation_model.find(|p| p.id == 54)
        {
            let test_presentation = test_presentation();
            assert_eq!(&test_presentation, presentation);
        } else {
            assert!(false);
        }
    }
}
