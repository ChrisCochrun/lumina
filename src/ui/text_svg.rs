use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use colors_transform::Rgb;
use cosmic::{
    iced::{
        font::{Style, Weight},
        Length,
    },
    prelude::*,
    widget::{container, lazy, responsive, svg::Handle, Svg},
};
use tracing::error;

use crate::TextAlignment;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextSvg {
    text: String,
    font: Font,
    shadow: Option<Shadow>,
    stroke: Option<Stroke>,
    fill: Color,
    alignment: TextAlignment,
    handle: Option<Handle>,
}

impl Hash for TextSvg {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        self.font.hash(state);
        self.shadow.hash(state);
        self.stroke.hash(state);
        self.fill.hash(state);
        self.alignment.hash(state);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Font {
    name: String,
    weight: Weight,
    style: Style,
    size: u8,
}

impl From<cosmic::font::Font> for Font {
    fn from(value: cosmic::font::Font) -> Self {
        Self {
            name: match value.family {
                cosmic::iced::font::Family::Name(name) => {
                    name.to_string()
                }
                _ => "Quicksand Bold".into(),
            },
            size: 20,
            ..Default::default()
        }
    }
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
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_weight(&self) -> Weight {
        self.weight
    }

    pub fn get_style(&self) -> Style {
        self.style
    }

    pub fn weight(mut self, weight: impl Into<Weight>) -> Self {
        self.weight = weight.into();
        self
    }

    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Color(Rgb);

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_css_hex_string().hash(state);
    }
}

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

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct Shadow {
    pub offset_x: i16,
    pub offset_y: i16,
    pub spread: u16,
    pub color: Color,
}

#[derive(Clone, Debug, Default, PartialEq, Hash)]
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

    // pub fn build(self)

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

    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.text = text.as_ref().to_string();
        self
    }

    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn view<'a>(self) -> Element<'a, Message> {
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
                "stroke=\"{}\" stroke-width=\"{}px\" paint-order=\"stroke\"",
                stroke.color, stroke.size
            )
        } else {
            "".into()
        };
        container(
            responsive(move |s| {
                let total_lines = self.text.lines().count();
                let half_lines = (total_lines / 2) as f32;
                let middle_position = s.height / 2.0;
                let line_spacing = 10.0;
                let text_and_line_spacing = self.font.size as f32 + line_spacing;
                let starting_y_position = middle_position - (half_lines * text_and_line_spacing);

                let text_pieces: Vec<String> = self.text.lines()
                    .enumerate()
                    .map(|(index, text)| {
                        format!("<tspan x=\"50%\" y=\"{}\">{}</tspan>", starting_y_position + (index as f32 * text_and_line_spacing), text)
                    }).collect();
                let text: String = text_pieces.join("\n");

                let final_svg = format!("<svg viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\"><defs>{}</defs><text x=\"50%\" y=\"50%\" dominant-baseline=\"middle\" text-anchor=\"middle\" font-weight=\"bold\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" {} style=\"filter:url(#shadow);\">{}</text></svg>",
                                        s.width,
                                        s.height,
                                        shadow,
                                        self.font.name,
                                        self.font.size,
                                        self.fill, stroke, text);

                // debug!(final_svg);
Svg::new(Handle::from_memory(
                    Box::leak(<std::string::String as Clone>::clone(&final_svg).into_boxed_str()).as_bytes(),
                ))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            })).width(Length::Fill).height(Length::Fill).into()
    }

    fn text_spans(&self) -> Vec<String> {
        let total_lines = self.text.lines().count();
        self.text
            .lines()
            .enumerate()
            .map(|(i, t)| format!("<tspan x=\"50%\">{}</tspan>", t))
            .collect()
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

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::TextSvg;

    #[test]
    fn test_text_spans() {
        let mut text = TextSvg::new();
        text.text = "This is
multiline
text."
            .into();
        assert_eq!(
            vec![
                String::from("<tspan>This is</tspan>"),
                String::from("<tspan>multiline</tspan>"),
                String::from("<tspan>text.</tspan>"),
            ],
            text.text_spans()
        )
    }
}
