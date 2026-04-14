use std::collections::HashMap;

use cosmic::widget::popover::Position;
use cosmic::widget::{
    Column, button, column, container, menu, mouse_area, popover,
};
use cosmic::{Element, theme};

#[derive(Default, Clone)]
pub struct Context<Msg> {
    menu: Vec<Entry<Msg>>,
    modal: bool,
    hovered_entry: Option<String>,
    hovered_point: Option<cosmic::iced::Point>,
    context_point: Option<cosmic::iced::Point>,
    popovers: Option<Vec<String>>,
    open: bool,
}

#[derive(Default, Clone, Debug)]
pub enum Entry<Msg> {
    Node((String, Msg)),
    Tree((String, Vec<Entry<Msg>>)),
    #[default]
    Empty,
}

#[derive(Clone, Debug)]
pub enum Message<Msg> {
    HoverTree((String, cosmic::iced::Point)),
    Activate(Msg),
    ClosePopover(String),
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
            menu: Vec::with_capacity(5),
            modal: false,
            hovered_entry: None,
            hovered_point: None,
            context_point: None,
            popovers: None,
            open: false,
        }
    }

    pub fn with_entries(entries: impl Into<Vec<Entry<Msg>>>) -> Self {
        Self {
            menu: entries.into(),
            modal: false,
            hovered_entry: None,
            hovered_point: None,
            context_point: None,
            popovers: None,
            open: false,
        }
    }

    pub fn view<'a>(
        &self,
        content: impl Into<Element<'a, Message<Msg>>>,
    ) -> Element<'a, Message<Msg>>
    where
        Msg: Clone + 'static,
    {
        let mut popover = popover(content)
            .position(Position::Point(
                self.context_point.unwrap_or_default(),
            ))
            .on_close(Message::ClosePopover(format!("root")));
        if self.open {
            popover
                .popup(self.widget_trees(0, self.menu.clone()))
                .into()
        } else {
            popover.into()
        }
    }

    pub fn update(&mut self, message: Message<Msg>) -> Action<Msg> {
        match message {
            Message::HoverTree((name, point)) => {
                self.hovered_entry = Some(name);
                self.hovered_point = Some(point);
            }
            Message::Activate(msg) => return Action::RunMsg(msg),
            Message::ClosePopover(index) => {}
        }
        Action::None
    }

    fn widget_trees<'a>(
        &self,
        level: usize,
        entries: Vec<Entry<Msg>>,
    ) -> Column<'a, Message<Msg>, cosmic::Theme>
    where
        Msg: Clone + 'static,
    {
        let mut column = column(vec![]);
        for entry in entries {
            match entry {
                Entry::Node((name, message)) => {
                    let button = button::text(name)
                        .on_press(Message::Activate(message))
                        .class(theme::Button::MenuItem);
                    column = column.push(button);
                }
                Entry::Tree((name, entries)) => {
                    let key = format!("{level}_{name}");

                    let container =
                        container(cosmic::widget::text(name))
                            .class(theme::Container::List);
                    let mouse_key = key.clone();
                    let mouse_area =
                        mouse_area(container).on_move(move |point| {
                            Message::<Msg>::HoverTree((
                                mouse_key.to_owned(),
                                point,
                            ))
                        });

                    let mut popover = popover(mouse_area)
                        .on_close(Message::ClosePopover(key.clone()))
                        .position(Position::Point(
                            self.hovered_point.unwrap_or(
                                cosmic::iced::Point::ORIGIN,
                            ),
                        ));
                    if let Some(name) = &self.hovered_entry
                        && name == &key
                    {
                        popover = popover.popup(
                            self.widget_trees(level + 1, entries),
                        );
                    }
                    column = column.push(popover);
                }
                Entry::Empty => (),
            }
        }
        column.width(240)
    }
}

fn get_map<Msg>(
    level: usize,
    entries: impl AsRef<[Entry<Msg>]>,
) -> HashMap<String, Entry<Msg>>
where
    Msg: Clone,
{
    let mut map: HashMap<String, Entry<Msg>> =
        HashMap::with_capacity(entries.as_ref().len());
    for entry in entries.as_ref().to_vec().into_iter() {
        match entry {
            Entry::Node((ref name, _)) => {
                map.insert(format!("{level}_{name}"), entry.clone());
            }
            Entry::Tree((_, nested)) => {
                let nested_map = get_map(level + 1, nested);
                map.extend(nested_map);
            }
            Entry::Empty => (),
        }
    }
    map
}
