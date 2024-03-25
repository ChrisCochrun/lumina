#[cxx_qt::bridge]
pub mod song_model {
    unsafe extern "C++" {
        include!(< QAbstractListModel >);
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray =
            cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant =
            cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
    }

    #[qenum(SongModel)]
    enum SongRoles {
        Id,
        Title,
        Lyrics,
        Author,
        Ccli,
        Audio,
        VerseOrder,
        Background,
        BackgroundType,
        HorizontalTextAlignment,
        VerticalTextAlignment,
        Font,
        FontSize,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QAbstractListModel"]
        #[qml_element]
        #[qproperty(i32, count)]
        type SongModel = super::SongModelRust;

        #[inherit]
        #[qsignal]
        fn data_changed(
            self: Pin<&mut SongModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );
        #[qsignal]
        fn title_changed(self: Pin<&mut SongModel>);
        #[qsignal]
        fn font_size_changed(self: Pin<&mut SongModel>);
        #[qsignal]
        fn background_changed(self: Pin<&mut SongModel>);

        #[qinvokable]
        fn clear(self: Pin<&mut SongModel>);
        #[qinvokable]
        fn setup(self: Pin<&mut SongModel>);
        #[qinvokable]
        fn remove_item(self: Pin<&mut SongModel>, index: i32)
            -> bool;
        #[qinvokable]
        fn new_song(self: Pin<&mut SongModel>) -> bool;
        #[qinvokable]
        fn get_item(
            self: Pin<&mut SongModel>,
            index: i32,
        ) -> QMap_QString_QVariant;
        #[qinvokable]
        fn get_lyric_list(
            self: Pin<&mut SongModel>,
            index: i32,
        ) -> QStringList;
        #[qinvokable]
        fn update_lyrics(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_lyrics: QString,
        ) -> bool;
        #[qinvokable]
        fn update_author(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_author: QString,
        ) -> bool;
        #[qinvokable]
        fn update_title(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_title: QString,
        ) -> bool;
        #[qinvokable]
        fn update_ccli(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_ccli: QString,
        ) -> bool;
        #[qinvokable]
        fn update_verse_order(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_verse_order: QString,
        ) -> bool;
        #[qinvokable]
        fn update_background(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_background: QString,
        ) -> bool;
        #[qinvokable]
        fn update_background_type(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_background_type: QString,
        ) -> bool;
        #[qinvokable]
        fn update_horizontal_text_alignment(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_horizontal_text_alignment: QString,
        ) -> bool;
        #[qinvokable]
        fn update_vertical_text_alignment(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_vertical_text_alignment: QString,
        ) -> bool;
        #[qinvokable]
        fn update_font(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_font: QString,
        ) -> bool;
        #[qinvokable]
        fn update_font_size(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_font_size: i32,
        ) -> bool;
        #[qinvokable]
        fn update_audio(
            self: Pin<&mut SongModel>,
            index: i32,
            updated_audio: QString,
        ) -> bool;
    }

    impl cxx_qt::Threading for SongModel {}

    unsafe extern "RustQt" {
        #[inherit]
        unsafe fn begin_insert_rows(
            self: Pin<&mut SongModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_insert_rows(self: Pin<&mut SongModel>);

        #[inherit]
        unsafe fn begin_remove_rows(
            self: Pin<&mut SongModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn begin_move_rows(
            self: Pin<&mut SongModel>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;

        #[inherit]
        unsafe fn end_move_rows(self: Pin<&mut SongModel>);

        #[inherit]
        unsafe fn end_remove_rows(self: Pin<&mut SongModel>);

        #[inherit]
        unsafe fn begin_reset_model(self: Pin<&mut SongModel>);

        #[inherit]
        unsafe fn end_reset_model(self: Pin<&mut SongModel>);

        #[inherit]
        fn can_fetch_more(
            self: &SongModel,
            parent: &QModelIndex,
        ) -> bool;

        #[inherit]
        fn index(
            self: &SongModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        fn data(
            self: &SongModel,
            index: &QModelIndex,
            role: i32,
        ) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        fn role_names(self: &SongModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn row_count(self: &SongModel, _parent: &QModelIndex) -> i32;
    }
}

use crate::models::*;
use crate::schema::songs::dsl::*;
use crate::songs::song_editor::song_editor::QList_QString;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{
    QByteArray, QModelIndex, QString, QStringList, QVariant,
};
use diesel::sqlite::SqliteConnection;
use diesel::{delete, insert_into, prelude::*, update};
use std::collections::HashMap;
use std::pin::Pin;
use tracing::{debug, debug_span, error, info, instrument};

use self::song_model::{
    QHash_i32_QByteArray, QMap_QString_QVariant, QVector_i32,
    SongRoles,
};

#[derive(Clone, Debug)]
pub struct Song {
    id: i32,
    title: String,
    lyrics: String,
    author: String,
    ccli: String,
    audio: String,
    verse_order: String,
    background: String,
    background_type: String,
    horizontal_text_alignment: String,
    vertical_text_alignment: String,
    font: String,
    font_size: i32,
}

impl Default for Song {
    fn default() -> Self {
        Self {
            horizontal_text_alignment: String::from("Center"),
            vertical_text_alignment: String::from("Center"),
            font: String::from("Quicksand Bold"),
            font_size: 50,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct SongModelRust {
    count: i32,
    highest_id: i32,
    songs: Vec<Song>,
}

impl song_model::SongModel {
    pub fn clear(mut self: Pin<&mut Self>) {
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().songs.clear();
            self.as_mut().end_reset_model();
        }
    }

    pub fn setup(mut self: Pin<&mut Self>) {
        let db = &mut self.as_mut().get_db();
        run_migrations(db);

        let results = songs
            .load::<crate::models::Song>(db)
            .expect("NO TABLE?????????????");
        self.as_mut().rust_mut().highest_id = 0;

        println!("SHOWING SONGS");
        println!("--------------");
        for song in results {
            println!("{}", song.title);
            println!("{}", song.id);
            println!(
                "{}",
                song.background.clone().unwrap_or_default()
            );
            println!("--------------");
            if self.as_mut().highest_id < song.id {
                self.as_mut().rust_mut().highest_id = song.id;
            }

            let song = Song {
                id: song.id,
                title: song.title,
                lyrics: song.lyrics.unwrap_or_default(),
                author: song.author.unwrap_or_default(),
                ccli: song.ccli.unwrap_or_default(),
                audio: song.audio.unwrap_or_default(),
                verse_order: song.verse_order.unwrap_or_default(),
                background: song.background.unwrap_or_default(),
                background_type: song
                    .background_type
                    .unwrap_or_default(),
                horizontal_text_alignment: song
                    .horizontal_text_alignment
                    .unwrap_or_default(),
                vertical_text_alignment: song
                    .vertical_text_alignment
                    .unwrap_or_default(),
                font: song.font.unwrap_or_default(),
                font_size: song.font_size.unwrap_or_default(),
            };

            self.as_mut().add_song(song);
        }
        println!("--------------------------------------");
        println!("{:?}", self.as_mut().songs);
        println!("--------------------------------------");
    }

    pub fn remove_item(mut self: Pin<&mut Self>, index: i32) -> bool {
        if index < 0 || (index as usize) >= self.songs.len() {
            return false;
        }
        let db = &mut self.as_mut().get_db();

        let song_id = self.songs.get(index as usize).unwrap().id;

        let result = delete(songs.filter(id.eq(song_id))).execute(db);

        match result {
            Ok(_i) => {
                unsafe {
                    self.as_mut().begin_remove_rows(
                        &QModelIndex::default(),
                        index,
                        index,
                    );
                    self.as_mut()
                        .rust_mut()
                        .songs
                        .remove(index as usize);
                    self.as_mut().end_remove_rows();
                }
                println!("removed-item-at-index: {:?}", song_id);
                println!("new-Vec: {:?}", self.as_mut().songs);
                true
            }
            Err(_e) => {
                println!("Cannot connect to database");
                false
            }
        }
    }

    fn get_db(self: Pin<&mut Self>) -> SqliteConnection {
        let mut data = dirs::data_local_dir().unwrap();
        data.push("lumina");
        data.push("library-db.sqlite3");
        let mut db_url = String::from("sqlite://");
        db_url.push_str(data.to_str().unwrap());
        println!("DB: {:?}", db_url);

        SqliteConnection::establish(&db_url).unwrap_or_else(|_| {
            panic!("error connecting to {}", db_url)
        })
    }

    pub fn new_song(mut self: Pin<&mut Self>) -> bool {
        let song_id = self.rust().highest_id + 1;
        let song_title = String::from("title");
        let db = &mut self.as_mut().get_db();

        let song = Song {
            id: song_id,
            title: song_title.clone(),
            ..Default::default()
        };

        let result = insert_into(songs)
            .values((
                id.eq(&song_id),
                title.eq(&song_title),
                lyrics.eq(&song.lyrics),
                author.eq(&song.author),
                ccli.eq(&song.ccli),
                audio.eq(&song.audio),
                verse_order.eq(&song.verse_order),
                background.eq(&song.background),
                background_type.eq(&song.background_type),
                horizontal_text_alignment
                    .eq(&song.horizontal_text_alignment),
                vertical_text_alignment
                    .eq(&song.vertical_text_alignment),
                font.eq(&song.font),
                font_size.eq(&song.font_size),
            ))
            .execute(db);
        println!("{:?}", result);

        match result {
            Ok(_i) => {
                self.as_mut().add_song(song);
                println!("{:?}", self.as_mut().songs);
                true
            }
            Err(_e) => {
                println!("Cannot connect to database");
                false
            }
        }
    }

    fn add_song(mut self: Pin<&mut Self>, song: self::Song) {
        let index = self.as_ref().songs.len() as i32;
        println!("{:?}", song);

        let count = self.as_ref().count;
        self.as_mut().set_count(count + 1);

        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut().rust_mut().songs.push(song);
            self.as_mut().end_insert_rows();
        }
    }

    fn get_indices(
        mut self: Pin<&mut Self>,
        song_id: i32,
        role: SongRoles,
    ) -> (usize, QModelIndex, QVector_i32) {
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.as_ref().get_role(role));
        if let Some(index) =
            self.as_ref().songs.iter().position(|x| x.id == song_id)
        {
            let model_index = self.as_ref().index(
                index as i32,
                0,
                &QModelIndex::default(),
            );
            (index, model_index, vector_roles)
        } else {
            error!(song_id, "This song appears to be missing");
            (0, QModelIndex::default(), vector_roles)
        }
    }

    pub fn update_title(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_title: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::Title);
        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(title.eq(updated_title.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                debug!(_i, index);
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(?song, title = updated_title.to_string());
                    song.title = updated_title.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_lyrics(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_lyrics: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::Lyrics);
        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(lyrics.eq(updated_lyrics.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(
                        ?song,
                        lyrics = updated_lyrics.to_string()
                    );
                    song.lyrics = updated_lyrics.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_author(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_author: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::Author);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(author.eq(updated_author.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(
                        ?song,
                        author = updated_author.to_string()
                    );
                    song.author = updated_author.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_audio(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_audio: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::Audio);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(audio.eq(updated_audio.to_string()))
            .execute(db);
        debug!(?result);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(?song, audio = updated_audio.to_string());
                    song.audio = updated_audio.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_ccli(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_ccli: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::Ccli);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(ccli.eq(updated_ccli.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(?song, ccli = updated_ccli.to_string());
                    song.ccli = updated_ccli.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_verse_order(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_verse_order: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::VerseOrder);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(verse_order.eq(updated_verse_order.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(
                        ?song,
                        verse_order = updated_verse_order.to_string()
                    );
                    song.verse_order =
                        updated_verse_order.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_background(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_background: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::Background);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(background.eq(updated_background.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    song.background = updated_background.to_string();
                    debug!(
                        background = ?updated_background,
                        model_index = ?model_index,
                        roles = vector_roles.get(0)
                    );
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    self.as_mut().background_changed();
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_background_type(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_background_type: QString,
    ) -> bool {
        let (index, model_index, vector_roles) = self
            .as_mut()
            .get_indices(song_id, SongRoles::BackgroundType);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(
                background_type
                    .eq(updated_background_type.to_string()),
            )
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(
                        ?song,
                        background_type =
                            updated_background_type.to_string()
                    );
                    song.background_type =
                        updated_background_type.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_horizontal_text_alignment(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_horizontal_text_alignment: QString,
    ) -> bool {
        let (index, model_index, vector_roles) = self
            .as_mut()
            .get_indices(song_id, SongRoles::HorizontalTextAlignment);

        let db = &mut self.as_mut().get_db();
        let result =
            update(songs.filter(id.eq(song_id)))
                .set(horizontal_text_alignment.eq(
                    updated_horizontal_text_alignment.to_string(),
                ))
                .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(
                        ?song,
                        horizontal =
                            updated_horizontal_text_alignment
                                .to_string()
                    );
                    song.horizontal_text_alignment =
                        updated_horizontal_text_alignment.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_vertical_text_alignment(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_vertical_text_alignment: QString,
    ) -> bool {
        let (index, model_index, vector_roles) = self
            .as_mut()
            .get_indices(song_id, SongRoles::VerticalTextAlignment);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(
                vertical_text_alignment
                    .eq(updated_vertical_text_alignment.to_string()),
            )
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(
                        ?song,
                        vertical = updated_vertical_text_alignment
                            .to_string()
                    );
                    song.vertical_text_alignment =
                        updated_vertical_text_alignment.to_string();
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_font(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_font: QString,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::Font);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(font.eq(updated_font.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    song.font = updated_font.to_string();
                    debug!(?song, font = updated_font.to_string());
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn update_font_size(
        mut self: Pin<&mut Self>,
        song_id: i32,
        updated_font_size: i32,
    ) -> bool {
        let (index, model_index, vector_roles) =
            self.as_mut().get_indices(song_id, SongRoles::FontSize);

        let db = &mut self.as_mut().get_db();
        let result = update(songs.filter(id.eq(song_id)))
            .set(font_size.eq(updated_font_size))
            .execute(db);
        match result {
            Ok(_i) => {
                if let Some(song) =
                    self.as_mut().rust_mut().songs.get_mut(index)
                {
                    debug!(
                        ?song,
                        font_size = updated_font_size.to_string()
                    );
                    song.font_size = updated_font_size;
                    self.as_mut().data_changed(
                        &model_index,
                        &model_index,
                        &vector_roles,
                    );
                    true
                } else {
                    false
                }
            }
            Err(_e) => false,
        }
    }

    pub fn get_item(
        self: Pin<&mut Self>,
        index: i32,
    ) -> QMap_QString_QVariant {
        debug!(index);
        let mut qvariantmap = QMap_QString_QVariant::default();
        let idx = self.index(index, 0, &QModelIndex::default());
        if !idx.is_valid() {
            return qvariantmap;
        }
        let role_names = self.as_ref().role_names();
        let role_names_iter = role_names.iter();
        if let Some(song) = self.rust().songs.get(index as usize) {
            debug!(?song);
            for i in role_names_iter {
                qvariantmap.insert(
                    QString::from(&i.1.to_string()),
                    self.as_ref().data(&idx, *i.0),
                );
            }
        };
        qvariantmap
    }

    pub fn get_lyric_list(
        mut self: Pin<&mut Self>,
        index: i32,
    ) -> QStringList {
        println!("LYRIC_LIST: {index}");
        let mut lyric_list = QList_QString::default();
        let idx = self.index(index, 0, &QModelIndex::default());
        if !idx.is_valid() {
            return QStringList::default();
        }
        if let Some(song) = self.rust().songs.get(index as usize) {
            if song.lyrics.is_empty() {
                return QStringList::default();
            }
            let raw_lyrics = song.lyrics.clone();
            println!("raw-lyrics: {:?}", raw_lyrics);
            let vorder: Vec<&str> =
                song.verse_order.split(' ').collect();
            let keywords = vec![
                "Verse 1", "Verse 2", "Verse 3", "Verse 4",
                "Verse 5", "Verse 6", "Verse 7", "Verse 8",
                "Chorus 1", "Chorus 2", "Chorus 3", "Chorus 4",
                "Bridge 1", "Bridge 2", "Bridge 3", "Bridge 4",
                "Intro 1", "Intro 2", "Ending 1", "Ending 2",
                "Other 1", "Other 2", "Other 3", "Other 4",
            ];
            let mut first_item = true;

            let mut lyric_map = HashMap::new();
            let mut verse_title = String::from("");
            let mut lyric = String::from("");
            for (i, line) in raw_lyrics.split("\n").enumerate() {
                if keywords.contains(&line) {
                    if i != 0 {
                        // println!("{verse_title}");
                        // println!("{lyric}");
                        lyric_map.insert(verse_title, lyric);
                        lyric = String::from("");
                        verse_title = line.to_string();
                        // println!("{line}");
                        // println!("\n");
                    } else {
                        verse_title = line.to_string();
                        // println!("{line}");
                        // println!("\n");
                    }
                } else {
                    lyric.push_str(line);
                    lyric.push_str("\n");
                }
            }
            lyric_map.insert(verse_title, lyric);
            // println!("da-map: {:?}", lyric_map);

            for mut verse in vorder {
                let mut verse_name = "";
                // debug!(verse = verse);
                for word in keywords.clone() {
                    let end_verse =
                        verse.get(1..2).unwrap_or_default();
                    let beg_verse =
                        verse.get(0..1).unwrap_or_default();
                    // println!(
                    //     "verse: {:?}, beginning: {:?}, end: {:?}, word: {:?}",
                    //     verse, beg_verse, end_verse, word
                    // );
                    if word.starts_with(beg_verse)
                        && word.ends_with(end_verse)
                    {
                        verse_name = word;
                        // println!("TITLE: {verse_name}");
                        continue;
                    }
                }
                if let Some(lyric) = lyric_map.get(verse_name) {
                    if lyric.contains("\n\n") {
                        let split_lyrics: Vec<&str> =
                            lyric.split("\n\n").collect();
                        for lyric in split_lyrics {
                            if lyric == "" {
                                continue;
                            }
                            lyric_list.append(QString::from(lyric));
                        }
                        continue;
                    }
                    lyric_list.append(QString::from(lyric));
                } else {
                    println!("NOT WORKING!");
                };
            }
            for lyric in lyric_list.iter() {
                // println!("da-list: {:?}", lyric);
                debug!(lyric = ?lyric)
            }
        }
        QStringList::from(&lyric_list)
    }

    fn get_role(&self, role: SongRoles) -> i32 {
        match role {
            SongRoles::Id => 0,
            SongRoles::Title => 1,
            SongRoles::Lyrics => 2,
            SongRoles::Author => 3,
            SongRoles::Ccli => 4,
            SongRoles::Audio => 5,
            SongRoles::VerseOrder => 6,
            SongRoles::Background => 7,
            SongRoles::BackgroundType => 8,
            SongRoles::HorizontalTextAlignment => 9,
            SongRoles::VerticalTextAlignment => 10,
            SongRoles::Font => 11,
            SongRoles::FontSize => 12,
            _ => 0,
        }
    }
}

// QAbstractListModel implementation
impl song_model::SongModel {
    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = SongRoles { repr: role };
        if let Some(song) = self.songs.get(index.row() as usize) {
            return match role {
                SongRoles::Id => QVariant::from(&song.id),
                SongRoles::Title => {
                    QVariant::from(&QString::from(&song.title))
                }
                SongRoles::Lyrics => {
                    QVariant::from(&QString::from(&song.lyrics))
                }
                SongRoles::Author => {
                    QVariant::from(&QString::from(&song.author))
                }
                SongRoles::Ccli => {
                    QVariant::from(&QString::from(&song.ccli))
                }
                SongRoles::Audio => {
                    QVariant::from(&QString::from(&song.audio))
                }
                SongRoles::VerseOrder => {
                    QVariant::from(&QString::from(&song.verse_order))
                }
                SongRoles::Background => {
                    QVariant::from(&QString::from(&song.background))
                }
                SongRoles::BackgroundType => QVariant::from(
                    &QString::from(&song.background_type),
                ),
                SongRoles::HorizontalTextAlignment => QVariant::from(
                    &QString::from(&song.horizontal_text_alignment),
                ),
                SongRoles::VerticalTextAlignment => QVariant::from(
                    &QString::from(&song.vertical_text_alignment),
                ),
                SongRoles::Font => {
                    QVariant::from(&QString::from(&song.font))
                }
                SongRoles::FontSize => {
                    QVariant::from(&song.font_size)
                }
                _ => QVariant::default(),
            };
        }

        QVariant::default()
    }

    // Example of overriding a C++ virtual method and calling the base class implementation.

    // pub fn can_fetch_more(&self, parent: &QModelIndex) -> bool {
    //     self.base_can_fetch_more(parent)
    // }

    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(SongRoles::Id.repr, QByteArray::from("id"));
        roles
            .insert(SongRoles::Title.repr, QByteArray::from("title"));
        roles.insert(
            SongRoles::Lyrics.repr,
            QByteArray::from("lyrics"),
        );
        roles.insert(
            SongRoles::Author.repr,
            QByteArray::from("author"),
        );
        roles.insert(SongRoles::Ccli.repr, QByteArray::from("ccli"));
        roles
            .insert(SongRoles::Audio.repr, QByteArray::from("audio"));
        roles.insert(
            SongRoles::VerseOrder.repr,
            QByteArray::from("vorder"),
        );
        roles.insert(
            SongRoles::Background.repr,
            QByteArray::from("background"),
        );
        roles.insert(
            SongRoles::BackgroundType.repr,
            QByteArray::from("backgroundType"),
        );
        roles.insert(
            SongRoles::HorizontalTextAlignment.repr,
            QByteArray::from("horizontalTextAlignment"),
        );
        roles.insert(
            SongRoles::VerticalTextAlignment.repr,
            QByteArray::from("verticalTextAlignment"),
        );
        roles.insert(SongRoles::Font.repr, QByteArray::from("font"));
        roles.insert(
            SongRoles::FontSize.repr,
            QByteArray::from("fontSize"),
        );
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        let cnt = self.rust().songs.len() as i32;
        // println!("row count is {cnt}");
        cnt
    }
}
