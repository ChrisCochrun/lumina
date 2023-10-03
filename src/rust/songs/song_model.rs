#[cxx_qt::bridge]
pub mod song_model {
    // use crate::models::*;
    // use crate::schema::songs::dsl::*;
    use crate::songs::song::Song;
    // use diesel::sqlite::SqliteConnection;
    use sqlx::{SqliteConnection, Connection, Executor, query, query_as, Error};
    // use diesel::{delete, insert_into, prelude::*, update};
    use tokio::runtime::Runtime;
    use std::collections::HashMap;
    use tracing::{debug, debug_span, error, info, instrument};

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

    #[cxx_qt::qobject(base = "QAbstractListModel")]
    #[derive(Default, Debug)]
    pub struct SongModel {
        highest_id: i32,
        songs: Vec<Song>,
    }

    #[cxx_qt::qsignals(SongModel)]
    pub enum Signals<'a> {
        #[inherit]
        DataChanged {
            top_left: &'a QModelIndex,
            bottom_right: &'a QModelIndex,
            roles: &'a QVector_i32,
        },
        TitleChanged {},
        FontSizeChanged {},
        BackgroundChanged {},
    }

    enum Role {
        IdRole,
        TitleRole,
        LyricsRole,
        AuthorRole,
        CcliRole,
        AudioRole,
        VerseOrderRole,
        BackgroundRole,
        BackgroundTypeRole,
        HorizontalTextAlignmentRole,
        VerticalTextAlignmentRole,
        FontRole,
        FontSizeRole,
    }

    impl qobject::SongModel {
        #[qinvokable]
        pub fn clear(mut self: Pin<&mut Self>) {
            unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().songs_mut().clear();
                self.as_mut().end_reset_model();
            }
        }

        #[qinvokable]
        pub async fn setup(mut self: Pin<&mut Self>) {
            // run_migrations(db);
            self.as_mut().set_highest_id(0);
            let thread = self.qt_thread();
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let db = &mut self.as_mut().get_db().await.unwrap();
            let handle = tokio::spawn(async move {
                let results: Vec<Song> = query_as("SELECT * FROM songs").fetch_all(db).await.unwrap();
                for song in results {
                    println!("{}", song.title);
                    println!("{}", song.id);
                    println!(
                        "{}",
                        song.background.clone().unwrap_or_default()
                    );
                    println!("--------------");
                    if self.as_mut().highest_id() < &song.id {
                        self.as_mut().set_highest_id(song.id);
                    }

                    let song = Song {
                        id: song.id,
                        title: song.title,
                        lyrics: song.lyrics,
                        author: song.author,
                        ccli: song.ccli,
                        audio: song.audio,
                        verse_order: song.verse_order,
                        background: song.background,
                        background_type: song
                            .background_type,
                        horizontal_text_alignment: song
                            .horizontal_text_alignment,
                        vertical_text_alignment: song
                            .vertical_text_alignment,
                        font: song.font,
                        font_size: song.font_size,
                    };

                    // thread.queue(move |mut song_model|
                    //              song_model.as_mut().add_song(song);
                    // )
                };
            });
            // let results = songs
            //     .load::<crate::models::Song>(db)
            //     .expect("NO TABLE?????????????");

            println!("SHOWING SONGS");
            println!("--------------");
            println!("--------------------------------------");
            println!("{:?}", self.as_mut().songs());
            println!("--------------------------------------");
        }

        #[qinvokable]
        pub fn remove_item(
            mut self: Pin<&mut Self>,
            index: i32,
        ) -> bool {
            if index < 0 || (index as usize) >= self.songs().len() {
                return false;
            }
            let db = &mut self.as_mut().get_db();

            let song_id =
                self.songs().get(index as usize).unwrap().id;

            let handle = tokio::spawn(async move {
                query!("DELETE {} FROM songs", song_id)
            });

            // let result =
            //     delete(songs.filter(id.eq(song_id))).execute(db);

            match result {
                Ok(_i) => {
                    unsafe {
                        self.as_mut().begin_remove_rows(
                            &QModelIndex::default(),
                            index,
                            index,
                        );
                        self.as_mut()
                            .songs_mut()
                            .remove(index as usize);
                        self.as_mut().end_remove_rows();
                    }
                    println!("removed-item-at-index: {:?}", song_id);
                    println!("new-Vec: {:?}", self.as_mut().songs());
                    true
                }
                Err(_e) => {
                    println!("Cannot connect to database");
                    false
                }
            }
        }

        async fn get_db(self: Pin<&mut Self>) -> Result<SqliteConnection, Error> {
            let mut data = dirs::data_local_dir().unwrap();
            data.push("lumina");
            data.push("library-db.sqlite3");
            let mut db_url = String::from("sqlite://");
            db_url.push_str(data.to_str().unwrap());
            println!("DB: {:?}", db_url);

            SqliteConnection::connect(&db_url).await?
        }

        #[qinvokable]
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
                    println!("{:?}", self.as_mut().songs());
                    true
                }
                Err(_e) => {
                    println!("Cannot connect to database");
                    false
                }
            }
        }

        fn add_song(mut self: Pin<&mut Self>, song: Song) {
            let index = self.as_ref().songs().len() as i32;
            println!("{:?}", song);
            unsafe {
                self.as_mut().begin_insert_rows(
                    &QModelIndex::default(),
                    index,
                    index,
                );
                self.as_mut().songs_mut().push(song);
                self.as_mut().end_insert_rows();
            }
        }

        #[qinvokable]
        pub fn update_title(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_title: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::TitleRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(title.eq(updated_title.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.title = updated_title.to_string();
                        self.as_mut().emit(Signals::DataChanged {
                            top_left: &model_index,
                            bottom_right: &model_index,
                            roles: &vector_roles,
                        });
                        self.as_mut().emit_title_changed();
                        true
                    } else {
                        false
                    }
                }
                Err(_e) => false,
            }
        }

        #[qinvokable]
        pub fn update_lyrics(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_lyrics: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::LyricsRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(lyrics.eq(updated_lyrics.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.lyrics = updated_lyrics.to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_author(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_author: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::AuthorRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(author.eq(updated_author.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.author = updated_author.to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_audio(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_audio: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::AudioRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(audio.eq(updated_audio.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.audio = updated_audio.to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_ccli(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_ccli: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::CcliRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(ccli.eq(updated_ccli.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.ccli = updated_ccli.to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_verse_order(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_verse_order: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::VerseOrderRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(verse_order.eq(updated_verse_order.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.verse_order =
                            updated_verse_order.to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_background(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_background: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::BackgroundRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(background.eq(updated_background.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.background =
                            updated_background.to_string();
                        debug!(
                            background = ?updated_background,
                            model_index = ?model_index,
                            roles = vector_roles.get(0)
                        );
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
                            &vector_roles,
                        );
                        self.as_mut().emit_background_changed();
                        true
                    } else {
                        false
                    }
                }
                Err(_e) => false,
            }
        }

        #[qinvokable]
        pub fn update_background_type(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_background_type: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(
                self.as_ref().get_role(Role::BackgroundTypeRole),
            );
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(
                    background_type
                        .eq(updated_background_type.to_string()),
                )
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.background_type =
                            updated_background_type.to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_horizontal_text_alignment(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_horizontal_text_alignment: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(
                self.as_ref()
                    .get_role(Role::HorizontalTextAlignmentRole),
            );
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(horizontal_text_alignment.eq(
                    updated_horizontal_text_alignment.to_string(),
                ))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.horizontal_text_alignment =
                            updated_horizontal_text_alignment
                                .to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_vertical_text_alignment(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_vertical_text_alignment: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(
                self.as_ref()
                    .get_role(Role::VerticalTextAlignmentRole),
            );
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result =
                update(songs.filter(id.eq(index)))
                    .set(vertical_text_alignment.eq(
                        updated_vertical_text_alignment.to_string(),
                    ))
                    .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.vertical_text_alignment =
                            updated_vertical_text_alignment
                                .to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_font(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_font: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::FontRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(font.eq(updated_font.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.font = updated_font.to_string();
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn update_font_size(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_font_size: i32,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::FontSizeRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(songs.filter(id.eq(index)))
                .set(font_size.eq(updated_font_size))
                .execute(db);
            match result {
                Ok(_i) => {
                    if let Some(song) = self
                        .as_mut()
                        .songs_mut()
                        .get_mut(index as usize)
                    {
                        song.font_size = updated_font_size;
                        self.as_mut().emit_data_changed(
                            model_index,
                            model_index,
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

        #[qinvokable]
        pub fn get_item(
            self: Pin<&mut Self>,
            index: i32,
        ) -> QMap_QString_QVariant {
            println!("{index}");
            let mut qvariantmap = QMap_QString_QVariant::default();
            let idx = self.index(index, 0, &QModelIndex::default());
            if !idx.is_valid() {
                return qvariantmap;
            }
            let role_names = self.as_ref().role_names();
            let role_names_iter = role_names.iter();
            if let Some(song) = self.rust().songs.get(index as usize)
            {
                for i in role_names_iter {
                    qvariantmap.insert(
                        QString::from(&i.1.to_string()),
                        self.as_ref().data(&idx, *i.0),
                    );
                }
            };
            qvariantmap
        }

        #[qinvokable]
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
            if let Some(song) = self.rust().songs.get(index as usize)
            {
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
                            println!("{verse_title}");
                            println!("{lyric}");
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
                println!("da-map: {:?}", lyric_map);

                for mut verse in vorder {
                    let mut verse_name = "";
                    debug!(verse = verse);
                    for word in keywords.clone() {
                        let end_verse =
                            verse.get(1..2).unwrap_or_default();
                        let beg_verse =
                            verse.get(0..1).unwrap_or_default();
                        println!(
                            "verse: {:?}, beginning: {:?}, end: {:?}, word: {:?}",
                            verse, beg_verse, end_verse, word
                        );
                        if word.starts_with(beg_verse)
                            && word.ends_with(end_verse)
                        {
                            verse_name = word;
                            println!("TITLE: {verse_name}");
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
                                lyric_list
                                    .append(QString::from(lyric));
                            }
                            continue;
                        }
                        lyric_list.append(QString::from(lyric));
                    } else {
                        println!("NOT WORKING!");
                    };
                }
                for lyric in lyric_list.iter() {
                    println!("da-list: {:?}", lyric);
                }
            }
            QStringList::from(&lyric_list)
        }

        fn get_role(&self, role: Role) -> i32 {
            match role {
                Role::IdRole => 0,
                Role::TitleRole => 1,
                Role::LyricsRole => 2,
                Role::AuthorRole => 3,
                Role::CcliRole => 4,
                Role::AudioRole => 5,
                Role::VerseOrderRole => 6,
                Role::BackgroundRole => 7,
                Role::BackgroundTypeRole => 8,
                Role::HorizontalTextAlignmentRole => 9,
                Role::VerticalTextAlignmentRole => 10,
                Role::FontRole => 11,
                Role::FontSizeRole => 12,
                _ => 0,
            }
        }
    }

    // Create Rust bindings for C++ functions of the base class (QAbstractItemModel)
    #[cxx_qt::inherit]
    extern "C++" {
        unsafe fn begin_insert_rows(
            self: Pin<&mut qobject::SongModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_insert_rows(self: Pin<&mut qobject::SongModel>);

        unsafe fn begin_remove_rows(
            self: Pin<&mut qobject::SongModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_remove_rows(self: Pin<&mut qobject::SongModel>);

        unsafe fn begin_reset_model(
            self: Pin<&mut qobject::SongModel>,
        );
        unsafe fn end_reset_model(self: Pin<&mut qobject::SongModel>);
    }

    #[cxx_qt::inherit]
    unsafe extern "C++" {
        #[cxx_name = "canFetchMore"]
        fn base_can_fetch_more(
            self: &qobject::SongModel,
            parent: &QModelIndex,
        ) -> bool;

        fn index(
            self: &qobject::SongModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    // QAbstractListModel implementation
    impl qobject::SongModel {
        #[qinvokable(cxx_override)]
        fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
            if let Some(song) = self.songs().get(index.row() as usize)
            {
                return match role {
                    0 => QVariant::from(&song.id),
                    1 => QVariant::from(&QString::from(&song.title)),
                    2 => QVariant::from(&QString::from(&song.lyrics)),
                    3 => QVariant::from(&QString::from(&song.author)),
                    4 => QVariant::from(&QString::from(&song.ccli)),
                    5 => QVariant::from(&QString::from(&song.audio)),
                    6 => QVariant::from(&QString::from(
                        &song.verse_order,
                    )),
                    7 => QVariant::from(&QString::from(
                        &song.background,
                    )),
                    8 => QVariant::from(&QString::from(
                        &song.background_type,
                    )),
                    9 => QVariant::from(&QString::from(
                        &song.horizontal_text_alignment,
                    )),
                    10 => QVariant::from(&QString::from(
                        &song.vertical_text_alignment,
                    )),
                    11 => QVariant::from(&QString::from(&song.font)),
                    12 => QVariant::from(&song.font_size),
                    _ => QVariant::default(),
                };
            }

            QVariant::default()
        }

        // Example of overriding a C++ virtual method and calling the base class implementation.
        #[qinvokable(cxx_override)]
        pub fn can_fetch_more(&self, parent: &QModelIndex) -> bool {
            self.base_can_fetch_more(parent)
        }

        #[qinvokable(cxx_override)]
        pub fn role_names(&self) -> QHash_i32_QByteArray {
            let mut roles = QHash_i32_QByteArray::default();
            roles.insert(0, cxx_qt_lib::QByteArray::from("id"));
            roles.insert(1, cxx_qt_lib::QByteArray::from("title"));
            roles.insert(2, cxx_qt_lib::QByteArray::from("lyrics"));
            roles.insert(3, cxx_qt_lib::QByteArray::from("author"));
            roles.insert(4, cxx_qt_lib::QByteArray::from("ccli"));
            roles.insert(5, cxx_qt_lib::QByteArray::from("audio"));
            roles.insert(6, cxx_qt_lib::QByteArray::from("vorder"));
            roles.insert(
                7,
                cxx_qt_lib::QByteArray::from("background"),
            );
            roles.insert(
                8,
                cxx_qt_lib::QByteArray::from("backgroundType"),
            );
            roles.insert(
                9,
                cxx_qt_lib::QByteArray::from(
                    "horizontalTextAlignment",
                ),
            );
            roles.insert(
                10,
                cxx_qt_lib::QByteArray::from("verticalTextAlignment"),
            );
            roles.insert(11, cxx_qt_lib::QByteArray::from("font"));
            roles
                .insert(12, cxx_qt_lib::QByteArray::from("fontSize"));
            roles
        }

        #[qinvokable(cxx_override)]
        pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
            let cnt = self.rust().songs.len() as i32;
            // println!("row count is {cnt}");
            cnt
        }

        #[qinvokable]
        pub fn count(&self) -> i32 {
            self.rust().songs.len() as i32
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cxx_qt_lib::QStringList; // Replace 'some_module' with the actual module where QModelIndex is defined.

//     #[test]
//     fn test_get_lyric_list() {
//         // Create a test instance of your struct (you might need to adjust this based on your actual struct).
//         let mut song_model = SongModel {
//             highest_id: 0,
//             songs: Vec::<song_model::Song>::new(),
//         };

//         // this sets up the songmodel with the database
//         // song_model.setup_wrapper(self);

//         // Call the get_lyric_list function with specific inputs.
//         let index = 0; // Replace with your desired test index.

//         let result = song_model.get_lyric_list(index);

//         // Define your expected result here. For simplicity, let's assume an empty QStringList.
//         let expected_result = QStringList::default();

//         // Compare the actual result with the expected result using an assertion.
//         assert_eq!(result, expected_result);
//     }
// }
