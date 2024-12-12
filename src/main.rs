use clap::{command, Parser};
use core::service_items::{ServiceItem, ServiceItemModel};
use cosmic::app::{Core, Settings, Task};
use cosmic::iced::keyboard::Key;
use cosmic::iced::window::{Mode, Position};
use cosmic::iced::{self, event, window, Font, Length, Point};
use cosmic::iced_core::SmolStr;
use cosmic::iced_widget::{column, row, stack};
use cosmic::prelude::ElementExt;
use cosmic::prelude::*;
use cosmic::widget::aspect_ratio::aspect_ratio_container;
use cosmic::widget::tooltip::Position as TPosition;
use cosmic::widget::{
    button, image, nav_bar, text, tooltip, Responsive, Space,
};
use cosmic::widget::{icon, slider};
use cosmic::{executor, Application, ApplicationExt, Element};
use cosmic::{widget::Container, Theme};
use crisp::types::Value;
use lisp::parse_lisp;
use miette::{miette, Result};
use std::fs::read_to_string;
use std::path::PathBuf;
use tracing::{debug, level_filters::LevelFilter};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

pub mod core;
pub mod lisp;
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
        debug!("main view");
        settings = Settings::default().debug(false);
    } else {
        debug!("window view");
        settings =
            Settings::default().debug(false).no_main_window(true);
    }

    Ok(cosmic::app::run::<App>(settings, args)
        .map_err(|e| miette!("Invalid things... {}", e))?)
}

fn theme(_state: &App) -> Theme {
    Theme::dark()
}

struct App {
    core: Core,
    nav_model: nav_bar::Model,
    file: PathBuf,
    presenter: Presenter,
    windows: Vec<window::Id>,
    slides: Vec<Slide>,
    current_slide: Slide,
    presentation_open: bool,
    cli_mode: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Present(presenter::Message),
    File(PathBuf),
    DndEnter(ServiceItem),
    DndDrop(ServiceItem),
    OpenWindow,
    CloseWindow(Option<window::Id>),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
    Quit,
    None,
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

        let mut windows = vec![];

        if input.ui {
            windows.push(core.main_window_id().unwrap());
        }

        let items = match read_to_string(input.file) {
            Ok(lisp) => {
                let mut slide_vector = vec![];
                let lisp = crisp::reader::read(&lisp);
                match lisp {
                    Value::List(vec) => {
                        for value in vec {
                            let mut inner_vector = parse_lisp(value);
                            slide_vector.append(&mut inner_vector);
                        }
                    }
                    _ => todo!(),
                }
                slide_vector
            }
            Err(e) => {
                warn!("Missing file or could not read: {e}");
                vec![]
            }
        };

        let items = ServiceItemModel::from(items);
        let presenter = Presenter::with_items(items.clone());
        let slides = if let Ok(slides) = items.to_slides() {
            slides
        } else {
            vec![]
        };
        let current_slide = slides[0].clone();

        for item in items.iter() {
            nav_model.insert().text(item.title()).data(item.clone());
        }

        nav_model.activate_position(0);

        let mut app = App {
            presenter,
            core,
            nav_model,
            file: PathBuf::default(),
            windows,
            slides,
            current_slide,
            presentation_open: false,
            cli_mode: !input.ui,
        };

        let command;
        if input.ui {
            debug!("main view");
            command = app.update_title()
        } else {
            debug!("window view");
            command = app.show_window()
        };

        (app, command)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    // fn nav_bar(
    //     &self,
    // ) -> Option<Element<cosmic::app::Message<Message>>> {
    //     if !self.core().nav_bar_active() {
    //         return None;
    //     }

    //     let nav_model = self.nav_model()?;

    //     let mut nav = cosmic::widget::nav_bar(nav_model, |id| {
    //         cosmic::app::Message::Cosmic(cosmic::Message::NavBar(id))
    //     })
    //     .on_context(|id| {
    //         Message::Cosmic(cosmic::Message::NavBarContext(id))
    //     })
    //     .context_menu(
    //         self.nav_context_menu(self.core.nav_bar_context()),
    //     )
    //     .into_container()
    //     // XXX both must be shrink to avoid flex layout from ignoring it
    //     .width(iced::Length::Shrink)
    //     .height(iced::Length::Shrink);

    //     if !self.core().is_condensed() {
    //         nav = nav.max_width(280);
    //     }
    //     Some(nav.into())
    // }

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
        let presenter_window = self.windows.get(1);
        let text = if self.presentation_open {
            text::body("Close Presentation")
        } else {
            text::body("Open Presentation")
        };
        vec![tooltip(
            button::custom(
                row!(
                    Container::new(
                        icon::from_name(if self.presentation_open {
                            "dialog-close"
                        } else {
                            "view-presentation-symbolic"
                        })
                        .scale(3)
                    )
                    .center_y(Length::Fill),
                    text
                )
                .padding(5)
                .spacing(5),
            )
            .class(cosmic::theme::style::Button::HeaderBar)
            .on_press({
                if self.presentation_open {
                    Message::CloseWindow(
                        presenter_window.map(|id| *id),
                    )
                } else {
                    Message::OpenWindow
                }
            }),
            "Start Presentation",
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
                        Some(Message::CloseWindow(Some(id)))
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
                            } else if key == SmolStr::from("q") {
                                Some(Message::Quit)
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
                debug!(?message);
                let task = self.presenter.update(message);
                debug!("Past");
                if self.presentation_open {
                    if let Some(video) = &mut self.presenter.video {
                        video.set_muted(false);
                    }
                }
                // self.core.nav_bar_toggle();
                task.then(|x| {
                    debug!(?x);
                    Task::none()
                })
                // task.map(|x| {
                //     debug!(?x);
                //     cosmic::app::Message::App(Message::None)
                // })
                // Task::batch([task])
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
            Message::CloseWindow(id) => {
                if let Some(id) = id {
                    window::close(id)
                } else {
                    Task::none()
                }
            }
            Message::WindowOpened(id, _) => {
                debug!(?id, "Window opened");
                if self.cli_mode
                    || id > self.core.main_window_id().unwrap()
                {
                    self.presentation_open = true;
                    if let Some(video) = &mut self.presenter.video {
                        video.set_muted(false);
                    }
                    warn!(self.presentation_open);
                    window::change_mode(id, Mode::Fullscreen)
                } else {
                    Task::none()
                }
            }
            Message::WindowClosed(id) => {
                warn!("Closing window: {id}");
                let Some(window) =
                    self.windows.iter().position(|w| *w == id)
                else {
                    error!("Nothing matches this window id: {id}");
                    return Task::none();
                };
                self.windows.remove(window);
                // This closes the app if using the cli example
                if self.windows.len() == 0 {
                    self.update(Message::Quit)
                } else {
                    self.presentation_open = false;
                    if let Some(video) = &mut self.presenter.video {
                        video.set_muted(true);
                    }
                    Task::none()
                }
            }
            Message::Quit => cosmic::iced::exit(),
            Message::DndEnter(service_item) => todo!(),
            Message::DndDrop(service_item) => todo!(),
            Message::None => Task::none(),
        }
    }

    // Main window view
    fn view(&self) -> Element<Message> {
        let icon_left = icon::from_name("arrow-left");
        let icon_right = icon::from_name("arrow-right");

        let video_range: f32 =
            if let Some(video) = &self.presenter.video {
                let duration = video.duration();
                duration.as_secs_f32()
            } else {
                0.0
            };

        let video_button_icon =
            if let Some(video) = &self.presenter.video {
                if video.paused() {
                    button::icon(icon::from_name("media-play"))
                        .tooltip("Play")
                        .on_press(Message::Present(
                            presenter::Message::StartVideo,
                        ))
                } else {
                    button::icon(icon::from_name("media-pause"))
                        .tooltip("Pause")
                        .on_press(Message::Present(
                            presenter::Message::StartVideo,
                        ))
                }
            } else {
                button::icon(icon::from_name("media-play"))
                    .tooltip("Play")
                    .on_press(Message::Present(
                        presenter::Message::StartVideo,
                    ))
            };

        let slide_preview = column![
            Space::with_height(Length::Fill),
            Container::new(
                self.presenter.view().map(|m| Message::Present(m)),
            )
            .align_bottom(Length::Fill),
            Container::new(if self.presenter.video.is_some() {
                row![
                    video_button_icon,
                    Container::new(
                        slider(
                            0.0..=video_range,
                            self.presenter.video_position,
                            |pos| {
                                Message::Present(
                                    presenter::Message::VideoPos(pos),
                                )
                            }
                        )
                        .step(0.1)
                    )
                    .center_x(Length::Fill)
                    .padding([7, 0, 0, 0])
                ]
                .padding(5)
            } else {
                row![]
            })
            .center_x(Length::Fill),
            Space::with_height(Length::Fill),
        ]
        .spacing(3);

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
            .width(Length::FillPortion(2)),
            Container::new(slide_preview)
                .center_y(Length::Fill)
                .width(Length::FillPortion(3)),
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
            .width(Length::FillPortion(2)),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(20);

        let column = column![
            Container::new(row).center_y(Length::Fill),
            Container::new(
                self.presenter
                    .slide_preview()
                    .map(|m| Message::Present(m))
            )
            .clip(true)
            .width(Length::Fill)
            .center_y(130)
        ];

        let element: Element<Message> = column.into();
        element
    }

    // View for presentation
    fn view_window(&self, _id: window::Id) -> Element<Message> {
        debug!("window");
        self.presenter.view().map(|m| Message::Present(m))
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
    //         assert_eq!(slide, data)
    //     } else {
    //         assert!(false)
    //     }
    // }
}
