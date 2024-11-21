use std::{path::PathBuf, rc::Rc, time::Duration};

use cosmic::{
    dialog::ashpd::url::Url,
    iced::{widget::text, Background, Color, ContentFit, Length},
    iced_widget::stack,
    prelude::*,
    widget::{container, image, Container, Row, Space},
    Task,
};
use iced_video_player::{Position, Video, VideoPlayer};
use miette::{Context, IntoDiagnostic, Result};
use tracing::{debug, error, info};

use crate::{core::slide::Slide, BackgroundKind};

// #[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    pub slides: Vec<Slide>,
    pub current_slide: Slide,
    pub current_slide_index: u16,
    pub video: Option<Video>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(u16),
    EndVideo,
    StartVideo,
    VideoPos(f32),
}

impl Presenter {
    pub fn with_slides(slides: Vec<Slide>) -> Self {
        Self {
            slides: slides.clone(),
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
                self.current_slide_index += 1;
                self.update(Message::SlideChange(
                    self.current_slide_index,
                ))
            }
            Message::PrevSlide => {
                debug!("prev slide");
                if 0 == self.current_slide_index {
                    debug!("beginning slides");
                    return Task::none();
                }
                self.current_slide_index -= 1;
                self.update(Message::SlideChange(
                    self.current_slide_index,
                ))
            }
            Message::SlideChange(id) => {
                debug!(id, "slide changed");
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
        }
    }

    pub fn view(&self) -> Element<Message> {
        let font_size = if self.current_slide.font_size() > 0 {
            self.current_slide.font_size() as u16
        } else {
            50
        };
        let text = text(self.current_slide.text()).size(font_size);
        let text = Container::new(text).center(Length::Fill);
        let black = Container::new(Space::new(0, 0))
            .style(|_| {
                container::background(Background::Color(Color::BLACK))
            })
            .width(Length::Fill)
            .height(Length::Fill);
        let container = match self.current_slide.background().kind {
            crate::BackgroundKind::Image => {
                let path =
                    self.current_slide.background().path.clone();
                Container::new(
                    image(path)
                        .content_fit(ContentFit::Contain)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
            }
            crate::BackgroundKind::Video => {
                if let Some(video) = &self.video {
                    Container::new(
                        VideoPlayer::new(video)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .on_end_of_stream(Message::EndVideo)
                            .content_fit(ContentFit::Cover),
                    )
                } else {
                    Container::new(Space::new(0, 0))
                }
            }
        };
        stack!(black, container.center(Length::Fill), text)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn slide_preview(&self) -> Element<Message> {
        let mut items = vec![];
        for slide in self.slides.iter() {
            items.push(Self::slide_delegate(slide));
        }
        Row::from_vec(items).spacing(10).padding(10).into()
    }

    fn slide_delegate(slide: &Slide) -> Element<Message> {
        let font_size = if slide.font_size() > 0 {
            slide.font_size() as u16
        } else {
            50
        };
        let text = text(slide.text()).size(font_size);
        let text = Container::new(text).center(Length::Fill);
        let container = match slide.background().kind {
            crate::BackgroundKind::Image => Container::new(
                image("/home/chris/pics/frodo.jpg")
                    .content_fit(ContentFit::Cover)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ),
            crate::BackgroundKind::Video => {
                Container::new(Space::new(Length::Fill, Length::Fill))
            }
        };
        stack!(container.center(Length::Fill), text)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
