use cosmic::{Element, Task};

use crate::core::{
    images::Image, model::Model, presentations::Presentation,
    songs::Song, videos::Video,
};

struct Library {
    song_library: Model<Song>,
    image_library: Model<Image>,
    video_library: Model<Video>,
    presentation_library: Model<Presentation>,
}

enum Message {
    AddItem,
    RemoveItem,
    OpenItem,
    None,
}

impl Library {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddItem => todo!(),
            Message::None => todo!(),
            Message::RemoveItem => todo!(),
            Message::OpenItem => todo!(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        todo!()
    }
}
