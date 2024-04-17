#[cxx_qt::bridge]
mod service_item_model {
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
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    #[qenum(ServiceItemModel)]
    enum ServiceRoles {
        Name,
        Type,
        Audio,
        Background,
        BackgroundType,
        Text,
        Font,
        FontSize,
        SlideCount,
        Active,
        Selected,
        Looping,
        VideoStartTime,
        VideoEndTime,
        Id,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QAbstractListModel"]
        #[qml_element]
        #[qproperty(i32, count)]
        #[qproperty(f32, save_progress)]
        #[qproperty(bool, saved)]
        type ServiceItemModel = super::ServiceItemModelRust;

        #[inherit]
        #[qsignal]
        fn data_changed(
            self: Pin<&mut ServiceItemModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[qsignal]
        fn active_changed(
            self: Pin<&mut ServiceItemModel>,
            index: &i32,
        );
        #[qsignal]
        fn selected_changed(self: Pin<&mut ServiceItemModel>);
        #[qsignal]
        fn item_inserted(
            self: Pin<&mut ServiceItemModel>,
            index: &i32,
            item: &QMap_QString_QVariant,
        );
        #[qsignal]
        fn item_added(
            self: Pin<&mut ServiceItemModel>,
            index: &i32,
            item: &QMap_QString_QVariant,
        );
        #[qsignal]
        fn item_removed(
            self: Pin<&mut ServiceItemModel>,
            index: &i32,
            item: &QMap_QString_QVariant,
        );
        #[qsignal]
        fn item_moved(
            self: Pin<&mut ServiceItemModel>,
            source_index: &i32,
            dest_index: &i32,
            item: &QMap_QString_QVariant,
        );
        #[qsignal]
        fn cleared(self: Pin<&mut ServiceItemModel>);

        #[qsignal]
        fn save_progress_updated(
            self: Pin<&mut ServiceItemModel>,
            progress: i32,
        );

        #[qsignal]
        fn saved_to_file(
            self: Pin<&mut ServiceItemModel>,
            saved: bool,
            file: &QUrl,
        );

        #[qinvokable]
        fn clear(self: Pin<&mut ServiceItemModel>);

        #[qinvokable]
        fn remove_item(self: Pin<&mut ServiceItemModel>, index: i32);

        #[qinvokable]
        fn add_item(
            self: Pin<&mut ServiceItemModel>,
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
            id: i32,
        );

        #[qinvokable]
        fn insert_item(
            self: Pin<&mut ServiceItemModel>,
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
            id: i32,
        );

        #[qinvokable]
        fn get_item(
            self: Pin<&mut ServiceItemModel>,
            index: i32,
        ) -> QMap_QString_QVariant;

        #[qinvokable]
        fn move_rows(
            self: Pin<&mut ServiceItemModel>,
            source_index: i32,
            dest_index: i32,
            count: i32,
        ) -> bool;

        #[qinvokable]
        fn move_up(
            self: Pin<&mut ServiceItemModel>,
            index: i32,
        ) -> bool;

        #[qinvokable]
        fn move_down(
            self: Pin<&mut ServiceItemModel>,
            index: i32,
        ) -> bool;

        #[qinvokable]
        fn select(self: Pin<&mut ServiceItemModel>, index: i32);

        #[qinvokable]
        fn select_items(
            self: Pin<&mut ServiceItemModel>,
            final_index: i32,
        ) -> bool;

        #[qinvokable]
        pub fn activate(
            self: Pin<&mut ServiceItemModel>,
            index: i32,
        ) -> bool;

        #[qinvokable]
        pub fn deactivate(
            self: Pin<&mut ServiceItemModel>,
            index: i32,
        ) -> bool;

        #[qinvokable]
        fn save(self: Pin<&mut ServiceItemModel>, file: QUrl)
            -> bool;

        #[qinvokable]
        fn load(self: Pin<&mut ServiceItemModel>, file: QUrl)
            -> bool;
    }

    impl cxx_qt::Threading for ServiceItemModel {}

    unsafe extern "RustQt" {
        #[inherit]
        unsafe fn begin_insert_rows(
            self: Pin<&mut ServiceItemModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_insert_rows(self: Pin<&mut ServiceItemModel>);

        #[inherit]
        unsafe fn begin_remove_rows(
            self: Pin<&mut ServiceItemModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn begin_move_rows(
            self: Pin<&mut ServiceItemModel>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;

        #[inherit]
        unsafe fn end_move_rows(self: Pin<&mut ServiceItemModel>);

        #[inherit]
        unsafe fn end_remove_rows(self: Pin<&mut ServiceItemModel>);

        #[inherit]
        unsafe fn begin_reset_model(self: Pin<&mut ServiceItemModel>);

        #[inherit]
        unsafe fn end_reset_model(self: Pin<&mut ServiceItemModel>);

        #[inherit]
        fn can_fetch_more(
            self: &ServiceItemModel,
            parent: &QModelIndex,
        ) -> bool;

        #[inherit]
        fn index(
            self: &ServiceItemModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        fn data(
            self: &ServiceItemModel,
            index: &QModelIndex,
            role: i32,
        ) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        fn role_names(
            self: &ServiceItemModel,
        ) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn row_count(
            self: &ServiceItemModel,
            _parent: &QModelIndex,
        ) -> i32;

    }
}

use crate::obs::Obs;
use crate::service_item_model::service_item_model::QList_QString;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{
    QByteArray, QModelIndex, QString, QStringList, QUrl, QVariant,
};
use dirs;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::iter;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Instant;
use std::{fs, println};
use tar::{Archive, Builder};
use tracing::{debug, error};
use zstd::{Decoder, Encoder};
use self::service_item_model::{
    QHash_i32_QByteArray, QMap_QString_QVariant, QVector_i32,
    ServiceRoles,
};

use super::service_item_model::service_item_model::ServiceItemModel;

#[derive(Clone, Debug)]
pub struct ServiceItem {
    name: QString,
    ty: QString,
    audio: QString,
    background: QString,
    background_type: QString,
    text: QStringList,
    font: QString,
    font_size: i32,
    slide_count: i32,
    active: bool,
    selected: bool,
    looping: bool,
    video_start_time: f32,
    video_end_time: f32,
    obs_scene: QString,
    id: i32,
}

impl ServiceItem {
    fn debug(self) {
        debug!(?self);
    }
}

impl Default for ServiceItem {
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
            obs_scene: QString::default(),
            id: 0,
        }
    }
}

#[derive(Debug)]
pub struct ServiceItemModelRust {
    id: i32,
    service_items: Vec<ServiceItem>,
    obs: Option<Obs>,
    count: i32,
    save_progress: f32,
    saved: bool,
}

impl Default for ServiceItemModelRust {
    fn default() -> Self {
        let obs =
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                match Obs::new().await {
                    Ok(o) => Some(o),
                    Err(e) => {
                        error!(e);
                        None
                    }
                }
            });
        Self {
            id: 0,
            service_items: Vec::new(),
            obs,
            count: 0,
            save_progress: 0.0,
            saved: false,
        }
    }
}

impl service_item_model::ServiceItemModel {
    pub fn setup(mut self: Pin<&mut Self>) {
        todo!()
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        println!("CLEARING ALL ITEMS");
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().service_items.clear();
            self.as_mut().end_reset_model();
        }
        self.as_mut().cleared();
    }

    pub fn remove_item(mut self: Pin<&mut Self>, index: i32) {
        if index < 0 || (index as usize) >= self.service_items.len() {
            return;
        }

        unsafe {
            self.as_mut().begin_remove_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut()
                .rust_mut()
                .service_items
                .remove(index as usize);
            self.as_mut().end_remove_rows();
        }
        let item = self.as_mut().get_item(index);
        self.as_mut().item_removed(&index, &item);
    }

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
        id: i32,
    ) {
        let service_item = ServiceItem {
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
            id,
            ..Default::default()
        };

        self.as_mut().add_service_item(&service_item);
    }

    fn add_service_item(
        mut self: Pin<&mut Self>,
        service_item: &ServiceItem,
    ) {
        let index = self.as_ref().service_items.len() as i32;
        println!("{:?}", service_item);
        let service_item = service_item.clone();
        let count = self.as_ref().count;
        self.as_mut().set_count(count + 1);
        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut().rust_mut().service_items.push(service_item);
            self.as_mut().end_insert_rows();
        }
        let item = self.as_mut().get_item(index);
        self.as_mut().item_added(&index, &item);
    }

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
        id: i32,
    ) {
        let service_item = ServiceItem {
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
            id,
            ..Default::default()
        };

        self.as_mut().insert_service_item(&service_item, index);
    }

    fn insert_service_item(
        mut self: Pin<&mut Self>,
        service_item: &ServiceItem,
        id: i32,
    ) {
        let service_item = service_item.clone();
        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                id,
                id,
            );
            self.as_mut()
                .rust_mut()
                .service_items
                .insert(id as usize, service_item);
            self.as_mut().end_insert_rows();
        }
        let item = self.as_mut().get_item(id);
        self.as_mut().item_inserted(&id, &item);
    }

    pub fn get_item(
        self: Pin<&mut Self>,
        index: i32,
    ) -> QMap_QString_QVariant {
        println!("{index}");
        let mut map = QMap_QString_QVariant::default();
        let idx = self.index(index, 0, &QModelIndex::default());
        if !idx.is_valid() {
            return map;
        }
        let rn = self.as_ref().role_names();
        let rn_iter = rn.iter();
        if let Some(service_item) =
            self.service_items.get(index as usize)
        {
            for i in rn_iter {
                map.insert(
                    QString::from(&i.1.to_string()),
                    self.as_ref().data(&idx, *i.0),
                );
            }
        };
        map
    }

    pub fn move_rows(
        mut self: Pin<&mut Self>,
        source_index: i32,
        dest_index: i32,
        count: i32,
    ) -> bool {
        debug!(
            source = source_index,
            dest = dest_index,
            count = count
        );
        let model_index =
            self.index(source_index, 0, &QModelIndex::default());
        let parent = model_index.parent();
        let source_id = source_index as usize;
        let source_first = source_index;
        let source_last = source_index + count - 1;
        let dest_id = dest_index as usize;
        let cnt = count as usize;
        let end_service_item = source_id + cnt - 1;
        // This needs to point to the index above the intended position if moving
        // up. Qt's begin_move_rows requires that knowledge for some reason.
        let qt_dest_index = if source_index < dest_index {
            dest_index + 1
        } else {
            dest_index
        };

        debug!(
            ?model_index,
            ?parent,
            source_id,
            dest_id,
            cnt,
            end_service_item,
            qt_dest_index
        );

        println!("rust-end-service_item: {:?}", end_service_item);
        println!("qt-dest-service_item: {:?}", qt_dest_index);
        unsafe {
            // this function doesn't build
            self.as_mut().begin_move_rows(
                &parent,
                source_first,
                source_last,
                &parent,
                qt_dest_index,
            );

            if source_id < dest_id {
                let move_amount = dest_id - source_id - cnt + 1;
                self.as_mut().rust_mut().service_items
                    [source_id..=dest_id]
                    .rotate_right(move_amount);
                println!("rust-move_amount: {:?}", move_amount);
            } else {
                let move_amount =
                    end_service_item - dest_id - cnt + 1;
                println!("rust-move_amount: {:?}", move_amount);
                self.as_mut().rust_mut().service_items
                    [dest_id..=end_service_item]
                    .rotate_left(move_amount);
            }

            self.as_mut().end_move_rows();
            let item = self.as_mut().get_item(dest_index);
            debug!(source_index, dest_index);
            self.as_mut().item_moved(
                &source_index,
                &dest_index,
                &item,
            );
            true
        }
    }

    pub fn move_up(self: Pin<&mut Self>, index: i32) -> bool {
        self.move_rows(index, index - 1, 1)
    }

    pub fn move_down(self: Pin<&mut Self>, index: i32) -> bool {
        self.move_rows(index, index + 1, 1)
    }

    pub fn select(mut self: Pin<&mut Self>, index: i32) {
        let rc = self.as_ref().count() - 1;
        let tl = &self.as_ref().index(0, 0, &QModelIndex::default());
        let br = &self.as_ref().index(rc, 0, &QModelIndex::default());
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.get_role(ServiceRoles::Selected));
        for service_item in
            self.as_mut().rust_mut().service_items.iter_mut()
        {
            debug!(deselecting = ?service_item);
            service_item.selected = false;
        }
        if let Some(service_item) = self
            .as_mut()
            .rust_mut()
            .service_items
            .get_mut(index as usize)
        {
            debug!(selecting_item = index, item = ?service_item);
            service_item.selected = true;
            self.as_mut().data_changed(tl, br, &vector_roles);
            // We use this signal generated by our signals enum to tell QML that
            // the selected service_item has changed which is used to reposition views.
            // self.as_mut().emit_selected_changed();
        }
    }

    pub fn select_items(
        mut self: Pin<&mut Self>,
        final_index: i32,
    ) -> bool {
        // setup the roles we are using so that we can tell QML
        // which properties to get again.
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.get_role(ServiceRoles::Selected));

        if let Some(current_index) = self
            .as_ref()
            .service_items
            .iter()
            .position(|i| i.selected == true)
        {
            // Here we will need to branch to get the selected items
            debug!(first_item = ?current_index);
            debug!(final_item = final_index);
            // Let's return early to prevent needing to do anything else
            if final_index == current_index as i32 {
                return false;
            }

            let lower = final_index > current_index as i32;
            if lower {
                let top_left = &self.as_ref().index(
                    current_index as i32,
                    0,
                    &QModelIndex::default(),
                );
                let bottom_right = &self.as_ref().index(
                    final_index,
                    0,
                    &QModelIndex::default(),
                );
                for (index, item) in self
                    .as_mut()
                    .rust_mut()
                    .service_items
                    .iter_mut()
                    .enumerate()
                    .filter(|i| {
                        i.0 >= current_index
                            && i.0 <= final_index as usize
                    })
                {
                    item.selected = true;
                    debug!(selected_item = ?item, index = index);
                }
                self.as_mut().data_changed(
                    top_left,
                    bottom_right,
                    &vector_roles,
                );
                // self.as_mut().emit_selected_changed();
            } else {
                let top_left = &self.as_ref().index(
                    final_index,
                    0,
                    &QModelIndex::default(),
                );
                let bottom_right = &self.as_ref().index(
                    current_index as i32,
                    0,
                    &QModelIndex::default(),
                );
                for (index, item) in self
                    .as_mut()
                    .rust_mut()
                    .service_items
                    .iter_mut()
                    .enumerate()
                    .filter(|i| {
                        i.0 >= final_index as usize
                            && i.0 <= current_index
                    })
                {
                    item.selected = true;
                    debug!(selected_item = ?item, index = index);
                }
                self.as_mut().data_changed(
                    top_left,
                    bottom_right,
                    &vector_roles,
                );
            }

            true
        } else {
            // Here let's branch to select from the first item to the
            // final item. Since we don't know which one is selected,
            // assume that the first one is "selected"

            let top_left =
                &self.as_ref().index(0, 0, &QModelIndex::default());
            let bottom_right = &self.as_ref().index(
                final_index,
                0,
                &QModelIndex::default(),
            );
            for (index, item) in self
                .as_mut()
                .rust_mut()
                .service_items
                .iter_mut()
                .enumerate()
                .filter(|i| i.0 <= final_index as usize)
            {
                item.selected = true;
                debug!(selected_item = ?item, index = index);
            }
            self.as_mut().data_changed(
                top_left,
                bottom_right,
                &vector_roles,
            );
            debug!(
                first_item = 0,
                final_item = final_index,
                "couldn't find first selected item"
            );
            false
        }
    }

    pub fn activate(mut self: Pin<&mut Self>, index: i32) -> bool {
        let rc = self.as_ref().count() - 1;
        let tl = &self.as_ref().index(0, 0, &QModelIndex::default());
        let br = &self.as_ref().index(rc, 0, &QModelIndex::default());
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.get_role(ServiceRoles::Active));
        for service_item in
            self.as_mut().rust_mut().service_items.iter_mut()
        {
            // println!("service_item is deactivating {:?}", i);
            service_item.active = false;
        }
        let obs = self.as_mut().obs.clone();

        if let Some(service_item) = self
            .as_mut()
            .rust_mut()
            .service_items
            .get_mut(index as usize)
        {
            debug!(activating_item = index,
                   title = ?service_item.name,
                   background = ?service_item.background,
                   background_type = ?service_item.background_type);
            service_item.active = true;
            // if let Some(obs) = obs {
            //     match obs
            //         .set_scene(service_item.obs_scene.to_string())
            //     {
            //         Ok(()) => debug!("Successfully set scene"),
            //         Err(e) => error!(e),
            //     }
            // }
            self.as_mut().data_changed(tl, br, &vector_roles);
            // We use this signal generated by our signals enum to tell QML that
            // the active service_item has changed which is used to reposition views.
            self.as_mut().active_changed(&index);
            true
        } else {
            false
        }
    }

    pub fn deactivate(mut self: Pin<&mut Self>, index: i32) -> bool {
        let rc = self.as_ref().count() - 1;
        let tl = &self.as_ref().index(0, 0, &QModelIndex::default());
        let br = &self.as_ref().index(rc, 0, &QModelIndex::default());
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.get_role(ServiceRoles::Active));
        if let Some(service_item) = self
            .as_mut()
            .rust_mut()
            .service_items
            .get_mut(index as usize)
        {
            println!("service_item is activating {:?}", index);
            println!("service_item_title: {:?}", service_item.name);
            println!(
                "service_item_background: {:?}",
                service_item.background
            );
            println!(
                "service_item_background_type: {:?}",
                service_item.background_type
            );
            service_item.active = false;
            self.as_mut().data_changed(tl, br, &vector_roles);
            // We use this signal generated by our signals enum to tell QML that
            // the active service_item has changed which is used to reposition views.
            // self.as_mut().emit_active_changed(index);
            true
        } else {
            false
        }
    }

    pub fn save(mut self: Pin<&mut Self>, file: QUrl) -> bool {
        println!("rust-save-file: {file}");
        let save_path =
            file.to_local_file().unwrap_or_default().to_string();
        println!("path: {:?}", save_path);

        let save_file = fs::File::create(&save_path);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let mut handles = vec![];

        if let Ok(save_file) = save_file {
            println!("archive: {:?}", save_file);
            self.as_mut().save_progress_updated(5);
            // let save_file = save_file.clone();
            let encoder = Encoder::new(save_file, 3).unwrap();
            let mut tar = Builder::new(encoder);
            let items = self.rust().service_items.clone();
            let mut temp_dir = dirs::data_dir().unwrap();
            temp_dir.push("lumina");
            let mut s: String =
                iter::repeat_with(fastrand::alphanumeric)
                    .take(5)
                    .collect();
            s.insert_str(0, "temp_");
            temp_dir.push(s);
            match fs::create_dir_all(&temp_dir) {
                Ok(f) => {
                    println!("created_temp_dir: {:?}", &temp_dir)
                }
                Err(e) => println!("temp-dir-error: {e}"),
            }
            let mut temp_service_file = temp_dir.clone();
            temp_service_file.push("serviceitems.json");
            self.as_mut().save_progress_updated(10);
            let mut service_json: Vec<Value> = vec![];
            let progress_fraction = items.len() as f32 / 100 as f32;
            for (id, item) in items.iter().enumerate() {
                let text_list = QList_QString::from(&item.text);
                let mut text_vec = Vec::<String>::default();

                let bg_string_path = item.background.to_string();
                let background_path = PathBuf::from(
                    bg_string_path
                        .to_string()
                        .strip_prefix("file://")
                        .unwrap_or(""),
                );
                println!("bg_path: {:?}", background_path);
                let flat_background_name =
                    background_path.file_name().clone();
                let flat_background;
                match flat_background_name {
                    Some(name) => {
                        println!("bg: {:?}", &name);
                        if name.to_str().unwrap() != "temp" {
                            flat_background = name.to_str().unwrap()
                        } else {
                            flat_background = "";
                        }
                    }
                    _ => {
                        println!("save-background: no background");
                        flat_background = "";
                    }
                }
                let mut temp_bg_path = temp_dir.clone();
                temp_bg_path.push(flat_background);

                let audio_path_str = item.audio.to_string();
                let audio_path = PathBuf::from(
                    audio_path_str
                        .strip_prefix("file://")
                        .unwrap_or(""),
                );
                println!("audio_path: {:?}", audio_path);
                let flat_audio_name = audio_path.file_name();
                let flat_audio;
                match flat_audio_name {
                    Some(name) => {
                        println!("audio: {:?}", &name);
                        if name.to_str().unwrap() != "temp" {
                            flat_audio =
                                name.to_str().unwrap().clone()
                        } else {
                            flat_audio = "";
                        }
                    }
                    _ => {
                        println!("save-audio: no audio");
                        flat_audio = "";
                    }
                }
                let mut temp_aud_path = temp_dir.clone();
                temp_aud_path.push(flat_audio);
                for (index, line) in text_list.iter().enumerate() {
                    text_vec.insert(index, line.to_string())
                }

                let item_json = json!({"name".to_owned(): Value::from(item.name.to_string()),
                                       "type".to_owned(): Value::from(item.ty.to_string()),
                                       "audio".to_owned(): Value::from(item.audio.to_string()),
                                       "background".to_owned(): Value::from(item.background.to_string()),
                                       "backgroundType".to_owned(): Value::from(item.background_type.to_string()),
                                       "font".to_owned(): Value::from(item.font.to_string()),
                                       "fontSize".to_owned(): Value::from(item.font_size),
                                       "flatAudio".to_owned(): Value::from(flat_audio),
                                       "flatBackground".to_owned(): Value::from(flat_background),
                                       "loop".to_owned(): Value::from(item.looping),
                                       "slideNumber".to_owned(): Value::from(item.slide_count),
                                       "text".to_owned(): Value::from(text_vec)});
                println!("item-json: {item_json}");

                let handle = runtime.spawn(async move {
                    match fs::copy(&background_path, &temp_bg_path) {
                        Ok(s) => debug!(
                            "background-copied: of size: {:?}",
                            s
                        ),
                        Err(e) => error!("bg-copy-error: {e}"),
                    }
                });
                handles.push(handle);

                let handle = runtime.spawn(async move {
                    match fs::copy(&audio_path, temp_aud_path) {
                        Ok(s) => {
                            debug!("audio-copied: of size: {:?}", s)
                        }
                        Err(e) => error!("audio-copy-error: {e}"),
                    }
                });
                handles.push(handle);

                service_json.push(item_json);
            }

            for handle in handles {
                match runtime.block_on(handle) {
                    Ok(_) => {}
                    Err(error) => error!(?error, "Error in tokio"),
                }
            }

            println!("{:?}", &temp_service_file);
            match fs::File::create(&temp_service_file) {
                Ok(o) => println!("created: {:?}", o),
                Err(e) => {
                    println!("error-creating-service-file: {:?}", e)
                }
            }
            let now = Instant::now();
            let thread = self.qt_thread();
            match fs::File::options()
                .write(true)
                .read(true)
                .open(&temp_service_file)
            {
                Ok(service_file) => {
                    match serde_json::to_writer(
                        service_file,
                        &service_json,
                    ) {
                        Ok(e) => {
                            debug!(time = ?now.elapsed(), "file written");
                            std::thread::spawn(move || {
                                debug!(time = ?now.elapsed(), "idk");
                                let dir = fs::read_dir(&temp_dir).expect("idk");
                                for (index, file) in dir.enumerate() {
                                    if let Ok(file) = file {
                                        let file_name = file.file_name();
                                        debug!(?file, ?file_name);
                                        let mut file = std::fs::File::open(file.path()).expect("missing file");
                                        tar.append_file(file_name, &mut file).expect("Error in moving file to tar");
                                        thread.queue(move |mut service| {
                                            service
                                                .as_mut()
                                                .set_save_progress(
                                                    progress_fraction *
                                                        (index as f32 + 1.0) * 100.0
                                                )
                                        }).expect("Problem queuing on cxx thread");
                                    }

                                }
                                if let Ok(encoder) = tar.into_inner() {
                                    if let Ok(done) = encoder.finish() {
                                        debug!(time = ?now.elapsed(), ?done, "tar finished");
                                        thread.queue(move |mut service| {
                                            service.as_mut().set_save_progress(100.0);
                                            service.as_mut().saved_to_file(true, &file);
                                        }).expect("Problem queuing on cxx thread");
                                        fs::remove_dir_all(&temp_dir)
                                            .expect(
                                                "error in removal",
                                            );
                                        true
                                    } else {
                                        fs::remove_dir_all(&temp_dir)
                                            .expect(
                                                "error in removal",
                                            );
                                        false
                                    }
                                } else {
                                    fs::remove_dir_all(&temp_dir)
                                        .expect(
                                            "error in removal",
                                        );
                                    false
                                }
                            });
                            true
                        }
                        Err(error) => {
                            error!(?error, "json error");
                            fs::remove_dir_all(&temp_dir)
                                .expect("error in removal");
                            false
                        }
                    }
                }
                Err(error) => {
                    error!(?error, "json service_file isn't open");
                    fs::remove_dir_all(&temp_dir)
                        .expect("error in removal");
                    false
                }
            }
        } else {
            error!(?save_file, "failed to save");
            false
        }
    }

    pub fn load(mut self: Pin<&mut Self>, file: QUrl) -> bool {
        self.as_mut().clear();
        println!("file is: {file}");
        let lfr = fs::File::open(
            file.to_local_file().unwrap_or_default().to_string(),
        );

        let mut datadir = dirs::data_dir().unwrap();
        datadir.push("lumina");
        datadir.push("temp");
        println!("datadir: {:?}", datadir);
        fs::remove_dir_all(&datadir);
        fs::create_dir_all(&datadir);

        if let Ok(lf) = &lfr {
            println!("archive: {:?}", lf);
            let dec = Decoder::new(lf).unwrap();
            let mut tar = Archive::new(dec);
            for mut file in
                tar.entries().unwrap().filter_map(|e| e.ok())
            {
                let mut file_path = datadir.clone();
                file_path.push(file.path().unwrap());
                // Inspect metadata about each file
                println!("filename: {:?}", file.path().unwrap());
                println!("size: {:?}", file.size());
                if !file_path.exists() {
                    file.unpack_in(&datadir);
                }
            }

            // older save files use servicelist.json instead of serviceitems.json
            // Let's check to see if that's the case and change it's name in the
            // temp dir.
            for mut file in
                fs::read_dir(datadir.clone()).unwrap().filter(|f| {
                    f.as_ref()
                        .map(|e| {
                            String::from(
                                e.file_name().to_str().unwrap(),
                            )
                        })
                        .unwrap_or(String::from(""))
                        == "servicelist.json"
                })
            {
                let mut service_path = datadir.clone();
                service_path.push("serviceitems.json");
                match fs::rename(file.unwrap().path(), service_path) {
                    Ok(i) => println!("We did it captain"),
                    Err(e) => println!("error: {:?}", e),
                }
            }

            let mut service_path = datadir.clone();
            service_path.push("serviceitems.json");
            // let mut service_list =
            //     fs::File::open(service_path).unwrap();

            let mut s = fs::read_to_string(service_path).unwrap();
            // service_list.read_to_string(&mut s);
            let ds: Value = serde_json::from_str(&s).unwrap();
            for obj in ds.as_array().unwrap() {
                println!(
                    "objname: {:?}",
                    obj.get("name").unwrap().as_str().unwrap()
                );
                println!(
                    "objtype: {:?}",
                    obj.get("type").unwrap().as_str().unwrap()
                );
                let name = QString::from(
                    obj.get("name").unwrap().as_str().unwrap(),
                );
                let ty = QString::from(
                    obj.get("type").unwrap().as_str().unwrap(),
                );
                // both audio and background will need to know if
                // it exists on disk, if not use the flat version
                let audio_string =
                    obj.get("audio").unwrap().as_str().unwrap();
                let mut audio;
                println!("audio_on_disk: {audio_string}");

                if !Path::new(&audio_string).exists() {
                    println!("#$#$#$#$#$#");
                    println!("The audio doesn't exist");
                    println!("#$#$#$#$#$#");
                    let string = obj
                        .get("flatAudio")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    if !string.is_empty() {
                        println!("before_audio_str: {:?}", string);
                        let mut audio_path = datadir.clone();
                        audio_path.push(string);
                        // Needed to ensure QML images and mpv will find the audio
                        let mut final_string =
                            audio_path.to_str().unwrap().to_owned();
                        final_string.insert_str(0, "file://");
                        audio = QString::from(&final_string);
                        println!(
                            "after_audio_str: {:?}",
                            final_string
                        );
                    } else {
                        audio = QString::default();
                    }
                } else {
                    audio = QString::from(audio_string);
                }

                let bgstr =
                    obj.get("background").unwrap().as_str().unwrap();
                let mut background;
                println!("background_on_disk: {bgstr}");
                let bgpath =
                    bgstr.strip_prefix("file://").unwrap_or("");

                // lets test to see if the background exists on disk.
                // if not we can use the flat version
                if !Path::new(&bgpath).exists() {
                    println!("#$#$#$#$#$#");
                    println!("The background doesn't exist");
                    println!("#$#$#$#$#$#");
                    let string = obj
                        .get("flatBackground")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    if !string.is_empty() {
                        println!("before_bgstr: {:?}", string);
                        let mut bgpath = datadir.clone();
                        bgpath.push(string);
                        // Needed to ensure QML images and mpv will find the background
                        let mut final_string =
                            bgpath.to_str().unwrap().to_owned();
                        final_string.insert_str(0, "file://");
                        background = QString::from(&final_string);
                        println!("after_bgstr: {:?}", final_string);
                    } else {
                        background = QString::default();
                    }
                } else {
                    background = QString::from(bgstr);
                }
                println!("realbg: {:?}", background);

                let background_type = QString::from(
                    obj.get("backgroundType")
                        .unwrap()
                        .as_str()
                        .unwrap(),
                );
                let font = QString::from(
                    obj.get("font").unwrap().as_str().unwrap(),
                );
                let font_size =
                    obj.get("fontSize").unwrap().as_i64().unwrap()
                        as i32;
                let looping;
                if let Some(lp) = obj.get("loop") {
                    looping = lp.as_bool().unwrap();
                } else {
                    looping = false;
                }
                let slide_count;
                if let Some(sc) = obj.get("slideNumber") {
                    slide_count = sc.as_i64().unwrap() as i32;
                } else {
                    slide_count = i32::default();
                }
                let mut video_start_time = f32::default();
                if let Some(video_start_value) =
                    obj.get("video_start_time")
                {
                    video_start_time =
                        video_start_value.as_f64().unwrap() as f32;
                }
                let mut video_end_time = f32::default();
                if let Some(video_end_value) =
                    obj.get("video_end_time")
                {
                    video_end_time =
                        video_end_value.as_f64().unwrap() as f32;
                }
                let text_array =
                    obj.get("text").unwrap().as_array().unwrap();
                let mut text_list = QList_QString::default();
                for txt in text_array {
                    text_list
                        .append(QString::from(txt.as_str().unwrap()));
                }
                let text = QStringList::from(&text_list);

                let service_item = ServiceItem {
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
                println!("Loaded Service: {:?}", ds);
                // // files implement the Read trait
            }
            true
        } else {
            println!("There is no file here: {file}");
            println!("Loading default service");
            false
        }
    }

    pub fn load_last_saved(mut self: Pin<&mut Self>) -> bool {
        todo!();
        // Don't actually need
    }

    fn get_role(&self, role: ServiceRoles) -> i32 {
        match role {
            ServiceRoles::Name => 0,
            ServiceRoles::Type => 1,
            ServiceRoles::Audio => 2,
            ServiceRoles::Background => 3,
            ServiceRoles::BackgroundType => 4,
            ServiceRoles::Text => 5,
            ServiceRoles::Font => 6,
            ServiceRoles::FontSize => 7,
            ServiceRoles::SlideCount => 8,
            ServiceRoles::Active => 9,
            ServiceRoles::Selected => 10,
            ServiceRoles::Looping => 11,
            ServiceRoles::VideoStartTime => 12,
            ServiceRoles::VideoEndTime => 13,
            ServiceRoles::Id => 14,
            _ => 0,
        }
    }
}

// QAbstractListModel implementation
impl service_item_model::ServiceItemModel {
    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = ServiceRoles { repr: role };
        if let Some(service_item) =
            self.service_items.get(index.row() as usize)
        {
            return match role {
                ServiceRoles::Name => {
                    QVariant::from(&service_item.name)
                }
                ServiceRoles::Type => {
                    QVariant::from(&service_item.ty)
                }
                ServiceRoles::Audio => {
                    QVariant::from(&service_item.audio)
                }
                ServiceRoles::Background => {
                    QVariant::from(&service_item.background)
                }
                ServiceRoles::BackgroundType => {
                    QVariant::from(&service_item.background_type)
                }
                ServiceRoles::Text => {
                    QVariant::from(&service_item.text)
                }
                ServiceRoles::Font => {
                    QVariant::from(&service_item.font)
                }
                ServiceRoles::FontSize => {
                    QVariant::from(&service_item.font_size)
                }
                ServiceRoles::SlideCount => {
                    QVariant::from(&service_item.slide_count)
                }
                ServiceRoles::Active => {
                    QVariant::from(&service_item.active)
                }
                ServiceRoles::Selected => {
                    QVariant::from(&service_item.selected)
                }
                ServiceRoles::Looping => {
                    QVariant::from(&service_item.looping)
                }
                ServiceRoles::VideoStartTime => {
                    QVariant::from(&service_item.video_start_time)
                }
                ServiceRoles::VideoEndTime => {
                    QVariant::from(&service_item.video_end_time)
                }
                ServiceRoles::Id => {
                    QVariant::from(&service_item.id)
                }
                _ => QVariant::default(),
            };
        }

        QVariant::default()
    }

    // // Example of overriding a C++ virtual method and calling the base class implementation.
    // pub fn can_fetch_more(&self, parent: &QModelIndex) -> bool {
    //     self.base_can_fetch_more(parent)
    // }

    pub fn role_names(&self) -> QHash_i32_QByteArray {
        let mut roles = QHash_i32_QByteArray::default();
        roles.insert(
            ServiceRoles::Name.repr,
            QByteArray::from("name"),
        );
        roles.insert(ServiceRoles::Type.repr, QByteArray::from("ty"));
        roles.insert(
            ServiceRoles::Audio.repr,
            QByteArray::from("audio"),
        );
        roles.insert(
            ServiceRoles::Background.repr,
            QByteArray::from("background"),
        );
        roles.insert(
            ServiceRoles::BackgroundType.repr,
            QByteArray::from("backgroundType"),
        );
        roles.insert(
            ServiceRoles::Text.repr,
            QByteArray::from("text"),
        );
        roles.insert(
            ServiceRoles::Font.repr,
            QByteArray::from("font"),
        );
        roles.insert(
            ServiceRoles::FontSize.repr,
            QByteArray::from("fontSize"),
        );
        roles.insert(
            ServiceRoles::SlideCount.repr,
            QByteArray::from("slideCount"),
        );
        roles.insert(
            ServiceRoles::Active.repr,
            QByteArray::from("active"),
        );
        roles.insert(
            ServiceRoles::Selected.repr,
            QByteArray::from("selected"),
        );
        roles.insert(
            ServiceRoles::Looping.repr,
            QByteArray::from("looping"),
        );
        roles.insert(
            ServiceRoles::VideoStartTime.repr,
            QByteArray::from("videoStartTime"),
        );
        roles.insert(
            ServiceRoles::VideoEndTime.repr,
            QByteArray::from("videoEndTime"),
        );
        roles.insert(
            ServiceRoles::Id.repr,
            QByteArray::from("id"),
        );
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        let cnt = self.service_items.len() as i32;
        // println!("row count is {cnt}");
        cnt
    }
}

impl ServiceItemModelRust {
    pub fn save(mut model: Pin<&mut ServiceItemModel>, file: QUrl) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_save_file() {
        let mut save_location =
            dirs::runtime_dir().expect("can't find runtime dir");
        save_location.push("test.pres");
        let save_location = QUrl::from(&QString::from(
            save_location.to_str().unwrap(),
        ));
        assert_eq!(save_location, "/run/user/1000/test.pres".into());

        // let mut items = vec![];
        // for i in 0..10 {
        //     let mut item = ServiceItem::default();
        //     item.name = QString::from(&format!("Item #{}", i));
        //     items.push(item);
        // }
        // // let model = super::service_item_model::ServiceItemModel::save(self: Pin<&mut Self>, save_location);
    }
}
