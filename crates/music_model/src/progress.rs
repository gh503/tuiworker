//! Progress tracking for music playback

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Progress tracker for playback
#[derive(Clone)]
pub struct ProgressTracker {
    position: Arc<Mutex<Duration>>,
    duration: Arc<Mutex<Option<Duration>>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    accumulated: Arc<Mutex<Duration>>,
    last_update: Arc<Mutex<Option<Instant>>>,
    is_playing: Arc<Mutex<bool>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            position: Arc::new(Mutex::new(Duration::default())),
            duration: Arc::new(Mutex::new(None)),
            start_time: Arc::new(Mutex::new(None)),
            accumulated: Arc::new(Mutex::new(Duration::default())),
            last_update: Arc::new(Mutex::new(None)),
            is_playing: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&self, duration: Option<Duration>) {
        *self.start_time.lock() = Some(Instant::now());
        *self.accumulated.lock() = Duration::default();
        *self.duration.lock() = duration;
        *self.position.lock() = Duration::default();
        *self.is_playing.lock() = true;
        *self.last_update.lock() = Some(Instant::now());
    }

    pub fn pause(&self) {
        if *self.is_playing.lock() {
            if let Some(start) = *self.start_time.lock() {
                let elapsed = start.elapsed();
                *self.accumulated.lock() += elapsed;
            }
            *self.is_playing.lock() = false;
            *self.last_update.lock() = Some(Instant::now());
        }
    }

    pub fn resume(&self) {
        if !*self.is_playing.lock() {
            *self.start_time.lock() = Some(Instant::now());
            *self.is_playing.lock() = true;
            *self.last_update.lock() = Some(Instant::now());
        }
    }

    pub fn stop(&self) {
        self.pause();
        *self.start_time.lock() = None;
        *self.accumulated.lock() = Duration::default();
        *self.position.lock() = Duration::default();
        *self.duration.lock() = None;
    }

    pub fn update(&self) {
        if *self.is_playing.lock() {
            if let Some(start) = *self.start_time.lock() {
                let elapsed = *self.accumulated.lock() + start.elapsed();
                *self.position.lock() = elapsed;
                *self.last_update.lock() = Some(Instant::now());
            }
        }
    }

    pub fn seek(&self, position: Duration) {
        *self.position.lock() = position;
        *self.accumulated.lock() = position;
        *self.start_time.lock() = Some(Instant::now());
        *self.last_update.lock() = Some(Instant::now());
    }

    pub fn get_position(&self) -> Duration {
        self.update();
        *self.position.lock()
    }

    pub fn get_duration(&self) -> Option<Duration> {
        *self.duration.lock()
    }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock()
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}
