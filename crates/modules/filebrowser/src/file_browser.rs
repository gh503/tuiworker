use copypasta::ClipboardProvider;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use ignore::WalkBuilder;
use log;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
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
    content_scroll_offset_x: usize,
    info_scroll_offset: usize,
    active_area: ActiveArea,
    search_mode: bool,
    search_query: String,
    search_results: Vec<usize>,
    search_result_index: usize,
    content_search_query: String,
    content_search_matches: Vec<usize>,
    content_search_index: usize,
    // Text selection fields
    text_selection: Option<(usize, usize)>, // (start_byte, end_byte)
    selection_start_line: Option<usize>,
    selection_start_col: Option<usize>,
    selecting: bool,
    context_menu_visible: bool,
    delete_pending: bool,
    rename_input: String,
    rename_mode: bool,
}

#[derive(Debug, Clone, Copy)]
enum SortBy {
    Name,
    Size,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ActiveArea {
    FileList,
    Content,
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
            content_scroll_offset_x: 0,
            info_scroll_offset: 0,
            active_area: ActiveArea::FileList,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_index: 0,
            content_search_query: String::new(),
            content_search_matches: Vec::new(),
            content_search_index: 0,
            text_selection: None,
            selection_start_line: None,
            selection_start_col: None,
            selecting: false,
            context_menu_visible: false,
            delete_pending: false,
            rename_input: String::new(),
            rename_mode: false,
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

        match self.sort_by {
            SortBy::Name => {
                directories.sort_by_key(|e| e.name.to_lowercase());
                files.sort_by_key(|e| e.name.to_lowercase());
            }
            SortBy::Size => {
                directories.sort_by_key(|e| e.size);
                files.sort_by_key(|e| e.size);
            }
            SortBy::Modified => {
                directories.sort_by_key(|e| e.modified);
                files.sort_by_key(|e| e.modified);
            }
        }

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
                            log::info!(
                                "Opened text file: {:?}, file_content set to Some",
                                entry_path
                            );
                            log::info!(
                                "Current state - file_content: {}, selected_file_path: {:?}",
                                self.file_content.is_some(),
                                self.selected_file_path
                            );
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

        let search_bar_height = if self.search_mode { 1 } else { 0 };

        let (files_area, info_area, split_bar_area) = if self.search_mode {
            if self.active_area == ActiveArea::FileList {
                let search_bar_area = Rect {
                    x: area.x,
                    y: area.y,
                    width: split_x - area.x,
                    height: 1,
                };
                self.render_search_bar(frame, search_bar_area);

                let files_area = Rect {
                    x: area.x,
                    y: area.y + 1,
                    width: split_x - area.x,
                    height: area.height - 1,
                };
                let split_bar_area = Rect {
                    x: split_x,
                    y: area.y + 1,
                    width: 1,
                    height: area.height - 1,
                };
                let info_area = Rect {
                    x: split_x + 1,
                    y: area.y + 1,
                    width: area.width - (split_x - area.x) - 1,
                    height: area.height - 1,
                };
                (files_area, info_area, split_bar_area)
            } else {
                let search_bar_area = Rect {
                    x: split_x + 1,
                    y: area.y,
                    width: area.width - (split_x - area.x) - 1,
                    height: 1,
                };
                self.render_search_bar(frame, search_bar_area);

                let files_area = Rect {
                    x: area.x,
                    y: area.y,
                    width: split_x - area.x,
                    height: area.height - 1,
                };
                let split_bar_area = Rect {
                    x: split_x,
                    y: area.y,
                    width: 1,
                    height: area.height - 1,
                };
                let info_area = Rect {
                    x: split_x + 1,
                    y: area.y + 1,
                    width: area.width - (split_x - area.x) - 1,
                    height: area.height - 1,
                };
                (files_area, info_area, split_bar_area)
            }
        } else {
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
            (files_area, info_area, split_bar_area)
        };

        self.render_file_list(frame, files_area, self.active_area == ActiveArea::FileList);
        self.render_split_bar(frame, split_bar_area);
        self.render_info_panel(frame, info_area, self.active_area == ActiveArea::Content);

        if self.delete_pending {
            self.render_delete_confirm(frame, area);
        }

        if self.rename_mode {
            self.render_rename_input(frame, area);
        }
    }

    fn render_delete_confirm(&self, frame: &mut Frame, area: Rect) {
        let overlay_width = 50.min(area.width - 4);
        let overlay_height = 8.min(area.height - 4);

        let overlay_area = Rect {
            x: area.x + (area.width.saturating_sub(overlay_width)) / 2,
            y: area.y + (area.height.saturating_sub(overlay_height)) / 2,
            width: overlay_width,
            height: overlay_height,
        };

        let entry_name = self
            .get_selected()
            .map(|e| e.name.clone())
            .unwrap_or_else(|| String::from("selected"));

        let content = vec![
            Line::from("Confirm Delete"),
            Line::from(""),
            Line::from(format!("Delete '{}'?", entry_name)),
            Line::from(""),
            Line::from(vec![
                Span::styled("[y]", Style::default().fg(Color::Green)),
                Span::raw(" Confirm  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(" Delete ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(Clear, overlay_area);
        frame.render_widget(paragraph, overlay_area);
    }

    fn render_rename_input(&self, frame: &mut Frame, area: Rect) {
        let overlay_width = 50.min(area.width - 4);
        let overlay_height = 8.min(area.height - 4);

        let overlay_area = Rect {
            x: area.x + (area.width.saturating_sub(overlay_width)) / 2,
            y: area.y + (area.height.saturating_sub(overlay_height)) / 2,
            width: overlay_width,
            height: overlay_height,
        };

        let content = vec![
            Line::from("Rename"),
            Line::from(""),
            Line::from(self.rename_input.clone()),
            Line::from(""),
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" Confirm  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ]),
        ];

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(" Rename ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(Clear, overlay_area);
        frame.render_widget(paragraph, overlay_area);
    }

    fn render_search_bar(&self, frame: &mut Frame, area: Rect) {
        let (query, matches, current) = if self.active_area == ActiveArea::FileList {
            (
                &self.search_query,
                &self.search_results,
                self.search_result_index,
            )
        } else {
            (
                &self.content_search_query,
                &self.content_search_matches,
                self.content_search_index,
            )
        };

        let search_type = if self.active_area == ActiveArea::FileList {
            "File"
        } else {
            "Content"
        };
        let match_info = if matches.is_empty() {
            format!("{}: [{}]", search_type, query)
        } else {
            format!(
                "{}: [{}] ({}/{})",
                search_type,
                query,
                current + 1,
                matches.len()
            )
        };

        let paragraph = Paragraph::new(match_info)
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));

        frame.render_widget(paragraph, area);
    }

    fn render_file_list(&self, frame: &mut Frame, area: Rect, is_active: bool) {
        let is_searching = self.search_mode && !self.search_query.is_empty();

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let is_selected = i == self.selected_index;
                let is_matched = is_searching && self.search_results.contains(&i);

                let name = if is_selected {
                    format!("> {}", entry.display_name())
                } else {
                    entry.display_name()
                };

                let (fg_color, modifier) = if is_selected {
                    (Color::White, Modifier::BOLD)
                } else if is_matched {
                    (Color::Green, Modifier::BOLD)
                } else {
                    (Color::Gray, Modifier::empty())
                };

                let style = Style::default().fg(fg_color).add_modifier(modifier);

                ListItem::new(name).style(style)
            })
            .collect();

        let (border_style, title_style, bg_style) = if is_active {
            (
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default().bg(Color::DarkGray),
            )
        } else {
            (
                self.theme.border(),
                Style::default().fg(Color::Gray),
                Style::default().bg(Color::Black),
            )
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Files: {}", self.current_path.display()))
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(title_style)
                    .style(bg_style),
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

    fn render_info_panel(&self, frame: &mut Frame, area: Rect, is_active: bool) {
        if let Some(ref content) = self.file_content {
            self.render_file_content(frame, area, is_active);
        } else {
            self.render_file_info(frame, area, is_active);
        }
    }

    fn render_file_info(&self, frame: &mut Frame, area: Rect, is_active: bool) {
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

        let (border_style, title_style, bg_style) = if is_active {
            (
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default().bg(Color::DarkGray),
            )
        } else {
            (
                self.theme.border(),
                Style::default().fg(Color::Gray),
                Style::default().bg(Color::Black),
            )
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Info")
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title_style(title_style)
                    .style(bg_style),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    fn render_file_content(&self, frame: &mut Frame, area: Rect, is_active: bool) {
        let (border_style, title_style, bg_style) = if is_active {
            (
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                Style::default().bg(Color::DarkGray),
            )
        } else {
            (
                self.theme.border(),
                Style::default().fg(Color::Gray),
                Style::default().bg(Color::Black),
            )
        };

        if let Some(ref content) = self.file_content {
            let border_height = 2;
            let available_height = area.height.saturating_sub(border_height) as usize;

            let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

            let scroll_offset = self
                .content_scroll_offset
                .min(lines.len().saturating_sub(1).max(0));

            let search_highlight = !self.content_search_query.is_empty();
            let query_lower = self.content_search_query.to_lowercase();
            let scroll_x = self.content_scroll_offset_x;

            let visible_lines_with_line_numbers: Vec<Line> = lines
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(available_height)
                .map(|(idx, line)| {
                    let line_num = idx + 1;
                    let line_num_str = format!("{:4} ", line_num);

                    // Line number color - use lighter color when active panel has dark background
                    let line_num_color = if is_active {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    };

                    // Calculate absolute byte position for this line
                    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
                    let line_start_byte =
                        self.calculate_line_start_byte(&lines_ref, idx + scroll_offset);

                    if search_highlight && line.to_lowercase().contains(&query_lower) {
                        let highlighted_line =
                            Self::highlight_text(line, &self.content_search_query);
                        let cropped_content: Vec<Span> =
                            highlighted_line.into_iter().skip(scroll_x).collect();

                        let mut spans = vec![Span::styled(
                            line_num_str,
                            Style::default().fg(line_num_color),
                        )];
                        spans.extend(cropped_content);
                        Line::from(spans)
                    } else if let Some((sel_start, sel_end)) = self.text_selection {
                        let start = sel_start.min(sel_end);
                        let end = sel_end.max(sel_start);
                        let line_end_byte = line_start_byte + line.as_bytes().len();

                        // Check if this line has any selected text
                        if line_end_byte > start && line_start_byte <= end {
                            let visible_line_spans = self.render_line_with_selection(
                                line,
                                line_start_byte,
                                start.min(line_end_byte),
                                end.max(line_start_byte),
                                scroll_x,
                                is_active,
                            );
                            let mut spans = vec![Span::styled(
                                line_num_str,
                                Style::default().fg(line_num_color),
                            )];
                            spans.extend(visible_line_spans);
                            Line::from(spans)
                        } else {
                            let cropped_line = line.chars().skip(scroll_x).collect::<String>();
                            Line::from(vec![
                                Span::styled(line_num_str, Style::default().fg(line_num_color)),
                                Span::raw(cropped_line),
                            ])
                        }
                    } else {
                        let cropped_line = line.chars().skip(scroll_x).collect::<String>();
                        Line::from(vec![
                            Span::styled(line_num_str, Style::default().fg(line_num_color)),
                            Span::raw(cropped_line),
                        ])
                    }
                })
                .collect();

            let paragraph = Paragraph::new(visible_lines_with_line_numbers)
                .block(
                    Block::default()
                        .title(format!(
                            "File Content ({} / {})",
                            scroll_offset + 1,
                            lines.len()
                        ))
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title_style(title_style)
                        .style(bg_style),
                )
                .alignment(Alignment::Left);

            frame.render_widget(paragraph, area);
        } else {
            let paragraph = Paragraph::new("No content available")
                .block(
                    Block::default()
                        .title("File Content")
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title_style(title_style)
                        .style(bg_style),
                )
                .alignment(Alignment::Left);
            frame.render_widget(paragraph, area);
        }
    }

    fn highlight_text<'a>(line: &'a str, query: &str) -> Vec<Span<'a>> {
        if query.is_empty() {
            return vec![Span::raw(line)];
        }

        let line_lower = line.to_lowercase();
        let query_lower = query.to_lowercase();

        let mut spans = Vec::new();
        let mut last_end = 0;

        if let Some(first_match) = line_lower.find(&query_lower) {
            if first_match > 0 {
                spans.push(Span::raw(&line[..first_match]));
            }

            spans.push(Span::styled(
                &line[first_match..first_match + query.len()],
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ));

            last_end = first_match + query.len();

            while let Some(next_match) = line_lower[last_end..].find(&query_lower) {
                let absolute_match = last_end + next_match;
                spans.push(Span::raw(&line[last_end..absolute_match]));
                spans.push(Span::styled(
                    &line[absolute_match..absolute_match + query.len()],
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ));
                last_end = absolute_match + query.len();
            }

            if last_end < line.len() {
                spans.push(Span::raw(&line[last_end..]));
            }
        } else {
            spans.push(Span::raw(line));
        }

        spans
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
                        let content_area = Rect {
                            x: split_x + 1,
                            y: content_y,
                            width: area.width.saturating_sub(split_x - area.x + 1),
                            height: area.height.saturating_sub(2),
                        };

                        // Check if click is in content area (right panel)
                        if content_area.contains(Position::new(mouse.column, mouse.row))
                            && self.file_content.is_some()
                        {
                            let relative_y = mouse.row - content_area.y;
                            let relative_x = mouse.column - content_area.x;
                            self.handle_text_selection_start(
                                relative_y as usize,
                                relative_x as usize,
                            );
                            return Action::Consumed;
                        }

                        // Check if click is in file list (left panel)
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
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging_split {
                    if let Some(area) = self.render_area {
                        let new_ratio = (mouse.column - area.x) as f32 / area.width as f32;
                        self.split_ratio = new_ratio.clamp(0.1, 0.9);
                        return Action::Consumed;
                    }
                }

                // Handle text selection drag
                if self.selecting && self.render_area.is_some() && self.file_content.is_some() {
                    let area = self.render_area.unwrap();
                    let split_x = area.x + (area.width as f32 * self.split_ratio) as u16;
                    let content_area = Rect {
                        x: split_x + 1,
                        y: area.y + 1,
                        width: area.width.saturating_sub(split_x - area.x + 1),
                        height: area.height.saturating_sub(2),
                    };

                    if content_area.contains(Position::new(mouse.column, mouse.row)) {
                        let relative_y = mouse.row - content_area.y;
                        let relative_x = mouse.column - content_area.x;
                        self.handle_text_selection_update(relative_y as usize, relative_x as usize);
                        return Action::Consumed;
                    }
                }

                Action::None
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.dragging_split = false;
                self.handle_text_selection_end();
                Action::Consumed
            }
            MouseEventKind::Down(MouseButton::Right) => {
                log::info!(
                    "Right click: text_selection={:?}",
                    self.text_selection.is_some()
                );
                if self.text_selection.is_some() {
                    match self.copy_selected_text() {
                        Ok(text) => {
                            log::info!("Copied text to clipboard: {} chars", text.len());
                        }
                        Err(e) => {
                            log::error!("Right click copy failed: {}", e);
                        }
                    }
                    return Action::Consumed;
                }
                Action::None
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

        if let Some((is_dir, _name)) = entry_info {
            if is_dir {
                let _ = self.enter();
                Action::None
            } else {
                let _ = self.open_selected();
                Action::None
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

    fn start_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.search_results.clear();
        self.search_result_index = 0;
    }

    fn start_content_search(&mut self) {
        self.search_mode = true;
        self.content_search_query.clear();
        self.content_search_matches.clear();
        self.content_search_index = 0;
    }

    fn start_delete(&mut self) {
        self.delete_pending = true;
    }

    fn confirm_delete(&mut self) -> anyhow::Result<()> {
        if let Some(entry) = self.get_selected() {
            let path = entry.path.clone();
            if path.exists() {
                if path.is_dir() {
                    // Check if directory is empty
                    let is_empty = path.read_dir()?.next().is_none();
                    if !is_empty {
                        return anyhow::bail!("Cannot delete non-empty directory");
                    }
                    std::fs::remove_dir(&path)?;
                } else {
                    std::fs::remove_file(&path)?;
                }
                log::info!("Deleted: {:?}", path);
                self.refresh()?;
                self.delete_pending = false;
            }
        }
        Ok(())
    }

    fn cancel_delete(&mut self) {
        self.delete_pending = false;
    }

    fn start_rename(&mut self) {
        if let Some(entry) = self.get_selected() {
            self.rename_input = entry.name.clone();
            self.rename_mode = true;
        }
    }

    fn confirm_rename(&mut self) -> anyhow::Result<()> {
        if self.rename_input.is_empty() {
            return anyhow::bail!("Name cannot be empty");
        }

        if let Some(entry) = self.get_selected() {
            let old_path = entry.path.clone();
            let new_path = self.current_path.join(&self.rename_input);

            if !old_path.exists() {
                return anyhow::bail!("Original file not found");
            }

            if new_path.exists() && new_path != old_path {
                return anyhow::bail!("Target already exists");
            }

            std::fs::rename(&old_path, &new_path)?;
            log::info!("Renamed: {:?} -> {:?}", old_path, new_path);
            self.refresh()?;
        }

        self.rename_mode = false;
        self.rename_input.clear();
        Ok(())
    }

    fn cancel_rename(&mut self) {
        self.rename_mode = false;
        self.rename_input.clear();
    }

    fn perform_file_search(&mut self) {
        self.search_results.clear();
        if self.search_query.is_empty() {
            return;
        }
        let query_lower = self.search_query.to_lowercase();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.name.to_lowercase().contains(&query_lower) {
                self.search_results.push(i);
            }
        }
        if !self.search_results.is_empty() {
            self.search_result_index = 0;
            self.selected_index = self.search_results[0];
        }
    }

    fn perform_content_search(&mut self) {
        self.content_search_matches.clear();
        if self.content_search_query.is_empty() || self.file_content.is_none() {
            return;
        }
        let content = self.file_content.as_ref().unwrap();
        let query_lower = self.content_search_query.to_lowercase();
        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                self.content_search_matches.push(i);
            }
        }
        if !self.content_search_matches.is_empty() {
            self.content_search_index = 0;
            self.content_scroll_offset = self.content_search_matches[0];
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                Action::Consumed
            }
            KeyCode::Enter => {
                if self.active_area == ActiveArea::FileList {
                    self.search_result_index =
                        (self.search_result_index + 1) % self.search_results.len().max(1);
                    if !self.search_results.is_empty() {
                        self.selected_index = self.search_results[self.search_result_index];
                    }
                } else {
                    self.content_search_index =
                        (self.content_search_index + 1) % self.content_search_matches.len().max(1);
                    if !self.content_search_matches.is_empty() {
                        self.content_scroll_offset =
                            self.content_search_matches[self.content_search_index];
                    }
                }
                Action::Consumed
            }
            KeyCode::Backspace => {
                if self.active_area == ActiveArea::FileList {
                    self.search_query.pop();
                    self.perform_file_search();
                } else {
                    self.content_search_query.pop();
                    self.perform_content_search();
                }
                Action::Consumed
            }
            KeyCode::Char(c) => {
                if self.active_area == ActiveArea::FileList {
                    self.search_query.push(c);
                    self.perform_file_search();
                } else {
                    self.content_search_query.push(c);
                    self.perform_content_search();
                }
                Action::Consumed
            }
            _ => Action::None,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if self.search_mode {
            return self.handle_search_key(key);
        }

        // Handle rename mode
        if self.rename_mode {
            match key.code {
                KeyCode::Esc => {
                    self.cancel_rename();
                    return Action::Consumed;
                }
                KeyCode::Enter => {
                    match self.confirm_rename() {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Failed to rename: {}", e);
                        }
                    }
                    return Action::Consumed;
                }
                KeyCode::Backspace => {
                    self.rename_input.pop();
                    return Action::Consumed;
                }
                KeyCode::Char(c) => {
                    if !c.is_control() {
                        self.rename_input.push(c);
                    }
                    return Action::Consumed;
                }
                _ => return Action::None,
            }
        }

        // Cancel delete with Esc
        if self.delete_pending && key.code == KeyCode::Esc {
            self.cancel_delete();
            return Action::Consumed;
        }

        // Delete with 'd' key
        if key.code == KeyCode::Char('d') && key.modifiers.is_empty() {
            if !self.delete_pending {
                self.start_delete();
            }
            return Action::Consumed;
        }

        // Rename with 'r' key
        if key.code == KeyCode::Char('r') && key.modifiers.is_empty() {
            if !self.rename_mode {
                self.start_rename();
            }
            return Action::Consumed;
        }

        // Confirm delete with 'y' key when pending
        if self.delete_pending && key.code == KeyCode::Char('y') && key.modifiers.is_empty() {
            match self.confirm_delete() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to delete: {}", e);
                }
            }
            return Action::Consumed;
        }

        // Quick copy with 'y' key (like vim) - only if not in delete pending mode
        if key.code == KeyCode::Char('y') && key.modifiers.is_empty() {
            if let Some((start, end)) = self.text_selection {
                log::info!("Y key: copying selection {} to {}", start, end);
                match self.copy_selected_text() {
                    Ok(text) => {
                        log::info!("Copied to clipboard: {} chars", text.len());
                    }
                    Err(e) => {
                        log::error!("Failed to copy: {}", e);
                    }
                }
                self.text_selection = None;
            }
            return Action::Consumed;
        }

        // Copy file/folder path with 'p' key
        if key.code == KeyCode::Char('p') && key.modifiers.is_empty() {
            match self.copy_path_to_clipboard() {
                Ok(path) => {
                    log::info!("Copied path to clipboard: {}", path);
                }
                Err(e) => {
                    log::error!("Failed to copy path: {}", e);
                }
            }
            return Action::Consumed;
        }

        // Search with '/' key
        if key.code == KeyCode::Char('/') && key.modifiers.is_empty() {
            if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                self.start_content_search();
            } else {
                self.start_search();
            }
            return Action::Consumed;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('c') => {
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    log::info!("Ctrl+C pressed, text_selection={:?}", self.text_selection);
                    if let Some((start, end)) = self.text_selection {
                        match self.copy_selected_text() {
                            Ok(text) => {
                                log::info!("Copied to clipboard: {} chars", text.len());
                            }
                            Err(e) => {
                                log::error!("Failed to copy: {}", e);
                            }
                        }
                        self.text_selection = None;
                    }
                    Action::Consumed
                } else if self.file_content.is_some() {
                    self.close_file();
                    self.active_area = ActiveArea::FileList;
                    Action::None
                } else {
                    Action::None
                }
            }
            KeyCode::Tab => {
                self.active_area = match self.active_area {
                    ActiveArea::FileList => {
                        if self.file_content.is_some() {
                            ActiveArea::Content
                        } else {
                            ActiveArea::FileList
                        }
                    }
                    ActiveArea::Content => ActiveArea::FileList,
                };
                Action::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
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
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                    if let Some(ref content) = self.file_content {
                        self.content_scroll_offset =
                            (self.content_scroll_offset + 1).min(content.lines().count() - 1);
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
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                    let page_size = 20;
                    self.content_scroll_offset =
                        self.content_scroll_offset.saturating_sub(page_size);
                    Action::Consumed
                } else {
                    let page_size = 10;
                    self.selected_index = self.selected_index.saturating_sub(page_size);
                    Action::Consumed
                }
            }
            KeyCode::PageDown => {
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                    let page_size = 20;
                    if let Some(ref content) = self.file_content {
                        let max_offset = content.lines().count().saturating_sub(1);
                        self.content_scroll_offset =
                            (self.content_scroll_offset + page_size).min(max_offset);
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
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                    self.content_scroll_offset = 0;
                    self.content_scroll_offset_x = 0;
                    Action::Consumed
                } else {
                    self.selected_index = 0;
                    Action::Consumed
                }
            }
            KeyCode::End => {
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                    if let Some(ref content) = self.file_content {
                        self.content_scroll_offset = content.lines().count().saturating_sub(1);
                    }
                    Action::Consumed
                } else {
                    self.selected_index = self.entries.len().saturating_sub(1);
                    Action::Consumed
                }
            }
            KeyCode::Left => {
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                    if self.content_scroll_offset_x > 0 {
                        self.content_scroll_offset_x -= 1;
                    }
                    Action::Consumed
                } else {
                    Action::None
                }
            }
            KeyCode::Right => {
                if self.active_area == ActiveArea::Content && self.file_content.is_some() {
                    self.content_scroll_offset_x += 1;
                    Action::Consumed
                } else {
                    Action::None
                }
            }
            KeyCode::Enter => {
                let entry_info = self.get_selected().map(|entry| entry.is_dir);

                if let Some(is_dir) = entry_info {
                    if is_dir {
                        let _ = self.enter();
                    } else {
                        let _ = self.open_selected();
                    }
                }
                Action::None
            }
            KeyCode::Char('o') => {
                let entry_info = self.get_selected();

                if entry_info.is_some() {
                    let _ = self.open_selected();
                }
                Action::None
            }
            KeyCode::Char('u') | KeyCode::Backspace => {
                let _ = self.go_up();
                Action::None
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
                let sort_mode = match self.sort_by {
                    SortBy::Name => "name",
                    SortBy::Size => "size",
                    SortBy::Modified => "modified",
                };
                Action::ShowMessage(core::event::Message::Info(
                format!("j/k: Navigate | Enter: Enter | o: Open | d: Delete | r: Rename | c/Esc: Close | h: Hidden | s: Sort ({}) | u: Up | Tab: Focus | p: Path | y: Text | /: Search", sort_mode)
            ))
            }
            _ => Action::None,
        }
    }

    pub fn get_status(&self) -> String {
        if self.file_content.is_some() {
            if let Some(path_str) = &self.selected_file_path {
                let path = std::path::Path::new(path_str);
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let line = self.content_scroll_offset + 1;
                let total = self
                    .file_content
                    .as_ref()
                    .map(|c| c.lines().count())
                    .unwrap_or(0);
                return format!("{}: Line {} / {}", file_name, line, total);
            }
        } else if let Some(entry) = self.get_selected() {
            return entry.name.clone();
        }
        "File Browser".to_string()
    }

    pub fn copy_selected_text(&self) -> Result<String, String> {
        let (sel_start, sel_end) = self.text_selection.ok_or("No text selected")?;
        let content = self.file_content.as_ref().ok_or("No file content")?;

        let start = sel_start.min(sel_end);
        let end = sel_start.max(sel_end);

        log::info!(
            "Copy: original=({},{}), start={}, end={}, content_len={}",
            sel_start,
            sel_end,
            start,
            end,
            content.len()
        );

        if end > content.len() {
            return Err(format!(
                "Selection {} exceeds content {}",
                end,
                content.len()
            ));
        }
        if start >= end {
            return Err(format!("Invalid selection: {} >= {}", start, end));
        }

        // Convert byte positions to character boundaries to avoid UTF-8 panics
        let safe_start = content
            .char_indices()
            .find(|(b, _)| *b >= start)
            .map(|(b, _)| b)
            .unwrap_or(start);

        let safe_end = content
            .char_indices()
            .find(|(b, _)| *b >= end)
            .map(|(b, _)| b)
            .unwrap_or(end);

        log::info!(
            "Adjusted byte positions: {}..{} -> {}..{}",
            start,
            end,
            safe_start,
            safe_end
        );

        let selected_text = content[safe_start..safe_end].to_string();
        log::info!(
            "Selected text length: {}, preview: {} chars",
            selected_text.chars().count(),
            if selected_text.chars().count() > 30 {
                format!("{}...", selected_text.chars().take(30).collect::<String>())
            } else {
                selected_text.clone()
            }
        );

        // Try clipboard
        use copypasta::ClipboardProvider;
        match copypasta::ClipboardContext::new() {
            Ok(mut ctx) => {
                log::info!("Clipboard context created successfully");
                match ctx.set_contents(selected_text.clone()) {
                    Ok(_) => {
                        log::info!("Copied to clipboard");

                        // Verify by reading back
                        match ctx.get_contents() {
                            Ok(read_back) => {
                                if read_back == selected_text {
                                    log::info!("Clipboard verification: OK");
                                } else {
                                    log::error!("Clipboard verification FAILED");
                                }
                            }
                            Err(e) => {
                                log::error!("Clipboard verification read failed: {:?}", e);
                            }
                        }

                        Ok(selected_text)
                    }
                    Err(e) => {
                        log::error!("Clipboard copy failed: {:?}", e);
                        let _ = self.save_to_temp_file(&selected_text);
                        return Ok(selected_text);
                    }
                }
            }
            Err(e) => {
                log::error!("Clipboard unavailable: {:?}", e);
                let _ = self.save_to_temp_file(&selected_text);
                Ok(selected_text)
            }
        }
    }

    fn save_to_temp_file(&self, text: &str) -> String {
        let temp_path = std::path::PathBuf::from("/tmp/tuiworker_copied_text.txt");
        match std::fs::write(&temp_path, text) {
            Ok(_) => format!("Saved to {}", temp_path.display()),
            Err(e) => format!("Failed to save: {}", e),
        }
    }

    pub fn copy_path_to_clipboard(&self) -> Result<String, String> {
        let path = if let Some(ref file_path) = self.selected_file_path {
            file_path.clone()
        } else if let Some(entry) = self.get_selected() {
            let full_path = self.current_path.join(&entry.name);
            full_path.to_string_lossy().to_string()
        } else {
            return Err("No file or folder selected".to_string());
        };

        log::info!("Copying path to clipboard: {}", path);

        use copypasta::ClipboardProvider;
        match copypasta::ClipboardContext::new() {
            Ok(mut ctx) => match ctx.set_contents(path.clone()) {
                Ok(_) => {
                    let _ = ctx.get_contents();
                    Ok(path)
                }
                Err(e) => Err(format!("Failed to copy path: {:?}", e)),
            },
            Err(e) => Err(format!("Clipboard unavailable: {:?}", e)),
        }
    }

    pub fn handle_text_selection_start(&mut self, line: usize, col: usize) {
        if let Some(content) = &self.file_content {
            self.selection_start_line = Some(line);
            self.selection_start_col = Some(col);
            self.selecting = true;

            // Calculate byte position
            let byte_pos = self.calculate_byte_position(
                content,
                line + self.content_scroll_offset,
                col + self.content_scroll_offset_x,
            );
            self.text_selection = Some((byte_pos, byte_pos));
        }
    }

    pub fn handle_text_selection_update(&mut self, line: usize, col: usize) {
        if self.selecting && self.text_selection.is_some() {
            if let Some(content) = &self.file_content {
                let (start, _) = self.text_selection.unwrap();
                let end_pos = self.calculate_byte_position(
                    content,
                    line + self.content_scroll_offset,
                    col + self.content_scroll_offset_x,
                );
                self.text_selection = Some((start, end_pos));
            }
        }
    }

    pub fn handle_text_selection_end(&mut self) {
        self.selecting = false;
    }

    fn calculate_byte_position(&self, content: &str, line: usize, col: usize) -> usize {
        let lines: Vec<&str> = content.lines().collect();
        let mut byte_pos = 0;

        for (i, l) in lines.iter().enumerate() {
            if i < line {
                byte_pos += l.len() + 1; // +1 for newline
            } else if i == line {
                let col_chars: Vec<char> = l.chars().take(col).collect();
                byte_pos += col_chars.len();
                break;
            }
        }

        byte_pos.min(content.len())
    }

    fn calculate_line_start_byte(&self, lines: &[&str], line_idx: usize) -> usize {
        lines.iter().take(line_idx).map(|l| l.len() + 1).sum()
    }

    fn render_line_with_selection(
        &self,
        line: &str,
        line_start_byte: usize,
        sel_start: usize,
        sel_end: usize,
        scroll_x: usize,
        is_active: bool,
    ) -> Vec<Span> {
        let mut result = Vec::new();
        let chars: Vec<char> = line.chars().collect();

        let selection_bg = if is_active {
            Color::Green
        } else {
            Color::DarkGray
        };

        let mut current_byte = line_start_byte;

        for ch in &chars {
            let char_byte_len = ch.len_utf8();
            let char_end_byte = current_byte + char_byte_len;
            let is_selected = char_end_byte > sel_start && current_byte < sel_end;

            if is_selected {
                result.push(Span::styled(
                    ch.to_string(),
                    Style::default().bg(selection_bg),
                ));
            } else {
                result.push(Span::raw(ch.to_string()));
            }

            current_byte = char_end_byte;
        }

        if scroll_x > 0 {
            let chars_to_skip = chars.len().min(scroll_x);
            result = result.into_iter().skip(chars_to_skip).collect();
        }

        result
    }

    pub fn get_selected_text(&self) -> Option<String> {
        if let Some((start, end)) = self.text_selection {
            if let Some(content) = &self.file_content {
                let start = start.min(end);
                let end = end.max(start);
                if content.len() >= end {
                    return Some(content[start..end].to_string());
                }
            }
        }
        None
    }

    pub fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "j/k",
                description: "Navigate up/down",
            },
            Shortcut {
                key: "Enter",
                description: "Enter directory/file",
            },
            Shortcut {
                key: "o",
                description: "Open file in external app",
            },
            Shortcut {
                key: "c/Esc",
                description: "Close file",
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
                description: "Sort by name/size",
            },
            Shortcut {
                key: "m",
                description: "Sort by modified",
            },
            Shortcut {
                key: "/",
                description: "Search files",
            },
            Shortcut {
                key: "d",
                description: "Delete file (press d, then y)",
            },
            Shortcut {
                key: "r",
                description: "Rename (type new name, Enter)",
            },
            Shortcut {
                key: "Tab",
                description: "Switch file list/content",
            },
            Shortcut {
                key: "PageUp/Down",
                description: "Scroll page",
            },
            Shortcut {
                key: "Home/End",
                description: "Go to start/end",
            },
            Shortcut {
                key: "y",
                description: "Copy selected text",
            },
            Shortcut {
                key: "p",
                description: "Copy file/folder path",
            },
            Shortcut {
                key: "Drag in content",
                description: "Select text",
            },
        ]
    }
}
