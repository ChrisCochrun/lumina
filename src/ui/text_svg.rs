use cosmic::iced::Font as IcedFont;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextSvg {
    text: String,
    font: Font,
    shadow: Shadow,
    stroke: Stroke,
    fill: Color,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Font(IcedFont);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Shadow {
    offset_x: i16,
    offset_y: i16,
    spread: u16,
    color: Color,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stroke {
    size: u16,
    color: Color,
}

impl TextSvg {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn fill(mut self, color: impl Into<Color>) -> Self {
        self.fill = color.into();
        self
    }

    pub fn shadow(mut self, shadow: impl Into<Shadow>) -> Self {
        self.shadow = shadow.into();
        self
    }

    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = font.into();
        self
    }
}
