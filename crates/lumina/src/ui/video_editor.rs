use std::borrow::Cow;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use cosmic::cosmic_theme::palette::WithAlpha;
use cosmic::dialog::file_chooser::FileFilter;
use cosmic::dialog::file_chooser::open::Dialog;
use cosmic::iced::alignment::Vertical;
use cosmic::iced::widget::{column, row};
use cosmic::iced::{Background, Border, Length, Padding};
use cosmic::prelude::*;
use cosmic::widget::space::{self, horizontal};
use cosmic::widget::{
    Space, button, container, icon, slider, spin_button, text, text_input,
};
use cosmic::{Element, Task, theme};
use iced_video_player::{Position, Video, VideoPlayer};
use tracing::{debug, error, warn};
use url::Url;

use crate::core::videos;
use crate::ui::gst_video;
use crate::ui::widgets::video_timeline::{TimelineEvent, VideoTimeline};

#[derive(Debug)]
pub struct VideoEditor {
    pub video: Option<Video>,
    core_video: Option<videos::Video>,
    title: String,
    editing: bool,
    position: f64,
    last_seek: Instant,
    timestamp_text: String,
}

pub enum Action {
    Task(Task<Message>),
    UpdateVideo(videos::Video),
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeVideo(videos::Video),
    Update(videos::Video),
    ChangeTitle(String),
    PickVideo,
    Edit(bool),
    None,
    PauseVideo,
    UpdateVideoFile(videos::Video),
    VideoPos(f32),
    SetStart(f32),
    SetEnd(f32),
    Volume(f64),
    NewFrame,
}

impl VideoEditor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            video: None,
            core_video: None,
            title: "Death was Arrested".to_string(),
            editing: false,
            position: 0.0,
            last_seek: Instant::now(),
            timestamp_text: String::new(),
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeVideo(video) => {
                self.update_entire_video(&video);
            }
            Message::ChangeTitle(title) => {
                self.title.clone_from(&title);
                if let Some(video) = &self.core_video {
                    let mut video = video.clone();
                    video.title = title;
                    return self.update(Message::Update(video));
                }
            }
            Message::Edit(edit) => {
                debug!(edit);
                self.editing = edit;
            }
            Message::PauseVideo => {
                if let Some(video) = &mut self.video {
                    let paused = video.paused();
                    video.set_paused(!paused);
                }
            }
            Message::Update(video) => {
                warn!(?video);
                return Action::UpdateVideo(video);
            }
            Message::VideoPos(position) => {
                if let Some(video) = self.video.as_mut()
                    && self
                        .last_seek
                        .checked_add(Duration::from_millis(50))
                        .is_some_and(|time| Instant::now() > time)
                {
                    let pausing = video.paused();
                    video.set_paused(true);
                    let position =
                        Position::Time(std::time::Duration::from_secs_f32(position));
                    match video.seek(position, false) {
                        Ok(_) => {
                            self.last_seek = Instant::now();
                            self.position = video.position().as_secs_f64();
                        }
                        Err(e) => {
                            error!(?e);
                        }
                    }
                    video.set_paused(pausing);
                }
                self.update_timestamp();
            }
            Message::PickVideo => {
                let video_id = self.core_video.as_ref().map(|v| v.id).unwrap_or_default();
                let task = Task::perform(pick_video(), move |video_result| {
                    video_result.map_or(Message::None, |video| {
                        let mut video = videos::Video::from(video);
                        video.id = video_id;
                        Message::UpdateVideoFile(video)
                    })
                });
                return Action::Task(task);
            }
            Message::UpdateVideoFile(video) => {
                self.update_entire_video(&video);
                return Action::UpdateVideo(video);
            }
            Message::NewFrame => {
                if let Some(video) = &self.video
                    && self.position >= 0.0
                    && video.position().as_secs_f64() != 0.0
                {
                    self.position = video.position().as_secs_f64();
                }
                self.update_timestamp();
                if self.core_video.as_ref().is_some_and(|core_video| {
                    core_video
                        .end_time
                        .is_some_and(|time| (time - self.position as f32).abs() <= 0.03)
                }) {
                    return self.update(Message::PauseVideo);
                }
            }
            Message::SetStart(time) => {
                if let Some(video) = &mut self.core_video {
                    let time = if time > 0.0 { time } else { 0.0 };
                    video.start_time = Some(time);
                    return Action::UpdateVideo(video.clone());
                }
            }
            Message::SetEnd(time) => {
                if let Some(video) = &mut self.core_video {
                    video.end_time = Some(time);
                    return Action::UpdateVideo(video.clone());
                }
            }
            Message::Volume(volume) => {
                if let Some(video) = &mut self.video {
                    video.set_volume(volume);
                }
            }
            Message::None => (),
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let video_elements = self.video.as_ref().map_or_else(
            || container(horizontal()),
            |video| {
                let play_button = button::icon(if video.paused() {
                    icon::from_name("media-playback-start-symbolic")
                } else {
                    icon::from_name("media-playback-pause-symbolic")
                })
                .icon_size(cosmic::theme::spacing().space_xxl)
                .on_press(Message::PauseVideo)
                .tooltip(if self.video.as_ref().is_some_and(|video| video.paused()) {
                    "Play"
                } else {
                    "Pause"
                })
                .padding(
                    Padding::ZERO
                        .vertical(cosmic::theme::spacing().space_m)
                        .horizontal(cosmic::theme::spacing().space_xxl),
                )
                .apply(container)
                .center_x(Length::Fill);
                let position_text = text(&self.timestamp_text);
                let trim_controls: Element<Message> = if let Some(core_video) =
                    &self.core_video
                {
                    let start_time = core_video.start_time.unwrap_or(0.0);
                    let start_label = format_time(start_time as f64);

                    let start_spin = spinner(start_label, start_time, Message::SetStart);
                    let end_time = core_video
                        .end_time
                        .unwrap_or(video.duration().as_secs_f32());
                    let end_label = format_time(end_time as f64);
                    let end_spin = spinner(end_label, end_time as f32, Message::SetEnd);
                    row![start_spin, end_spin]
                        .spacing(cosmic::theme::spacing().space_l)
                        .apply(container)
                        .padding(Padding::ZERO.right(cosmic::theme::spacing().space_m))
                        .align_right(Length::Fill)
                        .into()
                } else {
                    space::vertical().into()
                };
                let volume = slider(0.0..=1.0, video.volume(), Message::Volume)
                    .step(0.01)
                    .width(Length::Fill);
                let left_controls = row![position_text, volume]
                    .spacing(cosmic::theme::spacing().space_m)
                    .apply(container)
                    .padding(Padding::ZERO.left(cosmic::theme::spacing().space_m))
                    .align_left(Length::Fill);
                let controls_row = row![left_controls, play_button, trim_controls]
                    .align_y(Vertical::Center)
                    .spacing(cosmic::theme::spacing().space_m);
                let video_timeline = VideoTimeline::new(video)
                    .start(
                        self.core_video
                            .as_ref()
                            .map_or(0.0, |video| video.start_time.unwrap_or_default()),
                    )
                    .end(self.core_video.as_ref().map_or_else(
                        || video.duration().as_secs_f32(),
                        |core_video| {
                            core_video
                                .end_time
                                .unwrap_or_else(|| video.duration().as_secs_f32())
                        },
                    ))
                    .on_event(|event| match event {
                        TimelineEvent::TrimStart(trimmed_start) => {
                            Message::SetStart(trimmed_start)
                        }
                        TimelineEvent::TrimEnd(trimmed_end) => {
                            Message::SetEnd(trimmed_end)
                        }
                        TimelineEvent::Seek(pos) => Message::VideoPos(pos),
                        TimelineEvent::Released(_handle) => todo!(),
                    });
                let column = column![video_timeline, controls_row];
                column.apply(container)
            },
        );

        let video_player = self
            .video
            .as_ref()
            .map_or_else(
                || Space::new().apply(container),
                |video| {
                    VideoPlayer::new(video)
                        .on_new_frame(Message::NewFrame)
                        .apply(container)
                },
            )
            .center(Length::Fill);

        let video_section = column![video_player, video_elements]
            .spacing(cosmic::theme::spacing().space_s);
        let column = column![
            self.toolbar(),
            container(video_section).center_x(Length::FillPortion(2))
        ]
        .spacing(theme::active().cosmic().space_l());
        column.into()
    }

    fn toolbar(&self) -> Element<Message> {
        let title_box =
            text_input("Title...", &self.title).on_input(Message::ChangeTitle);

        let video_selector =
            button::icon(icon::from_name("folder-videos-symbolic").scale(2))
                .label("Video")
                .tooltip("Select a video")
                .on_press(Message::PickVideo)
                .padding(10);

        row![
            text::body("Title:"),
            title_box,
            space::horizontal(),
            video_selector
        ]
        .align_y(Vertical::Center)
        .spacing(10)
        .into()
    }

    pub const fn editing(&self) -> bool {
        self.editing
    }

    fn update_entire_video(&mut self, video: &videos::Video) {
        debug!(?video);
        let Ok(url) = Url::from_file_path(video.path.clone()) else {
            self.video = None;
            self.title.clone_from(&video.title);
            self.core_video = Some(video.clone());
            return;
        };

        let settings = gst_video::VideoSettings {
            mute: false,
            framerate: 60,
            appsink_name: "lumina_video".to_string(),
            preview: false,
        };

        let Ok(mut player_video) = gst_video::create_video(&url, &settings) else {
            self.video = None;
            self.title = format!("{}: {}", String::from("Video Missing"), &video.title);
            self.core_video = Some(video.clone());
            return;
        };

        player_video.set_paused(true);
        let start = video.start_time.unwrap_or(0.0).apply(|time| {
            if time < 0.0 {
                Duration::ZERO
            } else {
                Duration::from_secs_f32(time)
            }
        });
        if let Err(e) = player_video.seek(start, false) {
            error!(?e);
        }
        self.video = Some(player_video);
        self.title.clone_from(&video.title);
        self.core_video = Some(video.clone());
        self.position = 0.0;
        self.update_timestamp();
    }

    fn update_timestamp(&mut self) {
        if let Some(video) = self.video.as_ref() {
            let position_seconds = self.position;
            let duration_seconds = video.duration().as_secs_f64();
            let formatted = format!(
                "{} / {}",
                format_time(position_seconds),
                format_time(duration_seconds)
            );
            self.timestamp_text = formatted;
        }
    }
}
fn spinner<'a>(
    label: String,
    value: f32,
    on_press: impl Fn(f32) -> Message,
) -> Element<'a, Message> {
    const UP_ICON: &[u8] = include_bytes!("../../../../res/icons/caret-up.svg");
    const DOWN_ICON: &[u8] = include_bytes!("../../../../res/icons/caret-down.svg");
    let up_button = button::custom(
        icon::from_svg_bytes(UP_ICON)
            .symbolic(true)
            .icon()
            .class(theme::Svg::custom(|t| cosmic::widget::svg::Style {
                color: Some(if t.cosmic().is_dark {
                    t.cosmic().palette.neutral_10.into()
                } else {
                    t.cosmic().palette.neutral_0.into()
                }),
            }))
            .size(15),
    )
    .on_press((on_press)(value + 1.0))
    .padding(0);
    let down_button = button::custom(
        icon::from_svg_bytes(DOWN_ICON)
            .symbolic(true)
            .icon()
            .class(theme::Svg::custom(|t| cosmic::widget::svg::Style {
                color: Some(if t.cosmic().is_dark {
                    t.cosmic().palette.neutral_10.into()
                } else {
                    t.cosmic().palette.neutral_0.into()
                }),
            }))
            .size(15),
    )
    .on_press((on_press)(value - 1.0))
    .padding(0);
    let buttons = column![up_button, down_button];
    let text = text(label).size(18);
    row![text, buttons]
        .align_y(Vertical::Center)
        .spacing(cosmic::theme::spacing().space_xxl)
        .apply(container)
        .padding(
            Padding::ZERO
                .horizontal(cosmic::theme::spacing().space_s)
                .vertical(cosmic::theme::spacing().space_s),
        )
        .style(|theme| {
            container::Style::default()
                .background(Background::Color(
                    theme
                        .cosmic()
                        .background(false)
                        .component
                        .base
                        .with_alpha(0.25)
                        .into(),
                ))
                .border(
                    Border::default()
                        .rounded(theme.cosmic().radius_s())
                        .width(3)
                        .color(theme.cosmic().primary_component_divider()),
                )
        })
        .into()
}

fn format_time(seconds: f64) -> String {
    let total = if seconds.is_finite() && seconds > 0.0 {
        seconds.floor() as u64
    } else {
        0
    };
    let (h, m, s) = (total / 3600, (total / 60) % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

impl Default for VideoEditor {
    fn default() -> Self {
        Self::new()
    }
}

async fn pick_video() -> Result<PathBuf, VideoError> {
    let dialog = Dialog::new().title("Choose a video...");
    let bg_filter = FileFilter::new("Videos")
        .extension("mp4")
        .extension("webm")
        .extension("mkv");
    dialog
        .filter(bg_filter)
        .directory(dirs::home_dir().expect("oops"))
        .open_file()
        .await
        .map_err(|e| {
            error!(?e);
            VideoError::DialogClosed
        })
        .map(|file| file.url().to_file_path().expect("Should be a file here"))
    // rfd::AsyncFileDialog::new()
    //     .set_title("Choose a background...")
    //     .add_filter(
    //         "Images and Videos",
    //         &["png", "jpeg", "mp4", "webm", "mkv", "jpg", "mpeg"],
    //     )
    //     .set_directory(dirs::home_dir().unwrap())
    //     .pick_file()
    //     .await
    //     .ok_or(VideoError::BackgroundDialogClosed)
    //     .map(|file| file.path().to_owned())
}

#[derive(Debug, Clone)]
pub enum VideoError {
    DialogClosed,
    IOError(io::ErrorKind),
}
