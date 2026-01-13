use cosmic::{
    Apply, Element, Task,
    iced::{Length, alignment::Vertical},
    iced_widget::{column, row},
    theme,
    widget::{container, icon, text, text_editor},
};

use crate::core::songs::Verse;

#[derive(Debug)]
pub struct VerseEditor {
    verse: Verse,
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeText(text_editor::Action),
    None,
}

pub enum Action {
    Task(Task<Message>),
    UpdateVerse(Verse),
    None,
}

impl VerseEditor {
    pub fn new(verse: Verse) -> Self {
        let text = verse.get_lyric();
        Self {
            verse,
            content: text_editor::Content::with_text(&text),
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ChangeText(action) => {
                self.content.perform(action);
                let lyrics = self.content.text();
                self.verse.set_lyrics(lyrics);
                let verse = self.verse.clone();
                Action::UpdateVerse(verse)
            }
            Message::None => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let cosmic::cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            space_s,
            space_m,
            space_l,
            space_xl,
            space_xxl,
            ..
        } = theme::spacing();

        let verse_title =
            text::heading(self.verse.get_name()).size(space_m);
        let editor = text_editor(&self.content)
            .on_action(Message::ChangeText)
            .padding(space_s)
            .height(Length::Fill);

        let drag_handle = icon::from_name("object-rows")
            .prefer_svg(true)
            .size(space_xxl);

        let row = row![drag_handle, editor]
            .spacing(space_s)
            .align_y(Vertical::Center);
        container(column![verse_title, row].spacing(space_s))
            .padding(space_s)
            .class(theme::Container::Card)
            .into()
    }
}
