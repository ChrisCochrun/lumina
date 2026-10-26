//! Span-level text styles shared by the CPU layout path and the GPU renderer.
//!
//! Effect widths, offsets, and gradient regions are expressed in screen pixels.
//! FDSM currently supplies RGB MSDF data (not an MTSDF true-distance alpha
//! channel), so effects use the reconstructed median distance in the fragment
//! shader. This is sufficient for common outline/glow/shadow effects, but is
//! not a byte-for-byte reproduction of Klyff's distance-field semantics.

use bytemuck::{Pod, Zeroable};

/// Normalized RGBA components passed to the fragment shader.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub [f32; 4]);

impl Rgba {
    pub const TRANSPARENT: Self = Self([0.0, 0.0, 0.0, 0.0]);
    pub const WHITE: Self = Self([1.0, 1.0, 1.0, 1.0]);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self([r, g, b, 1.0])
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GlyphColoring {
    Solid(Rgba),
    Gradient {
        primary: Rgba,
        secondary: Rgba,
        /// Direction in normalized gradient-region coordinates. For example,
        /// `[1, 0]` is left-to-right and `[0, 1]` is top-to-bottom.
        direction: [f32; 2],
    },
}

impl Default for GlyphColoring {
    fn default() -> Self {
        Self::Solid(Rgba::WHITE)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    pub color: GlyphColoring,
    pub width: f32,
    /// A 0..=1 shaping control for the anti-aliased transition around the
    /// outline. With RGB-only MSDF input this is a visual approximation, not
    /// an independent true-distance corner radius.
    pub roundness: f32,
}

impl Default for Outline {
    fn default() -> Self {
        Self {
            color: GlyphColoring::Solid(Rgba::TRANSPARENT),
            width: 0.0,
            roundness: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub color: GlyphColoring,
    /// Maximum outside/inside glow reach, in screen pixels.
    pub width: f32,
    /// Falloff exponent. Values around 1.0 are a linear falloff; larger values
    /// make the glow more concentrated near its edge.
    pub spread: f32,
    /// A 0..=1 blend toward a smoothstep-shaped glow falloff.
    pub roundness: f32,
}

impl Default for Glow {
    fn default() -> Self {
        Self {
            color: GlyphColoring::Solid(Rgba::TRANSPARENT),
            width: 0.0,
            spread: 1.0,
            roundness: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub color: GlyphColoring,
    /// Screen-pixel offset, with positive Y pointing down.
    pub direction: [f32; 2],
    /// Expansion of the source glyph silhouette, in screen pixels.
    pub additional_width: f32,
    /// Approximate soft edge width, in screen pixels.
    pub spread: f32,
    /// A 0..=1 blend toward a smoothstep-shaped soft edge.
    pub roundness: f32,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            color: GlyphColoring::Solid(Rgba::TRANSPARENT),
            direction: [0.0, 0.0],
            additional_width: 0.0,
            spread: 0.0,
            roundness: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub color: GlyphColoring,
    pub stroke_out: Outline,
    pub stroke_in: Outline,
    pub glow_out: Glow,
    pub glow_in: Glow,
    pub shadow: Shadow,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: GlyphColoring::Solid(Rgba::rgb(0.90, 0.94, 1.0)),
            stroke_out: Outline::default(),
            stroke_in: Outline::default(),
            glow_out: Glow::default(),
            glow_in: Glow::default(),
            shadow: Shadow::default(),
        }
    }
}

impl TextStyle {
    pub fn solid(color: Rgba) -> Self {
        Self {
            color: GlyphColoring::Solid(color),
            ..Self::default()
        }
    }
}

/// Optional viewport-independent gradient area in physical screen pixels.
/// `None` means the active viewport. A zero-sized explicit region is treated
/// the same way by the shader.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct GpuColoring {
    pub primary: [f32; 4],
    pub secondary: [f32; 4],
    /// x/y = direction, z = 0 for solid or 1 for gradient.
    pub direction_mode: [f32; 4],
    /// x/y/width/height; a zero-sized region means the viewport.
    pub region: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct GpuOutline {
    pub color: GpuColoring,
    /// x = width, y = transition shaping control.
    pub params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct GpuGlow {
    pub color: GpuColoring,
    /// x = width, y = falloff exponent, z = smooth-falloff blend.
    pub params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct GpuShadow {
    pub color: GpuColoring,
    /// x/y = pixel offset, z = extra width, w = approximate softness.
    pub params: [f32; 4],
    /// x = smooth-falloff blend.
    pub extra: [f32; 4],
}

/// A style is 30 vec4s (480 bytes); all fields have 16-byte WGSL-compatible
/// boundaries, so a runtime storage array has a predictable stride.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct GpuTextStyle {
    pub fill: GpuColoring,
    pub stroke_out: GpuOutline,
    pub stroke_in: GpuOutline,
    pub glow_out: GpuGlow,
    pub glow_in: GpuGlow,
    pub shadow: GpuShadow,
}

impl TextStyle {
    pub(crate) fn to_gpu(self, region: Option<GradientRegion>) -> GpuTextStyle {
        GpuTextStyle {
            fill: pack_coloring(self.color, region),
            stroke_out: GpuOutline {
                color: pack_coloring(self.stroke_out.color, region),
                params: [
                    self.stroke_out.width.max(0.0),
                    self.stroke_out.roundness.clamp(0.0, 1.0),
                    0.0,
                    0.0,
                ],
            },
            stroke_in: GpuOutline {
                color: pack_coloring(self.stroke_in.color, region),
                params: [
                    self.stroke_in.width.max(0.0),
                    self.stroke_in.roundness.clamp(0.0, 1.0),
                    0.0,
                    0.0,
                ],
            },
            glow_out: GpuGlow {
                color: pack_coloring(self.glow_out.color, region),
                params: [
                    self.glow_out.width.max(0.0),
                    self.glow_out.spread.max(0.01),
                    self.glow_out.roundness.clamp(0.0, 1.0),
                    0.0,
                ],
            },
            glow_in: GpuGlow {
                color: pack_coloring(self.glow_in.color, region),
                params: [
                    self.glow_in.width.max(0.0),
                    self.glow_in.spread.max(0.01),
                    self.glow_in.roundness.clamp(0.0, 1.0),
                    0.0,
                ],
            },
            shadow: GpuShadow {
                color: pack_coloring(self.shadow.color, region),
                params: [
                    self.shadow.direction[0],
                    self.shadow.direction[1],
                    self.shadow.additional_width.max(0.0),
                    self.shadow.spread.max(0.0),
                ],
                extra: [self.shadow.roundness.clamp(0.0, 1.0), 0.0, 0.0, 0.0],
            },
        }
    }
}

fn pack_coloring(color: GlyphColoring, region: Option<GradientRegion>) -> GpuColoring {
    let region = region.map_or([0.0; 4], |r| [r.x, r.y, r.width, r.height]);
    match color {
        GlyphColoring::Solid(color) => GpuColoring {
            primary: color.0,
            secondary: color.0,
            direction_mode: [1.0, 0.0, 0.0, 0.0],
            region,
        },
        GlyphColoring::Gradient {
            primary,
            secondary,
            direction,
        } => {
            let length = direction[0].hypot(direction[1]);
            let direction = if length > 1.0e-5 {
                [direction[0] / length, direction[1] / length]
            } else {
                [1.0, 0.0]
            };
            GpuColoring {
                primary: primary.0,
                secondary: secondary.0,
                direction_mode: [direction[0], direction[1], 1.0, 0.0],
                region,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_style_has_vec4_aligned_stride() {
        assert_eq!(std::mem::size_of::<GpuColoring>(), 64);
        assert_eq!(std::mem::size_of::<GpuOutline>(), 80);
        assert_eq!(std::mem::size_of::<GpuGlow>(), 80);
        assert_eq!(std::mem::size_of::<GpuShadow>(), 96);
        assert_eq!(std::mem::size_of::<GpuTextStyle>(), 480);
        assert_eq!(std::mem::size_of::<GpuTextStyle>() % 16, 0);
    }

    #[test]
    fn gpu_gradient_defaults_zero_region_to_viewport() {
        let style = TextStyle {
            color: GlyphColoring::Gradient {
                primary: Rgba::rgb(1.0, 0.0, 0.0),
                secondary: Rgba::rgb(0.0, 0.0, 1.0),
                direction: [0.0, 0.0],
            },
            ..TextStyle::default()
        };
        let gpu = style.to_gpu(None);
        assert_eq!(gpu.fill.direction_mode, [1.0, 0.0, 1.0, 0.0]);
        assert_eq!(gpu.fill.region, [0.0; 4]);
    }
}
