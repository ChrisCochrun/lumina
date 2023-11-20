#[cxx_qt::bridge]
mod image_model {
    use crate::image_model::image_model::Image;
    use crate::schema::images::dsl::*;
    use diesel::sqlite::SqliteConnection;
    use diesel::{delete, insert_into, prelude::*, update};
    use std::path::PathBuf;

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

    #[derive(Default, Clone, Debug)]
    pub struct Image {
        id: i32,
        title: QString,
        path: QString,
    }

    #[cxx_qt::qobject(base = "QAbstractListModel")]
    #[derive(Default, Debug)]
    pub struct ImageModel {
        #[qproperty]
        count_rows: i32,
        highest_id: i32,
        images: Vec<self::Image>,
    }

    #[cxx_qt::qsignals(ImageModel)]
    pub enum Signals<'a> {
        #[inherit]
        DataChanged {
            top_left: &'a QModelIndex,
            bottom_right: &'a QModelIndex,
            roles: &'a QVector_i32,
        },
    }

    enum Role {
        IdRole,
        PathRole,
        TitleRole,
    }

    impl FromStr for Role {
        type Err = ();
        fn from_str(input: &str) -> Result<Role, Self::Err> {
            match input {
                "id" => Ok(Role::IdRole),
                "title" => Ok(Role::TitleRole),
                "path" => Ok(Role::PathRole),
                _ => Err(()),
            }
        }
    }

    // use crate::entities::{images, prelude::Images};
    // use sea_orm::{ConnectionTrait, Database, DbBackend, DbErr, Statement, ActiveValue};
    impl qobject::ImageModel {
        #[qinvokable]
        pub fn clear(mut self: Pin<&mut Self>) {
            unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().images_mut().clear();
                self.as_mut().end_reset_model();
            }
        }

        #[qinvokable]
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

        #[qinvokable]
        pub fn remove_item(
            mut self: Pin<&mut Self>,
            index: i32,
        ) -> bool {
            if index < 0 || (index as usize) >= self.images().len() {
                return false;
            }
            let db = &mut self.as_mut().get_db();

            let image_id =
                self.images().get(index as usize).unwrap().id;

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
                        self.as_mut()
                            .images_mut()
                            .remove(index as usize);
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

            SqliteConnection::establish(&db_url).unwrap_or_else(
                |_| panic!("error connecting to {}", db_url),
            )
        }

        #[qinvokable]
        pub fn new_item(mut self: Pin<&mut Self>, url: QUrl) {
            println!("LETS INSERT THIS SUCKER!");
            let file_path = PathBuf::from(url.path().to_string());
            let name =
                file_path.file_stem().unwrap().to_str().unwrap();
            let image_id = self.rust().highest_id + 1;
            let image_title = QString::from(name);
            let image_path = url.to_qstring();

            if self.as_mut().add_item(
                image_id,
                image_title,
                image_path,
            ) {
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

        #[qinvokable]
        pub fn update_title(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_title: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role_id(Role::TitleRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

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

        #[qinvokable]
        pub fn update_file_path(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_file_path: QString,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role_id(Role::PathRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

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
            if let Some(image) =
                self.rust().images.get(index as usize)
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

        fn get_role_id(&self, role: Role) -> i32 {
            match role {
                Role::IdRole => 0,
                Role::TitleRole => 1,
                Role::PathRole => 2,
                _ => 0,
            }
        }
    }

    // Create Rust bindings for C++ functions of the base class (QAbstractItemModel)
    #[cxx_qt::inherit]
    extern "C++" {
        unsafe fn begin_insert_rows(
            self: Pin<&mut qobject::ImageModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_insert_rows(
            self: Pin<&mut qobject::ImageModel>,
        );

        unsafe fn begin_remove_rows(
            self: Pin<&mut qobject::ImageModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_remove_rows(
            self: Pin<&mut qobject::ImageModel>,
        );

        unsafe fn begin_reset_model(
            self: Pin<&mut qobject::ImageModel>,
        );
        unsafe fn end_reset_model(
            self: Pin<&mut qobject::ImageModel>,
        );
    }

    #[cxx_qt::inherit]
    unsafe extern "C++" {
        #[cxx_name = "canFetchMore"]
        fn base_can_fetch_more(
            self: &qobject::ImageModel,
            parent: &QModelIndex,
        ) -> bool;

        fn index(
            self: &qobject::ImageModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    // QAbstractListModel implementation
    impl qobject::ImageModel {
        #[qinvokable(cxx_override)]
        fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
            if let Some(image) =
                self.images().get(index.row() as usize)
            {
                return match role {
                    0 => QVariant::from(&image.id),
                    1 => QVariant::from(&image.title),
                    2 => QVariant::from(&image.path),
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
            roles.insert(2, cxx_qt_lib::QByteArray::from("filePath"));
            roles
        }

        #[qinvokable(cxx_override)]
        pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
            let cnt = self.rust().images.len() as i32;
            // self.as_mut().set_count(cnt);
            // println!("row count is {cnt}");
            cnt
        }

        #[qinvokable]
        pub fn count(mut self: Pin<&mut Self>) -> i32 {
            let cnt = self.rust().images.len() as i32;
            self.as_mut().set_count_rows(cnt);
            cnt
        }
    }
}
