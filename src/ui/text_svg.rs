use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::Arc,
};

use cosmic::{
    cosmic_theme::palette::{IntoColor, Srgb, rgb::Rgba},
    iced::{
        ContentFit, Length, Size,
        font::{Style, Weight},
    },
    prelude::*,
    widget::{Image, image::Handle},
};
use miette::{IntoDiagnostic, Result};
use rapidhash::v3::rapidhash_v3;
use resvg::{
    tiny_skia::{self, Pixmap},
    usvg::{Tree, fontdb},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::TextAlignment;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TextSvg {
    text: String,
    font: Font,
    shadow: Option<Shadow>,
    stroke: Option<Stroke>,
    fill: Color,
    alignment: TextAlignment,
    #[serde(skip)]
    pub handle: Option<Handle>,
    #[serde(skip)]
    fontdb: Arc<resvg::usvg::fontdb::Database>,
}

impl PartialEq for TextSvg {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.font == other.font
            && self.shadow == other.shadow
            && self.stroke == other.stroke
            && self.fill == other.fill
            && self.alignment == other.alignment
            && self.handle == other.handle
    }
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

#[derive(
    Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct Font {
    name: String,
    weight: Weight,
    style: Style,
    size: u8,
}

#[derive(
    Clone, Debug, Default, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Shadow {
    pub offset_x: i16,
    pub offset_y: i16,
    pub spread: u16,
    pub color: Color,
}

#[derive(
    Clone, Debug, Default, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Stroke {
    size: u16,
    color: Color,
}

pub enum Message {
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color(Srgb);

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

impl From<String> for Font {
    fn from(value: String) -> Self {
        Self {
            name: value,
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
    #[must_use]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    #[must_use]
    pub const fn get_weight(&self) -> Weight {
        self.weight
    }

    #[must_use]
    pub const fn get_style(&self) -> Style {
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

    #[must_use]
    pub const fn size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_css_hex_string().hash(state);
    }
}

impl Color {
    pub fn to_css_hex_string(&self) -> String {
        format!("#{:x}", self.0.into_format::<u8>())
    }

    pub fn from_hex_str(color: impl AsRef<str>) -> Self {
        let color = color.as_ref();
        let color: Result<Srgb<u8>> = color.parse().into_diagnostic();
        match color {
            Ok(srgb) => Self(srgb.into()),
            Err(e) => {
                error!("error in making color from hex_str: {:?}", e);
                Self::default()
            }
        }
    }
}

impl From<Rgba> for Color {
    fn from(value: Rgba) -> Self {
        let rgba: Srgb = value.into_color();
        Self(rgba)
    }
}

impl From<Srgb> for Color {
    fn from(value: Srgb) -> Self {
        Self(value)
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        Self::from_hex_str(value)
    }
}

impl From<String> for Color {
    fn from(value: String) -> Self {
        Self::from_hex_str(value)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self(Srgb::new(0.0, 0.0, 0.0))
    }
}

impl Display for Color {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.to_css_hex_string())
    }
}

impl TextSvg {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
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

    pub fn fontdb(mut self, fontdb: Arc<fontdb::Database>) -> Self {
        self.fontdb = fontdb;
        self
    }

    pub const fn alignment(
        mut self,
        alignment: TextAlignment,
    ) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn build(mut self, size: Size, cache: bool) -> Self {
        // debug!("starting...");

        let mut path = dirs::data_local_dir().unwrap();
        path.push(PathBuf::from("lumina"));
        path.push(PathBuf::from("temp"));

        let mut final_svg = String::with_capacity(1024);

        let font_scale = size.height / 1080.0;
        let font_size = f32::from(self.font.size) * font_scale;
        let total_lines = self.text.lines().count();
        let half_lines = (total_lines / 2) as f32;
        let line_spacing = 10.0;
        let text_and_line_spacing = font_size + line_spacing;

        let center_y = (size.width / 2.0).to_string();
        let x_width_padded = (size.width - 10.0).to_string();

        let (text_anchor, starting_y_position, text_x_position) =
            match self.alignment {
                TextAlignment::TopLeft => ("start", font_size, "10"),
                TextAlignment::TopCenter => {
                    ("middle", font_size, center_y.as_str())
                }
                TextAlignment::TopRight => {
                    ("end", font_size, x_width_padded.as_str())
                }
                TextAlignment::MiddleLeft => {
                    let middle_position = size.height / 2.0;
                    let position = half_lines.mul_add(
                        -text_and_line_spacing,
                        middle_position,
                    );
                    ("start", position, "10")
                }
                TextAlignment::MiddleCenter => {
                    let middle_position = size.height / 2.0;
                    let position = half_lines.mul_add(
                        -text_and_line_spacing,
                        middle_position,
                    );
                    ("middle", position, center_y.as_str())
                }
                TextAlignment::MiddleRight => {
                    let middle_position = size.height / 2.0;
                    let position = half_lines.mul_add(
                        -text_and_line_spacing,
                        middle_position,
                    );
                    ("end", position, x_width_padded.as_str())
                }
                TextAlignment::BottomLeft => {
                    let position = size.height
                        - (total_lines as f32
                            * text_and_line_spacing);
                    ("start", position, "10")
                }
                TextAlignment::BottomCenter => {
                    let position = size.height
                        - (total_lines as f32
                            * text_and_line_spacing);
                    ("middle", position, center_y.as_str())
                }
                TextAlignment::BottomRight => {
                    let position = size.height
                        - (total_lines as f32
                            * text_and_line_spacing);
                    ("end", position, x_width_padded.as_str())
                }
            };

        final_svg.push_str(&format!("<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\"><defs>", size.width, size.height, size.width, size.height));

        if let Some(shadow) = &self.shadow {
            final_svg.push_str(&format!(
                "<filter id=\"shadow\"><feDropShadow dx=\"{}\" dy=\"{}\" stdDeviation=\"{}\" flood-color=\"{}\"/></filter>",
                shadow.offset_x,
                shadow.offset_y,
                shadow.spread,
                shadow.color
            ));
        }
        final_svg.push_str("</defs>");

        // This would be how to apply kerning
        // final_svg.push_str(
        //     "<style> text { letter-spacing: 0em; } </style>",
        // );

        final_svg.push_str(&format!("<text x=\"0\" y=\"50%\" transform=\"translate({}, 0)\" dominant-baseline=\"middle\" text-anchor=\"{}\" font-weight=\"bold\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" ", text_x_position, text_anchor, self.font.name, font_size, self.fill));

        if let Some(stroke) = &self.stroke {
            final_svg.push_str(&format!(
                "stroke=\"{}\" stroke-width=\"{}px\" stroke-linejoin=\"arcs\" paint-order=\"stroke\"",
                stroke.color, stroke.size
            ));
        }

        if self.shadow.is_some() {
            final_svg.push_str(" style=\"filter:url(#shadow);\"");
        }
        final_svg.push_str(">");

        let text: String = self
            .text
            .lines()
            .enumerate()
            .map(|(index, text)| {
                format!(
                    "<tspan x=\"0\" y=\"{}\">{}</tspan>",
                    (index as f32).mul_add(
                        text_and_line_spacing,
                        starting_y_position
                    ),
                    text
                )
            })
            .collect();
        final_svg.push_str(&text);

        final_svg.push_str("</text></svg>");

        // final_svg.push_str(&format!(
        //     "<svg viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\"><defs>{}</defs><text x=\"50%\" y=\"50%\" dominant-baseline=\"middle\" text-anchor=\"middle\" font-weight=\"bold\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" {} style=\"filter:url(#shadow);\">{}</text></svg>",
        //     size.width,
        //     size.height,
        //     shadow,
        //     self.font.name,
        //     font_size,
        //     self.fill,
        //     stroke,
        //     text
        // ));

        if cache {
            let hashed_title = rapidhash_v3(final_svg.as_bytes());
            path.push(PathBuf::from(hashed_title.to_string()));
            path.set_extension("png");

            if path.exists() {
                // debug!("cached");
                let handle = Handle::from_path(path);
                self.handle = Some(handle);
                return self;
            }
        }
        // debug!("text string built...");
        let resvg_tree = Tree::from_data(
            final_svg.as_bytes(),
            &resvg::usvg::Options {
                fontdb: Arc::clone(&self.fontdb),
                ..Default::default()
            },
        )
        .expect("Woops mama");
        // debug!("parsed");
        let transform = tiny_skia::Transform::default();
        let mut pixmap =
            Pixmap::new(size.width as u32, size.height as u32)
                .expect("opops");
        resvg::render(&resvg_tree, transform, &mut pixmap.as_mut());
        // debug!("rendered");

        if cache {
            if let Err(e) = pixmap.save_png(&path) {
                error!(?e, "Couldn't save a copy of the text");
            }
        }

        // debug!("saved");
        // let handle = Handle::from_path(path);
        let handle = Handle::from_rgba(
            size.width as u32,
            size.height as u32,
            pixmap.take(),
        );
        self.handle = Some(handle);
        // debug!("stored");
        self
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        Image::new(self.handle.clone().unwrap())
            .content_fit(ContentFit::Cover)
            .width(Length::Fill)
            .height(Length::Fill)
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

pub fn text_svg_generator(
    slide: &mut crate::core::slide::Slide,
    fontdb: Arc<fontdb::Database>,
) {
    text_svg_generator_with_cache(slide, fontdb, true);
}

pub fn text_svg_generator_with_cache(
    slide: &mut crate::core::slide::Slide,
    fontdb: Arc<fontdb::Database>,
    cache: bool,
) {
    if !slide.text().is_empty() {
        let font = if let Some(font) = slide.font() {
            font
        } else {
            Font::default()
        };
        let text_svg = TextSvg::new(slide.text())
            .alignment(slide.text_alignment())
            .fill(
                slide.text_color().unwrap_or_else(|| "#fff".into()),
            );
        let text_svg = if let Some(stroke) = slide.stroke() {
            text_svg.stroke(stroke)
        } else {
            text_svg
        };
        let text_svg = if let Some(shadow) = slide.shadow() {
            text_svg.shadow(shadow)
        } else {
            text_svg
        };
        let text_svg =
            text_svg.font(font).fontdb(Arc::clone(&fontdb));
        // debug!(fill = ?text_svg.fill, font = ?text_svg.font, stroke = ?text_svg.stroke, shadow = ?text_svg.shadow, text = ?text_svg.text);
        let text_svg =
            text_svg.build(Size::new(1280.0, 720.0), cache);
        slide.text_svg = Some(text_svg);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::slide::Slide;

    use super::*;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use resvg::usvg::fontdb::Database;

    #[test]
    fn test_generator() {
        let slide = Slide::default();

        let mut fontdb = Database::new();
        fontdb.load_system_fonts();
        let fontdb = Arc::new(fontdb);
        (0..40).into_par_iter().for_each(|_| {
            let mut slide = slide
                .clone()
                .set_font_size(120)
                .set_font("Quicksand")
                .set_text("This is the first slide of text\nAnd we are singing\nTo save the world!");
            text_svg_generator_with_cache(
                &mut slide,
                Arc::clone(&fontdb),
                false,
            );
            assert!(
                slide
                    .text_svg
                    .is_some_and(|svg| svg.handle.is_some())
            )
        });
    }
}
