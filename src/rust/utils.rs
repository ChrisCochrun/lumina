use std::pin::Pin;

use tokio::runtime::Runtime;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

use self::utils::{QString, QUrl};

mod db {
    use diesel::{Connection, SqliteConnection};
    use dirs::data_local_dir;

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
mod utils {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Utils = super::UtilsRust;

        #[qinvokable]
        fn setup(self: Pin<&mut Utils>);

        #[qinvokable]
        fn dbg(self: &Utils, message: QString);

        #[qinvokable]
        fn inf(self: &Utils, message: QString);

        #[qinvokable]
        fn url_to_string(self: &Utils, url: QUrl) -> QString;
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

impl utils::Utils {
    pub fn setup(self: Pin<&mut Self>) {
        crate::utils::setup();
    }

    pub fn dbg(&self, message: QString) {
        debug!(msg = ?message);
    }

    pub fn inf(&self, message: QString) {
        info!(msg = ?message);
    }

    pub fn url_to_string(&self, url: QUrl) -> QString {
        url.path()
    }
}

pub fn setup() {
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new(
        "%Y-%m-%d_%I:%M:%S%.6f %P".to_owned(),
    );
    tracing_subscriber::FmtSubscriber::builder()
        .pretty()
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_timer(timer)
        .init();
}
