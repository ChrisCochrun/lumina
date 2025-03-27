use cosmic::iced::Font;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextSvg {
    text: String,
    font: Font,
    shadow: Shadow,
    stroke: Color,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shadow {
    offset_x: i16,
    offset_y: i16,
    spread: u16,
    color: Color,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stroke {
    size: u16,
    color: Color,
}
