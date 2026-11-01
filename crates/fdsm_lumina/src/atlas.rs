//! Append-only GPU atlas. CPU FDSM generation lives in `glyph.rs`. Never repack during
//! frame preparation: earlier Iced primitives may already hold its UVs.
pub(crate) use crate::glyph::CacheKey;
use crate::{
    cosmic_text::FontSystem,
    glyph::{GlyphImage, generate_glyph_sync},
    renderer::RenderError,
    wgpu,
};
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
    /// Compatibility method: current live renderer remains synchronous until
    /// prewarming is wired into `TextRenderer`. Both CPU paths now use the same
    /// implementation in `glyph.rs`.
    pub fn obtain(
        &mut self,
        queue: &wgpu::Queue,
        font_system: &FontSystem,
        key: CacheKey,
    ) -> Result<GlyphPlacement, RenderError> {
        if let Some(info) = self.get(key) {
            return Ok(info);
        }
        let image = generate_glyph_sync(font_system, key)?;
        self.insert(queue, key, &image)
    }

    pub fn get(&self, key: CacheKey) -> Option<GlyphPlacement> {
        self.glyphs.get(&key).copied()
    }

    /// GPU-only operation. The worker's CPU image may be used by more than
    /// one atlas, but each atlas allocates its own rectangle and uploads it.
    pub fn insert(
        &mut self,
        queue: &wgpu::Queue,
        key: CacheKey,
        image: &GlyphImage,
    ) -> Result<GlyphPlacement, RenderError> {
        if let Some(info) = self.get(key) {
            return Ok(info);
        }
        let bitmap = match image {
            GlyphImage::Empty => {
                let info = GlyphPlacement {
                    rect: None,
                    bearing_x: 0.0,
                    bearing_y: 0.0,
                };
                self.glyphs.insert(key, info);
                return Ok(info);
            }
            GlyphImage::Bitmap(bitmap) => bitmap,
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
        // Expand RGB into RGBA without modifying the shared CPU bitmap.
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
