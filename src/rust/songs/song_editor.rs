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
        // type CxxSongs = crate::songs::song_model::qobject::SongModel;
    }

    #[derive(Clone, Debug)]
    #[cxx_qt::qobject]
    pub struct SongEditor {
        #[qproperty]
        title: QString,
        #[qproperty]
        lyrics: QString,
        #[qproperty]
        author: QString,
        #[qproperty]
        ccli: QString,
        #[qproperty]
        audio: QUrl,
        #[qproperty]
        verse_order: QString,
        #[qproperty]
        verse_order_format: bool,
        #[qproperty]
        background: QUrl,
        #[qproperty]
        background_type: QString,
        #[qproperty]
        horizontal_text_alignment: QString,
        #[qproperty]
        vertical_text_alignment: QString,
        #[qproperty]
        font: QString,
        #[qproperty]
        font_size: i32,
        // #[qproperty]
        // song_model: *mut CxxSongs,
    }

    impl Default for SongEditor {
        fn default() -> Self {
            Self {
                title: QString::default(),
                lyrics: QString::default(),
                author: QString::default(),
                ccli: QString::default(),
                audio: QUrl::default(),
                verse_order: QString::default(),
                verse_order_format: true,
                background: QUrl::default(),
                background_type: QString::default(),
                horizontal_text_alignment: QString::default(),
                vertical_text_alignment: QString::default(),
                font: QString::default(),
                font_size: 50,
                // song_model: std::ptr::null_mut(),
            }
        }
    }

    impl qobject::SongEditor {
        fn idk(mut self: Pin<&mut Self>) {
            // let mut model = self.song_model().as_mut().unwrap();
            // let pinned_model = Pin::new_unchecked(model);
            // pinned_model.update_ccli(0, QString::from("idk"));
            todo!()
        }
        // #[qinvokable]
        // fn set_song(
        //     mut self: Pin<&mut Self>,
        //     title: QString,
        //     lyrics: QString,
        //     author: QString,
        //     ccli: QString,
        //     audio: QUrl,
        //     verse_order: QString,
        //     background: QUrl,
        //     background_type: QString,
        //     horizontal_text_alignment: QString,
        //     vertical_text_alignment: QString,
        //     font: QString,
        //     font_size: i32,
        // ) -> bool {
        //     self.as_mut().set_title(title);
        //     self.as_mut().set_lyrics(lyrics);
        //     self.as_mut().set_author(author);
        //     self.as_mut().set_ccli(ccli);
        //     self.as_mut().set_audio(audio);
        //     self.as_mut().set_verse_order(verse_order);
        //     self.as_mut().set_background(background);
        //     self.as_mut().set_background_type(background_type);
        //     self.as_mut().set_horizontal_text_alignment(
        //         horizontal_text_alignment,
        //     );
        //     self.as_mut()
        //         .set_vertical_text_alignment(vertical_text_alignment);
        //     self.as_mut().set_font(font);
        //     self.as_mut().set_font_size(font_size);
        //     true
        // }
    }
}
