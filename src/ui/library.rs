use std::rc::Rc;

use cosmic::{
    iced::{
        alignment::Vertical, clipboard::dnd::DndAction,
        futures::FutureExt, Background, Border, Color, Length,
    },
    iced_core::widget::tree::State,
    iced_widget::{column, row as rowm, text as textm},
    theme,
    widget::{
        button, container, horizontal_space, icon, mouse_area,
        responsive, row, scrollable, text, text_input, Container,
        DndSource, Space, Widget,
    },
    Element, Task,
};
use miette::{IntoDiagnostic, Result};
use sqlx::{pool::PoolConnection, Sqlite, SqlitePool};
use tracing::{debug, error, warn};

use crate::core::{
    content::Content,
    images::{update_image_in_db, Image},
    model::{LibraryKind, Model},
    presentations::{update_presentation_in_db, Presentation},
    service_items::ServiceItem,
    songs::{update_song_in_db, Song},
    videos::{update_video_in_db, Video},
};

#[derive(Debug, Clone)]
pub(crate) struct Library {
    song_library: Model<Song>,
    image_library: Model<Image>,
    video_library: Model<Video>,
    presentation_library: Model<Presentation>,
    library_open: Option<LibraryKind>,
    library_hovered: Option<LibraryKind>,
    selected_item: Option<(LibraryKind, i32)>,
    hovered_item: Option<(LibraryKind, i32)>,
    pub dragged_item: Option<(LibraryKind, i32)>,
    editing_item: Option<(LibraryKind, i32)>,
    db: SqlitePool,
}

pub(crate) enum Action {
    OpenItem(Option<(LibraryKind, i32)>),
    DraggedItem(ServiceItem),
    Task(Task<Message>),
    None,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    AddItem,
    RemoveItem,
    OpenItem(Option<(LibraryKind, i32)>),
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
    None,
}

impl<'a> Library {
    pub async fn new() -> Self {
        let mut db = add_db().await.expect("probs");
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
            selected_item: None,
            hovered_item: None,
            dragged_item: None,
            editing_item: None,
            db,
        }
    }

    pub fn get_song(&self, index: i32) -> Option<&Song> {
        self.song_library.get_item(index)
    }

    pub fn update(&'a mut self, message: Message) -> Action {
        match message {
            Message::AddItem => (),
            Message::None => (),
            Message::RemoveItem => (),
            Message::OpenItem(item) => {
                debug!(?item);
                self.editing_item = item;
                return Action::OpenItem(item);
            }
            Message::HoverLibrary(library_kind) => {
                self.library_hovered = library_kind;
            }
            Message::OpenLibrary(library_kind) => {
                self.library_open = library_kind;
            }
            Message::HoverItem(item) => {
                self.hovered_item = item;
            }
            Message::SelectItem(item) => {
                self.selected_item = item;
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
                };

                match self
                    .song_library
                    .update_item(song.clone(), index)
                {
                    Ok(_) => {
                        return Action::Task(
                            Task::future(self.db.acquire()).and_then(
                                move |conn| update_in_db(&song, conn),
                            ),
                        )
                    }
                    Err(_) => todo!(),
                }
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
                };

                match self
                    .image_library
                    .update_item(image.clone(), index)
                {
                    Ok(_) => {
                        return Action::Task(
                            Task::future(self.db.acquire()).and_then(
                                move |conn| {
                                    Task::perform(
                                        update_image_in_db(
                                            image.clone(),
                                            conn,
                                        ),
                                        |_| Message::ImageChanged,
                                    )
                                },
                            ),
                        )
                    }
                    Err(_) => todo!(),
                }
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
                };

                match self
                    .video_library
                    .update_item(video.clone(), index)
                {
                    Ok(_) => {
                        return Action::Task(
                            Task::future(self.db.acquire()).and_then(
                                move |conn| {
                                    Task::perform(
                                        update_video_in_db(
                                            video.clone(),
                                            conn,
                                        ),
                                        |_| Message::VideoChanged,
                                    )
                                },
                            ),
                        )
                    }
                    Err(_) => todo!(),
                }
            }
            Message::VideoChanged => (),
            Message::UpdatePresentation(presentation) => {
                let Some((kind, index)) = self.editing_item else {
                    error!("Not editing an item");
                    return Action::None;
                };

                if kind != LibraryKind::Presentation {
                    error!("Not editing a presentation item");
                    return Action::None;
                };

                match self
                    .presentation_library
                    .update_item(presentation.clone(), index)
                {
                    Ok(_) => return Action::Task(
                        Task::future(self.db.acquire()).and_then(
                            move |conn| {
                                Task::perform(
                                    update_presentation_in_db(
                                        presentation.clone(),
                                        conn,
                                    ),
                                    |_| Message::PresentationChanged,
                                )
                            },
                        ),
                    ),
                    Err(_) => todo!(),
                }
            }
            Message::PresentationChanged => (),
            Message::Error(_) => (),
        };
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let song_library = self.library_item(&self.song_library);
        let image_library = self.library_item(&self.image_library);
        let video_library = self.library_item(&self.video_library);
        let presentation_library =
            self.library_item(&self.presentation_library);
        let column = column![
            song_library,
            image_library,
            video_library,
            presentation_library,
        ];
        column.height(Length::Fill).spacing(5).into()
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
        };
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
                            let service_item = item.to_service_item();
                            let drag_item =
                                Box::new(self.single_item(index, item, &model));
                            let visual_item = self
                                .single_item(index, item, model)
                                .map(|_| Message::None);
                            DndSource::<Message, ServiceItem>::new(
                                mouse_area(visual_item)
                                    .on_drag(Message::DragItem(service_item.clone()))
                                    .on_enter(Message::HoverItem(
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
                                    .on_exit(Message::HoverItem(None))
                                    .on_press(Message::SelectItem(
                                        Some((
                                            model.kind,
                                            index as i32,
                                        )),
                                    )),
                            )
                            .action(DndAction::Copy)
                                .drag_icon({
                                    let model = model.kind.clone();
                                    move |i| {
                                        let state = State::None;
                                        let icon = match model {
                                                    LibraryKind::Song => icon::from_name(
                                                        "folder-music-symbolic",
                                                    )
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
                                service_item.to_owned()
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
            );
            let library_column =
                column![library_toolbar, items].spacing(3);

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
        let subtext = container(responsive(|size| {
            let color: Color = if item.background().is_some() {
                theme::active().cosmic().accent_text_color().into()
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
                    if let Some((library, selected)) =
                        self.selected_item
                    {
                        if model.kind == library
                            && selected == index as i32
                        {
                            t.cosmic().accent.selected.into()
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
        .into()
    }

    // fn update_item<C: Content>(self, item: C) -> Task<Message> {
    //     let Some((kind, index)) = self.editing_item else {
    //         error!("Not editing an item");
    //         return Task::none();
    //     };

    //     match kind {
    //         LibraryKind::Song => todo!(),
    //         LibraryKind::Video => todo!(),
    //         LibraryKind::Image => {
    //             match self
    //                 .image_library
    //                 .update_item(item as Image, index)
    //             {
    //                 Ok(_) => Task::future(self.db.acquire())
    //                     .and_then(|conn| {
    //                         Task::perform(
    //                             update_image_in_db(item, conn),
    //                             |_| Message::ImageChanged,
    //                         )
    //                     }),
    //                 Err(_) => todo!(),
    //             }
    //         }
    //         LibraryKind::Presentation => todo!(),
    //     }
    // }
}

fn update_in_db(
    song: &Song,
    conn: PoolConnection<Sqlite>,
) -> Task<Message> {
    let song_title = song.title.clone();
    warn!("Should have updated song: {:?}", song_title);
    Task::perform(
        update_song_in_db(song.to_owned(), conn).map(
            move |r| match r {
                Ok(_) => {
                    warn!(
                        "Should have updated song: {:?}",
                        song_title
                    );
                }
                Err(e) => {
                    error!(?e);
                }
            },
        ),
        |_| Message::SongChanged,
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

fn elide_text(text: impl AsRef<str>, width: f32) -> String {
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
