//! One background MSDF worker, bounded requests, shared CPU bitmap results.
//! No wgpu dependencies and no long-lived FontSystem lock on the worker.
use crate::{
    cosmic_text::FontSystem,
    glyph::{CacheKey, FontFaceData, GlyphImage, GlyphJob, generate_glyph},
};
use fontdb::ID as FontId;
use std::{
    collections::HashMap,
    io,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender, TrySendError},
    },
    thread,
};

const JOB_QUEUE_CAPACITY: usize = 256;

/// Non-blocking cache lookup. `Ready` can be uploaded into any GPU atlas.
#[derive(Debug, Clone)]
pub(crate) enum GlyphLookup {
    Missing,
    Pending,
    Ready(GlyphImage),
    Failed(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestStatus {
    Queued,
    AlreadyPending,
    AlreadyReady,
    QueueFull,
    Failed,
}

#[derive(Debug)]
enum Entry {
    Pending,
    Ready(GlyphImage),
    Failed(Arc<str>),
}

#[derive(Debug, Default)]
struct SharedState {
    entries: HashMap<CacheKey, Entry>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct GlyphCacheStats {
    pub queued: usize,
    pub ready: usize,
    pub failed: usize,
    pub bytes: usize,
}

/// Owned by one FdsmContext. The single worker is created ONCE in new().
/// When the context drops, the worker stops after its current job.
pub(crate) struct GlyphGenerator {
    jobs: SyncSender<GlyphJob>,
    state: Arc<Mutex<SharedState>>,
    faces: Mutex<HashMap<FontId, Arc<FontFaceData>>>,
    stop: Arc<AtomicBool>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

impl GlyphGenerator {
    pub fn new() -> io::Result<Self> {
        let (jobs, receiver) = mpsc::sync_channel::<GlyphJob>(JOB_QUEUE_CAPACITY);
        let state = Arc::new(Mutex::new(SharedState::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_state = Arc::clone(&state);
        let worker_stop = Arc::clone(&stop);

        thread::Builder::new()
            .name("fdsm-glyph-generator".to_owned())
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    if worker_stop.load(Ordering::Acquire) {
                        break;
                    }
                    let key = job.key;
                    // The worker NEVER locks FontSystem or the cache during
                    // actual generation. Catching a single glyph panic keeps
                    // subsequent jobs from being permanently stuck as Pending.
                    let computed =
                        catch_unwind(AssertUnwindSafe(|| generate_glyph(&job)));
                    let entry = match computed {
                        Ok(Ok(image)) => Entry::Ready(image),
                        Ok(Err(err)) => Entry::Failed(err.to_string().into()),
                        Err(_) => Entry::Failed("glyph generator panicked".into()),
                    };
                    let mut state = lock(&worker_state);
                    state.entries.insert(key, entry);
                    // Release the lock before receiving/generating next glyph.
                }
            })?;

        Ok(Self {
            jobs,
            state,
            faces: Mutex::new(HashMap::new()),
            stop,
        })
    }

    /// Pure lookup; ready bitmaps are retained so different iced pipelines
    /// can each upload them to separate GPU atlases.
    pub fn lookup(&self, key: CacheKey) -> GlyphLookup {
        match lock(&self.state).entries.get(&key) {
            None => GlyphLookup::Missing,
            Some(Entry::Pending) => GlyphLookup::Pending,
            Some(Entry::Ready(image)) => GlyphLookup::Ready(image.clone()),
            Some(Entry::Failed(error)) => GlyphLookup::Failed(Arc::clone(error)),
        }
    }

    /// Register a missing glyph and enqueue it. Never blocks on a full queue.
    /// `font_system` is used only to snapshot immutable face bytes on the FIRST
    /// request for each font ID; the worker has no reference to it.
    ///
    /// Callers must ensure `key.font_source` belongs to the same FontSystem.
    pub fn request(&self, key: CacheKey, font_system: &FontSystem) -> RequestStatus {
        {
            let mut state = lock(&self.state);
            match state.entries.get(&key) {
                Some(Entry::Pending) => return RequestStatus::AlreadyPending,
                Some(Entry::Ready(_)) => return RequestStatus::AlreadyReady,
                Some(Entry::Failed(_)) => return RequestStatus::Failed,
                None => {
                    // Register before releasing the lock. Simultaneous
                    // renderers cannot queue the same glyph twice.
                    state.entries.insert(key, Entry::Pending);
                }
            }
        }

        let font = match self.face_data(font_system, key.glyph.font_id) {
            Ok(face) => face,
            Err(message) => {
                lock(&self.state)
                    .entries
                    .insert(key, Entry::Failed(message));
                return RequestStatus::Failed;
            }
        };

        match self.jobs.try_send(GlyphJob { key, font }) {
            Ok(()) => RequestStatus::Queued,
            Err(TrySendError::Full(_)) => {
                // Not queued: remove Pending so a later frame can retry.
                lock(&self.state).entries.remove(&key);
                RequestStatus::QueueFull
            }
            Err(TrySendError::Disconnected(_)) => {
                lock(&self.state)
                    .entries
                    .insert(key, Entry::Failed("glyph worker disconnected".into()));
                RequestStatus::Failed
            }
        }
    }

    fn face_data(
        &self,
        font_system: &FontSystem,
        font_id: FontId,
    ) -> Result<Arc<FontFaceData>, Arc<str>> {
        let mut faces = lock(&self.faces);
        if let Some(face) = faces.get(&font_id) {
            return Ok(Arc::clone(face));
        }
        // The font bytes are copied once per face. This is the only potentially
        // expensive operation initiated by request(); do it during prewarm.
        let face = FontFaceData::load(font_system, font_id)
            .map(Arc::new)
            .map_err(|err| Arc::<str>::from(err.to_string()))?;
        faces.insert(font_id, Arc::clone(&face));
        Ok(face)
    }

    pub fn stats(&self) -> GlyphCacheStats {
        let state = lock(&self.state);
        let mut stats = GlyphCacheStats::default();
        for entry in state.entries.values() {
            match entry {
                Entry::Pending => stats.queued += 1,
                Entry::Ready(GlyphImage::Empty) => stats.ready += 1,
                Entry::Ready(GlyphImage::Bitmap(bitmap)) => {
                    stats.ready += 1;
                    stats.bytes += bitmap.byte_len();
                }
                Entry::Failed(_) => stats.failed += 1,
            }
        }
        stats
    }
}

impl Drop for GlyphGenerator {
    fn drop(&mut self) {
        // A blocked recv() wakes when the sender is dropped; the stop flag
        // prevents draining the full queue on application shutdown.
        self.stop.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_starts_and_reports_empty_cache() {
        let generator = GlyphGenerator::new().unwrap();
        assert_eq!(generator.stats().ready, 0);
        assert_eq!(generator.stats().queued, 0);
    }

    #[test]
    fn async_image_matches_original_cpu_generation() {
        use crate::{
            glyph::{CacheKey, GlyphImage, generate_glyph_sync},
            text::GlyphIdKey,
        };
        use std::time::{Duration, Instant};

        let fonts = FontSystem::new();
        let Some(font_id) = fonts.db().faces().next().map(|face| face.id) else {
            // Linux test hosts may have an empty font database.
            assert!(false, "Empty db");
            return;
        };
        let key = CacheKey::new(14, GlyphIdKey::new(font_id, 1, 48.), 48., 16);
        let expected = generate_glyph_sync(&fonts, key).unwrap();
        let generator = GlyphGenerator::new().unwrap();
        assert_eq!(generator.request(key, &fonts), RequestStatus::Queued);
        assert!(matches!(
            generator.request(key, &fonts),
            RequestStatus::AlreadyPending | RequestStatus::AlreadyReady
        ));

        let deadline = Instant::now() + Duration::from_secs(15);
        let actual = loop {
            match generator.lookup(key) {
                GlyphLookup::Ready(image) => break image,
                GlyphLookup::Failed(error) => panic!("worker failed: {error}"),
                GlyphLookup::Missing => panic!("worker lost a requested glyph"),
                GlyphLookup::Pending if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                GlyphLookup::Pending => panic!("worker timed out"),
            }
        };
        match (expected, actual) {
            (GlyphImage::Empty, GlyphImage::Empty) => {}
            (GlyphImage::Bitmap(a), GlyphImage::Bitmap(b)) => {
                assert_eq!(a.pixels, b.pixels);
                assert_eq!(a.bearing_x.to_bits(), b.bearing_x.to_bits());
                assert_eq!(a.bearing_y.to_bits(), b.bearing_y.to_bits());
            }
            _ => panic!("worker changed the glyph's outline/empty result"),
        }
    }
}
