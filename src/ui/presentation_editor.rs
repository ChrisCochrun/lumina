use std::{io, path::Path, path::PathBuf};

use crate::core::presentations::Presentation;
use cosmic::{
    Element, Task,
    dialog::file_chooser::{FileFilter, open::Dialog},
    iced::{Background, ContentFit, Length, alignment::Vertical},
    iced_widget::{column, row},
    theme,
    widget::{
        self, Space, button, container, horizontal_space, icon,
        image::Handle, mouse_area, scrollable, text, text_input,
    },
};
use miette::IntoDiagnostic;
use mupdf::{Colorspace, Document, Matrix};
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct PresentationEditor {
    pub presentation: Option<Presentation>,
    document: Option<Document>,
    current_slide: Option<Handle>,
    slides: Option<Vec<Handle>>,
    page_count: Option<i32>,
    current_slide_index: Option<i32>,
    title: String,
    editing: bool,
    hovered_slide: Option<i32>,
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
    NextPage,
    PrevPage,
    None,
    ChangePresentationFile(Presentation),
    AddSlides(Option<Vec<Handle>>),
    ChangeSlide(usize),
    HoverSlide(Option<i32>),
}

impl PresentationEditor {
    pub fn new() -> Self {
        Self {
            presentation: None,
            document: None,
            title: "".to_string(),
            editing: false,
            current_slide: None,
            current_slide_index: None,
            page_count: None,
            slides: None,
            hovered_slide: None,
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangePresentation(presentation) => {
                self.update_entire_presentation(&presentation);
                if let Some(presentation) = self.presentation.clone()
                {
                    let task = Task::perform(
                        get_all_pages(presentation.path.clone()),
                        |pages| Message::AddSlides(pages),
                    );
                    return Action::Task(task);
                }
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
                warn!(?presentation, "about to update");
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
                            Message::ChangePresentationFile(
                                presentation,
                            )
                        } else {
                            Message::None
                        }
                    },
                );
                return Action::Task(task);
            }
            Message::ChangePresentationFile(presentation) => {
                self.update_entire_presentation(&presentation);
                if let Some(presentation) = self.presentation.clone()
                {
                    let task = Task::perform(
                        get_all_pages(presentation.path.clone()),
                        |pages| Message::AddSlides(pages),
                    )
                    .chain(Task::done(Message::Update(
                        presentation.clone(),
                    )));
                    return Action::Task(task);
                }
            }
            Message::AddSlides(slides) => {
                self.slides = slides;
            }
            Message::None => (),
            Message::NextPage => {
                let next_index =
                    self.current_slide_index.unwrap_or_default() + 1;
                if next_index > self.page_count.unwrap_or_default() {
                    return Action::None;
                }
                self.current_slide =
                    self.document.as_ref().and_then(|doc| {
                        let page = doc.load_page(next_index).ok()?;
                        let matrix = Matrix::IDENTITY;
                        let colorspace = Colorspace::device_rgb();
                        let Ok(pixmap) = page
                            .to_pixmap(
                                &matrix,
                                &colorspace,
                                true,
                                true,
                            )
                            .into_diagnostic()
                        else {
                            error!(
                                "Can't turn this page into pixmap"
                            );
                            return None;
                        };
                        debug!(?pixmap);
                        Some(Handle::from_rgba(
                            pixmap.width(),
                            pixmap.height(),
                            pixmap.samples().to_vec(),
                        ))
                    });
                self.current_slide_index = Some(next_index);
            }
            Message::PrevPage => {
                let previous_index =
                    self.current_slide_index.unwrap_or_default() - 1;
                if previous_index < 0 {
                    return Action::None;
                }
                self.current_slide =
                    self.document.as_ref().and_then(|doc| {
                        let page =
                            doc.load_page(previous_index).ok()?;
                        let matrix = Matrix::IDENTITY;
                        let colorspace = Colorspace::device_rgb();
                        let pixmap = page
                            .to_pixmap(
                                &matrix,
                                &colorspace,
                                true,
                                true,
                            )
                            .ok()?;

                        Some(Handle::from_rgba(
                            pixmap.width(),
                            pixmap.height(),
                            pixmap.samples().to_vec(),
                        ))
                    });
                self.current_slide_index = Some(previous_index);
            }
            Message::ChangeSlide(index) => {
                self.current_slide =
                    self.document.as_ref().and_then(|doc| {
                        let page =
                            doc.load_page(index as i32).ok()?;
                        let matrix = Matrix::IDENTITY;
                        let colorspace = Colorspace::device_rgb();
                        let pixmap = page
                            .to_pixmap(
                                &matrix,
                                &colorspace,
                                true,
                                true,
                            )
                            .ok()?;

                        Some(Handle::from_rgba(
                            pixmap.width(),
                            pixmap.height(),
                            pixmap.samples().to_vec(),
                        ))
                    });
                self.current_slide_index = Some(index as i32);
            }
            Message::HoverSlide(slide) => {
                self.hovered_slide = slide;
            }
        }
        Action::None
    }

    pub fn view(&self) -> Element<Message> {
        let presentation = if let Some(slide) = &self.current_slide {
            container(
                widget::image(slide)
                    .content_fit(ContentFit::ScaleDown),
            )
            .style(|_| {
                container::background(Background::Color(
                    cosmic::iced::Color::WHITE,
                ))
            })
        } else {
            container(Space::new(0, 0))
        };
        let pdf_pages: Vec<Element<Message>> = if let Some(pages) =
            &self.slides
        {
            pages
                .iter()
                .enumerate()
                .map(|(index, page)| {
                    let image = widget::image(page)
                        .height(theme::spacing().space_xxxl * 3)
                        .content_fit(ContentFit::ScaleDown);
                    let slide = container(image).style(|_| {
                        container::background(Background::Color(
                            cosmic::iced::Color::WHITE,
                        ))
                    });
                    container(
                        mouse_area(slide)
                            .on_enter(Message::HoverSlide(Some(
                                index as i32,
                            )))
                            .on_exit(Message::HoverSlide(None))
                            .on_press(Message::ChangeSlide(index)),
                    )
                    .padding(theme::spacing().space_m)
                    .clip(true)
                    .class(
                        if let Some(hovered_index) =
                            self.hovered_slide
                        {
                            if index as i32 == hovered_index {
                                theme::Container::Primary
                            } else {
                                theme::Container::Card
                            }
                        } else {
                            theme::Container::Card
                        },
                    )
                    .into()
                })
                .collect()
        } else {
            vec![horizontal_space().into()]
        };
        let pages_column = container(scrollable(
            column(pdf_pages)
                .spacing(theme::active().cosmic().space_xs())
                .padding(theme::spacing().space_xs),
        ))
        .class(theme::Container::Card);
        let main_row = row![
            pages_column,
            container(presentation).center(Length::FillPortion(2))
        ]
        .spacing(theme::spacing().space_xxl);
        let control_buttons = row![
            button::standard("Previous Page")
                .on_press(Message::PrevPage),
            horizontal_space(),
            button::standard("Next Page").on_press(Message::NextPage),
        ];
        let column =
            column![self.toolbar(), main_row, control_buttons]
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

    fn update_entire_presentation(
        &mut self,
        presentation: &Presentation,
    ) {
        self.presentation = Some(presentation.clone());
        self.title = presentation.title.clone();
        self.document =
            Document::open(&presentation.path.as_path()).ok();
        self.page_count = self
            .document
            .as_ref()
            .and_then(|doc| doc.page_count().ok());
        warn!("changing presentation");
        self.current_slide = self.document.as_ref().and_then(|doc| {
            let page = doc.load_page(0).ok()?;
            let matrix = Matrix::IDENTITY;
            let colorspace = Colorspace::device_rgb();
            let pixmap = page
                .to_pixmap(&matrix, &colorspace, true, true)
                .ok()?;

            Some(Handle::from_rgba(
                pixmap.width(),
                pixmap.height(),
                pixmap.samples().to_vec(),
            ))
        });
        self.current_slide_index = Some(0);
    }
}

impl Default for PresentationEditor {
    fn default() -> Self {
        Self::new()
    }
}

async fn get_all_pages(
    presentation_path: impl AsRef<Path>,
) -> Option<Vec<Handle>> {
    let document = Document::open(presentation_path.as_ref()).ok()?;
    let pages = document.pages().ok()?;
    Some(
        pages
            .filter_map(|page| {
                let page = page.ok()?;
                let matrix = Matrix::IDENTITY;
                let colorspace = Colorspace::device_rgb();
                let pixmap = page
                    .to_pixmap(&matrix, &colorspace, true, true)
                    .ok()?;

                Some(Handle::from_rgba(
                    pixmap.width(),
                    pixmap.height(),
                    pixmap.samples().to_vec(),
                ))
            })
            .collect(),
    )
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
