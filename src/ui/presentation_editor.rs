use std::{io, path::PathBuf};

use crate::core::{
    presentations::Presentation, service_items::ServiceTrait,
    slide::Slide,
};
use cosmic::{
    dialog::file_chooser::{open::Dialog, FileFilter},
    iced::{alignment::Vertical, ContentFit, Length},
    iced_widget::{column, row},
    theme,
    widget::{
        button, container, horizontal_space, icon, image, text,
        text_input, Space,
    },
    Element, Task,
};
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct PresentationEditor {
    pub presentation: Option<Presentation>,
    slides: Option<Vec<Slide>>,
    title: String,
    editing: bool,
    current_slide: i16,
}

pub enum Action {
    Task(Task<Message>),
    UpdatePresentation(Presentation),
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangePresentation(Presentation),
    Update(Presentation),
    ChangeTitle(String),
    PickPresentation,
    Edit(bool),
    None,
}

impl PresentationEditor {
    pub fn new() -> Self {
        Self {
            presentation: None,
            slides: None,
            title: "Death was Arrested".to_string(),
            editing: false,
            current_slide: 0,
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangePresentation(presentation) => {
                self.presentation = Some(presentation.clone());
                self.title = presentation.title.clone();
                warn!("changing presentation");
                let Ok(slides) = presentation.to_slides() else {
                    return Action::None;
                };
                self.slides = Some(slides);
                self.current_slide = 0;
                return self.update(Message::Update(presentation));
            }
            Message::ChangeTitle(title) => {
                self.title = title.clone();
                if let Some(presentation) = &self.presentation {
                    let mut presentation = presentation.clone();
                    presentation.title = title;
                    return self
                        .update(Message::Update(presentation));
                }
            }
            Message::Edit(edit) => {
                debug!(edit);
                self.editing = edit;
            }
            Message::Update(presentation) => {
                warn!(?presentation);
                return Action::UpdatePresentation(presentation);
            }
            Message::PickPresentation => {
                let presentation_id = self
                    .presentation
                    .as_ref()
                    .map(|v| v.id)
                    .unwrap_or_default();
                let task = Task::perform(
                    pick_presentation(),
                    move |presentation_result| {
                        if let Ok(presentation) = presentation_result
                        {
                            let mut presentation =
                                Presentation::from(presentation);
                            presentation.id = presentation_id;
                            Message::ChangePresentation(presentation)
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
        let container = if let Some(slides) = &self.slides {
            if let Some(slide) =
                slides.get(self.current_slide as usize)
            {
                container(
                    image(slide.pdf_page().unwrap_or(
                        image::Handle::from_path("res/chad.png"),
                    ))
                    .content_fit(ContentFit::Cover),
                )
            } else {
                container(Space::new(0, 0))
            }
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

        let presentation_selector = button::icon(
            icon::from_name("folder-presentations-symbolic").scale(2),
        )
        .label("Presentation")
        .tooltip("Select a presentation")
        .on_press(Message::PickPresentation)
        .padding(10);

        row![
            text::body("Title:"),
            title_box,
            horizontal_space(),
            presentation_selector
        ]
        .align_y(Vertical::Center)
        .spacing(10)
        .into()
    }

    pub const fn editing(&self) -> bool {
        self.editing
    }
}

impl Default for PresentationEditor {
    fn default() -> Self {
        Self::new()
    }
}

async fn pick_presentation() -> Result<PathBuf, PresentationError> {
    let dialog = Dialog::new().title("Choose a presentation...");
    let bg_filter = FileFilter::new("Presentations")
        .extension("pdf")
        .extension("html");
    dialog
        .filter(bg_filter)
        .directory(dirs::home_dir().expect("oops"))
        .open_file()
        .await
        .map_err(|e| {
            error!(?e);
            PresentationError::DialogClosed
        })
        .map(|file| file.url().to_file_path().unwrap())
    // rfd::AsyncFileDialog::new()
    //     .set_title("Choose a background...")
    //     .add_filter(
    //         "Presentations and Presentations",
    //         &["png", "jpeg", "mp4", "webm", "mkv", "jpg", "mpeg"],
    //     )
    //     .set_directory(dirs::home_dir().unwrap())
    //     .pick_file()
    //     .await
    //     .ok_or(PresentationError::BackgroundDialogClosed)
    //     .map(|file| file.path().to_owned())
}

#[derive(Debug, Clone)]
pub enum PresentationError {
    DialogClosed,
    IOError(io::ErrorKind),
}
