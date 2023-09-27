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

    use serde::{Deserialize, Serialize};
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
        Cleared {},
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
    use serde_json::{json, Deserializer, Map, Serializer, Value};
    use std::ffi::{OsStr, OsString};
    use std::io::{self, Read, Write};
    use std::iter;
    use std::path::{Path, PathBuf};
    use std::str;
    use std::{fs, println};
    use tar::{Archive, Builder};
    use tracing::{debug, debug_span, error, info, instrument};
    use zstd::{Decoder, Encoder};
    impl qobject::ServiceItemMod {
        #[qinvokable]
        pub fn clear(mut self: Pin<&mut Self>) {
            println!("CLEARING ALL ITEMS");
            unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().service_items_mut().clear();
                self.as_mut().end_reset_model();
            }
            self.emit(Signals::Cleared {});
        }

        #[qinvokable]
        pub fn remove_item(mut self: Pin<&mut Self>, index: i32) {
            if index < 0
                || (index as usize) >= self.service_items().len()
            {
                return;
            }

            unsafe {
                self.as_mut().begin_remove_rows(
                    &QModelIndex::default(),
                    index,
                    index,
                );
                self.as_mut()
                    .service_items_mut()
                    .remove(index as usize);
                self.as_mut().end_remove_rows();
            }
            let item = self.as_mut().get_item(index);
            self.as_mut().emit_item_removed(&index, &item);
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

        fn add_service_item(
            mut self: Pin<&mut Self>,
            service_item: &ServiceItm,
        ) {
            let index = self.as_ref().service_items().len() as i32;
            println!("{:?}", service_item);
            let service_item = service_item.clone();
            unsafe {
                self.as_mut().begin_insert_rows(
                    &QModelIndex::default(),
                    index,
                    index,
                );
                self.as_mut().service_items_mut().push(service_item);
                self.as_mut().end_insert_rows();
            }
            let item = self.as_mut().get_item(index);
            self.as_mut().emit_item_added(&index, &item);
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

        fn insert_service_item(
            mut self: Pin<&mut Self>,
            service_item: &ServiceItm,
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
                    .service_items_mut()
                    .insert(id as usize, service_item);
                self.as_mut().end_insert_rows();
            }
            let item = self.as_mut().get_item(id);
            self.as_mut().emit_item_inserted(&id, &item);
        }

        #[qinvokable]
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
                self.rust().service_items.get(index as usize)
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

        #[qinvokable]
        pub fn move_rows(
            mut self: Pin<&mut Self>,
            source_index: i32,
            dest_index: i32,
            count: i32,
        ) -> bool {
            let model_index =
                self.index(source_index, 0, &QModelIndex::default());
            // let parent = model_index.parent();
            let source_id = source_index as usize;
            let dest_id = dest_index as usize;
            let count = count as usize;
            let end_service_item = source_id + count - 1;

            println!("rust-end-service_item: {:?}", end_service_item);
            println!("rust-dest-service_item: {:?}", dest_index);
            unsafe {
                // this function doesn't build
                // self.as_mut().begin_move_rows(
                //     &parent,
                //     source_index,
                //     source_index + count - 1,
                //     &parent,
                //     dest_index,
                // );
                self.as_mut().begin_reset_model();

                if source_id < dest_id {
                    let move_amount = dest_id - source_id - count + 1;
                    self.as_mut().service_items_mut()
                        [source_id..=dest_id]
                        .rotate_right(move_amount);
                    println!("rust-move_amount: {:?}", move_amount);
                } else {
                    let move_amount =
                        end_service_item - dest_id - count + 1;
                    println!("rust-move_amount: {:?}", move_amount);
                    self.as_mut().service_items_mut()
                        [dest_id..=end_service_item]
                        .rotate_left(move_amount);
                }

                // this function works, begin does not
                // self.as_mut().end_move_rows();
                self.as_mut().end_reset_model();
                let item = self.as_mut().get_item(dest_index);
                self.as_mut().emit_item_moved(
                    &source_index,
                    &dest_index,
                    &item,
                );
                true
            }
        }

        #[qinvokable]
        pub fn move_up(self: Pin<&mut Self>, index: i32) -> bool {
            self.move_rows(index, index - 1, 1)
        }

        #[qinvokable]
        pub fn move_down(self: Pin<&mut Self>, index: i32) -> bool {
            self.move_rows(index, index + 1, 1)
        }

        #[qinvokable]
        pub fn select(mut self: Pin<&mut Self>, index: i32) -> bool {
            let rc = self.as_ref().count() - 1;
            let tl =
                &self.as_ref().index(0, 0, &QModelIndex::default());
            let br =
                &self.as_ref().index(rc, 0, &QModelIndex::default());
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(self.get_role(Role::SelectedRole));
            for service_item in
                self.as_mut().service_items_mut().iter_mut()
            {
                // println!("service_item is deactivating {:?}", i);
                service_item.selected = false;
            }
            if let Some(service_item) = self
                .as_mut()
                .service_items_mut()
                .get_mut(index as usize)
            {
                println!("selecting-item: {:?}", index);
                println!(
                    "service_item_title: {:?}",
                    service_item.name
                );
                println!(
                    "service_item_background: {:?}",
                    service_item.background
                );
                println!(
                    "service_item_background_type: {:?}",
                    service_item.background_type
                );
                service_item.selected = true;
                self.as_mut().emit_data_changed(
                    tl,
                    br,
                    &vector_roles,
                );
                // We use this signal generated by our signals enum to tell QML that
                // the selected service_item has changed which is used to reposition views.
                self.as_mut().emit_selected_changed();
                true
            } else {
                false
            }
        }

        #[qinvokable]
        pub fn select_items(
            mut self: Pin<&mut Self>,
            final_index: i32,
        ) -> bool {
            if let Some((current_index, current_item)) = self
                .as_ref()
                .service_items()
                .iter()
                .filter(|i| i.selected)
                .enumerate()
                .next()
            {
                // Here we will need to branch to get the selected items
                debug!(first_item = ?current_index);
                debug!(final_item = final_index);
                if final_index == current_index {
                    false
                }

                let lower = final_index > current_index;
                if lower {
                    let selected = self
                        .as_mut()
                        .service_items_mut()
                        .iter_mut()
                        .enumerate()
                        .filter(|i| {
                            i.0 >= current_index
                                && i.0 <= final_index as usize
                        })
                        .map(|i| i.1.selected = true);
                }

                true
            } else {
                // Here let's branch to select from the first item to the
                // final item. Since we don't know which one is selected,
                // assume that the first one is "selected"
                error!("ERROR: couldn't find first selected item");
                false
            }
        }

        #[qinvokable]
        pub fn activate(
            mut self: Pin<&mut Self>,
            index: i32,
        ) -> bool {
            let rc = self.as_ref().count() - 1;
            let tl =
                &self.as_ref().index(0, 0, &QModelIndex::default());
            let br =
                &self.as_ref().index(rc, 0, &QModelIndex::default());
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(self.get_role(Role::ActiveRole));
            for service_item in
                self.as_mut().service_items_mut().iter_mut()
            {
                // println!("service_item is deactivating {:?}", i);
                service_item.active = false;
            }
            if let Some(service_item) = self
                .as_mut()
                .service_items_mut()
                .get_mut(index as usize)
            {
                println!("service_item is activating {:?}", index);
                println!(
                    "service_item_title: {:?}",
                    service_item.name
                );
                println!(
                    "service_item_background: {:?}",
                    service_item.background
                );
                println!(
                    "service_item_background_type: {:?}",
                    service_item.background_type
                );
                service_item.active = true;
                self.as_mut().emit_data_changed(
                    tl,
                    br,
                    &vector_roles,
                );
                // We use this signal generated by our signals enum to tell QML that
                // the active service_item has changed which is used to reposition views.
                self.as_mut().emit_active_changed();
                true
            } else {
                false
            }
        }

        #[qinvokable]
        pub fn deactivate(
            mut self: Pin<&mut Self>,
            index: i32,
        ) -> bool {
            todo!();
            let rc = self.as_ref().count() - 1;
            let tl =
                &self.as_ref().index(0, 0, &QModelIndex::default());
            let br =
                &self.as_ref().index(rc, 0, &QModelIndex::default());
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(self.get_role(Role::ActiveRole));
            if let Some(service_item) = self
                .as_mut()
                .service_items_mut()
                .get_mut(index as usize)
            {
                println!("service_item is activating {:?}", index);
                println!(
                    "service_item_title: {:?}",
                    service_item.name
                );
                println!(
                    "service_item_background: {:?}",
                    service_item.background
                );
                println!(
                    "service_item_background_type: {:?}",
                    service_item.background_type
                );
                service_item.active = false;
                self.as_mut().emit_data_changed(
                    tl,
                    br,
                    &vector_roles,
                );
                // We use this signal generated by our signals enum to tell QML that
                // the active service_item has changed which is used to reposition views.
                self.as_mut().emit_active_changed();
                true
            } else {
                false
            }
        }
        #[qinvokable]
        pub fn save(mut self: Pin<&mut Self>, file: QUrl) -> bool {
            println!("rust-save-file: {file}");
            let path =
                file.to_local_file().unwrap_or_default().to_string();
            println!("path: {:?}", path);
            let lfr = fs::File::create(&path);
            if let Ok(lf) = &lfr {
                println!("archive: {:?}", lf);
                let encoder = Encoder::new(lf, 3).unwrap();
                let mut tar = Builder::new(encoder);
                let items = self.service_items();
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
                let mut service_json: Vec<Value> = vec![];

                for item in items {
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
                        &background_path.file_name();
                    let flat_background;
                    match flat_background_name {
                        Some(name) => {
                            println!("bg: {:?}", &name);
                            if name.to_str().unwrap() != "temp" {
                                flat_background =
                                    name.to_str().unwrap()
                            } else {
                                flat_background = "";
                            }
                        }
                        _ => {
                            println!(
                                "save-background: no background"
                            );
                            flat_background = "";
                        }
                    }
                    let mut temp_bg_path = temp_dir.clone();
                    temp_bg_path.push(flat_background);
                    match fs::copy(&background_path, &temp_bg_path) {
                        Ok(s) => println!(
                            "background-copied: of size: {:?}",
                            s
                        ),
                        Err(e) => println!("bg-copy-error: {e}"),
                    }

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
                                flat_audio = name.to_str().unwrap()
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
                    match fs::copy(&audio_path, temp_aud_path) {
                        Ok(s) => {
                            println!("audio-copied: of size: {:?}", s)
                        }
                        Err(e) => println!("audio-copy-error: {e}"),
                    }

                    for (index, line) in text_list.iter().enumerate()
                    {
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
                    service_json.push(item_json);
                }
                println!("{:?}", &temp_service_file);
                match fs::File::create(&temp_service_file) {
                    Ok(o) => println!("created: {:?}", o),
                    Err(e) => println!(
                        "error-creating-service-file: {:?}",
                        e
                    ),
                }
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
                                println!("json: file written");
                                match tar
                                    .append_dir_all("./", &temp_dir)
                                {
                                    Ok(i) => {
                                        println!("idk");

                                        match tar.finish() {
                                            Ok(i) => {
                                                println!("tar-written: {:?}", &lf);
                                                fs::remove_dir_all(
                                                    &temp_dir,
                                                );
                                                true
                                            }
                                            Err(e) => {
                                                println!(
                                                    "tar-error: {:?}",
                                                    e
                                                );
                                                fs::remove_dir_all(
                                                    &temp_dir,
                                                );
                                                false
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("err: {:?}", e);
                                        fs::remove_dir_all(&temp_dir);
                                        false
                                    }
                                }
                            }
                            Err(e) => {
                                println!("json: error: {:?}", e);
                                fs::remove_dir_all(&temp_dir);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        println!(
                            "json: service_file isn't open: {:?}",
                            e
                        );
                        fs::remove_dir_all(&temp_dir);
                        false
                    }
                }
            } else {
                println!("rust-save-file-failed: {:?}", lfr);
                false
            }
        }

        #[qinvokable]
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
                for mut file in fs::read_dir(datadir.clone())
                    .unwrap()
                    .filter(|f| {
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
                    match fs::rename(
                        file.unwrap().path(),
                        service_path,
                    ) {
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
                            println!(
                                "before_audio_str: {:?}",
                                string
                            );
                            let mut audio_path = datadir.clone();
                            audio_path.push(string);
                            // Needed to ensure QML images and mpv will find the audio
                            let mut final_string = audio_path
                                .to_str()
                                .unwrap()
                                .to_owned();
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

                    let bgstr = obj
                        .get("background")
                        .unwrap()
                        .as_str()
                        .unwrap();
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
                            println!(
                                "after_bgstr: {:?}",
                                final_string
                            );
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
                    let font_size = obj
                        .get("fontSize")
                        .unwrap()
                        .as_i64()
                        .unwrap()
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
                            video_start_value.as_f64().unwrap()
                                as f32;
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
                        text_list.append(QString::from(
                            txt.as_str().unwrap(),
                        ));
                    }
                    let text = QStringList::from(&text_list);

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

        #[qinvokable]
        pub fn load_last_saved(mut self: Pin<&mut Self>) -> bool {
            todo!();
            // Don't actually need
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
        unsafe fn end_insert_rows(
            self: Pin<&mut qobject::ServiceItemMod>,
        );

        // cxx-qt can't build this function for some reason
        // unsafe fn begin_move_rows(
        //     self: Pin<&mut qobject::ServiceItemMod>,
        //     source_parent: &QModelIndex,
        //     source_first: i32,
        //     source_last: i32,
        //     destination_parent: &QModelIndex,
        //     destination_child: i32,
        // );

        unsafe fn end_move_rows(
            self: Pin<&mut qobject::ServiceItemMod>,
        );

        unsafe fn begin_remove_rows(
            self: Pin<&mut qobject::ServiceItemMod>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_remove_rows(
            self: Pin<&mut qobject::ServiceItemMod>,
        );

        unsafe fn begin_reset_model(
            self: Pin<&mut qobject::ServiceItemMod>,
        );
        unsafe fn end_reset_model(
            self: Pin<&mut qobject::ServiceItemMod>,
        );
    }

    #[cxx_qt::inherit]
    unsafe extern "C++" {
        #[cxx_name = "canFetchMore"]
        fn base_can_fetch_more(
            self: &qobject::ServiceItemMod,
            parent: &QModelIndex,
        ) -> bool;

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
            if let Some(service_item) =
                self.service_items().get(index.row() as usize)
            {
                return match role {
                    0 => QVariant::from(&service_item.name),
                    1 => QVariant::from(&service_item.ty),
                    2 => QVariant::from(&service_item.audio),
                    3 => QVariant::from(&service_item.background),
                    4 => {
                        QVariant::from(&service_item.background_type)
                    }
                    5 => QVariant::from(&service_item.text),
                    6 => QVariant::from(&service_item.font),
                    7 => QVariant::from(&service_item.font_size),
                    8 => QVariant::from(&service_item.slide_count),
                    9 => QVariant::from(&service_item.active),
                    10 => QVariant::from(&service_item.selected),
                    11 => QVariant::from(&service_item.looping),
                    12 => {
                        QVariant::from(&service_item.video_start_time)
                    }
                    13 => {
                        QVariant::from(&service_item.video_end_time)
                    }
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
            roles.insert(
                3,
                cxx_qt_lib::QByteArray::from("background"),
            );
            roles.insert(
                4,
                cxx_qt_lib::QByteArray::from("backgroundType"),
            );
            roles.insert(5, cxx_qt_lib::QByteArray::from("text"));
            roles.insert(6, cxx_qt_lib::QByteArray::from("font"));
            roles.insert(7, cxx_qt_lib::QByteArray::from("fontSize"));
            roles.insert(
                8,
                cxx_qt_lib::QByteArray::from("slideCount"),
            );
            roles.insert(9, cxx_qt_lib::QByteArray::from("active"));
            roles
                .insert(10, cxx_qt_lib::QByteArray::from("selected"));
            roles.insert(11, cxx_qt_lib::QByteArray::from("looping"));
            roles.insert(
                12,
                cxx_qt_lib::QByteArray::from("videoStartTime"),
            );
            roles.insert(
                13,
                cxx_qt_lib::QByteArray::from("videoEndTime"),
            );
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
