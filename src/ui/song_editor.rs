use cosmic::{
    iced::Font,
    iced_widget::row,
    widget::{dropdown, Container},
    Element, Task,
};

use crate::core::songs::Song;

#[derive(Debug, Clone)]
pub struct SongEditor {
    song: Option<Song>,
    fonts: Vec<Font>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeSong(Song),
    UpdateSong(Song),
    ChangeFont(usize),
}

impl SongEditor {
    pub fn new() -> Self {
        let fonts = vec![
            Font::with_name("Quicksand"),
            Font::with_name("Noto Sans"),
        ];
        Self { song: None, fonts }
    }
    pub fn update(&self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeSong(song) => todo!(),
            Message::UpdateSong(song) => todo!(),
            Message::ChangeFont(font) => todo!(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let font_selector =
            dropdown(&["Quicksand", "Noto Sans"], None, |font| {
                Message::ChangeFont(font)
            });
        let toolbar = row![font_selector];
        toolbar.into()
    }
}
