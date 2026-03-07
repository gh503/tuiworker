//! Music source abstraction

use crate::error::Result;
use crate::events::MusicEvent;
use crate::types::{PlaybackState, SourceType, Track};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

/// Music source trait
pub trait MusicSource {
    fn load(&mut self, track: &Track) -> Result<()>;
    fn play(&mut self) -> Result<()>;
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn seek(&mut self, position: Duration) -> Result<()>;
    fn get_position(&self) -> Duration;
    fn get_duration(&self) -> Option<Duration>;
    fn get_state(&self) -> PlaybackState;
    fn get_cover_art(&self, track: &Track) -> Option<Vec<u8>>;
    fn search(&self, query: &str) -> Result<Vec<Track>>;
    fn get_source_type(&self) -> SourceType;
    fn authenticate(&mut self, credentials: Option<&Credentials>) -> Result<()>;
    fn cleanup(&mut self);
    fn supports_streaming(&self) -> bool;

    fn set_event_dispatcher(&mut self, dispatcher: Arc<crate::events::EventDispatcher>);
    fn set_volume(&mut self, volume: f32);
}

/// Credentials for music sources
#[derive(Debug, Clone, Default)]
pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

/// Local file music source
pub struct LocalSource {
    current_track: Option<Track>,
    rodio_sink: Option<Arc<Mutex<rodio::Sink>>>,
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    state: PlaybackState,
    position: Duration,
    duration: Option<Duration>,
    dispatcher: Option<Arc<crate::events::EventDispatcher>>,
}

impl LocalSource {
    pub fn new() -> Self {
        Self {
            current_track: None,
            rodio_sink: None,
            _stream: None,
            stream_handle: None,
            state: PlaybackState::Stopped,
            position: Duration::default(),
            duration: None,
            dispatcher: None,
        }
    }

    fn initialize_audio_output(&mut self) -> Result<()> {
        if self.stream_handle.is_none() {
            let (stream, handle) = rodio::OutputStream::try_default()
                .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;
            self._stream = Some(stream);
            self.stream_handle = Some(handle);
        }
        Ok(())
    }

    fn dispatch_event(&self, event: MusicEvent) {
        if let Some(dispatcher) = &self.dispatcher {
            dispatcher.dispatch(event);
        }
    }
}

impl Default for LocalSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicSource for LocalSource {
    fn load(&mut self, track: &Track) -> Result<()> {
        self.state = PlaybackState::Loading;
        self.dispatch_event(MusicEvent::StateChanged(
            PlaybackState::Loading,
            Some(PlaybackState::Stopped),
        ));

        self.initialize_audio_output()?;

        let file = std::fs::File::open(&track.path)?;
        let handle = self
            .stream_handle
            .as_ref()
            .ok_or_else(|| crate::error::MusicError::Io("No audio output handle".to_string()))?;

        let source = rodio::Decoder::new(std::io::BufReader::new(file))
            .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;
        let sink = rodio::Sink::try_new(handle)
            .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;

        sink.append(source);
        sink.pause();

        self.rodio_sink = Some(Arc::new(Mutex::new(sink)));
        self.current_track = Some(track.clone());
        self.state = PlaybackState::Stopped;
        self.position = Duration::default();
        self.duration = track.duration;

        self.dispatch_event(MusicEvent::TrackChanged(track.clone()));

        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        if let Some(sink) = &self.rodio_sink {
            sink.lock().play();
            self.state = PlaybackState::Playing;
            self.dispatch_event(MusicEvent::StateChanged(
                PlaybackState::Playing,
                Some(PlaybackState::Paused),
            ));
        }
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if let Some(sink) = &self.rodio_sink {
            sink.lock().pause();
            self.state = PlaybackState::Paused;
            self.dispatch_event(MusicEvent::StateChanged(
                PlaybackState::Paused,
                Some(PlaybackState::Playing),
            ));
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        self.play()
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(sink) = &self.rodio_sink {
            sink.lock().stop();
        }
        self.current_track = None;
        self.rodio_sink = None;
        self.state = PlaybackState::Stopped;
        self.position = Duration::default();
        self.dispatch_event(MusicEvent::StateChanged(
            PlaybackState::Stopped,
            Some(PlaybackState::Playing),
        ));
        Ok(())
    }

    fn seek(&mut self, _position: Duration) -> Result<()> {
        Err(crate::error::MusicError::PlaybackFailed(
            "Seeking not supported for local files".to_string(),
        ))
    }

    fn get_position(&self) -> Duration {
        self.position
    }

    fn get_duration(&self) -> Option<Duration> {
        self.duration
    }

    fn get_state(&self) -> PlaybackState {
        self.state
    }

    fn get_cover_art(&self, _track: &Track) -> Option<Vec<u8>> {
        None
    }

    fn search(&self, _query: &str) -> Result<Vec<Track>> {
        Ok(Vec::new())
    }

    fn get_source_type(&self) -> SourceType {
        SourceType::Local
    }

    fn authenticate(&mut self, _credentials: Option<&Credentials>) -> Result<()> {
        Ok(())
    }

    fn cleanup(&mut self) {
        self.stop().ok();
        self.dispatcher.take();
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn set_event_dispatcher(&mut self, dispatcher: Arc<crate::events::EventDispatcher>) {
        self.dispatcher = Some(dispatcher);
    }

    fn set_volume(&mut self, volume: f32) {
        if let Some(sink) = &self.rodio_sink {
            sink.lock().set_volume(volume);
        }
    }
}
