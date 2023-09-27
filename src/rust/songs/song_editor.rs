mod song_editor {
    pub enum Yes {
        Yes,
        No,
    }

    pub struct EditableSong {
        title: String,
        lyrics: String,
        author: String,
        ccli: String,
        audio: String,
        verse_order: String,
        background: String,
        background_type: String,
        horizontal_text_alignment: String,
        vertical_text_alignment: String,
        font: String,
        font_size: i32,
    }
}
