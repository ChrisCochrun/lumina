use std::{io, path::PathBuf};

use cosmic::{
    dialog::file_chooser::open::Dialog,
    iced::{
        font::{Family, Stretch, Style, Weight},
        Font, Length,
    },
    iced_wgpu::graphics::text::cosmic_text::fontdb,
    iced_widget::{row, stack},
    theme,
    widget::{
        button, column, combo_box, container, dropdown,
        horizontal_space, icon, scrollable, svg::Handle, text,
        text_editor, text_input, Svg,
    },
    Element, Task,
};
use dirs::font_dir;
use iced_video_player::Video;
use tracing::{debug, error};

use crate::{
    core::{service_items::ServiceTrait, songs::Song},
    Background, BackgroundKind,
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

pub enum Action {
    Task(Task<Message>),
    UpdateSong(Song),
    None,
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
    ChangeBackground(Result<PathBuf, SongError>),
    PickBackground,
    Edit(bool),
    None,
    ChangeAuthor(String),
}

impl SongEditor {
    pub fn new() -> Self {
        let fonts = font_dir();
        debug!(?fonts);
        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        let fonts: Vec<String> = fontdb
            .faces()
            .map(|f| {
                let mut font = f.to_owned().post_script_name;
                if let Some(at) = font.find("-") {
                    let _ = font.split_off(at);
                }
                let indices: Vec<usize> = font
                    .chars()
                    .enumerate()
                    .filter(|(index, c)| {
                        c.is_uppercase() && *index != 0
                    })
                    .map(|(index, c)| index)
                    .collect();

                let mut font_parts = vec![];
                for index in indices.iter().rev() {
                    let (first, last) = font.split_at(*index);
                    font_parts.push(first);
                    if !last.is_empty() {
                        font_parts.push(last);
                    }
                }
                font_parts
                    .iter()
                    .map(|s| {
                        let mut s = s.to_string();
                        s.push(' ');
                        s
                    })
                    .collect()
            })
            .collect();
        // let fonts = vec![
        //     String::from("Quicksand"),
        //     String::from("Noto Sans"),
        // ];
        let font_sizes = vec![
            "5".to_string(),
            "6".to_string(),
            "8".to_string(),
            "10".to_string(),
            "12".to_string(),
            "16".to_string(),
            "18".to_string(),
            "20".to_string(),
            "24".to_string(),
            "28".to_string(),
            "32".to_string(),
            "36".to_string(),
            "40".to_string(),
            "48".to_string(),
            "50".to_string(),
            "55".to_string(),
            "60".to_string(),
            "65".to_string(),
            "70".to_string(),
            "80".to_string(),
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
    pub fn update(&mut self, message: Message) -> Action {
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
                            s.push(' ');
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
                self.background_video(&song.background);
                self.background = song.background;
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
                // return self.update_song(song);
            }
            Message::ChangeFontSize(size) => {
                if let Some(size) = self.font_sizes.get(size) {
                    if let Ok(size) = size.parse() {
                        debug!(font_size = size);
                        self.font_size = size;
                        // return self.update_song(song);
                    }
                }
            }
            Message::ChangeTitle(title) => {
                self.title = title.clone();
                if let Some(song) = &mut self.song {
                    song.title = title;
                    let song = song.to_owned();
                    return self.update_song(song);
                }
            }
            Message::ChangeVerseOrder(verse_order) => {
                self.verse_order = verse_order.clone();
                if let Some(mut song) = self.song.clone() {
                    let verse_order = verse_order
                        .split(" ")
                        .map(|s| s.to_owned())
                        .collect();
                    song.verse_order = Some(verse_order);
                    return self.update_song(song);
                }
            }
            Message::ChangeLyrics(action) => {
                self.lyrics.perform(action);

                let lyrics = self.lyrics.text();

                if let Some(mut song) = self.song.clone() {
                    song.lyrics = Some(lyrics);
                    return self.update_song(song);
                }
            }
            Message::Edit(edit) => {
                debug!(edit);
                self.editing = edit;
            }
            Message::ChangeAuthor(author) => {
                debug!(author);
                self.author = author.clone();
                if let Some(mut song) = self.song.clone() {
                    song.author = Some(author);
                    return self.update_song(song);
                }
            }
            Message::ChangeBackground(Ok(path)) => {
                debug!(?path);
                if let Some(mut song) = self.song.clone() {
                    let background =
                        Background::try_from(path.clone()).ok();
                    self.background_video(&background);
                    song.background = background;
                    return self.update_song(song);
                }
            }
            Message::ChangeBackground(Err(error)) => {
                error!(?error);
            }
            Message::PickBackground => {
                return Action::Task(Task::perform(
                    pick_background(),
                    Message::ChangeBackground,
                ))
            }
            _ => (),
        }
        Action::None
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
                    .enumerate()
                    .map(|(index, slide)| {
                        let svg = Handle::from_memory(r#"<svg viewBox="0 0 1280 720" xmlns="http://www.w3.org/2000/svg">
<defs>
     <filter id="shadow">
      <feDropShadow dx="10" dy="10" stdDeviation="5" flood-color='#000' />
    </filter>
</defs>
<text dominant-baseline="middle" text-anchor="middle" font-weight="bold" font-family="Quicksand" font-size="80" fill="white" stroke="black" stroke-width="2" style="filter:url(#shadow);">
    <tspan x="50%" y="50" >Hello World this is</tspan>
    <tspan x="50%" y="140">longer chunks of text</tspan>
    <tspan x="50%" y="230">where we need to test whether the text</tspan>
    <tspan x="50%" y="320">will look ok!</tspan>
</text>
</svg>"#.as_bytes());
                        stack!(
                            container(
                                slide_view(
                                    slide,
                                    if index == 0 {
                                        &self.video
                                    } else {
                                        &None
                                    },
                                    self.current_font,
                                    false,
                                    false,
                                )
                                    .map(|_| Message::None),
                            )
                                .height(250)
                                .center_x(Length::Fill)
                                .padding([0, 20])
                                .clip(true),
                            Svg::new(svg),
                        ).into()
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
        .on_press(Message::PickBackground)
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

    fn update_song(&mut self, song: Song) -> Action {
        self.song = Some(song.clone());
        Action::UpdateSong(song)
    }

    fn background_video(&mut self, background: &Option<Background>) {
        if let Some(background) = background {
            if background.kind == BackgroundKind::Video {
                let video =
                    Video::try_from(background).ok().map(|mut v| {
                        v.set_looping(true);
                        v
                    });
                debug!(?video);
                self.video = video;
            }
        }
    }
}

impl Default for SongEditor {
    fn default() -> Self {
        Self::new()
    }
}

async fn pick_background() -> Result<PathBuf, SongError> {
    let dialog = Dialog::new().title("Choose a background...");
    dialog
        .open_file()
        .await
        .map_err(|_| SongError::DialogClosed)
        .map(|file| file.url().to_file_path().unwrap())
    // rfd::AsyncFileDialog::new()
    //     .set_title("Choose a background...")
    //     .pick_file()
    //     .await
    //     .ok_or(SongError::DialogClosed)
    //     .map(|file| file.path().to_owned())
}

#[derive(Debug, Clone)]
pub enum SongError {
    DialogClosed,
    IOError(io::ErrorKind),
}
