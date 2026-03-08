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
        // Use mount_point as the NAS identifier
        // For now, we'll use a default since mount_point is a PathBuf
        // TODO: Store actual mount point path when configuring NAS
        SourceType::Nas { mount_point: None }
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
            let _ = sink.lock().set_volume(volume);
        }
    }
}

/// QQ Music source
pub struct QqMusicSource {
    state: PlaybackState,
    position: Duration,
    duration: Option<Duration>,
    dispatcher: Option<Arc<crate::events::EventDispatcher>>,
    credentials: Option<Credentials>,
    api_base_url: String,
    current_audio_url: Option<String>,
    rodio_sink: Option<Arc<Mutex<rodio::Sink>>>,
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
}

impl QqMusicSource {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: Duration::default(),
            duration: None,
            dispatcher: None,
            credentials: None,
            api_base_url: "https://y.qq.com/api".to_string(),
            current_audio_url: None,
            rodio_sink: None,
            _stream: None,
            stream_handle: None,
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

    fn search_qq_music_sync(&self, query: &str, limit: u32) -> Result<Vec<Track>> {
        let url = format!(
            "{}/search/w/{}/{}/1/{}",
            self.api_base_url.trim_end_matches('/'),
            query,
            1,
            limit
        );

        let client = reqwest::blocking::Client::new();
        let response = client.get(&url).send().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("QQ Music search failed: {}", e))
        })?;

        let json: serde_json::Value = response.json().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("JSON parse failed: {}", e))
        })?;

        let mut tracks = Vec::new();

        if let Some(data) = json["data"]["song"]["list"].as_array() {
            for song in data.iter().take(limit as usize) {
                let id = song["songmid"].as_str().unwrap_or("");
                let title = song["songname"].as_str().unwrap_or("Unknown");
                let artist = song["singer"][0]["name"].as_str().unwrap_or("Unknown");

                let track = Track::qqmusic(id.to_string(), title.to_string(), artist.to_string());
                tracks.push(track);
            }
        }

        Ok(tracks)
    }

    fn get_song_url_sync(&self, songmid: &str) -> Result<String> {
        let url = format!(
            "{}/song/url?id={}&br=128",
            self.api_base_url.trim_end_matches('/'),
            songmid
        );

        let client = reqwest::blocking::Client::new();
        let response = client.get(&url).send().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("QQ Music URL fetch failed: {}", e))
        })?;

        let json: serde_json::Value = response.json().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("JSON parse failed: {}", e))
        })?;

        json["data"][0]["url"]
            .as_str()
            .ok_or_else(|| {
                crate::error::MusicError::PlaybackFailed("No song URL found".to_string())
            })
            .map(|s| s.to_string())
    }

    fn fetch_lyrics_sync(&self, songmid: &str) -> Result<String> {
        let url = format!(
            "{}/lyric?id={}",
            self.api_base_url.trim_end_matches('/'),
            songmid
        );

        let client = reqwest::blocking::Client::new();
        let response = client.get(&url).send().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("QQ Music lyrics fetch failed: {}", e))
        })?;

        let json: serde_json::Value = response.json().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("JSON parse failed: {}", e))
        })?;

        if let Some(lyric) = json["lyric"].as_str() {
            Ok(lyric.to_string())
        } else {
            Ok("".to_string())
        }
    }
}

impl Default for QqMusicSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicSource for QqMusicSource {
    fn load(&mut self, track: &Track) -> Result<()> {
        self.state = PlaybackState::Loading;
        self.dispatch_event(MusicEvent::StateChanged(
            PlaybackState::Loading,
            Some(PlaybackState::Stopped),
        ));

        self.initialize_audio_output()?;

        let song_id = track.id.clone();
        log::info!("[QQ Music] Loading song: {}", track.title);

        let audio_url = match self.get_song_url_sync(&song_id) {
            Ok(url) => url,
            Err(e) => {
                log::error!("[QQ Music] Failed to get song URL: {:?}", e);
                self.state = PlaybackState::Stopped;
                return Err(e);
            }
        };

        self.current_audio_url = Some(audio_url.clone());
        log::info!("[QQ Music] Got audio URL: {}", audio_url);

        let handle = self
            .stream_handle
            .as_ref()
            .ok_or_else(|| crate::error::MusicError::Io("No audio output handle".to_string()))?;

        let client = reqwest::blocking::Client::new();
        let response = client.get(&audio_url).send().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("HTTP request failed: {}", e))
        })?;

        let bytes = response
            .bytes()
            .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;

        log::info!("[QQ Music] Downloaded {} bytes", bytes.len());

        let cursor = std::io::Cursor::new(bytes.to_vec());
        let source = rodio::Decoder::new(std::io::BufReader::new(cursor)).map_err(
            |e: rodio::decoder::DecoderError| {
                crate::error::MusicError::PlaybackFailed(e.to_string())
            },
        )?;

        let sink = rodio::Sink::try_new(handle)
            .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;

        sink.append(source);
        sink.pause();

        self.rodio_sink = Some(Arc::new(Mutex::new(sink)));
        self.state = PlaybackState::Stopped;
        self.position = Duration::default();
        self.duration = track.duration;

        let song_id = track.id.clone();
        let lyrics_text = self.fetch_lyrics_sync(&song_id);

        log::info!("[QQ Music] Song loaded successfully: {}", track.title);

        let mut track_with_lyrics = track.clone();
        if let Ok(lyrics) = lyrics_text {
            log::info!("[QQ Music] Loaded {} characters of lyrics", lyrics.len());
            track_with_lyrics.lyrics = Some(lyrics);
        }

        self.dispatch_event(MusicEvent::TrackChanged(track_with_lyrics));
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
        self.rodio_sink = None;
        self.current_audio_url = None;
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
            "Seeking not supported for streamed audio".to_string(),
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

    fn search(&self, query: &str) -> Result<Vec<Track>> {
        log::info!("[QQ Music] Searching for: {}", query);
        match self.search_qq_music_sync(query, 20) {
            Ok(tracks) => {
                log::info!("[QQ Music] Found {} tracks", tracks.len());
                Ok(tracks)
            }
            Err(e) => {
                log::error!("[QQ Music] Search failed: {:?}", e);
                Ok(Vec::new())
            }
        }
    }

    fn get_source_type(&self) -> SourceType {
        SourceType::QqMusic
    }

    fn authenticate(&mut self, credentials: Option<&Credentials>) -> Result<()> {
        self.credentials = credentials.cloned();
        Ok(())
    }

    fn cleanup(&mut self) {
        self.stop().ok();
        self.dispatcher.take();
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn set_event_dispatcher(&mut self, dispatcher: Arc<crate::events::EventDispatcher>) {
        self.dispatcher = Some(dispatcher);
    }

    fn set_volume(&mut self, volume: f32) {
        if let Some(sink) = &self.rodio_sink {
            let _ = sink.lock().set_volume(volume);
        }
    }
}

/// NetEase Cloud Music source

/// NetEase Cloud Music source
pub struct NetEaseMusicSource {
    state: PlaybackState,
    position: Duration,
    duration: Option<Duration>,
    dispatcher: Option<Arc<crate::events::EventDispatcher>>,
    credentials: Option<Credentials>,
    api_base_url: String,
    current_audio_url: Option<String>,
    rodio_sink: Option<Arc<Mutex<rodio::Sink>>>,
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
}

impl NetEaseMusicSource {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: Duration::default(),
            duration: None,
            dispatcher: None,
            credentials: None,
            api_base_url: "https://music.163.com/api".to_string(),
            current_audio_url: None,
            rodio_sink: None,
            _stream: None,
            stream_handle: None,
        }
    }

    pub fn with_api_base(mut self, url: String) -> Self {
        self.api_base_url = url;
        self
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

    fn fetch_audio_url_sync(&self, track_id: &str) -> Result<String> {
        let url = format!("{}/song/url?id={}&br=320000", self.api_base_url, track_id);

        let response = reqwest::blocking::get(&url).map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("HTTP request failed: {}", e))
        })?;

        let json: serde_json::Value = response.json().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("JSON parse failed: {}", e))
        })?;

        Ok(json["data"][0]["url"]
            .as_str()
            .ok_or_else(|| {
                crate::error::MusicError::PlaybackFailed("No audio URL found".to_string())
            })?
            .to_string())
    }

    fn fetch_lyrics_sync(&self, track_id: &str) -> Result<String> {
        let url = format!("{}/lyric?id={}", self.api_base_url, track_id);

        let response = reqwest::blocking::get(&url).map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("HTTP request failed: {}", e))
        })?;

        let json: serde_json::Value = response.json().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("JSON parse failed: {}", e))
        })?;

        Ok(json["lrc"]["lyric"]
            .as_str()
            .ok_or_else(|| crate::error::MusicError::PlaybackFailed("No lyrics found".to_string()))?
            .to_string())
    }

    fn search_netease_sync(&self, query: &str, limit: u32) -> Result<Vec<Track>> {
        let url = format!(
            "{}/search?keywords={}&limit={}&type=1",
            self.api_base_url.trim_end_matches('/'),
            query,
            limit
        );

        let client = reqwest::blocking::Client::new();
        let response = client.get(&url).send().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("NetEase Music search failed: {}", e))
        })?;

        let json: serde_json::Value = response.json().map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("JSON parse failed: {}", e))
        })?;

        let mut tracks = Vec::new();

        if let Some(result) = json["result"]["songs"].as_array() {
            for song in result.iter().take(limit as usize) {
                let id = song["id"]
                    .as_u64()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                let title = song["name"].as_str().unwrap_or("Unknown");
                let artist = song["artists"][0]["name"].as_str().unwrap_or("Unknown");
                let album = song["album"]["name"].as_str().unwrap_or("");

                let track =
                    Track::netease(id, title.to_string(), artist.to_string(), album.to_string());
                tracks.push(track);
            }
        }

        log::info!("[NetEase Music] Search returned {} tracks", tracks.len());
        Ok(tracks)
    }
}

impl Default for NetEaseMusicSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicSource for NetEaseMusicSource {
    fn load(&mut self, track: &Track) -> Result<()> {
        self.state = PlaybackState::Loading;
        self.dispatch_event(MusicEvent::StateChanged(
            PlaybackState::Loading,
            Some(PlaybackState::Stopped),
        ));

        self.initialize_audio_output()?;

        let track_id = track.id.clone();
        let audio_url = self.fetch_audio_url_sync(&track_id)?;
        self.current_audio_url = Some(audio_url.clone());

        let lyrics_text = self.fetch_lyrics_sync(&track_id);

        let handle = self
            .stream_handle
            .as_ref()
            .ok_or_else(|| crate::error::MusicError::Io("No audio output handle".to_string()))?;

        let response = reqwest::blocking::get(&audio_url).map_err(|e| {
            crate::error::MusicError::PlaybackFailed(format!("HTTP request failed: {}", e))
        })?;

        let bytes = response
            .bytes()
            .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;

        let cursor = std::io::Cursor::new(bytes.to_vec());
        let source = rodio::Decoder::new(std::io::BufReader::new(cursor)).map_err(
            |e: rodio::decoder::DecoderError| {
                crate::error::MusicError::PlaybackFailed(e.to_string())
            },
        )?;

        let sink = rodio::Sink::try_new(handle)
            .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;

        sink.append(source);
        sink.pause();

        self.rodio_sink = Some(Arc::new(Mutex::new(sink)));
        self.state = PlaybackState::Stopped;
        self.position = Duration::default();
        self.duration = track.duration;

        let mut track_with_lyrics = track.clone();
        if let Ok(lyrics) = lyrics_text {
            track_with_lyrics.lyrics = Some(lyrics);
        }
        self.dispatch_event(MusicEvent::TrackChanged(track_with_lyrics));

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
        self.rodio_sink = None;
        self.current_audio_url = None;
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
            "Seeking not supported for streamed audio".to_string(),
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

    fn search(&self, query: &str) -> Result<Vec<Track>> {
        log::info!("[NetEase Music] Searching for: {}", query);
        match self.search_netease_sync(query, 30) {
            Ok(tracks) => {
                log::info!("[NetEase Music] Found {} tracks", tracks.len());
                Ok(tracks)
            }
            Err(e) => {
                log::error!("[NetEase Music] Search failed: {:?}", e);
                Ok(Vec::new())
            }
        }
    }

    fn get_source_type(&self) -> SourceType {
        SourceType::NetEaseMusic
    }

    fn authenticate(&mut self, credentials: Option<&Credentials>) -> Result<()> {
        self.credentials = credentials.cloned();
        Ok(())
    }

    fn cleanup(&mut self) {
        self.stop().ok();
        self.dispatcher.take();
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn set_event_dispatcher(&mut self, dispatcher: Arc<crate::events::EventDispatcher>) {
        self.dispatcher = Some(dispatcher);
    }

    fn set_volume(&mut self, volume: f32) {
        if let Some(sink) = &self.rodio_sink {
            let _ = sink.lock().set_volume(volume);
        }
    }
}

/// NAS network source
///
/// Supports:
/// - SMB2/3 protocol via `smb` crate (feature: nas-smb)
/// - WebDAV protocol via `reqwest_dav` or `webdav-request` (feature: nas-webdav)
///
/// Implementation requires adding optional dependencies to Cargo.toml:
/// ```toml
/// [dependencies]
/// reqwest = { version = "0.12", optional = true }
/// smb = { version = "0.4", optional = true }
///
/// [features]
/// nas-webdav = ["reqwest"]
/// nas-smb = ["smb"]
/// ```
pub struct NasSource {
    state: PlaybackState,
    position: Duration,
    duration: Option<Duration>,
    dispatcher: Option<Arc<crate::events::EventDispatcher>>,
    credentials: Option<Credentials>,
    nas_config: Option<NasConfig>,
    rodio_sink: Option<Arc<Mutex<rodio::Sink>>>,
    _stream: Option<rodio::OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
}

/// NAS configuration
#[derive(Debug, Clone)]
pub struct NasConfig {
    pub address: String,
    pub protocol: NasProtocol,
    pub share_path: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// NAS protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NasProtocol {
    Smb,
    WebDav,
}

impl NasSource {
    pub fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: Duration::default(),
            duration: None,
            dispatcher: None,
            credentials: None,
            nas_config: None,
            rodio_sink: None,
            _stream: None,
            stream_handle: None,
        }
    }

    pub fn with_config(config: NasConfig) -> Self {
        Self {
            nas_config: Some(config),
            ..Self::new()
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

impl Default for NasSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicSource for NasSource {
    fn load(&mut self, track: &Track) -> Result<()> {
        self.state = PlaybackState::Loading;
        self.dispatch_event(MusicEvent::StateChanged(
            PlaybackState::Loading,
            Some(PlaybackState::Stopped),
        ));

        // TODO: Implement NAS SMB/WebDAV client integration
        // For now, treat as local file if path is absolute
        if track.path.is_absolute() && std::path::Path::new(&track.path).exists() {
            self.initialize_audio_output()?;

            let file = std::fs::File::open(&track.path)?;
            let handle = self.stream_handle.as_ref().ok_or_else(|| {
                crate::error::MusicError::Io("No audio output handle".to_string())
            })?;

            let source = rodio::Decoder::new(std::io::BufReader::new(file))
                .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;
            let sink = rodio::Sink::try_new(handle)
                .map_err(|e| crate::error::MusicError::PlaybackFailed(e.to_string()))?;

            sink.append(source);
            sink.pause();

            self.rodio_sink = Some(Arc::new(Mutex::new(sink)));
            self.state = PlaybackState::Stopped;
            self.position = Duration::default();
            self.duration = track.duration;

            self.dispatch_event(MusicEvent::TrackChanged(track.clone()));

            Ok(())
        } else {
            Err(crate::error::MusicError::PlaybackFailed(
                "NAS SMB/WebDAV integration not yet implemented - only local absolute paths are supported".to_string(),
            ))
        }
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
            "Seeking not supported for NAS files".to_string(),
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
        // TODO: Implement NAS file search via SMB/WebDAV
        Ok(Vec::new())
    }

    fn get_source_type(&self) -> SourceType {
        SourceType::Nas { mount_point: None }
    }

    fn authenticate(&mut self, credentials: Option<&Credentials>) -> Result<()> {
        // TODO: Implement NAS authentication (SMB/WebDAV)
        self.credentials = credentials.cloned();
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
            let _ = sink.lock().set_volume(volume);
        }
    }
}
