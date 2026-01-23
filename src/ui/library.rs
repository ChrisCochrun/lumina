use std::collections::HashMap;

use cosmic::{
    Element, Task,
    dialog::file_chooser::open::Dialog,
    iced::{
        Background, Border, Color, Length, alignment::Vertical,
        clipboard::dnd::DndAction, keyboard::Modifiers,
    },
    iced_core::widget::tree::State,
    iced_widget::{column, row as rowm, text as textm},
    theme,
    widget::{
        Container, DndSource, Space, button, container, context_menu,
        dnd_destination, horizontal_space, icon,
        menu::{self, Action as MenuAction},
        mouse_area, responsive, row, scrollable, text, text_input,
    },
};
use miette::{IntoDiagnostic, Result};
use rapidfuzz::distance::levenshtein;
use sqlx::{SqlitePool, migrate};
use tracing::{debug, error, warn};

use crate::core::{
    content::Content,
    images::{self, Image, add_image_to_db, update_image_in_db},
    kinds::ServiceItemKind,
    model::{KindWrapper, LibraryKind, Model},
    presentations::{
        self, Presentation, add_presentation_to_db,
        update_presentation_in_db,
    },
    service_items::ServiceItem,
    songs::{self, Song, add_song_to_db, update_song_in_db},
    videos::{self, Video, add_video_to_db, update_video_in_db},
};

#[derive(Debug, Clone)]
pub struct Library {
    song_library: Model<Song>,
    image_library: Model<Image>,
    video_library: Model<Video>,
    presentation_library: Model<Presentation>,
    library_open: Option<LibraryKind>,
    library_hovered: Option<LibraryKind>,
    selected_items: Option<Vec<(LibraryKind, i32)>>,
    hovered_item: Option<(LibraryKind, i32)>,
    editing_item: Option<(LibraryKind, i32)>,
    db: SqlitePool,
    menu_keys: std::collections::HashMap<menu::KeyBind, MenuMessage>,
    context_menu: Option<i32>,
    modifiers_pressed: Option<Modifiers>,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum MenuMessage {
    Delete,
    Open,
}

impl MenuAction for MenuMessage {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuMessage::Delete => Message::DeleteItem,
            MenuMessage::Open => Message::OpenContextItem,
        }
    }
}

pub enum Action {
    OpenItem(Option<(LibraryKind, i32)>),
    DraggedItem(ServiceItem),
    Task(Task<Message>),
    None,
}

#[derive(Clone, Debug)]
pub enum Message {
    AddItem,
    DeleteItem,
    OpenItem(Option<(LibraryKind, i32)>),
    OpenContextItem,
    HoverLibrary(Option<LibraryKind>),
    OpenLibrary(Option<LibraryKind>),
    HoverItem(Option<(LibraryKind, i32)>),
    SelectItem(Option<(LibraryKind, i32)>),
    DragItem(ServiceItem),
    UpdateSong(Song),
    SongChanged,
    UpdateImage(Image),
    ImageChanged,
    UpdateVideo(Video),
    VideoChanged,
    UpdatePresentation(Presentation),
    PresentationChanged,
    Error(String),
    OpenContext(i32),
    None,
    AddFiles(Vec<ServiceItemKind>),
    AddSong(Song),
    AddImages(Option<Vec<Image>>),
    AddVideos(Option<Vec<Video>>),
    AddPresentations(Option<Vec<Presentation>>),
}

impl<'a> Library {
    pub async fn new() -> Self {
        let mut db = add_db().await.expect("probs");
        if let Err(e) = migrate!("./migrations").run(&db).await {
            error!(?e)
        }
        Self {
            song_library: Model::new_song_model(&mut db).await,
            image_library: Model::new_image_model(&mut db).await,
            video_library: Model::new_video_model(&mut db).await,
            presentation_library: Model::new_presentation_model(
                &mut db,
            )
            .await,
            library_open: None,
            library_hovered: None,
            selected_items: None,
            hovered_item: None,
            editing_item: None,
            db,
            menu_keys: HashMap::new(),
            context_menu: None,
            modifiers_pressed: None,
        }
    }

    pub fn get_song(&self, index: i32) -> Option<&Song> {
        self.song_library.get_item(index)
    }

    pub fn update(&'a mut self, message: Message) -> Action {
        match message {
            Message::None => (),
            Message::DeleteItem => {
                return self.delete_items();
            }
            Message::AddSong(song) => {
                if let Err(e) = self.song_library.add_item(song) {
                    error!(?e, "couldn't add song to model");
                } else {
                    let index =
                        (self.song_library.items.len() - 1) as i32;
                    return Action::Task(Task::done(
                        Message::OpenItem(Some((
                            LibraryKind::Song,
                            index,
                        ))),
                    ));
                }
            }
            Message::AddItem => {
                let kind =
                    self.library_open.unwrap_or(LibraryKind::Song);
                match kind {
                    LibraryKind::Song => {
                        let song = Song::default();
                        let task = Task::future(self.db.acquire()).and_then(move |db| {
                            Task::perform(add_song_to_db(db), move |res| {
                                match res {
                                    Ok(song) => {
                                        Message::AddSong(song)
                                    },
                                    Err(e) => {error!(?e, "couldn't add song to db"); Message::None}
                                }
                            })
                        });
                        return Action::Task(task);
                    }
                    LibraryKind::Video => {
                        return Action::Task(Task::perform(
                            add_videos(),
                            Message::AddVideos,
                        ));
                    }
                    LibraryKind::Image => {
                        return Action::Task(Task::perform(
                            add_images(),
                            Message::AddImages,
                        ));
                    }
                    LibraryKind::Presentation => {
                        return Action::Task(Task::perform(
                            add_presentations(),
                            Message::AddPresentations,
                        ));
                    }
                };
            }
            Message::AddVideos(videos) => {
                debug!(?videos);
                let mut index = self.video_library.items.len();
                // Check if empty
                let mut tasks = Vec::new();
                if let Some(videos) = videos {
                    let len = videos.len();
                    for video in videos {
                        if let Err(e) =
                            self.video_library.add_item(video.clone())
                        {
                            error!(?e);
                        }
                        let task = Task::future(self.db.acquire())
                            .and_then(move |db| {
                                Task::perform(
                                    add_video_to_db(
                                        video.to_owned(),
                                        db,
                                    ),
                                    move |res| {
                                        debug!(
                                            len,
                                            index, "added to db"
                                        );
                                        if let Err(e) = res {
                                            error!(?e);
                                        }
                                        if len == index {
                                            debug!("open the pres");
                                            Message::OpenItem(Some((
                                                LibraryKind::Video,
                                                index as i32,
                                            )))
                                        } else {
                                            Message::None
                                        }
                                    },
                                )
                            });
                        tasks.push(task);
                        index += 1;
                    }
                }
                let after_task =
                    Task::done(Message::OpenItem(Some((
                        LibraryKind::Video,
                        self.video_library.items.len() as i32 - 1,
                    ))));
                return Action::Task(
                    Task::batch(tasks).chain(after_task),
                );
            }
            Message::AddPresentations(presentations) => {
                debug!(?presentations);
                let mut index = self.presentation_library.items.len();
                // Check if empty
                let mut tasks = Vec::new();
                if let Some(presentations) = presentations {
                    let len = presentations.len();
                    for presentation in presentations {
                        if let Err(e) = self
                            .presentation_library
                            .add_item(presentation.clone())
                        {
                            error!(?e);
                        }
                        let task = Task::future(
                                        self.db.acquire(),
                                    )
                                    .and_then(move |db| {
                                        Task::perform(
                                            add_presentation_to_db(
                                                presentation.to_owned(),
                                                db,
                                            ),
                                            move |res| {
                                                debug!(
                                                    len,
                                                    index, "added to db"
                                                );
                                                if let Err(e) = res {
                                                    error!(?e);
                                                }
                                                if len == index {
                                                    debug!("open the pres");
                                                    Message::OpenItem(Some((
                                                    LibraryKind::Presentation,
                                                    index as i32,
                                                )))
                                                } else {
                                                    Message::None
                                                }
                                            },
                                        )
                                    });
                        tasks.push(task);
                        index += 1;
                    }
                }
                let after_task =
                    Task::done(Message::OpenItem(Some((
                        LibraryKind::Presentation,
                        self.presentation_library.items.len() as i32
                            - 1,
                    ))));
                return Action::Task(
                    Task::batch(tasks).chain(after_task),
                );
            }
            Message::AddImages(images) => {
                debug!(?images);
                let mut index = self.image_library.items.len();
                // Check if empty
                let mut tasks = Vec::new();
                if let Some(images) = images {
                    let len = images.len();
                    for image in images {
                        if let Err(e) =
                            self.image_library.add_item(image.clone())
                        {
                            error!(?e);
                        }
                        let task = Task::future(self.db.acquire())
                            .and_then(move |db| {
                                Task::perform(
                                    add_image_to_db(
                                        image.to_owned(),
                                        db,
                                    ),
                                    move |res| {
                                        debug!(
                                            len,
                                            index, "added to db"
                                        );
                                        if let Err(e) = res {
                                            error!(?e);
                                        }
                                        if len == index {
                                            debug!("open the pres");
                                            Message::OpenItem(Some((
                                                LibraryKind::Image,
                                                index as i32,
                                            )))
                                        } else {
                                            Message::None
                                        }
                                    },
                                )
                            });
                        tasks.push(task);
                        index += 1;
                    }
                }
                let after_task =
                    Task::done(Message::OpenItem(Some((
                        LibraryKind::Image,
                        self.image_library.items.len() as i32 - 1,
                    ))));
                return Action::Task(
                    Task::batch(tasks).chain(after_task),
                );
            }
            Message::OpenItem(item) => {
                debug!(?item);
                self.editing_item = item;
                return Action::OpenItem(item);
            }
            Message::OpenContextItem => {
                let Some(kind) = self.library_open else {
                    return Action::None;
                };
                let Some(index) = self.context_menu else {
                    return Action::None;
                };
                return self
                    .update(Message::OpenItem(Some((kind, index))));
            }
            Message::HoverLibrary(library_kind) => {
                self.library_hovered = library_kind;
            }
            Message::OpenLibrary(library_kind) => {
                self.selected_items = None;
                self.library_open = library_kind;
            }
            Message::HoverItem(item) => {
                self.hovered_item = item;
            }
            Message::SelectItem(item) => {
                let Some(modifiers) = self.modifiers_pressed else {
                    let Some(item) = item else {
                        return Action::None;
                    };
                    self.selected_items = Some(vec![item]);
                    return Action::None;
                };
                if modifiers.is_empty() {
                    let Some(item) = item else {
                        return Action::None;
                    };
                    self.selected_items = Some(vec![item]);
                    return Action::None;
                }
                if modifiers.shift() {
                    let Some(first_item) = self
                        .selected_items
                        .as_ref()
                        .and_then(|items| {
                            items
                                .iter()
                                .next()
                                .map(|(_, index)| index)
                        })
                    else {
                        let Some(item) = item else {
                            return Action::None;
                        };
                        self.selected_items = Some(vec![item]);
                        return Action::None;
                    };
                    let Some((kind, index)) = item else {
                        return Action::None;
                    };
                    if first_item < &index {
                        for id in *first_item..=index {
                            self.selected_items = self
                                .selected_items
                                .clone()
                                .map(|mut items| {
                                    items.push((kind, id));
                                    items
                                });
                        }
                    } else if first_item > &index {
                        for id in index..*first_item {
                            self.selected_items = self
                                .selected_items
                                .clone()
                                .map(|mut items| {
                                    items.push((kind, id));
                                    items
                                });
                        }
                    }
                }
                if modifiers.control() {
                    let Some(item) = item else {
                        return Action::None;
                    };
                    let Some(items) = self.selected_items.as_mut()
                    else {
                        self.selected_items = Some(vec![item]);
                        return Action::None;
                    };
                    items.push(item);
                    self.selected_items = Some(items.to_vec());
                }
            }
            Message::DragItem(item) => {
                debug!(?item);
                // self.dragged_item = item;
                return Action::DraggedItem(item);
            }
            Message::UpdateSong(song) => {
                let Some((kind, index)) = self.editing_item else {
                    error!("Not editing an item");
                    return Action::None;
                };

                if kind != LibraryKind::Song {
                    error!("Not editing a song item");
                    return Action::None;
                }

                if self
                    .song_library
                    .update_item(song.clone(), index)
                    .is_err()
                {
                    error!("Couldn't update song in model");
                    return Action::None;
                };

                return Action::Task(
                    Task::future(self.db.acquire()).and_then(
                        move |conn| {
                            Task::perform(
                                update_song_in_db(song.clone(), conn),
                                |r| match r {
                                    Ok(_) => Message::SongChanged,
                                    Err(e) => {
                                        error!(?e);
                                        Message::None
                                    }
                                },
                            )
                        },
                    ),
                );
            }
            Message::SongChanged => {
                // self.song_library.update_item(song, index);
                debug!("song changed");
            }
            Message::UpdateImage(image) => {
                let Some((kind, index)) = self.editing_item else {
                    error!("Not editing an item");
                    return Action::None;
                };

                if kind != LibraryKind::Image {
                    error!("Not editing a image item");
                    return Action::None;
                }

                if self
                    .image_library
                    .update_item(image.clone(), index)
                    .is_err()
                {
                    error!("Couldn't update image in model");
                    return Action::None;
                };

                if self
                    .image_library
                    .update_item(image.clone(), index)
                    .is_err()
                {
                    error!("Couldn't update image in model");
                    return Action::None;
                };

                return Action::Task(
                    Task::future(self.db.acquire()).and_then(
                        move |conn| {
                            Task::perform(
                                update_image_in_db(
                                    image.to_owned(),
                                    conn,
                                ),
                                |r| match r {
                                    Ok(_) => Message::ImageChanged,
                                    Err(e) => {
                                        error!(?e);
                                        Message::None
                                    }
                                },
                            )
                        },
                    ),
                );
            }
            Message::ImageChanged => (),
            Message::UpdateVideo(video) => {
                let Some((kind, index)) = self.editing_item else {
                    error!("Not editing an item");
                    return Action::None;
                };

                if kind != LibraryKind::Video {
                    error!("Not editing a video item");
                    return Action::None;
                }

                if self
                    .video_library
                    .update_item(video.clone(), index)
                    .is_err()
                {
                    error!("Couldn't update video in model");
                    return Action::None;
                };

                return Action::Task(
                    Task::future(self.db.acquire()).and_then(
                        move |conn| {
                            Task::perform(
                                update_video_in_db(
                                    video.to_owned(),
                                    conn,
                                ),
                                |r| match r {
                                    Ok(_) => Message::VideoChanged,
                                    Err(e) => {
                                        error!(?e);
                                        Message::None
                                    }
                                },
                            )
                        },
                    ),
                );
            }
            Message::VideoChanged => debug!("vid shoulda changed"),
            Message::UpdatePresentation(presentation) => {
                let Some((kind, index)) = self.editing_item else {
                    error!("Not editing an item");
                    return Action::None;
                };

                if kind != LibraryKind::Presentation {
                    error!("Not editing a presentation item");
                    return Action::None;
                }

                match self
                            .presentation_library
                            .update_item(presentation.clone(), index)
                        {
                            Ok(()) => return Action::Task(
                                Task::future(self.db.acquire()).and_then(
                                    move |conn| {
                                        Task::perform(
                                            update_presentation_in_db(
                                                presentation.clone(),
                                                conn,
                                            ),
                                            |r| match r {
                                                Ok(_) => Message::PresentationChanged,
                                                Err(e) => {
                                                    error!(?e);
                                                    Message::None
                                                }
                                            },
                                        )
                                    },
                                ),
                            ),
                            Err(_) => todo!(),
                        }
            }
            Message::PresentationChanged => (),
            Message::Error(_) => (),
            Message::OpenContext(index) => {
                let Some(kind) = self.library_open else {
                    return Action::None;
                };
                debug!(index, "should context");
                let Some(items) = self.selected_items.as_mut() else {
                    self.selected_items = vec![(kind, index)].into();
                    self.context_menu = Some(index);
                    return Action::None;
                };

                if items.contains(&(kind, index)) {
                    debug!(index, "should context contained");
                    self.selected_items = Some(items.to_vec());
                } else {
                    debug!(index, "should context not contained");
                    self.selected_items = vec![(kind, index)].into();
                }
                self.context_menu = Some(index);
            }
            Message::AddFiles(items) => {
                let mut tasks = Vec::new();
                let last_item = &items.last();
                let after_task = match last_item {
                    Some(ServiceItemKind::Image(image)) => {
                        Task::done(Message::OpenItem(Some((
                            LibraryKind::Image,
                            self.image_library.items.len() as i32 - 1,
                        ))))
                    }
                    _ => Task::none(),
                };
                for item in items {
                    match item {
                        ServiceItemKind::Song(song) => {
                            let Some(e) = self
                                .song_library
                                .add_item(song.clone())
                                .err()
                            else {
                                let task =
                                    Task::future(self.db.acquire())
                                        .and_then(move |db| {
                                            Task::perform(
                                                add_song_to_db(db),
                                                {
                                                    move |res| {
                                                        if let Err(
                                                            e,
                                                        ) = res
                                                        {
                                                            error!(
                                                                ?e
                                                            );
                                                        }
                                                        Message::None
                                                    }
                                                },
                                            )
                                        });
                                tasks.push(task);
                                continue;
                            };
                            error!(?e);
                        }
                        ServiceItemKind::Video(video) => {
                            let Some(e) = self
                                .video_library
                                .add_item(video.clone())
                                .err()
                            else {
                                let task = Task::future(
                                    self.db.acquire(),
                                )
                                .and_then(move |db| {
                                    Task::perform(
                                        add_video_to_db(
                                            video.clone(),
                                            db,
                                        ),
                                        {
                                            let video = video.clone();
                                            move |res| {
                                                debug!(
                                                    ?video,
                                                    "added to db"
                                                );
                                                if let Err(e) = res {
                                                    error!(?e);
                                                }
                                                Message::None
                                            }
                                        },
                                    )
                                });
                                tasks.push(task);
                                continue;
                            };
                            error!(?e);
                        }
                        ServiceItemKind::Image(image) => {
                            let Some(e) = self
                                .image_library
                                .add_item(image.clone())
                                .err()
                            else {
                                let task = Task::future(
                                    self.db.acquire(),
                                )
                                .and_then(move |db| {
                                    Task::perform(
                                        add_image_to_db(
                                            image.clone(),
                                            db,
                                        ),
                                        {
                                            let image = image.clone();
                                            move |res| {
                                                debug!(
                                                    ?image,
                                                    "added to db"
                                                );
                                                if let Err(e) = res {
                                                    error!(?e);
                                                }
                                                Message::None
                                            }
                                        },
                                    )
                                });
                                tasks.push(task);
                                continue;
                            };
                            error!(?e);
                        }
                        ServiceItemKind::Presentation(
                            presentation,
                        ) => {
                            let Some(e) = self
                                .presentation_library
                                .add_item(presentation.clone())
                                .err()
                            else {
                                let task =
                                    Task::future(self.db.acquire())
                                        .and_then(move |db| {
                                            Task::perform(
                                        add_presentation_to_db(
                                            presentation.clone(),
                                            db,
                                        ),
                                        {
                                            let presentation =
                                                presentation.clone();
                                            move |res| {
                                                debug!(
                                                    ?presentation,
                                                    "added to db"
                                                );
                                                if let Err(e) = res {
                                                    error!(?e);
                                                }
                                                Message::None
                                            }
                                        },
                                    )
                                        });
                                tasks.push(task);

                                continue;
                            };
                            error!(?e);
                        }
                        ServiceItemKind::Content(slide) => todo!(),
                    }
                }
                return Action::Task(
                    Task::batch(tasks).chain(after_task),
                );
            }
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let cosmic::cosmic_theme::Spacing { space_s, .. } =
            cosmic::theme::spacing();
        let song_library = self.library_item(&self.song_library);
        let image_library = self.library_item(&self.image_library);
        let video_library = self.library_item(&self.video_library);
        let presentation_library =
            self.library_item(&self.presentation_library);

        let library_column = column![
            text::heading("Library").center().width(Length::Fill),
            cosmic::iced::widget::horizontal_rule(1),
            song_library,
            image_library,
            video_library,
            presentation_library,
        ]
        .height(Length::Fill)
        .padding(10)
        .spacing(space_s);
        let library_dnd = dnd_destination(
            library_column,
            vec![
                "image/png".into(),
                "image/jpg".into(),
                "image/heif".into(),
                "image/gif".into(),
                "video/mp4".into(),
                "video/AV1".into(),
                "video/H264".into(),
                "video/H265".into(),
                "video/mpeg".into(),
                "video/mkv".into(),
                "video/webm".into(),
                "video/ogg".into(),
                "video/vnd.youtube.yt".into(),
                "video/x-matroska".into(),
                "application/pdf".into(),
                "text/html".into(),
                "text/md".into(),
                "text/org".into(),
                "text/uri-list".into(),
            ],
        )
        .on_enter(|_, _, mimes| {
            warn!(?mimes);
            Message::None
        })
        .on_finish(|mime, data, action, _, _| {
            // warn!(?mime, ?data, ?action);
            match mime.as_str() {
                "text/uri-list" => {
                    let Ok(text) = str::from_utf8(&data) else {
                        return Message::None;
                    };
                    let mut items = Vec::new();
                    for line in text.lines() {
                        let Ok(url) = url::Url::parse(line) else {
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
                        let item = ServiceItemKind::try_from(path);
                        match item {
                            Ok(item) => items.push(item),
                            Err(e) => error!(?e),
                        }
                    }
                    Message::AddFiles(items)
                }
                _ => Message::None,
            }
        });
        container(library_dnd).padding(2).into()
    }

    pub fn library_item<T>(
        &'a self,
        model: &'a Model<T>,
    ) -> Element<'a, Message>
    where
        T: Content,
    {
        let mut row = row::<Message>().spacing(5);
        match &model.kind {
            LibraryKind::Song => {
                row = row
                    .push(icon::from_name("folder-music-symbolic"));
                row = row
                    .push(textm!("Songs").align_y(Vertical::Center));
            }
            LibraryKind::Video => {
                row = row
                    .push(icon::from_name("folder-videos-symbolic"));
                row = row
                    .push(textm!("Videos").align_y(Vertical::Center));
            }
            LibraryKind::Image => {
                row = row.push(icon::from_name(
                    "folder-pictures-symbolic",
                ));
                row = row
                    .push(textm!("Images").align_y(Vertical::Center));
            }
            LibraryKind::Presentation => {
                row = row.push(icon::from_name(
                    "x-office-presentation-symbolic",
                ));
                row = row.push(
                    textm!("Presentations").align_y(Vertical::Center),
                );
            }
        }
        let item_count = model.items.len();
        row = row.push(horizontal_space());
        row = row
            .push(textm!("{}", item_count).align_y(Vertical::Center));
        row = row.push(
            icon::from_name({
                if self.library_open == Some(model.kind) {
                    "arrow-up"
                } else {
                    "arrow-down"
                }
            })
            .size(20),
        );
        let row_container =
            Container::new(row.align_y(Vertical::Center))
                .padding(5)
                .style(|t| {
                    container::Style::default()
                        .background({
                            match self.library_hovered {
                                Some(lib) => Background::Color(
                                    if lib == model.kind {
                                        t.cosmic().button.hover.into()
                                    } else {
                                        t.cosmic().button.base.into()
                                    },
                                ),
                                None => Background::Color(
                                    t.cosmic().button.base.into(),
                                ),
                            }
                        })
                        .border(Border::default().rounded(
                            t.cosmic().corner_radii.radius_s,
                        ))
                })
                .center_x(Length::Fill)
                .center_y(Length::Shrink);
        let library_button = mouse_area(row_container)
            .on_press({
                if self.library_open == Some(model.kind) {
                    Message::OpenLibrary(None)
                } else {
                    Message::OpenLibrary(Some(model.kind))
                }
            })
            .on_enter(Message::HoverLibrary(Some(model.kind)))
            .on_exit(Message::HoverLibrary(None));
        let lib_container = if self.library_open == Some(model.kind) {
            let items = scrollable(
                column({
                    model.items.iter().enumerate().map(
                        |(index, item)| {
                            let kind = model.kind;
                            let visual_item = self
                                .single_item(index, item, model)
                                .map(|()| Message::None);

                            DndSource::<Message, KindWrapper>::new({
                                let mouse_area = mouse_area(visual_item);
                                let mouse_area = mouse_area.on_enter(Message::HoverItem(
                                    Some((
                                        model.kind,
                                        index as i32,
                                    )),
                                ))
                                    .on_double_click(
                                        Message::OpenItem(Some((
                                            model.kind,
                                            index as i32,
                                        ))),
                                    )
                                    .on_right_press(Message::OpenContext(index as i32))
                                    .on_exit(Message::HoverItem(None))
                                    .on_press(Message::SelectItem(
                                        Some((
                                            model.kind,
                                            index as i32,
                                        )),
                                    ));

                                Element::from(mouse_area)
                            })
                                .action(DndAction::Copy)
                                .drag_icon({
                                    let model = model.kind;
                                    move |i| {
                                        let state = State::None;
                                        let icon = match model {
                                            LibraryKind::Song => icon::from_name(
                                                "folder-music-symbolic",
                                                    ).symbolic(true)
                                                        ,
                                                    LibraryKind::Video => icon::from_name("folder-videos-symbolic"),
                                                    LibraryKind::Image => icon::from_name("folder-pictures-symbolic"),
                                                    LibraryKind::Presentation => icon::from_name("x-office-presentation-symbolic"),
                                                };
                                            (
                                                icon.into(),
                                                state,
                                                i,
                                            )
                            }})
                            .drag_content(move || {
                                KindWrapper((kind, index as i32))
                            })
                            .into()
                        },
                    )
                })
                    .spacing(2)
                    .width(Length::Fill),
            )
                .spacing(5)
                .height(Length::Fill);

            let library_toolbar = rowm!(
                text_input("Search...", ""),
                button::icon(icon::from_name("add"))
                    .on_press(Message::AddItem)
            );
            let context_menu = self.context_menu(items.into());
            let library_column =
                column![library_toolbar, context_menu].spacing(3);
            Container::new(library_column).padding(5)
        } else {
            Container::new(Space::new(0, 0))
        };
        column![library_button, lib_container].into()
    }

    fn single_item<T>(
        &'a self,
        index: usize,
        item: &'a T,
        model: &'a Model<T>,
    ) -> Element<'a, ()>
    where
        T: Content,
    {
        let text = Container::new(responsive(|size| {
            text::heading(elide_text(item.title(), size.width))
                .center()
                .wrapping(textm::Wrapping::None)
                .into()
        }))
        .center_y(20)
        .center_x(Length::Fill);
        let subtext = container(responsive(move |size| {
            let color: Color = if item.background().is_some() {
                if let Some(items) = &self.selected_items {
                    if items.contains(&(model.kind, index as i32)) {
                        theme::active().cosmic().control_0().into()
                    } else {
                        theme::active()
                            .cosmic()
                            .accent_text_color()
                            .into()
                    }
                } else {
                    theme::active()
                        .cosmic()
                        .accent_text_color()
                        .into()
                }
            } else if let Some(items) = &self.selected_items {
                if items.contains(&(model.kind, index as i32)) {
                    theme::active().cosmic().control_0().into()
                } else {
                    theme::active()
                        .cosmic()
                        .destructive_text_color()
                        .into()
                }
            } else {
                theme::active()
                    .cosmic()
                    .destructive_text_color()
                    .into()
            };
            text::body(elide_text(item.subtext(), size.width))
                .center()
                .wrapping(textm::Wrapping::None)
                .class(color)
                .into()
        }))
        .center_y(20)
        .center_x(Length::Fill);

        let texts = column([text.into(), subtext.into()]);

        Container::new(
            rowm![horizontal_space().width(0), texts]
                .spacing(10)
                .align_y(Vertical::Center),
        )
        // .padding(5)
        .width(Length::Fill)
        .style(move |t| {
            container::Style::default()
                .background(Background::Color(
                    if let Some(items) = &self.selected_items {
                        if items.contains(&(model.kind, index as i32))
                        {
                            t.cosmic().accent.selected.into()
                        } else if let Some((library, hovered)) =
                            self.hovered_item
                        {
                            if model.kind == library
                                && hovered == index as i32
                            {
                                t.cosmic().button.hover.into()
                            } else {
                                t.cosmic().button.base.into()
                            }
                        } else {
                            t.cosmic().button.base.into()
                        }
                    } else if let Some((library, hovered)) =
                        self.hovered_item
                    {
                        if model.kind == library
                            && hovered == index as i32
                        {
                            t.cosmic().button.hover.into()
                        } else {
                            t.cosmic().button.base.into()
                        }
                    } else {
                        t.cosmic().button.base.into()
                    },
                ))
                .border(
                    Border::default()
                        .rounded(t.cosmic().corner_radii.radius_m),
                )
        })
        .padding([3, 0])
        .into()
    }

    fn context_menu<'b>(
        &self,
        items: Element<'b, Message>,
    ) -> Element<'b, Message> {
        if self.context_menu.is_some() {
            let menu_items = vec![
                menu::Item::Button("Open", None, MenuMessage::Open),
                menu::Item::Button(
                    "Delete",
                    None,
                    MenuMessage::Delete,
                ),
            ];
            let context_menu = context_menu(
                items,
                self.context_menu.map_or_else(
                    || None,
                    |_| {
                        Some(menu::items(&self.menu_keys, menu_items))
                    },
                ),
            );
            Element::from(context_menu)
        } else {
            items
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn search_items(
        &self,
        query: String,
    ) -> Vec<ServiceItemKind> {
        let query = query.to_lowercase();
        let mut items: Vec<ServiceItemKind> = self
            .song_library
            .items
            .clone()
            .into_iter()
            .filter(|song| song.title.to_lowercase().contains(&query))
            .map(ServiceItemKind::Song)
            .collect();
        let videos: Vec<ServiceItemKind> = self
            .video_library
            .items
            .clone()
            .into_iter()
            .filter(|vid| vid.title.to_lowercase().contains(&query))
            .map(ServiceItemKind::Video)
            .collect();
        let images: Vec<ServiceItemKind> = self
            .image_library
            .items
            .clone()
            .into_iter()
            .filter(|image| {
                image.title.to_lowercase().contains(&query)
            })
            .map(ServiceItemKind::Image)
            .collect();
        let presentations: Vec<ServiceItemKind> = self
            .presentation_library
            .items
            .clone()
            .into_iter()
            .filter(|pres| pres.title.to_lowercase().contains(&query))
            .map(ServiceItemKind::Presentation)
            .collect();
        items.extend(videos);
        items.extend(images);
        items.extend(presentations);
        let mut items: Vec<(usize, ServiceItemKind)> = items
            .into_iter()
            .map(|item| {
                (
                    levenshtein::distance(
                        query.bytes(),
                        item.title().bytes(),
                    ),
                    item,
                )
            })
            .collect();

        items.sort_by(|a, b| a.0.cmp(&b.0));
        items.into_iter().map(|item| item.1).collect()
    }

    pub fn get_video(&self, index: i32) -> Option<&Video> {
        self.video_library.get_item(index)
    }

    pub fn get_image(&self, index: i32) -> Option<&Image> {
        self.image_library.get_item(index)
    }

    pub fn get_presentation(
        &self,
        index: i32,
    ) -> Option<&Presentation> {
        self.presentation_library.get_item(index)
    }

    pub fn set_modifiers(&mut self, modifiers: Option<Modifiers>) {
        self.modifiers_pressed = modifiers;
    }

    fn delete_items(&mut self) -> Action {
        // Need to make this function collect tasks to be run off of
        // who should be deleted
        let Some(items) = self.selected_items.as_mut() else {
            return Action::None;
        };
        items.sort_by(|(_, index), (_, other)| index.cmp(other));
        let tasks: Vec<Task<Message>> = items
            .iter()
            .rev()
            .map(|(kind, index)| match kind {
                LibraryKind::Song => {
                    if let Some(song) =
                        self.song_library.get_item(*index)
                    {
                        let song = song.clone();
                        if let Err(e) =
                            self.song_library.remove_item(*index)
                        {
                            error!(?e);
                            Task::none()
                        } else {
                            Task::future(self.db.acquire()).and_then(
                                move |db| {
                                    Task::perform(
                                        songs::remove_from_db(
                                            db, song.id,
                                        ),
                                        |r| {
                                            if let Err(e) = r {
                                                error!(?e)
                                            }
                                            Message::None
                                        },
                                    )
                                },
                            )
                        }
                    } else {
                        Task::none()
                    }
                }
                LibraryKind::Video => {
                    if let Some(video) =
                        self.video_library.get_item(*index)
                    {
                        let video = video.clone();
                        if let Err(e) =
                            self.video_library.remove_item(*index)
                        {
                            error!(?e);
                            Task::none()
                        } else {
                            Task::future(self.db.acquire()).and_then(
                                move |db| {
                                    Task::perform(
                                        videos::remove_from_db(
                                            db, video.id,
                                        ),
                                        |r| {
                                            if let Err(e) = r {
                                                error!(?e)
                                            }
                                            Message::None
                                        },
                                    )
                                },
                            )
                        }
                    } else {
                        Task::none()
                    }
                }
                LibraryKind::Image => {
                    if let Some(image) =
                        self.image_library.get_item(*index)
                    {
                        let image = image.clone();
                        if let Err(e) =
                            self.image_library.remove_item(*index)
                        {
                            error!(?e);
                            Task::none()
                        } else {
                            debug!("let's remove {0}", image.id);
                            debug!("let's remove {0}", image.title);
                            Task::future(self.db.acquire()).and_then(
                                move |db| {
                                    Task::perform(
                                        images::remove_from_db(
                                            db, image.id,
                                        ),
                                        |r| {
                                            if let Err(e) = r {
                                                error!(?e)
                                            }
                                            Message::None
                                        },
                                    )
                                },
                            )
                        }
                    } else {
                        Task::none()
                    }
                }
                LibraryKind::Presentation => {
                    if let Some(presentation) =
                        self.presentation_library.get_item(*index)
                    {
                        let presentation = presentation.clone();
                        if let Err(e) = self
                            .presentation_library
                            .remove_item(*index)
                        {
                            error!(?e);
                            Task::none()
                        } else {
                            Task::future(self.db.acquire()).and_then(
                                move |db| {
                                    Task::perform(
                                        presentations::remove_from_db(
                                            db,
                                            presentation.id,
                                        ),
                                        |r| {
                                            if let Err(e) = r {
                                                error!(?e)
                                            }
                                            Message::None
                                        },
                                    )
                                },
                            )
                        }
                    } else {
                        Task::none()
                    }
                }
            })
            .collect();
        if !tasks.is_empty() {
            self.selected_items = None;
        }
        Action::Task(Task::batch(tasks))
    }
}

async fn add_images() -> Option<Vec<Image>> {
    let paths =
        Dialog::new().title("pick image").open_files().await.ok()?;
    Some(
        paths
            .urls()
            .iter()
            .map(|path| {
                Image::from(path.to_file_path().expect("oops"))
            })
            .collect(),
    )
}

async fn add_videos() -> Option<Vec<Video>> {
    let paths =
        Dialog::new().title("pick video").open_files().await.ok()?;
    Some(
        paths
            .urls()
            .iter()
            .map(|path| {
                Video::from(path.to_file_path().expect("oops"))
            })
            .collect(),
    )
}

async fn add_presentations() -> Option<Vec<Presentation>> {
    let paths = Dialog::new()
        .title("pick presentation")
        .open_files()
        .await
        .ok()?;
    Some(
        paths
            .urls()
            .iter()
            .map(|path| {
                Presentation::from(path.to_file_path().expect("oops"))
            })
            .collect(),
    )
}

async fn add_db() -> Result<SqlitePool> {
    let mut data = dirs::data_local_dir().unwrap();
    data.push("lumina");
    data.push("library-db.sqlite3");
    let mut db_url = String::from("sqlite://");
    db_url.push_str(data.to_str().unwrap());
    SqlitePool::connect(&db_url).await.into_diagnostic()
}

pub fn elide_text(text: impl AsRef<str>, width: f32) -> String {
    const CHAR_SIZE: f32 = 8.0;
    let text: String = text.as_ref().to_owned();
    let text_length = text.len() as f32 * CHAR_SIZE;
    if text_length > width {
        format!(
            "{}...",
            text.split_at(
                ((width / CHAR_SIZE) - 3.0).floor() as usize
            )
            .0
        )
    } else {
        text
    }
}
