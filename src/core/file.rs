use tar::{Archive, Builder};
use tracing::error;
use zstd::Encoder;
use std::{fs::{self, File}, iter, path::{Path, PathBuf}};
use color_eyre::eyre::{eyre, Context, Result};
use serde_json::Value;
use sqlx::{query, query_as, FromRow, SqliteConnection};
use crate::{images::{get_image_from_db, Image}, kinds::ServiceItemKind, model::get_db, presentations::{get_presentation_from_db, PresKind, Presentation}, service_items::ServiceItem, slides::Background, songs::{get_song_from_db, Song}, videos::{get_video_from_db, Video}};

pub async fn save(list: Vec<ServiceItem>, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let save_file = File::create(path)?;
    let mut db = get_db().await;
    let json = process_service_items(&list, &mut db).await?;
    let archive = store_service_items(&list, &mut db, &save_file, &json).await?;
    Ok(())
}

async fn store_service_items(items: &Vec<ServiceItem>, db: &mut SqliteConnection, save_file: &File, json: &Value) -> Result<()> {
    let encoder = Encoder::new(save_file, 3).unwrap();
    let mut tar = Builder::new(encoder);
    let mut temp_dir = dirs::data_dir().unwrap();
    temp_dir.push("lumina");
    let mut s: String =
        iter::repeat_with(fastrand::alphanumeric)
        .take(5)
        .collect();
    s.insert_str(0, "temp_");
    temp_dir.push(s);
    fs::create_dir_all(&temp_dir)?;
    let service_file = temp_dir.join("serviceitems.json");
    fs::File::create(&service_file)?;
    match fs::File::options().read(true).write(true).open(service_file) {
        Ok(f) => {
            serde_json::to_writer_pretty(f, json)?;
        },
        Err(e) => error!("There were problems making a file i guess: {e}"),
    };
    for item in items {
        let background;
        let audio: Option<PathBuf>;
        match item.kind {
            ServiceItemKind::Song => {
                let song = get_song_from_db(item.database_id, db).await?;
                background = song.background;
                audio = song.audio;
            },
            ServiceItemKind::Image => {
                let image = get_image_from_db(item.database_id, db).await?;
                background = Some(Background::try_from(image.path)?);
                audio = None;
            },
            ServiceItemKind::Video => {
                let video = get_video_from_db(item.database_id, db).await?;
                background = Some(Background::try_from(video.path)?);
                audio = None;
            },
            ServiceItemKind::Presentation(_) => {
                let presentation = get_presentation_from_db(item.database_id, db).await?;
                background = Some(Background::try_from(presentation.path)?);
                audio = None;
            },
            ServiceItemKind::Content => {
                todo!()
            },
        };
        if let Some(file) = audio {
            let audio_file = temp_dir.join(file.file_name().expect("Audio file couldn't be added to temp_dir"));
            if let Ok(file) = file.strip_prefix("file://") {
                fs::File::create(&audio_file).wrap_err("Couldn't create audio file")?;
                fs::copy(file, audio_file).wrap_err("Audio file could not be copied, the source file doesn't exist not be found");
            } else {
                fs::File::create(&audio_file).wrap_err("Couldn't create audio file")?;
                fs::copy(file, audio_file).wrap_err("Audio file could not be copied, the source file doesn't exist not be found");
            }
        };
        if let Some(file) = background {
            let background_file = temp_dir.join(file.path.file_name().expect("Background file couldn't be added to temp_dir"));
            if let Ok(file) = file.path.strip_prefix("file://") {
                fs::File::create(&background_file).wrap_err("Couldn't create background file")?;
                fs::copy(file, background_file).wrap_err("Background file could not be copied, the source file doesn't exist not be found");
            } else {
                fs::File::create(&background_file).wrap_err("Couldn't create background file")?;
                fs::copy(file.path, background_file).wrap_err("Background file could not be copied, the source file doesn't exist not be found");
            }
        }
    }
    Ok(())
}

async fn clear_temp_dir(temp_dir: &Path) -> Result<()> {
    todo!()
}

async fn process_service_items(items: &Vec<ServiceItem>, db: &mut SqliteConnection) -> Result<Value> {
    let mut values: Vec<Value> = vec![];
    for item in items {
        match item.kind {
            ServiceItemKind::Song => {
                let value = process_song(item.database_id, db).await?;
                values.push(value);
            },
            ServiceItemKind::Image => {
                let value = process_image(item.database_id, db).await?;
                values.push(value);
            },
            ServiceItemKind::Video => {
                let value = process_video(item.database_id, db).await?;
                values.push(value);
            },
            ServiceItemKind::Presentation(_) => {
                let value = process_presentation(item.database_id, db).await?;
                values.push(value);
            },
            ServiceItemKind::Content => {
                todo!()
            },
        }
    }
    let json = Value::from(values);
    Ok(json)
}

async fn process_song(database_id: i32, db: &mut SqliteConnection) -> Result<Value> {
    let song = get_song_from_db(database_id, db).await?;
    let song_json = serde_json::to_value(&song)?;
    let kind_json = serde_json::to_value(ServiceItemKind::Song)?;
    let json = serde_json::json!({"item": song_json, "kind": kind_json});
    Ok(json)
}

async fn process_image(database_id: i32, db: &mut SqliteConnection) -> Result<Value> {
    let image = get_image_from_db(database_id, db).await?;
    let image_json = serde_json::to_value(&image)?;
    let kind_json = serde_json::to_value(ServiceItemKind::Image)?;
    let json = serde_json::json!({"item": image_json, "kind": kind_json});
    Ok(json)
}

async fn process_video(database_id: i32, db: &mut SqliteConnection) -> Result<Value> {
    let video = get_video_from_db(database_id, db).await?;
    let video_json = serde_json::to_value(&video)?;
    let kind_json = serde_json::to_value(ServiceItemKind::Video)?;
    let json = serde_json::json!({"item": video_json, "kind": kind_json});
    Ok(json)
}

async fn process_presentation(database_id: i32, db: &mut SqliteConnection) -> Result<Value> {
    let presentation = get_presentation_from_db(database_id, db).await?;
    let presentation_json = serde_json::to_value(&presentation)?;
    let kind_json = match presentation.kind {
        PresKind::Html => serde_json::to_value(ServiceItemKind::Presentation(PresKind::Html))?,
        PresKind::Pdf => serde_json::to_value(ServiceItemKind::Presentation(PresKind::Pdf))?,
        PresKind::Generic => serde_json::to_value(ServiceItemKind::Presentation(PresKind::Generic))?,
    };
    let json = serde_json::json!({"item": presentation_json, "kind": kind_json});
    Ok(json)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use fs::canonicalize;
    use sqlx::Connection;
    use pretty_assertions::assert_eq;
    use tracing::debug;
    use super::*;

    async fn get_db() -> SqliteConnection {
        let mut data = dirs::data_local_dir().unwrap();
        data.push("lumina");
        data.push("library-db.sqlite3");
        let mut db_url = String::from("sqlite://");
        db_url.push_str(data.to_str().unwrap());
        SqliteConnection::connect(&db_url)
            .await
            .expect("problems")
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_process_song() {
        let mut db = get_db().await;
        let result = process_song(7, &mut db).await;
        let json_song_file = PathBuf::from("./test/test_song.json");
        if let Ok(path) = canonicalize(json_song_file) {
            debug!(file = ?&path);
            if let Ok(s) = fs::read_to_string(path) {
                debug!(s);
                match result {
                    Ok(json) => assert_eq!(json.to_string(), s),
                    Err(e) => panic!("There was an error in processing the song: {e}"),
                }
            } else {
                panic!("String wasn't read from file");
            }
        } else {
            panic!("Cannot find absolute path to test_song.json");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_process_image() {
        let mut db = get_db().await;
        let result = process_image(3, &mut db).await;
        let json_image_file = PathBuf::from("./test/test_image.json");
        if let Ok(path) = canonicalize(json_image_file) {
            debug!(file = ?&path);
            if let Ok(s) = fs::read_to_string(path) {
                debug!(s);
                match result {
                    Ok(json) => assert_eq!(json.to_string(), s),
                    Err(e) => panic!("There was an error in processing the image: {e}"),
                }
            } else {
                panic!("String wasn't read from file");
            }
        } else {
            panic!("Cannot find absolute path to test_image.json");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_process_video() {
        let mut db = get_db().await;
        let result = process_video(73, &mut db).await;
        let json_video_file = PathBuf::from("./test/test_video.json");
        if let Ok(path) = canonicalize(json_video_file) {
            debug!(file = ?&path);
            if let Ok(s) = fs::read_to_string(path) {
                debug!(s);
                match result {
                    Ok(json) => assert_eq!(json.to_string(), s),
                    Err(e) => panic!("There was an error in processing the video: {e}"),
                }
            } else {
                panic!("String wasn't read from file");
            }
        } else {
            panic!("Cannot find absolute path to test_video.json");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_process_presentation() {
        let mut db = get_db().await;
        let result = process_presentation(54, &mut db).await;
        let json_presentation_file = PathBuf::from("./test/test_presentation.json");
        if let Ok(path) = canonicalize(json_presentation_file) {
            debug!(file = ?&path);
            if let Ok(s) = fs::read_to_string(path) {
                debug!(s);
                match result {
                    Ok(json) => assert_eq!(json.to_string(), s),
                    Err(e) => panic!("There was an error in processing the presentation: {e}"),
                }
            } else {
                panic!("String wasn't read from file");
            }
        } else {
            panic!("Cannot find absolute path to test_presentation.json");
        }
    }

    fn get_items() -> Vec<ServiceItem> {
        let items = vec![
            ServiceItem {
                database_id: 7,
                kind: ServiceItemKind::Song,
                id: 0,
            },
            ServiceItem {
                database_id: 54,
                kind: ServiceItemKind::Presentation(PresKind::Html),
                id: 0,
            },
            ServiceItem {
                database_id: 73,
                kind: ServiceItemKind::Video,
                id: 0,
            },
        ];
        items
    }

    #[tokio::test]
    async fn test_service_items() {
        let mut db = get_db().await;
        let items = get_items();
        let json_item_file = PathBuf::from("./test/test_service_items.json");
        let result = process_service_items(&items, &mut db).await;
        if let Ok(path) = canonicalize(json_item_file) {
            if let Ok(s) = fs::read_to_string(path) {
                match result {
                    Ok(strings) => assert_eq!(strings.to_string(), s),
                    Err(e) => panic!("There was an error: {e}"),
                }
            }
        }
    }

    // #[tokio::test]
    // async fn test_save() {
    //     let path = PathBuf::from("~/dev/lumina/src/rust/core/test.pres");
    //     let list = get_items();
    //     match save(list, path).await {
    //         Ok(_) => assert!(true),
    //         Err(e) => panic!("There was an error: {e}"),
    //     }
    // }

    #[tokio::test]
    async fn test_store() {
        let path = PathBuf::from("/home/chris/dev/lumina/src/rust/core/test.pres");
        let save_file =  match File::create(path) {
            Ok(f) => f,
            Err(e) => panic!("Couldn't create save_file: {e}"),
        };
        let mut db = get_db().await;
        let list = get_items();
        if let Ok(json) = process_service_items(&list, &mut db).await {
            println!("{:?}", json);
            match store_service_items(&list, &mut db, &save_file, &json).await {
                Ok(_) => assert!(true),
                Err(e) => panic!("There was an error: {e}"),
            }
        } else {
            panic!("There was an error getting the json value");
        }
    }

    // #[tokio::test]
    // async fn test_things() {
    //     let mut temp_dir = dirs::data_dir().unwrap();
    //     temp_dir.push("lumina");
    //     let mut s: String =
    //         iter::repeat_with(fastrand::alphanumeric)
    //         .take(5)
    //         .collect();
    //     s.insert_str(0, "temp_");
    //     temp_dir.push(s);
    //     let _ = fs::create_dir_all(&temp_dir);
    //     let mut db = get_db().await;
    //     let service_file = temp_dir.join("serviceitems.json");
    //     let list = get_items();
    //     if let Ok(json) = process_service_items(&list, &mut db).await {
    //         let _ = fs::File::create(&service_file);
    //         match fs::write(service_file, json.to_string()) {
    //             Ok(_) => assert!(true),
    //             Err(e) => panic!("There was an error: {e}"),
    //         }
    //     } else {
    //         panic!("There was an error getting the json value");
    //     }
    // }
}
