use std::{io, path::PathBuf};

use crate::core::images::Image;
use cosmic::{
    dialog::file_chooser::{open::Dialog, FileFilter},
    iced::{alignment::Vertical, Length},
    iced_widget::{column, row},
    theme,
    widget::{
        self, button, container, horizontal_space, icon, text,
        text_input, Space,
    },
    Element, Task,
};
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct ImageEditor {
    pub image: Option<Image>,
    title: String,
    editing: bool,
}

pub enum Action {
    Task(Task<Message>),
    UpdateImage(Image),
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeImage(Image),
    Update(Image),
    ChangeTitle(String),
    PickImage,
    Edit(bool),
    None,
}

impl ImageEditor {
    pub fn new() -> Self {
        Self {
            image: None,
            title: "Death was Arrested".to_string(),
            editing: false,
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeImage(image) => {
                self.image = Some(image.clone());
                self.title = image.title.clone();
                return self.update(Message::Update(image));
            }
            Message::ChangeTitle(title) => {
                self.title = title.clone();
                if let Some(image) = &self.image {
                    let mut image = image.clone();
                    image.title = title;
                    return self.update(Message::Update(image));
                }
            }
            Message::Edit(edit) => {
                debug!(edit);
                self.editing = edit;
            }
            Message::Update(image) => {
                warn!(?image);
                return Action::UpdateImage(image);
            }
            Message::PickImage => {
                let image_id = self
                    .image
                    .as_ref()
                    .map(|v| v.id)
                    .unwrap_or_default();
                let task = Task::perform(
                    pick_image(),
                    move |image_result| {
                        if let Ok(image) = image_result {
                            let mut image = Image::from(image);
                            image.id = image_id;
                            Message::ChangeImage(image)
                        } else {
                            Message::None
                        }
                    },
                );
                return Action::Task(task);
            }
            Message::None => (),
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let container = if let Some(pic) = &self.image {
            let image = widget::image(pic.path.clone());
            container(image)
        } else {
            container(Space::new(0, 0))
        };
        let column = column![
            self.toolbar(),
            container.center_x(Length::FillPortion(2))
        ]
        .spacing(theme::active().cosmic().space_l());
        column.into()
    }

    fn toolbar(&self) -> Element<Message> {
        let title_box = text_input("Title...", &self.title)
            .on_input(Message::ChangeTitle);

        let image_selector = button::icon(
            icon::from_name("folder-images-symbolic").scale(2),
        )
        .label("Image")
        .tooltip("Select a image")
        .on_press(Message::PickImage)
        .padding(10);

        row![
            text::body("Title:"),
            title_box,
            horizontal_space(),
            image_selector
        ]
        .align_y(Vertical::Center)
        .spacing(10)
        .into()
    }

    pub const fn editing(&self) -> bool {
        self.editing
    }
}

impl Default for ImageEditor {
    fn default() -> Self {
        Self::new()
    }
}

async fn pick_image() -> Result<PathBuf, ImageError> {
    let dialog = Dialog::new().title("Choose a image...");
    let bg_filter = FileFilter::new("Images")
        .extension("png")
        .extension("jpeg")
        .extension("gif")
        .extension("heic")
        .extension("webp")
        .extension("jpg");
    dialog
        .filter(bg_filter)
        .directory(dirs::home_dir().expect("oops"))
        .open_file()
        .await
        .map_err(|e| {
            error!(?e);
            ImageError::DialogClosed
        })
        .map(|file| file.url().to_file_path().unwrap())
    // rfd::AsyncFileDialog::new()
    //     .set_title("Choose a background...")
    //     .add_filter(
    //         "Images and Images",
    //         &["png", "jpeg", "mp4", "webm", "mkv", "jpg", "mpeg"],
    //     )
    //     .set_directory(dirs::home_dir().unwrap())
    //     .pick_file()
    //     .await
    //     .ok_or(ImageError::BackgroundDialogClosed)
    //     .map(|file| file.path().to_owned())
}

#[derive(Debug, Clone)]
pub enum ImageError {
    DialogClosed,
    IOError(io::ErrorKind),
}
