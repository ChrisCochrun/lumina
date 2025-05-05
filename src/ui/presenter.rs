use miette::{IntoDiagnostic, Result};
use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

use cosmic::{
    iced::{
        alignment::Horizontal,
        font::{Family, Stretch, Style, Weight},
        Background, Border, Color, ContentFit, Font, Length, Shadow,
        Vector,
    },
    iced_widget::{
        rich_text,
        scrollable::{
            scroll_to, AbsoluteOffset, Direction, Scrollbar,
        },
        span, stack,
    },
    prelude::*,
    widget::{
        container, image, mouse_area, responsive, scrollable, Column,
        Container, Id, Row, Space,
    },
    Task,
};
use iced_video_player::{gst_pbutils, Position, Video, VideoPlayer};
use rodio::{Decoder, OutputStream, Sink};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::{
    core::{service_items::ServiceItemModel, slide::Slide},
    ui::text_svg::{self, Font as SvgFont},
    BackgroundKind,
};

const REFERENCE_WIDTH: f32 = 1920.0;
const REFERENCE_HEIGHT: f32 = 1080.0;

// #[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    pub slides: Vec<Slide>,
    pub items: ServiceItemModel,
    pub current_slide: Slide,
    pub current_slide_index: u16,
    pub video: Option<Video>,
    pub video_position: f32,
    pub audio: Option<PathBuf>,
    sink: (OutputStream, Arc<Sink>),
    hovered_slide: i32,
    scroll_id: Id,
    current_font: Font,
}

pub(crate) enum Action {
    Task(Task<Message>),
    None,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(u16),
    EndVideo,
    StartVideo,
    StartAudio,
    EndAudio,
    VideoPos(f32),
    VideoFrame,
    MissingPlugin(gstreamer::Message),
    HoveredSlide(i32),
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
            r#"playbin uri="{}" video-sink="videoscale ! videoconvert ! appsink name=iced_video drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1""#,
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
        let video_sink = bin.by_name("iced_video").unwrap();
        let video_sink =
            video_sink.downcast::<gst_app::AppSink>().unwrap();
        let result =
            Video::from_gst_pipeline(pipeline, video_sink, None);
        result.into_diagnostic()
    }
    pub fn with_items(items: ServiceItemModel) -> Self {
        let slides = items.to_slides().unwrap_or_default();
        let video = {
            if let Some(slide) = slides.first() {
                let path = slide.background().path.clone();
                if path.exists() {
                    let url = Url::from_file_path(path).unwrap();
                    let result = Self::create_video(url);
                    match result {
                        Ok(mut v) => {
                            v.set_paused(true);
                            Some(v)
                        }
                        Err(e) => {
                            error!("Had an error creating the video object: {e}, likely the first slide isn't a video");
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };
        Self {
            slides: slides.clone(),
            items,
            current_slide: slides[0].clone(),
            current_slide_index: 0,
            video,
            audio: slides[0].audio(),
            video_position: 0.0,
            hovered_slide: -1,
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
                debug!("next slide");
                if self.slides.len() as u16 - 1
                    == self.current_slide_index
                {
                    debug!("no more slides");
                    return Action::None;
                }
                return self.update(Message::SlideChange(
                    self.current_slide_index + 1,
                ));
            }
            Message::PrevSlide => {
                debug!("prev slide");
                if 0 == self.current_slide_index {
                    debug!("beginning slides");
                    return Action::None;
                }
                return self.update(Message::SlideChange(
                    self.current_slide_index - 1,
                ));
            }
            Message::SlideChange(id) => {
                debug!(id, "slide changed");
                let old_background =
                    self.current_slide.background().clone();
                self.current_slide_index = id;
                if let Some(slide) = self.slides.get(id as usize) {
                    self.current_slide = slide.clone();
                    let _ = self
                        .update(Message::ChangeFont(slide.font()));
                }
                if self.current_slide.background() != &old_background
                {
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
                let mut tasks = vec![];
                tasks.push(scroll_to(self.scroll_id.clone(), offset));

                if let Some(audio) = &mut self.current_slide.audio() {
                    let audio = audio.to_str().unwrap().to_string();
                    let audio = if let Some(audio) =
                        audio.strip_prefix(r"file://")
                    {
                        audio
                    } else {
                        audio.as_str()
                    };
                    let audio = PathBuf::from(audio);
                    debug!("{:?}", audio);
                    if audio.exists() {
                        match &self.audio {
                            Some(aud) if aud != &audio => {
                                self.audio = Some(audio.clone());
                                tasks.push(self.start_audio());
                            }
                            Some(_) => (),
                            None => {
                                self.audio = Some(audio.clone());
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
                return Action::Task(self.start_audio())
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

    fn start_audio(&mut self) -> Task<Message> {
        if let Some(audio) = &mut self.audio {
            let audio = audio.clone();
            Task::perform(
                start_audio(Arc::clone(&self.sink.1), audio),
                |_| Message::None,
            )
        } else {
            Task::none()
        }
    }

    pub fn view(&self) -> Element<Message> {
        slide_view(
            self.current_slide.clone(),
            &self.video,
            self.current_font,
            false,
            true,
        )
    }

    pub fn view_preview(&self) -> Element<Message> {
        slide_view(
            self.current_slide.clone(),
            &self.video,
            self.current_font,
            false,
            false,
        )
    }

    pub fn preview_bar(&self) -> Element<Message> {
        let mut items = vec![];
        for slide in self.slides.iter() {
            items.push(self.slide_delegate(slide));
        }
        let row =
            scrollable(Row::from_vec(items).spacing(10).padding(15))
                .direction(Direction::Horizontal(Scrollbar::new()))
                .height(Length::Fill)
                .width(Length::Fill)
                .id(self.scroll_id.clone());
        row.into()
    }

    fn slide_delegate<'a>(
        &'a self,
        slide: &'a Slide,
    ) -> Element<'a, Message> {
        let font_name = slide.font().into_boxed_str();
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

        let slide_id =
            self.slides.iter().position(|s| s == slide).unwrap()
                as i32;

        let container =
            slide_view(slide.clone(), &self.video, font, true, false);
        let delegate = mouse_area(
            Container::new(container)
                .style(move |t| {
                    let mut style = container::Style::default();
                    let theme = t.cosmic();
                    let hovered = self.hovered_slide == slide_id;
                    style.background = Some(Background::Color(
                        if self.current_slide_index as i32 == slide_id
                        {
                            theme.accent.base.into()
                        } else if hovered {
                            theme.accent.hover.into()
                        } else {
                            theme.palette.neutral_3.into()
                        },
                    ));
                    style.border = Border::default().rounded(10.0);
                    style.shadow = Shadow {
                        color: Color::BLACK,
                        offset: {
                            if self.current_slide_index as i32
                                == slide_id
                            {
                                Vector::new(5.0, 5.0)
                            } else if hovered {
                                Vector::new(5.0, 5.0)
                            } else {
                                Vector::new(0.0, 0.0)
                            }
                        },
                        blur_radius: {
                            if self.current_slide_index as i32
                                == slide_id
                            {
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
        .interaction(cosmic::iced::mouse::Interaction::Pointer)
        .on_move(move |_| Message::HoveredSlide(slide_id))
        .on_exit(Message::HoveredSlide(-1))
        .on_press(Message::SlideChange(slide_id as u16));
        delegate.into()
    }

    fn reset_video(&mut self) {
        if let Some(slide) =
            self.slides.get(self.current_slide_index as usize)
        {
            match slide.background().kind {
                BackgroundKind::Image => self.video = None,
                BackgroundKind::Video => {
                    let path = slide.background().path.clone();
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
                                error!("Had an error creating the video object: {e}");
                                self.video = None;
                            }
                        }
                    } else {
                        self.video = None;
                    }
                }
            }
        }
    }
}

#[allow(clippy::unused_async)]
async fn start_audio(sink: Arc<Sink>, audio: PathBuf) {
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

pub(crate) fn slide_view(
    slide: Slide,
    video: &Option<Video>,
    font: Font,
    delegate: bool,
    hide_mouse: bool,
) -> Element<'_, Message> {
    responsive(move |size| {
        let width = size.height * 16.0 / 9.0;
        let font_size = scale_font(slide.font_size() as f32, width);
        let font = SvgFont::from(font).size(font_size.floor() as u8);
        let slide_text = slide.text();

        // SVG based
        let text = text_svg::TextSvg::new()
            .text(&slide_text)
            .fill("#fff")
            .shadow(text_svg::shadow(2, 2, 5, "#000000"))
            .stroke(text_svg::stroke(1, "#000"))
            .font(font)
            .view()
            .map(|m| Message::None);

        // let text = text!("{}", &slide_text);
        // text widget based
        // let lines = slide_text.lines();
        // let text: Vec<Element<Message>> = lines
        //     .map(|t| {
        //         rich_text([span(format!("{}\n", t))
        //             .background(
        //                 Background::Color(Color::BLACK)
        //                     .scale_alpha(0.4),
        //             )
        //             .padding(1)])
        //         .size(font_size)
        //         .font(font)
        //         .center()
        //         .into()
        //     })
        //     .collect();
        // let text = Column::with_children(text).spacing(6);

        //Next
        let text_container = Container::new(text)
            .center(Length::Fill)
            .align_x(Horizontal::Left);
        let black = Container::new(Space::new(0, 0))
            .style(|_| {
                container::background(Background::Color(Color::BLACK))
            })
            .clip(true)
            .width(width)
            .height(size.height);
        let container = match slide.background().kind {
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
                        .center_x(width)
                        .center_y(size.height)
                        .clip(true)
                        .width(Length::Fill)
                        .height(Length::Fill)
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
                } else {
                    Container::new(Space::new(0, 0))
                }
            }
        };
        let stack = stack!(
            black,
            container.center(Length::Fill),
            text_container
        );
        Container::new(stack).center(Length::Fill).into()
    })
    .into()
}
