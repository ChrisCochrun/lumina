use std::mem::replace;

use miette::{miette, Result};
use sqlx::{Connection, SqliteConnection};

#[derive(Debug, Clone)]
pub struct Model<T> {
    pub items: Vec<T>,
    pub kind: LibraryKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum LibraryKind {
    Song,
    Video,
    Image,
    Presentation,
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
