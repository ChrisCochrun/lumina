use std::{
    collections::HashMap,
    io,
    ops::RangeBounds,
    path::{Path, PathBuf},
};

use crate::core::presentations::{PresKind, Presentation};
use cosmic::{
    Element, Task,
    dialog::file_chooser::{FileFilter, open::Dialog},
    iced::{Background, ContentFit, Length, alignment::Vertical},
    iced_widget::{column, row},
    theme,
    widget::{
        self, Space, button, container, context_menu,
        horizontal_space, icon, image::Handle, menu, mouse_area,
        scrollable, text, text_input,
    },
};
use miette::{IntoDiagnostic, Result, miette};
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
    context_menu_id: Option<i32>,
}

pub enum Action {
    Task(Task<Message>),
    UpdatePresentation(Presentation),
    SplitAddPresentation((Presentation, Presentation)),
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
    ContextMenu(usize),
    SplitBefore,
    SplitAfter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    SplitBefore,
    SplitAfter,
}

impl menu::Action for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            Self::SplitBefore => Message::SplitBefore,
            Self::SplitAfter => Message::SplitAfter,
        }
    }
}

impl PresentationEditor {
    #[must_use] 
    pub const fn new() -> Self {
        Self {
            presentation: None,
            document: None,
            title: String::new(),
            editing: false,
            current_slide: None,
            current_slide_index: None,
            page_count: None,
            slides: None,
            hovered_slide: None,
            context_menu_id: None,
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangePresentation(presentation) => {
                self.update_entire_presentation(&presentation);
                if let Some(presentation) = &self.presentation {
                    let task;
                    if let PresKind::Pdf {
                        starting_index,
                        ending_index,
                    } = presentation.kind.clone()
                    {
                        task = Task::perform(
                            get_pages(
                                starting_index..=ending_index,
                                presentation.path.clone(),
                            ),
                            Message::AddSlides,
                        );
                    } else {
                        task = Task::perform(
                            get_pages(.., presentation.path.clone()),
                            Message::AddSlides,
                        );
                    }
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
                if let Some(presentation) = &self.presentation {
                    let mut task;
                    if let PresKind::Pdf {
                        starting_index,
                        ending_index,
                    } = presentation.kind.clone()
                    {
                        task = Task::perform(
                            get_pages(
                                starting_index..=ending_index,
                                presentation.path.clone(),
                            ),
                            Message::AddSlides,
                        );
                    } else {
                        task = Task::perform(
                            get_pages(.., presentation.path.clone()),
                            Message::AddSlides,
                        );
                    }

                    task = task.chain(Task::done(Message::Update(
                        presentation.clone(),
                    )));
                    return Action::Task(task);
                }
            }
            Message::AddSlides(slides) => {
                debug!(?slides);
                self.slides = slides;
            }
            Message::None => (),
            Message::NextPage => {
                let next_index =
                    self.current_slide_index.unwrap_or_default() + 1;
                let mut last_index =
                    self.page_count.unwrap_or_default();
                if let Some(presentation) = self.presentation.as_ref()
                    && let PresKind::Pdf { ending_index, .. } =
                        presentation.kind
                    {
                        last_index = ending_index;
                    }

                if next_index > last_index {
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
                let mut first_index =
                    self.page_count.unwrap_or_default();
                if let Some(presentation) = self.presentation.as_ref()
                    && let PresKind::Pdf { starting_index, .. } =
                        presentation.kind
                    {
                        first_index = starting_index;
                    }

                if previous_index < first_index {
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
            Message::ContextMenu(index) => {
                self.context_menu_id = Some(index as i32);
            }
            Message::SplitBefore => {
                if let Ok((first, second)) = self.split_before() {
                    self.update_entire_presentation(&first);
                    return Action::SplitAddPresentation((
                        first, second,
                    ));
                }
            }
            Message::SplitAfter => {
                if let Ok((first, second)) = self.split_after() {
                    self.update_entire_presentation(&first);
                    return Action::SplitAddPresentation((
                        first, second,
                    ));
                }
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
                    let clickable_slide = container(
                        mouse_area(slide)
                            .on_enter(Message::HoverSlide(Some(
                                index as i32,
                            )))
                            .on_exit(Message::HoverSlide(None))
                            .on_right_press(Message::ContextMenu(
                                index,
                            ))
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
                    );
                    clickable_slide.into()
                })
                .collect()
        } else {
            vec![horizontal_space().into()]
        };
        let pages_column = container(
            self.context_menu(
                scrollable(
                    column(pdf_pages)
                        .spacing(theme::active().cosmic().space_xs())
                        .padding(theme::spacing().space_xs),
                )
                .into(),
            ),
        )
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

    fn context_menu<'b>(
        &self,
        items: Element<'b, Message>,
    ) -> Element<'b, Message> {
        if self.context_menu_id.is_some() {
            let before_icon =
                icon::from_path("./res/split-above.svg".into())
                    .symbolic(true);
            let after_icon =
                icon::from_path("./res/split-below.svg".into())
                    .symbolic(true);
            let menu_items = vec![
                menu::Item::Button(
                    "Spit Before",
                    Some(before_icon),
                    MenuAction::SplitBefore,
                ),
                menu::Item::Button(
                    "Split After",
                    Some(after_icon),
                    MenuAction::SplitAfter,
                ),
            ];
            let context_menu = context_menu(
                items,
                self.context_menu_id.map_or_else(
                    || None,
                    |_| {
                        Some(menu::items(&HashMap::new(), menu_items))
                    },
                ),
            );
            Element::from(context_menu)
        } else {
            items
        }
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

    fn split_before(&self) -> Result<(Presentation, Presentation)> {
        if let Some(index) = self.context_menu_id {
            let Some(current_presentation) =
                self.presentation.as_ref()
            else {
                return Err(miette!(
                    "There is no current presentation"
                ));
            };
            let first_presentation = Presentation {
                id: current_presentation.id,
                title: current_presentation.title.clone(),
                path: current_presentation.path.clone(),
                kind: match current_presentation.kind {
                    PresKind::Pdf { .. } => PresKind::Pdf {
                        starting_index: 0,
                        ending_index: index - 1,
                    },
                    _ => current_presentation.kind.clone(),
                },
            };
            let second_presentation = Presentation {
                id: 0,
                title: current_presentation.title.clone(),
                path: current_presentation.path.clone(),
                kind: match current_presentation.kind {
                    PresKind::Pdf { ending_index, .. } => {
                        PresKind::Pdf {
                            starting_index: index,
                            ending_index,
                        }
                    }
                    _ => current_presentation.kind.clone(),
                },
            };
            Ok((first_presentation, second_presentation))
        } else {
            error!("split before no index");
            Err(miette!(
                "No current index from context menu, has there been a right click on a presentation page"
            ))
        }
    }

    fn split_after(&self) -> Result<(Presentation, Presentation)> {
        if let Some(index) = self.context_menu_id {
            let Some(current_presentation) =
                self.presentation.as_ref()
            else {
                return Err(miette!(
                    "There is no current presentation"
                ));
            };
            let first_presentation = Presentation {
                id: current_presentation.id,
                title: current_presentation.title.clone(),
                path: current_presentation.path.clone(),
                kind: match current_presentation.kind {
                    PresKind::Pdf { .. } => PresKind::Pdf {
                        starting_index: 0,
                        ending_index: index,
                    },
                    _ => current_presentation.kind.clone(),
                },
            };
            let second_presentation = Presentation {
                id: 0,
                title: current_presentation.title.clone(),
                path: current_presentation.path.clone(),
                kind: match current_presentation.kind {
                    PresKind::Pdf { ending_index, .. } => {
                        PresKind::Pdf {
                            starting_index: index + 1,
                            ending_index,
                        }
                    }
                    _ => current_presentation.kind.clone(),
                },
            };
            Ok((first_presentation, second_presentation))
        } else {
            error!("split before no index");
            Err(miette!(
                "No current index from context menu, has there been a right click on a presentation page"
            ))
        }
    }
}

impl Default for PresentationEditor {
    fn default() -> Self {
        Self::new()
    }
}

async fn get_pages(
    range: impl RangeBounds<i32>,
    presentation_path: impl AsRef<Path>,
) -> Option<Vec<Handle>> {
    let document = Document::open(presentation_path.as_ref()).ok()?;
    let pages = document.pages().ok()?;
    Some(
        pages
            .enumerate()
            .filter_map(|(index, page)| {
                if !range.contains(&(index as i32)) {
                    return None;
                }
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
