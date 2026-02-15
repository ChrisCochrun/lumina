use std::{io, path::PathBuf};

use cosmic::{
    Element, Task,
    dialog::file_chooser::{FileFilter, open::Dialog},
    iced::{Length, alignment::Vertical},
    iced_widget::{column, row},
    theme,
    widget::{
        Space, button, container, horizontal_space, icon,
        progress_bar, text, text_input,
    },
};
use iced_video_player::{Video, VideoPlayer};
use tracing::{debug, error, warn};
use url::Url;

use crate::core::videos;

#[derive(Debug)]
pub struct VideoEditor {
    pub video: Option<Video>,
    core_video: Option<videos::Video>,
    title: String,
    editing: bool,
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
}

impl VideoEditor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            video: None,
            core_video: None,
            title: "Death was Arrested".to_string(),
            editing: false,
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
            Message::PickVideo => {
                let video_id = self
                    .core_video
                    .as_ref()
                    .map(|v| v.id)
                    .unwrap_or_default();
                let task = Task::perform(
                    pick_video(),
                    move |video_result| {
                        if let Ok(video) = video_result {
                            let mut video =
                                videos::Video::from(video);
                            video.id = video_id;
                            Message::UpdateVideoFile(video)
                        } else {
                            Message::None
                        }
                    },
                );
                return Action::Task(task);
            }
            Message::UpdateVideoFile(video) => {
                self.update_entire_video(&video);
                return Action::UpdateVideo(video);
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

        let video_player = self.video.as_ref().map_or_else(
            || Element::from(Space::new(0, 0)),
            |video| Element::from(VideoPlayer::new(video)),
        );

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
        let title_box = text_input("Title...", &self.title)
            .on_input(Message::ChangeTitle);

        let video_selector = button::icon(
            icon::from_name("folder-videos-symbolic").scale(2),
        )
        .label("Video")
        .tooltip("Select a video")
        .on_press(Message::PickVideo)
        .padding(10);

        row![
            text::body("Title:"),
            title_box,
            horizontal_space(),
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
        let Ok(mut player_video) =
            Url::from_file_path(video.path.clone())
                .map(|url| Video::new(&url).expect("Should be here"))
        else {
            self.video = None;
            self.title.clone_from(&video.title);
            self.core_video = Some(video.clone());
            return;
        };
        player_video.set_paused(true);
        self.video = Some(player_video);
        self.title.clone_from(&video.title);
        self.core_video = Some(video.clone());
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
        .map(|file| {
            file.url().to_file_path().expect("Should be a file here")
        })
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
