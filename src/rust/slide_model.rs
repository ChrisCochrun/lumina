#[cxx_qt::bridge]
mod slide_model {
    unsafe extern "C++" {
        include!(< QAbstractListModel >);
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
        // include!("cxx-qt-lib/qvector.h");
        // type QVector_Slide = cxx_qt_lib::QVector<Slide>;
    }

    #[cxx_qt::qobject]
    #[derive(Default, Clone, Debug)]
    pub struct Slide {
        text: QString,
        ty: QString,
        audio: QString,
        image_background: QString,
        video_background: QString,
        htext_alignment: QString,
        vtext_alignment: QString,
        font: QString,
        font_size: i32,
        image_count: i32,
        slide_id: i32,
        service_item_id: i32,
        active: bool,
        selected: bool,
        looping: bool,
        video_thumbnail: QString,
    }

    #[cxx_qt::qobject(
        base = "QAbstractListModel",
        // qml_uri = "com.kdab.cxx_qt.demo",
        // qml_version = "1.0"
    )]
    #[derive(Default, Debug)]
    pub struct SlideModel {
        id: i32,
        slides: Vec<Slide>,
    }

    #[cxx_qt::qsignals(SlideModel)]
    pub enum Signals<'a> {
        #[inherit]
        DataChanged {
            top_left: &'a QModelIndex,
            bottom_right: &'a QModelIndex,
            roles: &'a QVector_i32,
        },
    }

    impl qobject::SlideModel {
        // #[qinvokable]
        // pub fn add(self: Pin<&mut Self>) {
        //     self.add_cpp_context();
        // }

        // #[qinvokable]
        // pub fn add_on_thread(self: Pin<&mut Self>, mut counter: i32) {
        //     let qt_thread = self.qt_thread();

        //     std::thread::spawn(move || {
        //         while counter > 0 {
        //             counter -= 1;
        //             std::thread::sleep(std::time::Duration::from_millis(250));

        //             // Use our add helper to add a row on the Qt event loop
        //             // as seen in the threading demo channels could be used to pass info
        //             qt_thread
        //                 .queue(|custom_base_class| {
        //                     custom_base_class.add_cpp_context();
        //                 })
        //                 .unwrap();
        //         }
        //     });
        // }

        // fn add_cpp_context(mut self: Pin<&mut Self>) {
        //     let count = self.vector().len();
        //     unsafe {
        //         self.as_mut().begin_insert_rows(
        //             &QModelIndex::default(),
        //             count as i32,
        //             count as i32,
        //         );
        //         let id = *self.id();
        //         self.as_mut().set_id(id + 1);
        //         self.as_mut().vector_mut().push((id, (id as f64) / 3.0));
        //         self.as_mut().end_insert_rows();
        //     }
        // }

        #[qinvokable]
        pub fn clear(mut self: Pin<&mut Self>) {
            unsafe {
                self.as_mut().begin_reset_model();
                self.as_mut().slides_mut().clear();
                self.as_mut().end_reset_model();
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
        }

        #[qinvokable]
        pub fn add_item(
            mut self: Pin<&mut Self>,
            text: QString,
            ty: QString,
            image_background: QString,
            video_background: QString,
            audio: QString,
            font: QString,
            font_size: i32,
            htext_alignment: QString,
            vtext_alignment: QString,
            service_item_id: i32,
            slide_id: i32,
            image_count: i32,
            looping: bool,
        ) {
            let slide = Slide {
                ty,
                text,
                image_background,
                video_background,
                audio,
                font,
                font_size,
                htext_alignment,
                vtext_alignment,
                service_item_id,
                slide_id,
                image_count,
                looping,
                active: false,
                selected: false,
                video_thumbnail: QString::from(""),
            };

            self.as_mut().add_slide(&slide);
        }

        fn add_slide(mut self: Pin<&mut Self>, slide: &Slide) {
            let index = self.as_ref().slides().len() as i32;
            let slide = slide.clone();
            unsafe {
                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), index, index);
                self.as_mut().slides_mut().push(slide);
                self.as_mut().end_remove_rows();
            }
        }

        #[qinvokable]
        pub fn insert_item(
            mut self: Pin<&mut Self>,
            index: i32,
            text: QString,
            ty: QString,
            image_background: QString,
            video_background: QString,
            audio: QString,
            font: QString,
            font_size: i32,
            htext_alignment: QString,
            vtext_alignment: QString,
            service_item_id: i32,
            slide_id: i32,
            image_count: i32,
            looping: bool,
        ) {
            let slide = Slide {
                ty,
                text,
                image_background,
                video_background,
                audio,
                font,
                font_size,
                htext_alignment,
                vtext_alignment,
                service_item_id,
                slide_id,
                image_count,
                looping,
                active: false,
                selected: false,
                video_thumbnail: QString::from(""),
            };

            self.as_mut().insert_slide(&slide, index);
        }

        fn insert_slide(mut self: Pin<&mut Self>, slide: &Slide, id: i32) {
            let slide = slide.clone();
            unsafe {
                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), id, id);
                self.as_mut().slides_mut().insert(id as usize, slide);
                self.as_mut().end_remove_rows();
            }
        }
    }

    // Create Rust bindings for C++ functions of the base class (QAbstractItemModel)
    #[cxx_qt::inherit]
    extern "C++" {
        unsafe fn begin_insert_rows(
            self: Pin<&mut qobject::SlideModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_insert_rows(self: Pin<&mut qobject::SlideModel>);

        unsafe fn begin_remove_rows(
            self: Pin<&mut qobject::SlideModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        unsafe fn end_remove_rows(self: Pin<&mut qobject::SlideModel>);

        unsafe fn begin_reset_model(self: Pin<&mut qobject::SlideModel>);
        unsafe fn end_reset_model(self: Pin<&mut qobject::SlideModel>);
    }

    #[cxx_qt::inherit]
    unsafe extern "C++" {
        #[cxx_name = "canFetchMore"]
        fn base_can_fetch_more(self: &qobject::SlideModel, parent: &QModelIndex) -> bool;

        fn index(
            self: &qobject::SlideModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    // QAbstractListModel implementation
    impl qobject::SlideModel {
        #[qinvokable(cxx_override)]
        fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
            if let Some(slide) = self.rust().slides.get(index.row() as usize) {
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
                    10 => QVariant::from(&slide.slide_id),
                    11 => QVariant::from(&slide.image_count),
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
            roles.insert(10, cxx_qt_lib::QByteArray::from("slideId"));
            roles.insert(11, cxx_qt_lib::QByteArray::from("imageCount"));
            roles.insert(12, cxx_qt_lib::QByteArray::from("active"));
            roles.insert(13, cxx_qt_lib::QByteArray::from("selected"));
            roles.insert(14, cxx_qt_lib::QByteArray::from("looping"));
            roles.insert(15, cxx_qt_lib::QByteArray::from("videoThumbnail"));
            roles
        }

        #[qinvokable(cxx_override)]
        pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
            self.rust().slides.len() as i32
        }
    }
}
