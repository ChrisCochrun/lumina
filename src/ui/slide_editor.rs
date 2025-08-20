use std::{io, path::PathBuf};

use cosmic::{
    iced::{Color, Font},
    widget::{
        self,
        canvas::{self, Program, Stroke},
    },
    Renderer,
};

#[derive(Debug, Default)]
struct State {
    cache: canvas::Cache,
}

#[derive(Debug)]
struct SlideEditor<'a> {
    state: &'a State,
    font: &'a Font,
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

impl State {
    pub fn view<'a>(
        &'a self,
        font: &'a Font,
    ) -> cosmic::iced::Element<'a, SlideWidget> {
        widget::canvas(SlideEditor { state: self, font }).into()
    }
}

impl<'a> Program<SlideWidget> for SlideEditor<'a> {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &cosmic::iced_runtime::core::Theme,
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
                cosmic::iced::mouse::Event::CursorEntered => todo!(),
                cosmic::iced::mouse::Event::CursorLeft => todo!(),
                cosmic::iced::mouse::Event::CursorMoved {
                    position,
                } => todo!(),
                cosmic::iced::mouse::Event::ButtonPressed(button) => {
                    todo!()
                }
                cosmic::iced::mouse::Event::ButtonReleased(
                    button,
                ) => todo!(),
                cosmic::iced::mouse::Event::WheelScrolled {
                    delta,
                } => todo!(),
            },
            canvas::Event::Touch(event) => todo!(),
            canvas::Event::Keyboard(event) => todo!(),
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
