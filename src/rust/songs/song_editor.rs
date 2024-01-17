// use crate::songs::song_model::song_model::SongModel;

#[cxx_qt::bridge]
pub mod song_editor {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant =
            cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
        // #[cxx_name = "SongModel"]
        // type SongModel = crate::songs::song_model::qobject::SongModel;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, title)]
        #[qproperty(QString, lyrics)]
        #[qproperty(QString, author)]
        #[qproperty(QString, ccli)]
        #[qproperty(QString, audio)]
        #[qproperty(QString, verse_order)]
        #[qproperty(bool, verse_order_error)]
        #[qproperty(QString, background)]
        #[qproperty(QString, background_type)]
        #[qproperty(QString, horizontal_text_alignment)]
        #[qproperty(QString, vertical_text_alignment)]
        #[qproperty(QString, font)]
        #[qproperty(i32, font_size)]
        #[qproperty(bool, background_exists)]
        #[qproperty(bool, audio_exists)]
        // #[qproperty(*mut SongModel, song_model)]
        type SongEditor = super::SongEditorRust;

        #[qinvokable]
        fn check_verse_order(self: Pin<&mut SongEditor>);
        #[qinvokable]
        fn check_files(self: Pin<&mut SongEditor>);
    }
}

use cxx_qt_lib::QString;
use std::{path::PathBuf, pin::Pin};
use tracing::{debug, debug_span, error, info, instrument};

#[derive(Clone, Debug)]
pub struct SongEditorRust {
    title: QString,
    lyrics: QString,
    author: QString,
    ccli: QString,
    audio: QString,
    verse_order: QString,
    verse_order_error: bool,
    background: QString,
    background_type: QString,
    horizontal_text_alignment: QString,
    vertical_text_alignment: QString,
    font: QString,
    font_size: i32,
    background_exists: bool,
    audio_exists: bool,
    // song_model: *mut SongModel,
}

impl Default for SongEditorRust {
    fn default() -> Self {
        Self {
            title: QString::default(),
            lyrics: QString::default(),
            author: QString::default(),
            ccli: QString::default(),
            audio: QString::default(),
            verse_order: QString::default(),
            verse_order_error: false,
            background: QString::default(),
            background_type: QString::default(),
            horizontal_text_alignment: QString::default(),
            vertical_text_alignment: QString::default(),
            font: QString::default(),
            font_size: 50,
            background_exists: true,
            audio_exists: true,
            // song_model: std::ptr::null_mut(),
        }
    }
}

impl song_editor::SongEditor {
    fn idk(mut self: Pin<&mut Self>) {
        // if let Some(model) = unsafe { self.song_model().as_mut() } {
        //     let pinned_model = unsafe { Pin::new_unchecked(model) };
        //     pinned_model.update_ccli(0, QString::from("idk"));
        // }
        todo!();
    }

    pub fn check_verse_order(mut self: Pin<&mut Self>) {
        let vo = self.verse_order().to_string();
        let split = vo.split(" ");
        debug!(verse_order = ?vo, iterator = ?split);
        for s in split {
            if s.contains(",") || s.is_empty() {
                self.as_mut().set_verse_order_error(true);
            } else {
                self.as_mut().set_verse_order_error(false);
            }
        }
    }

    pub fn check_files(mut self: Pin<&mut Self>) {
        let background = PathBuf::from(
            self.background()
                .clone()
                .to_string()
                .trim_start_matches("file://"),
        );
        let audio = PathBuf::from(
            self.audio()
                .clone()
                .to_string()
                .trim_start_matches("file://"),
        );
        debug!(
            background = background.exists(),
            audio = audio.exists()
        );
        self.as_mut().set_background_exists(background.exists());
        self.as_mut().set_audio_exists(audio.exists());
    }
}
