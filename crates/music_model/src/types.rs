//! Core data types for the music player

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
        }
    }
}
