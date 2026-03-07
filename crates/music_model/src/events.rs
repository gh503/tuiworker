//! Event system for music player

use crate::types::{PlaybackMode, PlaybackState, SourceType, Track};
use parking_lot::RwLock;
use std::sync::Arc;

/// Music event type
#[derive(Debug, Clone)]
pub enum MusicEvent {
    TrackChanged(Track),
    StateChanged(PlaybackState, Option<PlaybackState>),
    ProgressUpdated {
        position: std::time::Duration,
        duration: Option<std::time::Duration>,
    },
    Error(crate::error::MusicError),
    SourceChanged(SourceType, SourceType),
    QueueChanged(Vec<Track>),
    VolumeChanged(f32, f32),
    ModeChanged(PlaybackMode, PlaybackMode),
}

/// Event listener trait
pub trait MusicEventListener: Send + Sync {
    fn on_event(&self, event: MusicEvent);
}

/// Event dispatcher for managing listeners and dispatching events
#[derive(Clone)]
pub struct EventDispatcher {
    listeners: Arc<RwLock<Vec<Box<dyn MusicEventListener>>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_listener(&self, listener: Box<dyn MusicEventListener>) {
        self.listeners.write().push(listener);
    }

    pub fn remove_listener(&self, index: usize) {
        let mut listeners = self.listeners.write();
        if index < listeners.len() {
            listeners.remove(index);
        }
    }

    pub fn dispatch(&self, event: MusicEvent) {
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener.on_event(event.clone());
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
