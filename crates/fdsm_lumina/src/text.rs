use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytemuck::{Pod, Zeroable};
use iced_graphics::text::cosmic_text::skrifa::{
    FontRef, GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
};

use fdsm::{
    bezier::scanline::FillRule, generate::generate_msdf, render::correct_sign_msdf,
    shape::Shape, transform::Transform,
};
use fdsm_skrifa::load_shape_from_face;
use fontdb::{Database, ID as FontId, Weight};
use iced_graphics::text::cosmic_text::{
    Attrs, AttrsOwned, Buffer, Family, FontSystem, Metrics, Shaping, Style, Wrap,
};
use iced_wgpu::wgpu;
use iced_wgpu::wgpu::util::DeviceExt;
use image::{RgbImage, Rgba, RgbaImage, imageops};
use nalgebra::{Affine2, Similarity2, Vector2};

use super::style::{
    Glow, GlyphColoring, GpuTextStyle, GradientRegion, Outline, Rgba as StyleRgba,
    Shadow, TextStyle,
};
// use winit::{
//     application::ApplicationHandler,
//     dpi::PhysicalSize,
//     event::WindowEvent,
//     event_loop::{ActiveEventLoop, EventLoop},
//     window::{Window, WindowId},
// };

const WINDOW_WIDTH: u32 = 1100;
const WINDOW_HEIGHT: u32 = 680;
const ATLAS_WIDTH: u32 = 2048;
const ATLAS_HEIGHT: u32 = 1024;
const FONT_PIXEL_HEIGHT: f32 = 164.0;
// FDSM's px-range is the full representable signed-distance interval, not the
// margin on each side. The bitmap margin is computed from the requested styles;
// the shader receives twice that margin as the full range.
const MIN_MSDF_PADDING_PX: f64 = 16.0;
const MSDF_FILTER_GUARD_PX: f64 = 2.0;
const LINE_HEIGHT: f32 = 182.0;
const FONT_FAMILY: &str = "Quicksand";
#[derive(Clone, Copy, Debug)]
struct AtlasRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// A glyph identity is the font face and glyph ID selected by cosmic-text.
/// The size is part of the key because the atlas stores glyphs at a particular
/// pixel size, rather than as a scale-independent vector outline.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct GlyphKey {
    font_id: FontId,
    glyph_id: u16,
    font_size_bits: u32,
}

impl GlyphKey {
    fn new(font_id: FontId, glyph_id: u16, font_size: f32) -> Self {
        Self {
            font_id,
            glyph_id,
            font_size_bits: font_size.to_bits(),
        }
    }

    fn font_size(self) -> f32 {
        f32::from_bits(self.font_size_bits)
    }
}

#[derive(Clone, Copy, Debug)]
struct GlyphInfo {
    /// Bitmap offset from the glyph origin / baseline, in screen pixels.
    bearing_x: f32,
    bearing_y: f32,
    atlas: Option<AtlasRect>,
}

struct GlyphAtlas {
    pixels: RgbaImage,
    glyphs: HashMap<GlyphKey, GlyphInfo>,
    /// Full encoded signed-distance range in atlas pixels (negative to positive).
    distance_range_px: f32,
}

/// A snapshot of cosmic-text's shaped and measured glyph layout. Keeping the
/// actual glyph IDs and positions means ligatures, fallback fonts, and shaping
/// advances are used consistently by both atlas generation and GPU placement.
#[derive(Clone, Debug)]
struct TextGlyph {
    key: GlyphKey,
    x: f32,
    y: f32,
    x_offset: f32,
    y_offset: f32,
    font_size: f32,
    /// Index into `TextLayout::styles`, propagated from cosmic-text metadata.
    style_index: u32,
}

#[derive(Clone, Debug)]
struct TextLine {
    width: f32,
    top: f32,
    baseline: f32,
    height: f32,
    glyphs: Vec<TextGlyph>,
}

#[derive(Clone, Debug)]
struct TextLayout {
    lines: Vec<TextLine>,
    styles: Vec<TextStyle>,
    gradient_region: Option<GradientRegion>,
}

struct GlyphBitmap {
    pixels: RgbImage,
    bearing_x: f32,
    bearing_y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    style_index: u32,
    /// Per-quad clamp bounds keep offset shadow samples inside this glyph tile.
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}

/// Build the font database used by cosmic-text. System fonts are always loaded;
/// optional project fonts can be provided with FDSM_FONT_DIR. If that variable
/// is unset, common local `fonts/` locations are checked for development setups.
fn make_font_system() -> FontSystem {
    let mut db = Database::new();
    db.load_system_fonts();

    let local_font_dir = env::var_os("FDSM_FONT_DIR").map(PathBuf::from).or_else(|| {
        [
            Path::new("fonts"),
            Path::new("../fonts"),
            Path::new("../../fonts"),
        ]
        .into_iter()
        .find(|path| path.is_dir())
        .map(Path::to_path_buf)
    });

    if let Some(font_dir) = local_font_dir {
        db.load_fonts_dir(&font_dir);
    }

    FontSystem::new_with_locale_and_db("en-US".to_owned(), db)
}

/// A source span associates cosmic-text font attributes with a GPU `TextStyle`.
/// The style index is stored as `Attrs::metadata`, which survives shaping and is
/// copied to the resulting `LayoutGlyph`s. This is important for ligatures and
/// fallback fonts: style is attached to the shaped output, not guessed from chars.
struct StyledSpan {
    text: String,
    attrs: AttrsOwned,
}

struct StyledTextBuilder {
    metrics: Metrics,
    default_attrs: AttrsOwned,
    spans: Vec<StyledSpan>,
    styles: Vec<TextStyle>,
    gradient_region: Option<GradientRegion>,
}

impl StyledTextBuilder {
    fn new(metrics: Metrics, default_attrs: &Attrs<'_>) -> Self {
        Self {
            metrics,
            default_attrs: AttrsOwned::new(default_attrs),
            spans: Vec::new(),
            styles: Vec::new(),
            gradient_region: None,
        }
    }

    fn set_gradient_region(&mut self, region: Option<GradientRegion>) -> &mut Self {
        self.gradient_region = region;
        self
    }

    fn push_text(
        &mut self,
        text: impl Into<String>,
        attrs: &Attrs<'_>,
        style: TextStyle,
    ) -> &mut Self {
        let text = text.into();
        if text.is_empty() {
            return self;
        }
        let style_index = self.styles.len();
        let styled_attrs = attrs.clone().metadata(style_index);
        self.spans.push(StyledSpan {
            text,
            attrs: AttrsOwned::new(&styled_attrs),
        });
        self.styles.push(style);
        self
    }

    /// Shape rich text once, then snapshot the glyph placements. All spans share
    /// one cosmic-text layout pass, preserving shaping, bidi, kerning, and fallback.
    fn finish(self, font_system: &mut FontSystem) -> Result<TextLayout, String> {
        let mut buffer = Buffer::new(font_system, self.metrics);
        buffer.set_wrap(Wrap::None);
        // Width is unconstrained; each visual line is centered using cosmic-text's
        // measured `line_w`, rather than wrapping at the window width.
        buffer.set_size(None, None);
        let default_attrs = self.default_attrs.as_attrs();
        buffer.set_rich_text(
            self.spans
                .iter()
                .map(|span| (span.text.as_str(), span.attrs.as_attrs())),
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(font_system, false);

        let mut lines = Vec::new();
        for run in buffer.layout_runs() {
            let glyphs = run
                .glyphs
                .iter()
                .map(|glyph| TextGlyph {
                    key: GlyphKey::new(glyph.font_id, glyph.glyph_id, glyph.font_size),
                    x: glyph.x,
                    y: glyph.y,
                    x_offset: glyph.x_offset,
                    y_offset: glyph.y_offset,
                    font_size: glyph.font_size,
                    style_index: u32::try_from(glyph.metadata).unwrap_or(0),
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

        if lines.is_empty() {
            return Err("cosmic-text produced no visible layout runs".to_owned());
        }

        let styles = if self.styles.is_empty() {
            vec![TextStyle::default()]
        } else {
            self.styles
        };
        Ok(TextLayout {
            lines,
            styles,
            gradient_region: self.gradient_region,
        })
    }
}

/// Example rich text displaying every effect family supported by the GPU style.
/// `StyledTextBuilder` is span-oriented like Klyff's builder; visual effects are
/// applied in the shader without extra draw calls for each span.
fn build_text_layout(font_system: &mut FontSystem) -> Result<TextLayout, String> {
    let attrs = Attrs::new()
        .family(Family::Name(FONT_FAMILY))
        .style(Style::Italic);
    let bold_attrs = attrs.clone().weight(Weight::BOLD);

    let muted = TextStyle {
        color: GlyphColoring::Solid(StyleRgba::rgb(0.78, 0.84, 0.94)),
        shadow: Shadow {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.01, 0.02, 0.06, 0.65)),
            direction: [0.0, 3.0],
            additional_width: 0.0,
            spread: 1.5,
            roundness: 0.35,
        },
        ..TextStyle::default()
    };

    let heading = TextStyle {
        color: GlyphColoring::Gradient {
            primary: StyleRgba::rgb(0.24, 0.92, 1.0),
            secondary: StyleRgba::rgb(1.0, 0.76, 0.36),
            direction: [1.0, 0.0],
        },
        stroke_out: Outline {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.015, 0.055, 0.12, 0.98)),
            width: 2.2,
            roundness: 0.25,
        },
        glow_out: Glow {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.10, 0.72, 1.0, 0.42)),
            width: 8.0,
            spread: 1.25,
            roundness: 0.35,
        },
        shadow: Shadow {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.0, 0.0, 0.0, 0.78)),
            direction: [4.0, 5.0],
            additional_width: 1.0,
            spread: 2.5,
            roundness: 0.45,
        },
        ..TextStyle::default()
    };

    let screen_text = TextStyle {
        color: GlyphColoring::Gradient {
            primary: StyleRgba::rgb(0.94, 0.97, 1.0),
            secondary: StyleRgba::rgb(0.54, 0.70, 1.0),
            direction: [1.0, 0.25],
        },
        stroke_in: Outline {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.22, 0.32, 0.60, 0.95)),
            width: 1.2,
            roundness: 0.3,
        },
        glow_out: Glow {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.35, 0.48, 1.0, 0.38)),
            width: 10.0,
            spread: 1.6,
            roundness: 0.6,
        },
        glow_in: Glow {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.65, 0.80, 1.0, 0.2)),
            width: 3.5,
            spread: 0.9,
            roundness: 0.55,
        },
        shadow: Shadow {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.01, 0.01, 0.04, 0.75)),
            direction: [2.0, 4.0],
            additional_width: 0.0,
            spread: 2.0,
            roundness: 0.5,
        },
        ..TextStyle::default()
    };

    let logo = TextStyle {
        color: GlyphColoring::Gradient {
            primary: StyleRgba::rgb(0.50, 1.0, 0.72),
            secondary: StyleRgba::rgb(0.98, 0.94, 0.48),
            direction: [0.2, -1.0],
        },
        stroke_out: Outline {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.0, 0.0, 0.0, 0.98)),
            width: 7.8,
            roundness: 1.0,
        },
        shadow: Shadow {
            color: GlyphColoring::Solid(StyleRgba::rgba(0.01, 0.01, 0.01, 0.75)),
            direction: [15.0, 15.0],
            additional_width: 0.0,
            spread: 18.0,
            roundness: 1.0,
        },

        ..TextStyle::default()
    };

    let mut builder =
        StyledTextBuilder::new(Metrics::new(FONT_PIXEL_HEIGHT, LINE_HEIGHT), &attrs);
    // None means gradients span the current viewport instead of restarting for
    // each glyph quad. Set an explicit rect for a text-local gradient.
    builder.set_gradient_region(None);
    builder.push_text("FDSM + ", &attrs, muted);
    builder.push_text("wgpu 28\n", &bold_attrs, heading);
    builder.push_text("MSDF text ", &attrs, muted);
    builder.push_text("on screen\n", &bold_attrs, screen_text);
    builder.push_text("Powered by ", &attrs, muted);
    builder.push_text("fdsm", &bold_attrs, logo);
    builder.finish(font_system)
}

/// Load one shaped glyph from the exact face chosen by cosmic-text and generate
/// its MSDF. Font bytes are obtained from cosmic-text's own font database, so
/// the outline face cannot accidentally differ from the face used for shaping.
fn build_glyph_bitmap(
    font_system: &FontSystem,
    key: GlyphKey,
    padding_px: f64,
) -> Result<Option<GlyphBitmap>, String> {
    // FDSM's `range` argument is the complete interval from minimum to maximum
    // representable distance: [-range / 2, range / 2]. A glyph inset by
    // `padding_px` needs a range of 2 * padding_px so the bitmap edge reaches
    // the full negative distance instead of saturating halfway through padding.
    let distance_range_px = 2.0 * padding_px;
    let loaded = font_system
        .db()
        .with_face_data(
            key.font_id,
            |font_data, face_index| -> Result<Option<GlyphBitmap>, String> {
                let face = FontRef::from_index(font_data, face_index)
                    .map_err(|error| format!("could not parse selected font face: {error:?}"))?;
                let glyph_id = GlyphId::new(u32::from(key.glyph_id));
                let location = LocationRef::default();
                let font_metrics = face.metrics(Size::unscaled(), location);
                let units_per_em = f64::from(font_metrics.units_per_em);
                if units_per_em <= 0.0 {
                    return Err("selected font has an invalid units-per-em value".to_owned());
                }

                let glyph_metrics = face.glyph_metrics(Size::unscaled(), location);
                let Some(bounds) = glyph_metrics.bounds(glyph_id) else {
                    return Ok(None);
                };

                // Whitespace and many control glyphs have no ink. cosmic-text still
                // positions them; their advance comes from the shaped layout.
                if bounds.x_max <= bounds.x_min || bounds.y_max <= bounds.y_min {
                    return Ok(None);
                }

                let scale = f64::from(key.font_size()) / units_per_em;
                let width = ((f64::from(bounds.x_max - bounds.x_min) * scale) + 2.0 * padding_px)
                    .ceil()
                    .max(1.0) as u32;
                let height = ((f64::from(bounds.y_max - bounds.y_min) * scale) + 2.0 * padding_px)
                    .ceil()
                    .max(1.0) as u32;

                let (mut shape, _) =
                    load_shape_from_face(&face, glyph_id, location).map_err(|error| {
                        format!(
                            "could not load outline for glyph {}: {error:?}",
                            key.glyph_id
                        )
                    })?;

                // Convert font units into atlas pixels and move the outline inside
                // the padded field. Font coordinates are Y-up.
                let transform = nalgebra::convert::<_, Affine2<f64>>(Similarity2::new(
                    Vector2::new(
                        padding_px - f64::from(bounds.x_min) * scale,
                        padding_px - f64::from(bounds.y_min) * scale,
                    ),
                    0.0,
                    scale,
                ));
                shape.transform(&transform);

                let colored_shape = Shape::edge_coloring_simple(
                    shape,
                    0.03,
                    69_441_337_420_u64.wrapping_add(u64::from(key.glyph_id)),
                );
                let prepared_shape = colored_shape.prepare();

                let mut msdf = RgbImage::new(width, height);
                generate_msdf(&prepared_shape, distance_range_px, &mut msdf);
                correct_sign_msdf(&mut msdf, &prepared_shape, FillRule::Nonzero);
                imageops::flip_vertical_in_place(&mut msdf);

                Ok(Some(GlyphBitmap {
                    pixels: msdf,
                    bearing_x: bounds.x_min * scale as f32 - padding_px as f32,
                    bearing_y: bounds.y_max * scale as f32 + padding_px as f32,
                }))
            },
        )
        .ok_or_else(|| {
            format!(
                "cosmic-text selected font face {:?}, but its font data could not be read",
                key.font_id
            )
        })??;

    Ok(loaded)
}

fn required_msdf_padding_px(text_layout: &TextLayout) -> f64 {
    // The outer glow is painted outside the outer stroke in Klyff, so these
    // extents add. This renderer samples a displaced shadow inside the same
    // glyph quad (instead of moving a separate shadow pass), so reserve room for
    // the largest absolute offset plus shadow expansion as well.
    let max_effect_extent = text_layout
        .styles
        .iter()
        .map(|style| {
            let outer_stroke_and_glow = style.stroke_out.width.max(0.0)
                + style.glow_out.width.max(0.0)
                + style.glow_out.spread.max(0.0);
            let shadow_offset =
                style.shadow.direction[0].hypot(style.shadow.direction[1]);
            let shadow_extent = shadow_offset
                + style.shadow.additional_width.max(0.0)
                + style.shadow.spread.max(0.0);
            outer_stroke_and_glow.max(shadow_extent)
        })
        .fold(0.0_f32, f32::max) as f64;

    // Keep a small margin beyond the largest effect for anti-aliasing and
    // bilinear filtering. With ordinary styles this stays at the 16px minimum.
    (max_effect_extent + MSDF_FILTER_GUARD_PX).max(MIN_MSDF_PADDING_PX)
}

fn build_glyph_atlas(
    font_system: &FontSystem,
    text_layout: &TextLayout,
) -> Result<GlyphAtlas, String> {
    let padding_px = required_msdf_padding_px(text_layout);
    let distance_range_px = 2.0 * padding_px;
    let mut unique_keys = Vec::new();
    let mut seen = HashSet::new();
    for glyph in text_layout.lines.iter().flat_map(|line| &line.glyphs) {
        if seen.insert(glyph.key) {
            unique_keys.push(glyph.key);
        }
    }

    let mut glyphs = HashMap::new();
    let mut bitmaps: Vec<(GlyphKey, GlyphBitmap)> = Vec::new();

    for key in unique_keys {
        let Some(bitmap) = build_glyph_bitmap(font_system, key, padding_px)? else {
            glyphs.insert(
                key,
                GlyphInfo {
                    bearing_x: 0.0,
                    bearing_y: 0.0,
                    atlas: None,
                },
            );
            continue;
        };
        glyphs.insert(
            key,
            GlyphInfo {
                bearing_x: bitmap.bearing_x,
                bearing_y: bitmap.bearing_y,
                atlas: None,
            },
        );
        bitmaps.push((key, bitmap));
    }

    // Neutral values (0.5) outside glyph rectangles avoid a false edge if a
    // filtered sample reaches atlas padding.
    let mut pixels =
        RgbaImage::from_pixel(ATLAS_WIDTH, ATLAS_HEIGHT, Rgba([128, 128, 128, 255]));
    let (mut x, mut y, mut row_height) = (2_u32, 2_u32, 0_u32);

    for (key, bitmap) in bitmaps {
        let width = bitmap.pixels.width();
        let height = bitmap.pixels.height();
        if width + 4 > ATLAS_WIDTH || height + 4 > ATLAS_HEIGHT {
            return Err(format!(
                "glyph {:?} is too large for the {}x{} atlas ({}x{})",
                key, ATLAS_WIDTH, ATLAS_HEIGHT, width, height
            ));
        }

        if x + width + 2 > ATLAS_WIDTH {
            x = 2;
            y = y.saturating_add(row_height + 2);
            row_height = 0;
        }
        if y + height + 2 > ATLAS_HEIGHT {
            return Err(format!(
                "glyph atlas exhausted while packing glyph {:?} ({}x{}); increase ATLAS_WIDTH/ATLAS_HEIGHT",
                key, width, height
            ));
        }

        for (gx, gy, rgb) in bitmap.pixels.enumerate_pixels() {
            pixels.put_pixel(x + gx, y + gy, Rgba([rgb[0], rgb[1], rgb[2], 255]));
        }

        let info = glyphs
            .get_mut(&key)
            .ok_or_else(|| format!("missing glyph metrics for {:?}", key))?;
        info.atlas = Some(AtlasRect {
            x,
            y,
            width,
            height,
        });

        x = x.saturating_add(width + 2);
        row_height = row_height.max(height);
    }

    Ok(GlyphAtlas {
        pixels,
        glyphs,
        distance_range_px: distance_range_px as f32,
    })
}

fn screen_to_clip(x: f32, y: f32, width: f32, height: f32) -> [f32; 2] {
    [2.0 * x / width - 1.0, 1.0 - 2.0 * y / height]
}

fn append_glyph_quad(
    vertices: &mut Vec<Vertex>,
    x: f32,
    y: f32,
    rect: AtlasRect,
    atlas_width: f32,
    atlas_height: f32,
    screen_width: f32,
    screen_height: f32,
    style_index: u32,
) {
    // Sample inside texel centers so filtering will not mix adjacent glyphs.
    let u0 = (rect.x as f32 + 0.5) / atlas_width;
    let v0 = (rect.y as f32 + 0.5) / atlas_height;
    let u1 = (rect.x as f32 + rect.width as f32 - 0.5) / atlas_width;
    let v1 = (rect.y as f32 + rect.height as f32 - 0.5) / atlas_height;
    let x1 = x + rect.width as f32;
    let y1 = y + rect.height as f32;

    let tl = Vertex {
        position: screen_to_clip(x, y, screen_width, screen_height),
        uv: [u0, v0],
        style_index,
        uv_min: [u0, v0],
        uv_max: [u1, v1],
    };
    let tr = Vertex {
        position: screen_to_clip(x1, y, screen_width, screen_height),
        uv: [u1, v0],
        style_index,
        uv_min: [u0, v0],
        uv_max: [u1, v1],
    };
    let bl = Vertex {
        position: screen_to_clip(x, y1, screen_width, screen_height),
        uv: [u0, v1],
        style_index,
        uv_min: [u0, v0],
        uv_max: [u1, v1],
    };
    let br = Vertex {
        position: screen_to_clip(x1, y1, screen_width, screen_height),
        uv: [u1, v1],
        style_index,
        uv_min: [u0, v0],
        uv_max: [u1, v1],
    };

    vertices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
}

fn layout_text_vertices(
    width: u32,
    height: u32,
    text_layout: &TextLayout,
    glyphs: &HashMap<GlyphKey, GlyphInfo>,
) -> Vec<Vertex> {
    let screen_width = width.max(1) as f32;
    let screen_height = height.max(1) as f32;
    let Some(first_line) = text_layout.lines.first() else {
        return Vec::new();
    };
    let Some(last_line) = text_layout.lines.last() else {
        return Vec::new();
    };

    let block_top = first_line.top;
    let block_bottom = last_line.top + last_line.height;
    let block_height = (block_bottom - block_top).max(0.0);
    let y_offset = (screen_height - block_height) * 0.5 - block_top;
    let mut vertices = Vec::new();

    for line in &text_layout.lines {
        // line.width is measured by cosmic-text after shaping. The placements
        // of individual glyphs retain ligature, kerning, and fallback decisions.
        let line_x = (screen_width - line.width) * 0.5;
        let baseline = line.baseline + y_offset;

        for glyph in &line.glyphs {
            let Some(info) = glyphs.get(&glyph.key) else {
                continue;
            };
            let Some(rect) = info.atlas else {
                // Spaces and glyphs without outlines still affect cosmic-text's
                // measured line width and following glyph positions.
                continue;
            };

            let glyph_origin_x = line_x + glyph.x + glyph.x_offset * glyph.font_size;
            let glyph_baseline_y = baseline + glyph.y - glyph.y_offset * glyph.font_size;
            let left = glyph_origin_x + info.bearing_x;
            let top = glyph_baseline_y - info.bearing_y;

            append_glyph_quad(
                &mut vertices,
                left,
                top,
                rect,
                ATLAS_WIDTH as f32,
                ATLAS_HEIGHT as f32,
                screen_width,
                screen_height,
                glyph.style_index,
            );
        }
    }

    vertices
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    // Keep resources alive for the lifetime of the bind group / updates.
    _atlas_texture: wgpu::Texture,
    _atlas_view: wgpu::TextureView,
    _atlas_sampler: wgpu::Sampler,
    _style_buffer: wgpu::Buffer,
    viewport_buffer: wgpu::Buffer,
    distance_range_px: f32,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    glyphs: HashMap<GlyphKey, GlyphInfo>,
    text_layout: TextLayout,
}

impl State {
    async fn new(window: Arc<Window>) -> Result<Self, String> {
        let mut font_system = make_font_system();
        let text_layout = build_text_layout(&mut font_system)?;
        let atlas = build_glyph_atlas(&font_system, &text_layout)?;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| format!("could not create wgpu surface: {error}"))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .map_err(|error| {
                format!("could not find a compatible GPU adapter: {error}")
            })?;

        // All resources are in one bind group (atlas, sampler, style storage
        // table, and viewport uniform). Limit the device to at most two groups.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("FDSM text device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_bind_groups: 2,
                    ..wgpu::Limits::default()
                },
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| format!("could not create wgpu device: {error}"))?;

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or_else(|| "surface has no compatible configuration".to_owned())?;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);

        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("FDSM MSDF glyph atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            atlas.pixels.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * ATLAS_WIDTH),
                rows_per_image: Some(ATLAS_HEIGHT),
            },
            wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
        );
        let atlas_view =
            atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("MSDF linear sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let gpu_styles: Vec<GpuTextStyle> = text_layout
            .styles
            .iter()
            .copied()
            .map(|style| style.to_gpu(text_layout.gradient_region))
            .collect();
        let style_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("per-span GPU text styles"),
            contents: bytemuck::cast_slice(&gpu_styles),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let initial_viewport = [
            config.width as f32,
            config.height as f32,
            atlas.distance_range_px,
            0.0,
        ];
        let viewport_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("text renderer viewport"),
                contents: bytemuck::cast_slice(&initial_viewport),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("MSDF atlas and style bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("MSDF atlas and text styles bind group"),
            layout: &texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: style_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: viewport_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("FDSM text pipeline layout"),
                bind_group_layouts: &[&texture_layout],
                immediate_size: 0,
            });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("FDSM MSDF shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("msdf.wgsl").into()),
        });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2,
                1 => Float32x2,
                2 => Uint32,
                3 => Float32x2,
                4 => Float32x2,
            ],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("FDSM MSDF text pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[vertex_layout],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertices = layout_text_vertices(
            config.width,
            config.height,
            &text_layout,
            &atlas.glyphs,
        );
        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("MSDF text vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            _atlas_texture: atlas_texture,
            _atlas_view: atlas_view,
            _atlas_sampler: atlas_sampler,
            _style_buffer: style_buffer,
            viewport_buffer,
            distance_range_px: atlas.distance_range_px,
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            glyphs: atlas.glyphs,
            text_layout,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        let viewport = [width as f32, height as f32, self.distance_range_px, 0.0];
        self.queue.write_buffer(
            &self.viewport_buffer,
            0,
            bytemuck::cast_slice(&viewport),
        );

        let vertices =
            layout_text_vertices(width, height, &self.text_layout, &self.glyphs);
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.vertex_count = vertices.len() as u32;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("FDSM text command encoder"),
                });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("FDSM text render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.035,
                            g: 0.045,
                            b: 0.075,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("FDSM + cosmic-text + wgpu 28 — MSDF text")
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                eprintln!("could not create window: {error}");
                event_loop.exit();
                return;
            }
        };

        match pollster::block_on(State::new(window.clone())) {
            Ok(state) => {
                self.window = Some(window);
                self.state = Some(state);
            }
            Err(error) => {
                eprintln!("failed to initialize FDSM/wgpu text app: {error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(|window| window.id()) != Some(window_id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(state) = self.state.as_mut() else {
                    return;
                };
                match state.render() {
                    Ok(()) | Err(wgpu::SurfaceError::Timeout) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let width = state.config.width;
                        let height = state.config.height;
                        state.resize(width, height);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(error) => eprintln!("surface render error: {error}"),
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout_with_styles(styles: Vec<TextStyle>) -> TextLayout {
        TextLayout {
            lines: Vec::new(),
            styles,
            gradient_region: None,
        }
    }

    #[test]
    fn padding_covers_outside_stroke_and_glow() {
        let style = TextStyle {
            stroke_out: Outline {
                color: GlyphColoring::Solid(StyleRgba::WHITE),
                width: 6.8,
                roundness: 0.4,
            },
            glow_out: Glow {
                color: GlyphColoring::Solid(StyleRgba::WHITE),
                width: 9.0,
                spread: 0.5,
                roundness: 0.5,
            },
            ..TextStyle::default()
        };
        let layout = layout_with_styles(vec![style]);
        let padding = required_msdf_padding_px(&layout);
        assert!(padding >= 6.8 + 9.0 + MSDF_FILTER_GUARD_PX);
        // FDSM represents [-range/2, range/2], so the full range must be 2x padding.
        let distance_range = 2.0 * padding;
        assert!(distance_range / 2.0 >= 6.8 + 9.0 + MSDF_FILTER_GUARD_PX);
    }

    #[test]
    fn padding_reserves_space_for_displaced_shadow() {
        let style = TextStyle {
            shadow: Shadow {
                color: GlyphColoring::Solid(StyleRgba::WHITE),
                direction: [3.0, 4.0],
                additional_width: 2.0,
                spread: 0.5,
                roundness: 0.5,
            },
            ..TextStyle::default()
        };
        let layout = layout_with_styles(vec![style]);
        let padding = required_msdf_padding_px(&layout);
        assert!(padding >= 5.0 + 2.0 + MSDF_FILTER_GUARD_PX);
        assert!(padding >= MIN_MSDF_PADDING_PX);
    }
}
