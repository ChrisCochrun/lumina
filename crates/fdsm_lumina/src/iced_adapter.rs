//! iced_wgpu adapter: Lumina supplies text, FDSM owns generation and GPU upload.
//! The pipeline remains iced-owned; CPU results are shared through FdsmContext.
use crate::{
    FdsmContext, PreparedText, StyledText, TextRenderer, renderer::PhysicalBounds, wgpu,
};
use iced_wgpu::{Primitive, core::Rectangle, graphics::Viewport, primitive::Pipeline};
use std::sync::{Arc, Mutex};

/// Bound the amount of GPU texture upload work done for upcoming text per
/// renderer prepare call. Tune using frame-time measurements.
const PREWARM_UPLOADS_PER_FRAME: usize = 4;

pub struct FdsmTextPrimitive {
    context: Arc<Mutex<FdsmContext>>,
    texts: Vec<Arc<StyledText>>,
    prewarm_texts: Vec<Arc<StyledText>>,
    prepared: Mutex<Vec<PreparedText>>,
}
impl std::fmt::Debug for FdsmTextPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FdsmTextPrimitive")
            .field("texts", &self.texts.len())
            .field("prewarm_texts", &self.prewarm_texts.len())
            .finish()
    }
}
impl FdsmTextPrimitive {
    pub fn new(context: Arc<Mutex<FdsmContext>>) -> Self {
        Self {
            context,
            texts: Vec::new(),
            prewarm_texts: Vec::new(),
            prepared: Mutex::new(Vec::new()),
        }
    }

    pub fn push_text(&mut self, text: Arc<StyledText>) {
        self.texts.push(text);
    }

    /// The upcoming texts share this primitive's destination physical bounds.
    /// Do not submit these as normal drawing texts: we only need their glyphs.
    pub fn prewarm(&mut self, texts: impl IntoIterator<Item = Arc<StyledText>>) {
        self.prewarm_texts.extend(texts);
    }

    pub fn total_height(&self) -> f32 {
        self.texts.iter().map(|t| t.height()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.texts.is_empty() && self.prewarm_texts.is_empty()
    }
}

/// One per iced rendering pipeline; each owns an independent GPU glyph atlas.
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
        prepared.clear();
        let Ok(context) = self.context.lock() else {
            return;
        };
        let Ok(font_system) = context.font_system().lock() else {
            return;
        };
        let sf = viewport.scale_factor() as f32;
        let physical = PhysicalBounds::new(
            bounds.x * sf,
            bounds.y * sf,
            bounds.width * sf,
            bounds.height * sf,
        );

        // Prepare visible text FIRST: preserve complete text even for a cold
        // direct jump. Warmed glyphs take the fast atlas lookup/upload path.
        for text in &self.texts {
            match pipeline.renderer.prepare_with_context(
                device,
                queue,
                &font_system,
                &context,
                text,
                physical,
            ) {
                Ok(Some(item)) => prepared.push(item),
                Ok(None) => {}
                Err(error) => {
                    tracing::error!(%error, "fdsm visible text preparation failed")
                }
            }
        }

        // Only glyphs, not text buffers. Future work never blocks for a worker;
        // GPU uploads are limited to a small count per prepare invocation.
        if !self.prewarm_texts.is_empty() {
            match pipeline.renderer.prewarm(
                queue,
                &font_system,
                &context,
                self.prewarm_texts.iter().map(|text| text.as_ref()),
                physical,
                PREWARM_UPLOADS_PER_FRAME,
            ) {
                Ok(progress) => {
                    tracing::trace!(
                        ?progress,
                        "fdsm prewarm progress for this GPU pipeline"
                    );
                }
                Err(error) => tracing::error!(%error, "fdsm prewarm failed"),
            }
        }
    }

    fn draw(&self, pipeline: &FdsmTextPipeline, pass: &mut wgpu::RenderPass<'_>) -> bool {
        let Ok(items) = self.prepared.lock() else {
            return true;
        };
        pipeline.renderer.render_group(pass, items.as_slice());
        true
    }
}
