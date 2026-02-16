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
use tracing::error;
use zstd::Encoder;

pub fn save(
    list: Vec<ServiceItem>,
    path: impl AsRef<Path>,
    overwrite: bool,
) -> Result<()> {
    let path = path.as_ref();
    if overwrite && path.exists() {
        fs::remove_file(path).into_diagnostic()?;
    }
    let save_file = File::create(path).into_diagnostic()?;
    let ron = process_service_items(&list)?;

    let encoder = Encoder::new(save_file, 3)
        .expect("file encoder shouldn't fail")
        .auto_finish();
    let mut tar = Builder::new(encoder);
    let mut temp_dir = dirs::data_dir().expect(
        "there should be a data directory, ~/.local/share/ for linux, but couldn't find it",
    );
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
            match f.write(ron.as_bytes()) {
                Ok(size) => {
                    dbg!(size);
                }
                Err(e) => {
                    error!(?e);
                    dbg!(&e);
                    return Err(miette!("PROBS: {e}"));
                }
            }
            match tar.append_file("serviceitems.ron", &mut f) {
                Ok(_) => {
                    dbg!(
                        "should have added serviceitems.ron to the file"
                    );
                }
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
        if let Some(path) = audio {
            let file_name = path.file_name().unwrap_or_default();
            let mut file = fs::File::open(&path).into_diagnostic()?;
            tar.append_file(file_name, &mut file)
                .into_diagnostic()?;
        }
        if let Some(background) = background {
            let path = background.path;
            let file_name = path.file_name().unwrap_or_default();
            let mut file = fs::File::open(&path).into_diagnostic()?;
            tar.append_file(file_name, &mut file)
                .into_diagnostic()?;
        }
    }

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
        .filter_map(|item| ron::ser::to_string(item).ok())
        .collect())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::{slide::Slide, songs::Song};
    use std::path::PathBuf;

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

    #[test]
    fn test_save() {
        let path = PathBuf::from("./test.pres");
        let list = get_items();
        match save(list, &path, true) {
            Ok(_) => {
                assert!(true);
                assert!(path.is_file());
                let Ok(file) = fs::File::open(path) else {
                    return assert!(false, "couldn't open file");
                };
                let Ok(size) = file.metadata().map(|data| data.len())
                else {
                    return assert!(
                        false,
                        "couldn't get file metadata"
                    );
                };
                assert!(size > 0);
            }
            Err(e) => assert!(false, "{e}"),
        }
    }
}
