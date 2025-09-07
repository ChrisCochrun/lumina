use miette::{IntoDiagnostic, Result};
use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

use cosmic::{
    iced::{
        alignment::Horizontal,
        border,
        font::{Family, Stretch, Style, Weight},
        Background, Border, Color, ContentFit, Font, Length, Shadow,
        Vector,
    },
    iced_widget::{
        rich_text,
        scrollable::{
            scroll_to, AbsoluteOffset, Direction, Scrollbar,
        },
        span, stack, vertical_rule,
    },
    prelude::*,
    widget::{
        container, image, mouse_area, responsive, scrollable, text,
        Column, Container, Id, Image, Row, Space,
    },
    Task,
};
use iced_video_player::{gst_pbutils, Position, Video, VideoPlayer};
use rodio::{Decoder, OutputStream, Sink};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::{
    core::{service_items::ServiceItem, slide::Slide},
    ui::text_svg,
    BackgroundKind,
};

const REFERENCE_WIDTH: f32 = 1920.0;
const REFERENCE_HEIGHT: f32 = 1080.0;

// #[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    pub service: Vec<ServiceItem>,
    pub current_slide: Slide,
    pub current_item: usize,
    pub current_slide_index: usize,
    pub absolute_slide_index: usize,
    pub total_slides: usize,
    pub video: Option<Video>,
    pub video_position: f32,
    pub audio: Option<PathBuf>,
    sink: (OutputStream, Arc<Sink>),
    hovered_slide: Option<(usize, usize)>,
    scroll_id: Id,
    current_font: Font,
}

pub(crate) enum Action {
    Task(Task<Message>),
    NextSlide,
    PrevSlide,
    None,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(Slide),
    EndVideo,
    StartVideo,
    StartAudio,
    EndAudio,
    VideoPos(f32),
    VideoFrame,
    MissingPlugin(gstreamer::Message),
    HoveredSlide(Option<(usize, usize)>),
    ChangeFont(String),
    Error(String),
    None,
}

impl Presenter {
    fn create_video(url: Url) -> Result<Video> {
        // Based on `iced_video_player::Video::new`,
        // but without a text sink so that the built-in subtitle functionality triggers.
        use gstreamer as gst;
        use gstreamer_app as gst_app;
        use gstreamer_app::prelude::*;

        gst::init().into_diagnostic()?;

        let pipeline = format!(
            r#"playbin uri="{}" video-sink="videoscale ! videoconvert ! appsink name=lumina_video drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1""#,
            url.as_str()
        );

        let pipeline = gst::parse::launch(pipeline.as_ref())
            .into_diagnostic()?;
        let pipeline = pipeline
            .downcast::<gst::Pipeline>()
            .map_err(|_| iced_video_player::Error::Cast)
            .into_diagnostic()?;

        let video_sink: gst::Element =
            pipeline.property("video-sink");
        let pad = video_sink.pads().first().cloned().unwrap();
        let pad = pad.dynamic_cast::<gst::GhostPad>().unwrap();
        let bin = pad
            .parent_element()
            .unwrap()
            .downcast::<gst::Bin>()
            .unwrap();
        let video_sink = bin.by_name("lumina_video").unwrap();
        let video_sink =
            video_sink.downcast::<gst_app::AppSink>().unwrap();
        let result =
            Video::from_gst_pipeline(pipeline, video_sink, None);
        result.into_diagnostic()
    }

    pub fn with_items(items: Vec<ServiceItem>) -> Self {
        let video = {
            if let Some(item) = items.first() {
                if let Some(slide) = item.slides.first() {
                    let path = slide.background().path.clone();
                    if path.exists() {
                        let url = Url::from_file_path(path).unwrap();
                        let result = Video::new(&url);
                        match result {
                            Ok(mut v) => {
                                v.set_paused(true);
                                Some(v)
                            }
                            Err(e) => {
                                error!(
                                    "Had an error creating the video object: {e}, likely the first slide isn't a video"
                                );
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };
        let total_slides: usize =
            items.iter().fold(0, |a, item| a + item.slides.len());

        Self {
            current_slide: items[0].slides[0].clone(),
            current_item: 0,
            current_slide_index: 0,
            absolute_slide_index: 0,
            total_slides,
            video,
            audio: items[0].slides[0].audio().clone(),
            service: items,
            video_position: 0.0,
            hovered_slide: None,
            sink: {
                let (stream, stream_handle) =
                    OutputStream::try_default().unwrap();
                (
                    stream,
                    Arc::new(Sink::try_new(&stream_handle).unwrap()),
                )
            },
            scroll_id: Id::unique(),
            current_font: cosmic::font::default(),
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::NextSlide => {
                return Action::NextSlide;
                // debug!("next slide");
                // if self.slides.len() as u16 - 1
                //     == self.current_slide_index
                // {
                //     debug!("no more slides");
                //     return Action::None;
                // }
                // return self.update(Message::SlideChange(
                //     self.current_slide_index + 1,
                // ));
            }
            Message::PrevSlide => {
                return Action::PrevSlide;
                // debug!("prev slide");
                // if 0 == self.current_slide_index {
                //     debug!("beginning slides");
                //     return Action::None;
                // }
                // return self.update(Message::SlideChange(
                //     self.current_slide_index - 1,
                // ));
            }
            Message::SlideChange(slide) => {
                let slide_text = slide.text();
                debug!(slide_text, "slide changed");
                debug!("comparing background...");
                let backgrounds_match =
                    self.current_slide.background()
                        == slide.background();
                // self.current_slide_index = slide;
                debug!("cloning slide...");
                self.current_slide = slide.clone();
                let _ =
                    self.update(Message::ChangeFont(slide.font()));
                debug!("changing video now...");
                if !backgrounds_match {
                    if let Some(video) = &mut self.video {
                        let _ = video.restart_stream();
                    }
                    self.reset_video();
                }

                let offset = AbsoluteOffset {
                    x: {
                        if self.current_slide_index > 2 {
                            self.current_slide_index as f32 * 187.5
                                - 187.5
                        } else {
                            0.0
                        }
                    },
                    y: 0.0,
                };
                debug!(?offset);
                let mut tasks = vec![];
                tasks.push(scroll_to(self.scroll_id.clone(), offset));

                if let Some(mut new_audio) =
                    self.current_slide.audio()
                {
                    if let Some(stripped_audio) = new_audio
                        .to_str()
                        .unwrap()
                        .to_string()
                        .strip_prefix(r"file://")
                    {
                        new_audio = PathBuf::from(stripped_audio);
                    }
                    debug!("{:?}", new_audio);
                    if new_audio.exists() {
                        let old_audio = self.audio.clone();
                        match old_audio {
                            Some(current_audio)
                                if current_audio != *new_audio =>
                            {
                                debug!(
                                    ?new_audio,
                                    ?current_audio,
                                    "audio needs to change"
                                );
                                self.audio = Some(new_audio.clone());
                                tasks.push(self.start_audio());
                            }
                            Some(current_audio) => {
                                debug!(
                                    ?new_audio,
                                    ?current_audio,
                                    "Same audio shouldn't change"
                                );
                            }
                            None => {
                                debug!(
                                    ?new_audio,
                                    "could not find audio, need to change"
                                );
                                self.audio = Some(new_audio.clone());
                                tasks.push(self.start_audio());
                            }
                        };
                    } else {
                        self.audio = None;
                        self.update(Message::EndAudio);
                    }
                } else {
                    self.audio = None;
                    self.update(Message::EndAudio);
                }
                let task_count = tasks.len();
                debug!(?task_count);
                return Action::Task(Task::batch(tasks));
            }
            Message::ChangeFont(s) => {
                let font_name = s.into_boxed_str();
                let family = Family::Name(Box::leak(font_name));
                let weight = Weight::Normal;
                let stretch = Stretch::Normal;
                let style = Style::Normal;
                let font = Font {
                    family,
                    weight,
                    stretch,
                    style,
                };
                self.current_font = font;
            }
            Message::EndVideo => {
                // if self.current_slide.video_loop() {
                //     if let Some(video) = &mut self.video {
                //         match video.restart_stream() {
                //             Ok(_) => info!("Restarting video"),
                //             Err(e) => {
                //                 error!("Couldn't restart video: {e}")
                //             }
                //         }
                //     }
                // }
            }
            Message::StartVideo => {
                if let Some(video) = &mut self.video {
                    video.set_paused(false);
                    video
                        .set_looping(self.current_slide.video_loop());
                }
            }
            Message::VideoPos(position) => {
                if let Some(video) = &mut self.video {
                    let position = Position::Time(
                        std::time::Duration::from_secs_f32(position),
                    );
                    match video.seek(position, false) {
                        Ok(_) => debug!(
                            "Video position changed: {:?}",
                            position
                        ),
                        Err(e) => error!(
                            "Problem changing video position: {e}"
                        ),
                    }
                }
            }
            Message::VideoFrame => {
                if let Some(video) = &self.video {
                    self.video_position =
                        video.position().as_secs_f32();
                }
            }
            Message::MissingPlugin(element) => {
                if let Some(video) = &mut self.video {
                    video.set_paused(true);
                }
                return Action::Task(Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            match gst_pbutils::MissingPluginMessage::parse(&element) {
                                Ok(missing_plugin) => {
                                    let mut install_ctx = gst_pbutils::InstallPluginsContext::new();
                                    install_ctx
                                        .set_desktop_id(&format!("{}.desktop", "org.chriscochrun.lumina"));
                                    let install_detail = missing_plugin.installer_detail();
                                    println!("installing plugins: {}", install_detail);
                                    let status = gst_pbutils::missing_plugins::install_plugins_sync(
                                        &[&install_detail],
                                        Some(&install_ctx),
                                    );
                                    info!("plugin install status: {}", status);
                                    info!(
                                        "gstreamer registry update: {:?}",
                                        gstreamer::Registry::update()
                                    );
                                }
                                Err(err) => {
                                    warn!("failed to parse missing plugin message: {err}");
                                }
                            }
                            Message::None
                        })
                        .await
                        .unwrap()
                    },
                    |x| x,
                ));
            }
            Message::HoveredSlide(slide) => {
                self.hovered_slide = slide;
            }
            Message::StartAudio => {
                return Action::Task(self.start_audio());
            }
            Message::EndAudio => {
                self.sink.1.stop();
            }
            Message::None => debug!("Presenter Message::None"),
            Message::Error(error) => {
                error!(error);
            }
        };
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        slide_view(
            &self.current_slide,
            &self.video,
            self.current_font,
            false,
            true,
        )
    }

    pub fn view_preview(&self) -> Element<Message> {
        slide_view(
            &self.current_slide,
            &self.video,
            self.current_font,
            false,
            false,
        )
    }

    pub fn preview_bar(&self) -> Element<Message> {
        let mut items = vec![];
        self.service.iter().enumerate().for_each(
            |(item_index, item)| {
                let mut slides = vec![];
                item.slides.iter().enumerate().for_each(
                    |(slide_index, slide)| {
                        let font_name = slide.font().into_boxed_str();
                        let family =
                            Family::Name(Box::leak(font_name));
                        let weight = Weight::Normal;
                        let stretch = Stretch::Normal;
                        let style = Style::Normal;
                        let font = Font {
                            family,
                            weight,
                            stretch,
                            style,
                        };

                        let is_current_slide =
                            (item_index, slide_index)
                                == (
                                    self.current_item,
                                    self.current_slide_index,
                                );

                        let container = slide_view(
                            &slide,
                            &self.video,
                            font,
                            true,
                            false,
                        );
                        let delegate = mouse_area(
                            Container::new(container)
                                .style(move |t| {
                                    let mut style =
                                        container::Style::default();
                                    let theme = t.cosmic();
                                    let hovered = self.hovered_slide
                                        == Some((
                                            item_index,
                                            slide_index,
                                        ));
                                    style.background =
                                        Some(Background::Color(
                                            if is_current_slide {
                                                theme
                                                    .accent
                                                    .base
                                                    .into()
                                            } else if hovered {
                                                theme
                                                    .accent
                                                    .hover
                                                    .into()
                                            } else {
                                                theme
                                                    .palette
                                                    .neutral_3
                                                    .into()
                                            },
                                        ));
                                    style.border = Border::default()
                                        .rounded(10.0);
                                    style.shadow = Shadow {
                                        color: Color::BLACK,
                                        offset: {
                                            if is_current_slide {
                                                Vector::new(5.0, 5.0)
                                            } else if hovered {
                                                Vector::new(5.0, 5.0)
                                            } else {
                                                Vector::new(0.0, 0.0)
                                            }
                                        },
                                        blur_radius: {
                                            if is_current_slide {
                                                10.0
                                            } else if hovered {
                                                10.0
                                            } else {
                                                0.0
                                            }
                                        },
                                    };
                                    style
                                })
                                .center_x(100.0 * 16.0 / 9.0)
                                .height(100)
                                .padding(10),
                        )
                        .interaction(
                            cosmic::iced::mouse::Interaction::Pointer,
                        )
                        .on_move(move |_| {
                            Message::HoveredSlide(Some((
                                item_index,
                                slide_index,
                            )))
                        })
                        .on_exit(Message::HoveredSlide(None))
                        .on_press(Message::SlideChange(
                            slide.clone(),
                        ));
                        slides.push(delegate.into());
                    },
                );
                let row = Row::from_vec(slides)
                    .spacing(10)
                    .padding([20, 15]);
                let label = text::body(item.title.clone());
                let label_container = container(label)
                    .align_top(Length::Fill)
                    .align_left(Length::Fill)
                    .padding([0, 0, 0, 35]);
                let divider = vertical_rule(2);
                items.push(
                    container(stack!(row, label_container))
                        .padding([5, 2])
                        .into(),
                );
                items.push(divider.into());
            },
        );
        let row =
            scrollable(container(Row::from_vec(items)).style(|t| {
                let style = container::Style::default();
                style.border(Border::default().width(2))
            }))
            .direction(Direction::Horizontal(Scrollbar::new()))
            .height(Length::Fill)
            .width(Length::Fill)
            .id(self.scroll_id.clone());
        row.into()
    }

    // fn slide_delegate(&self, slide: &Slide) -> Element<'_, Message> {
    //     let font_name = slide.font().into_boxed_str();
    //     let family = Family::Name(Box::leak(font_name));
    //     let weight = Weight::Normal;
    //     let stretch = Stretch::Normal;
    //     let style = Style::Normal;
    //     let font = Font {
    //         family,
    //         weight,
    //         stretch,
    //         style,
    //     };

    //     let is_current_slide = self.service[self.current_item]
    //         .slides
    //         .get(self.current_slide_index as usize)
    //         .is_some();
    //     let this_item_index = self
    //         .service
    //         .iter()
    //         .position(|item| item == self.service[self.current_item])
    //         .unwrap();
    //     let this_slide_index = self.service[this_item_index].slides;

    //     let container =
    //         slide_view(slide.clone(), &self.video, font, true, false);
    //     let delegate = mouse_area(
    //         Container::new(container)
    //             .style(move |t| {
    //                 let mut style = container::Style::default();
    //                 let theme = t.cosmic();
    //                 let hovered = self.hovered_slide == slide_id;
    //                 style.background = Some(Background::Color(
    //                     if is_current_slide {
    //                         theme.accent.base.into()
    //                     } else if hovered {
    //                         theme.accent.hover.into()
    //                     } else {
    //                         theme.palette.neutral_3.into()
    //                     },
    //                 ));
    //                 style.border = Border::default().rounded(10.0);
    //                 style.shadow = Shadow {
    //                     color: Color::BLACK,
    //                     offset: {
    //                         if is_current_slide {
    //                             Vector::new(5.0, 5.0)
    //                         } else if hovered {
    //                             Vector::new(5.0, 5.0)
    //                         } else {
    //                             Vector::new(0.0, 0.0)
    //                         }
    //                     },
    //                     blur_radius: {
    //                         if is_current_slide {
    //                             10.0
    //                         } else if hovered {
    //                             10.0
    //                         } else {
    //                             0.0
    //                         }
    //                     },
    //                 };
    //                 style
    //             })
    //             .center_x(100.0 * 16.0 / 9.0)
    //             .height(100)
    //             .padding(10),
    //     )
    //     .interaction(cosmic::iced::mouse::Interaction::Pointer)
    //     .on_move(move |_| Message::HoveredSlide(slide_id))
    //     .on_exit(Message::HoveredSlide(-1))
    //     .on_press(Message::SlideChange(slide.clone()));
    //     delegate.into()
    // }

    fn reset_video(&mut self) {
        match self.current_slide.background().kind {
            BackgroundKind::Image => self.video = None,
            BackgroundKind::Video => {
                let path = &self.current_slide.background().path;
                if path.exists() {
                    let url = Url::from_file_path(path).unwrap();
                    let result = Self::create_video(url);
                    match result {
                        Ok(mut v) => {
                            v.set_looping(
                                self.current_slide.video_loop(),
                            );
                            self.video = Some(v)
                        }
                        Err(e) => {
                            error!(
                                "Had an error creating the video object: {e}"
                            );
                            self.video = None;
                        }
                    }
                } else {
                    self.video = None;
                }
            }
        }
    }

    fn start_audio(&mut self) -> Task<Message> {
        if let Some(audio) = &mut self.audio {
            debug!(?audio, "This is where audio should be changing");
            let audio = audio.clone();
            Task::perform(
                start_audio(Arc::clone(&self.sink.1), audio),
                |_| Message::None,
            )
        } else {
            debug!(?self.audio, "Apparently this doesn't exist");
            Task::none()
        }
    }
}

// This needs to be async so that rodio's audio will work
#[allow(clippy::unused_async)]
async fn start_audio(sink: Arc<Sink>, audio: PathBuf) {
    debug!(?audio);
    let file = BufReader::new(File::open(audio).unwrap());
    debug!(?file);
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    let empty = sink.empty();
    let paused = sink.is_paused();
    debug!(empty, paused, "Finished running");
}

fn scale_font(font_size: f32, width: f32) -> f32 {
    let scale_factor = (REFERENCE_WIDTH / width).sqrt();
    // debug!(scale_factor);

    if font_size > 0.0 {
        font_size / scale_factor
    } else {
        50.0
    }
}

pub(crate) fn slide_view<'a>(
    slide: &'a Slide,
    video: &'a Option<Video>,
    font: Font,
    delegate: bool,
    hide_mouse: bool,
) -> Element<'a, Message> {
    let res = responsive(move |size| {
        let width = size.height * 16.0 / 9.0;
        // let slide_text = slide.text();

        // let font = SvgFont::from(font).size(font_size.floor() as u8);
        // let text_container = if delegate {
        //     // text widget based
        // let font_size = scale_font(slide.font_size() as f32, width);
        // let lines = slide_text.lines();
        // let text: Vec<Element<Message>> = lines
        //     .map(|t| {
        //         rich_text([span(format!("{}\n", t))
        //             .background(
        //                 Background::Color(Color::BLACK)
        //                     .scale_alpha(0.4),
        //             )
        //             .border(border::rounded(10))
        //             .padding(10)])
        //         .size(font_size)
        //         .font(font)
        //         .center()
        //         .into()
        //         // let chars: Vec<Span> = t
        //         //     .chars()
        //         //     .map(|c| -> Span {
        //         //         let character: String = format!("{}/n", c);
        //         //         span(character)
        //         //             .size(font_size)
        //         //             .font(font)
        //         //             .background(
        //         //                 Background::Color(Color::BLACK)
        //         //                     .scale_alpha(0.4),
        //         //             )
        //         //             .border(border::rounded(10))
        //         //             .padding(10)
        //     })
        //     .collect();
        // let text = Column::with_children(text).spacing(26);
        // let text = Container::new(text)
        //     .center(Length::Fill)
        //     .align_x(Horizontal::Left);
        // } else {
        //     // SVG based
        //     let text: Element<Message> =
        //         if let Some(text) = &slide.text_svg {
        //             if let Some(handle) = &text.handle {
        //                 debug!("we made it boys");
        //                 Image::new(handle)
        //                     .content_fit(ContentFit::Cover)
        //                     .width(Length::Fill)
        //                     .height(Length::Fill)
        //                     .into()
        //             } else {
        //                 Space::with_width(0).into()
        //             }
        //         } else {
        //             Space::with_width(0).into()
        //         };
        //     Container::new(text)
        //         .center(Length::Fill)
        //         .align_x(Horizontal::Left)
        //     // text widget based
        //     // let font_size =
        //     //     scale_font(slide.font_size() as f32, width);
        //     // let lines = slide_text.lines();
        //     // let text: Vec<Element<Message>> = lines
        //     //     .map(|t| {
        //     //         rich_text([span(format!("{}\n", t))
        //     //             .background(
        //     //                 Background::Color(Color::BLACK)
        //     //                     .scale_alpha(0.4),
        //     //             )
        //     //             .border(border::rounded(10))
        //     //             .padding(10)])
        //     //         .size(font_size)
        //     //         .font(font)
        //     //         .center()
        //     //         .into()
        //     //         // let chars: Vec<Span> = t
        //     //         //     .chars()
        //     //         //     .map(|c| -> Span {
        //     //         //         let character: String = format!("{}/n", c);
        //     //         //         span(character)
        //     //         //             .size(font_size)
        //     //         //             .font(font)
        //     //         //             .background(
        //     //         //                 Background::Color(Color::BLACK)
        //     //         //                     .scale_alpha(0.4),
        //     //         //             )
        //     //         //             .border(border::rounded(10))
        //     //         //             .padding(10)
        //     //     })
        //     //     .collect();
        //     // let text = Column::with_children(text).spacing(26);
        //     // Container::new(text)
        //     //     .center(Length::Fill)
        //     //     .align_x(Horizontal::Left)
        // };

        // let stroke_text_container = Container::new(stroke_text)
        //     .center(Length::Fill)
        //     .align_x(Horizontal::Left);

        // let text_stack =
        //     stack!(stroke_text_container, text_container);

        let text: Element<Message> =
            if let Some(text) = &slide.text_svg {
                if let Some(handle) = &text.handle {
                    image(handle)
                        .content_fit(ContentFit::ScaleDown)
                        .width(width)
                        .height(size.height)
                        .into()
                } else {
                    Space::with_width(0).into()
                }
            } else {
                Space::with_width(0).into()
            };
        let black = Container::new(Space::new(0, 0))
            .style(|_| {
                container::background(Background::Color(Color::BLACK))
            })
            .clip(true)
            .width(width)
            .height(size.height);
        let background = match slide.background().kind {
            BackgroundKind::Image => {
                let path = slide.background().path.clone();
                Container::new(
                    image(path)
                        .content_fit(ContentFit::Cover)
                        .width(width)
                        .height(size.height),
                )
                .center(Length::Shrink)
                .clip(true)
            }
            BackgroundKind::Video => {
                if delegate {
                    Container::new(Space::new(0, 0))
                        .style(|_| {
                            container::background(Background::Color(
                                Color::BLACK,
                            ))
                        })
                        .center(Length::Fill)
                        .clip(true)
                        .width(width)
                        .height(size.height)
                } else if let Some(video) = &video {
                    Container::new(
                        VideoPlayer::new(video)
                            .mouse_hidden(hide_mouse)
                            .width(width)
                            .height(size.height)
                            .on_end_of_stream(Message::EndVideo)
                            .on_new_frame(Message::VideoFrame)
                            .on_missing_plugin(Message::MissingPlugin)
                            .on_warning(|w| {
                                Message::Error(w.to_string())
                            })
                            .on_error(|e| {
                                Message::Error(e.to_string())
                            })
                            .content_fit(ContentFit::Cover),
                    )
                    .center(Length::Shrink)
                    .clip(true)
                    // Container::new(Space::new(0, 0))
                } else {
                    Container::new(Space::new(0, 0))
                }
            }
        };
        let stack =
            stack!(black, background.center(Length::Fill), text);
        Container::new(stack).center(Length::Fill).into()
    });
    // let vid = if let Some(video) = &video {
    //     Container::new(
    //         VideoPlayer::new(video)
    //             .mouse_hidden(hide_mouse)
    //             .width(Length::Fill)
    //             .height(Length::Fill)
    //             .on_end_of_stream(Message::EndVideo)
    //             .on_new_frame(Message::VideoFrame)
    //             .on_missing_plugin(Message::MissingPlugin)
    //             .on_warning(|w| Message::Error(w.to_string()))
    //             .on_error(|e| Message::Error(e.to_string()))
    //             .content_fit(ContentFit::Cover),
    //     )
    //     .center(Length::Shrink)
    //     .clip(true)
    // } else {
    //     Container::new(Space::new(0, 0))
    // };
    // stack!(vid, res).into()
    res.into()
}
