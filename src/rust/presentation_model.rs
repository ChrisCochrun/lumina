#[cxx_qt::bridge]
mod presentation_model {
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

    #[qenum(PresentationModel)]
    enum PresRoles {
        Id,
        Title,
        Path,
        Html,
        PageCount,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QAbstractListModel"]
        #[qml_element]
        #[qproperty(i32, count)]
        type PresentationModel = super::PresentationModelRust;

        #[inherit]
        #[qsignal]
        fn data_changed(
            self: Pin<&mut PresentationModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[qinvokable]
        fn clear(self: Pin<&mut PresentationModel>);
        #[qinvokable]
        fn setup(self: Pin<&mut PresentationModel>);
        #[qinvokable]
        fn remove_item(
            self: Pin<&mut PresentationModel>,
            index: i32,
        ) -> bool;
        #[qinvokable]
        fn new_item(
            self: Pin<&mut PresentationModel>,
            url: QUrl,
            new_page_count: i32,
        );
        // #[qinvokable]
        // fn update_path(
        //     self: Pin<&mut PresentationModel>,
        //     index: i32,
        //     updated_path: QString,
        // ) -> bool;
        // #[qinvokable]
        // fn duplicate_item(self: Pin<&mut Self>, index: i32) -> bool;

        #[qinvokable]
        fn get_item(
            self: Pin<&mut PresentationModel>,
            index: i32,
        ) -> QMap_QString_QVariant;
        #[qinvokable]
        fn update_title(
            self: Pin<&mut PresentationModel>,
            index: i32,
            updated_title: QString,
        ) -> bool;
        #[qinvokable]
        fn update_page_count(
            self: Pin<&mut PresentationModel>,
            index: i32,
            updated_page_count: i32,
        ) -> bool;
    }

    impl cxx_qt::Threading for PresentationModel {}

    unsafe extern "RustQt" {
        #[inherit]
        unsafe fn begin_insert_rows(
            self: Pin<&mut PresentationModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_insert_rows(self: Pin<&mut PresentationModel>);

        #[inherit]
        unsafe fn begin_remove_rows(
            self: Pin<&mut PresentationModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn begin_move_rows(
            self: Pin<&mut PresentationModel>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;

        #[inherit]
        unsafe fn end_move_rows(self: Pin<&mut PresentationModel>);

        #[inherit]
        unsafe fn end_remove_rows(self: Pin<&mut PresentationModel>);

        #[inherit]
        unsafe fn begin_reset_model(
            self: Pin<&mut PresentationModel>,
        );

        #[inherit]
        unsafe fn end_reset_model(self: Pin<&mut PresentationModel>);

        #[inherit]
        fn can_fetch_more(
            self: &PresentationModel,
            parent: &QModelIndex,
        ) -> bool;

        #[inherit]
        fn index(
            self: &PresentationModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        fn data(
            self: &PresentationModel,
            index: &QModelIndex,
            role: i32,
        ) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        fn role_names(
            self: &PresentationModel,
        ) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn row_count(
            self: &PresentationModel,
            _parent: &QModelIndex,
        ) -> i32;
    }
}

use crate::presentation_model::presentation_model::QMap_QString_QVariant;
use crate::reveal_js;
use crate::schema::presentations::dsl::*;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QModelIndex, QString, QUrl, QVariant};
use diesel::sqlite::SqliteConnection;
use diesel::{delete, insert_into, prelude::*, update};
// use sqlx::Connection;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::debug;

use self::presentation_model::{
    PresRoles, QHash_i32_QByteArray, QVector_i32,
};

#[derive(Default, Clone, Debug)]
pub struct Presentation {
    id: i32,
    title: String,
    html: bool,
    path: String,
    page_count: i32,
}

#[derive(Default, Debug)]
pub struct PresentationModelRust {
    count: i32,
    highest_id: i32,
    presentations: Vec<Presentation>,
}

impl presentation_model::PresentationModel {
    pub fn clear(mut self: Pin<&mut Self>) {
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().presentations.clear();
            self.as_mut().end_reset_model();
        }
    }

    pub fn setup(mut self: Pin<&mut Self>) {
        let db = &mut self.as_mut().get_db();
        // let table_info = diesel::sql_query("PRAGMA table_info(presentations)").load(db);
        // println!("{:?}", table_info);
        let results = presentations
            .load::<crate::models::Presentation>(db)
            .expect("Error loading presentations");
        self.as_mut().rust_mut().highest_id = 0;

        println!("SHOWING PRESENTATIONS");
        println!("--------------");
        for presentation in results {
            println!("{}", presentation.title);
            println!("{}", presentation.id);
            println!("{}", presentation.path);
            println!("{}", presentation.html);
            println!("--------------");
            if &self.as_mut().highest_id < &presentation.id {
                self.as_mut().rust_mut().highest_id = presentation.id;
            }

            let pres = self::Presentation {
                id: presentation.id,
                title: presentation.title,
                html: presentation.path.ends_with(".html"),
                path: presentation.path,
                page_count: presentation.page_count.unwrap(),
            };

            let count = self.as_ref().count;
            self.as_mut().set_count(count + 1);
            self.as_mut().add_presentation(pres);
        }
        println!("--------------------------------------");
        println!("{:?}", self.as_mut().presentations);
        println!("--------------------------------------");
    }

    pub fn remove_item(mut self: Pin<&mut Self>, index: i32) -> bool {
        if index < 0 || (index as usize) >= self.presentations.len() {
            return false;
        }
        let db = &mut self.as_mut().get_db();

        let presentation_id =
            self.presentations.get(index as usize).unwrap().id;

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
                        .rust_mut()
                        .presentations
                        .remove(index as usize);
                    self.as_mut().end_remove_rows();
                }
                println!(
                    "removed-item-at-index: {:?}",
                    presentation_id
                );
                println!(
                    "new-Vec: {:?}",
                    self.as_mut().presentations
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

        SqliteConnection::establish(&db_url).unwrap_or_else(|_| {
            panic!("error connecting to {}", db_url)
        })
    }

    pub fn new_item(
        mut self: Pin<&mut Self>,
        url: QUrl,
        new_page_count: i32,
    ) {
        println!("LETS INSERT THIS SUCKER!");
        let file_path = PathBuf::from(url.path().to_string());
        let name = file_path.file_stem().unwrap().to_str().unwrap();
        let presentation_id = self.highest_id + 1;
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
            debug!(filename = ?name, "Item inserted");
            self.as_mut().rust_mut().highest_id = presentation_id;
            let cnt =
                self.as_mut().row_count(&QModelIndex::default());
            self.as_mut().set_count(cnt);
        } else {
            debug!("Error in inserting item");
        }
    }

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
            let actual_path =
                PathBuf::from(presentation_path.path().to_string());
            actual_page_count =
                reveal_js::count_slides_and_fragments(&actual_path);
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
                println!("{:?}", self.as_mut().presentations);
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
        let index = self.as_ref().presentations.len() as i32;
        println!("{:?}", presentation);
        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut().rust_mut().presentations.push(presentation);
            self.as_mut().end_insert_rows();
        }
    }

    // fn insert_presentation(
    //     mut self: Pin<&mut Self>,
    //     presentation: Presentation,
    //     index: i32,
    // ) {
    //     unsafe {
    //         self.as_mut().begin_insert_rows(
    //             &QModelIndex::default(),
    //             index,
    //             index,
    //         );
    //         self.as_mut()
    //             .rust_mut()
    //             .presentations
    //             .insert(index as usize, presentation);
    //         self.as_mut().end_insert_rows();
    //     }
    //     let iter = self
    //         .as_mut()
    //         .presentations
    //         .iter()
    //         .enumerate()
    //         .filter(|p| p.id > index);
    //     for p in iter {
    //         p.id = p.id + 1;
    //     }
    // }

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
            self.presentations.get(index as usize)
        {
            for i in role_names_iter {
                qvariantmap.insert(
                    QString::from(&i.1.to_string()),
                    self.as_ref().data(&idx, *i.0),
                );
            }
            debug!("{:?}", presentation);
        };
        qvariantmap
    }

    pub fn duplicate_item(
        mut self: Pin<&mut Self>,
        index: i32,
    ) -> bool {
        let binding = self.as_mut();
        let pres = binding.presentations.get(index as usize).clone();
        if let Some(item) = pres {
            let item = item.clone();
            binding.add_presentation(item);
            true
        } else {
            false
        }
    }

    pub fn update_title(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_title: QString,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.as_ref().get_role(PresRoles::Title));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(presentations.filter(id.eq(index)))
            .set(title.eq(updated_title.to_string()))
            .execute(db);
        match result {
            Ok(_i) => {
                for presentation in self
                    .as_mut()
                    .rust_mut()
                    .presentations
                    .iter_mut()
                    .filter(|x| x.id == index)
                {
                    presentation.title = updated_title.to_string();
                    println!("rust-title: {:?}", presentation.title);
                }
                // TODO this seems to not be updating in the actual list
                self.as_mut().data_changed(
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

    pub fn update_page_count(
        mut self: Pin<&mut Self>,
        index: i32,
        updated_page_count: i32,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.as_ref().get_role(PresRoles::PageCount));
        let model_index =
            &self.as_ref().index(index, 0, &QModelIndex::default());

        let db = &mut self.as_mut().get_db();
        let result = update(presentations.filter(id.eq(index)))
            .set(page_count.eq(updated_page_count))
            .execute(db);
        match result {
            Ok(_i) => {
                for presentation in self
                    .as_mut()
                    .rust_mut()
                    .presentations
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
                self.as_mut().data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
                // self.as_mut().emit_page_count_changed();
                println!("rust-page_count: {:?}", updated_page_count);
                true
            }
            Err(_e) => false,
        }
    }

    fn get_role(&self, role: PresRoles) -> i32 {
        match role {
            PresRoles::Id => 0,
            PresRoles::Title => 1,
            PresRoles::Path => 2,
            PresRoles::Html => 3,
            PresRoles::PageCount => 4,
            _ => 0,
        }
    }
}

// QAbstractListModel implementation
impl presentation_model::PresentationModel {
    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = PresRoles { repr: role };
        if let Some(presentation) =
            self.presentations.get(index.row() as usize)
        {
            return match role {
                PresRoles::Id => QVariant::from(&presentation.id),
                PresRoles::Title => QVariant::from(&QString::from(
                    &presentation.title,
                )),
                PresRoles::Path => {
                    QVariant::from(&QString::from(&presentation.path))
                }
                PresRoles::Html => QVariant::from(&presentation.html),
                PresRoles::PageCount => {
                    QVariant::from(&presentation.page_count)
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
        roles.insert(
            PresRoles::Id.repr,
            cxx_qt_lib::QByteArray::from("id"),
        );
        roles.insert(
            PresRoles::Title.repr,
            cxx_qt_lib::QByteArray::from("title"),
        );
        roles.insert(
            PresRoles::Path.repr,
            cxx_qt_lib::QByteArray::from("filePath"),
        );
        roles.insert(
            PresRoles::Html.repr,
            cxx_qt_lib::QByteArray::from("html"),
        );
        roles.insert(
            PresRoles::PageCount.repr,
            cxx_qt_lib::QByteArray::from("pageCount"),
        );
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        let cnt = self.presentations.len() as i32;
        // println!("row count is {cnt}");
        cnt
    }
}
