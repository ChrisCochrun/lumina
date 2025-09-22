use clap::{command, Parser};
use core::service_items::ServiceItem;
use core::slide::{
    Background, BackgroundKind, Slide, SlideBuilder, TextAlignment,
};
use core::songs::Song;
use cosmic::app::context_drawer::ContextDrawer;
use cosmic::app::{Core, Settings, Task};
use cosmic::iced::alignment::Vertical;
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::window::{Mode, Position};
use cosmic::iced::{self, event, window, Color, Length, Point};
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::dnd::DndAction;
use cosmic::iced_widget::{column, row, stack};
use cosmic::theme;
use cosmic::widget::dnd_destination::dnd_destination;
use cosmic::widget::menu::key_bind::Modifier;
use cosmic::widget::menu::{ItemWidth, KeyBind};
use cosmic::widget::nav_bar::nav_bar_style;
use cosmic::widget::tooltip::Position as TPosition;
use cosmic::widget::{
    button, dnd_source, horizontal_space, mouse_area, nav_bar,
    search_input, tooltip, vertical_space, RcElementWrapper, Space,
};
use cosmic::widget::{container, text};
use cosmic::widget::{icon, slider};
use cosmic::widget::{menu, Container};
use cosmic::{executor, Application, ApplicationExt, Element};
use crisp::types::Value;
use lisp::parse_lisp;
use miette::{miette, Result};
use rayon::prelude::*;
use resvg::usvg::fontdb;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, level_filters::LevelFilter};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use ui::library::{self, Library};
use ui::presenter::{self, Presenter};
use ui::song_editor::{self, SongEditor};
use ui::EditorMode;

use crate::core::kinds::ServiceItemKind;
use crate::ui::text_svg::{self};

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
        settings = Settings::default()
            .debug(false)
            .is_daemon(true)
            .transparent(true);
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

// fn theme(_state: &App) -> Theme {
//     Theme::dark()
// }

struct App {
    core: Core,
    nav_model: nav_bar::Model,
    file: PathBuf,
    presenter: Presenter,
    windows: Vec<window::Id>,
    service: Vec<ServiceItem>,
    current_item: (usize, usize),
    presentation_open: bool,
    cli_mode: bool,
    library: Option<Library>,
    library_open: bool,
    editor_mode: Option<EditorMode>,
    song_editor: SongEditor,
    searching: bool,
    search_query: String,
    search_results: Vec<ServiceItem>,
    search_id: cosmic::widget::Id,
    library_dragged_item: Option<ServiceItem>,
    fontdb: Arc<fontdb::Database>,
    menu_keys: HashMap<KeyBind, MenuAction>,
}

#[derive(Debug, Clone)]
enum Message {
    Present(presenter::Message),
    Library(library::Message),
    SongEditor(song_editor::Message),
    File(PathBuf),
    OpenWindow,
    CloseWindow(Option<window::Id>),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
    AddLibrary(Library),
    LibraryToggle,
    Quit,
    Key(Key, Modifiers),
    None,
    EditorToggle(bool),
    ChangeServiceItem(usize),
    AddServiceItem(usize, ServiceItem),
    AddServiceItemDrop(usize),
    AppendServiceItem(ServiceItem),
    AddService(Vec<ServiceItem>),
    SearchFocus,
    Search(String),
    CloseSearch,
    UpdateSearchResults(Vec<ServiceItem>),
    OpenEditor(ServiceItem),
    New,
    Open,
    OpenFile(PathBuf),
    Save(Option<PathBuf>),
    SaveAs,
    OpenSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    New,
    Save,
    SaveAs,
    Open,
    OpenSettings,
}

impl menu::Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::New => Message::New,
            MenuAction::Save => Message::Save(None),
            MenuAction::SaveAs => Message::SaveAs,
            MenuAction::Open => Message::Open,
            MenuAction::OpenSettings => Message::OpenSettings,
        }
    }
}

const HEADER_SPACE: u16 = 6;

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
        let nav_model = nav_bar::Model::default();

        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        let fontdb = Arc::new(fontdb);

        let mut windows = vec![];

        if input.ui {
            windows.push(core.main_window_id().unwrap());
        }

        let items = match read_to_string(input.file) {
            Ok(lisp) => {
                let mut service_items = vec![];
                let lisp = crisp::reader::read(&lisp);
                match lisp {
                    Value::List(vec) => {
                        // let items = vec
                        //     .into_par_iter()
                        //     .map(|value| parse_lisp(value))
                        //     .collect();
                        // slide_vector.append(items);
                        for value in vec {
                            let mut inner_vector = parse_lisp(value);
                            service_items.append(&mut inner_vector);
                        }
                    }
                    _ => todo!(),
                }
                service_items
            }
            Err(e) => {
                warn!("Missing file or could not read: {e}");
                vec![]
            }
        };

        let items: Vec<ServiceItem> = items
            .into_par_iter()
            .map(|mut item| {
                item.slides = item
                    .slides
                    .into_par_iter()
                    .map(|mut slide| {
                        text_svg::text_svg_generator(
                            &mut slide,
                            Arc::clone(&fontdb),
                        );
                        slide
                    })
                    .collect();
                item
            })
            .collect();

        let presenter = Presenter::with_items(items.clone());
        let song_editor = SongEditor::new(Arc::clone(&fontdb));

        // for item in items.iter() {
        //     nav_model.insert().text(item.title()).data(item.clone());
        // }

        let mut menu_keys = HashMap::new();
        menu_keys.insert(
            KeyBind {
                modifiers: vec![Modifier::Ctrl],
                key: Key::Character("s".into()),
            },
            MenuAction::Save,
        );
        menu_keys.insert(
            KeyBind {
                modifiers: vec![Modifier::Ctrl],
                key: Key::Character("o".into()),
            },
            MenuAction::Open,
        );
        menu_keys.insert(
            KeyBind {
                modifiers: vec![Modifier::Ctrl],
                key: Key::Character(".".into()),
            },
            MenuAction::OpenSettings,
        );
        // nav_model.activate_position(0);
        let mut app = Self {
            presenter,
            core,
            nav_model,
            service: items,
            file: PathBuf::default(),
            windows,
            presentation_open: false,
            cli_mode: !input.ui,
            library: None,
            library_open: true,
            editor_mode: None,
            song_editor,
            searching: false,
            search_results: vec![],
            search_query: String::new(),
            search_id: cosmic::widget::Id::unique(),
            current_item: (0, 0),
            library_dragged_item: None,
            fontdb: Arc::clone(&fontdb),
            menu_keys,
        };

        let mut batch = vec![];

        if input.ui {
            debug!("main view");
            batch.push(app.update_title());
        } else {
            debug!("window view");
            batch.push(app.show_window());
        }

        batch.push(app.add_library());
        // batch.push(app.add_service(items, Arc::clone(&fontdb)));
        let batch = Task::batch(batch);
        (app, batch)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let file_menu = menu::Tree::with_children(
            Into::<Element<Message>>::into(menu::root("File")),
            menu::items(
                &self.menu_keys,
                vec![
                    menu::Item::Button("New", None, MenuAction::New),
                    menu::Item::Button(
                        "Open",
                        None,
                        MenuAction::Open,
                    ),
                    menu::Item::Button(
                        "Save",
                        None,
                        MenuAction::Save,
                    ),
                    menu::Item::Button(
                        "Save As",
                        None,
                        MenuAction::SaveAs,
                    ),
                ],
            ),
        );
        let settings_menu = menu::Tree::with_children(
            Into::<Element<Message>>::into(menu::root("Settings")),
            menu::items(
                &self.menu_keys,
                vec![menu::Item::Button(
                    "Open Settings",
                    None,
                    MenuAction::OpenSettings,
                )],
            ),
        );
        let menu_bar =
            menu::bar::<Message>(vec![file_menu, settings_menu])
                .item_width(ItemWidth::Static(250))
                .main_offset(10);
        vec![menu_bar.into()]
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        // let editor_toggle = toggler(self.editor_mode.is_some())
        //     .label("Editor")
        //     .spacing(10)
        //     .width(Length::Shrink)
        //     .on_toggle(Message::EditorToggle);

        let presenter_window = self.windows.get(1);
        let text = if self.presentation_open {
            text::body("End Presentation")
        } else {
            text::body("Present")
        };

        let row = row![
            tooltip(
                button::custom(
                    row!(
                        Container::new(
                            icon::from_name("search")
                                .scale(5)
                                .symbolic(true)
                        )
                        .center_y(Length::Fill),
                        text::body("Search")
                    )
                    .spacing(5),
                )
                .class(cosmic::theme::style::Button::HeaderBar)
                .on_press(Message::SearchFocus),
                "Search Library",
                TPosition::Bottom,
            ),
            tooltip(
                button::custom(
                    row!(
                        Container::new(
                            icon::from_name("document-edit-symbolic")
                                .scale(3)
                        )
                        .center_y(Length::Fill),
                        text::body(if self.editor_mode.is_some() {
                            "Present Mode"
                        } else {
                            "Edit Mode"
                        })
                    )
                    .spacing(5),
                )
                .class(cosmic::theme::style::Button::HeaderBar)
                .on_press(Message::EditorToggle(
                    self.editor_mode.is_none(),
                )),
                "Enter Edit Mode",
                TPosition::Bottom,
            ),
            tooltip(
                button::custom(
                    row!(
                        Container::new(
                            icon::from_name(
                                if self.presentation_open {
                                    "window-close-symbolic"
                                } else {
                                    "view-presentation-symbolic"
                                }
                            )
                            .scale(3)
                        )
                        .center_y(Length::Fill),
                        text
                    )
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
            ),
            tooltip(
                button::custom(
                    row!(
                        Container::new(
                            icon::from_name("view-list-symbolic")
                                .scale(3)
                        )
                        .center_y(Length::Fill),
                        text::body(if self.library_open {
                            "Close Library"
                        } else {
                            "Open Library"
                        })
                    )
                    .spacing(5),
                )
                .class(cosmic::theme::style::Button::HeaderBar)
                .on_press(Message::LibraryToggle),
                "Open Library",
                TPosition::Bottom,
            )
        ]
        .spacing(HEADER_SPACE)
        .into();
        vec![row]
    }

    fn footer(&self) -> Option<Element<Self::Message>> {
        let total_items_text =
            format!("Total Service Items: {}", self.service.len());

        let total_slides = self
            .service
            .iter()
            .fold(0, |a, item| a + item.slides.len());

        let total_slides_text =
            format!("Total Slides: {total_slides}");
        let row = row![
            text::body(total_items_text),
            text::body(total_slides_text)
        ]
        .spacing(10);
        Some(
            Container::new(row)
                .align_right(Length::Fill)
                .padding([5, 0, 0, 0])
                .into(),
        )
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

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        if self.searching {
            let items: Vec<Element<Message>> = self
                .search_results
                .iter()
                .map(|item| {
                    let title = text::title4(item.title.clone());
                    let subtitle = text::body(item.kind.to_string());
                    Element::from(Container::new(
                        row![
                            column![title, subtitle].spacing(
                                cosmic::theme::active()
                                    .cosmic()
                                    .space_xxs(),
                            ),
                            horizontal_space(),
                            tooltip(
                                button::icon(
                                    icon::from_name("add")
                                        .symbolic(true)
                                )
                                .icon_size(
                                    cosmic::theme::active()
                                        .cosmic()
                                        .space_l()
                                )
                                .on_press(
                                    Message::AppendServiceItem(
                                        item.clone()
                                    )
                                ),
                                "Add to service",
                                TPosition::FollowCursor
                            ),
                            tooltip(
                                button::icon(
                                    icon::from_name("edit")
                                        .symbolic(true)
                                )
                                .icon_size(
                                    cosmic::theme::active()
                                        .cosmic()
                                        .space_l()
                                )
                                .on_press(Message::OpenEditor(
                                    item.clone()
                                )),
                                "Edit Item",
                                TPosition::FollowCursor
                            ),
                        ]
                        .align_y(Vertical::Center),
                    ))
                })
                .collect();
            let modal = Container::new(
                column![
                    search_input(
                        "Amazing Grace",
                        self.search_query.clone()
                    )
                    .id(self.search_id.clone())
                    .select_on_focus(true)
                    .on_input(Message::Search)
                    .on_submit(Message::Search),
                    column(items).spacing(
                        cosmic::theme::active().cosmic().space_xxs()
                    )
                ]
                .spacing(cosmic::theme::active().cosmic().space_s()),
            )
            .padding(cosmic::theme::active().cosmic().space_xxxl())
            .style(nav_bar_style);
            let modal = Container::new(modal)
                .padding([
                    cosmic::theme::active().cosmic().space_xxl(),
                    cosmic::theme::active().cosmic().space_xxxl() * 2,
                ])
                .center_x(Length::Fill)
                .align_top(Length::Fill);
            let mouse_stack = stack!(
                mouse_area(Space::new(Length::Fill, Length::Fill))
                    .on_press(Message::CloseSearch),
                modal
            );
            Some(mouse_stack.into())
        } else {
            None
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Key(key, modifiers) => {
                self.process_key_press(key, modifiers)
            }
            Message::SongEditor(message) => {
                // debug!(?message);
                match self.song_editor.update(message) {
                    song_editor::Action::Task(task) => {
                        task.map(|m| {
                            cosmic::Action::App(Message::SongEditor(
                                m,
                            ))
                        })
                    }
                    song_editor::Action::UpdateSong(song) => {
                        if let Some(library) = &mut self.library {
                            self.update(Message::Library(
                                library::Message::UpdateSong(song),
                            ))
                        } else {
                            Task::none()
                        }
                    }
                    song_editor::Action::None => Task::none(),
                }
            }
            Message::Present(message) => {
                // debug!(?message);
                if self.presentation_open
                    && let Some(video) = &mut self.presenter.video
                {
                    video.set_muted(false);
                }
                match self.presenter.update(message) {
                    presenter::Action::Task(task) => task.map(|m| {
                        // debug!("Should run future");
                        cosmic::Action::App(Message::Present(m))
                    }),
                    presenter::Action::None => Task::none(),
                    presenter::Action::NextSlide => {
                        let slide_index = self.current_item.1;
                        let item_index = self.current_item.0;
                        let mut tasks = vec![];
                        if let Some(item) =
                            self.service.get(item_index)
                        {
                            if item.slides.len() > slide_index + 1 {
                                // let slide_length = item.slides.len();
                                // debug!(
                                //     slide_index,
                                //     slide_length,
                                //     ?item,
                                //     "Slides are longer"
                                // );
                                let slide = item.slides
                                    [slide_index + 1]
                                    .clone();
                                let action = self.presenter.update(
                                    presenter::Message::SlideChange(
                                        slide,
                                    ),
                                );
                                match action {
                                    presenter::Action::Task(task) => {
                                        tasks.push(task.map(|m| {
                                            cosmic::Action::App(
                                                Message::Present(m),
                                            )
                                        }));
                                    }
                                    _ => todo!(),
                                }
                                self.current_item =
                                    (item_index, slide_index + 1);
                                Task::batch(tasks)
                            } else {
                                // debug!("Slides are not longer");
                                self.current_item =
                                    (item_index + 1, 0);
                                if let Some(item) =
                                    self.service.get(item_index + 1)
                                {
                                    let action = self.presenter.update(presenter::Message::SlideChange(item.slides[0].clone()));
                                    match action {
                                        presenter::Action::Task(
                                            task,
                                        ) => {
                                            tasks
                                                .push(task.map(|m| {
                                                cosmic::Action::App(
                                                    Message::Present(
                                                        m,
                                                    ),
                                                )
                                            }));
                                        }
                                        _ => todo!(),
                                    }
                                }
                                Task::batch(tasks)
                            }
                        } else {
                            Task::none()
                        }
                    }
                    presenter::Action::PrevSlide => {
                        let slide_index = self.current_item.1;
                        let item_index = self.current_item.0;
                        let mut tasks = vec![];
                        if let Some(item) =
                            self.service.get(item_index)
                        {
                            if slide_index != 0 {
                                let slide = item.slides
                                    [slide_index - 1]
                                    .clone();
                                let action = self.presenter.update(
                                    presenter::Message::SlideChange(
                                        slide,
                                    ),
                                );
                                match action {
                                    presenter::Action::Task(task) => {
                                        tasks.push(task.map(|m| {
                                            cosmic::Action::App(
                                                Message::Present(m),
                                            )
                                        }));
                                    }
                                    _ => todo!(),
                                }
                                self.current_item =
                                    (item_index, slide_index - 1);
                                Task::batch(tasks)
                            } else if slide_index == 0
                                && item_index == 0
                            {
                                Task::none()
                            } else {
                                // debug!("Change slide to previous items slides");
                                let previous_item_slides_length =
                                    if let Some(item) = self
                                        .service
                                        .get(item_index - 1)
                                    {
                                        item.slides.len()
                                    } else {
                                        0
                                    };
                                self.current_item = (
                                    item_index - 1,
                                    previous_item_slides_length - 1,
                                );
                                if let Some(item) =
                                    self.service.get(item_index - 1)
                                {
                                    let action = self.presenter.update(presenter::Message::SlideChange(item.slides[previous_item_slides_length - 1].clone()));
                                    match action {
                                        presenter::Action::Task(
                                            task,
                                        ) => {
                                            tasks
                                                .push(task.map(|m| {
                                                cosmic::Action::App(
                                                    Message::Present(
                                                        m,
                                                    ),
                                                )
                                            }));
                                        }
                                        _ => todo!(),
                                    }
                                }
                                Task::batch(tasks)
                            }
                        } else {
                            Task::none()
                        }
                    }
                }
            }
            Message::Library(message) => {
                let mut song = Song::default();
                if let Some(library) = &mut self.library {
                    match library.update(message) {
                        library::Action::OpenItem(None) => {
                            return Task::none();
                        }
                        library::Action::Task(task) => {
                            return task.map(|message| {
                                cosmic::Action::App(Message::Library(
                                    message,
                                ))
                            });
                        }
                        library::Action::None => return Task::none(),
                        library::Action::OpenItem(Some((
                            kind,
                            index,
                        ))) => {
                            debug!(
                                "Should get song at index: {:?}",
                                index
                            );
                            let Some(lib_song) =
                                library.get_song(index)
                            else {
                                return Task::none();
                            };
                            self.editor_mode = Some(kind.into());
                            song = lib_song.to_owned();
                            debug!(
                                "Should change songs to: {:?}",
                                song
                            );
                        }
                        library::Action::DraggedItem(
                            service_item,
                        ) => {
                            debug!("hi");
                            self.library_dragged_item =
                                Some(service_item);
                            // self.nav_model
                            //     .insert()
                            //     .text(service_item.title.clone())
                            //     .data(service_item);
                        }
                    }
                }
                self.update(Message::SongEditor(
                    song_editor::Message::ChangeSong(song),
                ))
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
                _ = self
                    .set_window_title(format!("window_{count}"), id);

                spawn_window.map(|id| {
                    cosmic::Action::App(Message::WindowOpened(
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
            Message::LibraryToggle => {
                self.library_open = !self.library_open;
                Task::none()
            }
            Message::Quit => cosmic::iced::exit(),
            Message::AddLibrary(library) => {
                self.library = Some(library);
                Task::none()
            }
            Message::AddService(service) => {
                self.service = service;
                Task::none()
            }
            Message::None => Task::none(),
            Message::EditorToggle(edit) => {
                if edit {
                    self.editor_mode = Some(EditorMode::Song);
                } else {
                    self.editor_mode = None;
                }
                Task::none()
            }
            Message::SearchFocus => {
                self.searching = true;
                cosmic::widget::text_input::focus(
                    self.search_id.clone(),
                )
            }
            Message::ChangeServiceItem(index) => {
                if let Some((index, item)) = self
                    .service
                    .iter()
                    .enumerate()
                    .find(|(id, _)| index == *id)
                    && let Some(slide) = item.slides.first()
                {
                    self.current_item = (index, 0);
                    self.presenter.update(
                        presenter::Message::SlideChange(
                            slide.clone(),
                        ),
                    );
                }
                Task::none()
            }
            Message::AddServiceItem(index, mut item) => {
                item.slides = item
                    .slides
                    .into_par_iter()
                    .map(|mut slide| {
                        let fontdb = Arc::clone(&self.fontdb);
                        text_svg::text_svg_generator(
                            &mut slide, fontdb,
                        );
                        slide
                    })
                    .collect();
                self.service.insert(index, item);
                self.presenter.update_items(self.service.clone());
                Task::none()
            }
            Message::AddServiceItemDrop(index) => {
                if let Some(item) = &self.library_dragged_item {
                    self.service.insert(index, item.clone());
                }
                Task::none()
            }
            Message::AppendServiceItem(mut item) => {
                item.slides = item
                    .slides
                    .into_par_iter()
                    .map(|mut slide| {
                        let fontdb = Arc::clone(&self.fontdb);
                        text_svg::text_svg_generator(
                            &mut slide, fontdb,
                        );
                        slide
                    })
                    .collect();
                self.service.push(item);
                self.presenter.update_items(self.service.clone());
                Task::none()
            }
            Message::Search(query) => {
                self.search_query = query.clone();
                self.search(query)
            }
            Message::UpdateSearchResults(items) => {
                self.search_results = items;
                Task::none()
            }
            Message::CloseSearch => {
                self.search_query = String::new();
                self.search_results = vec![];
                self.searching = false;
                Task::none()
            }
            Message::OpenEditor(item) => {
                let kind = item.kind;
                match kind {
                    ServiceItemKind::Song(song) => {
                        self.editor_mode = Some(EditorMode::Song);
                        self.update(Message::SongEditor(
                            song_editor::Message::ChangeSong(song),
                        ))
                    }
                    ServiceItemKind::Video(_video) => todo!(),
                    ServiceItemKind::Image(_image) => todo!(),
                    ServiceItemKind::Presentation(_presentation) => {
                        todo!()
                    }
                    ServiceItemKind::Content(_slide) => todo!(),
                }
            }
            Message::New => {
                debug!("new file");
                Task::none()
            }
            Message::Open => {
                debug!("Open file");
                Task::none()
            }
            Message::OpenFile(file) => {
                debug!(?file, "opening file");
                Task::none()
            }
            Message::Save(file) => {
                let Some(file) = file else {
                    debug!("saving current");
                    return Task::none();
                };
                debug!(?file, "saving new file");
                Task::none()
            }
            Message::SaveAs => {
                debug!("saving as a file");
                Task::none()
            }
            Message::OpenSettings => {
                debug!("Opening settings");
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

        let service_list = Container::new(self.service_list())
            .padding(5)
            .width(Length::FillPortion(2));

        let library = if self.library_open {
            Container::new(
                Container::new(
                    if let Some(library) = &self.library {
                        library.view().map(Message::Library)
                    } else {
                        Space::new(0, 0).into()
                    },
                )
                .style(nav_bar_style),
            )
            .padding(5)
            .width(Length::FillPortion(2))
        } else {
            Container::new(horizontal_space().width(0))
        };

        let song_editor =
            self.song_editor.view().map(Message::SongEditor);

        let row = row![
            library,
            service_list,
            Container::new(
                button::icon(icon_left)
                    .icon_size(128)
                    .tooltip("Previous Slide")
                    .width(128)
                    .on_press(Message::Present(
                        presenter::Message::PrevSlide
                    ))
                    .class(theme::style::Button::Transparent)
            )
            .center_y(Length::Fill)
            .align_right(Length::FillPortion(1)),
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
                    .class(theme::style::Button::Transparent)
            )
            .center_y(Length::Fill)
            .align_left(Length::FillPortion(1)),
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
            .center_y(180)
        ];

        if let Some(_editor) = &self.editor_mode {
            container(song_editor)
                .padding(cosmic::theme::spacing().space_xxl)
                .into()
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
            cosmic::Action::App(Message::WindowOpened(id, None))
        })
    }

    fn add_library(&self) -> Task<Message> {
        Task::perform(async move { Library::new().await }, |x| {
            cosmic::Action::App(Message::AddLibrary(x))
        })
    }

    fn search(&self, query: String) -> Task<Message> {
        if let Some(library) = self.library.clone() {
            Task::perform(
                async move { library.search_items(query).await },
                |items| {
                    cosmic::Action::App(Message::UpdateSearchResults(
                        items,
                    ))
                },
            )
        } else {
            Task::none()
        }
    }

    fn add_service(
        &self,
        items: Vec<ServiceItem>,
        fontdb: Arc<fontdb::Database>,
    ) -> Task<Message> {
        Task::perform(
            async move {
                let items: Vec<ServiceItem> = items
                    .into_par_iter()
                    .map(|mut item| {
                        item.slides = item
                            .slides
                            .into_par_iter()
                            .map(|mut slide| {
                                text_svg::text_svg_generator(
                                    &mut slide,
                                    Arc::clone(&fontdb),
                                );
                                slide
                            })
                            .collect();
                        item
                    })
                    .collect();
                items
            },
            |x| cosmic::Action::App(Message::AddService(x)),
        )
    }

    fn process_key_press(
        &mut self,
        key: Key,
        modifiers: Modifiers,
    ) -> Task<Message> {
        // debug!(?key, ?modifiers);
        if self.editor_mode.is_some() {
            return Task::none();
        }
        if self.song_editor.editing() {
            return Task::none();
        }
        if self.searching {
            match (key, modifiers) {
                (
                    Key::Named(iced::keyboard::key::Named::Escape),
                    _,
                ) => return self.update(Message::CloseSearch),
                _ => return Task::none(),
            }
        }
        match (key, modifiers) {
            (Key::Character(k), Modifiers::CTRL) if k == *"s" => {
                self.update(Message::Save(None))
            }
            (Key::Character(k), Modifiers::CTRL) if k == *"o" => {
                self.update(Message::Open)
            }
            (Key::Character(k), Modifiers::CTRL) if k == *"." => {
                self.update(Message::OpenSettings)
            }
            (Key::Character(k), Modifiers::CTRL)
                if k == *"k" || k == *"f" =>
            {
                self.update(Message::SearchFocus)
            }
            (Key::Character(k), _) if k == *"/" => {
                self.update(Message::SearchFocus)
            }
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

    fn service_list(&self) -> Element<Message> {
        let list = self
            .service
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let button = button::standard(item.title.clone())
                    .leading_icon({
                        match item.kind {
                            core::kinds::ServiceItemKind::Song(_) => {
                                icon::from_name("folder-music-symbolic")
                            },
                            core::kinds::ServiceItemKind::Video(_) => {
                                icon::from_name("folder-videos-symbolic")
                            },
                            core::kinds::ServiceItemKind::Image(_) => {
                                icon::from_name("folder-pictures-symbolic")
                            },
                            core::kinds::ServiceItemKind::Presentation(_) => {
                                icon::from_name("x-office-presentation-symbolic")
                            },
                            core::kinds::ServiceItemKind::Content(_) => {
                                icon::from_name("x-office-presentation-symbolic")
                            },
                        }
                    })
                    // .icon_size(cosmic::theme::spacing().space_l)
                    .class(cosmic::theme::style::Button::HeaderBar)
                    // .spacing(cosmic::theme::spacing().space_l)
                    .height(cosmic::theme::spacing().space_xl)
                    .width(Length::Fill)
                    .on_press(Message::ChangeServiceItem(index));
                let tooltip = tooltip(button,
                                      text::body(item.kind.to_string()),
                                      TPosition::Right);
                dnd_destination(tooltip, vec!["application/service-item".into()])
                    .data_received_for::<ServiceItem>( move |item| {
                        if let Some(item) = item {
                            Message::AddServiceItem(index, item)
                        } else {
                            Message::None
                        }
                    }).on_finish(move |mime, data, action, x, y| {
                        debug!(mime, ?data, ?action, x, y);
                        let Ok(item) = ServiceItem::try_from((data, mime)) else {
                            return Message::None;
                        };
                        debug!(?item);
                        Message::AddServiceItem(index, item)
                    })
                    .into()
            });

        let column = column![
            text::heading("Service List")
                .center()
                .width(Length::Fill),
            iced::widget::horizontal_rule(1),
            column(list).spacing(10),
            dnd_destination(
                vertical_space().width(Length::Fill),
                vec!["application/service-item".into()]
            )
            .data_received_for::<ServiceItem>(|item| {
                item.map_or_else(
                    || Message::None,
                    Message::AppendServiceItem,
                )
            })
            .on_finish(
                move |mime, data, action, x, y| {
                    debug!(mime, ?data, ?action, x, y);
                    let Ok(item) =
                        ServiceItem::try_from((data, mime))
                    else {
                        return Message::None;
                    };
                    debug!(?item);
                    Message::AppendServiceItem(item)
                }
            )
        ]
        .padding(10)
        .spacing(10);
        let container = Container::new(column)
            // .height(Length::Fill)
            .style(nav_bar_style);

        container.center(Length::FillPortion(2)).into()
    }
}

#[cfg(test)]
mod test {

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
