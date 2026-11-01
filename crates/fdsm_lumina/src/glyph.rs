//! CPU-only MSDF glyph generation. This module never touches wgpu.
//!
//! The algorithm is moved from the original `atlas.rs` without changing its
//! geometry, distance range, edge coloring, or output format.
use crate::{
    cosmic_text::{
        FontSystem,
        skrifa::{
            FontRef, GlyphId, MetadataProvider,
            instance::{LocationRef, Size},
        },
    },
    renderer::RenderError,
    text::GlyphIdKey,
};
use fdsm::{
    bezier::scanline::FillRule, generate::generate_msdf, render::correct_sign_msdf,
    shape::Shape, transform::Transform,
};
use fdsm_skrifa::load_shape_from_face;
use fontdb::ID as FontId;
use image::{RgbImage, imageops};
use nalgebra::{Affine2, Similarity2, Vector2};
use std::sync::Arc;

/// Identifies a CPU-generated MSDF bitmap. Do not change identity while moving
/// generation off-thread: the live renderer and the prewarmer must agree.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct CacheKey {
    pub font_source: usize,
    pub glyph: GlyphIdKey,
    pub ppem_quarters: u32,
    pub padding: u32,
}

impl CacheKey {
    pub fn new(
        font_source: usize,
        glyph: GlyphIdKey,
        target_size: f32,
        padding: u32,
    ) -> Self {
        Self {
            font_source,
            glyph,
            ppem_quarters: (target_size.max(1.) * 4.).round() as u32,
            padding,
        }
    }

    pub fn font_size(self) -> f32 {
        self.ppem_quarters as f32 / 4.
    }
}

/// Immutable face data copied once per font ID and shared across jobs.
/// `face_index` matters for TTC/OTC font collections.
#[derive(Debug)]
pub(crate) struct FontFaceData {
    pub bytes: Arc<[u8]>,
    pub face_index: u32,
}

impl FontFaceData {
    pub fn load(font_system: &FontSystem, font_id: FontId) -> Result<Self, RenderError> {
        font_system
            .db()
            .with_face_data(font_id, |bytes, face_index| Self {
                bytes: Arc::from(bytes),
                face_index,
            })
            .ok_or_else(|| RenderError::Font(format!("unknown font id {font_id:?}")))
    }
}

/// A self-contained, Send-able job; no FontSystem lock is held by workers.
#[derive(Debug)]
pub(crate) struct GlyphJob {
    pub key: CacheKey,
    pub font: Arc<FontFaceData>,
}

#[derive(Debug)]
pub(crate) struct GlyphBitmap {
    pub pixels: RgbImage,
    pub bearing_x: f32,
    pub bearing_y: f32,
}

impl GlyphBitmap {
    pub fn byte_len(&self) -> usize {
        self.pixels.as_raw().len()
    }
}

/// `Empty` is a successful result for glyphs with no outline (e.g. space).
#[derive(Debug, Clone)]
pub(crate) enum GlyphImage {
    Empty,
    Bitmap(Arc<GlyphBitmap>),
}

/// The original FDSM CPU algorithm, with font data supplied by the job rather
/// than retrieved from FontSystem while rendering.
pub(crate) fn generate_glyph(job: &GlyphJob) -> Result<GlyphImage, RenderError> {
    generate_glyph_from_face(job.key, job.font.bytes.as_ref(), job.font.face_index)
}

/// Temporary synchronous compatibility path for GlyphAtlas::obtain(). Both
/// paths execute the exact same algorithm and produce the same RGB pixels.
pub(crate) fn generate_glyph_sync(
    font_system: &FontSystem,
    key: CacheKey,
) -> Result<GlyphImage, RenderError> {
    font_system
        .db()
        .with_face_data(key.glyph.font_id, |bytes, index| {
            generate_glyph_from_face(key, bytes, index)
        })
        .ok_or_else(|| {
            RenderError::Font(format!("unknown font id {:?}", key.glyph.font_id))
        })?
}

fn generate_glyph_from_face(
    key: CacheKey,
    font_bytes: &[u8],
    face_index: u32,
) -> Result<GlyphImage, RenderError> {
    let face = FontRef::from_index(font_bytes, face_index)
        .map_err(|e| RenderError::Font(format!("font face: {e:?}")))?;
    let gid = GlyphId::new(u32::from(key.glyph.glyph_id));
    let location = LocationRef::default();
    let upem = f64::from(face.metrics(Size::unscaled(), location).units_per_em);
    if upem <= 0. {
        return Err(RenderError::Font("zero units_per_em".into()));
    }
    let metrics = face.glyph_metrics(Size::unscaled(), location);
    let Some(b) = metrics.bounds(gid) else {
        return Ok(GlyphImage::Empty);
    };
    if b.x_max <= b.x_min || b.y_max <= b.y_min {
        return Ok(GlyphImage::Empty);
    }

    let scale = f64::from(key.font_size()) / upem;
    let padding = f64::from(key.padding);
    let w = ((f64::from(b.x_max - b.x_min) * scale) + 2. * padding)
        .ceil()
        .max(1.) as u32;
    let h = ((f64::from(b.y_max - b.y_min) * scale) + 2. * padding)
        .ceil()
        .max(1.) as u32;

    let (mut shape, _) = load_shape_from_face(&face, gid, location)
        .map_err(|e| RenderError::Font(format!("glyph {}: {e:?}", key.glyph.glyph_id)))?;
    let transform = nalgebra::convert::<_, Affine2<f64>>(Similarity2::new(
        Vector2::new(
            padding - f64::from(b.x_min) * scale,
            padding - f64::from(b.y_min) * scale,
        ),
        0.,
        scale,
    ));
    shape.transform(&transform);
    let colored = Shape::edge_coloring_simple(
        shape,
        0.03,
        69441337420_u64.wrapping_add(u64::from(key.glyph.glyph_id)),
    );
    let prepared = colored.prepare();
    let mut image = RgbImage::new(w, h);
    generate_msdf(&prepared, 2. * padding, &mut image);
    correct_sign_msdf(&mut image, &prepared, FillRule::Nonzero);
    imageops::flip_vertical_in_place(&mut image);

    Ok(GlyphImage::Bitmap(Arc::new(GlyphBitmap {
        pixels: image,
        bearing_x: b.x_min * scale as f32 - padding as f32,
        bearing_y: b.y_max * scale as f32 + padding as f32,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_key_size_is_quantized_to_quarters() {
        let system = FontSystem::new();
        // Select an actual fontdb ID only if this environment has fonts.
        let Some(font_id) = system.db().faces().next().map(|face| face.id) else {
            return;
        };
        let key = CacheKey::new(123, GlyphIdKey::new(font_id, 1, 48.0), 48.1, 16);
        assert_eq!(key.font_size(), 48.0);
    }
}
