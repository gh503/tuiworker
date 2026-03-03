//! Project module - Project management with milestones, risks, and progress tracking

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use std::path::PathBuf;

use uuid::Uuid;

use ui::Theme;

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
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

/// Project data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectData {
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
    projects: Vec<ProjectData>,
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

    pub fn load_projects(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn save_projects(&self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn create_project(&mut self, name: String, description: String) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn switch_view(&mut self, view: ViewMode) {
        self.view_mode = view;
        self.selected_index = 0;
    }

    pub fn navigate_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn navigate_down(&mut self) {
        self.selected_index = (self.selected_index + 1).min(self.projects.len().saturating_sub(1));
    }

    fn calculate_progress(&self, project: &ProjectData) -> f32 {
        0.0
    }

    fn get_status_style(_status: ProjectStatus) -> Style {
        Style::default()
    }

    fn get_risk_style(&self, _level: RiskLevel) -> Style {
        Style::default()
    }

    fn draw_projects_view(&self, frame: &mut Frame, area: Rect) {
        let text = Text::from(vec![Line::from("项目管理模块 - Coming Soon!")]);
        let paragraph = Paragraph::new(text)
            .block(Block::default().title("项目").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn draw_milestones_view(&self, frame: &mut Frame, area: Rect) {
        self.draw_projects_view(frame, area);
    }

    fn draw_risks_view(&self, frame: &mut Frame, area: Rect) {
        self.draw_projects_view(frame, area);
    }

    fn draw_help_bar(&self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new("1:项目 2:里程碑 3:风险 q:退出")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }

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
        match self.view_mode {
            ViewMode::Projects => self.draw_projects_view(frame, layout[0]),
            ViewMode::Milestones => self.draw_milestones_view(frame, layout[0]),
            ViewMode::Risks => self.draw_risks_view(frame, layout[0]),
        }
        self.draw_help_bar(frame, layout[1]);
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
                key: "1",
                description: "项目",
            },
            Shortcut {
                key: "q",
                description: "退出",
            },
        ]
    }
    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub use ProjectModule as Project;
