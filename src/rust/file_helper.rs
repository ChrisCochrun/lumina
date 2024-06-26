// The purpose of this file is to provide validation
// of whether or not a file exists
#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, name)]
        #[qproperty(QString, file_path)]
        type FileHelper = super::FileHelperRust;

        #[qinvokable]
        fn load(
            self: Pin<&mut FileHelper>,
            file: QUrl,
        ) -> Vec<String>;

        #[qinvokable]
        fn validate(self: Pin<&mut FileHelper>, file: QUrl) -> bool;
        #[qinvokable]
        fn save_file(self: Pin<&mut FileHelper>) -> QUrl;
        #[qinvokable]
        fn load_file(
            self: Pin<&mut FileHelper>,
            title: QString,
            filter: QString,
        ) -> QUrl;
    }
}

use cxx_qt_lib::{QString, QUrl};
use rfd::FileDialog;
use std::{path::Path, pin::Pin};
use tracing::{debug, error};

#[derive(Clone)]
pub struct FileHelperRust {
    name: QString,
    file_path: QString,
}

impl Default for FileHelperRust {
    fn default() -> Self {
        Self {
            name: QString::from(""),
            file_path: QString::from(""),
        }
    }
}

impl qobject::FileHelper {
    pub fn load(self: Pin<&mut Self>, file: QUrl) -> Vec<String> {
        println!("{file}");
        vec!["hi".to_string()]
    }

    pub fn validate(self: Pin<&mut Self>, file: QUrl) -> bool {
        let file_string = file.to_string();
        let file_string = file_string.strip_prefix("file://");
        match file_string {
            Some(file) => {
                let exists = Path::new(&file).exists();
                debug!(file, exists);
                exists
            }
            None => {
                let exists = Path::new(&file.to_string()).exists();
                debug!(?file, exists);
                exists
            }
        }
    }

    pub fn save_file(self: Pin<&mut Self>) -> QUrl {
        debug!("Saving file in rust");
        let file = FileDialog::new()
            .set_file_name("NVTFC.pres")
            .set_title("Save Presentation")
            .save_file();
        if let Some(file) = file {
            println!("saving-file: {:?}", file);
            let mut string =
                String::from(file.to_str().unwrap_or(""));
            if string.is_empty() {
                QUrl::default()
            } else {
                string.insert_str(0, "file://");
                QUrl::from(string.as_str())
            }
        } else {
            error!("There was an error, is xdg-desktop-portals correctly setup?");
            QUrl::default()
        }
    }

    pub fn load_file(
        self: Pin<&mut Self>,
        title: QString,
        filter: QString,
    ) -> QUrl {
        let video_filters = [
            "mp4", "webm", "avi", "mkv", "MP4", "WEBM", "AVI", "MKV",
        ];
        let image_filters = [
            "jpg", "png", "gif", "jpeg", "JPG", "PNG", "webp", "gif",
        ];
        let audio_filters = ["mp3", "opus", "ogg", "flac", "wav"];
        let pres_filter = ["pres"];
        let title = title.to_string();
        let filter = filter.to_string();
        let mut file = FileDialog::new().set_title(title);
        match filter.as_str() {
            "video" => {
                file = file.add_filter(filter, &video_filters);
            }
            "image" => {
                file = file.add_filter(filter, &image_filters);
            }
            "audio" => {
                file = file.add_filter(filter, &audio_filters);
            }
            "pres" => {
                file = file.add_filter(filter, &pres_filter);
            }
            _ => debug!("nothing"),
        };
        debug!("trying to load file");
        let file = file.pick_file();
        if let Some(file) = file {
            println!("loading-file: {:?}", file);
            let mut string =
                String::from(file.to_str().unwrap_or(""));
            if string.is_empty() {
                QUrl::default()
            } else {
                string.insert_str(0, "file://");
                QUrl::from(string.as_str())
            }
        } else {
            error!("Couldn't load file, is xdg-desktop-portals correctly setup?");
            QUrl::default()
        }
    }
}
