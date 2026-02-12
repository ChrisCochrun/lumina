use crate::core::{
    kinds::ServiceItemKind, service_items::ServiceItem,
    slide::Background,
};
use miette::{IntoDiagnostic, Result, miette};
use std::{
    fs::{self, File},
    io::Write,
    iter,
    path::{Path, PathBuf},
};
use tar::Builder;
use tracing::{debug, error};
use zstd::Encoder;

pub fn save(
    list: Vec<ServiceItem>,
    path: impl AsRef<Path>,
) -> Result<()> {
    let path = path.as_ref();
    let save_file = File::create(path).into_diagnostic()?;
    let ron = process_service_items(&list)?;

    let encoder = Encoder::new(save_file, 3).unwrap();
    let mut tar = Builder::new(encoder);
    let mut temp_dir = dirs::data_dir().unwrap();
    temp_dir.push("lumina");
    let mut s: String =
        iter::repeat_with(fastrand::alphanumeric).take(5).collect();
    s.insert_str(0, "temp_");
    temp_dir.push(s);
    fs::create_dir_all(&temp_dir).into_diagnostic()?;
    let service_file = temp_dir.join("serviceitems.ron");
    dbg!(&service_file);
    fs::File::create(&service_file).into_diagnostic()?;
    match fs::File::options()
        .read(true)
        .write(true)
        .open(service_file)
    {
        Ok(mut f) => {
            f.write(ron.as_bytes()).into_diagnostic()?;
            match tar.append_file("serviceitems.ron", &mut f) {
                Ok(_) => (),
                Err(e) => {
                    error!(?e);
                    dbg!(&e);
                    return Err(miette!("PROBS: {e}"));
                }
            }
        }
        Err(e) => {
            error!("There were problems making a file i guess: {e}");
            return Err(miette!("There was a problem: {e}"));
        }
    }
    // let list list.iter_mut().map(|item| {
    //     match item.kind {
    //         ServiceItemKind::Song(mut song) => {
    //             song.background
    //         }
    //     }
    // }).collect();
    for item in list {
        let background;
        let audio: Option<PathBuf>;
        match &item.kind {
            ServiceItemKind::Song(song) => {
                background = song.background.clone();
                audio = song.audio.clone();
            }
            ServiceItemKind::Image(image) => {
                background = Some(
                    Background::try_from(image.path.clone())
                        .into_diagnostic()?,
                );
                audio = None;
            }
            ServiceItemKind::Video(video) => {
                background = Some(
                    Background::try_from(video.path.clone())
                        .into_diagnostic()?,
                );
                audio = None;
            }
            ServiceItemKind::Presentation(presentation) => {
                background = Some(
                    Background::try_from(presentation.path.clone())
                        .into_diagnostic()?,
                );
                audio = None;
            }
            ServiceItemKind::Content(_slide) => {
                todo!()
            }
        }
        if let Some(file) = audio {
            let audio_file =
                temp_dir.join(file.file_name().expect(
                    "Audio file couldn't be added to temp_dir",
                ));
            if let Ok(file) = file.strip_prefix("file://") {
                let mut file =
                    fs::File::open(&audio_file).into_diagnostic()?;
                tar.append_file("", &mut file).into_diagnostic()?;
            } else {
                let mut file =
                    fs::File::open(&audio_file).into_diagnostic()?;
                tar.append_file("", &mut file).into_diagnostic()?;
            }
        }
        if let Some(file) = background {
            let background_file =
                temp_dir.join(file.path.file_name().expect(
                    "Background file couldn't be added to temp_dir",
                ));
            if let Ok(file) = file.path.strip_prefix("file://") {
                let mut file = fs::File::open(&background_file)
                    .into_diagnostic()?;
                tar.append_file("", &mut file).into_diagnostic()?;
            } else {
                let mut file = fs::File::open(&background_file)
                    .into_diagnostic()?;
                tar.append_file("", &mut file).into_diagnostic()?;
            }
        }
    }
    // match tar.append_dir_all(path, &temp_dir) {
    //     Ok(_) => (),
    //     Err(e) => {
    //         error!(?e);
    //         dbg!(&e);
    //         return Err(miette!("tar error: {e}"));
    //     }
    // }
    match tar.finish() {
        Ok(_) => (),
        Err(e) => {
            error!(?e);
            dbg!(&e);
            return Err(miette!("tar error: {e}"));
        }
    }
    fs::remove_dir_all(temp_dir).into_diagnostic()
}

fn process_service_items(items: &Vec<ServiceItem>) -> Result<String> {
    Ok(items
        .iter()
        .filter_map(|item| {
            let ron = ron::ser::to_string(item);
            ron.ok()
        })
        .collect())
}

// async fn process_song(
//     database_id: i32,
//     db: &mut SqliteConnection,
// ) -> Result<Value> {
//     let song = get_song_from_db(database_id, db).await?;
//     let song_ron = ron::to_value(&song)?;
//     let kind_ron = ron::to_value(ServiceItemKind::Song)?;
//     let json =
//         serde_json::json!({"item": song_json, "kind": kind_json});
//     Ok(json)
// }

// async fn process_image(
//     database_id: i32,
//     db: &mut SqliteConnection,
// ) -> Result<Value> {
//     let image = get_image_from_db(database_id, db).await?;
//     let image_json = serde_json::to_value(&image)?;
//     let kind_json = serde_json::to_value(ServiceItemKind::Image)?;
//     let json =
//         serde_json::json!({"item": image_json, "kind": kind_json});
//     Ok(json)
// }

// async fn process_video(
//     database_id: i32,
//     db: &mut SqliteConnection,
// ) -> Result<Value> {
//     let video = get_video_from_db(database_id, db).await?;
//     let video_json = serde_json::to_value(&video)?;
//     let kind_json = serde_json::to_value(ServiceItemKind::Video)?;
//     let json =
//         serde_json::json!({"item": video_json, "kind": kind_json});
//     Ok(json)
// }

// async fn process_presentation(
//     database_id: i32,
//     db: &mut SqliteConnection,
// ) -> Result<Value> {
//     let presentation =
//         get_presentation_from_db(database_id, db).await?;
//     let presentation_json = serde_json::to_value(&presentation)?;
//     let kind_json = match presentation.kind {
//         PresKind::Html => serde_json::to_value(
//             ServiceItemKind::Presentation(PresKind::Html),
//         )?,
//         PresKind::Pdf => serde_json::to_value(
//             ServiceItemKind::Presentation(PresKind::Pdf),
//         )?,
//         PresKind::Generic => serde_json::to_value(
//             ServiceItemKind::Presentation(PresKind::Generic),
//         )?,
//     };
//     let json = serde_json::json!({"item": presentation_json, "kind": kind_json});
//     Ok(json)
// }

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::core::{slide::Slide, songs::Song};

    use super::*;
    use pretty_assertions::assert_eq;
    use sqlx::{Connection, SqliteConnection};

    async fn get_db() -> SqliteConnection {
        let db_url = String::from("sqlite://./test.db");
        SqliteConnection::connect(&db_url).await.expect("problems")
    }

    // #[tokio::test(flavor = "current_thread")]
    // async fn test_process_song() {
    //     let mut db = get_db().await;
    //     let result = process_song(7, &mut db).await;
    //     let json_song_file = PathBuf::from("./test/test_song.json");
    //     if let Ok(path) = canonicalize(json_song_file) {
    //         debug!(file = ?&path);
    //         if let Ok(s) = fs::read_to_string(path) {
    //             debug!(s);
    //             match result {
    //                 Ok(json) => assert_eq!(json.to_string(), s),
    //                 Err(e) => panic!(
    //                     "There was an error in processing the song: {e}"
    //                 ),
    //             }
    //         } else {
    //             panic!("String wasn't read from file");
    //         }
    //     } else {
    //         panic!("Cannot find absolute path to test_song.json");
    //     }
    // }

    // #[tokio::test(flavor = "current_thread")]
    // async fn test_process_image() {
    //     let mut db = get_db().await;
    //     let result = process_image(3, &mut db).await;
    //     let json_image_file = PathBuf::from("./test/test_image.json");
    //     if let Ok(path) = canonicalize(json_image_file) {
    //         debug!(file = ?&path);
    //         if let Ok(s) = fs::read_to_string(path) {
    //             debug!(s);
    //             match result {
    //                 Ok(json) => assert_eq!(json.to_string(), s),
    //                 Err(e) => panic!(
    //                     "There was an error in processing the image: {e}"
    //                 ),
    //             }
    //         } else {
    //             panic!("String wasn't read from file");
    //         }
    //     } else {
    //         panic!("Cannot find absolute path to test_image.json");
    //     }
    // }

    // #[tokio::test(flavor = "current_thread")]
    // async fn test_process_video() {
    //     let mut db = get_db().await;
    //     let result = process_video(73, &mut db).await;
    //     let json_video_file = PathBuf::from("./test/test_video.json");
    //     if let Ok(path) = canonicalize(json_video_file) {
    //         debug!(file = ?&path);
    //         if let Ok(s) = fs::read_to_string(path) {
    //             debug!(s);
    //             match result {
    //                 Ok(json) => assert_eq!(json.to_string(), s),
    //                 Err(e) => panic!(
    //                     "There was an error in processing the video: {e}"
    //                 ),
    //             }
    //         } else {
    //             panic!("String wasn't read from file");
    //         }
    //     } else {
    //         panic!("Cannot find absolute path to test_video.json");
    //     }
    // }

    // #[tokio::test(flavor = "current_thread")]
    // async fn test_process_presentation() {
    //     let mut db = get_db().await;
    //     let result = process_presentation(54, &mut db).await;
    //     let json_presentation_file =
    //         PathBuf::from("./test/test_presentation.json");
    //     if let Ok(path) = canonicalize(json_presentation_file) {
    //         debug!(file = ?&path);
    //         if let Ok(s) = fs::read_to_string(path) {
    //             debug!(s);
    //             match result {
    //                 Ok(json) => assert_eq!(json.to_string(), s),
    //                 Err(e) => panic!(
    //                     "There was an error in processing the presentation: {e}"
    //                 ),
    //             }
    //         } else {
    //             panic!("String wasn't read from file");
    //         }
    //     } else {
    //         panic!(
    //             "Cannot find absolute path to test_presentation.json"
    //         );
    //     }
    // }

    fn get_items() -> Vec<ServiceItem> {
        let items = vec![ServiceItem {
            database_id: 7,
            kind: ServiceItemKind::Song(Song::default()),
            id: 0,
            title: "Glory Glory".into(),
            slides: vec![Slide::default()],
        }];
        items
    }

    // #[tokio::test]
    // async fn test_service_items() {
    //     let mut db = get_db().await;
    //     let items = get_items();
    //     let json_item_file =
    //         PathBuf::from("./test/test_service_items.json");
    //     let result = process_service_items(&items, &mut db).await;
    //     if let Ok(path) = canonicalize(json_item_file) {
    //         if let Ok(s) = fs::read_to_string(path) {
    //             match result {
    //                 Ok(strings) => assert_eq!(strings.to_string(), s),
    //                 Err(e) => panic!("There was an error: {e}"),
    //             }
    //         }
    //     }
    // }

    // #[tokio::test]
    // async fn test_save() {
    //     let path = PathBuf::from("~/dev/lumina/src/rust/core/test.pres");
    //     let list = get_items();
    //     match save(list, path).await {
    //         Ok(_) => assert!(true),
    //         Err(e) => panic!("There was an error: {e}"),
    //     }
    // }

    #[test]
    fn test_save() {
        let path =
            PathBuf::from("/home/chris/dev/lumina-iced/test.pres");
        let list = get_items();
        match save(list, path) {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "{e}"),
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
