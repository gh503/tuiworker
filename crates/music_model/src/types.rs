//! Core data types for the music player

use log;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Music source type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceType {
    Local,
    QqMusic,
    NetEaseMusic,
    Nas { mount_point: Option<PathBuf> },
}

impl SourceType {
    pub fn name(&self) -> &str {
        match self {
            SourceType::Local => "Local",
            SourceType::QqMusic => "QQ Music",
            SourceType::NetEaseMusic => "NetEase Music",
            SourceType::Nas { .. } => "NAS",
        }
    }
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Loading,
    Playing,
    Paused,
    Buffering,
}

impl PlaybackState {
    pub fn is_playing(&self) -> bool {
        matches!(self, PlaybackState::Playing)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, PlaybackState::Stopped)
    }
}

/// Playback mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackMode {
    Sequential,
    Random,
    RepeatOne,
    RepeatAll,
}

impl PlaybackMode {
    pub fn name(&self) -> &str {
        match self {
            PlaybackMode::Sequential => "Sequential",
            PlaybackMode::Random => "Random",
            PlaybackMode::RepeatOne => "Repeat One",
            PlaybackMode::RepeatAll => "Repeat All",
        }
    }

    pub fn is_repeat(&self) -> bool {
        matches!(self, PlaybackMode::RepeatOne | PlaybackMode::RepeatAll)
    }
}

/// Audio track metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<Duration>,
    pub source_type: SourceType,
    pub cover_url: Option<String>,
    pub parent: String,
    pub lyrics: Option<String>,
}

impl Track {
    pub fn new(
        id: String,
        path: PathBuf,
        title: String,
        artist: String,
        album: String,
        duration: Option<Duration>,
        source_type: SourceType,
    ) -> Self {
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Self {
            id,
            path,
            title,
            artist,
            album,
            duration,
            source_type,
            cover_url: None,
            parent,
            lyrics: None,
        }
    }

    pub fn local(path: PathBuf, title: String, artist: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let lyrics = Self::load_lyrics_from_file(&path);

        Self {
            id,
            path,
            title,
            artist,
            album: String::from("-"),
            duration: None,
            source_type: SourceType::Local,
            cover_url: None,
            parent,
            lyrics,
        }
    }

    fn load_lyrics_from_file(audio_path: &std::path::Path) -> Option<String> {
        let lyrics_path = audio_path.with_extension("lrc");

        log::debug!("[Track] Looking for lyrics: {}", lyrics_path.display());

        if let Ok(bytes) = std::fs::read(&lyrics_path) {
            log::info!(
                "[Track] Found lyrics file: {} ({} bytes)",
                lyrics_path.display(),
                bytes.len()
            );

            let encodings: Vec<(&str, &encoding_rs::Encoding)> = vec![
                ("UTF-8", encoding_rs::UTF_8),
                ("GBK", encoding_rs::GBK),
                ("GB18030", encoding_rs::GB18030),
                ("BIG5", encoding_rs::BIG5),
                ("WINDOWS-1252", encoding_rs::WINDOWS_1252),
            ];

            for encoding in encodings {
                let (decoder, _used_encoding, had_malformed) = encoding.1.decode(&bytes);
                if !had_malformed {
                    let text = decoder.to_string();
                    log::info!(
                        "[Track] Successfully decoded lyrics using {} ({} characters)",
                        encoding.0,
                        text.len()
                    );
                    return Some(text);
                }
            }

            log::warn!("[Track] Failed to decode lyrics with any known encoding");
        }

        let audio_str = audio_path.to_string_lossy();
        if let Some(pos) = audio_str.rfind('.') {
            let uppercase_path = format!("{}.LRC", &audio_str[..pos]);
            let upper_path = std::path::PathBuf::from(&uppercase_path);
            if let Ok(bytes) = std::fs::read(&upper_path) {
                log::info!(
                    "[Track] Found lyrics file (uppercase): {} ({} bytes)",
                    upper_path.display(),
                    bytes.len()
                );

                let encodings: Vec<(&str, &encoding_rs::Encoding)> = vec![
                    ("UTF-8", encoding_rs::UTF_8),
                    ("GBK", encoding_rs::GBK),
                    ("GB18030", encoding_rs::GB18030),
                    ("BIG5", encoding_rs::BIG5),
                    ("WINDOWS-1252", encoding_rs::WINDOWS_1252),
                ];

                for encoding in encodings {
                    let (decoder, _used_encoding, had_malformed) = encoding.1.decode(&bytes);
                    if !had_malformed {
                        let text = decoder.to_string();
                        log::info!("[Track] Successfully decoded uppercase lyrics using {} ({} characters)", encoding.0, text.len());
                        return Some(text);
                    }
                }

                log::warn!("[Track] Failed to decode uppercase lyrics with any known encoding");
            }
        }

        log::debug!("[Track] No lyrics file found for: {}", audio_path.display());
        None
    }

    pub fn netease(id: String, title: String, artist: String, album: String) -> Self {
        Self {
            id: id.clone(),
            path: PathBuf::from(format!("netease:{}", id)),
            title,
            artist,
            album,
            duration: None,
            source_type: SourceType::NetEaseMusic,
            cover_url: None,
            parent: String::from("NetEase Music"),
            lyrics: None,
        }
    }

    pub fn qqmusic(id: String, title: String, artist: String) -> Self {
        Self {
            id: id.clone(),
            path: PathBuf::from(format!("qqmusic:{}", id)),
            title,
            artist,
            album: String::from("-"),
            duration: None,
            source_type: SourceType::QqMusic,
            cover_url: None,
            parent: String::from("QQ Music"),
            lyrics: None,
        }
    }
}
