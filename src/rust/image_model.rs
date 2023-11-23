#[cxx_qt::bridge]
mod image_model {
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

    #[qenum(ImageModel)]
    enum Role {
        Id,
        Path,
        Title,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QAbstractListModel"]
        #[qml_element]
        #[qproperty(i32, count_rows)]
        type ImageModel = super::ImageModelRust;

        #[inherit]
        #[qsignal]
        fn data_changed(
            self: Pin<&mut ImageModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[qinvokable]
        fn clear(self: Pin<&mut ImageModel>);
        #[qinvokable]
        fn setup(self: Pin<&mut ImageModel>);
        #[qinvokable]
        fn remove_item(
            self: Pin<&mut ImageModel>,
            index: i32,
        ) -> bool;
        #[qinvokable]
        fn new_item(self: Pin<&mut ImageModel>, url: QUrl);
        #[qinvokable]
        fn update_title(
            self: Pin<&mut ImageModel>,
            index: i32,
            updated_title: QString,
        ) -> bool;
        #[qinvokable]
        fn update_file_path(
            self: Pin<&mut ImageModel>,
            index: i32,
            updated_file_path: QString,
        ) -> bool;
        #[qinvokable]
        fn get_item(
            self: Pin<&mut ImageModel>,
            index: i32,
        ) -> QMap_QString_QVariant;
    }

    impl cxx_qt::Threading for ImageModel {}

    unsafe extern "RustQt" {
        #[inherit]
        unsafe fn begin_insert_rows(
            self: Pin<&mut ImageModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_insert_rows(self: Pin<&mut ImageModel>);

        #[inherit]
        unsafe fn begin_remove_rows(
            self: Pin<&mut ImageModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn begin_move_rows(
            self: Pin<&mut ImageModel>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;

        #[inherit]
        unsafe fn end_move_rows(self: Pin<&mut ImageModel>);

        #[inherit]
        unsafe fn end_remove_rows(self: Pin<&mut ImageModel>);

        #[inherit]
        unsafe fn begin_reset_model(self: Pin<&mut ImageModel>);

        #[inherit]
        unsafe fn end_reset_model(self: Pin<&mut ImageModel>);

        #[inherit]
        fn can_fetch_more(
            self: &ImageModel,
            parent: &QModelIndex,
        ) -> bool;

        #[inherit]
        fn index(
            self: &ImageModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        fn data(
            self: &ImageModel,
            index: &QModelIndex,
            role: i32,
        ) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        fn role_names(self: &ImageModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn row_count(self: &ImageModel, _parent: &QModelIndex)
            -> i32;

        #[qinvokable]
        fn count(self: &ImageModel) -> i32;
    }
}

use crate::image_model::image_model::Image;
use crate::schema::images::dsl::*;
use diesel::sqlite::SqliteConnection;
use diesel::{delete, insert_into, prelude::*, update};
use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct Image {
    id: i32,
    title: QString,
    path: QString,
}

#[derive(Default, Debug)]
pub struct ImageModelRust {
    count_rows: i32,
    highest_id: i32,
    images: Vec<self::Image>,
}

impl qobject::ImageModel {
    pub fn clear(mut self: Pin<&mut Self>) {
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().images_mut().clear();
            self.as_mut().end_reset_model();
        }
    }

    pub fn setup(mut self: Pin<&mut Self>) {
        let db = &mut self.as_mut().get_db();
        let results = images
            .load::<crate::models::Image>(db)
            .expect("Error loading images");
        self.as_mut().set_highest_id(0);

        println!("SHOWING IMAGES");
        println!("--------------");
        for image in results {
            println!("{}", image.title);
            println!("{}", image.id);
            println!("{}", image.path);
            println!("--------------");
            if self.as_mut().highest_id() < &image.id {
                self.as_mut().set_highest_id(image.id);
            }

            let img = self::Image {
                id: image.id,
                title: QString::from(&image.title),
                path: QString::from(&image.path),
            };

            self.as_mut().add_image(img);
        }
        println!("--------------------------------------");
        println!("{:?}", self.as_mut().images());
        println!("--------------------------------------");
    }

    pub fn remove_item(mut self: Pin<&mut Self>, index: i32) -> bool {
        if index < 0 || (index as usize) >= self.images().len() {
            return false;
        }
        let db = &mut self.as_mut().get_db();

        let image_id = self.images().get(index as usize).unwrap().id;

        let result =
            delete(images.filter(id.eq(image_id))).execute(db);

        match result {
            Ok(_i) => {
                unsafe {
                    self.as_mut().begin_remove_rows(
                        &QModelIndex::default(),
                        index,
                        index,
                    );
                    self.as_mut().images_mut().remove(index as usize);
                    self.as_mut().end_remove_rows();
                }
                println!("removed-item-at-index: {:?}", image_id);
                println!("new-Vec: {:?}", self.as_mut().images());
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
        let image_id = self.rust().highest_id + 1;
        let image_title = QString::from(name);
        let image_path = url.to_qstring();

        if self.as_mut().add_item(image_id, image_title, image_path) {
            println!("filename: {:?}", name);
            self.as_mut().set_highest_id(image_id);
        } else {
            println!("Error in inserting item");
        }
    }

    fn add_item(
        mut self: Pin<&mut Self>,
        image_id: i32,
        image_title: QString,
        image_path: QString,
    ) -> bool {
        let db = &mut self.as_mut().get_db();
        // println!("{:?}", db);
        let image = self::Image {
            id: image_id,
            title: image_title.clone(),
            path: image_path.clone(),
        };
        println!("{:?}", image);

        let result = insert_into(images)
            .values((
                id.eq(&image_id),
                title.eq(&image_title.to_string()),
                path.eq(&image_path.to_string()),
            ))
            .execute(db);
        println!("{:?}", result);

        match result {
            Ok(_i) => {
                self.as_mut().add_image(image);
                println!("{:?}", self.as_mut().images());
                true
            }
            Err(_e) => {
                println!("Cannot connect to database");
                false
            }
        }
    }

    fn add_image(mut self: Pin<&mut Self>, image: self::Image) {
        let index = self.as_ref().images().len() as i32;
        println!("{:?}", image);
        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut().images_mut().push(image);
            self.as_mut().end_insert_rows();
        }
    }

    pub fn update_title(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_title: QString,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.as_ref().get_role_id(Role::TitleRole));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(images.filter(id.eq(index)))
            .set(title.eq(updated_title.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                for image in self
                    .as_mut()
                    .images_mut()
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    image.title = updated_title.clone();
                    println!("rust-title: {:?}", image.title);
                }
                self.as_mut().emit_data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                true
            }
            Err(_e) => false,
        }
    }

    pub fn update_file_path(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_file_path: QString,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.as_ref().get_role_id(Role::PathRole));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(images.filter(id.eq(index)))
            .set(path.eq(updated_file_path.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                for image in self
                    .as_mut()
                    .images_mut()
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    image.path = updated_file_path.clone();
                    println!("rust-title: {:?}", image.path);
                }
                self.as_mut().emit_data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                true
            }
            Err(_e) => false,
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
        if let Some(image) = self.rust().images.get(index as usize) {
            for i in role_names_iter {
                qvariantmap.insert(
                    QString::from(&i.1.to_string()),
                    self.as_ref().data(&idx, *i.0),
                );
            }
        };
        qvariantmap
    }

    fn get_role_id(&self, role: Role) -> i32 {
        match role {
            qobject::Role::Id => 0,
            qobject::Role::Title => 1,
            qobject::Role::Path => 2,
            _ => 0,
        }
    }
}

// QAbstractListModel implementation
impl qobject::ImageModel {
    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = qobject::Roles { repr: role };
        if let Some(image) = self.images().get(index.row() as usize) {
            return match role {
                qobject::Roles::Id => QVariant::from(&image.id),
                qobject::Roles::Title => QVariant::from(&image.title),
                qobject::Roles::Path => QVariant::from(&image.path),
                _ => QVariant::default(),
            };
        }

        QVariant::default()
    }

    // Example of overriding a C++ virtual method and calling the base class implementation.

    pub fn can_fetch_more(&self, parent: &QModelIndex) -> bool {
        self.base_can_fetch_more(parent)
    }

    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(
            qobject::Roles::Id.repr,
            cxx_qt_lib::QByteArray::from("id"),
        );
        roles.insert(
            qobject::Roles::Title.repr,
            cxx_qt_lib::QByteArray::from("title"),
        );
        roles.insert(
            qobject::Roles::Path.repr,
            cxx_qt_lib::QByteArray::from("filePath"),
        );
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        let cnt = self.rust().images.len() as i32;
        // self.as_mut().set_count(cnt);
        // println!("row count is {cnt}");
        cnt
    }

    pub fn count(mut self: Pin<&mut Self>) -> i32 {
        let cnt = self.rust().images.len() as i32;
        self.as_mut().set_count_rows(cnt);
        cnt
    }
}
