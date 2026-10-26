//! Append-only GPU atlas and CPU FDSM generation. Never repack an atlas during
//! frame preparation: earlier Iced primitives may already hold its UVs.
use crate::renderer::RenderError;
use crate::text::GlyphIdKey;
use crate::{
    cosmic_text::{
        FontSystem,
        skrifa::{
            FontRef, GlyphId, MetadataProvider,
            instance::{LocationRef, Size},
        },
    },
    wgpu,
};
use fdsm::{
    bezier::scanline::FillRule, generate::generate_msdf, render::correct_sign_msdf,
    shape::Shape, transform::Transform,
};
use fdsm_skrifa::load_shape_from_face;
use image::{RgbImage, imageops};
use nalgebra::{Affine2, Similarity2, Vector2};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlyphPlacement {
    pub rect: Option<AtlasRect>,
    pub bearing_x: f32,
    pub bearing_y: f32,
}
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
    fn font_size(self) -> f32 {
        self.ppem_quarters as f32 / 4.
    }
}
struct GlyphBitmap {
    pixels: RgbImage,
    bearing_x: f32,
    bearing_y: f32,
}

pub(crate) struct GlyphAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
    pub(crate) glyphs: HashMap<CacheKey, GlyphPlacement>,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}
impl GlyphAtlas {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("FDSM text atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("FDSM MSDF linear sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            width,
            height,
            glyphs: HashMap::new(),
            cursor_x: 2,
            cursor_y: 2,
            row_height: 0,
        }
    }
    pub fn obtain(
        &mut self,
        queue: &wgpu::Queue,
        font_system: &FontSystem,
        key: CacheKey,
    ) -> Result<GlyphPlacement, RenderError> {
        if let Some(info) = self.glyphs.get(&key) {
            return Ok(*info);
        }
        let Some(bitmap) = generate_glyph(font_system, key)? else {
            let info = GlyphPlacement {
                rect: None,
                bearing_x: 0.,
                bearing_y: 0.,
            };
            self.glyphs.insert(key, info);
            return Ok(info);
        };
        let w = bitmap.pixels.width();
        let h = bitmap.pixels.height();
        let (x, y, next_row_height, next_x) = shelf_position(
            self.cursor_x,
            self.cursor_y,
            self.row_height,
            self.width,
            self.height,
            w,
            h,
        )
        .ok_or(RenderError::AtlasFull {
            width: self.width,
            height: self.height,
        })?;
        // Expand RGB into RGBA. Write only the reserved glyph rectangle; the
        // glyph itself has a sufficient FDSM distance margin on every side.
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for pixel in bitmap.pixels.pixels() {
            rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        let info = GlyphPlacement {
            rect: Some(AtlasRect {
                x,
                y,
                width: w,
                height: h,
            }),
            bearing_x: bitmap.bearing_x,
            bearing_y: bitmap.bearing_y,
        };
        self.cursor_x = next_x;
        self.cursor_y = y;
        self.row_height = next_row_height;
        self.glyphs.insert(key, info);
        Ok(info)
    }
}

// Pure/testable shelf allocator. Keep 2px between neighboring atlas rectangles.
fn shelf_position(
    mut x: u32,
    mut y: u32,
    mut row_height: u32,
    atlas_w: u32,
    atlas_h: u32,
    w: u32,
    h: u32,
) -> Option<(u32, u32, u32, u32)> {
    if w.checked_add(4)? > atlas_w || h.checked_add(4)? > atlas_h {
        return None;
    }
    if x.checked_add(w)?.checked_add(2)? > atlas_w {
        x = 2;
        y = y.checked_add(row_height)?.checked_add(2)?;
        row_height = 0;
    }
    if y.checked_add(h)?.checked_add(2)? > atlas_h {
        return None;
    }
    Some((x, y, row_height.max(h), x + w + 2))
}

fn generate_glyph(
    font_system: &FontSystem,
    key: CacheKey,
) -> Result<Option<GlyphBitmap>, RenderError> {
    let result = font_system
        .db()
        .with_face_data(
            key.glyph.font_id,
            |bytes, index| -> Result<Option<GlyphBitmap>, RenderError> {
                let face = FontRef::from_index(bytes, index)
                    .map_err(|e| RenderError::Font(format!("font face: {e:?}")))?;
                let gid = GlyphId::new(u32::from(key.glyph.glyph_id));
                let location = LocationRef::default();
                let upem =
                    f64::from(face.metrics(Size::unscaled(), location).units_per_em);
                if upem <= 0. {
                    return Err(RenderError::Font("zero units_per_em".into()));
                }
                let metrics = face.glyph_metrics(Size::unscaled(), location);
                let Some(b) = metrics.bounds(gid) else {
                    return Ok(None);
                };
                if b.x_max <= b.x_min || b.y_max <= b.y_min {
                    return Ok(None);
                }
                let scale = f64::from(key.font_size()) / upem;
                let padding = f64::from(key.padding);
                let w = ((f64::from(b.x_max - b.x_min) * scale) + 2. * padding)
                    .ceil()
                    .max(1.) as u32;
                let h = ((f64::from(b.y_max - b.y_min) * scale) + 2. * padding)
                    .ceil()
                    .max(1.) as u32;
                let (mut shape, _) =
                    load_shape_from_face(&face, gid, location).map_err(|e| {
                        RenderError::Font(format!("glyph {}: {e:?}", key.glyph.glyph_id))
                    })?;
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
                Ok(Some(GlyphBitmap {
                    pixels: image,
                    bearing_x: b.x_min * scale as f32 - padding as f32,
                    bearing_y: b.y_max * scale as f32 + padding as f32,
                }))
            },
        )
        .ok_or_else(|| {
            RenderError::Font(format!("unknown font id {:?}", key.glyph.font_id))
        })?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pack_rows_and_overflow() {
        assert_eq!(
            shelf_position(2, 2, 0, 100, 100, 40, 20),
            Some((2, 2, 20, 44))
        );
        assert_eq!(
            shelf_position(90, 2, 20, 100, 100, 30, 40),
            Some((2, 24, 40, 34))
        );
        assert!(shelf_position(2, 80, 10, 100, 100, 40, 30).is_none());
    }
    #[test]
    fn packing_preserves_two_pixel_separation() {
        let (_, _, row, next_x) = shelf_position(2, 2, 0, 100, 100, 20, 20).unwrap();
        let (x, y, _, _) = shelf_position(next_x, 2, row, 100, 100, 20, 20).unwrap();
        assert_eq!(x, 24);
        assert_eq!(y, 2);
    }
}
