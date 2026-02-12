use std::{
    io::{self},
    path::PathBuf,
    sync::Arc,
};

use cosmic::{
    Apply, Element, Task,
    dialog::file_chooser::{FileFilter, open::Dialog},
    iced::{
        Background as ContainerBackground, Border, Color, Length,
        Padding, Shadow, Vector, alignment::Vertical, color,
        futures::StreamExt, task,
    },
    iced_core::widget::tree,
    iced_wgpu::graphics::text::cosmic_text::fontdb,
    iced_widget::{
        column, row,
        scrollable::{Direction, Scrollbar},
        stack,
    },
    theme,
    widget::{
        ColorPickerModel, RcElementWrapper, button,
        color_picker::{self, ColorPickerUpdate},
        combo_box, container, divider, dnd_destination, dnd_source,
        dropdown,
        grid::{self},
        horizontal_space, icon, mouse_area, popover, progress_bar,
        scrollable, spin_button, text, text_editor, text_input,
        tooltip,
    },
};
use derive_more::Debug;
use dirs::font_dir;
use iced_video_player::Video;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::{debug, error};

use crate::{
    Background, BackgroundKind,
    core::{
        service_items::ServiceTrait,
        slide::{Slide, TextAlignment},
        songs::{Song, VerseName},
    },
    ui::{
        presenter::slide_view,
        slide_editor::SlideEditor,
        text_svg,
        widgets::{
            draggable,
            verse_editor::{self, VerseEditor},
        },
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
    font_size_open: bool,
    font_selector_open: bool,
    verse_order: String,
    pub lyrics: text_editor::Content,
    editing: bool,
    editing_verse_order: bool,
    background: Option<Background>,
    video: Option<Video>,
    ccli: String,
    song_slides: Option<Vec<Slide>>,
    slide_state: SlideEditor,
    stroke_sizes: combo_box::State<i32>,
    stroke_size: u16,
    stroke_open: bool,
    #[debug(skip)]
    stroke_color_model: ColorPickerModel,
    verses: Option<Vec<VerseEditor>>,
    hovered_verse_chip: Option<usize>,
    hovered_dnd_verse_chip: Option<usize>,
    stroke_color_picker_open: bool,
    dragging_verse_chip: bool,
    update_slide_handle: Option<task::Handle>,
    alignment_popup: bool,
    #[debug(skip)]
    shadow_color_model: ColorPickerModel,
    shadow_tools_open: bool,
    importing: bool,
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
    ChangeFontSize(String),
    ChangeTitle(String),
    ChangeVerseOrder(String),
    ChangeLyrics(text_editor::Action),
    ChangeBackground(Result<PathBuf, SongError>),
    UpdateSlides(Vec<Slide>),
    UpdateSlide((usize, Slide)),
    PickBackground,
    Edit(bool),
    None,
    ChangeAuthor(String),
    PauseVideo,
    UpdateStrokeSize(u16),
    UpdateStrokeColor(ColorPickerUpdate),
    OpenStroke,
    CloseStroke,
    VerseEditorMessage((usize, verse_editor::Message)),
    FontSizeOpen(bool),
    FontSelectorOpen(bool),
    EditVerseOrder,
    OpenStrokeColorPicker,
    ChipHovered(Option<usize>),
    ChipDndHovered(Option<usize>),
    ChipDropped((usize, Vec<u8>, String)),
    ChipReorder(draggable::DragEvent),
    DraggingChipStart,
    ChipDroppedEnd((Vec<u8>, String)),
    AddVerse((VerseName, String)),
    RemoveVerse(usize),
    AlignmentPopupOpen,
    SetTextAlignment(TextAlignment),
    OpenShadowTools,
    UpdateShadowColor(ColorPickerUpdate),
    UpdateShadowSize(u16),
    UpdateShadowOffsetX(i16),
    UpdateShadowOffsetY(i16),
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
        let stroke_sizes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
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
            title: String::new(),
            font: String::new(),
            font_size: 100,
            font_sizes: combo_box::State::new(font_sizes),
            font_size_open: false,
            font_selector_open: false,
            verse_order: String::new(),
            lyrics: text_editor::Content::new(),
            editing: false,
            author: String::new(),
            audio: PathBuf::new(),
            background: None,
            video: None,
            ccli: String::new(),
            slide_state: SlideEditor::default(),
            song_slides: None,
            stroke_sizes: combo_box::State::new(stroke_sizes),
            stroke_size: 0,
            stroke_open: false,
            stroke_color_model: ColorPickerModel::new(
                "hex",
                "rgb",
                Some(Color::BLACK),
                Some(Color::BLACK),
            ),
            stroke_color_picker_open: false,
            verses: None,
            editing_verse_order: false,
            hovered_dnd_verse_chip: None,
            hovered_verse_chip: None,
            dragging_verse_chip: false,
            update_slide_handle: None,
            alignment_popup: false,
            shadow_color_model: ColorPickerModel::new(
                "hex",
                "rgb",
                Some(Color::BLACK),
                Some(Color::BLACK),
            ),
            shadow_tools_open: false,
            importing: false,
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeSong(song) => {
                let mut tasks = vec![];
                self.song = Some(song.clone());
                let song_slides = song.clone().to_slides();
                self.title = song.title;
                self.font_size_open = false;
                self.font_selector_open = false;
                self.editing_verse_order = false;
                self.alignment_popup = false;
                self.stroke_color_picker_open = false;
                self.shadow_tools_open = false;
                if let Some(stroke_size) = song.stroke_size {
                    self.stroke_size = stroke_size;
                }
                if let Some(stroke_color) = song.stroke_color {
                    self.stroke_color_model = ColorPickerModel::new(
                        "hex",
                        "rgb",
                        Some(Color::BLACK),
                        Some(stroke_color.into()),
                    );
                }
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
                self.song_slides = None;
                let font_db = Arc::clone(&self.font_db);

                tasks.push(Task::perform(
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
                    Message::UpdateSlides,
                ));

                self.verses = song.verse_map.map(|map| {
                    map.into_iter()
                        .sorted()
                        .map(|(verse_name, lyric)| {
                            VerseEditor::new(verse_name, lyric)
                        })
                        .collect()
                });
                return Action::Task(Task::batch(tasks));
            }
            Message::ChangeFont(font) => {
                self.font = font.clone();
                if let Some(song) = &mut self.song {
                    song.font = Some(font);
                    let song = song.to_owned();
                    return Action::Task(self.update_song(song));
                }
            }
            Message::ChangeFontSize(size) => {
                if let Ok(size) = size.parse() {
                    self.font_size = size;
                    if let Some(song) = &mut self.song {
                        song.font_size = Some(size as i32);
                        let song = song.to_owned();
                        return Action::Task(self.update_song(song));
                    }
                }
            }
            Message::ChangeTitle(title) => {
                self.title = title.clone();
                if let Some(song) = &mut self.song {
                    song.title = title;
                    let song = song.to_owned();
                    return Action::Task(self.update_song(song));
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
                    return Action::Task(self.update_song(song));
                }
            }
            Message::ChangeLyrics(action) => {
                self.lyrics.perform(action);

                let lyrics = self.lyrics.text();

                if let Some(mut song) = self.song.clone() {
                    song.lyrics = Some(lyrics);
                    return Action::Task(self.update_song(song));
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
                    return Action::Task(self.update_song(song));
                }
            }
            Message::ChangeBackground(Ok(path)) => {
                debug!(?path);
                if let Some(mut song) = self.song.clone() {
                    let background = Background::try_from(path).ok();
                    self.background_video(&background);
                    song.background = background;
                    return Action::Task(self.update_song(song));
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
                    video.set_paused(!video.paused());
                }
            }
            Message::UpdateStrokeSize(size) => {
                self.stroke_size = size;
                if let Some(song) = &mut self.song {
                    if size == 0 {
                        song.stroke_size = None;
                    } else {
                        song.stroke_size = Some(size);
                    }
                    let song = song.to_owned();
                    return Action::Task(self.update_song(song));
                }
            }
            Message::UpdateStrokeColor(update) => {
                let mut tasks = Vec::with_capacity(2);
                tasks.push(self.stroke_color_model.update(update));
                if let Some(mut song) = self.song.clone()
                    && let Some(color) =
                        self.stroke_color_model.get_applied_color()
                {
                    debug!(?color);
                    song.stroke_color = Some(color.into());
                    tasks.push(self.update_song(song));
                }
                return Action::Task(Task::batch(tasks));
            }
            Message::UpdateSlides(slides) => {
                self.song_slides = Some(slides);
                self.update_slide_handle = None;
            }
            Message::UpdateSlide((index, slide)) => {
                if let Some(slides) = self.song_slides.as_mut() {
                    if let Some(_old) = slides.get(index) {
                        let _ = slides.remove(index);
                        slides.insert(index, slide);
                    } else {
                        slides.push(slide);
                    }
                } else {
                    self.song_slides = Some(vec![slide]);
                }
                self.update_slide_handle = None;
            }
            Message::UpdateSong(song) => {
                self.song = Some(song.clone());
                return Action::UpdateSong(song);
            }
            Message::OpenStroke => {
                self.stroke_open = true;
            }
            Message::CloseStroke => {
                self.stroke_open = false;
            }
            Message::OpenStrokeColorPicker => {
                self.stroke_color_picker_open =
                    !self.stroke_color_picker_open;
            }
            Message::VerseEditorMessage((index, message)) => {
                if let Some(verses) = self.verses.as_mut()
                    && let Some(verse) = verses.get_mut(index)
                {
                    match verse.update(message) {
                        verse_editor::Action::Task(task) => {
                            return Action::Task(task.map(
                                move |m| {
                                    Message::VerseEditorMessage((
                                        index, m,
                                    ))
                                },
                            ));
                        }
                        verse_editor::Action::UpdateVerseName(
                            verse_name,
                        ) => {
                            if let Some(mut song) = self.song.clone()
                            {
                                let old_verse_name =
                                    verse.verse_name;

                                let verse_name = song
                                    .verse_name_from_str(
                                        verse_name,
                                        old_verse_name,
                                    );

                                verse.verse_name = verse_name;

                                if verse_name == VerseName::Blank {
                                    verse.lyric = String::new();
                                }

                                song.update_verse_name(
                                    verse_name,
                                    &old_verse_name,
                                );

                                return Action::Task(
                                    self.update_song(song),
                                );
                            }
                        }
                        verse_editor::Action::UpdateVerse((
                            verse,
                            lyric,
                        )) => {
                            if let Some(mut song) = self.song.clone()
                            {
                                song.set_lyrics(&verse, lyric);
                                // song.update_verse(
                                //     index, verse, lyric,
                                // );
                                return Action::Task(
                                    self.update_song(song),
                                );
                            }
                        }
                        verse_editor::Action::DeleteVerse(verse) => {
                            if let Some(mut song) = self.song.clone()
                            {
                                song.delete_verse(verse);
                                if let Some(verses) =
                                    self.verses.as_mut()
                                    && let Some(verse) = verses
                                        .iter()
                                        .position(|inner_verse| {
                                            inner_verse.verse_name
                                                == verse
                                        })
                                    {
                                        verses.remove(verse);
                                    }
                                return Action::Task(
                                    self.update_song(song),
                                );
                            }
                        }
                        verse_editor::Action::None => (),
                    }
                }
            }
            Message::FontSizeOpen(open) => {
                self.font_size_open = open;
            }
            Message::FontSelectorOpen(open) => {
                self.font_selector_open = open;
            }
            Message::EditVerseOrder => {
                self.editing_verse_order = !self.editing_verse_order;
            }
            Message::AddVerse((verse, lyric)) => {
                let verse_editor =
                    VerseEditor::new(verse, lyric.clone());
                if let Some(verses) = self.verses.as_mut() {
                    verses.push(verse_editor);
                }
                if let Some(mut song) = self.song.clone() {
                    song.add_verse(verse, lyric);
                    return Action::Task(self.update_song(song));
                }
            }
            Message::RemoveVerse(index) => {
                if let Some(mut song) = self.song.clone() {
                    song.verses.as_mut().map_or_else(
                        || (),
                        |verses| {
                            verses.remove(index);
                        },
                    );
                    return Action::Task(self.update_song(song));
                }
            }
            Message::ChipHovered(index) => {
                self.hovered_verse_chip = index;
            }
            Message::ChipDndHovered(index) => {
                self.hovered_dnd_verse_chip = index;
            }
            Message::ChipDropped((index, data, mime)) => {
                self.hovered_dnd_verse_chip = None;
                match VerseName::try_from((data, mime)) {
                    Ok(verse) => {
                        if let Some(song) = self.song.as_mut() {
                            if let Some(verses) = song.verses.as_mut()
                            {
                                verses.insert(index, verse);
                                let song = song.clone();
                                return Action::Task(
                                    self.update_song(song),
                                );
                            }
                            error!("No verses in this song?");
                        } else {
                            error!("No song here?");
                        }
                    }
                    Err(e) => {
                        error!(?e, "Couldn't convert verse back");
                    }
                }
            }
            Message::ChipDroppedEnd((data, mime)) => {
                self.hovered_dnd_verse_chip = None;
                match VerseName::try_from((data, mime)) {
                    Ok(verse) => {
                        if let Some(song) = self.song.as_mut()
                            && let Some(verses) = song.verses.as_mut()
                        {
                            verses.push(verse);
                            let song = song.clone();
                            return Action::Task(
                                self.update_song(song),
                            );
                        }
                        error!(
                            "No verses in this song or no song here"
                        );
                    }
                    Err(e) => {
                        error!(?e, "Couldn't convert verse back");
                    }
                }
            }
            Message::ChipReorder(event) => match event {
                draggable::DragEvent::Picked { index: _ } => (),
                draggable::DragEvent::Dropped {
                    index,
                    target_index,
                    drop_position: _,
                } => {
                    if let Some(mut song) = self.song.clone()
                        && let Some(verses) = song.verses.as_mut()
                    {
                        let verse = verses.remove(index);
                        verses.insert(target_index, verse);
                        debug!(?verses);
                        return Action::Task(self.update_song(song));
                    }
                }
                draggable::DragEvent::Canceled { index: _ } => (),
            },
            Message::DraggingChipStart => {
                self.dragging_verse_chip = !self.dragging_verse_chip;
            }
            Message::AlignmentPopupOpen => {
                self.alignment_popup = !self.alignment_popup;
            }
            Message::SetTextAlignment(alignment) => {
                if let Some(mut song) = self.song.clone() {
                    song.text_alignment = Some(alignment);
                    return Action::Task(self.update_song(song));
                }
            }
            Message::UpdateShadowSize(size) => {
                if let Some(song) = &mut self.song {
                    if size == 0 {
                        song.shadow_size = None;
                    } else {
                        song.shadow_size = Some(size);
                    }
                    let song = song.to_owned();
                    return Action::Task(self.update_song(song));
                }
            }
            Message::UpdateShadowOffsetX(x) => {
                if let Some(mut song) = self.song.clone() {
                    if let Some((offset_x, _offset_y)) =
                        song.shadow_offset.as_mut()
                    {
                        *offset_x = x;
                        debug!(offset = ?song.shadow_offset);
                    } else {
                        song.shadow_offset = Some((x, 0));
                    }
                    return Action::Task(self.update_song(song));
                }
            }
            Message::UpdateShadowOffsetY(y) => {
                if let Some(mut song) = self.song.clone() {
                    if let Some((_offset_x, offset_y)) =
                        song.shadow_offset.as_mut()
                    {
                        *offset_y = y;
                        debug!(offset = ?song.shadow_offset);
                    } else {
                        song.shadow_offset = Some((0, y));
                    }
                    return Action::Task(self.update_song(song));
                }
            }
            Message::UpdateShadowColor(update) => {
                let mut tasks = Vec::with_capacity(2);
                tasks.push(self.shadow_color_model.update(update));
                if let Some(mut song) = self.song.clone()
                    && let Some(color) =
                        self.shadow_color_model.get_applied_color()
                {
                    debug!(?color);
                    song.shadow_color = Some(color.into());
                    tasks.push(self.update_song(song));
                }
                return Action::Task(Task::batch(tasks));
            }
            Message::OpenShadowTools => {
                self.shadow_tools_open = !self.shadow_tools_open;
            }
            Message::None => (),
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
                    .height(250) // need to find out how to do this differently
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
        let cosmic::cosmic_theme::Spacing {
            space_xxs,
            space_s,
            space_m,
            space_l,
            ..
        } = theme::spacing();

        let title_input = text_input("song", &self.title)
            .on_input(Message::ChangeTitle)
            .label("Song Title");

        let author_input = text_input("author", &self.author)
            .on_input(Message::ChangeAuthor)
            .label("Song Author");

        let top_input_row =
            row![title_input, author_input].spacing(space_m);

        //         let verse_input = text_input(
        //             "Verse
        // order",
        //             &self.verse_order,
        //         )
        //         .label("Verse Order")
        //         .on_input(Message::ChangeVerseOrder);

        let verse_option_chips: Vec<Element<Message>> =
            if let Some(song) = &self.song {
                if let Some(verse_map) = &song.verse_map {
                    verse_map
                        .keys()
                        .sorted()
                        .map(|verse| {
                            let verse = *verse;
                            let chip = verse_chip(verse, None);
                            let verse_chip_wrapped =
                                RcElementWrapper::<Message>::new(
                                    chip,
                                );
                            Element::from(
                            dnd_source::<Message, Box<VerseName>>(
                                verse_chip_wrapped.clone(),
                            )
                            .on_start(Some(
                                Message::DraggingChipStart,
                            ))
                            .on_finish(Some(
                                Message::DraggingChipStart,
                            ))
                            .on_cancel(Some(
                                Message::DraggingChipStart,
                            ))
                            .drag_content(move || Box::new(verse))
                            .drag_icon(
                                move |_| {
                                    let state: tree::State =
                                        cosmic::widget::Widget::<
                                            Message,
                                            _,
                                            _,
                                        >::state(
                                            &verse_chip_wrapped
                                        );
                                    (
                                        Element::from(
                                            verse_chip_wrapped
                                                .clone(),
                                        )
                                        .map(|_| ()),
                                        state,
                                        Vector::new(-5.0, -15.0),
                                    )
                                },
                            ),
                        )
                        })
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

        let verse_options = container(
            scrollable(row(verse_option_chips).spacing(space_s))
                .direction(Direction::Horizontal(
                    Scrollbar::new().spacing(space_s),
                )),
        )
        .padding(space_s)
        .width(Length::Fill)
        .class(theme::Container::Primary);

        let verse_chips_edit_toggle =
            button::icon(if self.editing_verse_order {
                icon::from_name("arrow-up")
            } else {
                icon::from_name("edit")
            })
            .on_press(Message::EditVerseOrder);

        let verse_order_items: Vec<Element<Message>> = if let Some(
            song,
        ) =
            &self.song
        {
            if let Some(verses) = &song.verses {
                verses
                    .iter()
                    .enumerate()
                    .map(|(index, verse)| {
                        let verse = *verse;
                        let hovered_chip = self.hovered_verse_chip.filter(|hovered_index| hovered_index == &index);
                        let mut chip =
                            verse_chip(verse, hovered_chip).apply(mouse_area)
                            .on_enter(Message::ChipHovered(Some(index)))
                            .on_exit(Message::ChipHovered(None))
                            .into();
                        if let Some(hovered_chip) =
                            self.hovered_dnd_verse_chip
                            && index == hovered_chip {
                                let phantom_chip = horizontal_space().width(60).height(19)
                                    .apply(container)
                                    .padding(
                                        Padding::new(space_xxs.into())
                                            .right(space_s)
                                            .left(space_s),
                                    )
                                    .class(theme::Container::Custom(Box::new(move |t| {
                                        container::Style::default()
                                            .background(ContainerBackground::Color(
                                                Color::from(t.cosmic().secondary.base).scale_alpha(0.5)
                                            ))
                                            .border(Border::default().rounded(space_m).width(2))
                                    })));
                                chip = row![
                                    phantom_chip,
                                    chip
                                ]
                                .spacing(space_s)
                                .into();
                            }
                        let verse_chip_wrapped =
                            RcElementWrapper::<Message>::new(chip);
                        Element::from(
                            dnd_destination(
                                verse_chip_wrapped,
                                vec!["application/verse".into()],
                            )
                            .on_enter(move |x, y, mimes| {
                                debug!(x, y, ?mimes);
                                Message::ChipDndHovered(Some(index))
                            })
                            .on_leave(move || {
                                Message::ChipDndHovered(None)
                            })
                            .on_finish(
                                move |mime, data, action, _x, _y| {
                                    debug!(mime, ?data, ?action);
                                    Message::ChipDropped((index, data, mime))
                                },
                            ),
                        )
                    })
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let verse_order_items = if self.dragging_verse_chip {
            Element::from(row(verse_order_items).spacing(space_s))
        } else {
            Element::from(
                draggable::row(verse_order_items)
                    .on_drag(Message::ChipReorder)
                    .spacing(space_s),
            )
        };

        let mut verse_order_row;

        if self.dragging_verse_chip {
            let ending_dnd_dest = dnd_destination(
                horizontal_space().height(19),
                vec!["application/verse".into()],
            )
            .on_enter(|_, _, _| {
                debug!("Entering the space");
                Message::ChipDndHovered(None)
            })
            .on_leave(|| Message::ChipDndHovered(None))
            .on_finish(
                move |mime, data, _action, _x, _y| {
                    Message::ChipDroppedEnd((data, mime))
                },
            );
            verse_order_row = row![
                scrollable(verse_order_items)
                    .direction(
                        Direction::Horizontal(Scrollbar::new())
                    )
                    .spacing(space_s),
                ending_dnd_dest
            ]
            .width(Length::Fill);
        } else {
            verse_order_row = row![
                scrollable(verse_order_items)
                    .direction(
                        Direction::Horizontal(Scrollbar::new())
                    )
                    .width(Length::Fill)
                    .spacing(space_s),
            ]
            .width(Length::Fill);
        }
        verse_order_row =
            verse_order_row.push(verse_chips_edit_toggle);

        let verse_order = container(verse_order_row)
            .padding(space_s)
            .width(Length::Fill)
            .class(theme::Container::Primary);

        let verse_order = container(
            column![
                verse_order,
                if self.editing_verse_order {
                    Element::from(verse_options)
                } else {
                    Element::from(horizontal_space())
                }
            ]
            .spacing(space_s),
        )
        .padding(space_s)
        .class(theme::Container::Card);

        let verse_label = text("Verse Order");

        let verse_order =
            column![verse_label, verse_order].spacing(space_s);

        let lyric_title = text::heading("Lyrics");
        let _lyric_input = column![
            lyric_title,
            text_editor(&self.lyrics)
                .on_action(Message::ChangeLyrics)
                .height(Length::Fill)
        ]
        .spacing(5);

        let verse_list = if let Some(verse_list) = &self.verses {
            Element::from(
                column(verse_list.iter().enumerate().map(
                    |(index, v)| {
                        column![
                            v.view().map(move |message| {
                                Message::VerseEditorMessage((
                                    index, message,
                                ))
                            }),
                            divider::horizontal::heavy()
                        ]
                        .spacing(space_m)
                        .into()
                    },
                ))
                .spacing(space_m),
            )
        } else {
            Element::from(horizontal_space())
        };
        let verse_scroller = scrollable(
            verse_list
                .apply(container)
                .padding(Padding::default().right(space_l)),
        )
        .height(Length::Fill)
        .direction(Direction::Vertical(Scrollbar::new()));

        let verse_add_message = self.song.as_ref().map_or_else(
            || Message::None,
            |song| {
                Message::AddVerse((
                    song.get_next_verse_name(),
                    String::new(),
                ))
            },
        );
        let verse_toolbar = column![
            row![
                text::heading("Verses").width(Length::Fill),
                button::text("Import")
                    .trailing_icon(
                        icon::from_name("browser-download")
                            .symbolic(true)
                    )
                    .on_press(Message::None),
                button::text("Add Verse")
                    .trailing_icon(
                        icon::from_name("add").symbolic(true)
                    )
                    .on_press(verse_add_message)
            ]
            .padding(space_m),
            verse_scroller.height(Length::Fill)
        ]
        .apply(container)
        .padding(space_s)
        .class(theme::Container::Card);

        column![top_input_row, verse_order, verse_toolbar]
            .spacing(space_m)
            .width(Length::Fill)
            .into()
    }

    fn toolbar(&self) -> Element<Message> {
        let cosmic::cosmic_theme::Spacing {
            space_none,
            space_xxs,
            space_s,
            space_m,
            space_l,
            space_xxxl,
            ..
        } = theme::spacing();

        let floating_container_style = |t: &cosmic::Theme| {
            cosmic::widget::container::Style::default()
                .shadow(Shadow {
                    color: Color::BLACK,
                    offset: Vector { x: 0.0, y: 0.0 },
                    blur_radius: 5.0,
                })
                .border(
                    Border::default()
                        .width(1)
                        .color(t.cosmic().primary_container_divider())
                        .rounded(t.cosmic().radius_s()),
                )
                .background(cosmic::iced::Background::Color(
                    t.cosmic().primary_container_color().into(),
                ))
        };

        let selected_font = self
            .song
            .as_ref()
            .and_then(|song| song.font.as_ref());

        let font_selector = tooltip(
            stack![
                combo_box(
                    &self.fonts_combo,
                    "Font",
                    selected_font,
                    Message::ChangeFont,
                )
                .on_open(Message::FontSelectorOpen(true))
                .on_close(Message::FontSelectorOpen(false))
                .width(300),
                container(if self.font_selector_open {
                    Element::from(horizontal_space())
                } else {
                    Element::from(
                        icon::from_name("arrow-down").size(space_m),
                    )
                })
                .padding([
                    space_none, space_xxs, space_none, space_none
                ])
                .height(Length::Fill)
                .align_right(Length::Fill)
                .align_y(Vertical::Center)
            ],
            "Font used in the song",
            tooltip::Position::Bottom,
        )
        .gap(10);

        let selected_font_size = self
            .song
            .as_ref()
            .and_then(|song| song.font_size.map(|size| size.to_string()));

        let font_size = tooltip(
            stack![
                combo_box(
                    &self.font_sizes,
                    "Font Size",
                    selected_font_size.as_ref(),
                    Message::ChangeFontSize,
                )
                .on_input(Message::ChangeFontSize)
                .on_open(Message::FontSizeOpen(true))
                .on_close(Message::FontSizeOpen(false))
                .width(space_xxxl),
                container(if self.font_size_open {
                    Element::from(horizontal_space())
                } else {
                    Element::from(
                        icon::from_name("arrow-down").size(space_m),
                    )
                })
                .padding([
                    space_none, space_xxs, space_none, space_none
                ])
                .height(Length::Fill)
                .align_right(Length::Fill)
                .align_y(Vertical::Center)
            ],
            "Font size",
            tooltip::Position::Bottom,
        )
        .gap(10);

        let bold_button = tooltip(
            button::icon(icon::from_name("format-text-bold"))
                .on_press(Message::None),
            "Bold",
            tooltip::Position::Bottom,
        );
        let italic_button = tooltip(
            button::icon(icon::from_name("format-text-italic"))
                .on_press(Message::None),
            "Italicize",
            tooltip::Position::Bottom,
        );

        let underline_button = tooltip(
            button::icon(icon::from_name("format-text-underline"))
                .on_press(Message::None),
            "Underline",
            tooltip::Position::Bottom,
        );

        let stroke_size_row = row![
            icon(
                icon::from_path("./res/text-outline.svg".into())
                    .symbolic(true)
            ),
            dropdown(
                &[
                    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                    "10", "11", "12", "13", "14", "15"
                ],
                Some(self.stroke_size as usize),
                |i| Message::UpdateStrokeSize(i as u16),
            )
            .gap(5.0),
        ]
        .spacing(3)
        .align_y(Vertical::Center);

        let stroke_size_selector = tooltip(
            stroke_size_row,
            "Outline of the text",
            tooltip::Position::Bottom,
        )
        .gap(10);
        // let stroke_width_selector = combo_box(
        //     &self.stroke_sizes,
        //     "0",
        //     Some(&self.stroke_size),
        //     |v| Message::UpdateStrokeSize(v),
        // )
        // .width(theme::active().cosmic().space_xxl());

        let stroke_color_button = color_picker::color_button(
            Some(Message::OpenStrokeColorPicker),
            self.stroke_color_model.get_applied_color(),
            Length::Fixed(50.0),
        )
        .width(space_l)
        .height(space_l);

        let mut stroke_color_button = popover(stroke_color_button)
            .modal(false)
            .position(popover::Position::Bottom)
            .on_close(Message::OpenStrokeColorPicker);
        if self.stroke_color_picker_open {
            let stroke_color_picker = self
                .stroke_color_model
                .builder(Message::UpdateStrokeColor)
                .height(Length::Fixed(200.0))
                .width(Length::Fixed(200.0))
                .build("Recent Colors", "Copy", "Copied")
                .apply(container)
                .center_y(Length::Fixed(400.0))
                .center_x(Length::Fixed(200.0))
                .class(theme::Container::custom(
                    floating_container_style,
                ));

            stroke_color_button =
                stroke_color_button.popup(stroke_color_picker);
        }

        // let shadow_color_button = color_picker::color_button(
        //     Some(Message::OpenShadowTools),
        //     self.shadow_color_model.get_applied_color(),
        //     Length::Fixed(50.0),
        // )
        // .width(space_l)
        // .height(space_l);

        let shadow_color_picker = self
            .shadow_color_model
            .builder(Message::UpdateShadowColor)
            .height(Length::Fixed(300.0))
            .width(Length::Fixed(400.0))
            .build("Recent Colors", "Copy", "Copied");

        let _shadow_size_spinner = spin_button::vertical(
            "Shadow Size",
            self.song
                .as_ref()
                .and_then(|song| {
                    song.shadow_size.map(|size| size as usize)
                })
                .unwrap_or_default(),
            1,
            0,
            20,
            |i| Message::UpdateShadowSize(i as u16),
        );

        let _shadow_offset_x_spinner = spin_button::vertical(
            "Offset X",
            self.song
                .as_ref()
                .and_then(|song| song.shadow_offset)
                .map(|offset| offset.0)
                .unwrap_or_default(),
            1,
            0,
            50,
            Message::UpdateShadowOffsetX,
        );

        let _shadow_offset_y_spinner = spin_button::vertical(
            "Offset Y",
            self.song
                .as_ref()
                .and_then(|song| song.shadow_offset)
                .map(|offset| offset.1)
                .unwrap_or_default(),
            1,
            0,
            50,
            Message::UpdateShadowOffsetY,
        );

        let shadow_size_dropdown = dropdown(
            &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                "10", "11", "12", "13", "14", "15",
            ],
            self.song
                .as_ref()
                .and_then(|song| {
                    song.shadow_size.map(|size| size as usize)
                }),
            |i| Message::UpdateShadowSize(i as u16),
        )
        .gap(5.0);

        let shadow_offset_x_dropdown = dropdown(
            &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                "10", "11", "12", "13", "14", "15", "16", "17", "18",
                "19", "20",
            ],
            self.song
                .as_ref()
                .and_then(|song| {
                    song.shadow_offset.map(|offset| offset.0 as usize)
                }),
            |i| Message::UpdateShadowOffsetX(i as i16),
        )
        .gap(5.0);

        let shadow_offset_y_dropdown = dropdown(
            &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
                "10", "11", "12", "13", "14", "15", "16", "17", "18",
                "19", "20",
            ],
            self.song
                .as_ref()
                .and_then(|song| {
                    song.shadow_offset.map(|offset| offset.1 as usize)
                }),
            |i| Message::UpdateShadowOffsetY(i as i16),
        )
        .gap(5.0);

        let shadow_size = row!["Size:", shadow_size_dropdown]
            .align_y(Vertical::Center)
            .spacing(space_s);
        let shadow_offset_x =
            row!["Offset X:", shadow_offset_x_dropdown]
                .align_y(Vertical::Center)
                .spacing(space_s);
        let shadow_offset_y =
            row!["Offset Y:", shadow_offset_y_dropdown]
                .align_y(Vertical::Center)
                .spacing(space_s);

        let shadow_tools = column![
            row![shadow_size, shadow_offset_x, shadow_offset_y]
                .padding(space_m)
                .width(Length::Shrink)
                .spacing(space_s)
                .apply(container)
                .center_x(Length::Fill),
            shadow_color_picker
        ]
        .height(Length::Fill)
        .spacing(space_s);

        let mut shadow_tools_button = popover(tooltip(
            button::icon(
                icon::from_path("./res/shadow.svg".into())
                    .symbolic(true),
            )
            .label("Text Shadow")
            .padding(space_s)
            .on_press(Message::OpenShadowTools),
            "Set the shadow of the text",
            tooltip::Position::Bottom,
        ))
        .modal(false)
        .position(popover::Position::Bottom)
        .on_close(Message::OpenShadowTools);

        if self.shadow_tools_open {
            let shadow_tools = shadow_tools
                .apply(container)
                .center_y(Length::Fixed(600.0))
                .center_x(Length::Fixed(400.0))
                .class(theme::Container::custom(
                    floating_container_style,
                ));

            shadow_tools_button =
                shadow_tools_button.popup(shadow_tools);
        }
        let text_alignment_popover = popover(tooltip(
            button::icon(
                icon::from_name("align-on-canvas").symbolic(true),
            )
            .label("Text Alignment")
            .padding(space_s)
            .on_press(Message::AlignmentPopupOpen),
            "Set where text should be on slide",
            tooltip::Position::Bottom,
        ))
        .modal(false)
        .position(popover::Position::Bottom)
        .on_close(Message::AlignmentPopupOpen);

        let text_alignment_popup = if self.alignment_popup {
            text_alignment_popover.popup(
                grid::grid()
                    .row_spacing(space_s)
                    .column_spacing(space_s)
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_top_left",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::TopLeft,
                            ),
                        ),
                        |a| a.column(0).row(0),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_top",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::TopCenter,
                            ),
                        ),
                        |a| a.column(1).row(0),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_top_right",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::TopRight,
                            ),
                        ),
                        |a| a.column(2).row(0),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_left",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::MiddleLeft,
                            ),
                        ),
                        |a| a.column(0).row(1),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_center",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::MiddleCenter,
                            ),
                        ),
                        |a| a.column(1).row(1),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_right",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::MiddleRight,
                            ),
                        ),
                        |a| a.column(2).row(1),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_bottom_left",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::BottomLeft,
                            ),
                        ),
                        |a| a.column(0).row(2),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_bottom",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::BottomCenter,
                            ),
                        ),
                        |a| a.column(1).row(2),
                    )
                    .push_with(
                        button::icon(icon::from_name(
                            "boundingbox_bottom_right",
                        ))
                        .class(theme::Button::Standard)
                        .padding(space_s)
                        .on_press(
                            Message::SetTextAlignment(
                                TextAlignment::BottomRight,
                            ),
                        ),
                        |a| a.column(2).row(2),
                    )
                    .apply(container)
                    .padding(space_s)
                    .class(theme::Container::custom(
                        floating_container_style,
                    )),
            )
        } else {
            text_alignment_popover
        };

        let background_selector = tooltip(
            button::icon(
                icon::from_name("folder-pictures-symbolic").scale(2),
            )
            .label("Background")
            .on_press(Message::PickBackground)
            .padding(space_s),
            "Select an image or video background",
            tooltip::Position::Bottom,
        );

        // let stroke_size_selector = tooltip(
        //     stroke_popup,
        //     "Outline of the text",
        //     tooltip::Position::Bottom,
        // )
        // .gap(10);

        row![
            // text::body("Font:"),
            font_selector,
            // text::body("Font Size:"),
            font_size,
            divider::vertical::default().height(space_l),
            bold_button,
            italic_button,
            underline_button,
            divider::vertical::default().height(space_l),
            stroke_size_selector,
            text::body("Stroke Color:"),
            stroke_color_button,
            shadow_tools_button,
            divider::vertical::default().height(space_l),
            text_alignment_popup,
            horizontal_space(),
            background_selector
        ]
        .align_y(Vertical::Center)
        .spacing(space_s)
        .into()
    }

    pub fn import_view(&self) -> Element<Message> {
        todo!("need to add an import view")
    }

    pub const fn editing(&self) -> bool {
        self.editing
    }

    pub const fn importing(&self) -> bool {
        self.importing
    }

    fn update_song(&mut self, song: Song) -> Task<Message> {
        // use cosmic::iced_futures::futures::stream;
        // use cosmic::iced_futures::futures::{Stream, StreamExt};
        // use cosmic::iced_futures::stream::channel;
        // use cosmic::task::stream;
        let font_db = Arc::clone(&self.font_db);
        // need to test to see which of these methods yields faster
        // text_svg slide creation. There is a small thought in me that
        // believes it's better for the user to see the slides being added
        // one by one, rather than all at once, but that isn't how
        // the task appears to happen.

        // let slides = song.to_slides().ok();
        // let mut task = vec![];
        // if let Some(slides) = slides {
        //     for (index, mut slide) in slides.into_iter().enumerate() {
        //         let font_db = Arc::clone(&font_db);
        //         task.push(Task::perform(
        //             async move {
        //                 text_svg::text_svg_generator(
        //                     &mut slide, font_db,
        //                 );
        //                 (index, slide)
        //             },
        //             Message::UpdateSlide,
        //         ));
        //     }
        // }

        // I think this implementation is faster
        let mut tasks = Vec::with_capacity(2);
        if let Ok(slides) = song.to_slides() {
            if let Some(handle) = &self.update_slide_handle {
                handle.abort();
            }
            let _size = slides.len();

            // let (task, handle) = stream(stream::iter(
            //     slides.into_iter().enumerate().map(
            //         move |(index, mut slide)| {
            //             text_svg::text_svg_generator(
            //                 &mut slide,
            //                 Arc::clone(&font_db),
            //             );
            //             (index, slide)
            //         },
            //     ),
            // ))
            // .then(|(index, slide)| {
            //     Task::done(Message::UpdateSlide((index, slide)))
            // })
            // .abortable();

            let (task, handle) = Task::perform(
                async move {
                    slides
                        .into_par_iter()
                        .map(move |mut s| {
                            text_svg::text_svg_generator(
                                &mut s,
                                Arc::clone(&font_db),
                            );
                            s
                        })
                        .collect::<Vec<Slide>>()
                },
                Message::UpdateSlides,
            )
            .abortable();

            self.update_slide_handle = Some(handle);
            tasks.push(task);
        }
        tasks.push(Task::done(Message::UpdateSong(song)));

        Task::batch(tasks)
    }

    fn background_video(&mut self, background: &Option<Background>) {
        if let Some(background) = background
            && background.kind == BackgroundKind::Video
        {
            let video =
                Video::try_from(background).ok().map(|mut v| {
                    v.set_looping(true);
                    v.set_paused(true);
                    v
                });
            // debug!(?video);
            self.video = video;
        } else {
            self.video = None;
        }
    }
}

fn verse_chip(
    verse: VerseName,
    index: Option<usize>,
) -> Element<'static, Message> {
    let cosmic::cosmic_theme::Spacing {
        space_none,
        space_s,
        space_m,
        space_xxs,
        ..
    } = theme::spacing();

    const VERSE_COLOR: cosmic::iced::Color = color!(0xf26430);
    const CHORUS_COLOR: cosmic::iced::Color = color!(0x3A86ff);
    const BRIDGE_COLOR: cosmic::iced::Color = color!(0x47e5bc);
    const INSTRUMENTAL_COLOR: cosmic::iced::Color = color!(0xd90368);
    const OTHER_COLOR: cosmic::iced::Color = color!(0xffd400);

    let name = verse.get_name();
    let dark_text = Color::BLACK;
    let light_text = Color::WHITE;
    let (background_color, text_color) = match verse {
        VerseName::Verse { .. } => (VERSE_COLOR, light_text),
        VerseName::PreChorus { .. } => {
            (INSTRUMENTAL_COLOR, light_text)
        }
        VerseName::Chorus { .. } => (CHORUS_COLOR, light_text),
        VerseName::PostChorus { .. } => {
            todo!()
        }
        VerseName::Bridge { .. } => (BRIDGE_COLOR, dark_text),
        VerseName::Intro { .. } => (OTHER_COLOR, dark_text),
        VerseName::Outro { .. } => (OTHER_COLOR, dark_text),
        VerseName::Instrumental { .. } => {
            todo!()
        }
        VerseName::Other { .. } => (OTHER_COLOR, dark_text),
        VerseName::Blank => (OTHER_COLOR, dark_text),
    };

    if let Some(index) = index {
        let text = text(name)
            .apply(container)
            .padding(
                Padding::new(space_xxs.into())
                    .right(space_s)
                    .left(space_s),
            )
            .class(theme::Container::Custom(Box::new(move |_t| {
                container::Style::default()
                    .background(ContainerBackground::Color(
                        background_color,
                    ))
                    .color(text_color)
                    .border(
                        Border::default().rounded(space_m).width(2),
                    )
            })));
        let button = button::icon(icon::from_name("view-close"))
            .icon_size(19)
            .padding(space_none)
            .on_press(Message::RemoveVerse(index))
            .class(theme::Button::Destructive);
        stack![
            text,
            button
                .apply(container)
                .padding([0, space_xxs, 0, 0])
                .align_right(Length::Fill)
                .center_y(Length::Fill)
        ]
        .into()
    } else {
        text(name)
            .apply(container)
            .padding(
                Padding::new(space_xxs.into())
                    .right(space_s)
                    .left(space_s),
            )
            .class(theme::Container::Custom(Box::new(move |_t| {
                container::Style::default()
                    .background(ContainerBackground::Color(
                        background_color,
                    ))
                    .color(text_color)
                    .border(
                        Border::default().rounded(space_m).width(2),
                    )
            })))
            .into()
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
