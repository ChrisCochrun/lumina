use cosmic::{
    iced::{widget::text, ContentFit, Length},
    iced_widget::stack,
    prelude::*,
    widget::{image, Container},
    Task,
};
use tracing::debug;

use crate::core::slide::Slide;

#[derive(Default, Clone, Debug)]
pub(crate) struct Presenter {
    slides: Vec<Slide>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Message {
    NextSlide,
    PrevSlide,
    SlideChange(u8),
}

impl Presenter {
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
        // let video = Video::new(&Url::parse("/home/chris/vids/test/camprules2024.mp4").unwrap())
        let text = text!("This is frodo").size(50);
        let text = Container::new(text).center(Length::Fill);
        let image = Container::new(
            image("/home/chris/pics/frodo.jpg")
                .content_fit(ContentFit::Cover)
                .width(Length::Fill)
                .height(Length::Fill),
        );
        // let video = Container::new(VideoPlayer::new(&video))
        //     .width(Length::Fill)
        //     .height(Length::Fill);
        let stack = stack!(image, text).width(Length::Fill).height(Length::Fill);
        stack.into()
    }
}
