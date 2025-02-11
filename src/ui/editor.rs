use cosmic::{Element, Task};

use crate::core::songs::Song;

#[derive(Debug, Clone)]
pub struct SongEditor {
    song: Song,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeSong(Song),
    UpdateSong(Song),
}

impl SongEditor {
    pub fn update(&self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeSong(song) => todo!(),
            Message::UpdateSong(song) => todo!(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        todo!()
    }
}
