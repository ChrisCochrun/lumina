#[cxx_qt::bridge]
mod slide_model {
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
        // #[cxx_name = "Slidey"]
        // type CxxSlidey = super::qobject::Slidey;
        // include!("cxx-qt-lib/qvector.h");
        // type QVector_Slidey = cxx_qt_lib::QVector<Slidey>;
    }

    #[cxx_qt::qobject]
    #[derive(Clone, Debug)]
    pub struct Slidey {
        #[qproperty]
        text: QString,
        #[qproperty]
        ty: QString,
        #[qproperty]
        audio: QString,
        #[qproperty]
        image_background: QString,
        #[qproperty]
        video_background: QString,
        #[qproperty]
        htext_alignment: QString,
        #[qproperty]
        vtext_alignment: QString,
        #[qproperty]
        font: QString,
        #[qproperty]
        font_size: i32,
        #[qproperty]
        slide_count: i32,
        #[qproperty]
        slide_index: i32,
        #[qproperty]
        service_item_id: i32,
        #[qproperty]
        active: bool,
        #[qproperty]
        selected: bool,
        #[qproperty]
        looping: bool,
        #[qproperty]
        video_thumbnail: QString,
    }

    impl Default for Slidey {
        fn default() -> Self {
            Self {
                text: QString::default(),
                ty: QString::default(),
                audio: QString::default(),
                image_background: QString::default(),
                video_background: QString::default(),
                htext_alignment: QString::default(),
                vtext_alignment: QString::default(),
                font: QString::default(),
                font_size: 50,
                slide_count: 1,
                slide_index: 0,
                service_item_id: 0,
                active: false,
                selected: false,
                looping: false,
                video_thumbnail: QString::default(),
            }
        }
    }

    #[cxx_qt::qobject(
        base = "QAbstractListModel",
        // qml_uri = "com.kdab.cxx_qt.demo",
        // qml_version = "1.0"
    )]
    #[derive(Default, Debug)]
    pub struct SlideyMod {
        id: i32,
        slides: Vec<Slidey>,
    }

    #[cxx_qt::qsignals(SlideyMod)]
    pub enum Signals<'a> {
        #[inherit]
        DataChanged {
            top_left: &'a QModelIndex,
            bottom_right: &'a QModelIndex,
            roles: &'a QVector_i32,
        },
        ActiveChanged,
    }

    enum Role {
        ActiveRole,
        SelectedRole,
        LoopingRole,
        TextRole,
        VideoThumbnailRole,
    }

    // use crate::video_thumbnail;
    // use image::{ImageBuffer, Rgba};
    use std::path::PathBuf;
    impl qobject::SlideyMod {
        #[qinvokable]
        pub fn video_thumbnail(
            mut self: Pin<&mut Self>,
            video: QString,
            service_item_id: i32,
            index: i32,
        ) -> QString {
            let video = video.to_string();
            let mut path = PathBuf::from(video);
            println!("{:?}", path);
            // let mut image_iter = video_thumbnail::ImageIter::new(path)?;
            // image_iter.seek(2.0);

            // if let Some(image) = image_iter.next() {
            //     image.save("image.jpg");
            // }

            QString::default()
        }

        #[qinvokable]
        pub fn add_video_thumbnail(
            mut self: Pin<&mut Self>,
            video: QString,
            service_item_id: i32,
            index: i32,
        ) {
            let model_index = &self.as_ref().index(index, 0, &QModelIndex::default());
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(self.get_role(Role::VideoThumbnailRole));
            if let Some(slide) = self.as_mut().slides_mut().get_mut(index as usize) {
                slide.video_thumbnail = video;
            }
            self.as_mut()
                .emit_data_changed(model_index, model_index, &vector_roles);
            println!("AHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH");
        }

        #[qinvokable]
        pub fn clear(mut self: Pin<&mut Self>) {
            unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().slides_mut().clear();
                self.as_mut().end_reset_model();
            }
        }

        #[qinvokable]
        pub fn remove_item_from_service(
            mut self: Pin<&mut Self>,
            index: i32,
            _service_item: &QMap_QString_QVariant,
        ) {
            println!("Rusty-Removal-Time: {:?}", index);
            let slides = self.slides().clone();
            let slides_iter = slides.iter();
            for (i, slide) in slides_iter.enumerate().rev() {
                if slide.service_item_id == index {
                    self.as_mut().remove_item(i as i32);
                    println!("Removing-slide: {:?}", i);
                } else if slide.service_item_id > index {
                    if let Some(slide) = self.as_mut().slides_mut().get_mut(i) {
                        println!("changing-serviceid-of: {:?}", i);
                        println!(
                            "changing-serviceid-fromandto: {:?}-{:?}",
                            slide.service_item_id,
                            slide.service_item_id - 1
                        );
                        slide.service_item_id -= 1;
                    }
                }
            }
        }

        #[qinvokable]
        pub fn remove_item(mut self: Pin<&mut Self>, index: i32) {
            if index < 0 || (index as usize) >= self.slides().len() {
                return;
            }

            unsafe {
                self.as_mut()
                    .begin_remove_rows(&QModelIndex::default(), index, index);
                self.as_mut().slides_mut().remove(index as usize);
                self.as_mut().end_remove_rows();
            }
            println!("removed-row: {:?}", index);
        }

        fn add_slide(mut self: Pin<&mut Self>, slide: &Slidey) {
            let index = self.as_ref().slides().len() as i32;
            println!("{:?}", slide);
            let slide = slide.clone();
            unsafe {
                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), index, index);
                self.as_mut().slides_mut().push(slide);
                self.as_mut().end_insert_rows();
            }
        }

        fn insert_slide(mut self: Pin<&mut Self>, slide: &Slidey, id: i32) {
            let mut slide = slide.clone();
            slide.slide_index = id;

            unsafe {
                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), id, id);
                self.as_mut().slides_mut().insert(id as usize, slide);
                self.as_mut().end_insert_rows();
            }
        }

        #[qinvokable]
        pub fn insert_item_from_service(
            mut self: Pin<&mut Self>,
            index: i32,
            service_item: &QMap_QString_QVariant,
        ) {
            let ty = service_item
                .get(&QString::from("type"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QString>();

            let background = service_item
                .get(&QString::from("background"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QString>()
                .unwrap_or_default();

            let background_type = service_item
                .get(&QString::from("backgroundType"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QString>()
                .unwrap_or_default();

            let textlist = service_item
                .get(&QString::from("text"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QStringList>()
                .unwrap_or_default();

            let text_vec = Vec::<QString>::from(&QList_QString::from(&textlist));
            // let vec_slize: &[usize] = &text_vec;

            let mut slide = Slidey {
                ty: service_item
                    .get(&QString::from("type"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                text: service_item
                    .get(&QString::from("text"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                image_background: service_item
                    .get(&QString::from("imageBackground"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                video_background: service_item
                    .get(&QString::from("videoBackground"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                audio: service_item
                    .get(&QString::from("audio"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                font: service_item
                    .get(&QString::from("font"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                font_size: service_item
                    .get(&QString::from("fontSize"))
                    .unwrap_or(QVariant::from(&50))
                    .value()
                    .unwrap_or(50),
                htext_alignment: service_item
                    .get(&QString::from("vtextAlignment"))
                    .unwrap_or(QVariant::from(&QString::from("center")))
                    .value()
                    .unwrap_or(QString::from("center")),
                vtext_alignment: service_item
                    .get(&QString::from("vtextAlignment"))
                    .unwrap_or(QVariant::from(&QString::from("center")))
                    .value()
                    .unwrap_or(QString::from("center")),
                service_item_id: index,
                slide_index: service_item
                    .get(&QString::from("slideNumber"))
                    .unwrap_or(QVariant::from(&0))
                    .value()
                    .unwrap_or(0),
                slide_count: service_item
                    .get(&QString::from("slideNumber"))
                    .unwrap_or(QVariant::from(&1))
                    .value()
                    .unwrap_or(1),
                looping: service_item
                    .get(&QString::from("loop"))
                    .unwrap_or(QVariant::from(&false))
                    .value()
                    .unwrap_or(false),
                active: service_item
                    .get(&QString::from("active"))
                    .unwrap_or(QVariant::from(&false))
                    .value()
                    .unwrap_or(false),
                selected: service_item
                    .get(&QString::from("selected"))
                    .unwrap_or(QVariant::from(&false))
                    .value()
                    .unwrap_or(false),
                video_thumbnail: QString::from(""),
            };

            // We need to move all the current slides service_item_id's up by one.
            let slides_len = self.as_mut().slides_mut().len() as i32;
            for slide in index..slides_len {
                if let Some(slide) = self.as_mut().slides_mut().get_mut(slide as usize) {
                    slide.service_item_id += 1;
                }
            }

            match ty {
                Some(ty) if ty == QString::from("image") => {
                    slide.ty = ty;
                    slide.image_background = background;
                    slide.video_background = QString::from("");
                    slide.slide_index = 0;
                    self.as_mut().insert_slide(&slide, index);
                }
                Some(ty) if ty == QString::from("song") => {
                    for i in 0..text_vec.len() {
                        println!("add song of {:?} length", text_vec.len());
                        slide.ty = ty.clone();
                        // println!("{:?}", text_vec[i].clone());
                        slide.text = text_vec[i].clone();
                        slide.slide_count = text_vec.len() as i32;
                        slide.slide_index = i as i32;
                        if background_type == QString::from("image") {
                            slide.image_background = background.clone();
                            slide.video_background = QString::from("");
                        } else {
                            slide.video_background = background.clone();
                            slide.image_background = QString::from("");
                        }
                        self.as_mut().insert_slide(&slide, index + i as i32);
                    }
                }
                Some(ty) if ty == QString::from("video") => {
                    slide.ty = ty;
                    slide.image_background = QString::from("");
                    slide.video_background = background;
                    slide.slide_index = 0;
                    self.as_mut().insert_slide(&slide, index);
                }
                Some(ty) if ty == QString::from("presentation") => {
                    for i in 0..slide.slide_count {
                        slide.ty = ty.clone();
                        slide.image_background = background.clone();
                        slide.video_background = QString::from("");
                        slide.slide_index = i;
                        self.as_mut().insert_slide(&slide, index + i as i32);
                    }
                }
                _ => println!("It's somethign else!"),
            };

            println!("Item added in rust model!");
        }

        #[qinvokable]
        pub fn add_item_from_service(
            mut self: Pin<&mut Self>,
            index: i32,
            service_item: &QMap_QString_QVariant,
        ) {
            println!("add rust slide {:?}", index);
            let ty = service_item
                .get(&QString::from("type"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QString>();

            let background = service_item
                .get(&QString::from("background"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QString>()
                .unwrap_or_default();

            let background_type = service_item
                .get(&QString::from("backgroundType"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QString>()
                .unwrap_or_default();

            let textlist = service_item
                .get(&QString::from("text"))
                .unwrap_or(QVariant::from(&QString::from("")))
                .value::<QStringList>()
                .unwrap_or_default();

            let text_vec = Vec::<QString>::from(&QList_QString::from(&textlist));
            // let vec_slize: &[usize] = &text_vec;

            let mut slide = Slidey {
                ty: service_item
                    .get(&QString::from("type"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                text: service_item
                    .get(&QString::from("text"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                image_background: service_item
                    .get(&QString::from("imageBackground"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                video_background: service_item
                    .get(&QString::from("videoBackground"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                audio: service_item
                    .get(&QString::from("audio"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                font: service_item
                    .get(&QString::from("font"))
                    .unwrap_or(QVariant::from(&QString::from("")))
                    .value()
                    .unwrap_or(QString::from("")),
                font_size: service_item
                    .get(&QString::from("fontSize"))
                    .unwrap_or(QVariant::from(&50))
                    .value()
                    .unwrap_or(50),
                htext_alignment: service_item
                    .get(&QString::from("vtextAlignment"))
                    .unwrap_or(QVariant::from(&QString::from("center")))
                    .value()
                    .unwrap_or(QString::from("center")),
                vtext_alignment: service_item
                    .get(&QString::from("vtextAlignment"))
                    .unwrap_or(QVariant::from(&QString::from("center")))
                    .value()
                    .unwrap_or(QString::from("center")),
                service_item_id: index,
                slide_index: service_item
                    .get(&QString::from("slideNumber"))
                    .unwrap_or(QVariant::from(&0))
                    .value()
                    .unwrap_or(0),
                slide_count: service_item
                    .get(&QString::from("imageCount"))
                    .unwrap_or(QVariant::from(&1))
                    .value()
                    .unwrap_or(1),
                looping: service_item
                    .get(&QString::from("loop"))
                    .unwrap_or(QVariant::from(&false))
                    .value()
                    .unwrap_or(false),
                active: service_item
                    .get(&QString::from("active"))
                    .unwrap_or(QVariant::from(&false))
                    .value()
                    .unwrap_or(false),
                selected: service_item
                    .get(&QString::from("selected"))
                    .unwrap_or(QVariant::from(&false))
                    .value()
                    .unwrap_or(false),
                video_thumbnail: QString::from(""),
            };

            match ty {
                Some(ty) if ty == QString::from("image") => {
                    slide.ty = ty;
                    slide.image_background = background;
                    slide.video_background = QString::from("");
                    slide.slide_index = 0;
                    self.as_mut().add_slide(&slide);
                }
                Some(ty) if ty == QString::from("song") => {
                    for (i, text) in text_vec.iter().enumerate() {
                        slide.ty = ty.clone();
                        // println!("{:?}", text_vec[i].clone());
                        slide.text = text.clone();
                        slide.slide_count = text_vec.len() as i32;
                        slide.slide_index = i as i32;
                        if background_type == QString::from("image") {
                            slide.image_background = background.clone();
                            slide.video_background = QString::from("");
                        } else {
                            slide.video_background = background.clone();
                            slide.image_background = QString::from("");
                        }
                        self.as_mut().add_slide(&slide);
                    }
                }
                Some(ty) if ty == QString::from("video") => {
                    slide.ty = ty;
                    slide.image_background = QString::from("");
                    slide.video_background = background;
                    slide.slide_index = 0;
                    self.as_mut().add_slide(&slide);
                }
                Some(ty) if ty == QString::from("presentation") => {
                    for i in 0..slide.slide_count {
                        slide.ty = ty.clone();
                        slide.image_background = background.clone();
                        slide.video_background = QString::from("");
                        slide.slide_index = i;
                        self.as_mut().add_slide(&slide);
                    }
                }
                _ => println!("It's somethign else!"),
            };

            println!("Item added in rust model!");
        }

        #[qinvokable]
        pub fn move_item_from_service(
            mut self: Pin<&mut Self>,
            source_index: i32,
            destination_index: i32,
            _service_item: &QMap_QString_QVariant,
        ) {
            if source_index == destination_index {
                return;
            }

            let move_down = source_index < destination_index;
            let slides = self.slides().clone();
            let slides_iter = slides.iter();

            let mut first_slide = 0;
            let mut dest_slide = 0;
            let mut count = 0;

            if move_down {
                for (i, slide) in slides_iter.clone().enumerate() {
                    if slide.service_item_id == source_index {
                        first_slide = i as i32;
                        count = slide.slide_count;
                        println!("RUST_COUNT: {:?}", count);
                        break;
                    }
                }

                for (i, slide) in slides_iter.enumerate().rev() {
                    if slide.service_item_id == destination_index {
                        dest_slide = i as i32;
                        break;
                    }
                }
            } else {
                for (i, slide) in slides_iter.clone().enumerate() {
                    if slide.service_item_id == source_index {
                        first_slide = i as i32;
                        count = slide.slide_count;
                        println!("RUST_COUNT: {:?}", count);
                        break;
                    }
                }

                for (i, slide) in slides_iter.enumerate() {
                    if slide.service_item_id == destination_index {
                        dest_slide = i as i32;
                        break;
                    }
                }
            }

            unsafe {
                self.as_mut().begin_reset_model();
            }
            let slides = self.slides().clone();
            let slides_iter = slides.iter();

            self.as_mut().move_items(first_slide, dest_slide, count);

            if count > 1 {
                for (i, slide) in slides_iter
                    .clone()
                    .enumerate()
                    .filter(|x| x.0 >= dest_slide as usize)
                    .filter(|x| x.0 < (dest_slide + count) as usize)
                {
                    if let Some(slide) = self.as_mut().slides_mut().get_mut(i) {
                        slide.service_item_id = destination_index;
                    }
                    // println!("this one right here officer.");
                }
            } else {
                if let Some(slide) = self.as_mut().slides_mut().get_mut(dest_slide as usize) {
                    slide.service_item_id = destination_index;
                }
            }

            if move_down {
                for (i, slide) in slides_iter
                    .enumerate()
                    .filter(|x| x.0 < dest_slide as usize)
                    .filter(|x| x.1.service_item_id <= destination_index)
                    .filter(|x| x.1.service_item_id > source_index)
                {
                    if let Some(slide) = self.as_mut().slides_mut().get_mut(i) {
                        println!(
                            "rust-switching-service: {:?} to {:?}",
                            slide.service_item_id,
                            slide.service_item_id - 1
                        );
                        slide.service_item_id -= 1;
                    }
                    println!("rust-did:");
                }
            } else {
                for (i, slide) in slides_iter
                    .enumerate()
                    .filter(|x| x.0 >= (dest_slide as usize + count as usize))
                    .filter(|x| x.1.service_item_id >= destination_index)
                    .filter(|x| x.1.service_item_id <= source_index)
                {
                    if let Some(slide) = self.as_mut().slides_mut().get_mut(i) {
                        println!(
                            "rust-switching-service: {:?} to {:?}",
                            slide.service_item_id,
                            slide.service_item_id + 1
                        );
                        slide.service_item_id += 1;
                    }
                    println!("rust-did:");
                }
            }

            unsafe {
                self.as_mut().end_reset_model();
            }

            // for (i, slide) in slides_iter
            //     .enumerate()
            //     .filter(|x| x.0 < dest_slide as usize)
            // {
            //     if move_down {
            //         if slide.service_item_id <= destination_index
            //             && slide.service_item_id > source_index
            //         {
            //             if let Some(slide) = self.as_mut().slides_mut().get_mut(i) {
            //                 println!(
            //                     "rust-switching-service: {:?} to {:?}",
            //                     slide.service_item_id,
            //                     slide.service_item_id - 1
            //                 );
            //                 slide.service_item_id -= 1;
            //             }
            //             println!("rust-did:");
            //         }
            //         println!("rust-not-service_item_id: {:?}", slide.service_item_id);
            //     } else {
            //         if slide.service_item_id > destination_index
            //             && slide.service_item_id < source_index
            //         {
            //             if let Some(slide) = self.as_mut().slides_mut().get_mut(i) {
            //                 println!(
            //                     "rust-switching-service: {:?} to {:?}",
            //                     slide.service_item_id,
            //                     slide.service_item_id + 1
            //                 );
            //                 slide.service_item_id += 1;
            //             }
            //             println!("rust-did:");
            //         }
            //         println!("rust-not-service_item_id: {:?}", slide.service_item_id);
            //     }
            // }

            println!("rust-move: {first_slide} to {dest_slide} with {count} slides");
        }

        fn move_items(mut self: Pin<&mut Self>, source_index: i32, dest_index: i32, count: i32) {
            let end_slide = source_index + count;
            unsafe {
                self.as_mut().begin_reset_model();
                let drained: Vec<Slidey> = self
                    .as_mut()
                    .slides_mut()
                    .drain(source_index as usize..end_slide as usize)
                    .collect();
                for (i, slide) in drained.iter().enumerate() {
                    self.as_mut()
                        .slides_mut()
                        .insert(dest_index as usize + i, slide.clone());
                }
                self.as_mut().end_reset_model();
            }
        }

        #[qinvokable]
        pub fn get_item(self: Pin<&mut Self>, index: i32) -> QMap_QString_QVariant {
            println!("{index}");
            let mut qvariantmap = QMap_QString_QVariant::default();
            let idx = self.index(index, 0, &QModelIndex::default());
            if !idx.is_valid() {
                return qvariantmap;
            }
            let rn = self.as_ref().role_names();
            let rn_iter = rn.iter();
            if let Some(slide) = self.rust().slides.get(index as usize) {
                for i in rn_iter {
                    qvariantmap.insert(
                        QString::from(&i.1.to_string()),
                        self.as_ref().data(&idx, *i.0),
                    );
                }
            };
            qvariantmap
        }

        #[qinvokable]
        pub fn activate(mut self: Pin<&mut Self>, index: i32) -> bool {
            let rc = self.as_ref().count() - 1;
            let tl = &self.as_ref().index(0, 0, &QModelIndex::default());
            let br = &self.as_ref().index(rc, 0, &QModelIndex::default());
            let mut vector_roles = QVector_i32::default();
            vector_roles.append(self.get_role(Role::ActiveRole));
            for slide in self.as_mut().slides_mut().iter_mut() {
                // println!("slide is deactivating {:?}", i);
                slide.active = false;
            }
            if let Some(slide) = self.as_mut().slides_mut().get_mut(index as usize) {
                println!("slide is activating {:?}", index);
                println!("slide-title: {:?}", slide.service_item_id);
                println!("slide-image-background: {:?}", slide.image_background);
                println!("slide-video-background: {:?}", slide.video_background);
                slide.active = true;
                self.as_mut().emit_data_changed(tl, br, &vector_roles);
                // We use this signal generated by our signals enum to tell QML that
                // the active slide has changed which is used to reposition views.
                self.as_mut().emit_active_changed();
                true
            } else {
                false
            }
        }

        fn get_role(&self, role: Role) -> i32 {
            match role {
                Role::TextRole => 1,
                Role::ActiveRole => 12,
                Role::SelectedRole => 13,
                Role::LoopingRole => 14,
                Role::VideoThumbnailRole => 15,
                _ => 0,
            }
        }
    }

    // Create Rust bindings for C++ functions of the base class (QAbstractItemModel)
    #[cxx_qt::inherit]
    extern "C++" {
        unsafe fn begin_insert_rows(
            self: Pin<&mut qobject::SlideyMod>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_insert_rows(self: Pin<&mut qobject::SlideyMod>);

        unsafe fn begin_remove_rows(
            self: Pin<&mut qobject::SlideyMod>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_remove_rows(self: Pin<&mut qobject::SlideyMod>);

        unsafe fn begin_reset_model(self: Pin<&mut qobject::SlideyMod>);
        unsafe fn end_reset_model(self: Pin<&mut qobject::SlideyMod>);
    }

    #[cxx_qt::inherit]
    unsafe extern "C++" {
        #[cxx_name = "canFetchMore"]
        fn base_can_fetch_more(self: &qobject::SlideyMod, parent: &QModelIndex) -> bool;

        fn index(
            self: &qobject::SlideyMod,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    // QAbstractListModel implementation
    impl qobject::SlideyMod {
        #[qinvokable(cxx_override)]
        fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
            if let Some(slide) = self.slides().get(index.row() as usize) {
                return match role {
                    0 => QVariant::from(&slide.ty),
                    1 => QVariant::from(&slide.text),
                    2 => QVariant::from(&slide.audio),
                    3 => QVariant::from(&slide.image_background),
                    4 => QVariant::from(&slide.video_background),
                    5 => QVariant::from(&slide.htext_alignment),
                    6 => QVariant::from(&slide.vtext_alignment),
                    7 => QVariant::from(&slide.font),
                    8 => QVariant::from(&slide.font_size),
                    9 => QVariant::from(&slide.service_item_id),
                    10 => QVariant::from(&slide.slide_index),
                    11 => QVariant::from(&slide.slide_count),
                    12 => QVariant::from(&slide.active),
                    13 => QVariant::from(&slide.selected),
                    14 => QVariant::from(&slide.looping),
                    15 => QVariant::from(&slide.video_thumbnail),
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
            roles.insert(0, cxx_qt_lib::QByteArray::from("type"));
            roles.insert(1, cxx_qt_lib::QByteArray::from("text"));
            roles.insert(2, cxx_qt_lib::QByteArray::from("audio"));
            roles.insert(3, cxx_qt_lib::QByteArray::from("imageBackground"));
            roles.insert(4, cxx_qt_lib::QByteArray::from("videoBackground"));
            roles.insert(5, cxx_qt_lib::QByteArray::from("hTextAlignment"));
            roles.insert(6, cxx_qt_lib::QByteArray::from("vTextAlignment"));
            roles.insert(7, cxx_qt_lib::QByteArray::from("font"));
            roles.insert(8, cxx_qt_lib::QByteArray::from("fontSize"));
            roles.insert(9, cxx_qt_lib::QByteArray::from("serviceItemId"));
            roles.insert(10, cxx_qt_lib::QByteArray::from("slideIndex"));
            roles.insert(11, cxx_qt_lib::QByteArray::from("imageCount"));
            roles.insert(12, cxx_qt_lib::QByteArray::from("active"));
            roles.insert(13, cxx_qt_lib::QByteArray::from("selected"));
            roles.insert(14, cxx_qt_lib::QByteArray::from("looping"));
            roles.insert(15, cxx_qt_lib::QByteArray::from("videoThumbnail"));
            roles
        }

        #[qinvokable(cxx_override)]
        pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
            let cnt = self.rust().slides.len() as i32;
            // println!("row count is {cnt}");
            cnt
        }

        #[qinvokable]
        pub fn count(&self) -> i32 {
            self.rust().slides.len() as i32
        }
    }
}
