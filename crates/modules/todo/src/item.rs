#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TodoItem {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: TodoStatus,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

impl Priority {
    pub fn display_color(&self) -> ratatui::style::Color {
        match self {
            Priority::High => ratatui::style::Color::Red,
            Priority::Medium => ratatui::style::Color::Yellow,
            Priority::Low => ratatui::style::Color::Green,
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            Priority::High => "!",
            Priority::Medium => "-",
            Priority::Low => ".",
        }
    }
}

impl Default for TodoItem {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: String::new(),
            description: None,
            priority: Priority::Medium,
            status: TodoStatus::Pending,
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            due_date: None,
            completed_at: None,
        }
    }
}

impl TodoStatus {
    pub fn symbol(&self) -> char {
        match self {
            TodoStatus::Pending => ' ',
            TodoStatus::InProgress => '>',
            TodoStatus::Completed => 'X',
        }
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, TodoStatus::Completed)
    }
}
