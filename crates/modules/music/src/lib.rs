//! Music module - Audio playback with playlist management

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use rand::seq::SliceRandom;


use rodio::{OutputStream, OutputStreamHandle, Sink};

use ui::Theme;

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
};

/// Music track metadata
#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<Duration>,
}

impl Track {
    fn from_path(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Try to parse metadata from filename
        // Format: "Artist - Title.ext" or just "Title.ext"
        let (artist, title) = if let Some(pos) = filename.find(" - ") {
            let artist_part = &filename[..pos];
            let title_part = &filename[pos + 3..];
            let title_clean = title_part.trim_end_matches(".mp3")
                .trim_end_matches(".flac")
                .trim_end_matches(".ogg")
                .trim_end_matches(".wav")
                .trim_end_matches(".m4a")
                .to_string();
            (artist_part.to_string(), title_clean)
        } else {
            let title_clean = filename
                .trim_end_matches(".mp3")
                .trim_end_matches(".flac")
                .trim_end_matches(".ogg")
                .trim_end_matches(".wav")
                .trim_end_matches(".m4a")
                .to_string();
            ("Unknown".to_string(), title_clean)
        };

        Self {
            path,
            title,
            artist,
            album: "Unknown".to_string(),
            duration: None,
        }
    }
}

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Repeat mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    None,
    All,
    One,
}

/// Shuffle mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShuffleMode {
    Off,
    On,
}

/// Main music module
pub struct MusicModule {
    tracks: Vec<Track>,
    current_track_index: Option<usize>,
    playback_state: PlaybackState,
    volume: f32,
    repeat_mode: RepeatMode,
    shuffle_mode: ShuffleMode,
    selected_index: usize,
    music_dir: PathBuf,
    theme: Theme,

    // Audio components
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Arc<Mutex<Sink>>>,

    // Progress tracking
    track_progress: Duration,
    track_duration: Option<Duration>,
}

impl MusicModule {
    pub fn new(music_dir: PathBuf) -> Self {
        Self {
            tracks: Vec::new(),
            current_track_index: None,
            playback_state: PlaybackState::Stopped,
            volume: 0.7,
            repeat_mode: RepeatMode::None,
            shuffle_mode: ShuffleMode::Off,
            selected_index: 0,
            music_dir,
            theme: Theme::default(),
            _stream: None,
            stream_handle: None,
            sink: None,
            track_progress: Duration::default(),
            track_duration: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Load music from directory
    pub fn load_music(&mut self) -> anyhow::Result<()> {
        if !self.music_dir.exists() {
            return Ok(());
        }

        let supported_extensions = ["mp3", "flac", "ogg", "wav", "m4a"];
        let music_dir = &self.music_dir;
        self.tracks.clear();
        self.load_directory_recursive(music_dir, &supported_extensions)?;

        Ok(())
    }

    /// Recursively load directory
    fn load_directory_recursive(
        &mut self,
        dir: &Path,
        extensions: &[&str],
    ) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.load_directory_recursive(&path, extensions)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
                    let track = Track::from_path(path);
                    self.tracks.push(track);
                }
            }
        }

        Ok(())
    }

    /// Initialize audio output
    fn init_audio(&mut self) -> anyhow::Result<()> {
        if self.stream_handle.is_none() {
            let (stream, stream_handle) = OutputStream::try_default()?;
            self._stream = Some(stream);
            self.stream_handle = Some(stream_handle);

            let sink = Sink::try_new(&self.stream_handle.as_ref().unwrap())?;
            let sink = Arc::new(Mutex::new(sink));
            self.sink = Some(sink);
        }

        Ok(())
    }

    /// Play track
    pub fn play(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.tracks.len() {
            return Ok(());
        }

        self.init_audio()?;

        let track = self.tracks[index].clone();
        let file = std::fs::File::open(&track.path)?;

        if let Some(ref handle) = self.stream_handle {
            let decoder = rodio::Decoder::new(std::io::BufReader::new(file))?;

            if let Some(ref sink) = self.sink {
                let mut sink = sink.lock().unwrap();
                sink.stop();
                sink.append(decoder);
                sink.play();
                drop(sink);
            }

            self.current_track_index = Some(index);
            self.playback_state = PlaybackState::Playing;
            self.track_progress = Duration::default();
        }

        Ok(())
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.lock().unwrap().pause();
            self.playback_state = PlaybackState::Paused;
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.lock().unwrap().play();
            self.playback_state = PlaybackState::Playing;
        }
    }

    /// Stop playback
    pub fn stop(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.lock().unwrap().stop();
        }
        self.playback_state = PlaybackState::Stopped;
        self.current_track_index = None;
        self.track_progress = Duration::default();
    }

    /// Next track
    pub fn next_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let next_index = match self.current_track_index {
            Some(idx) => if idx + 1 < self.tracks.len() {
                idx + 1
            } else if self.repeat_mode == RepeatMode::All {
                0
            } else {
                return;
            },
            None => return,
        };

        let _ = self.play(next_index);
    }

    /// Previous track
    pub fn prev_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }

        let prev_index = match self.current_track_index {
            Some(idx) => if idx > 0 {
                idx - 1
            } else if self.repeat_mode == RepeatMode::All {
                self.tracks.len() - 1
            } else {
                return;
            },
            None => return,
        };

        let _ = self.play(prev_index);
    }

    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(ref sink) = self.sink {
            sink.lock().unwrap().set_volume(self.volume);
        }
    }

    /// Toggle repeat mode
    pub fn toggle_repeat(&mut self) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::None => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::None,
        };
    }

    /// Toggle shuffle mode
    pub fn toggle_shuffle(&mut self) {
        self.shuffle_mode = match self.shuffle_mode {
            ShuffleMode::Off => ShuffleMode::On,
            ShuffleMode::On => ShuffleMode::Off,
        };

        if self.shuffle_mode == ShuffleMode::On {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            self.tracks.shuffle(&mut rng);
        }
    }

    /// Navigate up
    pub fn navigate_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Navigate down
    pub fn navigate_down(&mut self) {
        self.selected_index = (self.selected_index + 1).min(self.tracks.len().saturating_sub(1));
    }

    /// Draw playlist
    fn draw_playlist(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    format!("播放列表 ({} 首)", self.tracks.len()),
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::default(),
        ];

        if self.tracks.is_empty() {
            lines.push(Line::from("没有找到音乐文件"));
            lines.push(Line::from(format!("音乐目录: {}", self.music_dir.display())));
        } else {
            let visible_count = area.height.saturating_sub(3) as usize / 2;
            let start = self.selected_index.saturating_sub(visible_count / 2);
            let end = (start + visible_count).min(self.tracks.len());

            for i in start..end {
                let track = &self.tracks[i];
                let is_current = self.current_track_index == Some(i);
                let is_selected = i == self.selected_index;

                let style = if is_selected {
                    Style::default().bg(self.theme.primary()).fg(Color::Black)
                } else if is_current {
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let line = format!(
                    "{} {} - {}",
                    if is_current { "►" } else { " " },
                    track.artist,
                    track.title
                );

                lines.push(Line::from(vec![Span::styled(line, style)]));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("播放列表")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw player controls
    fn draw_player(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![];

        // Current track info
        if let Some(idx) = self.current_track_index {
            if let Some(track) = self.tracks.get(idx) {
                lines.push(Line::from(vec![
                    Span::styled("正在播放: ", Style::default().fg(self.theme.muted())),
                    Span::styled(
                        format!("{} - {}", track.artist, track.title),
                        Style::default()
                            .fg(self.theme.primary())
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        } else {
            lines.push(Line::from("没有正在播放的曲目"));
        }

        lines.push(Line::default());

        // Playback controls
        let playback_status = match self.playback_state {
            PlaybackState::Playing => "播放中",
            PlaybackState::Paused => "已暂停",
            PlaybackState::Stopped => "已停止",
        };

        lines.push(Line::from(vec![
            Span::styled("状态: ", Style::default().fg(self.theme.muted())),
            Span::styled(playback_status, Style::default()),
            Span::styled("  |  ", Style::default()),
            Span::styled(
                format!("音量: {:.0}%", self.volume * 100.0),
                Style::default(),
            ),
            Span::styled("  |  ", Style::default()),
            Span::styled(
                match self.repeat_mode {
                    RepeatMode::None => "循环: 关",
                    RepeatMode::All => "循环: 全部",
                    RepeatMode::One => "循环: 单曲",
                },
                Style::default(),
            ),
            Span::styled("  |  ", Style::default()),
            Span::styled(
                if self.shuffle_mode == ShuffleMode::On {
                    "随机: 开"
                } else {
                    "随机: 关"
                },
                Style::default(),
            ),
        ]));

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("播放器")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw volume gauge
    fn draw_volume(&self, frame: &mut Frame, area: Rect) {
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title("音量")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .gauge_style(
                Style::default()
                    .fg(self.theme.primary())
                    .bg(self.theme.surface()),
            )
            .percent((self.volume * 100.0) as u16)
            .label(format!("{:.0}%", self.volume * 100.0));

        frame.render_widget(gauge, area);
    }

    /// Draw help bar
    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help = "Space:播放/停止 <:音量- >:音量+ r:切换循环 s:切换随机 n:下一首 p:上一首 j/k:导航 Enter:播放选中";
        let paragraph = Paragraph::new(help)
            .style(Style::default().fg(self.theme.muted()));
        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(' ') => {
                match self.playback_state {
                    PlaybackState::Playing => self.pause(),
                    PlaybackState::Paused => self.resume(),
                    PlaybackState::Stopped => {
                        if !self.tracks.is_empty() {
                            let _ = self.play(self.selected_index);
                        }
                    }
                }
                Action::None
            }
            KeyCode::Char('>') => {
                self.set_volume(self.volume + 0.1);
                Action::None
            }
            KeyCode::Char('<') => {
                self.set_volume(self.volume - 0.1);
                Action::None
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.toggle_repeat();
                Action::None
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.toggle_shuffle();
                Action::None
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.next_track();
                Action::None
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.prev_track();
                Action::None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
                Action::None
            }
            KeyCode::Enter => {
                if !self.tracks.is_empty() {
                    let _ = self.play(self.selected_index);
                }
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.stop();
                Action::Quit
            }
            _ => Action::None,
        }
    }
}

impl CoreModule for MusicModule {
    fn name(&self) -> &str {
        "music"
    }

    fn title(&self) -> &str {
        "音乐"
    }

    fn update(&mut self, event: CrosstermEvent) -> Action {
        match event {
            CrosstermEvent::Key(key) => self.handle_key_event(key),
            _ => Action::None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Player
                Constraint::Min(5),    // Playlist
                Constraint::Length(1), // Volume
                Constraint::Length(1), // Help
            ])
            .split(area);

        self.draw_player(frame, layout[0]);
        self.draw_playlist(frame, layout[1]);
        self.draw_volume(frame, layout[2]);
        self.draw_help_bar(frame, layout[3]);
    }

    fn save(&self) -> anyhow::Result<()> { Ok(()) }

    fn load(&mut self) -> anyhow::Result<()> {
        self.load_music()?;
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "Space",
                description: "播放/停止",
            },
            Shortcut {
                key: "</>",
                description: "音量",
            },
            Shortcut {
                key: "r",
                description: "切换循环",
            },
            Shortcut {
                key: "s",
                description: "切换随机",
            },
            Shortcut {
                key: "n/p",
                description: "上/下一首",
            },
            Shortcut {
                key: "j/k",
                description: "上下导航",
            },
            Shortcut {
                key: "Enter",
                description: "播放选中",
            },
        ]
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.load_music()?;
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.stop();
        Ok(())
    }
}

impl Drop for MusicModule {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

pub use MusicModule as Music;
