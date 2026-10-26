//! CPU text shaping. The GPU never has to parse Unicode or decide glyph IDs.
use crate::cosmic_text::{
    Align, Attrs, AttrsOwned, Buffer, FontSystem, Metrics, Shaping, Wrap,
};
use crate::style::{GradientRegion, TextStyle};
use fontdb::ID as FontId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRegion {
    pub width: f32,
    pub height: f32,
}
impl TextRegion {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct GlyphIdKey {
    pub font_id: FontId,
    pub glyph_id: u16,
    pub size_bits: u32,
}
impl GlyphIdKey {
    pub fn new(font_id: FontId, glyph_id: u16, font_size: f32) -> Self {
        Self {
            font_id,
            glyph_id,
            size_bits: font_size.to_bits(),
        }
    }
    pub fn font_size(self) -> f32 {
        f32::from_bits(self.size_bits)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct TextGlyph {
    pub key: GlyphIdKey,
    pub x: f32,
    pub y: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub font_size: f32,
    pub style_index: u32,
}
#[derive(Clone, Debug)]
pub(crate) struct TextLine {
    pub width: f32,
    pub top: f32,
    pub baseline: f32,
    pub height: f32,
    pub glyphs: Vec<TextGlyph>,
}
#[derive(Debug)]
pub struct StyledText {
    pub(crate) lines: Vec<TextLine>,
    pub(crate) styles: Vec<TextStyle>,
    pub region: TextRegion,
    pub scale: f32,
    pub gradient_region: Option<GradientRegion>,
    pub measured_height: f32,
}
impl StyledText {
    pub fn height(&self) -> f32 {
        self.measured_height
    }
    pub fn style_at(&self, idx: usize) -> &TextStyle {
        self.styles.get(idx).unwrap_or(&self.styles[0])
    }
    pub fn styles(&self) -> impl Iterator<Item = &TextStyle> {
        self.styles.iter()
    }
}
struct Span {
    text: String,
    attrs: AttrsOwned,
}
pub struct StyledTextBuilder {
    region: TextRegion,
    metrics: Metrics,
    default_attrs: AttrsOwned,
    spans: Vec<Span>,
    styles: Vec<TextStyle>,
    shaping: Shaping,
    align: Option<Align>,
    scale: f32,
    gradient_region: Option<GradientRegion>,
}
impl StyledTextBuilder {
    pub fn new(region: TextRegion, metrics: Metrics, default_attrs: &Attrs<'_>) -> Self {
        Self {
            region,
            metrics,
            default_attrs: AttrsOwned::new(default_attrs),
            spans: Vec::new(),
            styles: Vec::new(),
            shaping: Shaping::Advanced,
            align: None,
            scale: 1.,
            gradient_region: None,
        }
    }
    pub fn set_align(&mut self, a: Option<Align>) -> &mut Self {
        self.align = a;
        self
    }
    pub fn set_shaping(&mut self, s: Shaping) -> &mut Self {
        self.shaping = s;
        self
    }
    pub fn set_scale(&mut self, s: f32) -> &mut Self {
        self.scale = s.max(0.);
        self
    }
    pub fn set_gradient_region(&mut self, r: Option<GradientRegion>) -> &mut Self {
        self.gradient_region = r;
        self
    }
    pub fn push_text(
        &mut self,
        text: &str,
        attrs: &Attrs<'_>,
        style: TextStyle,
    ) -> usize {
        let idx = self.styles.len();
        self.spans.push(Span {
            text: text.to_string(),
            attrs: AttrsOwned::new(&attrs.clone().metadata(idx)),
        });
        self.styles.push(style);
        idx
    }
    pub fn finish(self, font_system: &mut FontSystem) -> StyledText {
        let mut buf = Buffer::new(font_system, self.metrics);
        buf.set_wrap(Wrap::Word);
        buf.set_size(
            Some(self.region.width.max(1.)),
            Some(self.region.height.max(1.)),
        );
        let attrs = self.default_attrs.as_attrs();
        buf.set_rich_text(
            self.spans
                .iter()
                .map(|s| (s.text.as_str(), s.attrs.as_attrs())),
            &attrs,
            self.shaping,
            self.align,
        );
        buf.shape_until_scroll(font_system, false);
        let mut lines = Vec::new();
        for run in buf.layout_runs() {
            let glyphs = run
                .glyphs
                .iter()
                .map(|g| TextGlyph {
                    key: GlyphIdKey::new(g.font_id, g.glyph_id, g.font_size),
                    x: g.x,
                    y: g.y,
                    x_offset: g.x_offset,
                    y_offset: g.y_offset,
                    font_size: g.font_size,
                    style_index: u32::try_from(g.metadata).unwrap_or(0),
                })
                .collect();
            lines.push(TextLine {
                width: run.line_w,
                top: run.line_top,
                baseline: run.line_y,
                height: run.line_height,
                glyphs,
            });
        }
        let measured_height = match (lines.first(), lines.last()) {
            (Some(first), Some(last)) => (last.top + last.height - first.top).max(0.),
            _ => 0.,
        };
        StyledText {
            lines,
            styles: if self.styles.is_empty() {
                vec![TextStyle::default()]
            } else {
                self.styles
            },
            region: self.region,
            scale: self.scale,
            gradient_region: self.gradient_region,
            measured_height,
        }
    }
}
