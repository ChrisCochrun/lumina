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
