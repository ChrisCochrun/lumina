#[cxx_qt::bridge]
mod service_item_model {
    unsafe extern "C++" {
        include!(< QAbstractListModel >);
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
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
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    #[cxx_qt::qobject]
    #[derive(Clone, Debug)]
    pub struct ServiceItm {
        #[qproperty]
        name: QString,
        #[qproperty]
        ty: QString,
        #[qproperty]
        audio: QString,
        #[qproperty]
        background: QString,
        #[qproperty]
        background_type: QString,
        #[qproperty]
        text: QStringList,
        #[qproperty]
        font: QString,
        #[qproperty]
        font_size: i32,
        #[qproperty]
        slide_count: i32,
        #[qproperty]
        active: bool,
        #[qproperty]
        selected: bool,
        #[qproperty]
        looping: bool,
        #[qproperty]
        video_start_time: f32,
        #[qproperty]
        video_end_time: f32,
    }

    impl Default for ServiceItm {
        fn default() -> Self {
            Self {
                name: QString::default(),
                ty: QString::default(),
                audio: QString::default(),
                background: QString::default(),
                background_type: QString::default(),
                text: QStringList::default(),
                font: QString::default(),
                font_size: 50,
                slide_count: 1,
                active: false,
                selected: false,
                looping: false,
                video_start_time: 0.0,
                video_end_time: 0.0,
            }
        }
    }

    #[cxx_qt::qobject(base = "QAbstractListModel")]
    #[derive(Default, Debug)]
    pub struct ServiceItemMod {
        id: i32,
        service_items: Vec<ServiceItm>,
    }

    #[cxx_qt::qsignals(ServiceItemMod)]
    pub enum Signals<'a> {
        #[inherit]
        DataChanged {
            top_left: &'a QModelIndex,
            bottom_right: &'a QModelIndex,
            roles: &'a QVector_i32,
        },
        ActiveChanged,
        SelectedChanged,
        ItemInserted {
            index: &'a i32,
            item: &'a QMap_QString_QVariant,
        },
        ItemAdded {
            index: &'a i32,
            item: &'a QMap_QString_QVariant,
        },
        ItemRemoved {
            index: &'a i32,
            item: &'a QMap_QString_QVariant,
        },
        ItemMoved {
            source_index: &'a i32,
            dest_index: &'a i32,
            // index: &'a i32,
            item: &'a QMap_QString_QVariant,
        },
    }

    enum Role {
        NameRole,
        TyRole,
        AudioRole,
        BackgroundRole,
        BackgroundTypeRole,
        TextRole,
        FontRole,
        FontSizeRole,
        SlideCountRole,
        ActiveRole,
        SelectedRole,
        LoopingRole,
        VideoStartTimeRole,
        VideoEndTimeRole,
    }

    // use crate::video_thumbnail;
    // use image::{ImageBuffer, Rgba};
    use dirs;
    use std::fs;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::str;
    impl qobject::ServiceItemMod {
        #[qinvokable]
        pub fn clear(mut self: Pin<&mut Self>) {
            unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().service_items_mut().clear();
                self.as_mut().end_reset_model();
            }
        }

        #[qinvokable]
        pub fn remove_item(mut self: Pin<&mut Self>, index: i32) {
            if index < 0 || (index as usize) >= self.service_items().len() {
                return;
            }

            unsafe {
                self.as_mut()
                    .begin_remove_rows(&QModelIndex::default(), index, index);
                self.as_mut().service_items_mut().remove(index as usize);
                self.as_mut().end_remove_rows();
            }
        }

        #[qinvokable]
        pub fn add_item(
            mut self: Pin<&mut Self>,
            name: QString,
            ty: QString,
            background: QString,
            background_type: QString,
            text: QStringList,
            audio: QString,
            font: QString,
            font_size: i32,
            slide_count: i32,
            looping: bool,
            video_start_time: f32,
            video_end_time: f32,
        ) {
            let service_item = ServiceItm {
                name,
                ty,
                text,
                background,
                background_type,
                audio,
                font,
                font_size,
                slide_count,
                looping,
                video_start_time,
                video_end_time,
                ..Default::default()
            };

            self.as_mut().add_service_item(&service_item);
        }

        fn add_service_item(mut self: Pin<&mut Self>, service_item: &ServiceItm) {
            let index = self.as_ref().service_items().len() as i32;
            println!("{:?}", service_item);
            let service_item = service_item.clone();
            unsafe {
                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), index, index);
                self.as_mut().service_items_mut().push(service_item);
                self.as_mut().end_insert_rows();
            }
        }

        #[qinvokable]
        pub fn insert_item(
            mut self: Pin<&mut Self>,
            index: i32,
            name: QString,
            text: QStringList,
            ty: QString,
            background: QString,
            background_type: QString,
            audio: QString,
            font: QString,
            font_size: i32,
            slide_count: i32,
            looping: bool,
            video_start_time: f32,
            video_end_time: f32,
        ) {
            let service_item = ServiceItm {
                name,
                ty,
                text,
                background,
                background_type,
                audio,
                font,
                font_size,
                slide_count,
                looping,
                video_start_time,
                video_end_time,
                ..Default::default()
            };

            self.as_mut().insert_service_item(&service_item, index);
        }

        fn insert_service_item(mut self: Pin<&mut Self>, service_item: &ServiceItm, id: i32) {
            let service_item = service_item.clone();
            unsafe {
                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), id, id);
                self.as_mut()
                    .service_items_mut()
                    .insert(id as usize, service_item);
                self.as_mut().end_insert_rows();
            }
        }

        #[qinvokable]
        pub fn get_item(self: Pin<&mut Self>, index: i32) -> QMap_QString_QVariant {
            println!("{index}");
            let mut map = QMap_QString_QVariant::default();
            let idx = self.index(index, 0, &QModelIndex::default());
            if !idx.is_valid() {
                return map;
            }
            let rn = self.as_ref().role_names();
            let rn_iter = rn.iter();
            if let Some(service_item) = self.rust().service_items.get(index as usize) {
                for i in rn_iter {
                    map.insert(
                        QString::from(&i.1.to_string()),
                        self.as_ref().data(&idx, *i.0),
                    );
                }
            };
            map
        }

        #[qinvokable]
        pub fn move_rows(
            mut self: Pin<&mut Self>,
            source_index: i32,
            dest_index: i32,
            count: i32,
        ) -> bool {
            todo!();
        }

        #[qinvokable]
        pub fn move_up(mut self: Pin<&mut Self>, index: i32) -> bool {
            todo!();
        }

        #[qinvokable]
        pub fn move_down(mut self: Pin<&mut Self>, index: i32) -> bool {
            todo!();
        }

        #[qinvokable]
        pub fn select(mut self: Pin<&mut Self>, index: i32) -> bool {
            todo!();
        }

        #[qinvokable]
        pub fn select_items(mut self: Pin<&mut Self>, items: QMap_QString_QVariant) -> bool {
            todo!();
        }

        #[qinvokable]
        pub fn activate(mut self: Pin<&mut Self>, index: i32) -> bool {
            let rc = self.as_ref().count() - 1;
            let tl = &self.as_ref().index(0, 0, &QModelIndex::default());
            let br = &self.as_ref().index(rc, 0, &QModelIndex::default());
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(self.get_role(Role::ActiveRole));
            for service_item in self.as_mut().service_items_mut().iter_mut() {
                // println!("service_item is deactivating {:?}", i);
                service_item.active = false;
            }
            if let Some(service_item) = self.as_mut().service_items_mut().get_mut(index as usize) {
                println!("service_item is activating {:?}", index);
                println!("service_item_title: {:?}", service_item.name);
                println!("service_item_background: {:?}", service_item.background);
                println!(
                    "service_item_background_type: {:?}",
                    service_item.background_type
                );
                service_item.active = true;
                self.as_mut().emit_data_changed(tl, br, &vector_roles);
                // We use this signal generated by our signals enum to tell QML that
                // the active service_item has changed which is used to reposition views.
                self.as_mut().emit_active_changed();
                true
            } else {
                false
            }
        }

        #[qinvokable]
        pub fn deactivate(mut self: Pin<&mut Self>, index: i32) -> bool {
            todo!();
        }

        #[qinvokable]
        pub fn save(mut self: Pin<&mut Self>, file: QUrl) -> bool {
            todo!();
        }

        #[qinvokable]
        pub fn load(mut self: Pin<&mut Self>, file: QUrl) -> bool {
            // todo!();
            println!("THE LAST SAVE FILE ISSSSS: {file}");
            let lf = PathBuf::from(file.to_local_file().unwrap_or_default().to_string());
            println!("{:?}", lf);
            false
        }

        #[qinvokable]
        pub fn load_last_saved(mut self: Pin<&mut Self>) -> bool {
            todo!();
        }

        fn get_role(&self, role: Role) -> i32 {
            match role {
                Role::NameRole => 0,
                Role::TyRole => 1,
                Role::AudioRole => 2,
                Role::BackgroundRole => 3,
                Role::BackgroundTypeRole => 4,
                Role::TextRole => 5,
                Role::FontRole => 6,
                Role::FontSizeRole => 7,
                Role::SlideCountRole => 8,
                Role::ActiveRole => 9,
                Role::SelectedRole => 10,
                Role::LoopingRole => 11,
                Role::VideoStartTimeRole => 12,
                Role::VideoEndTimeRole => 13,
                _ => 0,
            }
        }
    }

    // Create Rust bindings for C++ functions of the base class (QAbstractItem_Modelel)
    #[cxx_qt::inherit]
    extern "C++" {
        unsafe fn begin_insert_rows(
            self: Pin<&mut qobject::ServiceItemMod>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_insert_rows(self: Pin<&mut qobject::ServiceItemMod>);

        unsafe fn begin_remove_rows(
            self: Pin<&mut qobject::ServiceItemMod>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_remove_rows(self: Pin<&mut qobject::ServiceItemMod>);

        unsafe fn begin_reset_model(self: Pin<&mut qobject::ServiceItemMod>);
        unsafe fn end_reset_model(self: Pin<&mut qobject::ServiceItemMod>);
    }

    #[cxx_qt::inherit]
    unsafe extern "C++" {
        #[cxx_name = "canFetchMore"]
        fn base_can_fetch_more(self: &qobject::ServiceItemMod, parent: &QModelIndex) -> bool;

        fn index(
            self: &qobject::ServiceItemMod,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    // QAbstractListModel implementation
    impl qobject::ServiceItemMod {
        #[qinvokable(cxx_override)]
        fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
            if let Some(service_item) = self.service_items().get(index.row() as usize) {
                return match role {
                    0 => QVariant::from(&service_item.name),
                    1 => QVariant::from(&service_item.ty),
                    2 => QVariant::from(&service_item.audio),
                    3 => QVariant::from(&service_item.background),
                    4 => QVariant::from(&service_item.background_type),
                    5 => QVariant::from(&service_item.text),
                    6 => QVariant::from(&service_item.font),
                    7 => QVariant::from(&service_item.font_size),
                    8 => QVariant::from(&service_item.slide_count),
                    9 => QVariant::from(&service_item.active),
                    10 => QVariant::from(&service_item.selected),
                    11 => QVariant::from(&service_item.looping),
                    12 => QVariant::from(&service_item.video_start_time),
                    13 => QVariant::from(&service_item.video_end_time),
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
            roles.insert(0, cxx_qt_lib::QByteArray::from("name"));
            roles.insert(1, cxx_qt_lib::QByteArray::from("ty"));
            roles.insert(2, cxx_qt_lib::QByteArray::from("audio"));
            roles.insert(3, cxx_qt_lib::QByteArray::from("background"));
            roles.insert(4, cxx_qt_lib::QByteArray::from("backgroundType"));
            roles.insert(5, cxx_qt_lib::QByteArray::from("text"));
            roles.insert(6, cxx_qt_lib::QByteArray::from("font"));
            roles.insert(7, cxx_qt_lib::QByteArray::from("fontSize"));
            roles.insert(8, cxx_qt_lib::QByteArray::from("slideCount"));
            roles.insert(9, cxx_qt_lib::QByteArray::from("active"));
            roles.insert(10, cxx_qt_lib::QByteArray::from("selected"));
            roles.insert(11, cxx_qt_lib::QByteArray::from("looping"));
            roles.insert(12, cxx_qt_lib::QByteArray::from("videoStartTime"));
            roles.insert(13, cxx_qt_lib::QByteArray::from("videoEndTime"));
            roles
        }

        #[qinvokable(cxx_override)]
        pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
            let cnt = self.rust().service_items.len() as i32;
            // println!("row count is {cnt}");
            cnt
        }

        #[qinvokable]
        pub fn count(&self) -> i32 {
            self.rust().service_items.len() as i32
        }
    }
}
