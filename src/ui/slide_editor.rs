use std::{io, path::PathBuf};

use cosmic::{
    iced::{Color, Font, Length},
    prelude::*,
    widget::{
        self,
        canvas::{self, Program, Stroke},
        container, Canvas,
    },
    Renderer,
};
use tracing::debug;

#[derive(Debug, Default)]
struct State {
    cache: canvas::Cache,
}

#[derive(Debug, Default)]
pub struct SlideEditor {
    state: State,
    font: Font,
    program: EditorProgram,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeFont(String),
    ChangeFontSize(usize),
    ChangeTitle(String),
    ChangeBackground(Result<PathBuf, SlideError>),
    PickBackground,
    Edit(bool),
    None,
}

pub struct Text {
    text: String,
}

pub enum SlideWidget {
    Text(Text),
}

#[derive(Debug, Clone)]
pub enum SlideError {
    DialogClosed,
    IOError(io::ErrorKind),
}

#[derive(Debug, Default)]
struct EditorProgram {}

impl SlideEditor {
    pub fn view<'a>(
        &'a self,
        font: Font,
    ) -> cosmic::Element<'a, SlideWidget> {
        container(
            widget::canvas(&self.program)
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .into()
    }
}

/// Ensure to use the `cosmic::Theme and cosmic::Renderer` here
/// or else it will not compile
impl<'a> Program<SlideWidget, cosmic::Theme, cosmic::Renderer>
    for EditorProgram
{
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &cosmic::Theme,
        bounds: cosmic::iced::Rectangle,
        cursor: cosmic::iced_core::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // We create a `Path` representing a simple circle
        let circle = canvas::Path::circle(frame.center(), 50.0);

        // And fill it with some color
        frame.fill(&circle, Color::BLACK);
        frame.stroke(
            &circle,
            Stroke::default()
                .with_width(5.0)
                .with_color(Color::BLACK),
        );

        // Then, we produce the geometry
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        _bounds: cosmic::iced::Rectangle,
        _cursor: cosmic::iced_core::mouse::Cursor,
    ) -> (canvas::event::Status, Option<SlideWidget>) {
        match event {
            canvas::Event::Mouse(event) => match event {
                cosmic::iced::mouse::Event::CursorEntered => {
                    debug!("cursor entered")
                }
                cosmic::iced::mouse::Event::CursorLeft => {
                    debug!("cursor left")
                }
                cosmic::iced::mouse::Event::CursorMoved {
                    position,
                } => debug!(?position, "cursor moved"),
                cosmic::iced::mouse::Event::ButtonPressed(button) => {
                    debug!(?button, "mouse button pressed")
                }
                cosmic::iced::mouse::Event::ButtonReleased(
                    button,
                ) => debug!(?button, "mouse button released"),
                cosmic::iced::mouse::Event::WheelScrolled {
                    delta,
                } => debug!(?delta, "scroll wheel"),
            },
            canvas::Event::Touch(event) => debug!("test"),
            canvas::Event::Keyboard(event) => debug!("test"),
        }
        (canvas::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: cosmic::iced::Rectangle,
        _cursor: cosmic::iced_core::mouse::Cursor,
    ) -> cosmic::iced_core::mouse::Interaction {
        cosmic::iced_core::mouse::Interaction::default()
    }
}
