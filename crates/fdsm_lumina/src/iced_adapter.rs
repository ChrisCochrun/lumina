//! iced_wgpu adapter. This file intentionally uses only iced_wgpu's Primitive
//! and Pipeline traits and its re-exported GPU types. No winit/window/surface.
use crate::{
    PreparedText, StyledText, TextRenderer, cosmic_text::FontSystem,
    renderer::PhysicalBounds, wgpu,
};
use iced_wgpu::{Primitive, core::Rectangle, graphics::Viewport, primitive::Pipeline};
use std::sync::{Arc, Mutex};

pub struct FdsmTextPrimitive {
    pub(crate) font_system: Arc<Mutex<FontSystem>>,
    texts: Vec<StyledText>,
    prepared: Mutex<Vec<PreparedText>>,
}
impl std::fmt::Debug for FdsmTextPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FdsmTextPrimitive")
            .field("texts", &self.texts.len())
            .finish()
    }
}
impl FdsmTextPrimitive {
    pub fn new(font_system: Arc<Mutex<FontSystem>>) -> Self {
        Self {
            font_system,
            texts: Vec::new(),
            prepared: Mutex::new(Vec::new()),
        }
    }
    pub fn push_text(&mut self, text: StyledText) {
        self.texts.push(text);
    }
    pub fn total_height(&self) -> f32 {
        self.texts.iter().map(StyledText::height).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }
}

/// Shared once per Iced renderer type (not once per slide).
pub struct FdsmTextPipeline {
    pub renderer: TextRenderer,
}
impl Pipeline for FdsmTextPipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            renderer: TextRenderer::new(device, format),
        }
    }
}
impl Primitive for FdsmTextPrimitive {
    type Pipeline = FdsmTextPipeline;
    fn prepare(
        &self,
        pipeline: &mut FdsmTextPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let Ok(mut prepared) = self.prepared.lock() else {
            return;
        };
        // On failure, leave no stale draw data from a previous preparation.
        prepared.clear();
        let Ok(font_system) = self.font_system.lock() else {
            return;
        };
        let sf = viewport.scale_factor() as f32;
        let physical = PhysicalBounds::new(
            bounds.x * sf,
            bounds.y * sf,
            bounds.width * sf,
            bounds.height * sf,
        );
        let font_source = Arc::as_ptr(&self.font_system) as usize;
        for text in &self.texts {
            match pipeline.renderer.prepare(
                device,
                queue,
                &font_system,
                font_source,
                text,
                physical,
            ) {
                Ok(Some(item)) => prepared.push(item),
                Ok(None) => {}
                Err(err) => eprintln!("fdsm-iced-renderer prepare: {err}"),
            }
        }
    }
    fn draw(&self, pipeline: &FdsmTextPipeline, pass: &mut wgpu::RenderPass<'_>) -> bool {
        let Ok(items) = self.prepared.lock() else {
            return true;
        };
        // One call orders all prepared texts by effect layer. Drawing four
        // passes per item instead would let later glyphs' glows/strokes cover
        // earlier glyph bodies when letter spacing is tightened.
        pipeline.renderer.render_group(pass, items.as_slice());
        true
    }
}
