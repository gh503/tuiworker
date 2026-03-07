//! Music module - Audio playback with playlist management

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use std::path::PathBuf;

use ui::Theme;

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
};

use music_model::{
    MusicEvent, MusicEventListener, PlaybackMode, PlaybackState, PlayerController, Track,
};

/// UI event listener for music player
pub struct UIEventListener {
    module_id: String,
}

impl UIEventListener {
    pub fn new(module_id: String) -> Self {
        Self { module_id }
    }
}

impl MusicEventListener for UIEventListener {
    fn on_event(&self, event: MusicEvent) {
        match event {
            MusicEvent::StateChanged(new_state, _) => {
                log::debug!("[{}] State changed: {:?}", self.module_id, new_state);
            }
            MusicEvent::ProgressUpdated { position, duration } => {
                log::debug!(
                    "[{}] Progress: {:?}/{:?}",
                    self.module_id,
                    position,
                    duration
                );
            }
            _ => {}
        }
    }
}

/// Main music module
pub struct MusicModule {
    controller: PlayerController,
    music_dir: PathBuf,
    theme: Theme,
    selected_index: usize,
}

impl MusicModule {
    pub fn new(music_dir: PathBuf) -> Self {
        let mut controller = PlayerController::new();

        let listener = Box::new(UIEventListener::new("music".to_string()));
        controller.add_listener(listener);

        Self {
            controller,
            music_dir,
            theme: Theme::default(),
            selected_index: 0,
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
        let music_dir = self.music_dir.clone();
        self.load_directory_recursive(&music_dir, &supported_extensions)?;

        Ok(())
    }

    /// Recursively load directory
    fn load_directory_recursive(
        &mut self,
        dir: &PathBuf,
        extensions: &[&str],
    ) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.load_directory_recursive(&path, extensions)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
                    let filename = path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let title = filename
                        .trim_end_matches(".mp3")
                        .trim_end_matches(".flac")
                        .trim_end_matches(".ogg")
                        .trim_end_matches(".wav")
                        .trim_end_matches(".m4a")
                        .to_string();

                    let artist = String::from("Unknown");

                    let track = Track::local(path, title, artist);
                    self.controller.add_track(track);
                }
            }
        }

        Ok(())
    }

    /// Play track
    pub fn play(&mut self, index: usize) {
        if let Err(e) = self.controller.play_track(index) {
            log::error!("Failed to play track: {}", e);
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if let Err(e) = self.controller.pause() {
            log::error!("Failed to pause: {}", e);
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if let Err(e) = self.controller.resume() {
            log::error!("Failed to resume: {}", e);
        }
    }

    /// Stop playback
    pub fn stop(&mut self) {
        if let Err(e) = self.controller.stop() {
            log::error!("Failed to stop: {}", e);
        }
    }

    /// Next track
    pub fn next_track(&mut self) {
        self.controller.next_track();
        if let Some(index) = self.controller.get_queue().current_index() {
            self.play(index);
        }
    }

    /// Previous track
    pub fn prev_track(&mut self) {
        self.controller.prev_track();
        if let Some(index) = self.controller.get_queue().current_index() {
            self.play(index);
        }
    }

    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.controller.set_volume(volume);
    }

    /// Toggle repeat mode
    pub fn toggle_repeat(&mut self) {
        self.controller.cycle_playback_mode();
    }

    /// Navigate up
    pub fn navigate_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Navigate down
    pub fn navigate_down(&mut self) {
        self.selected_index =
            (self.selected_index + 1).min(self.controller.get_queue().len().saturating_sub(1));
    }

    /// Draw playlist
    fn draw_playlist(&self, frame: &mut Frame, area: Rect) {
        let tracks = self.controller.get_queue().get_tracks();
        let current_index = self.controller.get_queue().current_index();

        let mut lines = vec![
            Line::from(vec![Span::styled(
                format!("播放列表 ({} 首)", tracks.len()),
                Style::default()
                    .fg(self.theme.primary())
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
        ];

        if tracks.is_empty() {
            lines.push(Line::from("没有找到音乐文件"));
            lines.push(Line::from(format!(
                "音乐目录: {}",
                self.music_dir.display()
            )));
        } else {
            let visible_count = area.height.saturating_sub(3) as usize / 2;
            let start = self.selected_index.saturating_sub(visible_count / 2);
            let end = (start + visible_count).min(tracks.len());

            for i in start..end {
                let track = &tracks[i];
                let is_current = current_index == Some(i);
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

        let current_track = self.controller.get_current_track();
        let state = self.controller.get_state();

        if let Some(track) = current_track {
            lines.push(Line::from(vec![
                Span::styled("正在播放: ", Style::default().fg(self.theme.muted())),
                Span::styled(
                    format!("{} - {}", track.artist, track.title),
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from("没有正在播放的曲目"));
        }

        lines.push(Line::default());

        let playback_status = match state {
            PlaybackState::Playing => "播放中",
            PlaybackState::Paused => "已暂停",
            PlaybackState::Stopped => "已停止",
            PlaybackState::Loading => "加载中",
            PlaybackState::Buffering => "缓冲中",
        };

        let mode = self.controller.get_queue().get_mode();

        lines.push(Line::from(vec![
            Span::styled("状态: ", Style::default().fg(self.theme.muted())),
            Span::styled(playback_status, Style::default()),
            Span::styled("  |  ", Style::default()),
            Span::styled(
                format!("音量: {:.0}%", self.controller.get_volume() * 100.0),
                Style::default(),
            ),
            Span::styled("  |  ", Style::default()),
            Span::styled(
                match mode {
                    PlaybackMode::Sequential => "顺序",
                    PlaybackMode::Random => "随机",
                    PlaybackMode::RepeatOne => "单曲循环",
                    PlaybackMode::RepeatAll => "列表循环",
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
            .percent((self.controller.get_volume() * 100.0) as u16)
            .label(format!("{:.0}%", self.controller.get_volume() * 100.0));

        frame.render_widget(gauge, area);
    }

    /// Draw help bar
    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help =
            "Space:播放/停止 <:音量- >:音量+ r:切换循环 n:下一首 p:上一首 j/k:导航 Enter:播放选中";
        let paragraph = Paragraph::new(help).style(Style::default().fg(self.theme.muted()));
        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(' ') => {
                match self.controller.get_state() {
                    PlaybackState::Playing => self.pause(),
                    PlaybackState::Paused => self.resume(),
                    _ => {
                        if !self.controller.get_queue().is_empty() {
                            self.play(self.selected_index);
                        }
                    }
                }
                Action::None
            }
            KeyCode::Char('>') => {
                self.set_volume(self.controller.get_volume() + 0.1);
                Action::None
            }
            KeyCode::Char('<') => {
                self.set_volume(self.controller.get_volume() - 0.1);
                Action::None
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.toggle_repeat();
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
                if !self.controller.get_queue().is_empty() {
                    self.play(self.selected_index);
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
                Constraint::Length(5), // Player
                Constraint::Min(5),    // Playlist
                Constraint::Length(1), // Volume
                Constraint::Length(1), // Help
            ])
            .split(area);

        self.controller.update();
        self.draw_player(frame, layout[0]);
        self.draw_playlist(frame, layout[1]);
        self.draw_volume(frame, layout[2]);
        self.draw_help_bar(frame, layout[3]);
    }

    fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }

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

pub use MusicModule as Music;
