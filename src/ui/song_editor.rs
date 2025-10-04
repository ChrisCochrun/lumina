use std::{io, path::PathBuf, sync::Arc};

use cosmic::{
    Element, Task,
    dialog::file_chooser::{FileFilter, open::Dialog},
    iced::{Length, alignment::Vertical},
    iced_wgpu::graphics::text::cosmic_text::fontdb,
    iced_widget::{column, row},
    theme,
    widget::{
        button, combo_box, container, horizontal_space, icon,
        progress_bar, scrollable, text, text_editor, text_input,
    },
};
use dirs::font_dir;
use iced_video_player::Video;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::{debug, error};

use crate::{
    Background, BackgroundKind,
    core::{service_items::ServiceTrait, slide::Slide, songs::Song},
    ui::{
        presenter::slide_view, slide_editor::SlideEditor, text_svg,
    },
};

#[derive(Debug)]
pub struct SongEditor {
    pub song: Option<Song>,
    title: String,
    font_db: Arc<fontdb::Database>,
    fonts_combo: combo_box::State<String>,
    font_sizes: combo_box::State<String>,
    font: String,
    author: String,
    audio: PathBuf,
    font_size: usize,
    verse_order: String,
    pub lyrics: text_editor::Content,
    editing: bool,
    background: Option<Background>,
    video: Option<Video>,
    ccli: String,
    song_slides: Option<Vec<Slide>>,
    slide_state: SlideEditor,
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
    UpdateSlides(Vec<Slide>),
    PickBackground,
    Edit(bool),
    None,
    ChangeAuthor(String),
    PauseVideo,
}

impl SongEditor {
    pub fn new(font_db: Arc<fontdb::Database>) -> Self {
        let fonts = font_dir();
        debug!(?fonts);
        let mut fonts: Vec<String> = font_db
            .faces()
            .map(|f| f.families[0].0.clone())
            .collect();
        fonts.dedup();
        fonts.sort();
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
            "90".to_string(),
            "100".to_string(),
            "110".to_string(),
            "120".to_string(),
            "130".to_string(),
            "140".to_string(),
            "150".to_string(),
            "160".to_string(),
            "170".to_string(),
        ];
        Self {
            song: None,
            font_db,
            fonts_combo: combo_box::State::new(fonts),
            title: "Death was Arrested".to_string(),
            font: "Quicksand".to_string(),
            font_size: 16,
            font_sizes: combo_box::State::new(font_sizes),
            verse_order: "Death was Arrested".to_string(),
            lyrics: text_editor::Content::new(),
            editing: false,
            author: "North Point Worship".into(),
            audio: PathBuf::new(),
            background: None,
            video: None,
            ccli: "8".to_string(),
            slide_state: SlideEditor::default(),
            song_slides: None,
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeSong(song) => {
                self.song = Some(song.clone());
                let song_slides = song.clone().to_slides();
                self.title = song.title;
                if let Some(font) = song.font {
                    self.font = font;
                }
                if let Some(font_size) = song.font_size {
                    self.font_size = font_size as usize;
                }
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
                    self.author = author;
                }
                if let Some(audio) = song.audio {
                    self.audio = audio;
                }
                if let Some(ccli) = song.ccli {
                    self.ccli = ccli;
                }
                if let Some(lyrics) = song.lyrics {
                    self.lyrics =
                        text_editor::Content::with_text(&lyrics);
                }
                self.background_video(&song.background);
                self.background = song.background.clone();
                let font_db = Arc::clone(&self.font_db);
                let task = Task::perform(
                    async move {
                        song_slides
                            .ok()
                            .map(move |v| {
                                v.into_par_iter()
                                    .map(move |mut s| {
                                        text_svg::text_svg_generator(
                                            &mut s,
                                            Arc::clone(&font_db),
                                        );
                                        s
                                    })
                                    .collect::<Vec<Slide>>()
                            })
                            .unwrap_or_default()
                    },
                    |slides| Message::UpdateSlides(slides),
                );
                return Action::Task(task);
            }
            Message::ChangeFont(font) => {
                self.font = font.clone();
                if let Some(song) = &mut self.song {
                    song.font = Some(font);
                    let song = song.to_owned();
                    return self.update_song(song);
                }
            }
            Message::ChangeFontSize(size) => {
                self.font_size = size;
                if let Some(song) = &mut self.song {
                    song.font_size = Some(size as i32);
                    let song = song.to_owned();
                    return self.update_song(song);
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
                        .split(' ')
                        .map(std::borrow::ToOwned::to_owned)
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
                    let background = Background::try_from(path).ok();
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
                ));
            }
            Message::PauseVideo => {
                if let Some(video) = &mut self.video {
                    let paused = video.paused();
                    video.set_paused(!paused);
                };
            }
            Message::UpdateSlides(slides) => {
                self.song_slides = Some(slides);
            }
            Message::UpdateSong(song) => {
                self.song = Some(song.clone());
                return Action::UpdateSong(song);
            }
            _ => (),
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let video_elements = if let Some(video) = &self.video {
            let play_button = button::icon(if video.paused() {
                icon::from_name("media-playback-start")
            } else {
                icon::from_name("media-playback-pause")
            })
            .on_press(Message::PauseVideo);
            let video_track = progress_bar(
                0.0..=video.duration().as_secs_f32(),
                video.position().as_secs_f32(),
            )
            .height(cosmic::theme::spacing().space_s)
            .width(Length::Fill);
            container(
                row![play_button, video_track]
                    .align_y(Vertical::Center)
                    .spacing(cosmic::theme::spacing().space_m),
            )
            .padding(cosmic::theme::spacing().space_s)
            .center_x(Length::FillPortion(2))
        } else {
            container(horizontal_space())
        };
        let slide_preview = container(self.slide_preview())
            .width(Length::FillPortion(2));

        let slide_section = column![video_elements, slide_preview]
            .spacing(cosmic::theme::spacing().space_s);
        let column = column![
            self.toolbar(),
            row![
                container(self.left_column())
                    .center_x(Length::FillPortion(2)),
                container(slide_section)
                    .center_x(Length::FillPortion(2))
            ],
        ]
        .spacing(theme::active().cosmic().space_l());
        column.into()
    }

    fn slide_preview(&self) -> Element<Message> {
        if let Some(slides) = &self.song_slides {
            let slides: Vec<Element<Message>> = slides
                .iter()
                .enumerate()
                .map(|(index, slide)| {
                    container(
                        slide_view(
                            slide,
                            if index == 0 {
                                &self.video
                            } else {
                                &None
                            },
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
                cosmic::widget::column::with_children(slides)
                    .spacing(theme::active().cosmic().space_l()),
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
        } else {
            horizontal_space().into()
        }
        // self.slide_state
        //     .view(Font::with_name("Quicksand Bold"))
        //     .map(|_s| Message::None)
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
        let lyric_input = column![
            lyric_title,
            text_editor(&self.lyrics)
                .on_action(Message::ChangeLyrics)
                .height(Length::Fill)
        ]
        .spacing(5);

        column![title_input, author_input, verse_input, lyric_input,]
            .spacing(25)
            .width(Length::FillPortion(2))
            .into()
    }

    fn toolbar(&self) -> Element<Message> {
        let selected_font = &self.font;
        let selected_font_size = if self.font_size > 0 {
            Some(&self.font_size.to_string())
        } else {
            None
        };
        let font_selector = combo_box(
            &self.fonts_combo,
            "Font",
            Some(selected_font),
            Message::ChangeFont,
        )
        .width(300);
        let font_size = combo_box(
            &self.font_sizes,
            "Font Size",
            selected_font_size,
            |size| {
                Message::ChangeFontSize(
                    size.parse().expect("Should be a number"),
                )
            },
        )
        .width(theme::active().cosmic().space_xxl());

        let background_selector = button::icon(
            icon::from_name("folder-pictures-symbolic").scale(2),
        )
        .label("Background")
        .tooltip("Select an image or video background")
        .on_press(Message::PickBackground)
        .padding(10);

        row![
            text::body("Font:"),
            font_selector,
            text::body("Font Size:"),
            font_size,
            horizontal_space(),
            background_selector
        ]
        .align_y(Vertical::Center)
        .spacing(10)
        .into()
    }

    pub const fn editing(&self) -> bool {
        self.editing
    }

    fn update_song(&mut self, song: Song) -> Action {
        self.song = Some(song.clone());

        let font_db = Arc::clone(&self.font_db);
        let update_task =
            Task::done(Message::UpdateSong(song.clone()));
        let task = Task::perform(
            async move {
                song.to_slides()
                    .ok()
                    .map(move |v| {
                        v.into_par_iter()
                            .map(move |mut s| {
                                text_svg::text_svg_generator(
                                    &mut s,
                                    Arc::clone(&font_db),
                                );
                                s
                            })
                            .collect::<Vec<Slide>>()
                    })
                    .unwrap_or_default()
            },
            |slides| Message::UpdateSlides(slides),
        );
        Action::Task(task.chain(update_task))
    }

    fn background_video(&mut self, background: &Option<Background>) {
        if let Some(background) = background
            && background.kind == BackgroundKind::Video
        {
            let video =
                Video::try_from(background).ok().map(|mut v| {
                    v.set_looping(true);
                    v
                });
            // debug!(?video);
            self.video = video;
        } else {
            self.video = None;
        }
    }
}

impl Default for SongEditor {
    fn default() -> Self {
        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        Self::new(Arc::new(fontdb))
    }
}

async fn pick_background() -> Result<PathBuf, SongError> {
    let dialog = Dialog::new().title("Choose a background...");
    let bg_filter = FileFilter::new("Videos and Images")
        .extension("png")
        .extension("jpg")
        .extension("mp4")
        .extension("webm")
        .extension("mkv")
        .extension("jpeg");
    dialog
        .filter(bg_filter)
        .directory(dirs::home_dir().expect("oops"))
        .open_file()
        .await
        .map_err(|e| {
            error!(?e);
            SongError::BackgroundDialogClosed
        })
        .map(|file| file.url().to_file_path().unwrap())
    // rfd::AsyncFileDialog::new()
    //     .set_title("Choose a background...")
    //     .add_filter(
    //         "Images and Videos",
    //         &["png", "jpeg", "mp4", "webm", "mkv", "jpg", "mpeg"],
    //     )
    //     .set_directory(dirs::home_dir().unwrap())
    //     .pick_file()
    //     .await
    //     .ok_or(SongError::BackgroundDialogClosed)
    //     .map(|file| file.path().to_owned())
}

#[derive(Debug, Clone)]
pub enum SongError {
    BackgroundDialogClosed,
    IOError(io::ErrorKind),
}
