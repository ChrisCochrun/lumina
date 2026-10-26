//! Klyff-inspired style API. Widths and offsets are logical pixels and are
//! scaled to physical pixels when a text instance is prepared for the GPU.
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub [f32; 4]);
impl Color {
    pub const WHITE: Self = Self([1., 1., 1., 1.]);
    pub const TRANSPARENT: Self = Self([0., 0., 0., 0.]);
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self([r, g, b, 1.])
    }
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GlyphColoring {
    Solid(Color),
    Gradient {
        primary: Color,
        secondary: Color,
        direction: [f32; 2],
    },
}
impl Default for GlyphColoring {
    fn default() -> Self {
        Self::Solid(Color::WHITE)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outline {
    pub color: GlyphColoring,
    pub width: f32,
    pub roundness: f32,
}
impl Default for Outline {
    fn default() -> Self {
        Self {
            color: GlyphColoring::Solid(Color::TRANSPARENT),
            width: 0.,
            roundness: 0.,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub color: GlyphColoring,
    pub width: f32,
    pub spread: f32,
    pub roundness: f32,
}
impl Default for Glow {
    fn default() -> Self {
        Self {
            color: GlyphColoring::Solid(Color::TRANSPARENT),
            width: 0.,
            spread: 0.,
            roundness: 0.,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub color: GlyphColoring,
    pub direction: [f32; 2],
    pub width: f32,
    pub spread: f32,
    pub softness: f32,
    pub roundness: f32,
}
impl Default for Shadow {
    fn default() -> Self {
        Self {
            color: GlyphColoring::Solid(Color::TRANSPARENT),
            direction: [0., 0.],
            width: 0.,
            spread: 0.,
            softness: 0.,
            roundness: 0.,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextStyle {
    pub color: GlyphColoring,
    pub stroke_out: Outline,
    pub stroke_in: Outline,
    pub glow_out: Glow,
    pub glow_in: Glow,
    pub shadow: Shadow,
}
/// Coordinates are local to the prepared text; not global window coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// The GPU structures exclusively use vec4-compatible fields, so their layouts
// are identical under WGSL storage alignment rules. No implicit host padding.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct GpuColoring {
    primary: [f32; 4],
    secondary: [f32; 4],
    dir_mode: [f32; 4],
    region: [f32; 4],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct GpuOutline {
    color: GpuColoring,
    params: [f32; 4],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct GpuGlow {
    color: GpuColoring,
    params: [f32; 4],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct GpuShadow {
    color: GpuColoring,
    params: [f32; 4],
    extra: [f32; 4],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub(crate) struct GpuTextStyle {
    pub fill: GpuColoring,
    stroke_out: GpuOutline,
    stroke_in: GpuOutline,
    glow_out: GpuGlow,
    glow_in: GpuGlow,
    shadow: GpuShadow,
}

fn color(c: GlyphColoring, region: Option<GradientRegion>) -> GpuColoring {
    let r = region.map_or([0.; 4], |r| [r.x, r.y, r.width, r.height]);
    match c {
        GlyphColoring::Solid(c) => GpuColoring {
            primary: c.0,
            secondary: c.0,
            dir_mode: [1., 0., 0., 0.],
            region: r,
        },
        GlyphColoring::Gradient {
            primary,
            secondary,
            direction,
        } => {
            let l = direction[0].hypot(direction[1]);
            let dir = if l > 1e-6 {
                [direction[0] / l, direction[1] / l]
            } else {
                [1., 0.]
            };
            GpuColoring {
                primary: primary.0,
                secondary: secondary.0,
                dir_mode: [dir[0], dir[1], 1., 0.],
                region: r,
            }
        }
    }
}
impl TextStyle {
    pub fn scaled(self, factor: f32) -> Self {
        let f = factor.max(0.);
        Self {
            stroke_out: Outline {
                width: self.stroke_out.width * f,
                ..self.stroke_out
            },
            stroke_in: Outline {
                width: self.stroke_in.width * f,
                ..self.stroke_in
            },
            glow_out: Glow {
                width: self.glow_out.width * f,
                ..self.glow_out
            },
            glow_in: Glow {
                width: self.glow_in.width * f,
                ..self.glow_in
            },
            shadow: Shadow {
                direction: [self.shadow.direction[0] * f, self.shadow.direction[1] * f],
                width: self.shadow.width * f,
                softness: self.shadow.softness * f,
                spread: self.shadow.spread,
                roundness: self.shadow.roundness,
                color: self.shadow.color,
            },
            ..self
        }
    }
    pub(crate) fn to_gpu(self, region: Option<GradientRegion>) -> GpuTextStyle {
        GpuTextStyle {
            fill: color(self.color, region),
            stroke_out: GpuOutline {
                color: color(self.stroke_out.color, region),
                params: [
                    self.stroke_out.width.max(0.),
                    self.stroke_out.roundness.clamp(0., 1.),
                    0.,
                    0.,
                ],
            },
            stroke_in: GpuOutline {
                color: color(self.stroke_in.color, region),
                params: [
                    self.stroke_in.width.max(0.),
                    self.stroke_in.roundness.clamp(0., 1.),
                    0.,
                    0.,
                ],
            },
            glow_out: GpuGlow {
                color: color(self.glow_out.color, region),
                params: [
                    self.glow_out.width.max(0.),
                    self.glow_out.spread.clamp(0., 1.),
                    self.glow_out.roundness.clamp(0., 1.),
                    0.,
                ],
            },
            glow_in: GpuGlow {
                color: color(self.glow_in.color, region),
                params: [
                    self.glow_in.width.max(0.),
                    self.glow_in.spread.clamp(0., 1.),
                    self.glow_in.roundness.clamp(0., 1.),
                    0.,
                ],
            },
            shadow: GpuShadow {
                color: color(self.shadow.color, region),
                params: [
                    self.shadow.direction[0],
                    self.shadow.direction[1],
                    self.shadow.width.max(0.0),
                    self.shadow.spread.clamp(0.0, 1.0),
                ],
                extra: [
                    self.shadow.roundness.clamp(0.0, 1.0),
                    self.shadow.softness.max(0.0),
                    0.0,
                    0.0,
                ],
            },
        }
    }
    pub fn required_padding(self) -> f32 {
        let outer_stroke_and_glow =
            self.stroke_out.width.max(0.0) + self.glow_out.width.max(0.0);

        let shadow_width = self.shadow.width.max(0.0);

        let shadow_softness =
            self.shadow.softness.max(0.0) * (1.0 - self.shadow.spread.clamp(0.0, 1.0));

        let shadow_offset = self.shadow.direction[0].hypot(self.shadow.direction[1]);

        let shadow_extent = shadow_offset + shadow_width + 0.5 * shadow_softness;

        outer_stroke_and_glow.max(shadow_extent)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_struct_layouts() {
        assert_eq!(std::mem::size_of::<GpuColoring>(), 64);
        assert_eq!(std::mem::size_of::<GpuOutline>(), 80);
        assert_eq!(std::mem::size_of::<GpuGlow>(), 80);
        assert_eq!(std::mem::size_of::<GpuShadow>(), 96);
        assert_eq!(std::mem::size_of::<GpuTextStyle>(), 480);
    }
    #[test]
    fn range_includes_offset_and_softness() {
        let s = TextStyle {
            shadow: Shadow {
                direction: [3., 4.],
                width: 12.,
                spread: 0.3,
                ..Shadow::default()
            },
            ..TextStyle::default()
        };
        assert!((s.required_padding() - 17.).abs() < 0.0001);
    }
    #[test]
    fn soft_shadow_without_expansion_has_padding() {
        let style = TextStyle {
            shadow: Shadow {
                direction: [0.0, 0.0],
                width: 0.0,
                softness: 40.0,
                spread: 0.0,
                ..Shadow::default()
            },
            ..TextStyle::default()
        };

        assert_eq!(style.required_padding(), 20.0);
    }

    #[test]
    fn hard_shadow_does_not_expand_soft_transition() {
        let style = TextStyle {
            shadow: Shadow {
                width: 10.0,
                softness: 40.0,
                spread: 1.0,
                ..Shadow::default()
            },
            ..TextStyle::default()
        };

        assert_eq!(style.required_padding(), 10.0);
    }
}
