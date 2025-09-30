use std::{borrow::Cow, mem::replace};

use cosmic::iced::clipboard::mime::{AllowedMimeTypes, AsMimeTypes};
use miette::{IntoDiagnostic, Result, miette};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, SqliteConnection};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Model<T> {
    pub items: Vec<T>,
    pub kind: LibraryKind,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Copy, Hash, Serialize, Deserialize,
)]
pub enum LibraryKind {
    Song,
    Video,
    Image,
    Presentation,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Copy, Hash, Serialize, Deserialize,
)]
pub struct KindWrapper(pub (LibraryKind, i32));

impl TryFrom<(Vec<u8>, String)> for KindWrapper {
    type Error = miette::Error;

    fn try_from(
        value: (Vec<u8>, String),
    ) -> std::result::Result<Self, Self::Error> {
        debug!(?value);
        ron::de::from_bytes(&value.0).into_diagnostic()
    }
}

impl AllowedMimeTypes for KindWrapper {
    fn allowed() -> Cow<'static, [String]> {
        Cow::from(vec!["application/service-item".to_string()])
    }
}

impl AsMimeTypes for KindWrapper {
    fn available(&self) -> Cow<'static, [String]> {
        debug!(?self);
        Cow::from(vec!["application/service-item".to_string()])
    }

    fn as_bytes(
        &self,
        mime_type: &str,
    ) -> Option<std::borrow::Cow<'static, [u8]>> {
        debug!(?self);
        debug!(mime_type);
        let ron = ron::ser::to_string(self).ok()?;
        debug!(ron);
        Some(Cow::from(ron.into_bytes()))
    }
}

impl<T> Model<T> {
    pub fn add_item(&mut self, item: T) -> Result<()> {
        self.items.push(item);
        Ok(())
    }

    pub fn add_to_db(&mut self, item: T) -> Result<()> {
        todo!()
    }

    pub fn update_item(&mut self, item: T, index: i32) -> Result<()> {
        if let Some(current_item) = self.items.get_mut(index as usize)
        {
            let _old_item = replace(current_item, item);
            Ok(())
        } else {
            Err(miette!(
                "Item doesn't exist in model. Id was {}",
                index
            ))
        }
    }

    pub fn remove_item(&mut self, index: i32) -> Result<()> {
        self.items.remove(index as usize);
        Ok(())
    }

    #[must_use]
    pub fn get_item(&self, index: i32) -> Option<&T> {
        self.items.get(index as usize)
    }

    pub fn find<P>(&self, f: P) -> Option<&T>
    where
        P: FnMut(&&T) -> bool,
    {
        self.items.iter().find(f)
    }

    pub fn insert_item(&mut self, item: T, index: i32) -> Result<()> {
        self.items.insert(index as usize, item);
        Ok(())
    }
}

// impl<T> Default for Model<T> {
//     fn default() -> Self {
//         Self {
//             items: vec![],
//             db: {
//                 get_db().await
//             }
//         }
//     }
// }

pub async fn get_db() -> SqliteConnection {
    let mut data = dirs::data_local_dir().unwrap();
    data.push("lumina");
    data.push("library-db.sqlite3");
    let mut db_url = String::from("sqlite://");
    db_url.push_str(data.to_str().unwrap());
    SqliteConnection::connect(&db_url).await.expect("problems")
}

pub trait Modeling {
    type Item;

    // fn setup_db() -> SqliteConnection {
    //     let rt = executor::Default::new().unwrap();
    //     let rt = executor::multi::Executor::new().unwrap();
    //     let mut data = dirs::data_local_dir().unwrap();
    //     data.push("lumina");
    //     data.push("library-db.sqlite3");
    //     let mut db_url = String::from("sqlite://");
    //     db_url.push_str(data.to_str().unwrap());
    //     rt.spawn(async {
    //         SqliteConnection::connect(&db_url)
    //             .await
    //             .expect("problems")
    //     });
    //     rt.enter(async {
    //         SqliteConnection::connect(&db_url)
    //             .await
    //             .expect("problems")
    //     });
    // }
}

#[cfg(test)]
mod test {}
