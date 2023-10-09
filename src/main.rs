use iced::widget::{button, column, row, Button, Column, Container, Row, Text};
use iced::{executor, Alignment, Length, Rectangle, Renderer};
use iced::{Application, Command, Element, Settings, Theme};
use iced_aw::native::TabBar;
use iced_aw::split::Axis;
use iced_aw::{split, Split};

fn main() -> iced::Result {
    std::env::set_var("WINIT_UNIX_BACKEND", "wayland");
    App::run(Settings::default())
}

struct App {
    divider_position: Option<u16>,
}

#[derive(Debug, Clone)]
enum Message {
    OnVerResize(u16),
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        (
            App {
                divider_position: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("A cool application")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OnVerResize(position) => self.divider_position = Some(position),
        };
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let top: _ = Container::new(
            row(vec![
                Button::<Message, Renderer>::new("Edit")
                    .height(Length::Fill)
                    .padding(10)
                    .into(),
                Button::new("Present")
                    .height(Length::Fill)
                    .padding(10)
                    .into(),
                Button::new("Close Library")
                    .height(Length::Fill)
                    .padding(10)
                    .into(),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::End),
        );
        let first = Container::new(Text::new("First"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();
        let second = Container::new(Text::new("Second"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();
        Split::new(
            first,
            second,
            self.divider_position,
            Axis::Vertical,
            Message::OnVerResize,
        )
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
