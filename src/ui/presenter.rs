use std::{path::PathBuf, rc::Rc};

use cosmic::{
    dialog::ashpd::url::Url,
    iced::{widget::text, ContentFit, Length},
    iced_widget::{row, stack},
    prelude::*,
    widget::{
        button, container, icon::Named, image, Container, Space,
    },
    Task,
};
use iced_video_player::{Video, VideoPlayer};
use miette::{Context, IntoDiagnostic, Result};
use tracing::{debug, error};

use crate::core::slide::Slide;

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
                Task::none()
            }
            Message::PrevSlide => {
                debug!("prev slide");
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
        let stack = stack!(container, text)
            .width(Length::Fill)
            .height(Length::Fill);
        stack.into()
    }
}

pub(crate) fn slide_view<'a>(
    slide: &'a Slide,
    video: &'a Option<Video>,
) -> Element<'a, Message> {
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
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(ContentFit::Cover),
                )
            } else {
                Container::new(Space::new(Length::Fill, Length::Fill))
            }
        }
    };
    let stack = stack!(container, text)
        .width(Length::Fill)
        .height(Length::Fill);
    stack.into()
}
