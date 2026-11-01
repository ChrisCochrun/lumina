//! Shared, GPU-independent FDSM state. Lumina creates one context and passes
//! cheap clones through presenter/slide widgets to their iced primitives.
use crate::{
    cosmic_text::FontSystem,
    glyph::CacheKey,
    glyph_generator::{GlyphGenerator, GlyphLookup, RequestStatus},
};
use std::{
    io,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct FdsmContext {
    inner: Arc<FdsmContextInner>,
}

struct FdsmContextInner {
    font_system: Arc<Mutex<FontSystem>>,
    font_source: usize,
    generator: GlyphGenerator,
}

impl FdsmContext {
    /// Construct one worker and one shared CPU cache for this FontSystem.
    /// Prefer this at application startup rather than per slide or per window.
    pub fn try_new(font_system: Arc<Mutex<FontSystem>>) -> io::Result<Self> {
        let font_source = Arc::as_ptr(&font_system) as usize;
        Ok(Self {
            inner: Arc::new(FdsmContextInner {
                font_system,
                font_source,
                generator: GlyphGenerator::new()?,
            }),
        })
    }

    /// Convenience constructor for applications that cannot continue without
    /// a glyph worker. `try_new()` is available for graceful error handling.
    pub fn new(font_system: Arc<Mutex<FontSystem>>) -> Self {
        Self::try_new(font_system).expect("failed to spawn FDSM glyph worker")
    }

    pub fn font_system(&self) -> &Arc<Mutex<FontSystem>> {
        &self.inner.font_system
    }

    pub fn cache_stats(&self) -> FdsmCacheStats {
        let stats = self.inner.generator.stats();
        FdsmCacheStats {
            pending: stats.queued,
            ready: stats.ready,
            failed: stats.failed,
            cpu_bitmap_bytes: stats.bytes,
        }
    }

    pub(crate) fn font_source(&self) -> usize {
        self.inner.font_source
    }

    pub(crate) fn lookup(&self, key: CacheKey) -> GlyphLookup {
        self.inner.generator.lookup(key)
    }

    /// Requires a matching borrowed FontSystem (e.g. from Primitive::prepare).
    /// Avoids reacquiring an already-held font mutex.
    pub(crate) fn request(
        &self,
        font_system: &FontSystem,
        key: CacheKey,
    ) -> RequestStatus {
        debug_assert_eq!(key.font_source, self.font_source());
        self.inner.generator.request(key, font_system)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FdsmCacheStats {
    pub pending: usize,
    pub ready: usize,
    pub failed: usize,
    pub cpu_bitmap_bytes: usize,
}
