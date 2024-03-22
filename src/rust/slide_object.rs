#[cxx_qt::bridge]
mod slide_object {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant =
            cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        // #[cxx_name = "SlideModel"]
        // type SlideModel = crate::slide_model::SlideModelRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn playing_changed(
            self: Pin<&mut SlideObject>,
            is_playing: bool,
        );
        // #[qsignal]
        // fn slide_index_changed(
        //     self: Pin<&mut SlideObject>,
        //     slide_index: i32,
        // );
        // #[qsignal]
        // fn slide_size_changed(
        //     self: Pin<&mut SlideObject>,
        //     slide_size: i32,
        // );
        #[qsignal]
        fn slide_changed(self: Pin<&mut SlideObject>, slide: i32);
        #[qsignal]
        fn loop_changed(self: Pin<&mut SlideObject>, looping: bool);
        #[qsignal]
        fn reveal_next(self: Pin<&mut SlideObject>);
        #[qsignal]
        fn reveal_prev(self: Pin<&mut SlideObject>);

        #[qobject]
        #[qml_element]
        #[qproperty(i32, slide_index)]
        #[qproperty(i32, slide_size)]
        #[qproperty(i32, image_count)]
        #[qproperty(bool, is_playing)]
        #[qproperty(bool, looping)]
        #[qproperty(QString, text)]
        #[qproperty(QString, ty)]
        #[qproperty(QString, audio)]
        #[qproperty(QString, image_background)]
        #[qproperty(QString, video_background)]
        #[qproperty(bool, html)]
        #[qproperty(QString, vtext_alignment)]
        #[qproperty(QString, htext_alignment)]
        #[qproperty(QString, font)]
        #[qproperty(i32, font_size)]
        #[qproperty(f32, video_start_time)]
        #[qproperty(f32, video_end_time)]
        // #[qproperty(*mut SlideModel, slide_model)]
        type SlideObject = super::SlideObjectRust;

        #[qinvokable]
        fn change_slide(
            self: Pin<&mut SlideObject>,
            item: QMap_QString_QVariant,
            index: i32,
        );
        #[qinvokable]
        fn next(
            self: Pin<&mut SlideObject>,
            next_item: QMap_QString_QVariant,
        ) -> bool;
        #[qinvokable]
        fn previous(
            self: Pin<&mut SlideObject>,
            next_item: QMap_QString_QVariant,
        ) -> bool;
        #[qinvokable]
        pub fn play(self: Pin<&mut SlideObject>) -> bool;
        #[qinvokable]
        pub fn pause(self: Pin<&mut SlideObject>) -> bool;
        #[qinvokable]
        pub fn play_pause(self: Pin<&mut SlideObject>) -> bool;
    }
}

use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{CaseSensitivity, QString, QVariant};
use tracing::debug;

use self::slide_object::QMap_QString_QVariant;

#[derive(Clone, Debug)]
pub struct SlideObjectRust {
    slide_index: i32,
    slide_size: i32,
    image_count: i32,
    is_playing: bool,
    looping: bool,
    text: QString,
    ty: QString,
    audio: QString,
    image_background: QString,
    video_background: QString,
    html: bool,
    vtext_alignment: QString,
    htext_alignment: QString,
    font: QString,
    font_size: i32,
    video_start_time: f32,
    video_end_time: f32,
    // slide_model: *mut qobject::SlideModel,
}

impl Default for SlideObjectRust {
    fn default() -> Self {
        Self {
            slide_index: 0,
            slide_size: 0,
            is_playing: false,
            looping: false,
            text: QString::from(""),
            ty: QString::from(""),
            audio: QString::from(""),
            image_background: QString::from(""),
            html: false,
            video_background: QString::from(""),
            vtext_alignment: QString::from(""),
            htext_alignment: QString::from(""),
            font: QString::from(""),
            font_size: 50,
            image_count: 0,
            video_start_time: 0.0,
            video_end_time: 0.0,
            // slide_model: std::ptr::null_mut(),
        }
    }
}

impl slide_object::SlideObject {
    pub fn change_slide(
        mut self: Pin<&mut Self>,
        item: QMap_QString_QVariant,
        slide_index: i32,
    ) {
        let current_index = self.as_ref().get_ref().slide_index();
        let icount_variant = item
            .get(&QString::from("imageCount"))
            .unwrap_or(QVariant::from(&1));
        let count = icount_variant.value::<i32>().unwrap_or_default();

        let slindex = item
            .get(&QString::from("slideIndex"))
            .unwrap_or(QVariant::from(&0));
        // let slide_index = slindex.value::<i32>().unwrap_or_default();

        // let html = item
        //     .get(&QString::from("html"))
        //     .unwrap_or(QVariant::from(&false));
        // if let Some(html) = html.value::<bool>() {
        //     if html {
        //         debug!(?html, count, slide_index);
        //         if slide_index > 0 && slide_index < count - 1 {
        //             if current_index < &index {
        //                 self.as_mut().reveal_next();
        //                 debug!("RevealNext");
        //                 return;
        //             } else if slide_index > 0 {
        //                 self.as_mut().reveal_prev();
        //                 debug!("RevealPrev");
        //                 return;
        //             }
        //         }
        //     }
        // }
        debug!(slide_index, "Changing slide");

        println!("## Slide Details ##");
        let text = item
            .get(&QString::from("text"))
            .unwrap_or(QVariant::from(&QString::from("")));
        if let Some(txt) = text.value::<QString>() {
            if &txt != self.as_ref().text() {
                println!("text: {txt}");
                self.as_mut().set_text(txt);
            };
        } else {
            println!("text: empty");
        }
        let audio = item
            .get(&QString::from("audio"))
            .unwrap_or(QVariant::from(&QString::from("")));
        if let Some(audio) = audio.value::<QString>() {
            if &audio != self.as_ref().audio() {
                println!("audio: {audio}");
                self.as_mut().set_audio(audio);
            }
        } else {
            println!("audio: empty");
        }
        let ty = item
            .get(&QString::from("type"))
            .unwrap_or(QVariant::from(&QString::from("")));
        if let Some(ty) = ty.value::<QString>() {
            if &ty != self.as_ref().ty() {
                println!("type: {ty}");
                self.as_mut().set_ty(ty);
            }
        } else {
            println!("type: empty");
        }

        let image_background = item
            .get(&QString::from("imageBackground"))
            .unwrap_or(QVariant::from(&QString::from("")));
        if let Some(image_background) =
            image_background.value::<QString>()
        {
            if &image_background != self.as_ref().image_background() {
                println!("image-bg: {image_background}");
                self.as_mut().set_image_background(image_background);
            }
        } else {
            println!("image-bg: empty");
        }
        let video_background = item
            .get(&QString::from("videoBackground"))
            .unwrap_or(QVariant::from(&QString::from("")));
        if let Some(video_background) =
            video_background.value::<QString>()
        {
            if &video_background != self.as_ref().video_background() {
                println!("video-bg: {video_background}");
                self.as_mut().set_video_background(video_background);
            }
        } else {
            println!("video-bg: empty");
        }
        let font = item
            .get(&QString::from("font"))
            .unwrap_or(QVariant::from(&QString::from("Quicksand")));
        if let Some(font) = font.value::<QString>() {
            if &font != self.as_ref().font() {
                println!("font: {font}");
                self.as_mut().set_font(font);
            }
        } else {
            println!("font: empty");
        }
        let vtext_alignment = item
            .get(&QString::from("verticalTextAlignment"))
            .unwrap_or(QVariant::from(&QString::from("center")));
        if let Some(vtext_alignment) =
            vtext_alignment.value::<QString>()
        {
            if &vtext_alignment != self.as_ref().vtext_alignment() {
                println!("vertical-text-align: {vtext_alignment}");
                self.as_mut().set_vtext_alignment(vtext_alignment);
            }
        } else {
            println!("vertical-text-align: empty");
        }
        let htext_alignment = item
            .get(&QString::from("horizontalTextAlignment"))
            .unwrap_or(QVariant::from(&QString::from("center")));
        if let Some(htext_alignment) =
            htext_alignment.value::<QString>()
        {
            if &htext_alignment != self.as_ref().htext_alignment() {
                println!("horizontal-text-align: {htext_alignment}");
                self.as_mut().set_htext_alignment(htext_alignment);
            }
        } else {
            println!("horizontal-text-align: empty");
        }
        let font_size = item
            .get(&QString::from("fontSize"))
            .unwrap_or(QVariant::from(&50));
        if let Some(font_size) = font_size.value::<i32>() {
            if font_size != self.as_ref().font_size {
                println!("font-size: {font_size}");
                self.as_mut().rust_mut().font_size = font_size;
            }
        } else {
            println!("font-size: empty");
        }
        let looping = item
            .get(&QString::from("looping"))
            .unwrap_or(QVariant::from(&false));
        if let Some(looping) = looping.value::<bool>() {
            if &looping != self.as_ref().looping() {
                println!("looping: {looping}");
                self.as_mut().set_looping(looping);
                let lp = looping;
                self.as_mut().loop_changed(lp)
            }
        } else {
            println!("looping: empty")
        }
        let slide_size = item
            .get(&QString::from("slide_size"))
            .unwrap_or(QVariant::from(&1));
        if let Some(slide_size) = slide_size.value::<i32>() {
            if &slide_size != self.as_ref().slide_size() {
                println!("slide-size: {slide_size}");
                self.as_mut().set_slide_size(slide_size);
            }
        }
        let video_start_time = item
            .get(&QString::from("videoStartTime"))
            .unwrap_or(QVariant::from(&0.0));
        if let Some(int) = video_start_time.value::<f32>() {
            self.as_mut().set_video_start_time(int)
        }

        let html =
            image_background.value_or_default::<QString>().ends_with(
                &QString::from(".html"),
                CaseSensitivity::CaseInsensitive,
            );
        self.as_mut().set_html(html);

        self.as_mut().set_image_count(count);

        self.as_mut().set_slide_index(slide_index);

        // This is pointing to the slide in the overall model. So to get it
        // we need to either track the overall slide index, or get it from
        self.as_mut().slide_changed(slide_index);
        // self.as_mut().emit(Signals::SlideChanged { slide: &index });
        println!("## Slide End ##");
    }

    pub fn next(
        mut self: Pin<&mut Self>,
        next_item: QMap_QString_QVariant,
    ) -> bool {
        debug!(
            item = ?next_item.get(&QString::from("type")).unwrap().value::<QString>(),
            ibg = ?next_item.get(&QString::from("imageBackground")).unwrap().value::<QString>(),
            vbg = ?next_item.get(&QString::from("videoBackground")).unwrap().value::<QString>(),
            "advancing to next slide"
        );
        let new_id = self.as_ref().slide_index() + 1;
        let html = self.as_ref().image_background.ends_with(
            &QString::from(".html"),
            CaseSensitivity::CaseInsensitive,
        );
        // let service_item = next_item
        //     .get(&QString::from("serviceItemId"))
        //     .unwrap()
        //     .value::<i32>()
        //     .unwrap_or_default();
        if html {
            // Check to see if current slide is at the end
            // if not, advance to the next one.
            debug!(
                currentIndex = self.as_ref().slide_index,
                newIndex = new_id,
                slide_count = self.as_ref().image_count
            );
            if self.as_ref().slide_index
                < self.as_ref().image_count - 1
            {
                // self.as_mut().set_slide_index(new_id);
                self.as_mut().reveal_next();
                debug!("returning false");
                return false;
            }
        }
        self.as_mut().set_slide_index(new_id);
        // reset to empty before change to ensure that the web source gets unloaded
        self.as_mut().set_image_background(QString::from(""));
        self.as_mut().change_slide(next_item, new_id);
        debug!(new_id, "returning true");
        true
    }
    pub fn previous(
        mut self: Pin<&mut Self>,
        prev_item: QMap_QString_QVariant,
    ) -> bool {
        debug!(
            item = ?prev_item.get(&QString::from("type")).unwrap().value::<QString>(),
            ibg = ?prev_item.get(&QString::from("imageBackground")).unwrap().value::<QString>(),
            vbg = ?prev_item.get(&QString::from("videoBackground")).unwrap().value::<QString>(),
            "backing to previous slide"
        );
        let new_id = self.as_ref().slide_index() - 1;
        let html = self.as_ref().image_background.ends_with(
            &QString::from(".html"),
            CaseSensitivity::CaseInsensitive,
        );
        if html {
            // Check to see if current slide is at the beginning
            // if not, go back to the previous one.
            if self.as_ref().slide_index > 0 {
                self.as_mut().set_slide_index(new_id);
                self.as_mut().reveal_prev();
                return false;
            }
        }
        self.as_mut().set_slide_index(new_id);
        // reset to empty before change to ensure that the web source gets unloaded
        self.as_mut().set_image_background(QString::from(""));
        self.as_mut().change_slide(prev_item, new_id);
        true
    }
    pub fn play(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().set_is_playing(true);
        self.as_mut().playing_changed(true);
        true
    }
    pub fn pause(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().set_is_playing(false);
        self.as_mut().playing_changed(false);
        false
    }
    pub fn play_pause(mut self: Pin<&mut Self>) -> bool {
        let playing = self.as_ref().is_playing().clone();
        match playing {
            true => self.as_mut().set_is_playing(false),
            false => self.as_mut().set_is_playing(true),
        }
        self.as_mut().playing_changed(!playing);
        !playing
    }
}
