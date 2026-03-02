//! Terminal module - Multi-tab embedded terminal emulator

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

// Portable PTY import
use portable_pty::{
    native_pty_system, CommandBuilder, MasterPty, NativePtySystem, PtySize, SlavePty,
};

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
    ui::Theme,
};

/// Terminal tab configuration
#[derive(Clone)]
pub struct TerminalTab {
    pub id: String,
    pub title: String,
    pub shell: String,
    pub active: bool,
}

/// Terminal output buffer
#[derive(Clone)]
struct TerminalBuffer {
    output: VecDeque<String>,
    max_lines: usize,
}

impl TerminalBuffer {
    fn new(max_lines: usize) -> Self {
        Self {
            output: VecDeque::with_capacity(max_lines),
            max_lines,
        }
    }

    fn push(&mut self, line: String) {
        if self.output.len() >= self.max_lines {
            self.output.pop_front();
        }
        self.output.push_back(line);
    }

    fn as_lines(&self) -> Vec<String> {
        self.output.iter().cloned().collect()
    }
}

/// Terminal instance with PTY
struct TerminalInstance {
    _master: Box<dyn MasterPty + Send>,
    reader_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    buffer: Arc<Mutex<TerminalBuffer>>,
    title: String,
}

impl TerminalInstance {
    fn new(shell_cmd: &str, buffer: Arc<Mutex<TerminalBuffer>>) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_pair = pty_system.openpty(pty_size)?;

        // Build command
        let cmd_args: Vec<&str> = shell_cmd.split_whitespace().collect();
        let mut cmd_builder = CommandBuilder::new(cmd_args[0]);
        for arg in &cmd_args[1..] {
            cmd_builder.arg(arg);
        }

        let _reader: Box<dyn std::io::Read + Send> = pty_pair.master.try_clone_reader()?;
        let reader_thread = None;

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let buffer_clone = buffer.clone();

        // Spawn reader thread
        let handle = thread::spawn(move || {
            let mut reader = pty_pair.master.take_reader().unwrap();
            let mut buffer = [0u8; 4096];

            while running_clone.load(Ordering::SeqCst) {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(text) = std::str::from_utf8(&buffer[..n]) {
                            let lines: Vec<String> = text
                                .lines()
                                .map(|l| {
                                    l.to_string()
                                        .replace("\x1b[", "")
                                        .replace("\x1b", "")
                                        .replace("\x07", "")
                                })
                                .collect();

                            if let Ok(mut buf) = buffer_clone.lock() {
                                for line in lines {
                                    buf.push(line);
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Start shell
        let _child = pty_pair.slave.spawn_command(cmd_builder)?;

        Ok(Self {
            _master: pty_pair.master,
            reader_thread: Some(handle),
            running,
            buffer,
            title: String::from("Terminal"),
        })
    }

    fn send_input(&mut self, input: &[u8]) -> anyhow::Result<()> {
        if let Some(_master) = &mut self
            ._master
            .as_any()
            .downcast_ref::<Box<dyn MasterPty + Send>>()
        {
            // Try to write to master PTY
        }
        Ok(())
    }

    fn resize(&mut self, rows: u16, cols: u16) -> anyhow::Result<()> {
        if let Some(master) = self
            ._master
            .as_any()
            .downcast_ref::<Box<dyn MasterPty + Send>>()
        {
            // Try to resize PTY
        }
        Ok(())
    }

    fn terminate(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }
    }

    fn get_output(&self) -> Vec<String> {
        if let Ok(buf) = self.buffer.lock() {
            buf.as_lines()
        } else {
            vec![]
        }
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn get_title(&self) -> &str {
        &self.title
    }
}

/// Main terminal module with multi-tab support
pub struct TerminalModule {
    tabs: Vec<TerminalTab>,
    instances: Vec<TerminalInstance>,
    active_tab_index: usize,
    theme: Theme,
}

impl TerminalModule {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            instances: Vec::new(),
            active_tab_index: 0,
            theme: Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Create new tab
    pub fn new_tab(&mut self, shell: Option<String>) -> anyhow::Result<()> {
        let shell = shell.unwrap_or_else(|| self.get_default_shell());
        let tab_id = uuid::Uuid::new_v4().to_string();

        let buffer = Arc::new(Mutex::new(TerminalBuffer::new(1000)));
        let instance = TerminalInstance::new(&shell, buffer)?;

        let tab = TerminalTab {
            id: tab_id.clone(),
            title: instance.get_title().to_string(),
            shell: shell.clone(),
            active: false,
        };

        self.tabs.push(tab);
        self.instances.push(instance);
        self.active_tab_index = self.tabs.len() - 1;

        Ok(())
    }

    /// Close active tab
    pub fn close_tab(&mut self) {
        if !self.instances.is_empty() {
            self.instances[self.active_tab_index].terminate();
            self.instances.remove(self.active_tab_index);
            self.tabs.remove(self.active_tab_index);

            if self.active_tab_index >= self.instances.len() {
                self.active_tab_index = self.instances.len().max(1) - 1;
            }
        }
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        if self.active_tab_index > 0 {
            self.active_tab_index -= 1;
        }
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        if self.active_tab_index < self.instances.len().saturating_sub(1) {
            self.active_tab_index += 1;
        }
    }

    /// Get default shell based on platform
    fn get_default_shell(&self) -> String {
        #[cfg(target_os = "windows")]
        let shell = "cmd.exe".to_string();

        #[cfg(target_os = "macos")]
        let shell = if std::path::Path::new("/bin/zsh").exists() {
            "/bin/zsh".to_string()
        } else {
            "/bin/bash".to_string()
        };

        #[cfg(target_os = "linux")]
        let shell = if std::path::Path::new("/usr/bin/zsh").exists() {
            "/usr/bin/zsh".to_string()
        } else if std::path::Path::new("/bin/zsh").exists() {
            "/bin/zsh".to_string()
        } else {
            "/bin/bash".to_string()
        };

        shell
    }

    /// Send input to active terminal
    fn send_input(&mut self, input: &[u8]) {
        if !self.instances.is_empty() {
            let _ = self.instances[self.active_tab_index].send_input(input);
        }
    }

    /// Draw tab bar
    fn draw_tab_bar(&self, frame: &mut Frame, area: Rect) {
        let tabs: Vec<Span> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if i == self.active_tab_index {
                    Style::default()
                        .fg(self.theme.background())
                        .bg(self.theme.primary())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(self.theme.muted())
                        .bg(self.theme.surface())
                };

                Span::styled(format!(" {} ", tab.title), style)
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border());

        let paragraph = Paragraph::new(Line::from(tabs)).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Draw terminal output
    fn draw_terminal(&self, frame: &mut Frame, area: Rect) {
        if self.instances.is_empty() {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.border());

            let paragraph = Paragraph::new("按 Alt+N 创建新终端标签")
                .block(block)
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, area);
            return;
        }

        let output = self.instances[self.active_tab_index].get_output();
        let lines: Vec<Line> = output.into_iter().map(|l| Line::from(l)).collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border());

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('n') | KeyCode::Char('N') if key.modifiers == KeyModifiers::ALT => {
                let _ = self.new_tab(None);
                Action::None
            }
            KeyCode::Char('w') | KeyCode::Char('W') if key.modifiers == KeyModifiers::ALT => {
                self.close_tab();
                Action::None
            }
            KeyCode::Char('[') if key.modifiers == KeyModifiers::ALT => {
                self.prev_tab();
                Action::None
            }
            KeyCode::Char(']') if key.modifiers == KeyModifiers::ALT => {
                self.next_tab();
                Action::None
            }
            KeyCode::Char('1') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 0 {
                    self.active_tab_index = 0;
                }
                Action::None
            }
            KeyCode::Char('2') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 1 {
                    self.active_tab_index = 1;
                }
                Action::None
            }
            KeyCode::Char('3') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 2 {
                    self.active_tab_index = 2;
                }
                Action::None
            }
            KeyCode::Char('4') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 3 {
                    self.active_tab_index = 3;
                }
                Action::None
            }
            KeyCode::Char('5') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 4 {
                    self.active_tab_index = 4;
                }
                Action::None
            }
            KeyCode::Char('6') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 5 {
                    self.active_tab_index = 5;
                }
                Action::None
            }
            KeyCode::Char('7') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 6 {
                    self.active_tab_index = 6;
                }
                Action::None
            }
            KeyCode::Char('8') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 7 {
                    self.active_tab_index = 7;
                }
                Action::None
            }
            KeyCode::Char('9') if key.modifiers == KeyModifiers::ALT => {
                if self.tabs.len() > 8 {
                    self.active_tab_index = 8;
                }
                Action::None
            }
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }
}

impl CoreModule for TerminalModule {
    fn name(&self) -> &str {
        "terminal"
    }

    fn title(&self) -> &str {
        "终端"
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
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        // Draw tab bar
        self.draw_tab_bar(frame, layout[0]);

        // Draw terminal output
        self.draw_terminal(frame, layout[1]);
    }

    fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "Alt+N",
                description: "新建标签",
            },
            Shortcut {
                key: "Alt+W",
                description: "关闭标签",
            },
            Shortcut {
                key: "Alt+[",
                description: "上一标签",
            },
            Shortcut {
                key: "Alt+1-9",
                description: "切换标签",
            },
        ]
    }

    fn init(&mut self) -> anyhow::Result<()> {
        // Create first tab with default shell
        self.new_tab(None)?;
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        // Terminate all terminals
        for instance in &mut self.instances {
            instance.terminate();
        }
        self.instances.clear();
        self.tabs.clear();
        Ok(())
    }
}

impl Drop for TerminalModule {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

pub use TerminalModule as Terminal;
