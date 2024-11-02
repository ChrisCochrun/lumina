use std::path::PathBuf;

use cosmic::{
    dialog::ashpd::url::Url,
    iced::{widget::text, ContentFit, Length},
    iced_widget::stack,
    prelude::*,
    widget::{image, Container, Space},
    Task,
};
use iced_video_player::{Video, VideoPlayer};
use miette::{Context, IntoDiagnostic};
use tracing::{debug, error};

use crate::core::slide::Slide;

// #[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    slides: Vec<Slide>,
    current_slide: i16,
    video: Option<Video>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(u8),
}

impl Presenter {
    pub fn with_initial_slide(slide: Slide) -> Self {
        Self {
            slides: vec![slide],
            current_slide: 0,
            video: {
                if let Ok(path) =
                    PathBuf::from("/home/chris/vids/test/chosensmol.mp4").canonicalize()
                {
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
                    error!("File doesn't exist: ");
                    None
                }
            },
        }
    }

    pub fn update(&mut self, message: Message) -> Task<cosmic::app::Message<Message>> {
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
        // let window = self.windows.iter().position(|w| *w == id).unwrap();
        // if let Some(_window) = self.windows.get(window) {}
        let text = text!("This is frodo").size(50);
        let text = Container::new(text).center(Length::Fill);
        let image = Container::new(
            image("/home/chris/pics/frodo.jpg")
                .content_fit(ContentFit::Cover)
                .width(Length::Fill)
                .height(Length::Fill),
        );
        let vid_container;
        if let Some(video) = &self.video {
            vid_container = Container::new(
                VideoPlayer::new(video)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Cover),
            );
        } else {
            vid_container = Container::new(Space::new(0, 0));
        }
        let stack = stack!(image, vid_container, text)
            .width(Length::Fill)
            .height(Length::Fill);
        stack.into()
    }
}
