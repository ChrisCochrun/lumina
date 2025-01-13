use cosmic::{
    iced::Length,
    iced_widget::{column, text},
    widget::{button, horizontal_space, icon, row},
    Element, Task,
};

use crate::core::{
    images::Image, kinds::ServiceItemKind, model::Model,
    presentations::Presentation, service_items::ServiceItemModel,
    songs::Song, videos::Video,
};

pub(crate) struct Library {
    song_library: Model<Song>,
    image_library: Model<Image>,
    video_library: Model<Video>,
    presentation_library: Model<Presentation>,
    library_open: Option<ServiceItemKind>,
}

#[derive(Clone)]
pub(crate) enum Message {
    AddItem,
    RemoveItem,
    OpenItem,
    None,
}

impl Library {
    pub fn new(service_items: &ServiceItemModel) -> Library {
        todo!()
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddItem => todo!(),
            Message::None => todo!(),
            Message::RemoveItem => todo!(),
            Message::OpenItem => todo!(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let column = column![
            self.library_item(&self.song_library),
            self.library_item(&self.image_library),
            self.library_item(&self.video_library),
            self.library_item(&self.presentation_library),
        ];
        todo!()
    }

    pub fn library_item<T>(
        &self,
        model: &Model<T>,
    ) -> Element<Message> {
        let mut row = row::<Message>();
        let title = match &model.kind {
            ServiceItemKind::Song(song) => {
                row = row.push(text!("Songs"));
            }
            ServiceItemKind::Video(video) => {
                row = row.push(text!("Videos"));
            }
            ServiceItemKind::Image(image) => {
                row = row.push(text!("Images"));
            }
            ServiceItemKind::Presentation(presentation) => {
                row = row.push(text!("Presentations"));
            }
            ServiceItemKind::Content(slide) => todo!(),
        };
        let item_count = model.items.len();
        row = row.push(text!("{}", item_count));
        row = row.push(horizontal_space().width(Length::Fill));
        row = row.push(
            button::icon(icon::from_name("arrow-down"))
                .on_press(Message::None),
        );
        row.into()
    }
}
