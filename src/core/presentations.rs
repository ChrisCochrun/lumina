use cosmic::widget::image::Handle;
use crisp::types::{Keyword, Symbol, Value};
use miette::{IntoDiagnostic, Result};
use mupdf::{Colorspace, Document, Matrix};
use serde::{Deserialize, Serialize};
use sqlx::{
    Row, Sqlite, SqliteConnection, SqlitePool, pool::PoolConnection,
    prelude::FromRow, query, sqlite::SqliteRow,
};
use std::path::{Path, PathBuf};
use tracing::{debug, error};

use crate::{Background, Slide, SlideBuilder, TextAlignment};

use super::{
    content::Content,
    kinds::ServiceItemKind,
    model::{LibraryKind, Model},
    service_items::ServiceTrait,
};

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum PresKind {
    Html,
    Pdf {
        starting_index: i32,
        ending_index: i32,
    },
    #[default]
    Generic,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Presentation {
    pub id: i32,
    pub title: String,
    pub path: PathBuf,
    pub kind: PresKind,
}

impl Eq for Presentation {}

impl PartialEq for Presentation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.path == other.path
            && self.kind == other.kind
    }
}

impl From<PathBuf> for Presentation {
    fn from(value: PathBuf) -> Self {
        let title = value
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();
        let kind = match value
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
        {
            "pdf" => {
                if let Ok(document) = Document::open(&value.as_path())
                {
                    if let Ok(count) = document.page_count() {
                        PresKind::Pdf {
                            starting_index: 0,
                            ending_index: count - 1,
                        }
                    } else {
                        PresKind::Pdf {
                            starting_index: 0,
                            ending_index: 0,
                        }
                    }
                } else {
                    PresKind::Pdf {
                        starting_index: 0,
                        ending_index: 0,
                    }
                }
            }
            "html" => PresKind::Html,
            _ => PresKind::Generic,
        };
        Self {
            id: 0,
            title,
            path: value.canonicalize().unwrap_or(value),
            kind,
        }
    }
}

impl From<&Path> for Presentation {
    fn from(value: &Path) -> Self {
        Self::from(value.to_owned())
    }
}

impl From<&Presentation> for Value {
    fn from(_value: &Presentation) -> Self {
        Self::List(vec![Self::Symbol(Symbol("presentation".into()))])
    }
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

    fn background(&self) -> Option<Background> {
        Background::try_from(self.path.clone()).ok()
    }

    fn subtext(&self) -> String {
        if self.path.exists() {
            self.path
                .file_name()
                .map_or("Missing presentation".into(), |f| {
                    f.to_string_lossy().to_string()
                })
        } else {
            "Missing presentation".into()
        }
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
        debug!(?self);
        let background = Background::try_from(self.path.clone())
            .into_diagnostic()?;
        debug!(?background);
        let document = Document::open(background.path.as_path())
            .into_diagnostic()?;
        debug!(?document);
        let pages = document.pages().into_diagnostic()?;
        debug!(?pages);
        let pages: Vec<Handle> = pages
            .filter_map(|page| {
                let Some(page) = page.ok() else {
                    return None;
                };
                let matrix = Matrix::IDENTITY;
                let colorspace = Colorspace::device_rgb();
                let Ok(pixmap) = page
                    .to_pixmap(&matrix, &colorspace, true, true)
                    .into_diagnostic()
                else {
                    error!("Can't turn this page into pixmap");
                    return None;
                };
                debug!(?pixmap);
                Some(Handle::from_rgba(
                    pixmap.width(),
                    pixmap.height(),
                    pixmap.samples().to_vec(),
                ))
            })
            .collect();

        let mut slides: Vec<Slide> = vec![];
        for (index, page) in pages.into_iter().enumerate() {
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
                .pdf_index(index as u32)
                .pdf_page(page)
                .build()?;
            slides.push(slide);
        }
        debug!(?slides);
        Ok(slides)
    }

    fn box_clone(&self) -> Box<dyn ServiceTrait> {
        Box::new((*self).clone())
    }
}

impl Presentation {
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: String::new(),
            ..Default::default()
        }
    }

    #[must_use]
    pub const fn get_kind(&self) -> &PresKind {
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
                PresKind::Pdf {
                    starting_index: row.try_get(4)?,
                    ending_index: row.try_get(5)?,
                }
            },
        })
    }
}

impl Model<Presentation> {
    pub async fn new_presentation_model(db: &mut SqlitePool) -> Self {
        let mut model = Self {
            items: vec![],
            kind: LibraryKind::Presentation,
        };
        let mut db = db.acquire().await.expect("probs");

        model.load_from_db(&mut db).await;
        model
    }

    pub async fn load_from_db(&mut self, db: &mut SqliteConnection) {
        let result = query!(
            r#"SELECT id as "id: i32", title, file_path as "path", html, starting_index, ending_index from presentations"#
        )
            .fetch_all(db)
            .await;

        match result {
            Ok(v) => {
                for presentation in v {
                    let _ = self.add_item(Presentation {
                        id: presentation.id,
                        title: presentation.title,
                        path: presentation.path.into(),
                        kind: if presentation.html {
                            PresKind::Html
                        } else {
                            if let (
                                Some(starting_index),
                                Some(ending_index),
                            ) = (
                                presentation.starting_index,
                                presentation.ending_index,
                            ) {
                                PresKind::Pdf {
                                    starting_index: starting_index
                                        as i32,
                                    ending_index: ending_index as i32,
                                }
                            } else {
                                PresKind::Generic
                            }
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

pub async fn remove_from_db(
    db: PoolConnection<Sqlite>,
    id: i32,
) -> Result<()> {
    query!("DELETE FROM presentations WHERE id = $1", id)
        .execute(&mut db.detach())
        .await
        .into_diagnostic()
        .map(|_| ())
}

pub async fn add_presentation_to_db(
    presentation: Presentation,
    db: PoolConnection<Sqlite>,
) -> Result<()> {
    let path = presentation
        .path
        .to_str()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let html = presentation.kind == PresKind::Html;
    let mut db = db.detach();
    query!(
        r#"INSERT INTO presentations (title, file_path, html) VALUES ($1, $2, $3)"#,
        presentation.title,
        path,
        html,
    )
    .execute(&mut db)
    .await
    .into_diagnostic()?;
    Ok(())
}

pub async fn update_presentation_in_db(
    presentation: Presentation,
    db: PoolConnection<Sqlite>,
) -> Result<()> {
    let path = presentation
        .path
        .to_str()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    let html = presentation.kind == PresKind::Html;
    let mut db = db.detach();
    let mut starting_index = 0;
    let mut ending_index = 0;
    if let PresKind::Pdf {
        starting_index: s_index,
        ending_index: e_index,
    } = presentation.get_kind()
    {
        starting_index = *s_index;
        ending_index = *e_index;
    };
    let id = presentation.id;
    if let Err(e) =
        query!("SELECT id FROM presentations where id = $1", id)
            .fetch_one(&mut db)
            .await
    {
        if let Ok(ids) = query!("SELECT id FROM presentations")
            .fetch_all(&mut db)
            .await
        {
            let Some(mut max) = ids.iter().map(|r| r.id).max() else {
                return Err(miette::miette!("cannot find max id"));
            };
            debug!(?e, "Presentation not found");
            max += 1;
            let result = query!(
                r#"INSERT into presentations VALUES($1, $2, $3, $4, $5, $6)"#,
                max,
                presentation.title,
                path,
                html,
                starting_index,
                ending_index,
            )
            .execute(&mut db)
            .await
            .into_diagnostic();

            return match result {
                Ok(_) => {
                    debug!("should have been updated");
                    Ok(())
                }
                Err(e) => {
                    error! {?e};
                    Err(e)
                }
            };
        } else {
            return Err(miette::miette!("cannot find ids"));
        }
    };

    debug!(?presentation, "should be been updated");
    let result = query!(
        r#"UPDATE presentations SET title = $2, file_path = $3, html = $4 WHERE id = $1"#,
        presentation.id,
        presentation.title,
        path,
        html
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
            kind: LibraryKind::Presentation,
        };
        let mut db = crate::core::model::get_db().await;
        presentation_model.load_from_db(&mut db).await;
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
