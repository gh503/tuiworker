//! Git module - Git repository integration and visualization

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use git2::{
    BranchType, Commit, Diff, DiffOptions, ObjectType, Oid, Repository, Status, StatusOptions, Time,
};

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
};

use ui::Theme;

/// Git commit information
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Local>,
    pub parents: Vec<String>,
}

impl From<&Commit<'_>> for GitCommit {
    fn from(commit: &Commit) -> Self {
        let id = commit.id().to_string();
        let short_id = commit
            .as_object()
            .short_id()
            .unwrap_or_default()
            .as_str()
            .unwrap_or("<short>")
            .to_string();
        let message = commit.message().unwrap_or("<no message>").to_string();
        let author = commit.author().name().unwrap_or("<unknown>").to_string();

        let time = Time::new(commit.time().seconds(), 0);
        let timestamp = DateTime::from_timestamp(time.seconds(), 0)
            .unwrap()
            .with_timezone(&Local);

        let parents = commit
            .parent_ids()
            .map(|id| id.to_string())
            .collect::<Vec<_>>();

        Self {
            id,
            short_id,
            message,
            author,
            timestamp,
            parents,
        }
    }
}

/// Git file status
#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: Status,
}

/// Repository view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Status,
    Log,
    Diff,
    Branches,
}

/// Main Git module
pub struct GitModule {
    repo_path: Option<PathBuf>,
    repo: Option<Repository>,
    commits: Vec<GitCommit>,
    file_statuses: Vec<FileStatus>,
    branches: Vec<String>,
    current_branch: String,
    selected_index: usize,
    view_mode: ViewMode,
    theme: Theme,
}

impl GitModule {
    pub fn new(repo_path: Option<PathBuf>) -> Self {
        Self {
            repo_path,
            repo: None,
            commits: Vec::new(),
            file_statuses: Vec::new(),
            branches: Vec::new(),
            current_branch: String::new(),
            selected_index: 0,
            view_mode: ViewMode::Status,
            theme: Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Open repository
    pub fn open_repository(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let repo = Repository::open(&path)?;
        self.repo_path = Some(path.clone());
        self.repo = Some(repo);

        self.load_repository_info()?;
        Ok(())
    }

    /// Load repository information
    fn load_repository_info(&mut self) -> anyhow::Result<()> {
        let repo_path = self.repo.as_ref().map(|r| r.path().to_path_buf());

        if let Some(ref repo) = self.repo {
            self.commits = self.load_commits(repo, 100)?;
            self.file_statuses = self.load_file_status(repo)?;
        }

        if let Some(path) = repo_path {
            let repo = Repository::open(&path)?;
            self.load_branches(&repo)?;
            self.current_branch = self.get_current_branch(&repo)?;
        }
        Ok(())
    }

    /// Load commit history
    fn load_commits(&self, repo: &Repository, limit: usize) -> anyhow::Result<Vec<GitCommit>> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            commits.push(GitCommit::from(&commit));
        }

        Ok(commits)
    }

    /// Load file status
    fn load_file_status(&self, repo: &Repository) -> anyhow::Result<Vec<FileStatus>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut opts))?;

        let file_statuses: Vec<FileStatus> = statuses
            .iter()
            .map(|entry| {
                let path = entry.path().unwrap_or("<unknown>").to_string();
                let status = entry.status();
                FileStatus { path, status }
            })
            .collect();

        Ok(file_statuses)
    }

    /// Load branches
    fn load_branches(&mut self, repo: &Repository) -> anyhow::Result<()> {
        let branches = repo.branches(Some(BranchType::Local))?;

        self.branches = branches
            .filter_map(|b| b.ok())
            .filter(|(b, _)| b.is_head())
            .map(|(b, _)| match b.name() {
                Ok(Some(n)) => n.to_string(),
                _ => "<unknown>".to_string(),
            })
            .collect();

        Ok(())
    }

    /// Get current branch name
    fn get_current_branch(&self, repo: &Repository) -> anyhow::Result<String> {
        let head = repo.head()?;
        let name = match head.name() {
            Some(n) => n.to_string(),
            None => "<detached>".to_string(),
        };
        Ok(name)
    }

    /// Switch view mode
    pub fn switch_view(&mut self, view: ViewMode) {
        self.view_mode = view;
        self.selected_index = 0;
    }

    /// Navigate up
    pub fn navigate_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    /// Navigate down
    pub fn navigate_down(&mut self) {
        let max_index = match self.view_mode {
            ViewMode::Status => self.file_statuses.len().saturating_sub(1),
            ViewMode::Log => self.commits.len().saturating_sub(1),
            ViewMode::Diff => 0,
            ViewMode::Branches => self.branches.len().saturating_sub(1),
        };
        self.selected_index = (self.selected_index + 1).min(max_index);
    }

    /// Get status badge style
    fn get_status_style(&self, status: Status) -> Style {
        let (fg, bg) = if status.is_index_new() {
            (Color::Green, Color::Reset)
        } else if status.is_index_modified() {
            (Color::Yellow, Color::Reset)
        } else if status.is_index_deleted() {
            (Color::Red, Color::Reset)
        } else if status.is_wt_new() {
            (Color::LightGreen, Color::Reset)
        } else if status.is_wt_modified() {
            (Color::LightYellow, Color::Reset)
        } else if status.is_wt_deleted() {
            (Color::LightRed, Color::Reset)
        } else if status.is_index_renamed() {
            (Color::Cyan, Color::Reset)
        } else {
            (Color::Gray, Color::Reset)
        };

        Style::default().fg(fg).bg(bg)
    }

    /// Get status badge text
    fn get_status_badge(&self, status: Status) -> String {
        let mut badges = Vec::new();

        if status.is_index_new() {
            badges.push("A");
        }
        if status.is_index_modified() {
            badges.push("M");
        }
        if status.is_index_deleted() {
            badges.push("D");
        }
        if status.is_wt_new() {
            badges.push("??");
        }
        if status.is_wt_modified() {
            badges.push(" M");
        }
        if status.is_wt_deleted() {
            badges.push(" D");
        }

        badges.join("")
    }

    /// Draw status view
    fn draw_status_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("当前分支: ", Style::default().fg(self.theme.muted())),
                Span::styled(
                    &self.current_branch,
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::default(),
        ];

        lines.push(Line::from(vec![Span::styled(
            format!("文件状态 ({} 项)", self.file_statuses.len()),
            Style::default()
                .fg(self.theme.primary())
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::default());

        if self.file_statuses.is_empty() {
            lines.push(Line::from("工作区干净，没有任何更改"));
        } else {
            for (i, status) in self.file_statuses.iter().enumerate() {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    Style::default().bg(self.theme.primary()).fg(Color::Black)
                } else {
                    Style::default()
                };

                let badge = self.get_status_badge(status.status);
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{badge:<4} ",),
                        self.get_status_style(status.status),
                    ),
                    Span::styled(&status.path, style),
                ]));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("仓库状态")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw log view
    fn draw_log_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "提交历史",
                Style::default()
                    .fg(self.theme.primary())
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
        ];

        if self.commits.is_empty() {
            lines.push(Line::from("没有提交记录"));
        } else {
            for (i, commit) in self.commits.iter().enumerate() {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    Style::default().bg(self.theme.primary()).fg(Color::Black)
                } else {
                    Style::default()
                };

                let time_str = commit.timestamp.format("%Y-%m-%d %H:%M").to_string();

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} ", commit.short_id),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(commit.message.lines().next().unwrap_or(""), style),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(
                        format!("{} | {}", commit.author, time_str),
                        Style::default().fg(self.theme.muted()),
                    ),
                ]));
                lines.push(Line::default());
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("提交日志")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw diff view
    fn draw_diff_view(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from(vec![Span::styled(
                "差异对比",
                Style::default()
                    .fg(self.theme.primary())
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
            Line::from("按 Enter 选择文件查看变更"),
        ];

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("差异")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Draw branches view
    fn draw_branches_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "分支列表",
                Style::default()
                    .fg(self.theme.primary())
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
        ];

        if self.branches.is_empty() {
            lines.push(Line::from("没有分支"));
        } else {
            for (i, branch) in self.branches.iter().enumerate() {
                let is_selected = i == self.selected_index;
                let is_current = branch == &self.current_branch;

                let style = if is_selected {
                    Style::default()
                        .bg(self.theme.primary())
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let prefix = if is_current { "*" } else { " " };
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Green)),
                    Span::styled(format!(" {}", branch), style),
                ]));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("分支")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw help bar
    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help = "1:状态 2:日志 3:差异 4:分支 j/k:n/↑↓:导航 Enter:查看详情 q:退出";
        let paragraph = Paragraph::new(help).style(Style::default().fg(self.theme.muted()));
        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('1') => {
                self.switch_view(ViewMode::Status);
                Action::None
            }
            KeyCode::Char('2') => {
                self.switch_view(ViewMode::Log);
                Action::None
            }
            KeyCode::Char('3') => {
                self.switch_view(ViewMode::Diff);
                Action::None
            }
            KeyCode::Char('4') => {
                self.switch_view(ViewMode::Branches);
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
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }
}

impl CoreModule for GitModule {
    fn name(&self) -> &str {
        "git"
    }

    fn title(&self) -> &str {
        "Git"
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

        // Draw content based on view mode
        match self.view_mode {
            ViewMode::Status => self.draw_status_view(frame, layout[0]),
            ViewMode::Log => self.draw_log_view(frame, layout[0]),
            ViewMode::Diff => self.draw_diff_view(frame, layout[0]),
            ViewMode::Branches => self.draw_branches_view(frame, layout[0]),
        }

        // Draw help bar
        self.draw_help_bar(frame, layout[1]);
    }

    fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        if let Some(ref path) = self.repo_path {
            self.open_repository(path.clone())?;
        } else {
            // Try to open repository from current directory
            if let Ok(repo) = Repository::open(".") {
                self.repo_path = Some(std::env::current_dir()?);
                self.repo = Some(repo);
                self.load_repository_info()?;
            }
        }
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "1",
                description: "状态",
            },
            Shortcut {
                key: "2",
                description: "日志",
            },
            Shortcut {
                key: "3",
                description: "差异",
            },
            Shortcut {
                key: "4",
                description: "分支",
            },
            Shortcut {
                key: "j/k",
                description: "上下导航",
            },
            Shortcut {
                key: "Enter",
                description: "详情",
            },
        ]
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.load()?;
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub use GitModule as Git;
