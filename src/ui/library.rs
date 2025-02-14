use cosmic::{
    iced::{alignment::Vertical, Background, Border, Length},
    iced_widget::{column, row as rowm, text as textm},
    widget::{
        container, horizontal_space, icon, mouse_area, responsive,
        row, scrollable, text, Container, Space,
    },
    Element, Task,
};
use tracing::debug;

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
    selected_item: Option<(LibraryKind, i32)>,
    hovered_item: Option<(LibraryKind, i32)>,
    dragged_item: Option<(LibraryKind, i32)>,
    dragged_item_pos: Option<(usize, usize)>,
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    AddItem,
    RemoveItem,
    OpenItem,
    HoverLibrary(Option<LibraryKind>),
    OpenLibrary(Option<LibraryKind>),
    HoverItem(Option<(LibraryKind, i32)>),
    SelectItem(Option<(LibraryKind, i32)>),
    DragItem(Option<(LibraryKind, i32)>),
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
            selected_item: None,
            hovered_item: None,
            dragged_item: None,
            dragged_item_pos: None,
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
            Message::HoverItem(item) => {
                self.hovered_item = item;
                Task::none()
            }
            Message::SelectItem(item) => {
                self.selected_item = item;
                Task::none()
            }
            Message::DragItem(item) => {
                self.dragged_item = item;
                debug!(?self.dragged_item);
                Task::none()
            }
        }
    }

    pub fn view(
        &self,
    ) -> (Element<Message>, Option<Element<Message>>) {
        let (song_library, song_dragged) =
            self.library_item(&self.song_library);
        let (image_library, image_dragged) =
            self.library_item(&self.image_library);
        let (video_library, video_dragged) =
            self.library_item(&self.video_library);
        let (presentation_library, presentation_dragged) =
            self.library_item(&self.presentation_library);
        let column = column![
            song_library,
            image_library,
            video_library,
            presentation_library,
        ];
        let dragged_vector = vec![
            song_dragged,
            image_dragged,
            video_dragged,
            presentation_dragged,
        ];
        let dragged = dragged_vector
            .into_iter()
            .filter(|x| x.is_some())
            .next()
            .unwrap_or(None);

        (column.height(Length::Fill).spacing(5).into(), dragged)
    }

    pub fn library_item<'a, T>(
        &'a self,
        model: &'a Model<T>,
    ) -> (Element<'a, Message>, Option<Element<'a, Message>>)
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
        let mut dragged_item = None;
        let lib_container = if self.library_open == Some(model.kind) {
            let items = scrollable(
                column({
                    model.items.iter().enumerate().map(
                        |(index, item)| {
                            mouse_area(
                                self.single_item(index, item, model),
                            )
                            .on_drag({
                                dragged_item =
                                    Some(self.single_item(
                                        index, item, model,
                                    ));
                                Message::DragItem(Some((
                                    model.kind,
                                    index as i32,
                                )))
                            })
                            .on_enter(Message::HoverItem(Some((
                                model.kind,
                                index as i32,
                            ))))
                            .on_exit(Message::HoverItem(None))
                            .on_press(Message::SelectItem(Some((
                                model.kind,
                                index as i32,
                            ))))
                            .into()
                        },
                    )
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
        (column![button, lib_container].into(), dragged_item)
    }

    fn single_item<'a, T>(
        &'a self,
        index: usize,
        item: &'a T,
        model: &'a Model<T>,
    ) -> Element<'a, Message>
    where
        T: Content,
    {
        let text = Container::new(responsive(|size| {
            text::heading(elide_text(item.title(), size.width))
                .center()
                .wrapping(textm::Wrapping::None)
                .into()
        }))
        .center_y(25)
        .center_x(Length::Fill);
        let icon = icon::from_name({
            match model.kind {
                LibraryKind::Song => "folder-music-symbolic",
                LibraryKind::Video => "folder-videos-symbolic",
                LibraryKind::Image => "folder-pictures-symbolic",
                LibraryKind::Presentation => {
                    "x-office-presentation-symbolic"
                }
            }
        });
        Container::new(
            rowm![horizontal_space().width(0), icon, text]
                .spacing(10)
                .align_y(Vertical::Center),
        )
        .padding(5)
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
                        .rounded(t.cosmic().corner_radii.radius_l),
                )
        })
        .into()
    }
}

fn elide_text(text: String, width: f32) -> String {
    const CHAR_SIZE: f32 = 8.0;
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
