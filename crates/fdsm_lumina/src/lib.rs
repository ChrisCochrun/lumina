//! FDSM-backed styled text for iced/libcosmic.
//!
//! The crate intentionally **does not** own a window, surface, or event loop.
//! `TextRenderer` owns a long-lived glyph atlas and GPU pipeline; a `PreparedText`
//! contains the resources needed to draw one instance, allowing many Iced
//! primitives to prepare before any of them draws.
//!
//! Use the *same* `iced_wgpu` revision as Lumina. For glyph outlines,
//! cosmic-text 0.19 and fdsm-skrifa 0.2 both use skrifa 0.40.
#![forbid(unsafe_code)]

pub use iced_graphics::text::cosmic_text;
pub use iced_wgpu::wgpu;
mod atlas;
pub mod iced_adapter;
mod renderer;
pub mod style;
mod text;
pub use renderer::{
    PreparedText, RenderError, RenderLayer, RendererConfig, TextRenderer,
};
pub use text::{StyledText, StyledTextBuilder, TextRegion};
