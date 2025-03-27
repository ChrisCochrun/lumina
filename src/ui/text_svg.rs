use std::fmt::Display;

use colors_transform::Rgb;
use cosmic::{
    iced::font::{Style, Weight},
    prelude::*,
    widget::{svg::Handle, Svg},
};
use tracing::error;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextSvg {
    text: String,
    font: Font,
    shadow: Option<Shadow>,
    stroke: Option<Stroke>,
    fill: Color,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Font {
    name: String,
    weight: Weight,
    style: Style,
    size: u8,
}

impl From<&str> for Font {
    fn from(value: &str) -> Self {
        Self {
            name: value.to_owned(),
            ..Default::default()
        }
    }
}

impl Font {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_weight(&self) -> Weight {
        self.weight.clone()
    }

    fn get_style(&self) -> Style {
        self.style.clone()
    }

    fn weight(mut self, weight: impl Into<Weight>) -> Self {
        self.weight = weight.into();
        self
    }

    fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Color(Rgb);

impl Color {
    pub fn from_hex_str(color: impl AsRef<str>) -> Color {
        match Rgb::from_hex_str(color.as_ref()) {
            Ok(rgb) => Color(rgb),
            Err(e) => {
                error!("error in making color from hex_str: {:?}", e);
                Color::default()
            }
        }
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        Self::from_hex_str(value)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(
            Rgb::from_hex_str("#000")
                .expect("This is not a hex color"),
        )
    }
}

impl Display for Color {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.0.to_css_hex_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shadow {
    pub offset_x: i16,
    pub offset_y: i16,
    pub spread: u16,
    pub color: Color,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stroke {
    size: u16,
    color: Color,
}

pub enum Message {
    None,
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
        self.shadow = Some(shadow.into());
        self
    }

    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }

    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.font = font.into();
        self
    }

    pub fn view(&self) -> Element<Message> {
        let shadow = if let Some(shadow) = &self.shadow {
            format!("<filter id=\"shadow\"><feDropShadow dx=\"{}\" dy=\"{}\" stdDeviation=\"{}\" flood-color=\"{}\"/></filter>",
                shadow.offset_x,
                shadow.offset_y,
                shadow.spread,
                shadow.color)
        } else {
            "".into()
        };
        let stroke = if let Some(stroke) = &self.stroke {
            format!(
                "stroke=\"{}\" stroke-width=\"{}\"",
                stroke.color, stroke.size
            )
        } else {
            "".into()
        };
        // text y coords are based on bottom left corner so we need to take height into consideration
        let base = format!("<svg viewBox=\"0 0 240 100\" xmlns=\"http://www.w3.org/2000/svg\"><defs>{}</defs>
<text x=\"0\" y=\"50\" font-weight=\"bold\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" {} style=\"filter:url(#shadow);\">
    {}
</text></svg>", shadow, self.font.name, self.font.size, self.fill, stroke, self.text);
        Svg::new(Handle::from_memory(
            Box::leak(base.into_boxed_str()).as_bytes(),
        ))
        .into()
    }
}

pub fn shadow(
    offset_x: i16,
    offset_y: i16,
    spread: u16,
    color: impl Into<Color>,
) -> Shadow {
    Shadow {
        offset_x,
        offset_y,
        spread,
        color: color.into(),
    }
}

pub fn stroke(size: u16, color: impl Into<Color>) -> Stroke {
    Stroke {
        size,
        color: color.into(),
    }
}

pub fn color(color: impl AsRef<str>) -> Color {
    Color::from_hex_str(color)
}
