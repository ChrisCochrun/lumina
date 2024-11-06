use std::path::PathBuf;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, sqlite::SqliteRow, Row, SqliteConnection};
use tracing::error;

use crate::model::Model;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresKind {
    Html,
    #[default]
    Pdf,
    Generic,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presentation {
    pub id: i32,
    pub title: String,
    pub path: PathBuf,
    pub kind: PresKind,
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
    pub fn load_from_db(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = query!(r#"SELECT id as "id: i32", title, filePath as "path", html from presentations"#).fetch_all(&mut self.db).await;
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
                            }
                        });
                    }
                }
                Err(e) => error!("There was an error in converting presentations: {e}"),
            }
        });
    }
}

pub async fn get_presentation_from_db(database_id: i32, db: &mut SqliteConnection) -> Result<Presentation> {
    let row = query(r#"SELECT id as "id: i32", title, filePath as "path", html from presentations where id = $1"#).bind(database_id).fetch_one(db).await?;
    Ok(Presentation::from_row(&row)?)
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

    #[test]
    pub fn test_db_and_model() {
        let mut presentation_model: Model<Presentation> =
            Model::default();
        presentation_model.load_from_db();
        if let Some(presentation) = presentation_model.find(|p| p.id == 54) {
            let test_presentation = test_presentation();
            assert_eq!(&test_presentation, presentation);
        } else {
            assert!(false);
        }
    }
}
