use clap::{command, Parser};
use core::service_items::ServiceItem;
use core::slide::*;
use core::songs::Song;
use crisp::types::Value;
use iced::keyboard::{Key, Modifiers};
use iced::theme::{self, Palette};
use iced::widget::tooltip::Position as TPosition;
use iced::widget::{
    button, horizontal_space, slider, text, tooltip, vertical_space,
    Space,
};
use iced::widget::{column, row};
use iced::window::{Mode, Position};
use iced::{self, event, window, Length, Padding, Point};
use iced::{color, Subscription};
use iced::{executor, Application, Element};
use iced::{widget::Container, Theme};
use iced::{Settings, Task};
use lisp::parse_lisp;
use miette::{miette, Result};
use rayon::prelude::*;
use std::collections::BTreeMap;
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

    iced::daemon(move || App::init(args), App::update, App::view)
        .settings(settings)
        .subscription(App::subsrciption)
        .theme(App::theme)
        .title(App::title)
        .run()
        .map_err(|e| miette!("Invalid things... {}", e))
}

struct App {
    file: PathBuf,
    presenter: Presenter,
    windows: BTreeMap<window::Id, Window>,
    service: Vec<ServiceItem>,
    current_item: (usize, usize),
    presentation_open: bool,
    cli_mode: bool,
    library: Option<Library>,
    library_open: bool,
    library_width: f32,
    editor_mode: Option<EditorMode>,
    song_editor: SongEditor,
    searching: bool,
    library_dragged_item: Option<ServiceItem>,
}

#[derive(Debug, Clone)]
enum Message {
    Present(presenter::Message),
    Library(library::Message),
    SongEditor(song_editor::Message),
    File(PathBuf),
    DndEnter(Vec<String>),
    DndDrop,
    OpenWindow,
    CloseWindow(Option<window::Id>),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
    AddLibrary(Library),
    LibraryToggle,
    Quit,
    Key(Key, Modifiers),
    None,
    DndLeave(),
    EditorToggle(bool),
    SearchFocus,
    ChangeServiceItem(usize),
    AddServiceItem(usize, ServiceItem),
    AddServiceItemDrop(usize),
    AppendServiceItem(ServiceItem),
}

#[derive(Debug)]
struct Window {
    title: String,
    scale_input: String,
    current_scale: f64,
    theme: Theme,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: Default::default(),
            scale_input: Default::default(),
            current_scale: Default::default(),
            theme: App::theme(),
        }
    }
}

const HEADER_SPACE: u16 = 6;

impl App {
    const APP_ID: &'static str = "lumina";
    fn init(input: Cli) -> (Self, Task<Self::Message>) {
        debug!("init");

        let mut batch = vec![];
        let mut windows = BTreeMap::new();
        if input.ui {
            let settings = window::Settings {
                ..Default::default()
            };
            let (id, open) = window::open(settings);
            batch.push(open);

            windows.insert(id, Window::default());
        }

        let items = match read_to_string(input.file) {
            Ok(lisp) => {
                let mut slide_vector = vec![];
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

        let presenter = Presenter::with_items(items.clone());
        let song_editor = SongEditor::new();

        // for item in items.iter() {
        //     nav_model.insert().text(item.title()).data(item.clone());
        // }

        // nav_model.activate_position(0);

        let mut app = App {
            presenter,
            service: items,
            file: PathBuf::default(),
            windows,
            presentation_open: false,
            cli_mode: !input.ui,
            library: None,
            library_open: true,
            library_width: 60.0,
            editor_mode: None,
            song_editor,
            searching: false,
            current_item: (0, 0),
            library_dragged_item: None,
        };

        if input.ui {
            debug!("main view");
            batch.push(app.update_title())
        } else {
            debug!("window view");
            batch.push(app.show_window())
        };

        batch.push(app.add_library());
        // batch.push(app.add_service(items));
        let batch = Task::batch(batch);
        (app, batch)
    }

    fn nav_bar(&self) -> Option<Element<Message>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        // let nav_model = self.nav_model()?;

        // let mut nav = iced::widget::nav_bar(nav_model, |id| {
        //     iced::Action::Iced(iced::app::Action::NavBar(id))
        // })
        // .on_dnd_drop::<ServiceItem>(|entity, data, action| {
        //     debug!(?entity);
        //     debug!(?data);
        //     debug!(?action);
        //     iced::Action::App(Message::DndDrop)
        // })
        // .on_dnd_enter(|entity, data| {
        //     debug!("entered");
        //     iced::Action::App(Message::DndEnter(entity, data))
        // })
        // .on_dnd_leave(|entity| {
        //     debug!("left");
        //     iced::Action::App(Message::DndLeave(entity))
        // })
        // .drag_id(DragId::new())
        // .on_context(|id| {
        //     iced::Action::Iced(
        //         iced::app::Action::NavBarContext(id),
        //     )
        // })
        // .context_menu(None)
        // .into_container()
        // // XXX both must be shrink to avoid flex layout from ignoring it
        // .width(Length::Shrink)
        // .height(Length::Shrink);

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
                    .class(iced::theme::style::Button::HeaderBar)
                    .padding(5)
                    .width(Length::Fill)
                    .on_press(iced::Action::App(Message::ChangeServiceItem(index)));
                let tooltip = tooltip(button,
                                      text::body(item.kind.to_string()),
                                      TPosition::Right);
                dnd_destination(tooltip, vec!["application/service-item".into()])
                    .data_received_for::<ServiceItem>( move |item| {
                        if let Some(item) = item {
                            iced::Action::App(Message::AddServiceItem(index, item))
                        } else {
                            iced::Action::None
                        }
                    }).on_drop(move |x, y| {
                        debug!(x, y);
                        iced::Action::App(Message::AddServiceItemDrop(index))
                    }).on_finish(move |mime, data, action, x, y| {
                        debug!(mime, ?data, ?action, x, y);
                        let Ok(item) = ServiceItem::try_from((data, mime)) else {
                            return iced::Action::None;
                        };
                        debug!(?item);
                        iced::Action::App(Message::AddServiceItem(index, item))
                    })
                    .into()
            });

        let end_index = self.service.len();
        let column = column![
            text::heading("Service List").center().width(280),
            column(list).spacing(10),
            dnd_destination(
                vertical_space(),
                vec!["application/service-item".into()]
            )
            .data_received_for::<ServiceItem>(|item| {
                if let Some(item) = item {
                    iced::Action::App(Message::AppendServiceItem(
                        item,
                    ))
                } else {
                    iced::Action::None
                }
            })
            .on_finish(
                move |mime, data, action, x, y| {
                    debug!(mime, ?data, ?action, x, y);
                    let Ok(item) =
                        ServiceItem::try_from((data, mime))
                    else {
                        return iced::Action::None;
                    };
                    debug!(?item);
                    iced::Action::App(Message::AddServiceItem(
                        end_index, item,
                    ))
                }
            )
        ]
        .padding(10)
        .spacing(10);
        let padding = Padding::new(0.0).top(20);
        let mut container = Container::new(column)
            // .height(Length::Fill)
            .style(nav_bar_style)
            .padding(padding);

        if !self.core().is_condensed() {
            container = container.max_width(280);
        }
        Some(container.into())
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        vec![search_input("Search...", "")
            .on_input(|_| Message::None)
            .on_submit(|_| Message::None)
            .on_focus(Message::SearchFocus)
            .width(1200)
            .into()]
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
                .class(iced::theme::style::Button::HeaderBar)
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
                .class(iced::theme::style::Button::HeaderBar)
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
                .class(iced::theme::style::Button::HeaderBar)
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
        let total_slides_text =
            format!("Total Slides: {}", self.presenter.total_slides);
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
    ) -> Option<iced::app::context_drawer::ContextDrawer<Self::Message>>
    {
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
            Some(text("hello").into())
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
                            iced::Action::App(Message::SongEditor(m))
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
                        iced::Action::App(Message::Present(m))
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
                                            iced::Action::App(
                                                Message::Present(m),
                                            )
                                        }))
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
                                        ) => tasks.push(task.map(
                                            |m| {
                                                iced::Action::App(
                                                    Message::Present(
                                                        m,
                                                    ),
                                                )
                                            },
                                        )),
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
                                            iced::Action::App(
                                                Message::Present(m),
                                            )
                                        }))
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
                                        ) => tasks.push(task.map(
                                            |m| {
                                                iced::Action::App(
                                                    Message::Present(
                                                        m,
                                                    ),
                                                )
                                            },
                                        )),
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
                                iced::Action::App(Message::Library(
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
                _ = self.set_window_title(
                    format!("window_{}", count),
                    id,
                );

                spawn_window.map(|id| {
                    iced::Action::App(Message::WindowOpened(id, None))
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
                                            || id > self.core.main_window_id().expect("Iced core seems to be missing a main window, was this started in cli mode?")
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
            Message::Quit => iced::iced::exit(),
            Message::DndEnter(entity, data) => {
                debug!(?entity);
                debug!(?data);
                Task::none()
            }
            Message::DndDrop => {
                // debug!(?entity);
                // debug!(?action);
                // debug!(?service_item);

                if let Some(library) = &self.library
                    && let Some((lib, item)) = library.dragged_item
                {
                    // match lib {
                    //     core::model::LibraryKind::Song => ,
                    //     core::model::LibraryKind::Video => todo!(),
                    //     core::model::LibraryKind::Image => todo!(),
                    //     core::model::LibraryKind::Presentation => todo!(),
                    // }
                    let item = library.get_song(item).unwrap();
                    let item = ServiceItem::from(item);
                    self.nav_model
                        .insert()
                        .text(item.title.clone())
                        .data(item);
                }
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
            Message::SearchFocus => {
                self.searching = true;
                Task::none()
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
            Message::AddServiceItem(index, item) => {
                self.service.insert(index, item);
                Task::none()
            }
            Message::AddServiceItemDrop(index) => {
                if let Some(item) = &self.library_dragged_item {
                    self.service.insert(index, item.clone());
                }
                Task::none()
            }
            Message::AppendServiceItem(item) => {
                self.service.push(item);
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

        let library = if self.library_open {
            Container::new(if let Some(library) = &self.library {
                library.view().map(Message::Library)
            } else {
                Space::new(0, 0).into()
            })
            .style(nav_bar_style)
            .center(Length::FillPortion(2))
        } else {
            Container::new(horizontal_space().width(0))
        };

        let song_editor =
            self.song_editor.view().map(Message::SongEditor);

        let row = row![
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
            library
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
    Self: iced::Application,
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
            iced::Action::App(Message::WindowOpened(id, None))
        })
    }

    fn add_library(&mut self) -> Task<Message> {
        Task::perform(async move { Library::new().await }, |x| {
            iced::Action::App(Message::AddLibrary(x))
        })
    }

    // fn add_service(
    //     &mut self,
    //     items: Vec<ServiceItem>,
    // ) -> Task<Message> {
    //     Task::perform(
    //         async move {
    //             for item in items {
    //                 debug!(?item, "Item to be appended");
    //                 let slides = item.to_slides().unwrap_or(vec![]);
    //                 map.insert(item, slides);
    //             }
    //             let len = map.len();
    //             debug!(len, "to be append: ");
    //             map
    //         },
    //         |x| {
    //             let len = x.len();
    //             debug!(len, "to append: ");
    //             iced::Action::App(Message::AppendService(x))
    //         },
    //     )
    // }

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

    fn theme() -> Theme {
        Theme::custom(
            "Snazzy",
            Palette {
                background: color!(0x282a36),
                text: color!(0xe2e4e5),
                primary: color!(0x57c7ff),
                success: color!(0x5af78e),
                warning: color!(0xff9f43),
                danger: color!(0xff5c57),
            },
        )
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
