use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

use cosmic::{
    dialog::ashpd::url::Url,
    iced::{
        alignment::Horizontal,
        font::{Family, Stretch, Style, Weight},
        Background, Border, Color, ContentFit, Font, Length, Shadow,
        Vector,
    },
    iced_widget::{
        scrollable::{
            scroll_to, AbsoluteOffset, Direction, Scrollbar,
        },
        stack,
    },
    prelude::*,
    widget::{
        container, image, mouse_area, responsive, scrollable, text,
        Column, Container, Id, Responsive, Row, Space,
    },
    Task,
};
use iced_video_player::{Position, Video, VideoPlayer};
use rodio::{Decoder, OutputStream, Sink};
use tracing::{debug, error};

use crate::{
    core::{service_items::ServiceItemModel, slide::Slide},
    BackgroundKind,
};

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

#[derive(Debug, Clone, PartialEq)]
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
    HoveredSlide(i32),
    ChangeFont(String),
    None,
}

impl Presenter {
    pub fn with_items(items: ServiceItemModel) -> Self {
        let slides = items.to_slides().unwrap_or_default();
        Self {
            slides: slides.clone(),
            items,
            current_slide: slides[0].clone(),
            current_slide_index: 0,
            video: {
                if let Some(slide) = slides.first() {
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
                                error!("Had an error creating the video object: {e}");
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
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

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NextSlide => {
                debug!("next slide");
                if self.slides.len() as u16 - 1
                    == self.current_slide_index
                {
                    debug!("no more slides");
                    return Task::none();
                }
                self.update(Message::SlideChange(
                    self.current_slide_index + 1,
                ))
            }
            Message::PrevSlide => {
                debug!("prev slide");
                if 0 == self.current_slide_index {
                    debug!("beginning slides");
                    return Task::none();
                }
                self.update(Message::SlideChange(
                    self.current_slide_index - 1,
                ))
            }
            Message::SlideChange(id) => {
                debug!(id, "slide changed");
                self.current_slide_index = id;
                if let Some(slide) = self.slides.get(id as usize) {
                    self.current_slide = slide.clone();
                    let _ = self
                        .update(Message::ChangeFont(slide.font()));
                }
                if let Some(video) = &mut self.video {
                    let _ = video.restart_stream();
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

                self.reset_video();
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
                                tasks.push(
                                    self.update(Message::StartAudio),
                                );
                            }
                            Some(_) => (),
                            None => {
                                self.audio = Some(audio.clone());
                                tasks.push(
                                    self.update(Message::StartAudio),
                                );
                            }
                        };
                    } else {
                        self.audio = None;
                        tasks.push(self.update(Message::EndAudio));
                    }
                }
                Task::batch(tasks)
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
                Task::none()
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
                Task::none()
            }
            Message::StartVideo => {
                if let Some(video) = &mut self.video {
                    video.set_paused(false);
                    video
                        .set_looping(self.current_slide.video_loop());
                }
                Task::none()
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
                Task::none()
            }
            Message::VideoFrame => {
                if let Some(video) = &self.video {
                    self.video_position =
                        video.position().as_secs_f32();
                }
                Task::none()
            }
            Message::HoveredSlide(slide) => {
                self.hovered_slide = slide;
                Task::none()
            }
            Message::StartAudio => {
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
            Message::EndAudio => {
                self.sink.1.stop();
                Task::none()
            }
            Message::None => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        responsive(|size| {
            let font = self.current_font;
            let font_size = if self.current_slide.font_size() > 0 {
                (size.width / self.current_slide.font_size() as f32)
                    * 3.0
            } else {
                50.0
            };
            let slide_text = self.current_slide.text();
            let lines = slide_text.lines();
            let line_size = lines.clone().count();
            // debug!(?lines);
            let text: Vec<Element<Message>> = lines
                .map(|t| {
                    text(t.to_string())
                        .size(font_size)
                        .font(font)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .into()
                })
                .collect();
            let texts = Column::with_children(text);
            // let text = text(self.current_slide.text())
            //     .size(font_size)
            //     .font(font)
            //     .align_x(Horizontal::Center);
            let text = Container::new(texts).center(Length::Fill);
            let text_background = Container::new(Space::new(0, 0))
                .style(|_| {
                    container::background(
                        Background::Color(Color::BLACK)
                            .scale_alpha(0.3),
                    )
                })
                .width(size.width)
                .height(
                    font_size * line_size as f32 * line_size as f32,
                );
            let text_stack = stack!(text_background, text);
            let text_container =
                Container::new(text_stack).center(Length::Fill);
            let black = Container::new(Space::new(0, 0))
                .style(|_| {
                    container::background(Background::Color(
                        Color::BLACK,
                    ))
                })
                .width(size.width)
                .height(size.height);
            let container = match self.current_slide.background().kind
            {
                BackgroundKind::Image => {
                    let path =
                        self.current_slide.background().path.clone();
                    Container::new(
                        image(path)
                            .content_fit(ContentFit::Cover)
                            .width(size.width)
                            .height(size.height),
                    )
                }
                BackgroundKind::Video => {
                    if let Some(video) = &self.video {
                        Container::new(
                            VideoPlayer::new(video)
                                .mouse_hidden(true)
                                .width(size.width)
                                .height(size.width * 9.0 / 16.0)
                                .on_end_of_stream(Message::EndVideo)
                                .on_new_frame(Message::VideoFrame)
                                .content_fit(ContentFit::Cover),
                        )
                    } else {
                        Container::new(Space::new(0, 0))
                    }
                }
            };
            stack!(
                black,
                container.center(Length::Fill),
                text_container
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        })
        .into()
    }

    pub fn view_preview(&self) -> Element<Message> {
        responsive(|size| {
            let font = self.current_font;
            let font_size = if self.current_slide.font_size() > 0 {
                (size.width / self.current_slide.font_size() as f32)
                    * 3.0
            } else {
                50.0
            };
            let text = text(self.current_slide.text())
                .size(font_size)
                .font(font);
            let text = Container::new(text).center(Length::Fill);
            let black = Container::new(Space::new(0, 0))
                .style(|_| {
                    container::background(Background::Color(
                        Color::BLACK,
                    ))
                })
                .width(size.width)
                .height(size.height);
            let container = match self.current_slide.background().kind
            {
                BackgroundKind::Image => {
                    let path =
                        self.current_slide.background().path.clone();
                    Container::new(
                        image(path)
                            .content_fit(ContentFit::Cover)
                            .width(size.width)
                            .height(size.height),
                    )
                }
                BackgroundKind::Video => {
                    if let Some(video) = &self.video {
                        Container::new(
                            VideoPlayer::new(video)
                                .width(size.width)
                                .height(size.width * 9.0 / 16.0)
                                .on_end_of_stream(Message::EndVideo)
                                .on_new_frame(Message::VideoFrame)
                                .content_fit(ContentFit::Cover),
                        )
                    } else {
                        Container::new(Space::new(0, 0))
                    }
                }
            };
            stack!(black, container.center(Length::Fill), text)
                .width(size.width)
                .height(size.width * 9.0 / 16.0)
                .into()
        })
        .into()
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
        let text = Responsive::new(move |size| {
            let font_size = if slide.font_size() > 0 {
                (size.width / slide.font_size() as f32) * 3.0
            } else {
                50.0
            };
            let text = text(slide.text()).size(font_size).font(font);
            let text = Container::new(text).center(Length::Fill);
            text.into()
        });
        let container = match slide.background().kind {
            BackgroundKind::Image => {
                let path = slide.background().path.clone();

                Container::new(
                    image(path)
                        .content_fit(ContentFit::Contain)
                        .border_radius([10.0; 4]),
                )
            }
            BackgroundKind::Video => Container::new(Space::new(0, 0))
                .style(|_| {
                    container::background(Background::Color(
                        Color::BLACK,
                    ))
                })
                .width(Length::Fill)
                .height(Length::Fill),
        };
        let delegate = mouse_area(
            Container::new(
                stack!(container.center(Length::Fill), text)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .style(move |t| {
                let mut style = container::Style::default();
                let theme = t.cosmic();
                let hovered = self.hovered_slide == slide_id;
                style.background = Some(Background::Color(
                    if self.current_slide_index as i32 == slide_id {
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
                        if self.current_slide_index as i32 == slide_id
                        {
                            Vector::new(5.0, 5.0)
                        } else if hovered {
                            Vector::new(5.0, 5.0)
                        } else {
                            Vector::new(0.0, 0.0)
                        }
                    },
                    blur_radius: {
                        if self.current_slide_index as i32 == slide_id
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
        .on_enter(Message::HoveredSlide(slide_id))
        .on_exit(Message::HoveredSlide(-1))
        .on_press({
            let id =
                self.slides.iter().position(|s| s == slide).unwrap();
            Message::SlideChange(id as u16)
        });
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
                        let result = Video::new(&url);
                        match result {
                            Ok(v) => self.video = Some(v),
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

async fn start_audio(sink: Arc<Sink>, audio: PathBuf) {
    let file = BufReader::new(File::open(audio).unwrap());
    debug!(?file);
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    let empty = sink.empty();
    let paused = sink.is_paused();
    debug!(empty, paused, "Finished running");
}
