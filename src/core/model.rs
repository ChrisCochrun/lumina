use std::borrow::Cow;
use std::fs;
use std::mem::replace;
use std::path::PathBuf;

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

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Serialize, Deserialize)]
pub enum LibraryKind {
    Song,
    Video,
    Image,
    Presentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KindWrapper(pub (LibraryKind, i32));

impl From<PathBuf> for LibraryKind {
    fn from(_value: PathBuf) -> Self {
        todo!()
    }
}

impl TryFrom<(Vec<u8>, String)> for KindWrapper {
    type Error = miette::Error;

    fn try_from(value: (Vec<u8>, String)) -> std::result::Result<Self, Self::Error> {
        let (data, mime) = value;
        match mime.as_str() {
            "application/service-item" => ron::de::from_bytes(&data).into_diagnostic(),
            _ => Err(miette!("Wrong mime type: {mime}")),
        }
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

    fn as_bytes(&self, mime_type: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
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

    pub fn add_to_db(&mut self, _item: T) -> Result<()> {
        todo!()
    }

    pub fn update_item<P>(&mut self, item: T, predicate: P) -> Result<()>
    where
        P: Fn(&T) -> bool,
    {
        self.items
            .iter()
            .position(predicate)
            .ok_or_else(|| miette!("Item cannot be found"))
            .map(|index| {
                self.items
                    .get_mut(index)
                    .expect("Since we found position this should always exist")
            })
            .map(|current_item| {
                let _old_item = replace(current_item, item);
            })
    }

    pub fn remove_item<P>(&mut self, predicate: P) -> Result<()>
    where
        P: Fn(&T) -> bool,
    {
        self.items
            .iter()
            .position(predicate)
            .ok_or_else(|| miette!("Item cannot be found"))
            .map(|index| {
                self.items.remove(index);
            })
    }

    #[must_use]
    pub fn get_item(&self, index: i32) -> Option<&T> {
        self.items
            .get(usize::try_from(index).expect("shouldn't be negative"))
    }

    pub fn find<P>(&self, f: P) -> Option<&T>
    where
        P: FnMut(&&T) -> bool,
    {
        self.items.iter().find(f)
    }

    pub fn insert_item(&mut self, item: T, index: usize) -> Result<()> {
        self.items.insert(index, item);
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
    let mut data = dirs::data_local_dir().expect("Should be able to find a data dir");
    data.push("lumina");
    let _ = fs::create_dir_all(&data);
    data.push("library-db.sqlite3");
    let mut db_url = String::from("sqlite://");
    db_url.push_str(data.to_str().expect("Should be there"));
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
