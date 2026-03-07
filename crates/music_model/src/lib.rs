//! Music player core model library
//!
//! This crate provides the core business logic and abstractions for the music player,
//! including the MusicSource trait, player controller, queue management, and event system.

pub mod config;
pub mod controller;
pub mod error;
pub mod events;
pub mod progress;
pub mod queue;
pub mod source;
pub mod types;

#[cfg(test)]
mod types_test;

pub use config::{ConfigManager, MusicConfig};
pub use controller::PlayerController;
pub use error::{MusicError, Result};
pub use events::{EventDispatcher, MusicEvent, MusicEventListener};
pub use progress::ProgressTracker;
pub use queue::PlayQueue;
pub use source::{
    Credentials, LocalSource, MusicSource, NasConfig, NasProtocol, NasSource, NetEaseMusicSource,
    QqMusicSource,
};
pub use types::{PlaybackMode, PlaybackState, SourceType, Track};
