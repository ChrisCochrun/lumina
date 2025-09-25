use crate::core::model::LibraryKind;

pub mod double_ended_slider;
pub mod image_editor;
pub mod library;
pub mod presenter;
pub mod service;
pub mod slide_editor;
pub mod song_editor;
pub mod text_svg;
pub mod video;
pub mod video_editor;
pub mod widgets;

pub enum EditorMode {
    Song,
    Image,
    Video,
    Presentation,
    Slide,
}

impl From<LibraryKind> for EditorMode {
    fn from(value: LibraryKind) -> Self {
        match value {
            LibraryKind::Song => Self::Song,
            LibraryKind::Video => Self::Video,
            LibraryKind::Image => Self::Image,
            LibraryKind::Presentation => Self::Presentation,
        }
    }
}
