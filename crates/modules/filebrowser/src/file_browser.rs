use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use ignore::WalkBuilder;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

use crate::file_entry::FileEntry;
use core::event::Action;
use core::module::Shortcut;

pub struct FileBrowser {
    current_path: std::path::PathBuf,
    entries: Vec<FileEntry>,
    selected_index: usize,
    show_hidden: bool,
    sort_by: SortBy,
    theme: ui::Theme,
    render_area: Option<Rect>,
    last_click_index: Option<usize>,
    last_click_time: Option<Instant>,
    file_content: Option<String>,
    selected_file_path: Option<String>,
    split_ratio: f32,
    dragging_split: bool,
    content_scroll_offset: usize,
    info_scroll_offset: usize,
}

#[derive(Debug, Clone, Copy)]
enum SortBy {
    Name,
    Size,
    Modified,
}

impl FileBrowser {
    pub fn new(start_path: std::path::PathBuf) -> Self {
        let mut browser = Self {
            current_path: start_path.clone(),
            entries: Vec::new(),
            selected_index: 0,
            show_hidden: false,
            sort_by: SortBy::Name,
            theme: ui::Theme::default(),
            render_area: None,
            last_click_index: None,
            last_click_time: None,
            file_content: None,
            selected_file_path: None,
            split_ratio: 0.4,
            dragging_split: false,
            content_scroll_offset: 0,
            info_scroll_offset: 0,
        };

        let _ = browser.refresh();
        browser
    }

    pub fn with_theme(mut self, theme: ui::Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        if !self.current_path.exists() {
            self.entries.clear();
            return Ok(());
        }

        let walker = WalkBuilder::new(&self.current_path)
            .hidden(self.show_hidden)
            .max_depth(Some(1))
            .build();

        let mut entries = Vec::new();
        let mut directories = Vec::new();
        let mut files = Vec::new();

        for result in walker {
            if let Ok(entry) = result {
                let entry_path = entry.path();
                if entry_path == self.current_path {
                    continue;
                }

                // Only process direct children
                if let Some(parent) = entry_path.parent() {
                    if parent != self.current_path {
                        continue;
                    }
                }

                let file_entry = FileEntry::new(entry_path, self.show_hidden)?;
                if file_entry.is_dir {
                    directories.push(file_entry);
                } else {
                    files.push(file_entry);
                }
            }
        }

        directories.sort_by_key(|e| e.name.clone());
        files.sort_by_key(|e| e.name.clone());

        entries.extend(directories);
        entries.extend(files);

        self.entries = entries;
        self.selected_index = self
            .selected_index
            .min(self.entries.len().saturating_sub(1));

        Ok(())
    }

    pub fn get_selected(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn open_selected(&mut self) -> anyhow::Result<()> {
        if let Some(entry) = self.get_selected() {
            let entry_path = entry.path.clone();
            let entry_name = entry.name.clone();

            if !entry.is_dir {
                if self.is_text_file(&entry_path) {
                    let content = std::fs::read_to_string(&entry_path);
                    match content {
                        Ok(text) => {
                            self.file_content = Some(text);
                            self.selected_file_path = Some(entry_path.display().to_string());
                            self.content_scroll_offset = 0;
                            log::info!("Opened text file: {:?}", entry_path);
                        }
                        Err(e) => {
                            self.file_content = Some(format!("Error reading file: {}", e));
                            self.selected_file_path = Some(entry_path.display().to_string());
                            log::error!("Failed to open file: {:?}", e);
                        }
                    }
                } else {
                    self.open_with_default_program(&entry_path)?;
                    log::info!("Opened file with system default: {:?}", entry_path);
                }
            }
        }
        Ok(())
    }

    pub fn close_file(&mut self) {
        self.file_content = None;
        self.selected_file_path = None;
        self.content_scroll_offset = 0;
    }

    fn is_text_file(&self, path: &std::path::Path) -> bool {
        let extension = path.extension().and_then(|ext| ext.to_str());

        let text_extensions = [
            "txt",
            "md",
            "rst",
            "adoc",
            "rs",
            "c",
            "cpp",
            "h",
            "hpp",
            "java",
            "kt",
            "py",
            "js",
            "ts",
            "jsx",
            "tsx",
            "html",
            "css",
            "scss",
            "less",
            "sass",
            "json",
            "toml",
            "yaml",
            "yml",
            "xml",
            "ini",
            "cfg",
            "conf",
            "sh",
            "bash",
            "zsh",
            "fish",
            "ps1",
            "bat",
            "cmd",
            "lua",
            "vim",
            "gitignore",
            "gitattributes",
            "gitmodules",
        ];

        if let Some(ext) = extension {
            if text_extensions.contains(&ext.to_lowercase().as_str()) {
                return true;
            }
        }

        if let Ok(meta) = std::fs::metadata(path) {
            if meta.is_file() && meta.len() < 1024 * 1024 {
                if let Ok(bytes) = std::fs::read(path) {
                    if bytes.is_empty() {
                        return true;
                    }
                    let binary_count = bytes
                        .iter()
                        .take(8192)
                        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
                        .count();
                    if binary_count < bytes.len().min(8192) / 100 {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn open_with_default_program(&self, path: &std::path::Path) -> anyhow::Result<()> {
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open").arg(path).spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(path).spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", &path.as_os_str().to_string_lossy()])
                .spawn()?;
        }

        Ok(())
    }

    pub fn navigate_to(&mut self, path: std::path::PathBuf) -> anyhow::Result<()> {
        if path.exists() {
            self.current_path = path;
            self.selected_index = 0;
            self.refresh()?;
        }
        Ok(())
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        let _ = self.refresh();
    }

    pub fn toggle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Name => SortBy::Size,
            SortBy::Size => SortBy::Modified,
            SortBy::Modified => SortBy::Name,
        };
        let _ = self.refresh();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.render_area = Some(area);

        let split_x = area.x + (area.width as f32 * self.split_ratio) as u16;
        let split_x = split_x.max(area.x + 10).min(area.x + area.width - 10);

        let files_area = Rect {
            x: area.x,
            y: area.y,
            width: split_x - area.x,
            height: area.height,
        };

        let split_bar_area = Rect {
            x: split_x,
            y: area.y,
            width: 1,
            height: area.height,
        };

        let info_area = Rect {
            x: split_x + 1,
            y: area.y,
            width: area.width - (split_x - area.x) - 1,
            height: area.height,
        };

        self.render_file_list(frame, files_area);
        self.render_split_bar(frame, split_bar_area);
        self.render_info_panel(frame, info_area);
    }

    fn render_file_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let name = if i == self.selected_index {
                    format!("> {}", entry.display_name())
                } else {
                    entry.display_name()
                };

                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                ListItem::new(name).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Files: {}", self.current_path.display()))
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .highlight_style(self.theme.highlight());

        frame.render_widget(list, area);
    }

    fn render_split_bar(&self, frame: &mut Frame, area: Rect) {
        let bar_style = if self.dragging_split {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        for y in area.y..(area.y + area.height) {
            frame.render_widget(
                Paragraph::new("│")
                    .style(bar_style)
                    .alignment(Alignment::Center),
                Rect {
                    x: area.x,
                    y,
                    width: 1,
                    height: 1,
                },
            );
        }
    }

    fn render_info_panel(&self, frame: &mut Frame, area: Rect) {
        if let Some(ref content) = self.file_content {
            self.render_file_content(frame, area);
        } else {
            self.render_file_info(frame, area);
        }
    }

    fn render_file_info(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![Line::from("File Info"), Line::from(""), Line::from("")];

        if let Some(entry) = self.get_selected() {
            let is_text = self.is_text_file(&entry.path);
            let file_type = if entry.is_dir {
                "Directory".to_string()
            } else if is_text {
                "Text File".to_string()
            } else {
                "Binary File".to_string()
            };

            let open_action = if entry.is_dir {
                "Press Enter to enter directory"
            } else if is_text {
                "Press 'o' or double-click to view content"
            } else {
                "Press 'o' or double-click to open with default app"
            };

            let info = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Gray)),
                    Span::raw(&entry.name),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Gray)),
                    Span::styled(file_type, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("Size: ", Style::default().fg(Color::Gray)),
                    Span::raw(entry.format_size()),
                ]),
                Line::from(vec![
                    Span::styled("Modified: ", Style::default().fg(Color::Gray)),
                    Span::raw(entry.format_modified()),
                ]),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::Gray)),
                    Span::raw(entry.path.display().to_string()),
                ]),
                Line::from(""),
                Line::from(open_action),
            ];
            lines.extend(info);
        } else {
            lines.push(Line::from("No file selected"));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Info")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    fn render_file_content(&self, frame: &mut Frame, area: Rect) {
        if let Some(ref content) = self.file_content {
            let border_height = 2;
            let available_height = area.height.saturating_sub(border_height) as usize;

            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

            let scroll_offset = self
                .content_scroll_offset
                .min(lines.len().saturating_sub(1).max(0));

            let visible_lines: Vec<Line> = lines
                .iter()
                .skip(scroll_offset)
                .take(available_height)
                .map(|line| Line::from(line.clone()))
                .collect();

            let paragraph = Paragraph::new(visible_lines)
                .block(
                    Block::default()
                        .title(format!(
                            "File Content ({} / {})",
                            scroll_offset + 1,
                            lines.len()
                        ))
                        .borders(Borders::ALL)
                        .border_style(self.theme.border()),
                )
                .alignment(Alignment::Left);

            frame.render_widget(paragraph, area);
        } else {
            let paragraph = Paragraph::new("No content available")
                .block(
                    Block::default()
                        .title("File Content")
                        .borders(Borders::ALL)
                        .border_style(self.theme.border()),
                )
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, area);
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) -> Action {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(area) = self.render_area {
                    let mouse_in_bounds = area.contains(Position::new(mouse.column, mouse.row));

                    if mouse_in_bounds {
                        let split_x = area.x + (area.width as f32 * self.split_ratio) as u16;

                        if mouse.column == split_x {
                            self.dragging_split = true;
                            return Action::Consumed;
                        }

                        let border: u16 = 1;
                        let content_y = area.y + border;

                        if mouse.row >= content_y && mouse.column < split_x {
                            let relative_y = mouse.row - content_y;
                            let clicked_index = relative_y as usize;

                            if clicked_index < self.entries.len() {
                                let is_double_click = self.check_double_click(clicked_index);

                                if is_double_click {
                                    return self.enter_or_open_file();
                                } else {
                                    self.selected_index = clicked_index;
                                    Action::Consumed
                                }
                            } else {
                                Action::None
                            }
                        } else {
                            Action::None
                        }
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.dragging_split {
                    self.dragging_split = false;
                    Action::Consumed
                } else {
                    Action::None
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging_split {
                    if let Some(area) = self.render_area {
                        let new_ratio = (mouse.column - area.x) as f32 / area.width as f32;
                        self.split_ratio = new_ratio.clamp(0.1, 0.9);
                        Action::Consumed
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                }
            }
            MouseEventKind::ScrollDown => {
                if self.selected_index < self.entries.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                Action::Consumed
            }
            MouseEventKind::ScrollUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                Action::Consumed
            }
            _ => Action::None,
        }
    }

    fn check_double_click(&mut self, clicked_index: usize) -> bool {
        let DOUBLE_CLICK_THRESHOLD = Duration::from_millis(500);

        let now = Instant::now();

        if let Some((last_index, last_time)) = self.last_click_index.zip(self.last_click_time) {
            if last_index == clicked_index && now - last_time < DOUBLE_CLICK_THRESHOLD {
                self.last_click_index = None;
                self.last_click_time = None;
                return true;
            }
        }

        self.last_click_index = Some(clicked_index);
        self.last_click_time = Some(now);
        false
    }

    fn enter_or_open_file(&mut self) -> Action {
        let entry_info = self
            .get_selected()
            .map(|entry| (entry.is_dir, entry.name.clone()));

        if let Some((is_dir, name)) = entry_info {
            if is_dir {
                let _ = self.enter();
                Action::ShowMessage(core::event::Message::Info(format!(
                    "Entered directory: {}",
                    name
                )))
            } else {
                let content_was_none = self.file_content.is_none();
                let _ = self.open_selected();

                let message = if self.file_content.is_some() {
                    format!("Opened text file: {}", name)
                } else if content_was_none {
                    format!("Opened with system default: {}", name)
                } else {
                    format!("Opened file: {}", name)
                };

                Action::ShowMessage(core::event::Message::Info(message))
            }
        } else {
            Action::None
        }
    }

    fn enter(&mut self) -> anyhow::Result<()> {
        let entry_info = self
            .get_selected()
            .map(|entry| (entry.is_dir, entry.path.clone()));

        if let Some((is_dir, path)) = entry_info {
            if is_dir {
                self.navigate_to(path)?;
            } else {
                self.open_selected()?;
            }
        }
        Ok(())
    }

    fn go_up(&mut self) -> anyhow::Result<()> {
        if let Some(parent) = self.current_path.parent() {
            self.navigate_to(parent.to_path_buf())?;
        }
        Ok(())
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc | KeyCode::Char('c') => {
                if self.file_content.is_some() {
                    self.close_file();
                    Action::ShowMessage(core::event::Message::Info("Closed file".to_string()))
                } else {
                    Action::None
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.file_content.is_some() {
                    if self.content_scroll_offset > 0 {
                        self.content_scroll_offset -= 1;
                    }
                    Action::Consumed
                } else {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    Action::Consumed
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.file_content.is_some() {
                    if let Some(ref content) = self.file_content {
                        let max_offset = content.lines().count().saturating_sub(1);
                        if self.content_scroll_offset < max_offset {
                            self.content_scroll_offset += 1;
                        }
                    }
                    Action::Consumed
                } else {
                    if self.selected_index < self.entries.len().saturating_sub(1) {
                        self.selected_index += 1;
                    }
                    Action::Consumed
                }
            }
            KeyCode::PageUp => {
                if self.file_content.is_some() {
                    let page_size = 20;
                    self.content_scroll_offset = self.content_scroll_offset.saturating_sub(page_size);
                    Action::Consumed
                } else {
                    let page_size = 10;
                    self.selected_index = self.selected_index.saturating_sub(page_size);
                    Action::Consumed
                }
            }
            KeyCode::PageDown => {
                if self.file_content.is_some() {
                    let page_size = 20;
                    if let Some(ref content) = self.file_content {
                        let max_offset = content.lines().count().saturating_sub(1);
                        self.content_scroll_offset = (self.content_scroll_offset + page_size).min(max_offset);
                    }
                    Action::Consumed
                } else {
                    let page_size = 10;
                    let max_index = self.entries.len().saturating_sub(1);
                    self.selected_index = (self.selected_index + page_size).min(max_index);
                    Action::Consumed
                }
            }
            KeyCode::Home => {
                if self.file_content.is_some() {
                    self.content_scroll_offset = 0;
                    Action::Consumed
                } else {
                    self.selected_index = 0;
                    Action::Consumed
                }
            }
            KeyCode::End => {
                if self.file_content.is_some() {
                    if let Some(ref content) = self.file_content {
                        self.content_scroll_offset = content.lines().count().saturating_sub(1);
                    }
                    Action::Consumed
                } else {
                    self.selected_index = self.entries.len().saturating_sub(1);
                    Action::Consumed
                }
            }
            KeyCode::Enter => {
                let entry_info = self
                    .get_selected()
                    .map(|entry| (entry.is_dir, entry.name.clone()));

                if let Some((is_dir, name)) = entry_info {
                    if is_dir {
                        let _ = self.enter();
                        Action::ShowMessage(core::event::Message::Info(format!(
                            "Entered directory: {}",
                            name
                        )))
                    } else {
                        let content_was_none = self.file_content.is_none();
                        let _ = self.open_selected();

                        let message = if self.file_content.is_some() {
                            format!("Opened text file: {}", name)
                        } else if content_was_none {
                            format!("Opened with system default: {}", name)
                        } else {
                            format!("Opened file: {}", name)
                        };

                        Action::ShowMessage(core::event::Message::Info(message))
                    }
                } else {
                    Action::None
                }
            }
            KeyCode::Char('o') => {
                let entry_info = self.get_selected().map(|entry| entry.name.clone());

                if let Some(name) = entry_info {
                    let content_was_none = self.file_content.is_none();
                    let _ = self.open_selected();

                    let message = if self.file_content.is_some() {
                        format!("Opened text file: {}", name)
                    } else if content_was_none {
                        format!("Opened with system default: {}", name)
                    } else {
                        format!("Opened file: {}", name)
                    };

                    Action::ShowMessage(core::event::Message::Info(message))
                } else {
                    Action::None
                }
            }
            KeyCode::Char('c') => {
                if self.file_content.is_some() {
                    self.close_file();
                    Action::ShowMessage(core::event::Message::Info("Closed file".to_string()))
                } else {
                    Action::None
                }
            }
            KeyCode::Char('u') | KeyCode::Backspace => {
                let _ = self.go_up();
                Action::ShowMessage(core::event::Message::Info("Navigated to parent directory".to_string()))
            }
            KeyCode::Char('h') => {
                self.toggle_hidden();
                Action::ShowMessage(core::event::Message::Info(format!(
                    "Hidden files now: {}",
                    if self.show_hidden { "shown" } else { "hidden" }
                )))
            }
            KeyCode::Char('s') => {
                self.toggle_sort();
                Action::Consumed
            }
            KeyCode::Char('?') => Action::ShowMessage(core::event::Message::Info(
                "j/k: Navigate | Enter: Enter dir | o: Open | c: Close file | Esc: Close file | h: Toggle hidden | s: Sort | u: Up"
                    .to_string(),
            )),
            _ => Action::None,
        }
    }

    pub fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "j/k",
                description: "Navigate up/down",
            },
            Shortcut {
                key: "Enter",
                description: "Enter directory",
            },
            Shortcut {
                key: "o",
                description: "Open file",
            },
            Shortcut {
                key: "u",
                description: "Go to parent",
            },
            Shortcut {
                key: "h",
                description: "Toggle hidden files",
            },
            Shortcut {
                key: "s",
                description: "Toggle sort",
            },
        ]
    }
}
