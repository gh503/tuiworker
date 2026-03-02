//! Mail module - IMAP/SMTP email client

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use std::{collections::HashMap, io::BufReader};

use chrono::{DateTime, Local, Utc};

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
    ui::Theme,
};

/// Email structure
#[derive(Debug, Clone)]
pub struct Email {
    pub id: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub date: DateTime<Local>,
    pub body: String,
    pub read: bool,
}

/// Email folder
#[derive(Debug, Clone)]
pub struct EmailFolder {
    pub name: String,
    pub count: usize,
    pub unread: usize,
}

/// View mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Folders,
    EmailList,
    EmailView,
    Compose,
}

/// Main mail module
pub struct MailModule {
    folders: Vec<EmailFolder>,
    emails: Vec<Email>,
    selected_folder_index: usize,
    selected_email_index: usize,
    view_mode: ViewMode,
    compose_to: String,
    compose_subject: String,
    compose_body: String,
    theme: Theme,

    // Connection settings (placeholder)
    imap_server: String,
    imap_port: u16,
    smtp_server: String,
    smtp_port: u16,
}

impl MailModule {
    pub fn new() -> Self {
        let folders = vec![
            EmailFolder {
                name: "INBOX".to_string(),
                count: 0,
                unread: 0,
            },
            EmailFolder {
                name: "Sent".to_string(),
                count: 0,
                unread: 0,
            },
            EmailFolder {
                name: "Drafts".to_string(),
                count: 0,
                unread: 0,
            },
            EmailFolder {
                name: "Trash".to_string(),
                count: 0,
                unread: 0,
            },
        ];

        Self {
            folders,
            emails: Vec::new(),
            selected_folder_index: 0,
            selected_email_index: 0,
            view_mode: ViewMode::Folders,
            compose_to: String::new(),
            compose_subject: String::new(),
            compose_body: String::new(),
            theme: Theme::default(),
            imap_server: "imap.example.com".to_string(),
            imap_port: 993,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Switch view mode
    pub fn switch_view(&mut self, view: ViewMode) {
        self.view_mode = view;
        if view == ViewMode::EmailList {
            self.selected_email_index = 0;
        }
    }

    /// Navigate up
    pub fn navigate_up(&mut self) {
        match self.view_mode {
            ViewMode::Folders => {
                self.selected_folder_index = self.selected_folder_index.saturating_sub(1);
            }
            ViewMode::EmailList => {
                self.selected_email_index = self.selected_email_index.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Navigate down
    pub fn navigate_down(&mut self) {
        match self.view_mode {
            ViewMode::Folders => {
                self.selected_folder_index = (self.selected_folder_index + 1)
                    .min(self.folders.len().saturating_sub(1));
            }
            ViewMode::EmailList => {
                self.selected_email_index = (self.selected_email_index + 1)
                    .min(self.emails.len().saturating_sub(1));
            }
            _ => {}
        }
    }

    /// Select folder and load emails
    pub fn select_folder(&mut self, index: usize) {
        if index < self.folders.len() {
            self.selected_folder_index = index;
            self.emails = Vec::new();
            // In a real implementation, connect to IMAP and fetch emails
            // For now, add placeholder emails for demonstration
            self.load_placeholder_emails();
            self.view_mode = ViewMode::EmailList;
        }
    }

    /// Select email for viewing
    pub fn select_email(&mut self, index: usize) {
        if index < self.emails.len() {
            self.selected_email_index = index;
            self.emails[index].read = true;
            self.view_mode = ViewMode::EmailView;
        }
    }

    /// Load placeholder emails for demo
    fn load_placeholder_emails(&mut self) {
        use chrono::Utc;

        self.emails = vec![
            Email {
                id: "1".to_string(),
                subject: "项目更新 - TUI Workstation 需求确认".to_string(),
                from: "user@example.com".to_string(),
                to: "me@example.com".to_string(),
                date: Utc::now().with_timezone(&Local),
                body: "你好，\n\n关于 TUI Workstation 项目，我们需要确认以下需求：\n\n1. 完全使用 Rust 实现\n2. 支持多个功能模块\n3. 模块化架构设计\n\n请确认以上需求是否正确。\n\n谢谢！".to_string(),
                read: false,
            },
            Email {
                id: "2".to_string(),
                subject: "周报 - 2025年第8周".to_string(),
                from: "boss@example.com".to_string(),
                to: "me@example.com".to_string(),
                date: Utc::now().with_timezone(&Local) - chrono::Duration::days(1),
                body: "本周工作总结：\n\n1. 完成了 TUI Workstation 的核心架构设计\n2. 实现了 5 个功能模块\n3. 编写了详细的 API 文档\n\n下周计划：\n- 完成剩余模块实现\n- 进行跨平台测试\n- 准备发布 v0.1.0".to_string(),
                read: true,
            },
            Email {
                id: "3".to_string(),
                subject: "Rust 最佳实践分享".to_string(),
                from: "rust@example.com".to_string(),
                to: "team@example.com".to_string(),
                date: Utc::now().with_timezone(&Local) - chrono::Duration::days(3),
                body: "大家好，\n\n本周我们将进行 Rust 最佳实践分享，涵盖以下主题：\n\n1. 错误处理模式\n2. 并发编程技巧\n3. 内存管理优化\n4. 性能调优方法\n\n时间：周五下午 3 点\n地点：会议室 A".to_string(),
                read: false,
            },
        ];

        // Update folder counts
        if self.selected_folder_index < self.folders.len() {
            self.folders[self.selected_folder_index].count = self.emails.len();
            self.folders[self.selected_folder_index].unread = self
                .emails
                .iter()
                .filter(|e| !e.read)
                .count();
        }
    }

    /// Start composing new email
    pub fn compose(&mut self) {
        self.compose_to = String::new();
        self.compose_subject = String::new();
        self.compose_body = String::new();
        self.view_mode = ViewMode::Compose;
    }

    /// Send composed email
    pub fn send_email(&mut self) -> anyhow::Result<()> {
        // In a real implementation, use lettre to send via SMTP
        log::info!(
            "Sending email to: {}, subject: {}",
            self.compose_to,
            self.compose_subject
        );

        // Save to Sent folder
        let sent_email = Email {
            id: uuid::Uuid::new_v4().to_string(),
            subject: self.compose_subject.clone(),
            from: "me@example.com".to_string(),
            to: self.compose_to.clone(),
            date: Local::now(),
            body: self.compose_body.clone(),
            read: true,
        };

        // Update Sent folder count
        if let Some(sent_folder) = self.folders.iter_mut().find(|f| f.name == "Sent") {
            sent_folder.count += 1;
        }

        // Clear compose fields
        self.compose_to = String::new();
        self.compose_subject = String::new();
        self.compose_body = String::new();

        // Switch back to folder view
        self.view_mode = ViewMode::Folders;

        Ok(())
    }

    /// Draw folders view
    fn draw_folders_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "邮箱文件夹",
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::default(),
        ];

        for (i, folder) in self.folders.iter().enumerate() {
            let is_selected = i == self.selected_folder_index;
            let style = if is_selected {
                Style::default()
                    .bg(self.theme.primary())
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let unread_badge = if folder.unread > 0 {
                format!(" ({})", folder.unread)
            } else {
                String::new()
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("📁 {}{} ({})", folder.name, unread_badge, folder.count),
                    style,
                ),
            ]));
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("文件夹")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw email list view
    fn draw_email_list_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    format!(
                        "{} ({})",
                        self.folders[self.selected_folder_index].name,
                        self.emails.len()
                    ),
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::default(),
        ];

        if self.emails.is_empty() {
            lines.push(Line::from("此文件夹为空"));
        } else {
            for (i, email) in self.emails.iter().enumerate() {
                let is_selected = i == self.selected_email_index;
                let read_marker = if email.read { "  " } else { "● " };
                let is_unread = !email.read;

                let style = if is_selected {
                    Style::default()
                        .bg(self.theme.primary())
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else if is_unread {
                    Style::default().fg(self.theme.primary())
                } else {
                    Style::default().fg(self.theme.text())
                };

                lines.push(Line::from(vec![
                    Span::styled(read_marker, Style::default().fg(if is_unread { Color::Blue } else { Color::Reset })),
                    Span::styled(
                        truncate_string(&email.subject, 50),
                        style.add_modifier(if is_unread { Modifier::BOLD } else { Modifier::empty() }),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(
                        format!("{} | {}", email.from, email.date.format("%Y-%m-%d %H:%M")),
                        Style::default().fg(self.theme.muted()),
                    ),
                ]));
                lines.push(Line::default());
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("邮件列表")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw email view
    fn draw_email_view(&self, frame: &mut Frame, area: Rect) {
        if let Some(email) = self.emails.get(self.selected_email_index) {
            let header_lines = vec![
                Line::from(vec![
                    Span::styled("主题: ", Style::default().fg(self.theme.muted())),
                    Span::styled(&email.subject, Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("发件人: ", Style::default().fg(self.theme.muted())),
                    Span::styled(&email.from, Style::default()),
                ]),
                Line::from(vec![
                    Span::styled("收件人: ", Style::default().fg(self.theme.muted())),
                    Span::styled(&email.to, Style::default()),
                ]),
                Line::from(vec![
                    Span::styled("日期: ", Style::default().fg(self.theme.muted())),
                    Span::styled(
                        email.date.format("%Y-%m-%d %H:%M:%S").to_string(),
                        Style::default(),
                    ),
                ]),
                Line::default(),
            ];

            let body_lines: Vec<Line> = email.body.lines().map(|l| Line::from(l)).collect();

            let mut lines = header_lines;
            lines.extend(body_lines);

            let paragraph = Paragraph::new(Text::from(lines))
                .block(
                    Block::default()
                        .title("邮件内容")
                        .borders(Borders::ALL)
                        .border_style(self.theme.border()),
                )
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, area);
        }
    }

    /// Draw compose view
    fn draw_compose_view(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // To
                Constraint::Length(3),  // Subject
                Constraint::Min(5),    // Body
                Constraint::Length(1), // Help
            ])
            .split(area);

        // To field
        let to_text = vec![
            Line::from("收件人: "),
            Line::from(self.compose_to.clone()),
        ];
        let to_paragraph = Paragraph::new(Text::from(to_text))
            .block(
                Block::default()
                    .title("收件人")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            );
        frame.render_widget(to_paragraph, layout[0]);

        // Subject field
        let subject_text = vec![
            Line::from("主题: "),
            Line::from(self.compose_subject.clone()),
        ];
        let subject_paragraph = Paragraph::new(Text::from(subject_text))
            .block(
                Block::default()
                    .title("主题")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            );
        frame.render_widget(subject_paragraph, layout[1]);

        // Body field
        let body_lines: Vec<Line> = self
            .compose_body
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();
        let body_paragraph = Paragraph::new(Text::from(body_lines))
            .block(
                Block::default()
                    .title("正文")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(body_paragraph, layout[2]);

        // Help
        let help = "Ctrl+S: 发送 Esc: 取消返回";
        let help_paragraph = Paragraph::new(help)
            .style(Style::default().fg(self.theme.muted()));
        frame.render_widget(help_paragraph, layout[3]);
    }

    /// Draw help bar
    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help = match self.view_mode {
            ViewMode::Folders => "j/k:导航 Enter:选择 q:退出",
            ViewMode::EmailList => "j/k:导航 Enter:查看 c:写邮件 q:退出",
            ViewMode::EmailView => "Esc:返回列表 d:删除 r:回复 f:转发",
            ViewMode::Compose => "Ctrl+S:发送 Esc:取消",
        };

        let paragraph = Paragraph::new(help)
            .style(Style::default().fg(self.theme.muted()));
        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
                Action::None
            }
            KeyCode::Enter => {
                match self.view_mode {
                    ViewMode::Folders => {
                        self.select_folder(self.selected_folder_index);
                    }
                    ViewMode::EmailList => {
                        self.select_email(self.selected_email_index);
                    }
                    _ => {}
                }
                Action::None
            }
            KeyCode::Esc => {
                match self.view_mode {
                    ViewMode::EmailList | ViewMode::Compose => {
                        self.view_mode = ViewMode::Folders;
                    }
                    ViewMode::EmailView => {
                        self.view_mode = ViewMode::EmailList;
                    }
                    _ => {
                        return Action::Quit;
                    }
                }
                Action::None
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.view_mode == ViewMode::EmailList {
                    self.compose();
                }
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => Action::Quit,
            _ => Action::None,
        }
    }
}

/// Helper to truncate string
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

impl CoreModule for MailModule {
    fn name(&self) -> &str {
        "mail"
    }

    fn title(&self) -> &str {
        "邮件"
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
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        // Draw based on view mode
        match self.view_mode {
            ViewMode::Folders => self.draw_folders_view(frame, layout[0]),
            ViewMode::EmailList => self.draw_email_list_view(frame, layout[0]),
            ViewMode::EmailView => self.draw_email_view(frame, layout[0]),
            ViewMode::Compose => self.draw_compose_view(frame, layout[0]),
        }

        // Draw help bar
        self.draw_help_bar(frame, layout[1]);
    }

    fn save(&self) -> anyhow::Result<()> { Ok(()) }

    fn load(&mut self) -> anyhow::Result<()> { Ok(()) }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "j/k",
                description: "导航",
            },
            Shortcut {
                key: "Enter",
                description: "选择/查看",
            },
            Shortcut {
                key: "Esc",
                description: "返回",
            },
            Shortcut {
                key: "c",
                description: "写邮件",
            },
        ]
    }

    fn init(&mut self) -> anyhow::Result<()> {
        // In a real implementation, connect to IMAP server
        // For now, load placeholder data
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> { Ok(()) }
}

pub use MailModule as Mail;
