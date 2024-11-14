use clap::{command, Parser};
use cosmic::app::{Core, Settings, Task};
use cosmic::iced::keyboard::Key;
use cosmic::iced::window::Position;
use cosmic::iced::{
    self, event, window, ContentFit, Font, Length, Point,
};
use cosmic::iced_core::SmolStr;
use cosmic::iced_widget::{row, stack};
use cosmic::widget::icon;
use cosmic::widget::tooltip::Position as TPosition;
use cosmic::widget::{button, image, nav_bar, text, tooltip, Space};
use cosmic::{executor, Application, ApplicationExt, Element};
use cosmic::{widget::Container, Theme};
use iced_video_player::VideoPlayer;
use miette::{miette, Result};
use std::path::PathBuf;
use tracing::{debug, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

pub mod core;
pub mod ui;
use core::slide::*;
use ui::presenter::{self, Presenter};

#[derive(Debug, Parser)]
#[command(name = "lumina", version, about)]
struct Cli {
    #[arg(short, long)]
    watch: bool,
    #[arg(short = 'i', long)]
    ui: bool,
    file: PathBuf,
}

fn main() -> Result<()> {
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new(
        "%Y-%m-%d_%I:%M:%S%.6f %P".to_owned(),
    );
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .parse_lossy("lumina=debug");
    tracing_subscriber::FmtSubscriber::builder()
        .pretty()
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_env_filter(filter)
        .with_target(true)
        .with_timer(timer)
        .init();

    let args = Cli::parse();

    let settings;
    if args.ui {
        settings = Settings::default().debug(false);
    } else {
        settings =
            Settings::default().debug(false).no_main_window(true);
    }

    Ok(cosmic::app::run::<App>(settings, args)
        .map_err(|e| miette!("Invalid things... {}", e))?)
}

fn theme(_state: &App) -> Theme {
    Theme::dark()
}

struct App<'a> {
    core: Core,
    nav_model: nav_bar::Model,
    file: PathBuf,
    presenter: Presenter<'a>,
    windows: Vec<window::Id>,
    slides: Vec<Slide>,
}

impl Default for App<'_> {
    fn default() -> Self {
        let initial_slide = SlideBuilder::new()
            .background(PathBuf::from(
                "/home/chris/vids/test/chosensmol.mp4",
            ))
            .expect("oops video")
            .text("Hello")
            .build()
            .expect("oops slide");
        let slides = vec![initial_slide];
        let presenter = Presenter::with_app_slides(&slides);
        Self {
            presenter,
            core: Core::default(),
            nav_model: nav_bar::Model::default(),
            file: PathBuf::default(),
            windows: vec![],
            slides: vec![initial_slide],
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Present(presenter::Message),
    File(PathBuf),
    OpenWindow,
    CloseWindow(window::Id),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = Cli;
    type Message = Message;
    const APP_ID: &'static str = "lumina";
    fn core(&self) -> &Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }
    fn init(
        core: Core,
        input: Self::Flags,
    ) -> (Self, Task<Self::Message>) {
        debug!("init");
        let mut nav_model = nav_bar::Model::default();

        nav_model.insert().text("Preview").data("Preview");

        nav_model.activate_position(0);
        let mut windows = vec![];

        if input.ui {
            windows.push(core.main_window_id().unwrap());
        }
        let initial_slide = SlideBuilder::new()
            .background(
                PathBuf::from(
                    "/home/chris/vids/test/camprules2024.mp4",
                )
                .canonicalize()
                .unwrap(),
            )
            .expect("oops video")
            .text("Hello")
            .font("Quicksand")
            .font_size(50)
            .text_alignment(TextAlignment::MiddleCenter)
            .video_loop(true)
            .video_start_time(0.0)
            .video_end_time(0.0)
            .build()
            .expect("oops slide");
        let presenter =
            Presenter::with_initial_slide(initial_slide.clone());

        let mut app = App {
            core,
            nav_model,
            file: input.file,
            windows,
            presenter,
            slides: vec![initial_slide],
        };

        let command;
        if input.ui {
            command = app.update_title()
        } else {
            command = app.show_window()
        };

        (app, command)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(
        &mut self,
        id: nav_bar::Id,
    ) -> Task<Self::Message> {
        self.nav_model.activate(id);
        self.update_title()
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }
    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }
    fn header_end(&self) -> Vec<Element<Self::Message>> {
        let window_open = self.windows.len() > 1;
        let presenter_window = self.windows.get(1);
        vec![tooltip(
            button::custom(
                row!(
                    Container::new(
                        icon::from_name(if window_open {
                            "dialog-close"
                        } else {
                            "view-presentation-symbolic"
                        })
                        .scale(3)
                    )
                    .center_y(Length::Fill),
                    text::body(if window_open {
                        "Close Presentation"
                    } else {
                        "Open Presentation"
                    })
                )
                .padding(5)
                .spacing(5),
            )
            .class(cosmic::theme::style::Button::HeaderBar)
            .on_press(if window_open {
                Message::CloseWindow(*presenter_window.unwrap())
            } else {
                Message::OpenWindow
            }),
            "Open Window",
            TPosition::Bottom,
        )
        .into()]
    }

    fn footer(&self) -> Option<Element<Self::Message>> {
        Some(text::body("Sux").into())
    }

    fn subscription(
        &self,
    ) -> cosmic::iced_futures::Subscription<Self::Message> {
        event::listen_with(|event, _, id| {
            if let iced::Event::Window(window_event) = event {
                match window_event {
                    window::Event::CloseRequested => {
                        debug!("Closing window");
                        Some(Message::CloseWindow(id))
                    }
                    window::Event::Opened { position, .. } => {
                        debug!(?window_event, ?id);
                        Some(Message::WindowOpened(id, position))
                    }
                    window::Event::Closed => {
                        debug!("Closed window");
                        Some(Message::WindowClosed(id))
                    }
                    _ => None,
                }
            } else if let iced::Event::Keyboard(key) = event {
                match key {
                    iced::keyboard::Event::KeyReleased {
                        key,
                        modified_key: _,
                        physical_key: _,
                        location: _,
                        modifiers: _,
                    } => match key {
                        Key::Named(
                            iced::keyboard::key::Named::ArrowRight,
                        ) => Some(Message::Present(
                            ui::presenter::Message::NextSlide,
                        )),
                        Key::Named(
                            iced::keyboard::key::Named::ArrowLeft,
                        ) => Some(Message::Present(
                            ui::presenter::Message::NextSlide,
                        )),
                        Key::Named(
                            iced::keyboard::key::Named::Space,
                        ) => Some(Message::Present(
                            ui::presenter::Message::NextSlide,
                        )),
                        Key::Character(key) => {
                            if key == SmolStr::from("j")
                                || key == SmolStr::from("l")
                            {
                                Some(Message::Present(
                                    ui::presenter::Message::NextSlide,
                                ))
                            } else if key == SmolStr::from("k")
                                || key == SmolStr::from("h")
                            {
                                Some(Message::Present(
                                    ui::presenter::Message::PrevSlide,
                                ))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    },
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    fn update(
        &mut self,
        message: Message,
    ) -> cosmic::Task<cosmic::app::Message<Message>> {
        match message {
            Message::Present(message) => {
                self.presenter.update(message);
                // self.core.nav_bar_toggle();
                Task::none()
            }
            Message::File(file) => {
                self.file = file;
                Task::none()
            }
            Message::OpenWindow => {
                let count = self.windows.len() + 1;

                let (id, spawn_window) =
                    window::open(window::Settings {
                        position: Position::Centered,
                        exit_on_close_request: count % 2 == 0,
                        decorations: false,
                        ..Default::default()
                    });

                self.windows.push(id);
                _ = self.set_window_title(
                    format!("window_{}", count),
                    id,
                );

                spawn_window.map(|id| {
                    cosmic::app::Message::App(Message::WindowOpened(
                        id, None,
                    ))
                })
            }
            Message::CloseWindow(id) => window::close(id),
            Message::WindowOpened(id, _) => {
                debug!(?id, "Window opened");
                Task::none()
            }
            Message::WindowClosed(id) => {
                let window = self
                    .windows
                    .iter()
                    .position(|w| *w == id)
                    .unwrap();
                self.windows.remove(window);
                // This closes the app if using the cli example
                if self.windows.len() == 0 {
                    cosmic::iced::exit()
                } else {
                    Task::none()
                }
            }
        }
    }

    // Main window view
    fn view(&self) -> Element<Message> {
        let text = text::body("This is frodo").size(20);
        let text = Container::new(text).center(Length::Fill);
        let slide = self
            .presenter
            .slides
            .get(self.presenter.current_slide as usize)
            .unwrap();
        let container = match slide.background().kind {
            crate::BackgroundKind::Image => Container::new(
                image("/home/chris/pics/frodo.jpg")
                    .content_fit(ContentFit::Cover)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ),
            crate::BackgroundKind::Video => {
                if let Some(video) = &self.presenter.video {
                    Container::new(
                        VideoPlayer::new(&video)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .content_fit(ContentFit::Contain),
                    )
                    .center(Length::Fill)
                } else {
                    Container::new(Space::new(0, 0))
                }
            }
        };
        let preview = stack!(container, text);
        let icon_left = icon::from_name("arrow-left");
        let icon_right = icon::from_name("arrow-right");
        let row = row![
            Container::new(
                button::icon(icon_left)
                    .icon_size(128)
                    .tooltip("Previous Slide")
                    .width(128)
                    .on_press(Message::Present(
                        presenter::Message::PrevSlide
                    ))
            )
            .center_y(Length::Fill)
            .align_right(Length::Fill)
            .width(Length::FillPortion(1)),
            preview.width(Length::FillPortion(3)),
            Container::new(
                button::icon(icon_right)
                    .icon_size(128)
                    .tooltip("Next Slide")
                    .width(128)
                    .on_press(Message::Present(
                        presenter::Message::NextSlide
                    ))
            )
            .center_y(Length::Fill)
            .align_left(Length::Fill)
            .width(Length::FillPortion(1)),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(20);
        row.into()
    }

    // View for presentation
    fn view_window(&self, _id: window::Id) -> Element<Message> {
        self.presenter
            .view()
            .map(|message| Message::Present(message))
    }
}

impl App
where
    Self: cosmic::Application,
{
    fn active_page_title(&mut self) -> &str {
        // self.nav_model
        //     .text(self.nav_model.active())
        //     .unwrap_or("Unknown Page")
        "Lumina"
    }

    fn update_title(&mut self) -> Task<Message> {
        let header_title = self.active_page_title().to_owned();
        let window_title = format!("{header_title} — Lumina");
        // self.set_header_title(header_title);
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    fn show_window(&mut self) -> Task<Message> {
        let (id, spawn_window) = window::open(window::Settings {
            position: Position::Centered,
            exit_on_close_request: true,
            decorations: false,
            ..Default::default()
        });
        self.windows.push(id);
        _ = self.set_window_title("Lumina Presenter".to_owned(), id);
        spawn_window.map(|id| {
            cosmic::app::Message::App(Message::WindowOpened(id, None))
        })
    }
}

fn test_slide<'a>() -> Element<'a, Message> {
    if let Ok(slide) = SlideBuilder::new()
        .background(PathBuf::from("/home/chris/pics/frodo.jpg"))
        .unwrap()
        .text("This is a frodo")
        .text_alignment(TextAlignment::TopCenter)
        .font_size(50)
        .font("Quicksand")
        .build()
    {
        let font = Font::with_name("Noto Sans");
        let stack = stack!(
            image(slide.background().path.clone()),
            text(slide.text())
                .size(slide.font_size() as u16)
                .font(font)
        );

        stack.into()
    } else {
        text("Slide is broken").into()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    fn test_slide() -> String {
        let slide = r#"(slide (image :source "./somehting.jpg" :fill cover
              (text "Something cooler" :font-size 50)))"#;
        String::from(slide)
    }
    // #[test]
    // fn test_lisp() {
    //     let slide = test_slide();
    //     if let Ok(data) = lexpr::parse::from_str_elisp(slide.as_str()) {
    //         println!("{data:?}");
    //         assert_eq!(slide, data)
    //     } else {
    //         assert!(false)
    //     }
    // }
}
