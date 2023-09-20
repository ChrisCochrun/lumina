#[cxx_qt::bridge]
mod slide_obj {
    // use cxx_qt_lib::QVariantValue;
    // use std::path::Path;
    // use std::task::Context;

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant =
            cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    #[cxx_qt::qsignals(SlideObj)]
    pub enum Signals<'a> {
        PlayingChanged { is_playing: &'a bool },
        SlideIndexChanged { slide_index: &'a i32 },
        SlideSizeChanged { slide_size: &'a i32 },
        SlideChanged { slide: &'a i32 },
        LoopChanged { looping: &'a bool },
    }

    #[derive(Clone, Debug)]
    #[cxx_qt::qobject]
    pub struct SlideObj {
        #[qproperty]
        slide_index: i32,
        #[qproperty]
        slide_size: i32,
        #[qproperty]
        image_count: i32,
        #[qproperty]
        is_playing: bool,
        #[qproperty]
        looping: bool,
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
        html: QString,
        #[qproperty]
        vtext_alignment: QString,
        #[qproperty]
        htext_alignment: QString,
        #[qproperty]
        font: QString,
        #[qproperty]
        font_size: i32,
        #[qproperty]
        video_start_time: f32,
        #[qproperty]
        video_end_time: f32,
    }

    impl Default for SlideObj {
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
                html: QString::from(""),
                video_background: QString::from(""),
                vtext_alignment: QString::from(""),
                htext_alignment: QString::from(""),
                font: QString::from(""),
                font_size: 50,
                image_count: 0,
                video_start_time: 0.0,
                video_end_time: 0.0,
            }
        }
    }

    impl qobject::SlideObj {
        #[qinvokable]
        pub fn change_slide(
            mut self: Pin<&mut Self>,
            item: QMap_QString_QVariant,
            index: i32,
        ) {
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
                if &image_background
                    != self.as_ref().image_background()
                {
                    println!("image-bg: {image_background}");
                    self.as_mut()
                        .set_image_background(image_background);
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
                if &video_background
                    != self.as_ref().video_background()
                {
                    println!("video-bg: {video_background}");
                    self.as_mut()
                        .set_video_background(video_background);
                }
            } else {
                println!("video-bg: empty");
            }
            let font = item.get(&QString::from("font")).unwrap_or(
                QVariant::from(&QString::from("Quicksand")),
            );
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
                if &vtext_alignment != self.as_ref().vtext_alignment()
                {
                    println!(
                        "vertical-text-align: {vtext_alignment}"
                    );
                    self.as_mut()
                        .set_vtext_alignment(vtext_alignment);
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
                if &htext_alignment != self.as_ref().htext_alignment()
                {
                    println!(
                        "horizontal-text-align: {htext_alignment}"
                    );
                    self.as_mut()
                        .set_htext_alignment(htext_alignment);
                }
            } else {
                println!("horizontal-text-align: empty");
            }
            let font_size = item
                .get(&QString::from("fontSize"))
                .unwrap_or(QVariant::from(&50));
            if let Some(font_size) = font_size.value::<i32>() {
                if &font_size != self.as_ref().font_size() {
                    println!("font-size: {font_size}");
                    self.as_mut().set_font_size(font_size);
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
                    self.as_mut()
                        .emit(Signals::LoopChanged { looping: &lp });
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
            let icount = item
                .get(&QString::from("imageCount"))
                .unwrap_or(QVariant::from(&1));
            if let Some(int) = icount.value::<i32>() {
                self.as_mut().set_image_count(int);
            }
            let slindex = item
                .get(&QString::from("slideIndex"))
                .unwrap_or(QVariant::from(&0));
            if let Some(int) = slindex.value::<i32>() {
                println!("New slide index = {}", int);
                self.as_mut().set_slide_index(int);
            };
            self.as_mut()
                .emit(Signals::SlideChanged { slide: &index });
            println!("## Slide End ##");
        }

        #[qinvokable]
        pub fn next(
            mut self: Pin<&mut Self>,
            next_item: QMap_QString_QVariant,
        ) -> bool {
            let new_id = self.as_ref().slide_index() + 1;
            self.as_mut().change_slide(next_item, new_id);
            true
        }
        #[qinvokable]
        pub fn previous(
            mut self: Pin<&mut Self>,
            prev_item: QMap_QString_QVariant,
        ) -> bool {
            let new_id = self.as_ref().slide_index() - 1;
            self.as_mut().change_slide(prev_item, new_id);
            true
        }
        #[qinvokable]
        pub fn play(mut self: Pin<&mut Self>) -> bool {
            self.as_mut().set_is_playing(true);
            self.as_mut().emit_playing_changed(&true);
            true
        }
        #[qinvokable]
        pub fn pause(mut self: Pin<&mut Self>) -> bool {
            self.as_mut().set_is_playing(false);
            self.as_mut().emit_playing_changed(&false);
            false
        }
        #[qinvokable]
        pub fn play_pause(mut self: Pin<&mut Self>) -> bool {
            let playing = self.as_ref().is_playing().clone();
            match playing {
                true => self.as_mut().set_is_playing(false),
                false => self.as_mut().set_is_playing(true),
            }
            self.as_mut().emit_playing_changed(&!playing);
            !playing
        }
    }
}
