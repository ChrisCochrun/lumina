use cosmic::{
    iced_widget::{column, row},
    widget::{dropdown, text, text_input},
    Element, Task,
};
use tracing::debug;

use crate::core::songs::Song;

#[derive(Debug, Clone)]
pub struct SongEditor {
    song: Option<Song>,
    title: String,
    fonts: Vec<String>,
    font_sizes: Vec<String>,
    font: String,
    font_size: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeSong(Song),
    UpdateSong(Song),
    ChangeFont(usize),
    ChangeFontSize(usize),
    ChangeTitle(String),
}

impl SongEditor {
    pub fn new() -> Self {
        let fonts = vec![
            String::from("Quicksand"),
            String::from("Noto Sans"),
        ];
        let font_sizes = vec![
            "10".to_string(),
            "12".to_string(),
            "16".to_string(),
            "18".to_string(),
            "20".to_string(),
        ];
        Self {
            song: None,
            fonts,
            title: String::from("Death was Arrested"),
            font: String::from("Quicksand"),
            font_size: 16,
            font_sizes,
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeSong(song) => todo!(),
            Message::UpdateSong(song) => todo!(),
            Message::ChangeFont(font) => {
                if let Some(font) = self.fonts.get(font) {
                    debug!(font);
                    self.font = font.to_owned();
                }
                Task::none()
            }
            Message::ChangeFontSize(size) => {
                if let Some(size) = self.font_sizes.get(size) {
                    if let Ok(size) = size.parse() {
                        debug!(font_size = size);
                        self.font_size = size;
                    }
                }
                Task::none()
            }
            Message::ChangeTitle(title) => {
                self.title = title;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let selected_font =
            self.fonts.iter().position(|f| *f == self.font);
        let selected_font_size = self
            .font_sizes
            .iter()
            .position(|s| *s == self.font_size.to_string());
        let font_selector =
            dropdown(&self.fonts, selected_font, Message::ChangeFont)
                .width(200);
        let font_size = dropdown(
            &self.font_sizes,
            selected_font_size,
            Message::ChangeFontSize,
        );
        let title = text(&self.title);
        let title_input = text_input("song", &self.title)
            .on_input(Message::ChangeTitle);
        let toolbar = row![font_selector, font_size];
        let column = column![toolbar, title, title_input];
        column.into()
    }
}
