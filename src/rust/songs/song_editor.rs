#[cxx_qt::bridge]
mod song_editor {
    use crate::songs::song_model::qobject::SongModel;

    unsafe extern "C++" {
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant =
            cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
        #[cxx_name = "SongModel"]
        type SongModel = SongModel;
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
        #[qproperty]
        song_model: *mut SongModel,
    }

    impl SongEditor {
        fn idk(mut self: Pin<&mut Self>) {
            // let mut model = self.song_model().as_mut().unwrap();
            // let pinned_model = Pin::new_unchecked(model);
            // pinned_model.update_ccli(0, QString::from("idk"));
            todo!();
        }
    }
}
