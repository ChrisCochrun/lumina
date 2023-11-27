use std::pin::Pin;

use time::macros::format_description;
use tokio::runtime::Runtime;
use tracing_subscriber::{
    fmt::{self, time::LocalTime},
    EnvFilter,
};

mod db {
    use diesel::{Connection, SqliteConnection};
    use dirs::data_local_dir;
    use sqlx::{Connection as SqlxConnection, Error};

    pub enum Model {
        Songs,
        Presentations,
        Videos,
        Images,
    }

    fn get_db() -> SqliteConnection {
        let mut data = data_local_dir().unwrap();
        data.push("lumina");
        data.push("library-db.sqlite3");
        let mut db_url = String::from("sqlite://");
        db_url.push_str(data.to_str().unwrap());
        println!("DB: {:?}", db_url);

        SqliteConnection::establish(&db_url).unwrap_or_else(|_| {
            panic!("error connecting to {}", db_url)
        })
    }

    // fn get_items(model: Model) -> Result<(), Error> {
    //     let conn = sqlx::SqliteConnection::connect(
    //         "sqlite::/home/chris/.local/share/lumina/library-db.sqlite3",
    //     );
    //     match model {
    //         Songs => {
    //             let select = sqlx::query_as("SELECT $1")
    //                 .bind("songs")
    //                 .fetch_all(&mut conn);
    //         }
    //     }

    //     Ok(())
    // }
}

#[cxx_qt::bridge]
mod utilities {
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Utils = super::UtilsRust;

        #[qinvokable]
        fn setup(self: Pin<&mut Utils>);
    }
}

#[derive(Debug)]
pub struct UtilsRust {
    runtime: Runtime,
}

impl Default for UtilsRust {
    fn default() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
}

impl utilities::Utils {
    pub fn setup(mut self: Pin<&mut Self>) {
        crate::utils::setup();
    }
}

pub fn setup() {
    tracing_subscriber::FmtSubscriber::builder()
        .pretty()
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
