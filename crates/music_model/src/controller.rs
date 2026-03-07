//! Player controller for coordinating music playback

use crate::error::Result;
use crate::events::{EventDispatcher, MusicEvent};
use crate::progress::ProgressTracker;
use crate::queue::PlayQueue;
use crate::source::{Credentials, LocalSource, MusicSource};
use crate::types::{PlaybackMode, PlaybackState, SourceType, Track};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

/// Player controller for managing playback
pub struct PlayerController {
    queue: PlayQueue,
    current_source: Option<MusicSourceWrapper>,
    volume: Arc<Mutex<f32>>,
    state: Arc<Mutex<PlaybackState>>,
    progress_tracker: ProgressTracker,
    event_dispatcher: EventDispatcher,
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            queue: PlayQueue::new(),
            current_source: None,
            volume: Arc::new(Mutex::new(0.75)),
            state: Arc::new(Mutex::new(PlaybackState::Stopped)),
            progress_tracker: ProgressTracker::new(),
            event_dispatcher: EventDispatcher::new(),
        }
    }

    pub fn play_track(&mut self, index: usize) -> Result<()> {
        let track = self.queue.get_track(index).ok_or_else(|| {
            crate::error::MusicError::PlaybackFailed("Track not found".to_string())
        })?;

        self.queue.set_current_index(Some(index));
        self.load_and_play_track(track)
    }

    fn load_and_play_track(&mut self, track: Track) -> Result<()> {
        *self.state.lock() = PlaybackState::Loading;
        let old_state = PlaybackState::Stopped;
        self.event_dispatcher.dispatch(MusicEvent::StateChanged(
            PlaybackState::Loading,
            Some(old_state),
        ));

        if self.current_source.is_none() {
            self.current_source = Some(MusicSourceWrapper::Local(Box::new(LocalSource::new())));
            if let Some(source) = &mut self.current_source {
                let dispatcher = Arc::new(self.event_dispatcher.clone());
                source.set_event_dispatcher(dispatcher.clone());
            }
        }

        if let Some(source) = &mut self.current_source {
            source.load(&track)?;
            source.play()?;
        }

        *self.state.lock() = PlaybackState::Playing;
        self.event_dispatcher.dispatch(MusicEvent::StateChanged(
            PlaybackState::Playing,
            Some(PlaybackState::Loading),
        ));

        self.progress_tracker.start(track.duration);

        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        if let Some(source) = &mut self.current_source {
            source.pause()?;
        }
        self.progress_tracker.pause();
        *self.state.lock() = PlaybackState::Paused;
        self.event_dispatcher.dispatch(MusicEvent::StateChanged(
            PlaybackState::Paused,
            Some(PlaybackState::Playing),
        ));
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        if let Some(source) = &mut self.current_source {
            source.resume()?;
        }
        self.progress_tracker.resume();
        *self.state.lock() = PlaybackState::Playing;
        self.event_dispatcher.dispatch(MusicEvent::StateChanged(
            PlaybackState::Playing,
            Some(PlaybackState::Paused),
        ));
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(source) = &mut self.current_source {
            source.stop()?;
        }
        self.progress_tracker.stop();
        self.queue.set_current_index(None);
        *self.state.lock() = PlaybackState::Stopped;
        self.event_dispatcher.dispatch(MusicEvent::StateChanged(
            PlaybackState::Stopped,
            Some(*self.state.lock()),
        ));
        Ok(())
    }

    pub fn set_volume(&mut self, volume: f32) {
        let old_volume = *self.volume.lock();
        let new_volume = volume.clamp(0.0, 1.0);
        *self.volume.lock() = new_volume;

        if let Some(source) = &mut self.current_source {
            source.set_volume(new_volume);
        }

        self.event_dispatcher
            .dispatch(MusicEvent::VolumeChanged(new_volume, old_volume));
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock()
    }

    pub fn cycle_playback_mode(&mut self) {
        let old_mode = self.queue.get_mode();
        let new_mode = match old_mode {
            PlaybackMode::Sequential => PlaybackMode::Random,
            PlaybackMode::Random => PlaybackMode::RepeatOne,
            PlaybackMode::RepeatOne => PlaybackMode::RepeatAll,
            PlaybackMode::RepeatAll => PlaybackMode::Sequential,
        };

        self.queue.set_mode(new_mode);

        if new_mode == PlaybackMode::Random {
            self.queue.shuffle();
        }

        self.event_dispatcher
            .dispatch(MusicEvent::ModeChanged(new_mode, old_mode));
    }

    pub fn add_track(&mut self, track: Track) {
        let old_len = self.queue.len();
        self.queue.add_track(track);
        if self.queue.len() != old_len {
            self.event_dispatcher
                .dispatch(MusicEvent::QueueChanged(self.queue.get_tracks()));
        }
    }

    pub fn add_tracks(&mut self, tracks: Vec<Track>) {
        let old_len = self.queue.len();
        self.queue.add_tracks(tracks);
        if self.queue.len() != old_len {
            self.event_dispatcher
                .dispatch(MusicEvent::QueueChanged(self.queue.get_tracks()));
        }
    }

    pub fn remove_track(&mut self, index: usize) -> Result<()> {
        self.queue
            .remove_track(index)
            .map_err(|e| crate::error::MusicError::Unknown(e.to_string()))?;
        self.event_dispatcher
            .dispatch(MusicEvent::QueueChanged(self.queue.get_tracks()));
        Ok(())
    }

    pub fn clear_queue(&mut self) {
        let was_empty = self.queue.is_empty();
        self.queue.clear();
        if !was_empty {
            self.event_dispatcher
                .dispatch(MusicEvent::QueueChanged(Vec::new()));
        }
    }

    pub fn next_track(&mut self) {
        if let Some(index) = self.queue.next_track_index() {
            if let Some(_track) = self.queue.get_track(index) {
                self.queue.set_current_index(Some(index));
            }
        }
    }

    pub fn prev_track(&mut self) {
        if let Some(index) = self.queue.prev_track_index() {
            if let Some(_track) = self.queue.get_track(index) {
                self.queue.set_current_index(Some(index));
            }
        }
    }

    pub fn goto_track(&mut self, index: usize) -> Result<()> {
        self.play_track(index)
    }

    pub fn get_queue(&self) -> &PlayQueue {
        &self.queue
    }

    pub fn get_current_track(&self) -> Option<Track> {
        self.queue.get_current_track()
    }

    pub fn get_state(&self) -> PlaybackState {
        *self.state.lock()
    }

    pub fn add_listener(&mut self, listener: Box<dyn crate::events::MusicEventListener>) {
        self.event_dispatcher.add_listener(listener);
    }

    pub fn update(&mut self) {
        self.progress_tracker.update();

        let position = self.progress_tracker.get_position();
        let duration = self.progress_tracker.get_duration();

        self.event_dispatcher
            .dispatch(MusicEvent::ProgressUpdated { position, duration });
    }
}

impl Default for PlayerController {
    fn default() -> Self {
        Self::new()
    }
}

pub enum MusicSourceWrapper {
    Local(Box<LocalSource>),
    QqMusic(Box<crate::source::QqMusicSource>),
    NetEaseMusic(Box<crate::source::NetEaseMusicSource>),
    Nas(Box<crate::source::NasSource>),
}

impl MusicSourceWrapper {
    fn as_mut_source(&mut self) -> &mut dyn MusicSource {
        match self {
            MusicSourceWrapper::Local(source) => source.as_mut(),
            MusicSourceWrapper::QqMusic(source) => source.as_mut(),
            MusicSourceWrapper::NetEaseMusic(source) => source.as_mut(),
            MusicSourceWrapper::Nas(source) => source.as_mut(),
        }
    }
}

impl MusicSource for MusicSourceWrapper {
    fn load(&mut self, track: &Track) -> Result<()> {
        self.as_mut_source().load(track)
    }

    fn play(&mut self) -> Result<()> {
        self.as_mut_source().play()
    }

    fn pause(&mut self) -> Result<()> {
        self.as_mut_source().pause()
    }

    fn resume(&mut self) -> Result<()> {
        self.as_mut_source().resume()
    }

    fn stop(&mut self) -> Result<()> {
        self.as_mut_source().stop()
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        self.as_mut_source().seek(position)
    }

    fn get_position(&self) -> Duration {
        match self {
            MusicSourceWrapper::Local(source) => source.get_position(),
            MusicSourceWrapper::QqMusic(source) => source.get_position(),
            MusicSourceWrapper::NetEaseMusic(source) => source.get_position(),
            MusicSourceWrapper::Nas(source) => source.get_position(),
        }
    }

    fn get_duration(&self) -> Option<Duration> {
        match self {
            MusicSourceWrapper::Local(source) => source.get_duration(),
            MusicSourceWrapper::QqMusic(source) => source.get_duration(),
            MusicSourceWrapper::NetEaseMusic(source) => source.get_duration(),
            MusicSourceWrapper::Nas(source) => source.get_duration(),
        }
    }

    fn get_state(&self) -> PlaybackState {
        match self {
            MusicSourceWrapper::Local(source) => source.get_state(),
            MusicSourceWrapper::QqMusic(source) => source.get_state(),
            MusicSourceWrapper::NetEaseMusic(source) => source.get_state(),
            MusicSourceWrapper::Nas(source) => source.get_state(),
        }
    }

    fn get_cover_art(&self, track: &Track) -> Option<Vec<u8>> {
        match self {
            MusicSourceWrapper::Local(source) => source.get_cover_art(track),
            MusicSourceWrapper::QqMusic(source) => source.get_cover_art(track),
            MusicSourceWrapper::NetEaseMusic(source) => source.get_cover_art(track),
            MusicSourceWrapper::Nas(source) => source.get_cover_art(track),
        }
    }

    fn search(&self, query: &str) -> Result<Vec<Track>> {
        match self {
            MusicSourceWrapper::Local(source) => source.search(query),
            MusicSourceWrapper::QqMusic(source) => source.search(query),
            MusicSourceWrapper::NetEaseMusic(source) => source.search(query),
            MusicSourceWrapper::Nas(source) => source.search(query),
        }
    }

    fn get_source_type(&self) -> SourceType {
        match self {
            MusicSourceWrapper::Local(source) => source.get_source_type(),
            MusicSourceWrapper::QqMusic(source) => source.get_source_type(),
            MusicSourceWrapper::NetEaseMusic(source) => source.get_source_type(),
            MusicSourceWrapper::Nas(source) => source.get_source_type(),
        }
    }

    fn authenticate(&mut self, credentials: Option<&Credentials>) -> Result<()> {
        match self {
            MusicSourceWrapper::Local(source) => source.authenticate(credentials),
            MusicSourceWrapper::QqMusic(source) => source.authenticate(credentials),
            MusicSourceWrapper::NetEaseMusic(source) => source.authenticate(credentials),
            MusicSourceWrapper::Nas(source) => source.authenticate(credentials),
        }
    }

    fn cleanup(&mut self) {
        match self {
            MusicSourceWrapper::Local(source) => source.cleanup(),
            MusicSourceWrapper::QqMusic(source) => source.cleanup(),
            MusicSourceWrapper::NetEaseMusic(source) => source.cleanup(),
            MusicSourceWrapper::Nas(source) => source.cleanup(),
        }
    }

    fn supports_streaming(&self) -> bool {
        match self {
            MusicSourceWrapper::Local(source) => source.supports_streaming(),
            MusicSourceWrapper::QqMusic(source) => source.supports_streaming(),
            MusicSourceWrapper::NetEaseMusic(source) => source.supports_streaming(),
            MusicSourceWrapper::Nas(source) => source.supports_streaming(),
        }
    }

    fn set_event_dispatcher(&mut self, dispatcher: Arc<EventDispatcher>) {
        match self {
            MusicSourceWrapper::Local(source) => source.set_event_dispatcher(dispatcher),
            MusicSourceWrapper::QqMusic(source) => source.set_event_dispatcher(dispatcher),
            MusicSourceWrapper::NetEaseMusic(source) => source.set_event_dispatcher(dispatcher),
            MusicSourceWrapper::Nas(source) => source.set_event_dispatcher(dispatcher),
        }
    }

    fn set_volume(&mut self, volume: f32) {
        match self {
            MusicSourceWrapper::Local(source) => source.set_volume(volume),
            MusicSourceWrapper::QqMusic(source) => source.set_volume(volume),
            MusicSourceWrapper::NetEaseMusic(source) => source.set_volume(volume),
            MusicSourceWrapper::Nas(source) => source.set_volume(volume),
        }
    }
}
