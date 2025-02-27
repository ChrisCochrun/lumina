use std::path::PathBuf;

use cosmic::{
    iced::{
        font::{Family, Stretch, Style, Weight},
        Font, Length,
    },
    iced_widget::row,
    theme,
    widget::{
        button, column, combo_box, container, dropdown,
        horizontal_space, icon, scrollable, text, text_editor,
        text_input,
    },
    Element, Task,
};
use iced_video_player::Video;
use tracing::debug;

use crate::{
    core::{service_items::ServiceTrait, songs::Song},
    Background,
};

use super::presenter::slide_view;

#[derive(Debug)]
pub struct SongEditor {
    pub song: Option<Song>,
    title: String,
    fonts: combo_box::State<String>,
    font_sizes: Vec<String>,
    font: String,
    author: String,
    audio: PathBuf,
    font_size: usize,
    verse_order: String,
    pub lyrics: text_editor::Content,
    editing: bool,
    background: Option<Background>,
    video: Option<Video>,
    current_font: Font,
    ccli: String,
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
    None,
    ChangeAuthor(String),
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
            title: "Death was Arrested".to_owned(),
            font: "Quicksand".to_owned(),
            font_size: 16,
            font_sizes,
            verse_order: "Death was Arrested".to_owned(),
            lyrics: text_editor::Content::new(),
            editing: false,
            author: "North Point Worship".into(),
            audio: PathBuf::new(),
            background: None,
            video: None,
            current_font: cosmic::font::default(),
            ccli: "8".to_owned(),
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChangeSong(song) => {
                self.song = Some(song.clone());
                self.title = song.title;
                if let Some(font) = song.font {
                    self.font = font
                };
                if let Some(font_size) = song.font_size {
                    self.font_size = font_size as usize
                };
                if let Some(verse_order) = song.verse_order {
                    self.verse_order = verse_order
                        .into_iter()
                        .map(|mut s| {
                            s.push_str(" ");
                            s
                        })
                        .collect();
                }
                if let Some(author) = song.author {
                    self.author = author
                };
                if let Some(audio) = song.audio {
                    self.audio = audio
                };
                if let Some(ccli) = song.ccli {
                    self.ccli = ccli
                };
                if let Some(lyrics) = song.lyrics {
                    self.lyrics =
                        text_editor::Content::with_text(&lyrics)
                };
                self.background = song.background;

                Task::none()
            }
            Message::UpdateSong(song) => {
                self.song = Some(song);
                Task::none()
            }
            Message::ChangeFont(font) => {
                self.font = font.clone();

                let font_name = font.into_boxed_str();
                let family = Family::Name(Box::leak(font_name));
                let weight = Weight::Normal;
                let stretch = Stretch::Normal;
                let style = Style::Normal;
                let font = Font {
                    family,
                    weight,
                    stretch,
                    style,
                };
                self.current_font = font;
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
                self.title = title.clone();
                Task::none()
            }
            Message::ChangeVerseOrder(verse_order) => {
                self.verse_order = verse_order.clone();
                if let Some(mut song) = self.song.clone() {
                    let verse_order = verse_order
                        .split(" ")
                        .into_iter()
                        .map(|s| s.to_owned())
                        .collect();
                    song.verse_order = Some(verse_order);
                    self.update(Message::UpdateSong(song))
                } else {
                    Task::none()
                }
            }
            Message::ChangeLyrics(action) => {
                self.lyrics.perform(action);

                let lyrics = self.lyrics.text();

                if let Some(mut song) = self.song.clone() {
                    song.lyrics = Some(lyrics);
                    self.update(Message::UpdateSong(song))
                } else {
                    Task::none()
                }
            }
            Message::Edit(edit) => {
                debug!(edit);
                self.editing = edit;
                Task::none()
            }
            Message::None => Task::none(),
            Message::ChangeAuthor(author) => {
                debug!(author);
                self.author = author.clone();
                if let Some(mut song) = self.song.clone() {
                    song.author = Some(author);
                    self.update(Message::UpdateSong(song))
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let slide_preview = container(self.slide_preview())
            .width(Length::FillPortion(2));

        let column = column::with_children(vec![
            self.toolbar(),
            row![
                container(self.left_column())
                    .center_x(Length::FillPortion(2)),
                container(slide_preview)
                    .center_x(Length::FillPortion(3))
            ]
            .into(),
        ])
        .spacing(theme::active().cosmic().space_l());
        column.into()
    }

    fn slide_preview(&self) -> Element<Message> {
        if let Some(song) = &self.song {
            if let Ok(slides) = song.to_slides() {
                let slides = slides
                    .into_iter()
                    .map(|slide| {
                        container(
                            slide_view(
                                slide,
                                &self.video,
                                self.current_font,
                                false,
                                false,
                            )
                            .map(|_| Message::None),
                        )
                        .height(250)
                        .center_x(Length::Fill)
                        .padding([0, 20])
                        .clip(true)
                        .into()
                    })
                    .collect();
                scrollable(
                    column::with_children(slides)
                        .spacing(theme::active().cosmic().space_l()),
                )
                .height(Length::Fill)
                .width(Length::Fill)
                .into()
            } else {
                horizontal_space().into()
            }
        } else {
            horizontal_space().into()
        }
    }

    fn left_column(&self) -> Element<Message> {
        let title_input = text_input("song", &self.title)
            .on_input(Message::ChangeTitle)
            .label("Song Title");

        let author_input = text_input("author", &self.author)
            .on_input(Message::ChangeAuthor)
            .label("Song Author");

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
            author_input.into(),
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

        let background_selector = button::icon(
            icon::from_name("folder-pictures-symbolic").scale(2),
        )
        .label("Background")
        .tooltip("Select an image or video background")
        .on_press(Message::None)
        .padding(10);

        row![
            font_selector,
            font_size,
            horizontal_space(),
            background_selector
        ]
        .into()
    }

    pub fn editing(&self) -> bool {
        self.editing
    }
}

impl Default for SongEditor {
    fn default() -> Self {
        Self::new()
    }
}
