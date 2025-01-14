use cosmic::{
    iced::{
        alignment::{Horizontal, Vertical},
        Background, Border, Color, Length,
    },
    iced_widget::{column, text},
    widget::{
        button, container, horizontal_space, icon, mouse_area, row,
        Container, Space,
    },
    Element, Task,
};

use crate::core::{
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
}

#[derive(Clone, Debug)]
pub(crate) enum Message {
    AddItem,
    RemoveItem,
    OpenItem,
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
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddItem => Task::none(),
            Message::None => Task::none(),
            Message::RemoveItem => Task::none(),
            Message::OpenItem => Task::none(),
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

    pub fn library_item<T>(
        &self,
        model: &Model<T>,
    ) -> Element<Message> {
        let mut row = row::<Message>().spacing(5);
        match &model.kind {
            LibraryKind::Song => {
                row = row
                    .push(text!("Songs").align_y(Vertical::Center));
            }
            LibraryKind::Video => {
                row = row
                    .push(text!("Videos").align_y(Vertical::Center));
            }
            LibraryKind::Image => {
                row = row
                    .push(text!("Images").align_y(Vertical::Center));
            }
            LibraryKind::Presentation => {
                row = row.push(
                    text!("Presentations").align_y(Vertical::Center),
                );
            }
        };
        let item_count = model.items.len();
        row = row.push(horizontal_space());
        row = row.push(
            text!("{}", item_count)
                .align_y(Vertical::Center)
                .size(18),
        );
        row = row.push(icon::from_name("arrow-down").size(20));
        let row_container =
            Container::new(row)
                .padding(5)
                .style(|t| {
                    container::Style::default()
                        .background(Background::Color(
                            t.cosmic().secondary.base.into(),
                        ))
                        .border(Border::default().rounded(
                            t.cosmic().corner_radii.radius_s,
                        ))
                })
                .center_x(Length::Fill)
                .center_y(Length::Shrink);
        let button = mouse_area(row_container);
        button.into()
    }
}
