use clap::{command, Parser};
use core::model::{get_db, LibraryKind};
use core::service_items::{ServiceItem, ServiceItemModel};
use core::slide::*;
use cosmic::app::context_drawer::ContextDrawer;
use cosmic::app::{Core, Settings, Task};
use cosmic::iced::clipboard::dnd::DndAction;
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::window::{Mode, Position};
use cosmic::iced::{
    self, event, window, Color, Length, Padding, Point,
};
use cosmic::iced_futures::Subscription;
use cosmic::iced_widget::{column, row};
use cosmic::prelude::*;
use cosmic::widget::dnd_destination::DragId;
use cosmic::widget::nav_bar::nav_bar_style;
use cosmic::widget::segmented_button::Entity;
use cosmic::widget::tooltip::Position as TPosition;
use cosmic::widget::{
    button, horizontal_space, nav_bar, tooltip, Space,
};
use cosmic::widget::{icon, slider};
use cosmic::widget::{text, toggler};
use cosmic::{executor, Application, ApplicationExt, Element};
use cosmic::{widget::Container, Theme};
use crisp::types::Value;
use lisp::parse_lisp;
use miette::{miette, Result};
use sqlx::{SqliteConnection, SqlitePool};
use std::fs::read_to_string;
use std::path::PathBuf;
use tracing::{debug, level_filters::LevelFilter};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use ui::library::{self, Library};
use ui::presenter::{self, Presenter};
use ui::song_editor::{self, SongEditor};
use ui::EditorMode;

pub mod core;
pub mod lisp;
pub mod ui;

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
        settings = Settings::default().debug(false).is_daemon(true);
    } else {
        debug!("window view");
        settings = Settings::default()
            .debug(false)
            .no_main_window(true)
            .is_daemon(true);
    }

    cosmic::app::run::<App>(settings, args)
        .map_err(|e| miette!("Invalid things... {}", e))
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
    library: Option<Library>,
    library_open: bool,
    library_width: f32,
    editor_mode: Option<EditorMode>,
    song_editor: SongEditor,
}

#[derive(Debug, Clone)]
enum Message {
    Present(presenter::Message),
    Library(library::Message),
    SongEditor(song_editor::Message),
    File(PathBuf),
    DndEnter(Entity, Vec<String>),
    DndDrop(Entity, Option<ServiceItem>, DndAction),
    OpenWindow,
    CloseWindow(Option<window::Id>),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
    AddLibrary(Library),
    Quit,
    Key(Key, Modifiers),
    None,
    DndLeave(Entity),
    EditorToggle(bool),
}

const HEADER_SPACE: u16 = 20;

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
        let slides = items.to_slides().unwrap_or_default();
        let current_slide = slides[0].clone();
        let song_editor = SongEditor::new();

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
            library: None,
            library_open: true,
            library_width: 60.0,
            editor_mode: None,
            song_editor,
        };

        let mut batch = vec![];

        if input.ui {
            debug!("main view");
            batch.push(app.update_title())
        } else {
            debug!("window view");
            batch.push(app.show_window())
        };

        batch.push(app.add_library());
        let batch = Task::batch(batch);
        (app, batch)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    fn nav_bar(
        &self,
    ) -> Option<Element<cosmic::app::Message<Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = cosmic::widget::nav_bar(nav_model, |id| {
            cosmic::app::Message::Cosmic(
                cosmic::app::cosmic::Message::NavBar(id),
            )
        })
        .on_dnd_enter(|entity, data| {
            debug!("entered");
            cosmic::app::Message::App(Message::DndEnter(entity, data))
        })
        .on_dnd_leave(|entity| {
            debug!("left");
            cosmic::app::Message::App(Message::DndLeave(entity))
        })
        .drag_id(DragId::new())
        .on_context(|id| {
            cosmic::app::Message::Cosmic(
                cosmic::app::cosmic::Message::NavBarContext(id),
            )
        })
        .on_dnd_drop::<ServiceItem>(|entity, data, action| {
            debug!("dropped");
            cosmic::app::Message::App(Message::DndDrop(
                entity, data, action,
            ))
        })
        .context_menu(None)
        .into_container()
        // XXX both must be shrink to avoid flex layout from ignoring it
        .width(Length::Shrink)
        .height(Length::Shrink);

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        let column = column![
            text::heading("Service List").center().width(280),
            nav
        ]
        .spacing(10);
        let padding = Padding::new(0.0).top(20);
        let container = Container::new(column)
            .style(nav_bar_style)
            .padding(padding);
        Some(container.into())
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(
        &mut self,
        id: nav_bar::Id,
    ) -> Task<Self::Message> {
        self.nav_model.activate(id);
        // debug!(?id);
        self.update_title()
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }
    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }
    fn header_end(&self) -> Vec<Element<Self::Message>> {
        let editor_toggle = toggler(self.editor_mode.is_some())
            .label("Editor")
            .spacing(10)
            .on_toggle(Message::EditorToggle);

        let presenter_window = self.windows.get(1);
        let text = if self.presentation_open {
            text::body("Close Presentation")
        } else {
            text::body("Open Presentation")
        };

        vec![
            editor_toggle.into(),
            horizontal_space().width(HEADER_SPACE).into(),
            tooltip(
                button::custom(
                    row!(
                        Container::new(
                            icon::from_name(
                                if self.presentation_open {
                                    "dialog-close"
                                } else {
                                    "view-presentation-symbolic"
                                }
                            )
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
                            presenter_window.copied(),
                        )
                    } else {
                        Message::OpenWindow
                    }
                }),
                "Start Presentation",
                TPosition::Bottom,
            )
            .into(),
            horizontal_space().width(HEADER_SPACE).into(),
        ]
    }

    fn footer(&self) -> Option<Element<Self::Message>> {
        Some(text::body("Sux").into())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen_with(|event, _, id| {
            // debug!(?event);
            match event {
                iced::Event::Keyboard(event) => match event {
                    iced::keyboard::Event::KeyReleased {
                        key,
                        modifiers,
                        ..
                    } => Some(Message::Key(key, modifiers)),
                    _ => None,
                },
                iced::Event::Mouse(_event) => None,
                iced::Event::Window(window_event) => {
                    match window_event {
                        window::Event::CloseRequested => {
                            debug!("Closing window");
                            Some(Message::CloseWindow(Some(id)))
                        }
                        window::Event::Opened {
                            position, ..
                        } => {
                            debug!(?window_event, ?id);
                            Some(Message::WindowOpened(id, position))
                        }
                        window::Event::Closed => {
                            debug!("Closed window");
                            Some(Message::WindowClosed(id))
                        }
                        _ => None,
                    }
                }
                iced::Event::Touch(_touch) => None,
                iced::Event::A11y(_id, _action_request) => None,
                iced::Event::Dnd(_dnd_event) => None,
                iced::Event::PlatformSpecific(_platform_specific) => {
                    None
                }
            }
        })
    }

    fn context_drawer(
        &self,
    ) -> Option<
        cosmic::app::context_drawer::ContextDrawer<Self::Message>,
    > {
        ContextDrawer {
            title: Some("Context".into()),
            header_actions: vec![],
            header: Some("hi".into()),
            content: "Sup".into(),
            footer: Some("foot".into()),
            on_close: Message::None,
        };
        None
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Key(key, modifiers) => {
                self.process_key_press(key, modifiers)
            }
            Message::SongEditor(message) => {
                debug!(?message);
                let library_task = if let Some(mut song) =
                    self.song_editor.song.clone()
                {
                    match message {
                        song_editor::Message::ChangeFont(
                            ref font,
                        ) => {
                            song.font = Some(font.to_string());
                            self.song_editor.song =
                                Some(song.clone());
                        }
                        song_editor::Message::ChangeFontSize(
                            font_size,
                        ) => {
                            song.font_size = Some(font_size as i32);
                            self.song_editor.song =
                                Some(song.clone());
                        }
                        song_editor::Message::ChangeTitle(
                            ref title,
                        ) => {
                            song.title = title.to_string();
                            self.song_editor.song =
                                Some(song.clone());
                        }
                        song_editor::Message::ChangeVerseOrder(
                            ref vo,
                        ) => {
                            let verse_order = vo
                                .split(" ")
                                .into_iter()
                                .map(|s| s.to_owned())
                                .collect();
                            song.verse_order = Some(verse_order);
                            self.song_editor.song =
                                Some(song.clone());
                        }
                        song_editor::Message::ChangeLyrics(
                            ref action,
                        ) => {
                            self.song_editor
                                .lyrics
                                .perform(action.clone());
                            let lyrics =
                                self.song_editor.lyrics.text();
                            song.lyrics = Some(lyrics.to_string());
                            self.song_editor.song =
                                Some(song.clone());
                        }
                        song_editor::Message::ChangeAuthor(
                            ref author,
                        ) => {
                            song.author = Some(author.to_string());
                            self.song_editor.song =
                                Some(song.clone());
                        }
                        song_editor::Message::Edit(_) => todo!(),
                        _ => (),
                    };

                    if let Some(library) = &mut self.library {
                        let task = library.update(
                            library::Message::UpdateSong(song),
                        );
                        task.map(|_m| {
                            cosmic::app::Message::App(Message::None)
                        })
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                };
                let song_editor_task =
                    self.song_editor.update(message).map(|m| {
                        debug!(?m);
                        cosmic::app::Message::App(Message::None)
                    });
                Task::batch(vec![song_editor_task, library_task])
            }
            Message::Present(message) => {
                // debug!(?message);
                if self.presentation_open {
                    if let Some(video) = &mut self.presenter.video {
                        video.set_muted(false);
                    }
                }
                self.presenter.update(message).map(|x| {
                    debug!(?x);
                    cosmic::app::Message::App(Message::None)
                })
            }

            Message::Library(message) => {
                // debug!(?message);
                let (mut kind, mut index): (LibraryKind, i32) =
                    (LibraryKind::Song, 0);
                let mut opened_item = false;
                match message {
                    library::Message::OpenItem(item) => {
                        let Some(item) = item else {
                            return ().into();
                        };
                        debug!("opening: {:?}", item);
                        kind = item.0;
                        index = item.1;
                        opened_item = true;
                    }
                    _ => {
                        debug!("none");
                    }
                };
                if let Some(library) = &mut self.library {
                    if opened_item {
                        if let Some(song) = library.get_song(index) {
                            self.editor_mode = Some(EditorMode::Song);
                            let _ = self.song_editor.update(
                                song_editor::Message::ChangeSong(
                                    song.clone(),
                                ),
                            );
                        }
                    }
                    library.update(message).map(|x| {
                        debug!(?x);
                        cosmic::app::Message::App(Message::None)
                    })
                } else {
                    Task::none()
                }
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
                    || id > self.core.main_window_id().expect("Cosmic core seems to be missing a main window, was this started in cli mode?")
                {
                    self.presentation_open = true;
                    if let Some(video) = &mut self.presenter.video {
                        video.set_muted(false);
                    }
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
                if self.windows.is_empty() {
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
            Message::DndEnter(entity, data) => {
                debug!(?entity);
                debug!(?data);
                Task::none()
            }
            Message::DndDrop(entity, service_item, action) => {
                debug!(?entity);
                debug!(?action);
                debug!(?service_item);
                Task::none()
            }
            Message::AddLibrary(library) => {
                self.library = Some(library);
                Task::none()
            }
            Message::None => Task::none(),
            Message::DndLeave(entity) => {
                // debug!(?entity);
                Task::none()
            }
            Message::EditorToggle(edit) => {
                if edit {
                    self.editor_mode = Some(EditorMode::Song);
                } else {
                    self.editor_mode = None;
                }
                Task::none()
            }
        }
    }

    // Main window view
    fn view(&self) -> Element<Message> {
        let icon_left = icon::from_name("arrow-left");
        let icon_right = icon::from_name("arrow-right");

        let video_range = self.presenter.video.as_ref().map_or_else(
            || 0.0,
            |video| video.duration().as_secs_f32(),
        );

        let video_button_icon =
            if let Some(video) = &self.presenter.video {
                let (icon_name, tooltip) = if video.paused() {
                    ("media-play", "Play")
                } else {
                    ("media-pause", "Pause")
                };
                button::icon(icon::from_name(icon_name))
                    .tooltip(tooltip)
                    .on_press(Message::Present(
                        presenter::Message::StartVideo,
                    ))
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
                self.presenter.view_preview().map(Message::Present),
            )
            .height(250)
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

        let library =
            Container::new(if let Some(library) = &self.library {
                library.view().map(|m| Message::Library(m))
            } else {
                Space::new(0, 0).into()
            })
            .style(nav_bar_style)
            .center(Length::Fill);

        let song_editor =
            self.song_editor.view().map(|m| Message::SongEditor(m));

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
            library.width(Length::FillPortion(2))
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(20);

        let column = column![
            Container::new(row).center_y(Length::Fill),
            Container::new(
                self.presenter.preview_bar().map(Message::Present)
            )
            .clip(true)
            .width(Length::Fill)
            .center_y(130)
        ];

        if let Some(_editor) = &self.editor_mode {
            Element::from(song_editor)
        } else {
            Element::from(column)
        }
    }

    // View for presentation
    fn view_window(&self, _id: window::Id) -> Element<Message> {
        self.presenter.view().map(Message::Present)
    }
}

impl App
where
    Self: cosmic::Application,
{
    fn active_page_title(&self) -> &str {
        let Some(label) =
            self.nav_model.text(self.nav_model.active())
        else {
            return "Lumina";
        };
        label
    }

    fn update_title(&mut self) -> Task<Message> {
        let header_title = self.active_page_title().to_owned();
        let window_title = format!("{header_title} — Lumina");
        self.core.main_window_id().map_or_else(Task::none, |id| {
            self.set_window_title(window_title, id)
        })
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

    fn add_library(&mut self) -> Task<Message> {
        Task::perform(async move { Library::new().await }, |x| {
            cosmic::app::Message::App(Message::AddLibrary(x))
        })
    }

    fn process_key_press(
        &mut self,
        key: Key,
        modifiers: Modifiers,
    ) -> Task<Message> {
        debug!(?key, ?modifiers);
        if self.editor_mode.is_some() {
            return Task::none();
        }
        if self.song_editor.editing() {
            return Task::none();
        }
        match (key, modifiers) {
            (
                Key::Named(iced::keyboard::key::Named::ArrowRight),
                _,
            ) => self.update(Message::Present(
                presenter::Message::NextSlide,
            )),
            (
                Key::Named(iced::keyboard::key::Named::ArrowLeft),
                _,
            ) => self.update(Message::Present(
                presenter::Message::PrevSlide,
            )),
            (Key::Named(iced::keyboard::key::Named::Space), _) => {
                self.update(Message::Present(
                    presenter::Message::NextSlide,
                ))
            }
            (Key::Character(k), _) if k == *"j" || k == *"l" => self
                .update(Message::Present(
                    presenter::Message::NextSlide,
                )),
            (Key::Character(k), _) if k == *"k" || k == *"h" => self
                .update(Message::Present(
                    presenter::Message::PrevSlide,
                )),
            (Key::Character(k), _) if k == *"q" => {
                self.update(Message::Quit)
            }
            _ => Task::none(),
        }
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
