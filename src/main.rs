use clap::{Parser, command};
use core::service_items::ServiceItem;
use core::slide::{
    Background, BackgroundKind, Slide, SlideBuilder, TextAlignment,
};
use cosmic::app::context_drawer::ContextDrawer;
use cosmic::app::{Core, Settings, Task};
use cosmic::cosmic_config::{Config, CosmicConfigEntry};
use cosmic::dialog::file_chooser::save;
use cosmic::iced::alignment::Vertical;
use cosmic::iced::keyboard::{Key, Modifiers};
use cosmic::iced::window::{Mode, Position};
use cosmic::iced::{
    self, Background as IcedBackground, Border, Color, Length, event,
    window,
};
use cosmic::iced_core::text::Wrapping;
use cosmic::iced_futures::Subscription;
use cosmic::iced_widget::{column, row, stack};
use cosmic::widget::dnd_destination::dnd_destination;
use cosmic::widget::menu::key_bind::Modifier;
use cosmic::widget::menu::{ItemWidth, KeyBind};
use cosmic::widget::nav_bar::nav_bar_style;
use cosmic::widget::tooltip::Position as TPosition;
use cosmic::widget::{
    Container, divider, menu, settings, text_input,
};
use cosmic::widget::{
    Space, button, context_menu, horizontal_space, mouse_area,
    nav_bar, nav_bar_toggle, responsive, scrollable, search_input,
    tooltip,
};
use cosmic::widget::{container, text};
use cosmic::widget::{icon, slider};
use cosmic::{Application, ApplicationExt, Apply, Element, executor};
use cosmic::{cosmic_config, theme};
use crisp::types::Value;
use lisp::parse_lisp;
use miette::{IntoDiagnostic, Result, miette};
use rayon::prelude::*;
use resvg::usvg::fontdb;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, level_filters::LevelFilter};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use ui::EditorMode;
use ui::library::{self, Library};
use ui::presenter::{self, Presenter};
use ui::song_editor::{self, SongEditor};

use crate::core::content::Content;
use crate::core::file;
use crate::core::kinds::ServiceItemKind;
use crate::core::model::KindWrapper;
use crate::ui::image_editor::{self, ImageEditor};
use crate::ui::presentation_editor::{self, PresentationEditor};
use crate::ui::text_svg::{self};
use crate::ui::video_editor::{self, VideoEditor};
use crate::ui::widgets::draggable;

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
    file: Option<PathBuf>,
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

    let (config_handler, config) = match cosmic_config::Config::new(
        App::APP_ID,
        core::settings::SETTINGS_VERSION,
    ) {
        Ok(config_handler) => {
            let config = match core::settings::Settings::get_entry(
                &config_handler,
            ) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    error!("errors loading settings: {:?}", errs);
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            error!("failed to create settings handler: {}", err);
            (None, core::settings::Settings::default())
        }
    };

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

    cosmic::app::run::<App>(settings, (args, config_handler, config))
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
    selected_items: Vec<usize>,
    current_item: (usize, usize),
    hovered_item: Option<usize>,
    hovered_dnd: Option<usize>,
    presentation_open: bool,
    cli_mode: bool,
    library: Option<Library>,
    library_open: bool,
    editor_mode: Option<EditorMode>,
    song_editor: SongEditor,
    video_editor: VideoEditor,
    image_editor: ImageEditor,
    presentation_editor: PresentationEditor,
    searching: bool,
    search_query: String,
    search_results: Vec<ServiceItemKind>,
    search_id: cosmic::widget::Id,
    library_dragged_item: Option<ServiceItem>,
    fontdb: Arc<fontdb::Database>,
    menu_keys: HashMap<KeyBind, MenuAction>,
    context_menu: Option<usize>,
    modifiers_pressed: Option<Modifiers>,
    settings_open: bool,
    settings: core::settings::Settings,
    config_handler: Option<Config>,
    obs_connection: String,
}

#[derive(Debug, Clone)]
enum Message {
    Present(presenter::Message),
    Library(library::Message),
    SongEditor(song_editor::Message),
    VideoEditor(video_editor::Message),
    ImageEditor(image_editor::Message),
    PresentationEditor(presentation_editor::Message),
    File(PathBuf),
    OpenWindow,
    CloseWindow(Option<window::Id>),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    AddLibrary(Library),
    LibraryToggle,
    Quit,
    Key(Key, Modifiers),
    None,
    EditorToggle(bool),
    ChangeServiceItem(usize),
    SelectServiceItem(usize),
    AddSelectServiceItem(usize),
    HoveredServiceItem(Option<usize>),
    HoveredServiceDrop(Option<usize>),
    AddServiceItem(usize, KindWrapper),
    AddServiceItemsFiles(usize, Vec<ServiceItem>),
    RemoveServiceItem(usize),
    AddServiceItemDrop(usize),
    AppendServiceItem(ServiceItem),
    AppendServiceItemKind(ServiceItemKind),
    ReorderService(usize, usize),
    ContextMenuItem(usize),
    SearchFocus,
    Search(String),
    CloseSearch,
    UpdateSearchResults(Vec<ServiceItemKind>),
    OpenEditor(ServiceItem),
    OpenEditorKind(ServiceItemKind),
    New,
    Open,
    OpenFile(PathBuf),
    Save,
    SaveAsDialog,
    SaveAs(PathBuf),
    OpenSettings,
    CloseSettings,
    SetObsUrl(String),
    SetObsConnection(String),
    ModifiersPressed(Modifiers),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    New,
    Save,
    SaveAs,
    Open,
    OpenSettings,
    DeleteItem(usize),
}

impl menu::Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::New => Message::New,
            MenuAction::Save => Message::Save,
            MenuAction::SaveAs => Message::SaveAsDialog,
            MenuAction::Open => Message::Open,
            MenuAction::OpenSettings => Message::OpenSettings,
            MenuAction::DeleteItem(index) => {
                Message::RemoveServiceItem(*index)
            }
        }
    }
}

const HEADER_SPACE: u16 = 6;

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = (
        Cli,
        Option<cosmic_config::Config>,
        core::settings::Settings,
    );
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

        if input.0.ui {
            windows.push(core.main_window_id().unwrap());
        }

        let (config_handler, settings) = (input.1, input.2);

        let items = if let Some(file) = input.0.file {
            match read_to_string(file) {
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
                                let mut inner_vector =
                                    parse_lisp(value);
                                service_items
                                    .append(&mut inner_vector);
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
            }
        } else {
            vec![]
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
        let presenter_obs_task = Task::perform(
            async {
                obws::Client::connect("localhost", 4455, Some(""))
                    .await
            },
            |res| match res {
                Ok(client) => cosmic::Action::App(Message::Present(
                    presenter::Message::AddObsClient(Arc::new(
                        client,
                    )),
                )),
                Err(e) => {
                    warn!("Obs may not be running: {e}");
                    cosmic::Action::None
                }
            },
        );

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
                key: Key::Character(",".into()),
            },
            MenuAction::OpenSettings,
        );
        // nav_model.activate_position(0);
        let mut app = Self {
            presenter,
            core,
            nav_model,
            service: items,
            selected_items: vec![],
            file: PathBuf::default(),
            windows,
            presentation_open: false,
            cli_mode: !input.0.ui,
            library: None,
            library_open: true,
            editor_mode: None,
            song_editor,
            video_editor: VideoEditor::new(),
            image_editor: ImageEditor::new(),
            presentation_editor: PresentationEditor::new(),
            searching: false,
            search_results: vec![],
            search_query: String::new(),
            search_id: cosmic::widget::Id::unique(),
            current_item: (0, 0),
            library_dragged_item: None,
            fontdb: Arc::clone(&fontdb),
            menu_keys,
            hovered_item: None,
            hovered_dnd: None,
            context_menu: None,
            modifiers_pressed: None,
            settings_open: false,
            settings,
            config_handler,
            obs_connection: "".into(),
        };

        let mut batch = vec![];
        batch.push(presenter_obs_task);

        if input.0.ui {
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
                    menu::Item::Button(
                        "New",
                        Some(
                            icon::from_name("document-new")
                                .symbolic(true)
                                .into(),
                        ),
                        MenuAction::New,
                    ),
                    menu::Item::Button(
                        "Open",
                        Some(
                            icon::from_name("document-open")
                                .symbolic(true)
                                .into(),
                        ),
                        MenuAction::Open,
                    ),
                    menu::Item::Button(
                        "Save",
                        Some(
                            icon::from_name("document-save")
                                .symbolic(true)
                                .into(),
                        ),
                        MenuAction::Save,
                    ),
                    menu::Item::Button(
                        "Save As",
                        Some(
                            icon::from_name("document-save-as")
                                .symbolic(true)
                                .into(),
                        ),
                        MenuAction::SaveAs,
                    ),
                ],
            ),
        );
        let settings_menu = menu::Tree::with_children(
            Into::<Element<Message>>::into(
                menu::root("Settings").on_press(Message::None),
            ),
            menu::items(
                &self.menu_keys,
                vec![menu::Item::Button(
                    "Open Settings",
                    Some(
                        icon::from_name("settings")
                            .symbolic(true)
                            .into(),
                    ),
                    MenuAction::OpenSettings,
                )],
            ),
        );
        let menu_bar =
            menu::bar::<Message>(vec![file_menu, settings_menu])
                .item_width(ItemWidth::Uniform(250))
                .path_highlight(Some(menu::PathHighlight::Full))
                .main_offset(10);
        let library_button = tooltip(
            nav_bar_toggle().on_toggle(Message::LibraryToggle),
            if self.library_open {
                "Hide library"
            } else {
                "Show library"
            },
            TPosition::Bottom,
        )
        .gap(cosmic::theme::spacing().space_xs);
        vec![library_button.into(), menu_bar.into()]
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
            )
            .gap(cosmic::theme::spacing().space_xs),
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
            )
            .gap(cosmic::theme::spacing().space_xs),
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
            )
            .gap(cosmic::theme::spacing().space_xs),
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
        event::listen_with(|event, status, id| {
            // debug!(?event);
            match status {
                event::Status::Ignored => {
                    match event {
                        iced::Event::Keyboard(event) => match event {
                            iced::keyboard::Event::KeyReleased {
                                key,
                                modifiers,
                                ..
                            } => Some(Message::Key(key, modifiers)),
                            iced::keyboard::Event::ModifiersChanged(
                                modifiers,
                            ) => Some(Message::ModifiersPressed(modifiers)),
                            _ => None,
                        },
                        iced::Event::Mouse(_event) => None,
                        iced::Event::Window(window_event) => {
                            match window_event {
                                window::Event::CloseRequested => {
                                    debug!("Closing window");
                                    Some(Message::CloseWindow(Some(id)))
                                }
                                window::Event::Opened { .. } => {
                                    debug!(?window_event, ?id);
                                    Some(Message::WindowOpened(id))
                                }
                                window::Event::Closed => {
                                    debug!("Closed window");
                                    Some(Message::WindowClosed(id))
                                }
                                window::Event::FileHovered(file) => {
                                    debug!(?file);
                                    None
                                }
                                window::Event::FileDropped(file) => {
                                    debug!(?file);
                                    None
                                }
                                _ => None,
                            }
                        }
                        iced::Event::Touch(_touch) => None,
                        iced::Event::A11y(_id, _action_request) => None,
                        iced::Event::Dnd(_dnd_event) => {
                            // debug!(?dnd_event);
                            None
                        }
                        iced::Event::PlatformSpecific(_platform_specific) => {
                            // debug!(?platform_specific);
                            None
                        }
                    }
                }
                event::Status::Captured => None,
            }
        })
    }

    fn context_drawer(
        &self,
    ) -> Option<
        cosmic::app::context_drawer::ContextDrawer<Self::Message>,
    > {
        None
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        let cosmic::cosmic_theme::Spacing {
            space_none,
            space_xxxs,
            space_xxs,
            space_xs,
            space_s,
            space_m,
            space_l,
            space_xl,
            space_xxl,
            space_xxxl,
        } = cosmic::theme::spacing();
        if self.searching {
            let items: Vec<Element<Message>> = self
                .search_results
                .iter()
                .map(|item| {
                    let title = text::title4(item.title().clone());
                    let subtitle = text::body(item.to_string());
                    Element::from(
                        row![
                            column![title, subtitle]
                                .spacing(space_xxs),
                            horizontal_space(),
                            tooltip(
                                icon::from_name("add")
                                    .symbolic(true).apply(button::icon)
                                    .icon_size(space_l)
                                    .on_press(
                                        Message::AppendServiceItemKind(
                                            item.clone()
                                        )
                                    ),
                                "Add to service",
                                TPosition::FollowCursor
                            ),
                            tooltip(
                                icon::from_name("edit")
                                    .symbolic(true).apply(button::icon)
                                    .icon_size(space_l)
                                    .on_press(Message::OpenEditorKind(
                                        item.clone()
                                    )),
                                "Edit Item",
                                TPosition::FollowCursor
                            ),
                        ]
                        .align_y(Vertical::Center)
                        .apply(container),
                    )
                })
                .collect();
            let modal = column![
                search_input("Amazing Grace", &self.search_query)
                    .id(self.search_id.clone())
                    .select_on_focus(true)
                    .on_input(Message::Search)
                    .on_submit(Message::Search),
                column(items).spacing(space_xxs)
            ]
            .spacing(space_s)
            .apply(container)
            .padding(space_xl)
            .style(nav_bar_style);
            let modal = mouse_area(modal)
                .on_press(Message::None)
                .apply(container)
                .padding([space_xxl, space_xxxl * 2])
                .center_x(Length::Fill)
                .align_top(Length::Fill);
            let mouse_stack = stack!(
                Space::new(Length::Fill, Length::Fill)
                    .apply(container)
                    .style(|_| {
                        container::background(
                            cosmic::iced::Background::Color(
                                Color::BLACK,
                            )
                            .scale_alpha(0.3),
                        )
                    })
                    .apply(mouse_area)
                    .on_press(Message::CloseSearch),
                modal
            );
            Some(mouse_stack.into())
        } else if self.settings_open {
            let obs_socket = settings::item(
                "Obs Connection",
                text_input("127.0.0.1", &self.obs_connection)
                    .select_on_focus(true)
                    .on_input(Message::SetObsConnection)
                    .on_submit(Message::SetObsConnection),
            );
            let apply_button = settings::item::builder("").control(
                button::standard("Connect").on_press(
                    Message::SetObsUrl(self.obs_connection.clone()),
                ),
            );
            let settings_column = column![
                icon::from_name("dialog-close")
                    .symbolic(true)
                    .prefer_svg(true)
                    .apply(button::icon)
                    .class(theme::Button::Icon)
                    .on_press(Message::CloseSettings)
                    .apply(container)
                    .padding(space_s)
                    .align_right(Length::Fill)
                    .align_top(60),
                horizontal_space().height(space_xxl),
                settings::section()
                    .title("Obs Settings")
                    .add(obs_socket)
                    .add(apply_button)
                    .apply(container)
                    .center_x(Length::Fill)
                    .align_top(Length::Fill)
                    .padding([0, space_xxxl * 2])
            ]
            .height(Length::Fill);
            let settings_container = settings_column
                .apply(container)
                .style(nav_bar_style)
                .center_x(Length::Fill)
                .align_top(Length::Fill);
            let modal = mouse_area(settings_container)
                .on_press(Message::None)
                .apply(container)
                .padding([space_xxl, space_xxxl * 2]);
            let mouse_stack = stack!(
                Space::new(Length::Fill, Length::Fill)
                    .apply(container)
                    .style(|_| {
                        container::background(
                            cosmic::iced::Background::Color(
                                Color::BLACK,
                            )
                            .scale_alpha(0.3),
                        )
                    })
                    .apply(mouse_area)
                    .on_press(Message::CloseSettings),
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
                        if self.library.is_some() {
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
            Message::ImageEditor(message) => {
                match self.image_editor.update(message) {
                    image_editor::Action::Task(task) => {
                        task.map(|m| {
                            cosmic::Action::App(Message::ImageEditor(
                                m,
                            ))
                        })
                    }
                    image_editor::Action::UpdateImage(image) => {
                        if self.library.is_some() {
                            self.update(Message::Library(
                                library::Message::UpdateImage(image),
                            ))
                        } else {
                            Task::none()
                        }
                    }
                    image_editor::Action::None => Task::none(),
                }
            }
            Message::VideoEditor(message) => {
                match self.video_editor.update(message) {
                    video_editor::Action::Task(task) => {
                        task.map(|m| {
                            cosmic::Action::App(Message::VideoEditor(
                                m,
                            ))
                        })
                    }
                    video_editor::Action::UpdateVideo(video) => {
                        if self.library.is_some() {
                            self.update(Message::Library(
                                library::Message::UpdateVideo(video),
                            ))
                        } else {
                            Task::none()
                        }
                    }
                    video_editor::Action::None => Task::none(),
                }
            }
            Message::PresentationEditor(message) => {
                match self.presentation_editor.update(message) {
                    presentation_editor::Action::Task(task) => {
                        task.map(|m| {
                            cosmic::Action::App(Message::PresentationEditor(
                                m,
                            ))
                        })
                    }
                    presentation_editor::Action::UpdatePresentation(presentation) => {
                        if self.library.is_some() {
                            self.update(Message::Library(
                                library::Message::UpdatePresentation(presentation),
                            ))
                        } else {
                            Task::none()
                        }
                    }
                    presentation_editor::Action::None => Task::none(),
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
                    presenter::Action::ChangeSlide(
                        item_index,
                        slide_index,
                    ) => {
                        self.current_item = (item_index, slide_index);
                        let action = self.presenter.update(
                            presenter::Message::ActivateSlide(
                                item_index,
                                slide_index,
                            ),
                        );

                        if let presenter::Action::Task(task) = action
                        {
                            task.map(|m| {
                                cosmic::Action::App(Message::Present(
                                    m,
                                ))
                            })
                        } else {
                            Task::none()
                        }
                    }
                    presenter::Action::NextSlide => {
                        let slide_index = self.current_item.1;
                        let item_index = self.current_item.0;
                        let mut tasks = vec![];
                        debug!(slide_index, item_index);
                        if let Some(item) =
                            self.service.get(item_index)
                        {
                            if item.slides.len() > slide_index + 1 {
                                let slide_index = slide_index + 1;
                                debug!(slide_index, item_index);
                                let action = self.presenter.update(
                                    presenter::Message::ActivateSlide(
                                        item_index,
                                        slide_index,
                                    ),
                                );
                                if let presenter::Action::Task(task) =
                                    action
                                {
                                    tasks.push(task.map(|m| {
                                        cosmic::Action::App(
                                            Message::Present(m),
                                        )
                                    }));
                                }
                                self.current_item =
                                    (item_index, slide_index);
                                Task::batch(tasks)
                            } else {
                                // debug!("Slides are not longer");
                                if self
                                    .service
                                    .get(item_index + 1)
                                    .is_some()
                                {
                                    let action = self.presenter.update(presenter::Message::ActivateSlide(self.current_item.0, self.current_item.1));
                                    if let presenter::Action::Task(
                                        task,
                                    ) = action
                                    {
                                        tasks.push(task.map(|m| {
                                            cosmic::Action::App(
                                                Message::Present(m),
                                            )
                                        }));
                                    }
                                    self.current_item =
                                        (item_index + 1, 0);
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
                                let slide_index = slide_index - 1;
                                let action = self.presenter.update(
                                    presenter::Message::ActivateSlide(
                                        item_index,
                                        slide_index,
                                    ),
                                );
                                if let presenter::Action::Task(task) =
                                    action
                                {
                                    tasks.push(task.map(|m| {
                                        cosmic::Action::App(
                                            Message::Present(m),
                                        )
                                    }));
                                }
                                self.current_item =
                                    (item_index, slide_index);
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
                                if self
                                    .service
                                    .get(item_index - 1)
                                    .is_some()
                                {
                                    let action = self.presenter.update(presenter::Message::ActivateSlide(self.current_item.0, self.current_item.1));
                                    if let presenter::Action::Task(
                                        task,
                                    ) = action
                                    {
                                        tasks.push(task.map(|m| {
                                            cosmic::Action::App(
                                                Message::Present(m),
                                            )
                                        }));
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
                                    match kind {
                                        core::model::LibraryKind::Song => {
                                            let Some(lib_song) = library.get_song(index) else {
                                                return Task::none();
                                            };
                                            self.editor_mode = Some(kind.into());
                                            let song = lib_song.to_owned();
                                            return self.update(Message::SongEditor(
                                                song_editor::Message::ChangeSong(song),
                                            ));
                                        },
                                        core::model::LibraryKind::Video => {
                                            let Some(lib_video) = library.get_video(index) else {
                                                return Task::none();
                                            };
                                            self.editor_mode = Some(kind.into());
                                            let video = lib_video.to_owned();
                                            return self.update(Message::VideoEditor(video_editor::Message::ChangeVideo(video)));
                                        },
                                        core::model::LibraryKind::Image => {
                                            let Some(lib_image) = library.get_image(index) else {
                                                return Task::none();
                                            };
                                            self.editor_mode = Some(kind.into());
                                            let image = lib_image.to_owned();
                                            return self.update(Message::ImageEditor(image_editor::Message::ChangeImage(image)));
                                        },
                                        core::model::LibraryKind::Presentation => {
                                            let Some(lib_presentation) = library.get_presentation(index) else {
                                                return Task::none();
                                            };
                                            self.editor_mode = Some(kind.into());
                                            let presentation = lib_presentation.to_owned();
                                            return self.update(Message::PresentationEditor(presentation_editor::Message::ChangePresentation(presentation)));
                                        },
                                    }
                                }
                                library::Action::DraggedItem(
                                    service_item,
                                ) => {
                                    debug!("hi");
                                    self.library_dragged_item =
                                        Some(service_item);
                                }
                            }
                }
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
                        exit_on_close_request: count
                            .is_multiple_of(2),
                        decorations: false,
                        ..Default::default()
                    });

                self.windows.push(id);
                _ = self
                    .set_window_title(format!("window_{count}"), id);

                spawn_window.map(|id| {
                    cosmic::Action::App(Message::WindowOpened(id))
                })
            }
            Message::CloseWindow(id) => {
                if let Some(id) = id {
                    window::close(id)
                } else {
                    Task::none()
                }
            }
            Message::WindowOpened(id) => {
                debug!(?id, "Window opened");
                let radii =
                    self.core.sync_window_border_radii_to_theme();
                self.core.set_sync_window_border_radii_to_theme(true);
                debug!(radii);
                let radii = self.core.window.sharp_corners;
                debug!(radii);
                self.core.window.sharp_corners = true;
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
                self.settings_open = false;
                self.searching = true;
                cosmic::widget::text_input::focus(
                    self.search_id.clone(),
                )
            }
            Message::HoveredServiceItem(index) => {
                self.hovered_item = index;
                Task::none()
            }
            Message::HoveredServiceDrop(index) => {
                self.hovered_dnd = index;
                Task::none()
            }
            Message::SelectServiceItem(index) => {
                self.selected_items = vec![index];
                Task::none()
            }
            Message::AddSelectServiceItem(index) => {
                self.selected_items.push(index);
                Task::none()
            }
            Message::ChangeServiceItem(index) => {
                if let Some((index, item)) = self
                    .service
                    .iter()
                    .enumerate()
                    .find(|(id, _)| index == *id)
                    && let Some(_slide) = item.slides.first()
                {
                    self.current_item = (index, 0);
                    self.presenter.update(
                        presenter::Message::ActivateSlide(
                            self.current_item.0,
                            self.current_item.1,
                        ),
                    );
                }
                Task::none()
            }
            Message::AddServiceItem(index, item) => {
                let item_index = item.0.1;
                let kind = item.0.0;
                let mut item;
                match kind {
                    core::model::LibraryKind::Song => {
                        let Some(library) = self.library.as_mut()
                        else {
                            return Task::none();
                        };
                        let Some(song) = library.get_song(item_index)
                        else {
                            return Task::none();
                        };
                        item = song.to_service_item();
                    }
                    core::model::LibraryKind::Video => {
                        let Some(library) = self.library.as_mut()
                        else {
                            return Task::none();
                        };
                        let Some(video) =
                            library.get_video(item_index)
                        else {
                            return Task::none();
                        };
                        item = video.to_service_item();
                    }
                    core::model::LibraryKind::Image => {
                        let Some(library) = self.library.as_mut()
                        else {
                            return Task::none();
                        };
                        let Some(image) =
                            library.get_image(item_index)
                        else {
                            return Task::none();
                        };
                        item = image.to_service_item();
                    }
                    core::model::LibraryKind::Presentation => {
                        let Some(library) = self.library.as_mut()
                        else {
                            return Task::none();
                        };
                        let Some(presentation) =
                            library.get_presentation(item_index)
                        else {
                            return Task::none();
                        };
                        item = presentation.to_service_item();
                    }
                }
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
                self.hovered_dnd = None;
                Task::none()
            }
            Message::AddServiceItemsFiles(index, items) => {
                self.hovered_dnd = None;
                for item in items {
                    self.service.insert(index, item);
                }
                self.presenter.update_items(self.service.clone());
                Task::none()
            }
            Message::RemoveServiceItem(index) => {
                self.service.remove(index);
                self.presenter.update_items(self.service.clone());
                Task::none()
            }
            Message::ContextMenuItem(index) => {
                self.context_menu = Some(index);
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
            Message::AppendServiceItemKind(item) => {
                let item = item.to_service_item();
                self.update(Message::AppendServiceItem(item))
            }
            Message::ReorderService(index, target_index) => {
                let item = self.service.remove(index);
                self.service.insert(target_index, item);
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
                self.search_query = String::new();
                self.search_results = vec![];
                self.searching = false;
                match kind {
                    ServiceItemKind::Song(song) => {
                        self.editor_mode = Some(EditorMode::Song);
                        self.update(Message::SongEditor(
                            song_editor::Message::ChangeSong(song),
                        ))
                    }
                    ServiceItemKind::Video(video) => {
                        self.editor_mode = Some(EditorMode::Video);
                        self.update(Message::VideoEditor(
                            video_editor::Message::ChangeVideo(video),
                        ))
                    }
                    ServiceItemKind::Image(image) => {
                        self.editor_mode = Some(EditorMode::Image);
                        self.update(Message::ImageEditor(
                            image_editor::Message::ChangeImage(image),
                        ))
                    }
                    ServiceItemKind::Presentation(presentation) => {
                        self.editor_mode =
                            Some(EditorMode::Presentation);
                        self.update(Message::PresentationEditor(
                            presentation_editor::Message::ChangePresentation(presentation),
                        ))
                    }
                    ServiceItemKind::Content(_slide) => todo!(),
                }
            }
            Message::OpenEditorKind(item) => {
                let item = item.to_service_item();
                self.update(Message::OpenEditor(item))
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
            Message::Save => {
                let service = self.service.clone();
                let file = self.file.clone();
                Task::perform(
                    file::save(service, file.clone()),
                    move |res| match res {
                        Ok(_) => {
                            tracing::info!(
                                "saving file to: {:?}",
                                file
                            );
                            cosmic::Action::None
                        }
                        Err(e) => {
                            error!(?e, "There was a problem saving");
                            cosmic::Action::None
                        }
                    },
                )
            }
            Message::SaveAs(file) => {
                debug!(?file, "saving as a file");
                self.file = file;
                return self.update(Message::Save);
            }
            Message::SaveAsDialog => {
                Task::perform(save_as_dialog(), |file| match file {
                    Ok(file) => {
                        cosmic::Action::App(Message::SaveAs(file))
                    }
                    Err(e) => {
                        error!(
                            ?e,
                            "There was an error during saving"
                        );
                        cosmic::Action::None
                    }
                })
            }
            Message::OpenSettings => {
                self.searching = false;
                self.settings_open = true;
                Task::none()
            }
            Message::CloseSettings => {
                self.settings_open = false;
                Task::none()
            }
            Message::SetObsUrl(url) => {
                if let Some(config) = &self.config_handler {
                    if let Err(e) = self.settings.set_obs_url(
                        &config,
                        url::Url::parse(&url).ok(),
                    ) {
                        error!(?e, "Can't write to disk obs url")
                    };
                };
                Task::none()
            }
            Message::SetObsConnection(url) => {
                self.obs_connection = url;
                Task::none()
            }
            Message::ModifiersPressed(modifiers) => {
                if modifiers.is_empty() {
                    self.modifiers_pressed = None;
                    if let Some(library) = self.library.as_mut() {
                        library.set_modifiers(None);
                    }
                    return Task::none();
                }
                if let Some(library) = self.library.as_mut() {
                    library.set_modifiers(Some(modifiers));
                }
                self.modifiers_pressed = Some(modifiers);
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

        let editor = self.editor_mode.as_ref().map_or_else(
            || Element::from(Space::new(0, 0)),
            |mode| match mode {
                EditorMode::Song => {
                    self.song_editor.view().map(Message::SongEditor)
                }
                EditorMode::Image => {
                    self.image_editor.view().map(Message::ImageEditor)
                }
                EditorMode::Video => {
                    self.video_editor.view().map(Message::VideoEditor)
                }
                EditorMode::Presentation => self
                    .presentation_editor
                    .view()
                    .map(Message::PresentationEditor),
                EditorMode::Slide => todo!(),
            },
        );

        let service_row = row![
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

        let preview_bar = if self.editor_mode.is_none() {
            if self.service.len() == 0 {
                Container::new(horizontal_space())
            } else {
                Container::new(
                    self.presenter
                        .preview_bar()
                        .map(Message::Present),
                )
                .clip(true)
                .width(Length::Fill)
                .center_y(180)
            }
        } else {
            Container::new(horizontal_space())
        };

        let main_area = self.editor_mode.as_ref().map_or_else(
            || Container::new(service_row).center_y(Length::Fill),
            |_| {
                container(editor)
                    .padding(cosmic::theme::spacing().space_xxl)
            },
        );

        let column = column![
            row![
                if self.library_open {
                    library.width(Length::FillPortion(1))
                } else {
                    container(Space::new(0, 0))
                },
                main_area.width(Length::FillPortion(4))
            ],
            preview_bar
        ];

        column.into()
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
        spawn_window
            .map(|id| cosmic::Action::App(Message::WindowOpened(id)))
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

    fn process_key_press(
        &mut self,
        key: Key,
        modifiers: Modifiers,
    ) -> Task<Message> {
        // debug!(?key, ?modifiers);
        // if self.editor_mode.is_some() {
        //     return Task::none();
        // }
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
                self.update(Message::Save)
            }
            (Key::Character(k), Modifiers::CTRL) if k == *"o" => {
                self.update(Message::Open)
            }
            (Key::Character(k), Modifiers::CTRL) if k == *"," => {
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
        let list =
            self.service.iter().enumerate().map(|(index, item)| {
                let icon = match item.kind {
                    ServiceItemKind::Song(_) => {
                        icon::from_name("folder-music-symbolic")
                    }
                    ServiceItemKind::Video(_) => {
                        icon::from_name("folder-videos-symbolic")
                    }
                    ServiceItemKind::Image(_) => {
                        icon::from_name("folder-pictures-symbolic")
                    }
                    ServiceItemKind::Presentation(_) => {
                        icon::from_name(
                            "x-office-presentation-symbolic",
                        )
                    }
                    ServiceItemKind::Content(_) => icon::from_name(
                        "x-office-presentation-symbolic",
                    ),
                };
                let title = responsive(|size| {
                    text::heading(library::elide_text(
                        &item.title,
                        size.width,
                    ))
                    .align_y(Vertical::Center)
                    .wrapping(Wrapping::None)
                    .into()
                });
                let container = container(
                    row![icon, title]
                        .align_y(Vertical::Center)
                        .spacing(cosmic::theme::spacing().space_xs),
                )
                .height(cosmic::theme::spacing().space_xl)
                .padding(cosmic::theme::spacing().space_s)
                .class(cosmic::theme::style::Container::Secondary)
                .style(move |t| {
                    container::Style::default()
                        .background(IcedBackground::Color(
                            if self.hovered_item.is_some_and(
                                |hovered_index| {
                                    index == hovered_index
                                },
                            ) || self
                                .selected_items
                                .contains(&index)
                            {
                                t.cosmic().button.hover.into()
                            } else {
                                t.cosmic().button.base.into()
                            },
                        ))
                        .border(Border::default().rounded(
                            t.cosmic().corner_radii.radius_m,
                        ))
                })
                .width(Length::Fill);
                let visual_item = if self.hovered_dnd.is_some_and(|h| h == index) {
                    let divider = divider::horizontal::default().class(theme::Rule::custom(|t| {
                        let color = t.cosmic().accent_color();
                        let style = cosmic::iced_widget::rule::Style {
                            color: color.into(),
                            width: 2,
                            radius: t.cosmic().corner_radii.radius_xs.into(),
                            fill_mode: cosmic::iced_widget::rule::FillMode::Full,
                        };
                        style
                    } ));
                    Container::new(column![divider, container].spacing(theme::spacing().space_s))
                } else { container };
                let mouse_area = mouse_area(visual_item)
                    .on_enter(Message::HoveredServiceItem(Some(
                        index,
                    )))
                    .on_exit(Message::HoveredServiceItem(None))
                    .on_double_press(Message::ChangeServiceItem(
                        index,
                    ))
                    .on_right_press(Message::ContextMenuItem(index))
                    .on_release(Message::SelectServiceItem(index));
                let single_item = if let Some(context_menu_item) =
                    self.context_menu
                {
                    if context_menu_item == index {
                        let context_menu = context_menu(
                            mouse_area,
                            self.context_menu.map_or_else(
                                || None,
                                |i| {
                                    if i == index {
                                        let menu =
                                            vec![menu::Item::Button(
                                    "Delete",
                                    None,
                                    MenuAction::DeleteItem(index),
                                )];
                                        Some(menu::items(
                                            &HashMap::new(),
                                            menu,
                                        ))
                                    } else {
                                        None
                                    }
                                },
                            ),
                        )
                        .close_on_escape(true);
                        Element::from(context_menu)
                    } else {
                        Element::from(mouse_area)
                    }
                } else {
                    Element::from(mouse_area)
                };
                let tooltip = tooltip(
                    single_item,
                    text::body(item.kind.to_string()),
                    TPosition::Right,
                )
                .gap(cosmic::theme::spacing().space_xs);
                dnd_destination(
                    tooltip,
                    vec!["application/service-item".into(), "text/uri-list".into(), "x-special/gnome-copied-files".into()],
                )
                .on_enter(move |_, _, _| Message::HoveredServiceDrop(Some(index)))
                .on_leave(move || Message::HoveredServiceDrop(None))
                .on_finish(move |mime, data, _, _, _| {

                    match mime.as_str() {
                        "application/service-item" => {
                            let Ok(item) =
                                KindWrapper::try_from((data, mime))
                            else {
                                error!("couldn't drag in Service item");
                                return Message::None;
                            };
                            debug!(?item, index, "adding Service item");
                            Message::AddServiceItem(index, item)
                        }
                        "text/uri-list" => {
                            let Ok(text) = str::from_utf8(&data) else {
                                return Message::None;
                            };
                            let mut items = Vec::new();
                            for line in text.lines() {
                                let Ok(url) = url::Url::parse(line) else {
                                    error!(?line, "problem parsing this file url");
                                    continue;
                                };
                                let Ok(path) = url.to_file_path() else {
                                    error!(?url, "invalid file URL");
                                    continue;
                                };
                                let item = ServiceItem::try_from(path);
                                match item {
                                    Ok(item) => items.push(item),
                                    Err(e) => error!(?e),
                                }
                            }
                            Message::AddServiceItemsFiles(index, items)
                        }
                        _ => Message::None
                    }
                })
                .into()
            });

        let scrollable = scrollable(
            draggable::column::column(list)
                .spacing(10)
                .on_drag(|event| match event {
                    draggable::DragEvent::Picked { .. } => {
                        Message::None
                    }
                    draggable::DragEvent::Dropped {
                        index,
                        target_index,
                        ..
                    } => Message::ReorderService(index, target_index),
                    draggable::DragEvent::Canceled { .. } => {
                        Message::None
                    }
                })
                .style(|t| draggable::column::Style {
                    scale: 1.05,
                    moved_item_overlay: Color::from(
                        t.cosmic().primary.base,
                    )
                    .scale_alpha(0.2),
                    ghost_border: Border {
                        width: 1.0,
                        color: t.cosmic().secondary.base.into(),
                        radius: t.cosmic().radius_m().into(),
                    },
                    ghost_background: Color::from(
                        t.cosmic().secondary.base,
                    )
                    .scale_alpha(0.2)
                    .into(),
                })
                .height(Length::Shrink),
        )
        .anchor_top()
        .height(Length::Fill);

        let column = column![
            text::heading("Service List")
                .center()
                .width(Length::Fill),
            iced::widget::horizontal_rule(1),
            scrollable
        ]
        .padding(10)
        .spacing(10);
        let last_index = self.service.len();
        let container = Container::new(
            dnd_destination(
                column,
                vec![
                    "application/service-item".into(),
                    "text/uri-list".into(),
                ],
            )
            .on_finish(move |mime, data, _, _, _| {
                match mime.as_str() {
                    "application/service-item" => {
                        let Ok(item) =
                            KindWrapper::try_from((data, mime))
                        else {
                            error!("couldn't drag in Service item");
                            return Message::None;
                        };
                        debug!(?item, "adding Service item");
                        Message::AddServiceItem(last_index, item)
                    }
                    "text/uri-list" => {
                        let Ok(text) = str::from_utf8(&data) else {
                            return Message::None;
                        };
                        let mut items = Vec::new();
                        for line in text.lines() {
                            let Ok(url) = url::Url::parse(line)
                            else {
                                error!(
                                    ?line,
                                    "problem parsing this file url"
                                );
                                continue;
                            };
                            let Ok(path) = url.to_file_path() else {
                                error!(?url, "invalid file URL");
                                continue;
                            };
                            let item = ServiceItem::try_from(path);
                            match item {
                                Ok(item) => items.push(item),
                                Err(e) => error!(?e),
                            }
                        }
                        Message::AddServiceItemsFiles(
                            last_index, items,
                        )
                    }
                    _ => Message::None,
                }
            }),
        )
        .style(nav_bar_style);

        container.center(Length::FillPortion(2)).into()
    }
}

async fn save_as_dialog() -> Result<PathBuf> {
    let dialog = save::Dialog::new();
    save::file(dialog).await.into_diagnostic().map(|response| {
        match response.url() {
            Some(url) => Ok(url.to_file_path().unwrap()),
            None => {
                Err(miette!("Can't convert url of file to a path"))
            }
        }
    })?
}
