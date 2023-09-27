mod db {
    use diesel::{Connection, SqliteConnection};
    use dirs::data_local_dir;
    use sqlx::{Error, SqliteConnection as SqlxConn};

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

    // async fn get_items(model: Model) -> Result<(), Error> {
    //     let conn = SqlxConn::connect(
    //         "/home/chris/.local/share/lumina/library-db.sqlite3",
    //     );
    //     match model {
    //         Songs => {
    //             let select = sqlx::query_as("SELECT $1")
    //                 .bind("songs")
    //                 .fetch_all(&mut conn).await?;
    //         }
    //     }

    //     Ok(());
    // }
}