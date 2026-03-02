use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ignore::WalkBuilder;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

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
        };

        let _ = browser.refresh();
        browser
    }

    pub fn with_theme(mut self, theme: ui::Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        self.entries.clear();

        if !self.current_path.exists() {
            return Err(anyhow::anyhow!(
                "Path does not exist: {:?}",
                self.current_path
            ));
        }

        let mut entries: Vec<FileEntry> = Vec::new();
        let walker = WalkBuilder::new(&self.current_path)
            .hidden(!self.show_hidden)
            .max_depth(Some(1))
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if let Some(path) = entry.path().canonicalize().ok() {
                        if path != self.current_path {
                            if let Ok(file_entry) = FileEntry::new(&path, self.show_hidden) {
                                entries.push(file_entry);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read entry: {:?}", e);
                }
            }
        }

        // 排序
        match self.sort_by {
            SortBy::Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Size => entries.sort_by(|a, b| b.size.cmp(&a.size)),
            SortBy::Modified => entries.sort_by(|a, b| b.modified.cmp(&a.modified)),
        }

        self.entries = entries;
        self.selected_index = self
            .selected_index
            .min(self.entries.len().saturating_sub(1));

        Ok(())
    }

    pub fn enter(&mut self) -> anyhow::Result<()> {
        if self.selected_index < self.entries.len() {
            let entry = &self.entries[self.selected_index];
            if entry.is_dir {
                self.current_path = entry.path.clone();
                self.selected_index = 0;
                self.refresh()?;
            } else {
                // Enter pressed on file - could open preview
            }
        }
        Ok(())
    }

    pub fn go_up(&mut self) -> anyhow::Result<()> {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.selected_index = 0;
            self.refresh()?;
        } else {
            // Already at root
        }
        Ok(())
    }

    pub fn get_selected(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn open_selected(&self) -> anyhow::Result<()> {
        if let Some(entry) = self.get_selected() {
            if !entry.is_dir {
                opener::open(&entry.path)?;
                log::info!("Opened file: {:?}", entry.path);
            }
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
        // Split into file list and info panel
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.render_file_list(frame, chunks[0]);
        self.render_info_panel(frame, chunks[1]);
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

    fn render_info_panel(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![Line::from("File Info"), Line::from(""), Line::from("")];

        if let Some(entry) = self.get_selected() {
            let info = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Gray)),
                    Span::raw(&entry.name),
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
                Line::from("Press 'o' to open with default app"),
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

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                Action::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.entries.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                Action::Consumed
            }
            KeyCode::Enter => {
                let _ = self.enter();
                Action::Consumed
            }
            KeyCode::Char('o') => {
                let _ = self.open_selected();
                Action::Consumed
            }
            KeyCode::Char('u') | KeyCode::Backspace => {
                let _ = self.go_up();
                Action::Consumed
            }
            KeyCode::Char('h') => {
                self.toggle_hidden();
                Action::Consumed
            }
            KeyCode::Char('s') => {
                self.toggle_sort();
                Action::Consumed
            }
            KeyCode::Char('?') => {
                // Show help
                Action::ShowMessage(core::event::Message::Info(
                    "j/k: Navigate | Enter: Enter dir | o: Open | h: Toggle hidden | s: Sort | u: Up".to_string(),
                ))
            }
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
                description: "Go up",
            },
            Shortcut {
                key: "h",
                description: "Toggle hidden files",
            },
            Shortcut {
                key: "s",
                description: "Change sort order",
            },
        ]
    }
}
