use std::ops::RangeInclusive;

use iced::Length;

struct DoubleSlider<'a, T, Message> {
    range: RangeInclusive<T>,
    step: T,
    beginning_value: T,
    end_value: T,
    default_beginning: Option<T>,
    default_end: Option<T>,
    on_change: Box<dyn Fn(T) -> Message + 'a>,
    width: Length,
    height: i32,
}
