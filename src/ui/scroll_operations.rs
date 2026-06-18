use cosmic::{
    Task,
    iced::{
        Rectangle, Vector,
        advanced::widget::operate,
        core::{
            self,
            widget::{
                self,
                operation::{Outcome, search_id::search_id},
            },
        },
        daemon::program::graphics::core::widget::operation::Scrollable,
        widget::{
            operation::AbsoluteOffset,
            scrollable::{self, scroll_to},
        },
    },
    widget::{Operation, container},
};

use crate::ui::presenter::Message;

pub(crate) fn focus_target(
    scrollable_id: core::id::Id,
    target_item_id: Option<core::id::Id>,
    padding: f32,
) -> Task<Message> {
    let Some(target_item_id) = target_item_id else {
        return scroll_to(scrollable_id.clone(), AbsoluteOffset::default());
    };

    struct CalculateScrollToIdOffset {
        scrollable: widget::Id,
        target: widget::Id,
        viewport_rectangle: Option<Rectangle>,
        viewport_translation: Option<Vector>,
        target_rectangle: Option<Rectangle>,
        padding: f32,
    }

    impl Operation<AbsoluteOffset> for CalculateScrollToIdOffset {
        fn container(&mut self, id: Option<&widget::Id>, bounds: Rectangle) {
            if Some(&self.target) == id {
                self.target_rectangle = Some(bounds)
            }
        }

        fn scrollable(
            &mut self,
            id: Option<&widget::Id>,
            bounds: Rectangle,
            _content_bounds: Rectangle,
            translation: Vector,
            _state: &mut dyn Scrollable,
        ) {
            if Some(&self.scrollable) == id {
                self.viewport_rectangle = Some(bounds);
                self.viewport_translation = Some(translation);
            }
        }

        fn finish(&self) -> Outcome<AbsoluteOffset> {
            let Some(target_rectangle) = self.target_rectangle else {
                return Outcome::None;
            };

            let Some(viewport_rectangle) = self.viewport_rectangle else {
                return Outcome::None;
            };

            let Some(viewport_translation) = self.viewport_translation else {
                return Outcome::None;
            };

            let r_x = target_rectangle.x;
            let r_y = target_rectangle.y;
            let r_w = target_rectangle.width;
            let r_h = target_rectangle.height;
            let t_x = viewport_translation.x;
            let t_y = viewport_translation.y;
            let v_w = viewport_rectangle.width;
            let v_h = viewport_rectangle.height;
            let v_x = viewport_rectangle.x;
            let v_y = viewport_rectangle.y;

            let pad = self.padding;

            let offset_x = t_x.max(r_x + r_w - (v_x + v_w) + pad).min(r_x - v_x - pad);
            let offset_y = t_y.max(r_y + r_h - (v_y + v_h) + pad).min(r_y - v_y - pad);

            let offset = AbsoluteOffset {
                x: offset_x,
                y: offset_y,
            };

            Outcome::Some(offset)
        }

        fn traverse(
            &mut self,
            operate: &mut dyn FnMut(&mut dyn Operation<AbsoluteOffset>),
        ) {
            operate(self);
        }
    }

    let operation = CalculateScrollToIdOffset {
        scrollable: scrollable_id.clone().into(),
        target: target_item_id.clone().into(),
        viewport_rectangle: None,
        viewport_translation: None,
        target_rectangle: None,
        padding,
    };

    let scrollable_id = scrollable_id.clone();
    operate(operation).then(move |offset| scroll_to(scrollable_id.clone(), offset.into()))
}
