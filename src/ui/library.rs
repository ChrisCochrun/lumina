use cosmic::{
    font::default,
    iced::{
        alignment::{Horizontal, Vertical},
        Background, Border, Color, Length,
    },
    iced_widget::{column, row as rowm, text as textm},
    theme,
    widget::{
        button, container, horizontal_space, icon, mouse_area, row,
        scrollable, text, Column, Container, Space,
    },
    Element, Task,
};

use crate::core::{
    content::Content,
    images::Image,
    model::{get_db, LibraryKind, Model},
    presentations::Presentation,
    songs::Song,
    videos::Video,
};

#[derive(Debug, Clone)]
pub(crate) struct Library {
    song_library: Model<Song>,
    image_library: Model<Image>,
    video_library: Model<Video>,
    presentation_library: Model<Presentation>,
    library_open: Option<LibraryKind>,
    library_hovered: Option<LibraryKind>,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    AddItem,
    RemoveItem,
    OpenItem,
    HoverLibrary(Option<LibraryKind>),
    OpenLibrary(Option<LibraryKind>),
    None,
}

impl Library {
    pub async fn new() -> Self {
        let mut db = get_db().await;
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
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddItem => Task::none(),
            Message::None => Task::none(),
            Message::RemoveItem => Task::none(),
            Message::OpenItem => Task::none(),
            Message::HoverLibrary(library_kind) => {
                self.library_hovered = library_kind;
                Task::none()
            }
            Message::OpenLibrary(library_kind) => {
                self.library_open = library_kind;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let column = column![
            self.library_item(&self.song_library),
            self.library_item(&self.image_library),
            self.library_item(&self.video_library),
            self.library_item(&self.presentation_library),
        ];
        column.height(Length::Fill).spacing(5).into()
    }

    pub fn library_item<'a, T>(
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
                    .push(textm!("Songs").align_y(Vertical::Center));
            }
            LibraryKind::Video => {
                row = row
                    .push(textm!("Videos").align_y(Vertical::Center));
            }
            LibraryKind::Image => {
                row = row
                    .push(textm!("Images").align_y(Vertical::Center));
            }
            LibraryKind::Presentation => {
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
            Container::new(row)
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
        let button = mouse_area(row_container)
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
                    model.items.iter().map(|item| {
                        let text = text::heading(elide_text(
                            item.title(),
                            18,
                        ))
                        .center()
                        .wrapping(textm::Wrapping::None);
                        let icon = icon::from_name({
                            match model.kind {
                                LibraryKind::Song => {
                                    "folder-music-symbolic"
                                }
                                LibraryKind::Video => {
                                    "folder-videos-symbolic"
                                }
                                LibraryKind::Image => {
                                    "folder-pictures-symbolic"
                                }
                                LibraryKind::Presentation => {
                                    "x-office-presentation-symbolic"
                                }
                            }
                        });
                        Container::new(rowm![icon, text].spacing(10))
                            .padding(5)
                            .width(Length::Fill)
                            .style(|t| {
                                container::Style::default()
                                    .background(Background::Color(
                                        t.cosmic().button.base.into(),
                                    ))
                                    .border(
                                        Border::default().rounded(
                                            t.cosmic()
                                                .corner_radii
                                                .radius_l,
                                        ),
                                    )
                            })
                            .into()
                    })
                })
                .spacing(2)
                .width(Length::Fill),
            );
            Container::new(items).padding(5).style(|t| {
                container::Style::default()
                    .background(Background::Color(
                        t.cosmic().primary.base.into(),
                    ))
                    .border(
                        Border::default().rounded(
                            t.cosmic().corner_radii.radius_m,
                        ),
                    )
            })
        } else {
            Container::new(Space::new(0, 0))
        };
        column![button, lib_container].into()
    }
}

fn elide_text(text: String, amount: usize) -> String {
    if text.len() > amount {
        format!("{}...", text.split_at(amount).0)
    } else {
        text
    }
}
