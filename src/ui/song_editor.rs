use cosmic::{
    iced::Length,
    iced_widget::row,
    widget::{
        column, combo_box, container, dropdown, text, text_editor,
        text_input, vertical_space,
    },
    Element, Task,
};
use tracing::debug;

use crate::core::songs::Song;

#[derive(Debug)]
pub struct SongEditor {
    song: Option<Song>,
    title: String,
    fonts: combo_box::State<String>,
    font_sizes: Vec<String>,
    font: String,
    font_size: usize,
    verse_order: String,
    lyrics: text_editor::Content,
    editing: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeSong(Song),
    UpdateSong(Song),
    ChangeFont(String),
    ChangeFontSize(usize),
    ChangeTitle(String),
    ChangeVerseOrder(String),
    ChangeLyrics(text_editor::Action),
    Edit(bool),
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
            fonts: combo_box::State::new(fonts),
            title: String::from("Death was Arrested"),
            font: String::from("Quicksand"),
            font_size: 16,
            font_sizes,
            verse_order: String::from("Death was Arrested"),
            lyrics: text_editor::Content::new(),
            editing: false,
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeSong(song) => {
                self.song = Some(song);
                Task::none()
            }
            Message::UpdateSong(song) => {
                self.song = Some(song);
                Task::none()
            }
            Message::ChangeFont(font) => {
                self.font = font;
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
                debug!(title);
                self.title = title;
                Task::none()
            }
            Message::ChangeVerseOrder(verse_order) => {
                self.verse_order = verse_order;
                Task::none()
            }
            Message::ChangeLyrics(action) => {
                self.lyrics.perform(action);
                Task::none()
            }
            Message::Edit(edit) => {
                debug!(edit);
                self.editing = edit;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let slide_preview =
            container(vertical_space()).width(Length::FillPortion(2));

        let column = column::with_children(vec![
            self.toolbar(),
            row![self.left_column(), slide_preview].into(),
        ]);
        column.into()
    }

    fn left_column(&self) -> Element<Message> {
        let title_input = text_input("song", &self.title)
            .on_input(Message::ChangeTitle)
            .label("Song Title");

        let verse_input = text_input(
            "Verse
order",
            &self.verse_order,
        )
        .label("Verse Order")
        .on_input(Message::ChangeVerseOrder);

        let lyric_title = text("Lyrics");
        let lyric_input = column::with_children(vec![
            lyric_title.into(),
            text_editor(&self.lyrics)
                .on_action(Message::ChangeLyrics)
                .height(Length::Fill)
                .into(),
        ])
        .spacing(5);

        column::with_children(vec![
            title_input.into(),
            verse_input.into(),
            lyric_input.into(),
        ])
        .spacing(25)
        .width(Length::FillPortion(2))
        .into()
    }

    fn toolbar(&self) -> Element<Message> {
        let selected_font = &self.font;
        let selected_font_size = self
            .font_sizes
            .iter()
            .position(|s| *s == self.font_size.to_string());
        let font_selector = combo_box(
            &self.fonts,
            "Font",
            Some(selected_font),
            Message::ChangeFont,
        )
        .width(200);
        let font_size = dropdown(
            &self.font_sizes,
            selected_font_size,
            Message::ChangeFontSize,
        );

        row![font_selector, font_size].into()
    }

    pub fn editing(&self) -> bool {
        self.editing
    }
}
