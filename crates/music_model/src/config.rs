//! Configuration management for music player

use crate::types::SourceType;
use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Music player configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicConfig {
    pub default_source: String,

    #[serde(default)]
    pub local: LocalConfig,

    #[serde(default)]
    pub playback: PlaybackConfig,

    #[serde(default)]
    pub repeat: RepeatConfig,

    #[serde(default)]
    pub qq_music: QqMusicConfig,

    #[serde(default)]
    pub netease_music: NetEaseMusicConfig,

    #[serde(default)]
    pub nas: NasConfig,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            default_source: String::from("local"),
            local: LocalConfig::default(),
            playback: PlaybackConfig::default(),
            repeat: RepeatConfig::default(),
            qq_music: QqMusicConfig::default(),
            netease_music: NetEaseMusicConfig::default(),
            nas: NasConfig::default(),
        }
    }
}

/// Local music configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub music_directory: String,
    #[serde(default)]
    pub supported_formats: Vec<String>,
    #[serde(default = "default_true")]
    pub auto_refresh: bool,
}

impl Default for LocalConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir()
            .map(|p| p.join("Music").to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("~/Music"));

        Self {
            music_directory: home_dir,
            supported_formats: vec![
                String::from("mp3"),
                String::from("flac"),
                String::from("ogg"),
                String::from("wav"),
                String::from("m4a"),
            ],
            auto_refresh: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Playback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackConfig {
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default)]
    pub playback_mode: String,
    #[serde(default = "default_true")]
    pub auto_play_next: bool,
    #[serde(default = "default_crossfade_duration")]
    pub crossfade_duration: u64,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            volume: 0.75,
            playback_mode: String::from("sequential"),
            auto_play_next: true,
            crossfade_duration: 2000,
        }
    }
}

fn default_volume() -> f32 {
    0.75
}

fn default_crossfade_duration() -> u64 {
    2000
}

/// Repeat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatConfig {
    #[serde(default)]
    pub mode: String,
    #[serde(default = "default_true")]
    pub remember_position: bool,
}

impl Default for RepeatConfig {
    fn default() -> Self {
        Self {
            mode: String::from("all"),
            remember_position: true,
        }
    }
}

/// QQ Music configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QqMusicConfig {
    #[serde(default)]
    pub enabled: bool,
}

impl Default for QqMusicConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

/// NetEase Music configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetEaseMusicConfig {
    #[serde(default)]
    pub enabled: bool,
}

impl Default for NetEaseMusicConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

/// NAS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NasConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub local_mount_point: String,
    #[serde(default)]
    pub smb_url: String,
    #[serde(default)]
    pub webdav_url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

impl Default for NasConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            local_mount_point: String::from("/mnt/nas/music"),
            smb_url: String::from("smb://192.168.1.100/Music"),
            webdav_url: String::from("https://192.168.1.100/music/"),
            username: String::new(),
            password: String::new(),
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config_dir: PathBuf,
    config_path: PathBuf,
    config: MusicConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = config_dir()
            .map(|p| p.join("tuiworker"))
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?;

        let config_path = config_dir.join("music.toml");

        fs::create_dir_all(&config_dir)?;

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?
        } else {
            MusicConfig::default()
        };

        Ok(Self {
            config_dir,
            config_path,
            config,
        })
    }

    pub fn load(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            self.config = toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn get_config(&self) -> &MusicConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut MusicConfig {
        &mut self.config
    }

    pub fn update_config<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut MusicConfig),
    {
        f(&mut self.config);
        self.save()
    }

    pub fn get_default_source_type(&self) -> SourceType {
        match self.config.default_source.as_str() {
            "qq_music" => SourceType::QqMusic,
            "netease_music" => SourceType::NetEaseMusic,
            "nas" => SourceType::Nas {
                mount_point: if self.config.nas.local_mount_point.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&self.config.nas.local_mount_point))
                },
            },
            _ => SourceType::Local,
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}
