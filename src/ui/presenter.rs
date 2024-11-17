use std::{path::PathBuf, rc::Rc};

use cosmic::{
    dialog::ashpd::url::Url,
    iced::{widget::text, ContentFit, Length},
    iced_widget::{row, stack, Stack},
    prelude::*,
    widget::{
        button, container, icon::Named, image, Container, Row, Space,
    },
    Task,
};
use iced_video_player::{Video, VideoPlayer};
use miette::{Context, IntoDiagnostic, Result};
use tracing::{debug, error};

use crate::{core::slide::Slide, BackgroundKind};

// #[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    pub slides: Vec<Slide>,
    pub current_slide: i16,
    pub video: Option<Video>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(u8),
}

impl Presenter {
    pub fn with_app_slides(slides: Vec<Slide>) -> Self {
        Self {
            slides: slides.clone(),
            current_slide: 0,
            video: {
                if let Some(slide) = slides.get(0) {
                    let path = slide.background().path.clone();
                    if path.exists() {
                        let url = Url::from_file_path(path).unwrap();
                        let result = Video::new(&url);
                        match result {
                            Ok(v) => Some(v),
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
    // pub fn with_initial_slide(slide: Slide) -> Self {
    //     Self {
    //         slides: vec![slide.clone()],
    //         current_slide: 0,
    //         video: {
    //             let path = slide.background().path.clone();
    //             if path.exists() {
    //                 let url = Url::from_file_path(path).unwrap();
    //                 let result = Video::new(&url);
    //                 match result {
    //                     Ok(v) => Some(v),
    //                     Err(e) => {
    //                         error!("Had an error creating the video object: {e}");
    //                         None
    //                     }
    //                 }
    //             } else {
    //                 error!("File doesn't exist: ");
    //                 None
    //             }
    //         },
    //     }
    // }

    pub fn update(
        &mut self,
        message: Message,
    ) -> Task<cosmic::app::Message<Message>> {
        match message {
            Message::NextSlide => {
                debug!("next slide");
                if let Some(video) = &mut self.video {
                    let _ = video.restart_stream();
                }
                self.current_slide += 1;
                self.reset_video();
                Task::none()
            }
            Message::PrevSlide => {
                debug!("prev slide");
                if let Some(video) = &mut self.video {
                    let _ = video.restart_stream();
                }
                self.current_slide -= 1;
                self.reset_video();
                Task::none()
            }
            Message::SlideChange(id) => {
                debug!(id, "slide changed");
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let text = text!("This is frodo").size(50);
        let text = Container::new(text).center(Length::Fill);
        let container = match self
            .slides
            .get(self.current_slide as usize)
            .unwrap()
            .background()
            .kind
        {
            crate::BackgroundKind::Image => Container::new(
                image("/home/chris/pics/frodo.jpg")
                    .content_fit(ContentFit::Cover)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ),
            crate::BackgroundKind::Video => {
                if let Some(video) = &self.video {
                    Container::new(
                        VideoPlayer::new(video)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .content_fit(ContentFit::Cover),
                    )
                } else {
                    Container::new(Space::new(0, 0))
                }
            }
        };
        stack!(container, text)
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
        let text =
            text!("{}", slide.text()).size(slide.font_size() as u16);
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
        stack!(container, text)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn reset_video(&mut self) {
        if let Some(slide) =
            self.slides.get(self.current_slide as usize)
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

pub(crate) fn slide_view<'a>(
    slide: &'a Slide,
    video: &'a Option<Video>,
) -> Stack<'a, crate::Message, cosmic::Theme> {
    let text = text!("{}", slide.text()).size(50);
    let text = Container::new(text).center(Length::Fill);
    let container = match slide.background().kind {
        crate::BackgroundKind::Image => Container::new(
            image("/home/chris/pics/frodo.jpg")
                .content_fit(ContentFit::Cover)
                .width(Length::Fill)
                .height(Length::Fill),
        ),
        crate::BackgroundKind::Video => {
            if let Some(video) = video {
                Container::new(
                    VideoPlayer::new(video)
                        .width(Length::Shrink)
                        .height(Length::Fill)
                        .content_fit(ContentFit::Cover),
                )
            } else {
                Container::new(Space::new(Length::Fill, Length::Fill))
            }
        }
    };
    stack!(container, text)
    // .width(Length::Fill)
    // .height(Length::Fill)
}
