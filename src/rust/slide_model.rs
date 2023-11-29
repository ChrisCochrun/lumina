#[cxx_qt::bridge]
mod slide_model {
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
        // #[cxx_name = "Slidey"]
        // type CxxSlidey = super::qobject::Slidey;
        // include!("cxx-qt-lib/qvector.h");
        // type QVector_Slidey = cxx_qt_lib::QVector<Slidey>;
    }

    #[qenum(SlideModel)]
    enum SlideRoles {
        Ty,
        Text,
        Audio,
        ImageBackground,
        VideoBackground,
        HTextAlignment,
        VTextAlignment,
        Font,
        FontSize,
        ServiceItemId,
        SlideIndex,
        SlideCount,
        Active,
        Selected,
        Looping,
        VideoThumbnail,
        VideoStartTime,
        VideoEndTime,
        Html,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[base = "QAbstractListModel"]
        #[qml_element]
        #[qproperty(i32, count)]
        type SlideModel = super::SlideModelRust;

        #[inherit]
        #[qsignal]
        fn data_changed(
            self: Pin<&mut SlideModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );

        #[qsignal]
        fn active_change(self: Pin<&mut SlideModel>, index: &i32);

        #[qinvokable]
        fn add_video_thumbnail(
            self: Pin<&mut SlideModel>,
            index: i32,
        ) -> bool;

        #[qinvokable]
        fn clear(self: Pin<&mut SlideModel>);

        #[qinvokable]
        fn remove_item_from_service(
            self: Pin<&mut SlideModel>,
            index: i32,
            _service_item: &QMap_QString_QVariant,
        );

        #[qinvokable]
        fn remove_item(self: Pin<&mut SlideModel>, index: i32);

        #[qinvokable]
        fn insert_item_from_service(
            self: Pin<&mut SlideModel>,
            index: i32,
            service_item: &QMap_QString_QVariant,
        );

        #[qinvokable]
        fn add_item_from_service(
            self: Pin<&mut SlideModel>,
            index: i32,
            service_item: &QMap_QString_QVariant,
        );

        #[qinvokable]
        fn move_item_from_service(
            self: Pin<&mut SlideModel>,
            source_index: i32,
            destination_index: i32,
            _service_item: &QMap_QString_QVariant,
        );

        #[qinvokable]
        fn get_item(
            self: Pin<&mut SlideModel>,
            index: i32,
        ) -> QMap_QString_QVariant;

        #[qinvokable]
        fn get_slide_from_service(
            self: Pin<&mut SlideModel>,
            index: i32,
        ) -> i32;

        #[qinvokable]
        fn activate(self: Pin<&mut SlideModel>, index: i32) -> bool;
    }

    impl cxx_qt::Threading for SlideModel {}

    unsafe extern "RustQt" {
        #[inherit]
        unsafe fn begin_insert_rows(
            self: Pin<&mut SlideModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_insert_rows(self: Pin<&mut SlideModel>);

        #[inherit]
        unsafe fn begin_remove_rows(
            self: Pin<&mut SlideModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_remove_rows(self: Pin<&mut SlideModel>);

        #[inherit]
        unsafe fn begin_reset_model(self: Pin<&mut SlideModel>);

        #[inherit]
        unsafe fn end_reset_model(self: Pin<&mut SlideModel>);

        #[inherit]
        fn can_fetch_more(
            self: &SlideModel,
            parent: &QModelIndex,
        ) -> bool;

        #[inherit]
        fn index(
            self: &SlideModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[qinvokable]
        #[cxx_override]
        fn data(
            self: &SlideModel,
            index: &QModelIndex,
            role: i32,
        ) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        fn role_names(self: &SlideModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn row_count(self: &SlideModel, _parent: &QModelIndex)
            -> i32;

    }
}

use crate::ffmpeg;
use crate::slide_model::slide_model::QList_QString;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{
    CaseSensitivity, QByteArray, QModelIndex, QString, QStringList,
    QVariant,
};
use std::thread;
use std::{path::PathBuf, pin::Pin};
use tracing::{debug, debug_span, error, info, instrument};

use self::slide_model::{
    QHash_i32_QByteArray, QMap_QString_QVariant, QVector_i32,
    SlideRoles,
};

#[derive(Clone, Debug)]
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
    slide_count: i32,
    slide_index: i32,
    service_item_id: i32,
    active: bool,
    selected: bool,
    looping: bool,
    video_thumbnail: QString,
    video_start_time: f32,
    video_end_time: f32,
    html: bool,
    obs_scene: QString,
}

impl Default for Slide {
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
            video_start_time: 0.0,
            video_end_time: 0.0,
            html: false,
            obs_scene: QString::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct SlideModelRust {
    id: i32,
    slides: Vec<Slide>,
    obs: Option<Obs>,
    count: i32,
}

impl slide_model::SlideModel {
    pub fn add_video_thumbnail(
        mut self: Pin<&mut Self>,
        index: i32,
    ) -> bool {
        let mut vector_roles = QVector_i32::default();
        vector_roles
            .append(self.get_role(SlideRoles::VideoThumbnail));

        let model_index =
            &self.index(index, 0, &QModelIndex::default());
        if let Some(slide) =
            self.as_mut().rust_mut().slides.get_mut(index as usize)
        {
            if !slide.video_background.is_empty() {
                let path =
                    PathBuf::from(slide.video_background.to_string());
                let video = QString::from(
                    ffmpeg::bg_from_video(&path).to_str().unwrap(),
                )
                .insert(0, &QString::from("file://"))
                .to_owned();
                slide.video_thumbnail = video;
                self.as_mut().data_changed(
                    model_index,
                    model_index,
                    &vector_roles,
                );
            }
        }
        true
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        println!("CLEARING ALL SLIDES");
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().slides.clear();
            self.as_mut().end_reset_model();
        }
    }

    pub fn remove_item_from_service(
        mut self: Pin<&mut Self>,
        index: i32,
        _service_item: &QMap_QString_QVariant,
    ) {
        println!("Rusty-Removal-Time: {:?}", index);
        let slides = self.slides.clone();
        let slides_iter = slides.iter();
        for (i, slide) in slides_iter.enumerate().rev() {
            if slide.service_item_id == index {
                self.as_mut().remove_item(i as i32);
                println!("Removing-slide: {:?}", i);
            } else if slide.service_item_id > index {
                if let Some(slide) =
                    self.as_mut().rust_mut().slides.get_mut(i)
                {
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

    pub fn remove_item(mut self: Pin<&mut Self>, index: i32) {
        if index < 0 || (index as usize) >= self.slides.len() {
            return;
        }

        unsafe {
            self.as_mut().begin_remove_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut().rust_mut().slides.remove(index as usize);
            self.as_mut().end_remove_rows();
        }
        println!("removed-row: {:?}", index);
    }

    fn add_slide(mut self: Pin<&mut Self>, slide: &Slide) {
        let index = self.as_ref().slides.len() as i32;
        println!("{:?}", slide);
        let slide = slide.clone();

        let count = self.as_ref().count;
        self.as_mut().set_count(count + 1);

        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut().rust_mut().slides.push(slide);
            self.as_mut().end_insert_rows();
        }
        let thread = self.qt_thread();
        thread::spawn(move || {
            thread
                .queue(move |slidemodel| {
                    slidemodel.add_video_thumbnail(index);
                })
                .unwrap();
        });
        // self.as_mut().add_video_thumbnail(index);
    }

    fn insert_slide(
        mut self: Pin<&mut Self>,
        slide: &Slide,
        index: i32,
    ) {
        let mut slide = slide.clone();
        // slide.slide_index = index;

        unsafe {
            self.as_mut().begin_insert_rows(
                &QModelIndex::default(),
                index,
                index,
            );
            self.as_mut()
                .rust_mut()
                .slides
                .insert(index as usize, slide);
            self.as_mut().end_insert_rows();
        }
        let thread = self.qt_thread();
        thread::spawn(move || {
            thread
                .queue(move |slidemodel| {
                    slidemodel.add_video_thumbnail(index);
                })
                .unwrap();
        });
    }

    pub fn insert_item_from_service(
        mut self: Pin<&mut Self>,
        index: i32,
        service_item: &QMap_QString_QVariant,
    ) {
        for (key, data) in service_item.iter() {
            debug!(
                ?key,
                data = data.value_or_default::<QString>().to_string()
            );
        }
        let ty = service_item
            .get(&QString::from("ty"))
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

        let text_vec =
            Vec::<QString>::from(&QList_QString::from(&textlist));
        // let vec_slize: &[usize] = &text_vec;

        let mut slide = Slide::default();

        slide.ty = service_item
            .get(&QString::from("type"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.text = service_item
            .get(&QString::from("text"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.image_background = service_item
            .get(&QString::from("imageBackground"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.video_background = service_item
            .get(&QString::from("videoBackground"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.audio = service_item
            .get(&QString::from("audio"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.font = service_item
            .get(&QString::from("font"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.font_size = service_item
            .get(&QString::from("fontSize"))
            .unwrap_or(QVariant::from(&50))
            .value()
            .unwrap_or(50);
        slide.htext_alignment = service_item
            .get(&QString::from("vtextAlignment"))
            .unwrap_or(QVariant::from(&QString::from("center")))
            .value()
            .unwrap_or(QString::from("center"));
        slide.vtext_alignment = service_item
            .get(&QString::from("vtextAlignment"))
            .unwrap_or(QVariant::from(&QString::from("center")))
            .value()
            .unwrap_or(QString::from("center"));
        slide.service_item_id = index;
        slide.slide_index = service_item
            .get(&QString::from("slideNumber"))
            .unwrap_or(QVariant::from(&0))
            .value()
            .unwrap_or(0);
        slide.slide_count = service_item
            .get(&QString::from("slideCount"))
            .unwrap_or(QVariant::from(&1))
            .value()
            .unwrap_or(1);
        slide.looping = service_item
            .get(&QString::from("loop"))
            .unwrap_or(QVariant::from(&false))
            .value()
            .unwrap_or(false);
        slide.active = service_item
            .get(&QString::from("active"))
            .unwrap_or(QVariant::from(&false))
            .value()
            .unwrap_or(false);
        slide.selected = service_item
            .get(&QString::from("selected"))
            .unwrap_or(QVariant::from(&false))
            .value()
            .unwrap_or(false);
        slide.video_thumbnail = QString::from("");

        let mut binding = self.as_mut().rust_mut();
        let slides_iter = binding.slides.iter_mut();
        let mut slide_index = 0;
        for (i, slide) in slides_iter.enumerate().rev() {
            if slide.service_item_id == index {
                slide_index = i as i32;
                break;
            }
        }

        // We need to move all the current slides service_item_id's up by one.
        let slides_iter = binding.slides.iter_mut();
        for slide in
            slides_iter.filter(|x| x.service_item_id >= index)
        {
            slide.service_item_id += 1;
        }

        match ty {
            Some(ty) if ty == QString::from("image") => {
                slide.ty = ty;
                slide.image_background = background;
                slide.video_background = QString::from("");
                slide.slide_index = 0;
                self.as_mut().insert_slide(&slide, slide_index);
                println!("Image added to slide model!");
            }
            Some(ty) if ty == QString::from("song") => {
                let count = text_vec.len();
                for (i, text) in text_vec.iter().enumerate() {
                    println!(
                        "rust: add song of {:?} length at index {:?}",
                        &count, &slide_index
                    );
                    slide.ty = ty.clone();
                    // println!("{:?}", text_vec[i].clone());
                    slide.text = text.clone();
                    slide.slide_count = count as i32;
                    slide.slide_index = i as i32;
                    if background_type == QString::from("image") {
                        slide.image_background = background.clone();
                        slide.video_background = QString::from("");
                    } else {
                        slide.video_background = background.clone();
                        slide.image_background = QString::from("");
                    }
                    self.as_mut()
                        .insert_slide(&slide, slide_index + i as i32);
                }
            }
            Some(ty) if ty == QString::from("video") => {
                slide.ty = ty;
                slide.image_background = QString::from("");
                slide.video_background = background;
                slide.slide_index = 0;
                self.as_mut().insert_slide(&slide, slide_index);
            }
            Some(ty) if ty == QString::from("presentation") => {
                for i in 0..slide.slide_count {
                    slide.ty = ty.clone();
                    if background.ends_with(
                        &QString::from(".html"),
                        CaseSensitivity::CaseInsensitive,
                    ) {
                        slide.html = true;
                    }
                    slide.image_background = background.clone();
                    slide.video_background = QString::from("");
                    slide.slide_index = i;
                    self.as_mut()
                        .insert_slide(&slide, slide_index + i as i32);
                }
            }
            _ => println!("It's somethign else!"),
        };

        println!("Item added in slide model!");
    }

    pub fn add_item_from_service(
        mut self: Pin<&mut Self>,
        index: i32,
        service_item: &QMap_QString_QVariant,
    ) {
        println!("add rust slide {:?}", index);
        let mut slide = Slide::default();
        let iter = service_item.iter();

        for (key, value) in iter {
            debug!(?key);
            match key.to_string().to_str() {
                "ty" => slide.ty = QString::from(value),
                "background" => {
                    slide.background = QString::from(value)
                }
                "backgroundType" => {
                    slide.background_type = QString::from(value)
                }
                "audio" => slide.audio = QString::from(value),
                "font" => slide.font = QString::from(value),
                "fontSize" => slide.font_size = QString::from(value),
                "looping" => slide.looping = QString::from(value),
                "slideCount" => {
                    slide.slide_count = QString::from(value)
                }
                "videoEndTime" => {
                    slide.video_end_time = QString::from(value)
                }
                "videoStartTime" => {
                    slide.video_end_time = QString::from(value)
                }
            }
        }

        let ty = service_item
            .get(&QString::from("ty"))
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

        let text_vec =
            Vec::<QString>::from(&QList_QString::from(&textlist));
        // let vec_slize: &[usize] = &text_vec;

        slide.ty = service_item
            .get(&QString::from("type"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.text = service_item
            .get(&QString::from("text"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.image_background = service_item
            .get(&QString::from("imageBackground"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.video_background = service_item
            .get(&QString::from("videoBackground"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.audio = service_item
            .get(&QString::from("audio"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.font = service_item
            .get(&QString::from("font"))
            .unwrap_or(QVariant::from(&QString::from("")))
            .value()
            .unwrap_or(QString::from(""));
        slide.font_size = service_item
            .get(&QString::from("fontSize"))
            .unwrap_or(QVariant::from(&50))
            .value()
            .unwrap_or(50);
        slide.htext_alignment = service_item
            .get(&QString::from("vtextAlignment"))
            .unwrap_or(QVariant::from(&QString::from("center")))
            .value()
            .unwrap_or(QString::from("center"));
        slide.vtext_alignment = service_item
            .get(&QString::from("vtextAlignment"))
            .unwrap_or(QVariant::from(&QString::from("center")))
            .value()
            .unwrap_or(QString::from("center"));
        slide.service_item_id = index;
        slide.slide_index = service_item
            .get(&QString::from("slideNumber"))
            .unwrap_or(QVariant::from(&0))
            .value()
            .unwrap_or(0);
        slide.slide_count = service_item
            .get(&QString::from("imageCount"))
            .unwrap_or(QVariant::from(&1))
            .value()
            .unwrap_or(1);
        slide.looping = service_item
            .get(&QString::from("loop"))
            .unwrap_or(QVariant::from(&false))
            .value()
            .unwrap_or(false);
        slide.active = service_item
            .get(&QString::from("active"))
            .unwrap_or(QVariant::from(&false))
            .value()
            .unwrap_or(false);
        slide.selected = service_item
            .get(&QString::from("selected"))
            .unwrap_or(QVariant::from(&false))
            .value()
            .unwrap_or(false);
        slide.video_thumbnail = QString::from("");

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
        let slides = self.slides.clone();
        let slides_iter = slides.iter();

        let mut first_slide = 0;
        let mut dest_slide = 0;
        let mut count = 0;
        let mut dest_count = 0;

        for (i, slide) in slides_iter.clone().enumerate() {
            if slide.service_item_id == source_index {
                first_slide = i as i32;
                count = slide.slide_count;
                break;
            }
        }

        if move_down {
            for (i, slide) in slides_iter.enumerate().rev() {
                if slide.service_item_id == destination_index {
                    // if count > 1 {
                    //     dest_slide = i as i32 - count
                    // } else {
                    dest_slide = i as i32;
                    dest_count = slide.slide_count;
                    println!(
                        "RUST_dest_slide: {:?} with {:?} slides",
                        dest_slide, dest_count
                    );
                    // }
                    break;
                }
            }
        } else {
            for (i, slide) in slides_iter.enumerate() {
                if slide.service_item_id == destination_index {
                    // if count > 1 {
                    //     dest_slide = i as i32 - count
                    // } else {
                    dest_slide = i as i32;
                    println!("RUST_dest_slide: {:?}", dest_slide);
                    // }
                    break;
                }
            }
        }

        println!("RUST_COUNT: {:?}", count);
        println!("RUST_first_slide: {:?}", first_slide);
        println!("RUST_dest_slide: {:?}", dest_slide);
        println!("RUST_len: {:?}", self.rust().slides.len());

        let slides = self.slides.clone();
        let slides_iter = slides.iter();

        unsafe {
            self.as_mut().begin_reset_model();
        }
        self.as_mut().move_items(
            first_slide as usize,
            dest_slide as usize,
            count as usize,
        );

        if count > 1 {
            if move_down {
                for (i, slide) in slides_iter
                    .clone()
                    .enumerate()
                    .filter(|x| {
                        x.0 >= (first_slide + dest_count) as usize
                    })
                    .filter(|x| {
                        x.0 < (first_slide + dest_count + count)
                            as usize
                    })
                {
                    if let Some(slide) =
                        self.as_mut().rust_mut().slides.get_mut(i)
                    {
                        println!(
                            "rust: these ones right here officer. from {:?} to {:?}",
                            slide.service_item_id, destination_index
                        );
                        slide.service_item_id = destination_index;
                    }
                }
            } else {
                for (i, slide) in slides_iter
                    .clone()
                    .enumerate()
                    .filter(|x| x.0 >= dest_slide as usize)
                    .filter(|x| x.0 < (dest_slide + count) as usize)
                {
                    if let Some(slide) =
                        self.as_mut().rust_mut().slides.get_mut(i)
                    {
                        println!(
                            "rust: these ones right here officer. from {:?} to {:?}",
                            slide.service_item_id, destination_index
                        );
                        slide.service_item_id = destination_index;
                    }
                }
            }
        } else {
            if let Some(slide) = self
                .as_mut()
                .rust_mut()
                .slides
                .get_mut(dest_slide as usize)
            {
                println!(
                    "rust: this one right here officer. {:?} from {:?} to {:?}",
                    slide.slide_index, slide.service_item_id, destination_index
                );
                slide.service_item_id = destination_index;
            }
        }

        if move_down {
            for (i, slide) in slides_iter
                .enumerate()
                .filter(|x| x.0 < (first_slide + dest_count) as usize)
                .filter(|x| x.1.service_item_id <= destination_index)
                .filter(|x| x.1.service_item_id >= source_index)
            {
                if let Some(slide) =
                    self.as_mut().rust_mut().slides.get_mut(i)
                {
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
                .filter(|x| {
                    x.0 >= (dest_slide as usize + count as usize)
                })
                .filter(|x| x.1.service_item_id >= destination_index)
                .filter(|x| x.1.service_item_id <= source_index)
            {
                if let Some(slide) =
                    self.as_mut().rust_mut().slides.get_mut(i)
                {
                    println!(
                        "rust-switching-service-of: {:?} to {:?}",
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

        println!("rust-move: {first_slide} to {dest_slide} with {count} slides");
    }

    fn move_items(
        mut self: Pin<&mut Self>,
        source_index: usize,
        dest_index: usize,
        count: usize,
    ) {
        let end_slide = source_index + count - 1;
        println!("rust-end-slide: {:?}", end_slide);
        println!("rust-dest-slide: {:?}", dest_index);
        unsafe {
            self.as_mut().begin_reset_model();
            if source_index < dest_index {
                let move_amount =
                    dest_index - source_index - count + 1;
                // println!("rust-move_amount: {:?}", move_amount);
                self.as_mut().rust_mut().slides
                    [source_index..=dest_index]
                    .rotate_right(move_amount);
            } else {
                let move_amount = end_slide - dest_index - count + 1;
                println!("rust-move_amount: {:?}", move_amount);
                self.as_mut().rust_mut().slides
                    [dest_index..=end_slide]
                    .rotate_left(move_amount);
            }
            self.as_mut().end_reset_model();
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

    pub fn get_slide_from_service(
        self: Pin<&mut Self>,
        index: i32,
    ) -> i32 {
        let slides = self.slides.clone();
        let slides_iter = slides.iter();
        debug!(service_item = index, "Getting slide from this item");
        let mut id = 0;
        if let Some((i, slide)) = slides_iter
            .enumerate()
            .filter(|(i, slide)| slide.service_item_id == index)
            .next()
        {
            debug!(slide_id = i, ?slide);
            id = i as i32;
        }
        id
    }

    pub fn activate(mut self: Pin<&mut Self>, index: i32) -> bool {
        let rc = self.as_ref().count() - 1;
        let tl = &self.as_ref().index(0, 0, &QModelIndex::default());
        let br = &self.as_ref().index(rc, 0, &QModelIndex::default());
        let mut vector_roles = QVector_i32::default();
        vector_roles.append(self.get_role(SlideRoles::Active));
        for slide in self.as_mut().rust_mut().slides.iter_mut() {
            // println!("slide is deactivating {:?}", i);
            slide.active = false;
        }

        let obs = self.as_mut().obs.clone();

        if let Some(slide) =
            self.as_mut().rust_mut().slides.get_mut(index as usize)
        {
            debug!(
                slide = index,
                service_item = slide.service_item_id,
                "This slide is activating"
            );
            // println!("slide is activating {:?}", index);
            // println!("slide-title: {:?}", slide.service_item_id);
            // println!(
            //     "slide-image-background: {:?}",
            //     slide.image_background
            // );
            // println!(
            //     "slide-video-background: {:?}",
            //     slide.video_background
            // );
            slide.active = true;
            self.as_mut().data_changed(tl, br, &vector_roles);

            if let Some(obs) = obs {
                match obs.set_scene(slide.obs_scene.to_string()) {
                    Ok(()) => debug!("Successfully set scene"),
                    Err(e) => error!(e),
                }
            }

            // We use this signal generated by our signals enum to tell QML that
            // the active slide has changed which is used to reposition views.
            self.as_mut().active_change(&index);
            true
        } else {
            false
        }
    }

    fn get_role(&self, role: SlideRoles) -> i32 {
        match role {
            SlideRoles::Text => 1,
            SlideRoles::Active => 12,
            SlideRoles::Selected => 13,
            SlideRoles::Looping => 14,
            SlideRoles::VideoThumbnail => 15,
            SlideRoles::VideoStartTime => 16,
            SlideRoles::VideoEndTime => 17,
            _ => 0,
        }
    }
}

// QAbstractListModel implementation
impl slide_model::SlideModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let role = SlideRoles { repr: role };
        if let Some(slide) = self.slides.get(index.row() as usize) {
            return match role {
                SlideRoles::Ty => QVariant::from(&slide.ty),
                SlideRoles::Text => QVariant::from(&slide.text),
                SlideRoles::Audio => QVariant::from(&slide.audio),
                SlideRoles::ImageBackground => {
                    QVariant::from(&slide.image_background)
                }
                SlideRoles::VideoBackground => {
                    QVariant::from(&slide.video_background)
                }
                SlideRoles::HTextAlignment => {
                    QVariant::from(&slide.htext_alignment)
                }
                SlideRoles::VTextAlignment => {
                    QVariant::from(&slide.vtext_alignment)
                }
                SlideRoles::Font => QVariant::from(&slide.font),
                SlideRoles::FontSize => {
                    QVariant::from(&slide.font_size)
                }
                SlideRoles::ServiceItemId => {
                    QVariant::from(&slide.service_item_id)
                }
                SlideRoles::SlideIndex => {
                    QVariant::from(&slide.slide_index)
                }
                SlideRoles::SlideCount => {
                    QVariant::from(&slide.slide_count)
                }
                SlideRoles::Active => QVariant::from(&slide.active),
                SlideRoles::Selected => {
                    QVariant::from(&slide.selected)
                }
                SlideRoles::Looping => QVariant::from(&slide.looping),
                SlideRoles::VideoThumbnail => {
                    QVariant::from(&slide.video_thumbnail)
                }
                SlideRoles::VideoStartTime => {
                    QVariant::from(&slide.video_start_time)
                }
                SlideRoles::VideoEndTime => {
                    QVariant::from(&slide.video_end_time)
                }
                SlideRoles::Html => QVariant::from(&slide.html),
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
        roles.insert(SlideRoles::Ty.repr, QByteArray::from("type"));
        roles.insert(SlideRoles::Text.repr, QByteArray::from("text"));
        roles.insert(
            SlideRoles::Audio.repr,
            QByteArray::from("audio"),
        );
        roles.insert(
            SlideRoles::ImageBackground.repr,
            QByteArray::from("imageBackground"),
        );
        roles.insert(
            SlideRoles::VideoBackground.repr,
            QByteArray::from("videoBackground"),
        );
        roles.insert(
            SlideRoles::HTextAlignment.repr,
            QByteArray::from("hTextAlignment"),
        );
        roles.insert(
            SlideRoles::VTextAlignment.repr,
            QByteArray::from("vTextAlignment"),
        );
        roles.insert(SlideRoles::Font.repr, QByteArray::from("font"));
        roles.insert(
            SlideRoles::FontSize.repr,
            QByteArray::from("fontSize"),
        );
        roles.insert(
            SlideRoles::ServiceItemId.repr,
            QByteArray::from("serviceItemId"),
        );
        roles.insert(
            SlideRoles::SlideIndex.repr,
            QByteArray::from("slideIndex"),
        );
        roles.insert(
            SlideRoles::SlideCount.repr,
            QByteArray::from("imageCount"),
        );
        roles.insert(
            SlideRoles::Active.repr,
            QByteArray::from("active"),
        );
        roles.insert(
            SlideRoles::Selected.repr,
            QByteArray::from("selected"),
        );
        roles.insert(
            SlideRoles::Looping.repr,
            QByteArray::from("looping"),
        );
        roles.insert(
            SlideRoles::VideoThumbnail.repr,
            QByteArray::from("videoThumbnail"),
        );
        roles.insert(
            SlideRoles::VideoStartTime.repr,
            QByteArray::from("videoStartTime"),
        );
        roles.insert(
            SlideRoles::VideoEndTime.repr,
            QByteArray::from("videoEndTime"),
        );
        roles.insert(SlideRoles::Html.repr, QByteArray::from("html"));
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        let cnt = self.rust().slides.len() as i32;
        // println!("row count is {cnt}");
        cnt
    }
}
