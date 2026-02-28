use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

// ============ 笔记模型 ============
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Note {
    pub fn new(title: String, content: String, category: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Local::now();
        Self {
            id,
            title,
            content,
            category,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, title: String, content: String, category: String) {
        self.title = title;
        self.content = content;
        self.category = category;
        self.updated_at = Local::now();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
}

// ============ 待办事项模型 ============
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TodoPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: TodoPriority,
    pub status: TodoStatus,
    pub category: String,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
}

impl Todo {
    pub fn new(title: String, description: String, category: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Local::now();
        Self {
            id,
            title,
            description,
            priority: TodoPriority::Medium,
            status: TodoStatus::Pending,
            category,
            due_date: None,
            created_at: now,
            completed_at: None,
        }
    }

    pub fn complete(&mut self) {
        self.status = TodoStatus::Completed;
        self.completed_at = Some(Local::now());
    }

    pub fn set_status(&mut self, status: TodoStatus) {
        self.status = status;
        if status == TodoStatus::Completed {
            self.completed_at = Some(Local::now());
        } else {
            self.completed_at = None;
        }
    }

    pub fn set_priority(&mut self, priority: TodoPriority) {
        self.priority = priority;
    }
}

// ============ 命令执行历史 ============
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandHistory {
    pub id: String,
    pub command: String,
    pub executed_at: DateTime<Local>,
    pub success: bool,
    pub output: Option<String>,
}

impl CommandHistory {
    pub fn new(command: String, success: bool, output: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            command,
            executed_at: Local::now(),
            success,
            output,
        }
    }
}

// ============ 日历事项 ============
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CalendarEventType {
    Meeting,
    Reminder,
    Deadline,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub event_type: CalendarEventType,
    pub date: NaiveDate,
    pub time: Option<String>,
    pub completed: bool,
}

impl CalendarEvent {
    pub fn new(
        title: String,
        description: String,
        event_type: CalendarEventType,
        date: NaiveDate,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            description,
            event_type,
            date,
            time: None,
            completed: false,
        }
    }

    pub fn toggle_complete(&mut self) {
        self.completed = !self.completed;
    }
}

// ============ 应用数据 ============
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub notes: Vec<Note>,
    pub todos: Vec<Todo>,
    pub command_history: Vec<CommandHistory>,
    pub calendar_events: Vec<CalendarEvent>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            notes: Vec::new(),
            todos: Vec::new(),
            command_history: Vec::new(),
            calendar_events: Vec::new(),
        }
    }
}

impl AppData {
    pub fn new() -> Self {
        Self::default()
    }
}
