use cosmic::{
    Element, Task,
    iced_widget::column,
    theme,
    widget::{container, text, text_editor},
};

use crate::core::songs::VerseName;

#[derive(Debug)]
pub struct VerseEditor {
    verse_name: VerseName,
    lyric: String,
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeText(text_editor::Action),
    None,
}

pub enum Action {
    Task(Task<Message>),
    UpdateVerse((VerseName, String)),
    None,
}

impl VerseEditor {
    #[must_use]
    pub fn new(verse: VerseName, lyric: String) -> Self {
        Self {
            verse_name: verse,
            lyric: lyric.clone(),
            content: text_editor::Content::with_text(&lyric),
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeText(action) => {
                self.content.perform(action);
                let lyrics = self.content.text();
                self.lyric = lyrics.clone();
                let verse = self.verse_name;
                Action::UpdateVerse((verse, lyrics))
            }
            Message::None => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let cosmic::cosmic_theme::Spacing {
            space_s, space_m, ..
        } = theme::spacing();

        let verse_title =
            text::heading(self.verse_name.get_name()).size(space_m);
        let lyric = text_editor(&self.content)
            .on_action(Message::ChangeText)
            .padding(space_s)
            // .style(|theme, status| {
            //     let mut style =
            //         text_editor::default(theme, status);
            //     style.border = Border::default().rounded(space_s);
            //     style
            // })
            .height(150);

        container(column![verse_title, lyric].spacing(space_s))
            .padding(space_s)
            .class(theme::Container::Card)
            .into()
    }
}
