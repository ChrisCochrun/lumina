use std::any::Any;

use cosmic::iced::{self, Size};
use cosmic::iced_core::window;

use cosmic::{
    iced::{
        clipboard::dnd::{DndAction, DndEvent, SourceEvent},
        event, mouse, overlay, Event, Length, Point, Rectangle,
        Vector,
    },
    iced_core::{
        self, layout, renderer,
        widget::{tree, Tree},
        Clipboard, Shell,
    },
    widget::{container, Id, Widget},
    Element,
};

use crate::core::service_items::ServiceItem;

pub fn service<'a, Message: Clone + 'static>(
    service: &'a Vec<ServiceItem>,
) -> Service<'a, Message> {
    Service::new(service)
}

pub struct Service<'a, Message> {
    service: &'a Vec<ServiceItem>,
    on_start: Option<Message>,
    on_cancelled: Option<Message>,
    on_finish: Option<Message>,
    drag_threshold: f32,
    width: Length,
    height: Length,
}

impl<'a, Message: Clone + 'static> Service<'a, Message> {
    pub fn new(service: &'a Vec<ServiceItem>) -> Self {
        Self {
            service,
            drag_threshold: 8.0,
            on_start: None,
            on_cancelled: None,
            on_finish: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    #[must_use]
    pub fn drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold;
        self
    }

    // pub fn start_dnd(
    //     &self,
    //     clipboard: &mut dyn Clipboard,
    //     bounds: Rectangle,
    //     offset: Vector,
    // ) {
    //     let Some(content) = self.drag_content.as_ref().map(|f| f())
    //     else {
    //         return;
    //     };

    //     iced_core::clipboard::start_dnd(
    //         clipboard,
    //         false,
    //         if let Some(window) = self.window.as_ref() {
    //             Some(iced_core::clipboard::DndSource::Surface(
    //                 *window,
    //             ))
    //         } else {
    //             Some(iced_core::clipboard::DndSource::Widget(
    //                 self.id.clone(),
    //             ))
    //         },
    //         self.drag_icon.as_ref().map(|f| {
    //             let (icon, state, offset) = f(offset);
    //             iced_core::clipboard::IconSurface::new(
    //                 container(icon)
    //                     .width(Length::Fixed(bounds.width))
    //                     .height(Length::Fixed(bounds.height))
    //                     .into(),
    //                 state,
    //                 offset,
    //             )
    //         }),
    //         Box::new(content),
    //         DndAction::Move,
    //     );
    // }

    pub fn on_start(mut self, on_start: Option<Message>) -> Self {
        self.on_start = on_start;
        self
    }

    pub fn on_cancel(
        mut self,
        on_cancelled: Option<Message>,
    ) -> Self {
        self.on_cancelled = on_cancelled;
        self
    }

    pub fn on_finish(mut self, on_finish: Option<Message>) -> Self {
        self.on_finish = on_finish;
        self
    }
}

impl<Message: Clone + 'static>
    Widget<Message, cosmic::Theme, cosmic::Renderer>
    for Service<'_, Message>
{
    fn size(&self) -> iced_core::Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &cosmic::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    // fn operate(
    //     &self,
    //     tree: &mut Tree,
    //     layout: layout::Layout<'_>,
    //     renderer: &cosmic::Renderer,
    //     operation: &mut dyn iced_core::widget::Operation<()>,
    // ) {
    //     operation.custom(
    //         (&mut tree.state) as &mut dyn Any,
    //         Some(&self.id),
    //     );
    //     operation.container(
    //         Some(&self.id),
    //         layout.bounds(),
    //         &mut |operation| {
    //             self.container.as_widget().operate(
    //                 &mut tree.children[0],
    //                 layout,
    //                 renderer,
    //                 operation,
    //             )
    //         },
    //     );
    // }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &cosmic::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(position) = cursor.position() {
                        if !state.hovered {
                            return event::Status::Ignored;
                        }

                        state.left_pressed_position = Some(position);
                        return event::Status::Captured;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left)
                    if state.left_pressed_position.is_some() =>
                {
                    state.left_pressed_position = None;
                    return event::Status::Captured;
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(position) = cursor.position() {
                        if state.hovered {
                            // We ignore motion if we do not possess drag content by now.
                            if let Some(left_pressed_position) =
                                state.left_pressed_position
                            {
                                if position
                                    .distance(left_pressed_position)
                                    > self.drag_threshold
                                {
                                    if let Some(on_start) =
                                        self.on_start.as_ref()
                                    {
                                        shell
                                            .publish(on_start.clone())
                                    }
                                    let offset = Vector::new(
                                        left_pressed_position.x
                                            - layout.bounds().x,
                                        left_pressed_position.y
                                            - layout.bounds().y,
                                    );
                                    state.is_dragging = true;
                                    state.left_pressed_position =
                                        None;
                                }
                            }
                            if !cursor.is_over(layout.bounds()) {
                                state.hovered = false;

                                return event::Status::Ignored;
                            }
                        } else if cursor.is_over(layout.bounds()) {
                            state.hovered = true;
                        }
                        return event::Status::Captured;
                    }
                }
                _ => return event::Status::Ignored,
            },
            Event::Dnd(DndEvent::Source(SourceEvent::Cancelled)) => {
                if state.is_dragging {
                    if let Some(m) = self.on_cancelled.as_ref() {
                        shell.publish(m.clone());
                    }
                    state.is_dragging = false;
                    return event::Status::Captured;
                }
                return event::Status::Ignored;
            }
            Event::Dnd(DndEvent::Source(SourceEvent::Finished)) => {
                if state.is_dragging {
                    if let Some(m) = self.on_finish.as_ref() {
                        shell.publish(m.clone());
                    }
                    state.is_dragging = false;
                    return event::Status::Captured;
                }
                return event::Status::Ignored;
            }
            _ => return event::Status::Ignored,
        }
        event::Status::Ignored
    }

    // fn mouse_interaction(
    //     &self,
    //     tree: &Tree,
    //     layout: layout::Layout<'_>,
    //     cursor_position: mouse::Cursor,
    //     viewport: &Rectangle,
    //     renderer: &cosmic::Renderer,
    // ) -> mouse::Interaction {
    //     let state = tree.state.downcast_ref::<State>();
    //     if state.is_dragging {
    //         return mouse::Interaction::Grabbing;
    //     }
    //     self.container.as_widget().mouse_interaction(
    //         &tree.children[0],
    //         layout,
    //         cursor_position,
    //         viewport,
    //         renderer,
    //     )
    // }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut cosmic::Renderer,
        theme: &cosmic::Theme,
        renderer_style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        for item in self.service {
            todo!()
        }
    }

    // fn overlay<'b>(
    //     &'b mut self,
    //     tree: &'b mut Tree,
    //     layout: layout::Layout<'_>,
    //     renderer: &cosmic::Renderer,
    //     translation: Vector,
    // ) -> Option<
    //     overlay::Element<
    //         'b,
    //         Message,
    //         cosmic::Theme,
    //         cosmic::Renderer,
    //     >,
    // > {
    //     self.container.as_widget_mut().overlay(
    //         &mut tree.children[0],
    //         layout,
    //         renderer,
    //         translation,
    //     )
    // }

    // #[cfg(feature = "a11y")]
    // /// get the a11y nodes for the widget
    // fn a11y_nodes(
    //     &self,
    //     layout: iced_core::Layout<'_>,
    //     state: &Tree,
    //     p: mouse::Cursor,
    // ) -> iced_accessibility::A11yTree {
    //     let c_state = &state.children[0];
    //     self.container.as_widget().a11y_nodes(layout, c_state, p)
    // }
}

impl<'a, Message: Clone + 'static> From<Service<'a, Message>>
    for Element<'a, Message>
{
    fn from(e: Service<'a, Message>) -> Element<'a, Message> {
        Element::new(e)
    }
}

/// Local state of the [`MouseListener`].
#[derive(Debug, Default)]
struct State {
    hovered: bool,
    left_pressed_position: Option<Point>,
    is_dragging: bool,
    cached_bounds: Rectangle,
}

impl State {
    fn new() -> Self {
        Self::default()
    }
}
