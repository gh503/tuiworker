//! Music module - Audio playback with playlist management - Absolute paths and source switching

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
    MusicEvent, MusicEventListener, PlaybackMode, PlaybackState, PlayerController, SourceType,
    Track,
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

/// Main music module with absolute paths and source switching
pub struct MusicModule {
    controller: PlayerController,
    music_dir: PathBuf,
    theme: Theme,
    selected_index: usize,
    current_source: SourceType,
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
            current_source: SourceType::Local,
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

    /// Clear and reload music from directory
    pub fn reload_music(&mut self) -> anyhow::Result<()> {
        self.controller.clear_queue();
        self.load_music()
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
                    let (artist, title) = self.extract_metadata(&path);
                    let track = Track::local(path, title, artist);
                    self.controller.add_track(track);
                }
            }
        }

        Ok(())
    }

    /// Extract metadata from audio file using symphonia
    /// TODO: Implement proper metadata extraction using symphonia
    /// Currently returns fallback values based on filename
    fn extract_metadata(&self, path: &PathBuf) -> (String, String) {
        // TODO: Implement symphonia metadata extraction properly
        // For now, fall back to filename-based metadata
        (String::from("-"), self.get_filename_title(path))
    }

    fn get_filename_title(&self, path: &PathBuf) -> String {
        path.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("-")
            .to_string()
            .trim_end_matches(".mp3")
            .trim_end_matches(".flac")
            .trim_end_matches(".ogg")
            .trim_end_matches(".wav")
            .trim_end_matches(".m4a")
            .to_string()
    }

    /// Play track
    pub fn play(&mut self, index: usize) {
        if let Err(e) = self.controller.play_track(index) {
            log::error!("Failed to play track: {}", e);
        }
    }

    /// Pause playback
    pub fn pause(&mut self) -> Result<(), anyhow::Error> {
        log::info!("[Music] Pausing playback");
        self.controller.pause().map_err(|e| {
            log::error!("[Music] Failed to pause playback: {}", e);
            anyhow::anyhow!(e)
        })
    }

    /// Resume playback
    pub fn resume(&mut self) -> Result<(), anyhow::Error> {
        log::info!("[Music] Resuming playback");
        self.controller.resume().map_err(|e| {
            log::error!("[Music] Failed to resume playback: {}", e);
            anyhow::anyhow!(e)
        })
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

    /// Get current source name
    fn get_source_name(&self) -> &str {
        match self.current_source {
            SourceType::Local => "Local",
            SourceType::QqMusic => "QQ Music",
            SourceType::NetEaseMusic => "NetEase Music",
            SourceType::Nas { .. } => "NAS",
        }
    }

    /// Switch music source
    pub fn switch_source(&mut self) {
        let old_source = self.get_source_name().to_string();

        self.current_source = match self.current_source {
            SourceType::Local => SourceType::QqMusic,
            SourceType::QqMusic => SourceType::NetEaseMusic,
            SourceType::NetEaseMusic => SourceType::Nas { mount_point: None },
            SourceType::Nas { .. } => SourceType::Local,
        };

        let music_dir = self.music_dir.display().to_string();
        let new_source = self.get_source_name();
        log::info!("[Music] Switched from {} to {}", old_source, new_source);

        self.controller.clear_queue();
        self.selected_index = 0;

        let is_local = matches!(self.current_source, SourceType::Local);
        let is_netease = matches!(self.current_source, SourceType::NetEaseMusic);
        let is_qq = matches!(self.current_source, SourceType::QqMusic);

        if is_local {
            log::info!("[Music] Loading local music from: {}", music_dir);
            if let Err(e) = self.load_music() {
                log::error!("[Music] Failed to load music: {}", e);
            }
            log::info!(
                "[Music] Loaded {} tracks",
                self.controller.get_queue().len()
            );
        } else if is_netease {
            log::info!(
                "[Music] NetEase Music selected. Search functionality is not yet implemented."
            );
            log::info!("[Music] To use NetEase Music, you need to enable it in config: ~/.config/tuiworker/music.toml");
        } else if is_qq {
            log::info!("[Music] QQ Music selected. Search functionality is not yet implemented.");
            log::info!("[Music] Reference: QQMusicApi at /home/gh503/Code/QQMusicApi");
        }
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

    /// Draw playlist with absolute path display
    fn draw_playlist(&self, frame: &mut Frame, area: Rect) {
        let tracks = self.controller.get_queue().get_tracks();
        let current_index = self.controller.get_queue().current_index();

        let mut lines = vec![
            Line::from(vec![Span::styled(
                format!(
                    "播放列表 ({} 首) [{}]",
                    tracks.len(),
                    self.get_source_name()
                ),
                Style::default()
                    .fg(self.theme.primary())
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
        ];

        if tracks.is_empty() {
            if self.current_source == SourceType::NetEaseMusic {
                lines.push(Line::from("NetEase Music: 搜索功能尚未实现"));
                lines.push(Line::from("提示: 在 ~/.config/tuiworker/music.toml 中配置"));
            } else if self.current_source == SourceType::QqMusic {
                lines.push(Line::from("QQ Music: 搜索功能尚未实现"));
            } else if let SourceType::Nas { .. } = self.current_source {
                lines.push(Line::from("NAS: 请配置网络存储或手动添加文件"));
            } else {
                lines.push(Line::from("没有找到音乐文件"));
                lines.push(Line::from(format!(
                    "音乐目录: {}",
                    self.music_dir.display()
                )));
            }
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

                let path_display = if track.path.is_absolute() {
                    format!("📁 {}", track.path.display())
                } else {
                    format!("📁 {}/{}", self.music_dir.display(), track.path.display())
                };

                let line = format!(
                    "{} {} - {} | {}",
                    if is_current { "►" } else { " " },
                    track.artist,
                    track.title,
                    path_display
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
    /// Draw lyrics panel
    fn draw_lyrics(&self, frame: &mut Frame, area: Rect) {
        let current_track = self.controller.get_current_track();

        if current_track.is_none() {
            let paragraph = Paragraph::new(Text::from(vec![
                Line::from(vec![Span::styled(
                    "歌词",
                    Style::default().fg(self.theme.muted()),
                )]),
                Line::default(),
                Line::from("没有正在播放的歌曲"),
            ]))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, area);
            return;
        }

        let track = current_track.unwrap();

        if let Some(lrc_text) = &track.lyrics {
            use music_model::{LrcParser, Lyrics};

            match LrcParser::parse(lrc_text) {
                Ok(lyrics) => {
                    let position = self.controller.get_position();
                    let current_index = lyrics.find_current_line(position);

                    let visible_count = area.height.saturating_sub(2) as usize;
                    let start_index = if let Some(idx) = current_index {
                        idx.saturating_sub(visible_count / 2)
                    } else {
                        0
                    };
                    let end_index = (start_index + visible_count).min(lyrics.lines.len());

                    let mut lines = vec![
                        Line::from(vec![Span::styled(
                            format!("歌词 - {} [{}]", track.title, track.artist),
                            Style::default()
                                .fg(self.theme.primary())
                                .add_modifier(Modifier::BOLD),
                        )]),
                        Line::default(),
                    ];

                    for i in start_index..end_index {
                        if let Some(line) = lyrics.get_line(i) {
                            let is_current = current_index == Some(i);
                            let style = if is_current {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(self.theme.muted())
                            };

                            lines.push(Line::from(vec![Span::styled(&line.text, style)]));
                        }
                    }

                    if lyrics.lines.is_empty() {
                        lines.push(Line::from("暂无歌词"));
                    }

                    let paragraph = Paragraph::new(Text::from(lines))
                        .block(Block::default().borders(Borders::ALL))
                        .wrap(Wrap { trim: true });

                    frame.render_widget(paragraph, area);
                }
                Err(_) => {
                    let paragraph = Paragraph::new(Text::from(vec![
                        Line::from(vec![Span::styled(
                            "歌词",
                            Style::default().fg(self.theme.muted()),
                        )]),
                        Line::default(),
                        Line::from("歌词解析失败"),
                        Line::default(),
                        Line::from("LRC格式: [MM:SS.ms] 歌词文本"),
                        Line::from("示例: [00:00.00] First line"),
                    ]))
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });

                    frame.render_widget(paragraph, area);
                }
            }
        } else {
            let paragraph = Paragraph::new(Text::from(vec![
                Line::from(vec![Span::styled(
                    format!("歌词 - {} [{}]", track.title, track.artist),
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::default(),
                Line::from("暂无歌词"),
                Line::default(),
                Line::from("NetEase Music: 播放时自动获取歌词"),
                Line::from("Local: 支持内嵌歌词文件"),
            ]))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, area);
        }
    }

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

        let playback_status = match state {
            PlaybackState::Playing => "播放中",
            PlaybackState::Paused => "已暂停",
            PlaybackState::Stopped => "已停止",
            PlaybackState::Loading => "加载中",
            PlaybackState::Buffering => "缓冲中",
        };

        let mode = self.controller.get_queue().get_mode();
        let source_name = self.get_source_name();
        let volume = self.controller.get_volume();

        log::debug!(
            "[Music] Display: source={}, state={}, volume={:.0}%, mode={:?}",
            source_name,
            playback_status,
            volume * 100.0,
            mode
        );

        let mode_text = match mode {
            PlaybackMode::Sequential => "顺序",
            PlaybackMode::Random => "随机",
            PlaybackMode::RepeatOne => "单曲循环",
            PlaybackMode::RepeatAll => "列表循环",
        };

        lines.push(
            vec![
                Span::styled("源:", Style::default().fg(self.theme.muted())),
                Span::styled(source_name, Style::default().fg(self.theme.primary())),
                Span::styled(" 状态:", Style::default().fg(self.theme.muted())),
                Span::styled(playback_status, Style::default()),
            ]
            .into(),
        );

        lines.push(
            vec![
                Span::styled("音量:", Style::default().fg(self.theme.muted())),
                Span::styled(
                    format!("{:.0}%", volume * 100.0),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  模式:", Style::default().fg(self.theme.muted())),
                Span::styled(
                    mode_text,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]
            .into(),
        );

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("播放器")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: true });

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
        let help = "Space:播放/停止 <:音量- >:音量+ r:切换循环 s:切换源 n:下一首 p:上一首 j/k:导航 Enter:播放选中";
        let paragraph = Paragraph::new(help).style(Style::default().fg(self.theme.muted()));
        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(' ') => {
                let state_before = self.controller.get_state();
                log::info!(
                    "[Music] Space key pressed, current state: {:?}",
                    state_before
                );

                match state_before {
                    PlaybackState::Playing => {
                        if let Err(e) = self.pause() {
                            log::error!("[Music] Pause error: {}", e);
                        } else {
                            log::info!("[Music] Paused successfully");
                        }
                    }
                    PlaybackState::Paused => {
                        if let Err(e) = self.resume() {
                            log::error!("[Music] Resume error: {}", e);
                        } else {
                            log::info!("[Music] Resumed successfully");
                        }
                    }
                    _ => {
                        if !self.controller.get_queue().is_empty() {
                            log::info!("[Music] Playing track at index {}", self.selected_index);
                            self.play(self.selected_index);
                        } else {
                            log::warn!("[Music] Cannot play - queue is empty");
                        }
                    }
                }

                let state_after = self.controller.get_state();
                log::info!("[Music] State after space: {:?}", state_after);
                Action::Consumed
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
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.switch_source();
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(8),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        self.draw_player(frame, chunks[0]);
        self.draw_lyrics(frame, chunks[1]);
        self.draw_playlist(frame, chunks[2]);
        self.draw_help_bar(frame, chunks[3]);
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
                key: "s",
                description: "切换源",
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
        self.reload_music()?;
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.stop();
        Ok(())
    }
}

pub use MusicModule as Music;
