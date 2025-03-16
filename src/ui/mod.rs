use crate::core::model::LibraryKind;

pub mod double_ended_slider;
pub mod library;
pub mod presenter;
pub mod song_editor;
pub mod video;

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
