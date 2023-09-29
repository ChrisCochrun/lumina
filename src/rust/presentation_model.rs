#[cxx_qt::bridge]
mod presentation_model {
    use crate::models::*;
    use crate::presentation_model::presentation_model::Presentation;
    use crate::reveal_js;
    use crate::schema::presentations::dsl::*;
    use diesel::sqlite::SqliteConnection;
    use diesel::{delete, insert_into, prelude::*, update};
    // use sqlx::Connection;
    use std::path::{Path, PathBuf};
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
    pub struct Presentation {
        id: i32,
        title: String,
        html: bool,
        path: String,
        page_count: i32,
    }

    #[cxx_qt::qobject(base = "QAbstractListModel")]
    #[derive(Default, Debug)]
    pub struct PresentationModel {
        highest_id: i32,
        presentations: Vec<self::Presentation>,
    }

    #[cxx_qt::qsignals(PresentationModel)]
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
        HtmlRole,
        PageCountRole,
    }

    // use crate::entities::{presentations, prelude::Presentations};
    // use sea_orm::{ConnectionTrait, Database, DbBackend, DbErr, Statement, ActiveValue};
    impl qobject::PresentationModel {
        #[qinvokable]
        pub fn clear(mut self: Pin<&mut Self>) {
            unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().presentations_mut().clear();
                self.as_mut().end_reset_model();
            }
        }

        #[qinvokable]
        pub fn setup(mut self: Pin<&mut Self>) {
            let db = &mut self.as_mut().get_db();
            // let table_info = diesel::sql_query("PRAGMA table_info(presentations)").load(db);
            // println!("{:?}", table_info);
            let results = presentations
                .load::<crate::models::Presentation>(db)
                .expect("Error loading presentations");
            self.as_mut().set_highest_id(0);

            println!("SHOWING PRESENTATIONS");
            println!("--------------");
            for presentation in results {
                println!("{}", presentation.title);
                println!("{}", presentation.id);
                println!("{}", presentation.path);
                println!("{}", presentation.html);
                println!("--------------");
                if self.as_mut().highest_id() < &presentation.id {
                    self.as_mut().set_highest_id(presentation.id);
                }

                let pres = self::Presentation {
                    id: presentation.id,
                    title: presentation.title,
                    html: presentation.path.ends_with(".html"),
                    path: presentation.path,
                    page_count: presentation.page_count.unwrap(),
                };

                self.as_mut().add_presentation(pres);
            }
            println!("--------------------------------------");
            println!("{:?}", self.as_mut().presentations());
            println!("--------------------------------------");
        }

        #[qinvokable]
        pub fn remove_item(
            mut self: Pin<&mut Self>,
            index: i32,
        ) -> bool {
            if index < 0
                || (index as usize) >= self.presentations().len()
            {
                return false;
            }
            let db = &mut self.as_mut().get_db();

            let presentation_id =
                self.presentations().get(index as usize).unwrap().id;

            let result =
                delete(presentations.filter(id.eq(presentation_id)))
                    .execute(db);

            match result {
                Ok(_i) => {
                    unsafe {
                        self.as_mut().begin_remove_rows(
                            &QModelIndex::default(),
                            index,
                            index,
                        );
                        self.as_mut()
                            .presentations_mut()
                            .remove(index as usize);
                        self.as_mut().end_remove_rows();
                    }
                    println!(
                        "removed-item-at-index: {:?}",
                        presentation_id
                    );
                    println!(
                        "new-Vec: {:?}",
                        self.as_mut().presentations()
                    );
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
        pub fn new_item(
            mut self: Pin<&mut Self>,
            url: QUrl,
            new_page_count: i32,
        ) {
            println!("LETS INSERT THIS SUCKER!");
            let file_path = PathBuf::from(url.path().to_string());
            let name =
                file_path.file_stem().unwrap().to_str().unwrap();
            let presentation_id = self.rust().highest_id + 1;
            let presentation_title = QString::from(name);
            let presentation_path = url;
            let presentation_html =
                file_path.extension().unwrap() == "html";
            debug!(html = presentation_html, ?file_path, extension = ?file_path.extension());

            if self.as_mut().add_item(
                presentation_id,
                presentation_title,
                presentation_path,
                presentation_html,
                new_page_count,
            ) {
                println!("filename: {:?}", name);
                self.as_mut().set_highest_id(presentation_id);
            } else {
                println!("Error in inserting item");
            }
        }

        #[qinvokable]
        pub fn add_item(
            mut self: Pin<&mut Self>,
            presentation_id: i32,
            presentation_title: QString,
            presentation_path: QUrl,
            presentation_html: bool,
            new_page_count: i32,
        ) -> bool {
            let db = &mut self.as_mut().get_db();
            // println!("{:?}", db);
            let mut actual_page_count = new_page_count;
            if presentation_html {
                let actual_path = PathBuf::from(
                    presentation_path.path().to_string(),
                );
                actual_page_count =
                    reveal_js::count_slides_and_fragments(
                        actual_path,
                    );
            }
            debug!(
                page_count = actual_page_count,
                html = presentation_html
            );

            let presentation = self::Presentation {
                id: presentation_id,
                title: presentation_title.clone().to_string(),
                html: presentation_html,
                path: presentation_path.clone().to_string(),
                page_count: actual_page_count,
            };
            println!("{:?}", presentation);

            let result = insert_into(presentations)
                .values((
                    id.eq(&presentation_id),
                    title.eq(&presentation_title.to_string()),
                    path.eq(&presentation_path.to_string()),
                    html.eq(&presentation_html),
                    page_count.eq(&presentation.page_count),
                ))
                .execute(db);
            println!("{:?}", result);

            match result {
                Ok(_i) => {
                    self.as_mut().add_presentation(presentation);
                    println!("{:?}", self.as_mut().presentations());
                    true
                }
                Err(_e) => {
                    println!("Cannot connect to database");
                    false
                }
            }
        }

        fn add_presentation(
            mut self: Pin<&mut Self>,
            presentation: self::Presentation,
        ) {
            let index = self.as_ref().presentations().len() as i32;
            println!("{:?}", presentation);
            unsafe {
                self.as_mut().begin_insert_rows(
                    &QModelIndex::default(),
                    index,
                    index,
                );
                self.as_mut().presentations_mut().push(presentation);
                self.as_mut().end_insert_rows();
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
            if let Some(presentation) =
                self.rust().presentations.get(index as usize)
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
            let result = update(presentations.filter(id.eq(index)))
                .set(title.eq(updated_title.to_string()))
                .execute(db);
            match result {
                Ok(_i) => {
                    for presentation in self
                        .as_mut()
                        .presentations_mut()
                        .iter_mut()
                        .filter(|x| x.id == index)
                    {
                        presentation.title =
                            updated_title.to_string();
                        println!(
                            "rust-title: {:?}",
                            presentation.title
                        );
                    }
                    // TODO this seems to not be updating in the actual list
                    self.as_mut().emit_data_changed(
                        model_index,
                        model_index,
                        &vector_roles,
                    );
                    // self.as_mut().emit_title_changed();
                    println!("rust-title: {:?}", updated_title);
                    true
                }
                Err(_e) => false,
            }
        }

        #[qinvokable]
        pub fn update_page_count(
            mut self: Pin<&mut Self>,
            index: i32,
            updated_page_count: i32,
        ) -> bool {
            let mut vector_roles = QVector_i32::default();
            vector_roles
                .append(self.as_ref().get_role(Role::PageCountRole));
            let model_index = &self.as_ref().index(
                index,
                0,
                &QModelIndex::default(),
            );

            let db = &mut self.as_mut().get_db();
            let result = update(presentations.filter(id.eq(index)))
                .set(page_count.eq(updated_page_count))
                .execute(db);
            match result {
                Ok(_i) => {
                    for presentation in self
                        .as_mut()
                        .presentations_mut()
                        .iter_mut()
                        .filter(|x| x.id == index)
                    {
                        presentation.page_count = updated_page_count;
                        println!(
                            "rust-page_count: {:?}",
                            presentation.page_count
                        );
                    }
                    // TODO this seems to not be updating in the actual list
                    self.as_mut().emit_data_changed(
                        model_index,
                        model_index,
                        &vector_roles,
                    );
                    // self.as_mut().emit_page_count_changed();
                    println!(
                        "rust-page_count: {:?}",
                        updated_page_count
                    );
                    true
                }
                Err(_e) => false,
            }
        }

        fn get_role(&self, role: Role) -> i32 {
            match role {
                Role::IdRole => 0,
                Role::TitleRole => 1,
                Role::PathRole => 2,
                Role::HtmlRole => 3,
                Role::PageCountRole => 4,
                _ => 0,
            }
        }
    }

    // Create Rust bindings for C++ functions of the base class (QAbstractItemModel)
    #[cxx_qt::inherit]
    extern "C++" {
        unsafe fn begin_insert_rows(
            self: Pin<&mut qobject::PresentationModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_insert_rows(
            self: Pin<&mut qobject::PresentationModel>,
        );

        unsafe fn begin_remove_rows(
            self: Pin<&mut qobject::PresentationModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_remove_rows(
            self: Pin<&mut qobject::PresentationModel>,
        );

        unsafe fn begin_reset_model(
            self: Pin<&mut qobject::PresentationModel>,
        );
        unsafe fn end_reset_model(
            self: Pin<&mut qobject::PresentationModel>,
        );
    }

    #[cxx_qt::inherit]
    unsafe extern "C++" {
        #[cxx_name = "canFetchMore"]
        fn base_can_fetch_more(
            self: &qobject::PresentationModel,
            parent: &QModelIndex,
        ) -> bool;

        fn index(
            self: &qobject::PresentationModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    // QAbstractListModel implementation
    impl qobject::PresentationModel {
        #[qinvokable(cxx_override)]
        fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
            if let Some(presentation) =
                self.presentations().get(index.row() as usize)
            {
                return match role {
                    0 => QVariant::from(&presentation.id),
                    1 => QVariant::from(&QString::from(
                        &presentation.title,
                    )),
                    2 => QVariant::from(&QString::from(
                        &presentation.path,
                    )),
                    3 => QVariant::from(&presentation.html),
                    4 => QVariant::from(&presentation.page_count),
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
            roles.insert(3, cxx_qt_lib::QByteArray::from("html"));
            roles
                .insert(4, cxx_qt_lib::QByteArray::from("pageCount"));
            roles
        }

        #[qinvokable(cxx_override)]
        pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
            let cnt = self.rust().presentations.len() as i32;
            // println!("row count is {cnt}");
            cnt
        }

        #[qinvokable]
        pub fn count(&self) -> i32 {
            self.rust().presentations.len() as i32
        }
    }
}
