use std::{io, path::PathBuf};

use iced::{
    widget::{
        self,
        canvas::{self, Program, Stroke},
        container, Canvas,
    },
    Color, Font, Length, Renderer, Size,
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
struct EditorProgram {
    mouse_button_pressed: Option<iced::mouse::Button>,
}

impl SlideEditor {
    pub fn view<'a>(
        &'a self,
        font: Font,
    ) -> iced::Element<'a, SlideWidget> {
        container(
            widget::canvas(&self.program)
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .into()
    }
}

/// Ensure to use the `iced::Theme and iced::Renderer` here
/// or else it will not compile
impl<'a> Program<SlideWidget, iced::Theme, iced::Renderer>
    for EditorProgram
{
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let frame_rect = bounds.shrink(10);

        // We create a `Path` representing a simple circle
        let circle = canvas::Path::circle(frame.center(), 50.0);
        let border = canvas::Path::rectangle(
            iced::Point { x: 10.0, y: 10.0 },
            Size::new(frame_rect.width, frame_rect.height),
        );

        // And fill it with some color
        frame.fill(&circle, Color::BLACK);
        frame.stroke(
            &circle,
            Stroke::default()
                .with_width(5.0)
                .with_color(Color::BLACK),
        );
        frame.stroke(
            &border,
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
        event: &iced::Event,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> std::option::Option<iced::widget::Action<SlideWidget>> {
        match event {
            iced::Event::Mouse(event) => match event {
                iced::mouse::Event::CursorEntered => {
                    debug!("cursor entered")
                }
                iced::mouse::Event::CursorLeft => {
                    debug!("cursor left")
                }
                iced::mouse::Event::CursorMoved { position } => {
                    if bounds.x < position.x
                        && bounds.y < position.y
                        && (bounds.width + bounds.x) > position.x
                        && (bounds.height + bounds.y) > position.y
                    {
                        debug!(?position, "cursor moved");
                    }
                }
                iced::mouse::Event::ButtonPressed(button) => {
                    // self.mouse_button_pressed = Some(button);
                    debug!(?button, "mouse button pressed")
                }
                iced::mouse::Event::ButtonReleased(button) => {
                    debug!(?button, "mouse button released")
                }
                iced::mouse::Event::WheelScrolled { delta } => {
                    debug!(?delta, "scroll wheel")
                }
            },
            iced::Event::Touch(event) => debug!("test"),
            iced::Event::Keyboard(event) => debug!("test"),
            iced::Event::Keyboard(event) => todo!(),
            iced::Event::Mouse(event) => todo!(),
            iced::Event::Window(event) => todo!(),
            iced::Event::Touch(event) => todo!(),
            iced::Event::InputMethod(event) => todo!(),
        }
        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        iced::mouse::Interaction::default()
    }
}
