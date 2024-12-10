use std::{rc::Rc, time::Duration};

use cosmic::{
    dialog::ashpd::url::Url,
    iced::{
        font::{Family, Stretch, Style, Weight},
        Background, Border, Color, ContentFit, Font, Length, Shadow,
        Vector,
    },
    iced_widget::{
        scrollable::{Direction, Scrollbar},
        stack,
    },
    prelude::*,
    widget::{
        container, image, mouse_area, responsive, scrollable, text,
        Container, Responsive, Row, Space,
    },
    Task,
};
use iced_video_player::{Position, Video, VideoPlayer};
use miette::{Context, IntoDiagnostic, Result};
use tracing::{debug, error, info};

use crate::{
    core::{
        service_items::{ServiceItem, ServiceItemModel},
        slide::Slide,
    },
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
    hovered_slide: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(u16),
    EndVideo,
    StartVideo,
    VideoPos(f32),
    VideoFrame,
    HoveredSlide(i32),
}

impl Presenter {
    pub fn with_items(items: ServiceItemModel) -> Self {
        let slides = if let Ok(slides) = items.to_slides() {
            slides
        } else {
            vec![]
        };
        Self {
            slides: slides.clone(),
            items: items.into(),
            current_slide: slides[0].clone(),
            current_slide_index: 0,
            video: {
                if let Some(slide) = slides.get(0) {
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
            video_position: 0.0,
            hovered_slide: -1,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
    ) -> Task<cosmic::app::Message<Message>> {
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
                }
                if let Some(video) = &mut self.video {
                    let _ = video.restart_stream();
                }
                self.reset_video();
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
                        Duration::from_secs_f32(position),
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
        }
    }

    pub fn view(&self) -> Element<Message> {
        responsive(|size| {
            let family = Family::Name("VictorMono Nerd Font");
            let weight = Weight::Normal;
            let stretch = Stretch::Normal;
            let style = Style::Normal;
            let font = Font {
                family,
                weight,
                stretch,
                style,
            };
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

    pub fn slide_preview(&self) -> Element<Message> {
        let mut items = vec![];
        for slide in self.slides.iter() {
            items.push(self.slide_delegate(slide));
        }
        let row =
            scrollable(Row::from_vec(items).spacing(10).padding(15))
                .direction(Direction::Horizontal(Scrollbar::new()))
                .height(Length::Fill)
                .width(Length::Fill);
        row.into()
    }

    fn slide_delegate<'a>(
        &'a self,
        slide: &'a Slide,
    ) -> Element<'a, Message> {
        let family = Family::Name("VictorMono Nerd Font");
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
                        if hovered {
                            Vector::new(5.0, 5.0)
                        } else {
                            Vector::new(0.0, 0.0)
                        }
                    },
                    blur_radius: {
                        if hovered {
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
