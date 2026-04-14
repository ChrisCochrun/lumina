use cosmic::widget::{Column, button, column, container, mouse_area};
use cosmic::{Element, theme};

#[derive(Default, Clone, Debug)]
pub struct Context<Msg> {
    entries: Vec<Entry<Msg>>,
    modal: bool,
}

#[derive(Default, Clone, Debug)]
pub enum Entry<Msg> {
    Node((String, Msg)),
    Tree((String, Vec<Entry<Msg>>)),
    #[default]
    Empty,
}

pub enum Message {
    HoverTree,
}

pub enum Action<Msg> {
    RunMsg(Msg),
    None,
}

impl<Msg> Context<Msg>
where
    Msg: Clone,
{
    pub fn new() -> Self {
        Self {
            entries: vec![],
            modal: false,
        }
    }

    pub fn with_entries(entries: impl Into<Vec<Entry<Msg>>>) -> Self {
        Self {
            entries: entries.into(),
            modal: false,
        }
    }

    pub fn view(
        content: impl Into<Element<Message>>,
    ) -> Element<Message> {
        todo!()
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::HoverTree => todo!(),
        }
        Action::None
    }
}

fn widget_trees(entries: Vec<Entry<Msg>>) -> Column<Message> {
    let mut column = column(vec![]);
    for entry in entries {
        match entry {
            Entry::Node((name, message)) => {
                let button = button::text(name)
                    .on_press(message)
                    .class(theme::Button::MenuItem);
                column.push(button);
            }
            Entry::Tree((name, entries)) => {
                let container =
                    container(name).class(theme::Container::List);
                let mouse_area =
                    mouse_area(container).on_enter(Msg::HoverTree);
            }
            Entry::Empty => (),
        }
    }
    column
}
