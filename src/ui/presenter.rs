use cosmic::{
    dialog::ashpd::url::Url,
    iced::{widget::text, ContentFit, Length},
    iced_widget::stack,
    prelude::*,
    widget::{image, Container},
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
    video: Video,
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
                let url = Url::from_file_path("/home/chris/vids/test/camprules2024.mp4").unwrap();
                let result = Video::new(&url);
                match result {
                    Ok(v) => v,
                    Err(e) => {
                        // let root = e.source();
                        panic!("Error here: {e}")
                    }
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
        let video = Container::new(
            VideoPlayer::new(&self.video)
                .width(Length::Fill)
                .height(Length::Fill),
        );
        let stack = stack!(image, video, text)
            .width(Length::Fill)
            .height(Length::Fill);
        stack.into()
    }
}
