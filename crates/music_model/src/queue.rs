//! Queue management for music tracks

use crate::types::{PlaybackMode, Track};
use parking_lot::Mutex;
use std::sync::Arc;

/// Playback queue
#[derive(Clone)]
pub struct PlayQueue {
    tracks: Arc<Mutex<Vec<Track>>>,
    current_index: Arc<Mutex<Option<usize>>>,
    mode: Arc<Mutex<PlaybackMode>>,
    history: Arc<Mutex<Vec<usize>>>,
    position: Arc<Mutex<usize>>,
}

impl PlayQueue {
    pub fn new() -> Self {
        Self {
            tracks: Arc::new(Mutex::new(Vec::new())),
            current_index: Arc::new(Mutex::new(None)),
            mode: Arc::new(Mutex::new(PlaybackMode::Sequential)),
            history: Arc::new(Mutex::new(Vec::new())),
            position: Arc::new(Mutex::new(0)),
        }
    }

    pub fn add_track(&self, track: Track) {
        let mut tracks = self.tracks.lock();
        tracks.push(track);
    }

    pub fn add_tracks(&self, mut tracks: Vec<Track>) {
        let mut queue = self.tracks.lock();
        queue.append(&mut tracks);
    }

    pub fn remove_track(&self, index: usize) -> anyhow::Result<()> {
        let mut tracks = self.tracks.lock();
        if index >= tracks.len() {
            anyhow::bail!("Index out of bounds");
        }
        tracks.remove(index);

        let mut current_index = self.current_index.lock();
        if let Some(idx) = *current_index {
            if idx == index {
                *current_index = None;
            } else if idx > index {
                *current_index = Some(idx - 1);
            }
        }

        Ok(())
    }

    pub fn clear(&self) {
        self.tracks.lock().clear();
        self.current_index.lock().take();
        self.history.lock().clear();
    }

    pub fn get_tracks(&self) -> Vec<Track> {
        self.tracks.lock().clone()
    }

    pub fn get_current_track(&self) -> Option<Track> {
        let index = *self.current_index.lock();
        let index = index?;
        self.tracks.lock().get(index).cloned()
    }

    pub fn get_track(&self, index: usize) -> Option<Track> {
        self.tracks.lock().get(index).cloned()
    }

    pub fn current_index(&self) -> Option<usize> {
        *self.current_index.lock()
    }

    pub fn set_current_index(&self, index: Option<usize>) {
        *self.current_index.lock() = index;
    }

    pub fn len(&self) -> usize {
        self.tracks.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.lock().is_empty()
    }

    pub fn get_mode(&self) -> PlaybackMode {
        *self.mode.lock()
    }

    pub fn set_mode(&self, mode: PlaybackMode) {
        *self.mode.lock() = mode;
    }

    pub fn position(&self) -> usize {
        *self.position.lock()
    }

    pub fn set_position(&self, position: usize) {
        let len = self.tracks.lock().len();
        *self.position.lock() = position.min(len.saturating_sub(1));
    }

    pub fn next_track_index(&self) -> Option<usize> {
        let tracks = self.tracks.lock();
        let mode = *self.mode.lock();
        let current_index = *self.current_index.lock();

        if tracks.is_empty() {
            return None;
        }

        match mode {
            PlaybackMode::Sequential => match current_index {
                Some(idx) if idx + 1 < tracks.len() => Some(idx + 1),
                _ => None,
            },
            PlaybackMode::RepeatAll => match current_index {
                Some(idx) if idx + 1 < tracks.len() => Some(idx + 1),
                Some(_) => Some(0),
                None => {
                    if !tracks.is_empty() {
                        Some(0)
                    } else {
                        None
                    }
                }
            },
            PlaybackMode::RepeatOne => current_index,
            PlaybackMode::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                let indices: Vec<usize> = (0..tracks.len()).collect();
                indices.choose(&mut rng).copied()
            }
        }
    }

    pub fn prev_track_index(&self) -> Option<usize> {
        let tracks = self.tracks.lock();
        let mode = *self.mode.lock();
        let current_index = *self.current_index.lock();

        if tracks.is_empty() {
            return None;
        }

        match mode {
            PlaybackMode::Sequential => match current_index {
                Some(idx) if idx > 0 => Some(idx - 1),
                _ => None,
            },
            PlaybackMode::RepeatAll => match current_index {
                Some(idx) if idx > 0 => Some(idx - 1),
                Some(_) => Some(tracks.len() - 1),
                None => {
                    if !tracks.is_empty() {
                        Some(0)
                    } else {
                        None
                    }
                }
            },
            PlaybackMode::RepeatOne => current_index,
            PlaybackMode::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                let indices: Vec<usize> = (0..tracks.len()).collect();
                indices.choose(&mut rng).copied()
            }
        }
    }

    pub fn get_history(&self) -> Vec<usize> {
        self.history.lock().clone()
    }

    pub fn shuffle(&self) {
        use rand::seq::SliceRandom;
        let mut tracks = self.tracks.lock();
        tracks.shuffle(&mut rand::thread_rng());
    }
}

impl Default for PlayQueue {
    fn default() -> Self {
        Self::new()
    }
}
