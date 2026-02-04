use cosmic::{
    Element, Task,
    cosmic_theme::palette::WithAlpha,
    iced::{Background, Border, Color},
    iced_widget::{column, row},
    theme,
    widget::{
        button, combo_box, container, horizontal_space, icon, text,
        text_editor, text_input,
    },
};

use crate::core::songs::VerseName;

#[derive(Debug)]
pub struct VerseEditor {
    pub verse_name: VerseName,
    pub lyric: String,
    content: text_editor::Content,
    editing_verse_name: bool,
    verse_name_combo: combo_box::State<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateLyric(text_editor::Action),
    UpdateVerseName(String),
    EditVerseName,
    None,
}

pub enum Action {
    Task(Task<Message>),
    UpdateVerse((VerseName, String)),
    UpdateVerseName(String),
    None,
}

impl VerseEditor {
    #[must_use]
    pub fn new(verse: VerseName, lyric: String) -> Self {
        Self {
            verse_name: verse,
            lyric: lyric.clone(),
            content: text_editor::Content::with_text(&lyric),
            editing_verse_name: false,
            verse_name_combo: combo_box::State::new(
                VerseName::all_names(),
            ),
        }
    }
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::UpdateLyric(action) => {
                self.content.perform(action);
                let lyrics = self.content.text();
                self.lyric = lyrics.clone();
                let verse = self.verse_name;
                Action::UpdateVerse((verse, lyrics))
            }
            Message::UpdateVerseName(verse_name) => {
                Action::UpdateVerseName(verse_name)
            }
            Message::EditVerseName => {
                self.editing_verse_name = !self.editing_verse_name;
                Action::None
            }
            Message::None => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let cosmic::cosmic_theme::Spacing {
            space_xxs,
            space_s,
            space_m,
            ..
        } = theme::spacing();

        let delete_button = button::text("Delete")
            .trailing_icon(icon::from_name("view-close"))
            .class(theme::Button::Destructive)
            .on_press(Message::None);
        let combo = combo_box(
            &self.verse_name_combo,
            "Verse 1",
            Some(&self.verse_name.get_name()),
            Message::UpdateVerseName,
        );

        let verse_title =
            row![combo, horizontal_space(), delete_button];

        let lyric = text_editor(&self.content)
            .on_action(Message::UpdateLyric)
            .padding(space_m)
            .class(theme::iced::TextEditor::Custom(Box::new(
                move |t, s| {
                    let neutral = t.cosmic().palette.neutral_9;
                    let mut base_style = text_editor::Style {
                        background: Background::Color(
                            t.cosmic()
                                .background
                                .small_widget
                                .with_alpha(0.25)
                                .into(),
                        ),
                        border: Border::default()
                            .rounded(space_s)
                            .width(2)
                            .color(t.cosmic().bg_component_divider()),
                        icon: t
                            .cosmic()
                            .primary_component_color()
                            .into(),
                        placeholder: neutral.with_alpha(0.7).into(),
                        value: neutral.into(),
                        selection: t.cosmic().accent.base.into(),
                    };
                    let hovered_border = Border::default()
                        .rounded(space_s)
                        .width(3)
                        .color(t.cosmic().accent.hover);
                    match s {
                        text_editor::Status::Active => base_style,
                        text_editor::Status::Hovered => {
                            base_style.border = hovered_border;
                            base_style
                        }
                        text_editor::Status::Focused => {
                            base_style.border = hovered_border;
                            base_style
                        }
                        text_editor::Status::Disabled => base_style,
                    }
                },
            )))
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
