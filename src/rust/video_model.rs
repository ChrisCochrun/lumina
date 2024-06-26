#[cxx_qt::bridge]
mod qobject {
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
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
    }

    #[qenum(VideoModel)]
    enum VideoRoles {
        Id,
        Title,
        Path,
        StartTime,
        EndTime,
        Looping,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QAbstractListModel"]
        #[qml_element]
        #[qproperty(i32, count)]
        type VideoModel = super::VideoModelRust;

        #[inherit]
        #[qsignal]
        fn data_changed(
            self: Pin<&mut VideoModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[qinvokable]
        fn clear(self: Pin<&mut VideoModel>);
        #[qinvokable]
        fn setup(self: Pin<&mut VideoModel>);
        #[qinvokable]
        fn remove_item(
            self: Pin<&mut VideoModel>,
            index: i32,
        ) -> bool;
        #[qinvokable]
        fn new_item(self: Pin<&mut VideoModel>, url: QUrl);
        #[qinvokable]
        fn update_path(
            self: Pin<&mut VideoModel>,
            index: i32,
            updated_path: QString,
        ) -> bool;
        #[qinvokable]
        fn get_item(
            self: Pin<&mut VideoModel>,
            index: i32,
        ) -> QMap_QString_QVariant;
        #[qinvokable]
        fn update_loop(
            self: Pin<&mut VideoModel>,
            index: i32,
            loop_value: bool,
        ) -> bool;
        #[qinvokable]
        fn update_title(
            self: Pin<&mut VideoModel>,
            index: i32,
            updated_title: QString,
        ) -> bool;
        #[qinvokable]
        fn update_start_time(
            self: Pin<&mut VideoModel>,
            index: i32,
            updated_start_time: f32,
        ) -> bool;
        #[qinvokable]
        fn update_end_time(
            self: Pin<&mut VideoModel>,
            index: i32,
            updated_end_time: f32,
        ) -> bool;
    }

    impl cxx_qt::Threading for VideoModel {}

    unsafe extern "RustQt" {
        #[inherit]
        unsafe fn begin_insert_rows(
            self: Pin<&mut VideoModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_insert_rows(self: Pin<&mut VideoModel>);

        #[inherit]
        unsafe fn begin_remove_rows(
            self: Pin<&mut VideoModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn begin_move_rows(
            self: Pin<&mut VideoModel>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;

        #[inherit]
        unsafe fn end_move_rows(self: Pin<&mut VideoModel>);

        #[inherit]
        unsafe fn end_remove_rows(self: Pin<&mut VideoModel>);

        #[inherit]
        unsafe fn begin_reset_model(self: Pin<&mut VideoModel>);

        #[inherit]
        unsafe fn end_reset_model(self: Pin<&mut VideoModel>);

        #[inherit]
        fn can_fetch_more(
            self: &VideoModel,
            parent: &QModelIndex,
        ) -> bool;

        #[inherit]
        fn index(
            self: &VideoModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        fn data(
            self: &VideoModel,
            index: &QModelIndex,
            role: i32,
        ) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        fn role_names(self: &VideoModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn row_count(self: &VideoModel, _parent: &QModelIndex)
            -> i32;

    }
}


use crate::schema::videos::dsl::*;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QModelIndex, QString, QUrl, QVariant};
use diesel::sqlite::SqliteConnection;
use diesel::{delete, insert_into, prelude::*, update};
use std::path::{PathBuf};
use std::pin::Pin;

use self::qobject::{
    QHash_i32_QByteArray, QMap_QString_QVariant, QVector_i32,
    VideoRoles,
};

#[derive(Default, Clone, Debug)]
pub struct Video {
    id: i32,
    title: String,
    path: String,
    start_time: f32,
    end_time: f32,
    looping: bool,
}

#[derive(Default, Debug)]
pub struct VideoModelRust {
    count: i32,
    highest_id: i32,
    videos: Vec<self::Video>,
}

impl qobject::VideoModel {
    pub fn clear(mut self: Pin<&mut Self>) {
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().videos.clear();
            self.as_mut().end_reset_model();
        }
    }

    pub fn setup(mut self: Pin<&mut Self>) {
        let db = &mut self.as_mut().get_db();
        let results = videos
            .load::<crate::models::Video>(db)
            .expect("Error loading videos");
        self.as_mut().rust_mut().highest_id = 0;

        println!("SHOWING VIDEOS");
        println!("--------------");
        for video in results {
            println!("{}", video.title);
            println!("{}", video.id);
            println!("{}", video.path);
            println!("--------------");
            if self.as_mut().highest_id < video.id {
                self.as_mut().rust_mut().highest_id = video.id;
            }

            let img = self::Video {
                id: video.id,
                title: video.title,
                path: video.path,
                start_time: video.start_time.unwrap_or(0.0),
                end_time: video.end_time.unwrap_or(0.0),
                looping: video.looping,
            };

            self.as_mut().add_video(img);
        }
        println!("--------------------------------------");
        println!("{:?}", self.as_mut().videos);
        println!("--------------------------------------");
    }

    pub fn remove_item(mut self: Pin<&mut Self>, index: i32) -> bool {
        if index < 0 || (index as usize) >= self.videos.len() {
            return false;
        }
        let db = &mut self.as_mut().get_db();

        let video_id = self.videos.get(index as usize).unwrap().id;

        let result =
            delete(videos.filter(id.eq(video_id))).execute(db);

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
                        .videos
                        .remove(index as usize);
                    self.as_mut().end_remove_rows();
                }
                println!("removed-item-at-index: {:?}", video_id);
                println!("new-Vec: {:?}", self.as_mut().videos);
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

    pub fn new_item(mut self: Pin<&mut Self>, url: QUrl) {
        println!("LETS INSERT THIS SUCKER!");
        let file_path = PathBuf::from(url.path().to_string());
        let name = file_path.file_stem().unwrap().to_str().unwrap();
        let video_id = self.rust().highest_id + 1;
        let video_title = QString::from(name);
        let video_path = url.to_qstring();

        if self.as_mut().add_item(video_id, video_title, video_path) {
            println!("filename: {:?}", name);
            self.as_mut().rust_mut().highest_id = video_id;
        } else {
            println!("Error in inserting item");
        }
    }

    pub fn add_item(
        mut self: Pin<&mut Self>,
        video_id: i32,
        video_title: QString,
        video_path: QString,
    ) -> bool {
        let db = &mut self.as_mut().get_db();
        // println!("{:?}", db);
        let video = self::Video {
            id: video_id,
            title: video_title.clone().to_string(),
            path: video_path.clone().to_string(),
            start_time: 0.0,
            end_time: 0.0,
            looping: false,
        };
        println!("{:?}", video);

        let result = insert_into(videos)
            .values((
                id.eq(&video_id),
                title.eq(&video_title.to_string()),
                path.eq(&video_path.to_string()),
                start_time.eq(&video.start_time),
                end_time.eq(&video.end_time),
                looping.eq(&video.looping),
            ))
            .execute(db);
        println!("{:?}", result);

        match result {
            Ok(_i) => {
                self.as_mut().add_video(video);
                println!("{:?}", self.as_mut().videos);
                true
            }
            Err(_e) => {
                println!(
                    "Cannot connect to database or there was an error in inserting the video"
                );
                false
            }
        }
    }

    fn add_video(mut self: Pin<&mut Self>, video: self::Video) {
        let index = self.as_ref().videos.len() as i32;
        println!("{:?}", video);
        let count = self.as_mut().count;
        self.as_mut().set_count(count + 1);
        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut().rust_mut().videos.push(video);
            self.as_mut().end_insert_rows();
        }
    }

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
        if let Some(video) = self.rust().videos.get(index as usize) {
            for i in role_names_iter {
                qvariantmap.insert(
                    QString::from(&i.1.to_string()),
                    self.as_ref().data(&idx, *i.0),
                );
            }
            println!("gotted-video: {:?}", video);
        };
        qvariantmap
    }

    fn get_role(&self, role: VideoRoles) -> i32 {
        match role {
            VideoRoles::Id => 0,
            VideoRoles::Title => 1,
            VideoRoles::Path => 2,
            VideoRoles::StartTime => 3,
            VideoRoles::EndTime => 4,
            VideoRoles::Looping => 5,
            _ => 0,
        }
    }

    pub fn update_loop(
        mut self: Pin<&mut Self>,
        index: i32,
        loop_value: bool,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.as_ref().get_role(VideoRoles::Looping));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());
        println!("rust-video: {:?}", index);
        println!("rust-loop: {:?}", loop_value);

        let db = &mut self.as_mut().get_db();
        let result = update(videos.filter(id.eq(index)))
            .set(looping.eq(loop_value))
            .execute(db);
        match result {
            Ok(_i) => {
                for video in self
                    .as_mut()
                    .rust_mut()
                    .videos
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    video.looping = loop_value;
                    println!("rust-video: {:?}", video.title);
                }
                self.as_mut().data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                println!("rust-looping: {:?}", loop_value);
                true
            }
            Err(_e) => false,
        }
    }

    pub fn update_end_time(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_end_time: f32,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.as_ref().get_role(VideoRoles::EndTime));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(videos.filter(id.eq(index)))
            .set(end_time.eq(updated_end_time))
            .execute(db);
        match result {
            Ok(_i) => {
                for video in self
                    .as_mut()
                    .rust_mut()
                    .videos
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    video.end_time = updated_end_time;
                }
                self.as_mut().data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                println!("rust-end-time: {:?}", updated_end_time);
                true
            }
            Err(_e) => false,
        }
    }

    pub fn update_start_time(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_start_time: f32,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.as_ref().get_role(VideoRoles::StartTime));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(videos.filter(id.eq(index)))
            .set(start_time.eq(updated_start_time))
            .execute(db);
        match result {
            Ok(_i) => {
                for video in self
                    .as_mut()
                    .rust_mut()
                    .videos
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    video.start_time = updated_start_time;
                }
                self.as_mut().data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                println!("rust-start-time: {:?}", updated_start_time);
                true
            }
            Err(_e) => false,
        }
    }

    pub fn update_title(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_title: QString,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.as_ref().get_role(VideoRoles::Title));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(videos.filter(id.eq(index)))
            .set(title.eq(updated_title.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                for video in self
                    .as_mut()
                    .rust_mut()
                    .videos
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    video.title = updated_title.clone().to_string();
                    println!("rust-title: {:?}", video.title);
                }
                // TODO this seems to not be updating in the actual list
                self.as_mut().data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                println!("rust-title: {:?}", updated_title);
                true
            }
            Err(_e) => false,
        }
    }

    pub fn update_path(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_path: QString,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.as_ref().get_role(VideoRoles::Path));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(videos.filter(id.eq(index)))
            .set(path.eq(updated_path.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                for video in self
                    .as_mut()
                    .rust_mut()
                    .videos
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    video.path = updated_path.clone().to_string();
                    println!("rust-title: {:?}", video.title);
                }
                self.as_mut().data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                println!("rust-path: {:?}", updated_path);
                true
            }
            Err(_e) => false,
        }
    }
}

// QAbstractListModel implementation
impl qobject::VideoModel {
    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = VideoRoles { repr: role };
        if let Some(video) = self.videos.get(index.row() as usize) {
            return match role {
                VideoRoles::Id => QVariant::from(&video.id),
                VideoRoles::Title => {
                    QVariant::from(&QString::from(&video.title))
                }
                VideoRoles::Path => {
                    QVariant::from(&QString::from(&video.path))
                }
                VideoRoles::StartTime => {
                    QVariant::from(&video.start_time)
                }
                VideoRoles::EndTime => {
                    QVariant::from(&video.end_time)
                }
                VideoRoles::Looping => QVariant::from(&video.looping),
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
        roles.insert(VideoRoles::Id.repr, QByteArray::from("id"));
        roles.insert(
            VideoRoles::Title.repr,
            QByteArray::from("title"),
        );
        roles.insert(
            VideoRoles::Path.repr,
            QByteArray::from("filePath"),
        );
        roles.insert(
            VideoRoles::StartTime.repr,
            QByteArray::from("startTime"),
        );
        roles.insert(
            VideoRoles::EndTime.repr,
            QByteArray::from("endTime"),
        );
        roles.insert(
            VideoRoles::Looping.repr,
            QByteArray::from("loop"),
        );
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        let cnt = self.rust().videos.len() as i32;
        // println!("row count is {cnt}");
        cnt
    }
}
