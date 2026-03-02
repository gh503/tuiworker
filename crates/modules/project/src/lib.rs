//! Project module - Project management with milestones, risks, and progress tracking

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, ListItem, List, Paragraph, Wrap},
    Frame,
};

use std::{collections::HashMap, path::PathBuf};

use uuid::Uuid;

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
    ui::Theme,
};

/// Project status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProjectStatus {
    NotStarted,
    InProgress,
    OnHold,
    Completed,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk level
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee: String,
}

/// Milestone
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Milestone {
    pub id: String,
    pub title: String,
    pub description: String,
    pub due_date: String,
    pub tasks: Vec<Task>,
}

/// Risk
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Risk {
    pub id: String,
    pub title: String,
    pub description: String,
    pub level: RiskLevel,
    pub impact: String,
    pub mitigation: String,
}

/// Project
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub milestones: Vec<Milestone>,
    pub risks: Vec<Risk>,
}

/// View mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Projects,
    Milestones,
    Risks,
}

/// Main project module
pub struct ProjectModule {
    projects: Vec<Project>,
    current_project_index: Option<usize>,
    selected_index: usize,
    view_mode: ViewMode,
    project_dir: PathBuf,
    theme: Theme,
}

impl ProjectModule {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            projects: Vec::new(),
            current_project_index: None,
            selected_index: 0,
            view_mode: ViewMode::Projects,
            project_dir,
            theme: Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Load projects
    pub fn load_projects(&mut self) -> anyhow::Result<()> {
        if !self.project_dir.exists() {
            std::fs::create_dir_all(&self.project_dir)?;
        }

        for entry in std::fs::read_dir(&self.project_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(project) = serde_json::from_str::<Project>(&content) {
                        self.projects.push(project);
                    }
                }
            }
        }

        Ok(())
    }

    /// Save projects
    pub fn save_projects(&self) -> anyhow::Result<()> {
        if !self.project_dir.exists() {
            std::fs::create_dir_all(&self.project_dir)?;
        }

        for project in &self.projects {
            let filename = format!("{}.json", project.id);
            let filepath = self.project_dir.join(filename);
            let json = serde_json::to_string_pretty(project)?;
            std::fs::write(filepath, json)?;
        }

        Ok(())
    }

    /// Create new project
    pub fn create_project(&mut self, name: String, description: String) -> anyhow::Result<()> {
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            status: ProjectStatus::NotStarted,
            milestones: Vec::new(),
            risks: Vec::new(),
        };

        self.save_project(&project)?;
        self.projects.push(project);
        self.current_project_index = Some(self.projects.len() - 1);
        self.selected_index = self.projects.len() - 1;

        Ok(())
    }

    /// Save single project
    fn save_project(&self, project: &Project) -> anyhow::Result<()> {
        if !self.project_dir.exists() {
            std::fs::create_dir_all(&self.project_dir)?;
        }

        let filename = format!("{}.json", project.id);
        let filepath = self.project_dir.join(filename);
        let json = serde_json::to_string_pretty(project)?;
        std::fs::write(filepath, json)?;

        Ok(())
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
            ViewMode::Projects => self.projects.len().saturating_sub(1),
            ViewMode::Milestones => {
                if let Some(idx) = self.current_project_index {
                    if let Some(project) = self.projects.get(idx) {
                        project.milestones.len().saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            ViewMode::Risks => {
                if let Some(idx) = self.current_project_index {
                    if let Some(project) = self.projects.get(idx) {
                        project.risks.len().saturating_sub(1)
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        };
        self.selected_index = (self.selected_index + 1).min(max_index);
    }

    /// Calculate project progress
    fn calculate_progress(&self, project: &Project) -> f32 {
        let total_tasks: usize = project
            .milestones
            .iter()
            .map(|m| m.tasks.len())
            .sum();

        if total_tasks == 0 {
            return 0.0;
        }

        let completed_tasks: usize = project
            .milestones
            .iter()
            .map(|m| m.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count())
            .sum();

        (completed_tasks as f32 / total_tasks as f32) * 100.0
    }

    /// Get status style
    fn get_status_style(status: ProjectStatus) -> Style {
        match status {
            ProjectStatus::NotStarted => Style::default().fg(Color::Gray),
            ProjectStatus::InProgress => Style::default().fg(Color::Blue),
            ProjectStatus::OnHold => Style::default().fg(Color::Yellow),
            ProjectStatus::Completed => Style::default().fg(Color::Green),
        }
    }

    /// Get priority style
    fn get_priority_style(&self, priority: TaskPriority) -> Style {
        match priority {
            TaskPriority::Low => Style::default().fg(Color::DarkGray),
            TaskPriority::Medium => Style::default().fg(Color::Blue),
            TaskPriority::High => Style::default().fg(Color::Yellow),
            TaskPriority::Critical => Style::default().fg(Color::Red),
        }
    }

    /// Get risk level style
    fn get_risk_style(&self, level: RiskLevel) -> Style {
        match level {
            RiskLevel::Low => Style::default().fg(Color::Green),
            RiskLevel::Medium => Style::default().fg(Color::Yellow),
            RiskLevel::High => Style::default().fg(Color::Orange),
            RiskLevel::Critical => Style::default().fg(Color::Red),
        }
    }

    /// Draw projects view
    fn draw_projects_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    format!("项目列表 ({} 个)", self.projects.len()),
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::default(),
        ];

        if self.projects.is_empty() {
            lines.push(Line::from("没有项目，按 n 创建新项目"));
        } else {
            for (i, project) in self.projects.iter().enumerate() {
                let is_selected = i == self.selected_index;
                let is_current = self.current_project_index == Some(i);

                let style = if is_selected {
                    Style::default().bg(self.theme.primary()).fg(Color::Black)
                } else if is_current {
                    Style::default()
                        .fg(self.theme.primary())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Self::get_status_style(project.status)
                };

                let progress = self.calculate_progress(project);

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("[{:.0}%] ", progress),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(&project.name, style),
                ]));

                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&project.description, Style::default().fg(self.theme.muted())),
                ]));

                lines.push(Line::default());
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("项目")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw milestones view
    fn draw_milestones_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![];

        if let Some(project_idx) = self.current_project_index {
            if let Some(project) = self.projects.get(project_idx) {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} - 里程碑", project.name),
                        Style::default()
                            .fg(self.theme.primary())
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::default());

                if project.milestones.is_empty() {
                    lines.push(Line::from("没有里程碑，按 m 添加新里程碑"));
                } else {
                    for (i, milestone) in project.milestones.iter().enumerate() {
                        let is_selected = i == self.selected_index;
                        let style = if is_selected {
                            Style::default().bg(self.theme.primary()).fg(Color::Black)
                        } else {
                            Style::default()
                        };

                        lines.push(Line::from(vec![
                            Span::styled("● ", Style::default().fg(self.theme.primary())),
                            Span::styled(&milestone.title, style),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("  截止日期: ", Style::default().fg(self.theme.muted())),
                            Span::styled(&milestone.due_date, Style::default()),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("  任务: {}/{}",
                                    milestone.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count(),
                                    milestone.tasks.len()),
                                Style::default().fg(self.theme.muted()),
                            ),
                        ]));
                        lines.push(Line::default());
                    }
                }
            }
        } else {
            lines.push(Line::from("未选择项目，请先选择一个项目"));
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("里程碑")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw risks view
    fn draw_risks_view(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![];

        if let Some(project_idx) = self.current_project_index {
            if let Some(project) = self.projects.get(project_idx) {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} - 风险", project.name),
                        Style::default()
                            .fg(self.theme.primary())
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::default());

                if project.risks.is_empty() {
                    lines.push(Line::from("没有风险，按 r 添加新风险"));
                } else {
                    for (i, risk) in project.risks.iter().enumerate() {
                        let is_selected = i == self.selected_index;
                        let level_style = self.get_risk_style(risk.level);
                        let bg_style = if is_selected {
                            Style::default().bg(self.theme.primary()).fg(Color::Black)
                        } else {
                            Style::default()
                        };

                        lines.push(Line::from(vec![
                            Span::styled("⚠ ", Style::default().fg(Color::Yellow)),
                            Span::styled(&risk.title, level_style),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("  影响: ", Style::default().fg(self.theme.muted())),
                            Span::styled(&risk.impact, bg_style),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("  缓解措施: ", Style::default().fg(self.theme.muted())),
                            Span::styled(&risk.mitigation, bg_style),
                        ]));
                        lines.push(Line::default());
                    }
                }
            }
        } else {
            lines.push(Line::from("未选择项目，请先选择一个项目"));
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("风险")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    /// Draw help bar
    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let help = "1:项目 2:里程碑 3:风险 j/k:导航 Enter:选择 n:新建 a:添加 q:退出";
        let paragraph = Paragraph::new(help)
            .style(Style::default().fg(self.theme.muted()));
        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('1') => {
                self.switch_view(ViewMode::Projects);
                Action::None
            }
            KeyCode::Char('2') => {
                self.switch_view(ViewMode::Milestones);
                Action::None
            }
            KeyCode::Char('3') => {
                self.switch_view(ViewMode::Risks);
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
            KeyCode::Char('n') | KeyCode::Char('N') => {
                // Create new project (placeholder)
                if self.view_mode == ViewMode::Projects {
                    let _ = self.create_project("新项目".to_string(), "项目描述".to_string());
                }
                Action::None
            }
            KeyCode::Enter => {
                // Select project
                if self.view_mode == ViewMode::Projects && !self.projects.is_empty() {
                    self.current_project_index = Some(self.selected_index);
                }
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            _ => Action::None,
        }
    }
}

impl CoreModule for ProjectModule {
    fn name(&self) -> &str {
        "project"
    }

    fn title(&self) -> &str {
        "项目"
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
            ViewMode::Projects => self.draw_projects_view(frame, layout[0]),
            ViewMode::Milestones => self.draw_milestones_view(frame, layout[0]),
            ViewMode::Risks => self.draw_risks_view(frame, layout[0]),
        }

        // Draw help bar
        self.draw_help_bar(frame, layout[1]);
    }

    fn save(&self) -> anyhow::Result<()> {
        self.save_projects()
    }

    fn load(&mut self) -> anyhow::Result<()> {
        self.load_projects()?;
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "1",
                description: "项目",
            },
            Shortcut {
                key: "2",
                description: "里程碑",
            },
            Shortcut {
                key: "3",
                description: "风险",
            },
            Shortcut {
                key: "j/k",
                description: "导航",
            },
            Shortcut {
                key: "Enter",
                description: "选择",
            },
            Shortcut {
                key: "n",
                description: "新建",
            },
        ]
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.load_projects()?;
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.save_projects()?;
        Ok(())
    }
}

pub use ProjectModule as Project;
