use crate::event::AppEvent;
use crate::models::{AppData, CommandHistory, Note, Todo, TodoStatus};
use crate::terminal_manager::PtySession;
use std::path::PathBuf;

// ============ 终端标签页 ============
#[derive(Debug)]
pub struct TerminalTab {
    pub id: usize,
    pub title: String,
    pub shell: String,
    pub pty_session: Option<PtySession>,
    pub command_output_buffer: String,
    pub is_active: bool,
}

impl TerminalTab {
    pub fn start_pty(&mut self) -> Result<(), String> {
        let mut pty_session = PtySession::new(self.shell.clone());
        pty_session.start()?;
        self.pty_session = Some(pty_session);
        Ok(())
    }

    pub fn stop_pty(&mut self) {
        if let Some(ref mut pty) = self.pty_session {
            pty.stop();
        }
        self.pty_session = None;
    }
}

// ============ 标签页枚举 ============
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Notes,
    Todos,
    Commands,
    Calendar,
    FileBrowser,
    Search,
    Settings,
}

impl Tab {
    pub fn all() -> [Tab; 8] {
        [
            Tab::Dashboard,
            Tab::Notes,
            Tab::Todos,
            Tab::Commands,
            Tab::Calendar,
            Tab::FileBrowser,
            Tab::Search,
            Tab::Settings,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Dashboard => "仪表板",
            Tab::Notes => "笔记",
            Tab::Todos => "待办",
            Tab::Commands => "终端",
            Tab::Calendar => "日历",
            Tab::FileBrowser => "文件",
            Tab::Search => "搜索",
            Tab::Settings => "设置",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Tab::Dashboard => "1",
            Tab::Notes => "2",
            Tab::Todos => "3",
            Tab::Commands => "4",
            Tab::Calendar => "5",
            Tab::FileBrowser => "6",
            Tab::Search => "7",
            Tab::Settings => "8",
        }
    }
}

// ============ 输入模式 ============
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

// ============ 文件浏览器状态 ============
#[derive(Debug, Clone)]
pub struct FileBrowserState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub file_size: u64,
}

// ============ 搜索状态 ============
#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected_index: usize,
}

#[derive(Debug, Clone)]
pub enum SearchResult {
    Note { title: String, content: String },
    Todo { title: String, description: String },
}

// ============ 应用主状态 ============
#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub in_terminal_mode: bool,
    pub current_tab: Tab,
    pub input_mode: InputMode,
    pub input_buffer: String,

    // 数据
    pub data: AppData,

    // 笔记状态
    pub selected_note_index: usize,
    pub note_filter: Option<String>,

    // 待办事项状态
    pub selected_todo_index: usize,
    pub todo_filter_status: Option<TodoStatus>,

    // 命令状态
    pub command_input: String,
    pub command_history_index: Option<usize>,

    // 日历状态
    pub selected_date: String,
    pub selected_calendar_event_index: usize,

    // 文件浏览器状态
    pub file_browser: FileBrowserState,

    // 搜索状态
    pub search: SearchState,

    // 模态对话框
    pub show_modal: bool,
    pub modal_message: String,
    pub modal_waiting_input: bool,
    pub input_prompt: String,

    // 终端标签页管理
    pub terminal_tabs: Vec<TerminalTab>,
    pub current_terminal_tab_index: Option<usize>,
    pub mru_terminal_tabs: Vec<usize>,
}

impl Default for App {
    fn default() -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let home_path = PathBuf::from(home_dir);

        Self {
            should_quit: false,
            in_terminal_mode: false,
            current_tab: Tab::Dashboard,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),

            data: AppData::new(),

            selected_note_index: 0,
            note_filter: None,

            selected_todo_index: 0,
            todo_filter_status: None,

            command_input: String::new(),
            command_history_index: None,

            selected_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            selected_calendar_event_index: 0,

            file_browser: FileBrowserState {
                current_path: home_path.clone(),
                entries: Vec::new(),
                selected_index: 0,
            },

            search: SearchState {
                query: String::new(),
                results: Vec::new(),
                selected_index: 0,
            },

            show_modal: false,
            modal_message: String::new(),
            modal_waiting_input: false,
            input_prompt: String::new(),

            terminal_tabs: Vec::new(),
            current_terminal_tab_index: None,
            mru_terminal_tabs: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_data(mut self, data: AppData) -> Self {
        self.data = data;
        self.refresh_file_browser();
        self
    }

    // ============ 终端标签页操作 ============
    pub fn create_terminal_tab(&mut self) {
        let new_id = if let Some(last_tab) = self.terminal_tabs.last() {
            last_tab.id + 1
        } else {
            0
        };

        let shell = if cfg!(target_os = "windows") {
            "powershell.exe".to_string()
        } else {
            "bash".to_string()
        };

        let tab = TerminalTab {
            id: new_id,
            title: format!("Terminal {}", new_id + 1),
            shell,
            pty_session: None,
            command_output_buffer: String::new(),
            is_active: true,
        };

        self.terminal_tabs.push(tab);
        self.current_terminal_tab_index = Some(self.terminal_tabs.len() - 1);
        self.in_terminal_mode = true;

        // Start PTY session for the new tab
        if let Some(tab) = self.get_current_terminal_tab_mut() {
            if let Err(e) = tab.start_pty() {
                tab.command_output_buffer = format!("Failed to start PTY: {}\n", e);
            }
        }

        // Update MRU list
        if let Some(idx) = self.current_terminal_tab_index {
            self.mru_terminal_tabs.retain(|&i| i != idx);
            self.mru_terminal_tabs.push(idx);
        }
    }

    pub fn close_terminal_tab(&mut self) {
        if let Some(index) = self.current_terminal_tab_index {
            if index < self.terminal_tabs.len() {
                self.terminal_tabs.remove(index);

                if self.terminal_tabs.is_empty() {
                    self.current_terminal_tab_index = None;
                    self.in_terminal_mode = false;
                } else {
                    self.current_terminal_tab_index = Some(index.min(self.terminal_tabs.len() - 1));
                }
            }
        }
    }

    pub fn switch_terminal_tab(&mut self, index: usize) {
        if index < self.terminal_tabs.len() {
            self.current_terminal_tab_index = Some(index);
            self.terminal_tabs
                .iter_mut()
                .for_each(|t| t.is_active = false);
            self.terminal_tabs[index].is_active = true;

            // Update MRU list
            self.mru_terminal_tabs.retain(|&i| i != index);
            self.mru_terminal_tabs.push(index);
        }
    }

    pub fn get_current_terminal_tab(&self) -> Option<&TerminalTab> {
        self.current_terminal_tab_index
            .and_then(|idx| self.terminal_tabs.get(idx))
    }

    pub fn get_current_terminal_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        self.current_terminal_tab_index
            .and_then(move |idx| self.terminal_tabs.get_mut(idx))
    }

    pub fn update_current_terminal_output(&mut self) {
        if let Some(tab) = self.get_current_terminal_tab_mut() {
            if let Some(ref pty) = tab.pty_session {
                tab.command_output_buffer = pty.get_output();
            }
        }
    }

    pub fn send_char_to_terminal(&mut self, c: char) {
        if let Some(tab) = self.get_current_terminal_tab_mut() {
            if let Some(ref mut pty) = tab.pty_session {
                let _ = pty.send_char(c);
            }
        }
    }

    pub fn send_enter_to_terminal(&mut self) {
        if let Some(tab) = self.get_current_terminal_tab_mut() {
            if let Some(ref mut pty) = tab.pty_session {
                let _ = pty.send_enter();
            }
        }
    }

    pub fn send_backspace_to_terminal(&mut self) {
        if let Some(tab) = self.get_current_terminal_tab_mut() {
            if let Some(ref mut pty) = tab.pty_session {
                let _ = pty.send_backspace();
            }
        }
    }

    // ============ 笔记操作 ============
    pub fn add_note(&mut self, title: String, content: String, category: String) {
        let note = Note::new(title, content, category);
        self.data.notes.push(note);
    }

    pub fn delete_note(&mut self, index: usize) -> Result<(), String> {
        if index < self.data.notes.len() {
            self.data.notes.remove(index);
            if self.selected_note_index >= self.data.notes.len() && !self.data.notes.is_empty() {
                self.selected_note_index = self.data.notes.len() - 1;
            }
            Ok(())
        } else {
            Err("索引超出范围".to_string())
        }
    }

    pub fn get_notes(&self) -> Vec<&Note> {
        if let Some(filter) = &self.note_filter {
            self.data
                .notes
                .iter()
                .filter(|n| {
                    n.title.to_lowercase().contains(&filter.to_lowercase())
                        || n.content.to_lowercase().contains(&filter.to_lowercase())
                        || n.category.to_lowercase().contains(&filter.to_lowercase())
                        || n.tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&filter.to_lowercase()))
                })
                .collect()
        } else {
            self.data.notes.iter().collect()
        }
    }

    // ============ 待办事项操作 ============
    pub fn add_todo(&mut self, title: String, description: String, category: String) {
        let todo = Todo::new(title, description, category);
        self.data.todos.push(todo);
    }

    pub fn toggle_todo_status(&mut self, index: usize) -> Result<(), String> {
        if index < self.data.todos.len() {
            let todo = &self.data.todos[index];
            let new_status = match todo.status {
                TodoStatus::Pending => TodoStatus::InProgress,
                TodoStatus::InProgress => TodoStatus::Completed,
                TodoStatus::Completed => TodoStatus::Pending,
                TodoStatus::Cancelled => TodoStatus::Pending,
            };
            self.data.todos[index].set_status(new_status);
            Ok(())
        } else {
            Err("索引超出范围".to_string())
        }
    }

    pub fn delete_todo(&mut self, index: usize) -> Result<(), String> {
        if index < self.data.todos.len() {
            self.data.todos.remove(index);
            if self.selected_todo_index >= self.data.todos.len() && !self.data.todos.is_empty() {
                self.selected_todo_index = self.data.todos.len() - 1;
            }
            Ok(())
        } else {
            Err("索引超出范围".to_string())
        }
    }

    pub fn get_todos(&self) -> Vec<&Todo> {
        if let Some(filter) = &self.todo_filter_status {
            self.data
                .todos
                .iter()
                .filter(|t| t.status == *filter)
                .collect()
        } else {
            self.data.todos.iter().collect()
        }
    }

    // ============ 命令操作 ============
    pub fn execute_command(&mut self, command: String) {
        let success = true;
        let output = Some("命令已执行".to_string());
        let history = CommandHistory::new(command.clone(), success, output);
        self.data.command_history.push(history);

        if self.data.command_history.len() > 100 {
            self.data.command_history.remove(0);
        }

        self.command_input.clear();
    }

    // ============ 文件浏览器操作 ============
    pub fn refresh_file_browser(&mut self) {
        self.file_browser.entries.clear();

        if let Ok(entries) = std::fs::read_dir(&self.file_browser.current_path) {
            let mut files: Vec<_> = entries
                .filter_map(Result::ok)
                .map(|entry| {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    let metadata = entry.metadata().ok();
                    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                    let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                    FileEntry {
                        name,
                        path: path.clone(),
                        is_dir,
                        file_size,
                    }
                })
                .collect();

            files.sort_by(|a, b| {
                if a.is_dir && !b.is_dir {
                    std::cmp::Ordering::Less
                } else if !a.is_dir && b.is_dir {
                    std::cmp::Ordering::Greater
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            });

            self.file_browser.entries = files;
        }

        if self.file_browser.selected_index >= self.file_browser.entries.len()
            && !self.file_browser.entries.is_empty()
        {
            self.file_browser.selected_index = self.file_browser.entries.len() - 1;
        }
    }

    pub fn navigate_file_browser_up(&mut self) {
        if let Some(parent) = self.file_browser.current_path.parent() {
            self.file_browser.current_path = parent.to_path_buf();
            self.refresh_file_browser();
        }
    }

    pub fn navigate_file_browser_into(&mut self) {
        if let Some(entry) = self
            .file_browser
            .entries
            .get(self.file_browser.selected_index)
        {
            if entry.is_dir {
                self.file_browser.current_path = entry.path.clone();
                self.file_browser.selected_index = 0;
                self.refresh_file_browser();
            }
        }
    }

    // ============ 搜索操作 ============
    pub fn perform_search(&mut self) {
        self.search.results.clear();
        let query = self.search.query.to_lowercase();

        if query.is_empty() {
            return;
        }

        for note in &self.data.notes {
            if note.title.to_lowercase().contains(&query)
                || note.content.to_lowercase().contains(&query)
            {
                self.search.results.push(SearchResult::Note {
                    title: note.title.clone(),
                    content: note.content.clone(),
                });
            }
        }

        for todo in &self.data.todos {
            if todo.title.to_lowercase().contains(&query)
                || todo.description.to_lowercase().contains(&query)
            {
                self.search.results.push(SearchResult::Todo {
                    title: todo.title.clone(),
                    description: todo.description.clone(),
                });
            }
        }
    }

    // ============ 模态对话框 ============
    pub fn show_message(&mut self, message: String) {
        self.modal_message = message;
        self.show_modal = true;
        self.modal_waiting_input = false;
    }

    pub fn show_input_prompt(&mut self, prompt: String) {
        self.input_prompt = prompt;
        self.input_buffer.clear();
        self.show_modal = true;
        self.modal_waiting_input = true;
    }

    pub fn close_modal(&mut self) {
        self.show_modal = false;
        self.modal_waiting_input = false;
        self.input_buffer.clear();
    }

    // ============ 事件处理 ============
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::TabNext | AppEvent::TabPrevious => {
                let tabs = Tab::all();
                let current_index = tabs.iter().position(|&t| t == self.current_tab).unwrap();
                let new_index = match event {
                    AppEvent::TabNext => (current_index + 1) % tabs.len(),
                    AppEvent::TabPrevious => (current_index + tabs.len() - 1) % tabs.len(),
                    _ => current_index,
                };
                self.current_tab = tabs[new_index];
                self.input_mode = InputMode::Normal;
                // Update in_terminal_mode based on current tab
                self.in_terminal_mode = self.current_tab == Tab::Commands && !self.terminal_tabs.is_empty();
            }
            AppEvent::NewTab => self.create_terminal_tab(),
            AppEvent::CloseTab => self.close_terminal_tab(),
            AppEvent::Up => self.handle_up(),
            AppEvent::Down => self.handle_down(),
            AppEvent::Left => self.handle_left(),
            AppEvent::Right => self.handle_right(),
            AppEvent::Enter => self.handle_enter(),
            AppEvent::Esc => self.handle_esc(),
            AppEvent::Backspace | AppEvent::Delete => self.handle_backspace(),
            AppEvent::Char(c) => self.handle_char(c),
            AppEvent::Other(_) => {}
        }
    }

    fn handle_up(&mut self) {
        match self.current_tab {
            Tab::Notes => {
                let notes = self.get_notes();
                if !notes.is_empty() && self.selected_note_index > 0 {
                    self.selected_note_index -= 1;
                }
            }
            Tab::Todos => {
                let todos = self.get_todos();
                if !todos.is_empty() && self.selected_todo_index > 0 {
                    self.selected_todo_index -= 1;
                }
            }
            Tab::FileBrowser => {
                if !self.file_browser.entries.is_empty() && self.file_browser.selected_index > 0 {
                    self.file_browser.selected_index -= 1;
                }
            }
            Tab::Search => {
                if !self.search.results.is_empty() && self.search.selected_index > 0 {
                    self.search.selected_index -= 1;
                }
            }
            _ => {}
        }
    }

    fn handle_down(&mut self) {
        match self.current_tab {
            Tab::Notes => {
                let notes = self.get_notes();
                if !notes.is_empty() && self.selected_note_index < notes.len() - 1 {
                    self.selected_note_index += 1;
                }
            }
            Tab::Todos => {
                let todos = self.get_todos();
                if !todos.is_empty() && self.selected_todo_index < todos.len() - 1 {
                    self.selected_todo_index += 1;
                }
            }
            Tab::FileBrowser => {
                if !self.file_browser.entries.is_empty()
                    && self.file_browser.selected_index < self.file_browser.entries.len() - 1
                {
                    self.file_browser.selected_index += 1;
                }
            }
            Tab::Search => {
                if !self.search.results.is_empty()
                    && self.search.selected_index < self.search.results.len() - 1
                {
                    self.search.selected_index += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_left(&mut self) {
        if self.current_tab == Tab::FileBrowser {
            self.navigate_file_browser_up();
        }
    }

    fn handle_right(&mut self) {
        if self.current_tab == Tab::FileBrowser {
            self.navigate_file_browser_into();
        }
    }

    fn handle_enter(&mut self) {
        if self.show_modal {
            if self.modal_waiting_input {
                if !self.input_buffer.is_empty() {
                    let input = self.input_buffer.clone();
                    self.close_modal();

                    match self.current_tab {
                        Tab::Notes => {
                            self.add_note(
                                input.clone(),
                                "新建笔记".to_string(),
                                "默认".to_string(),
                            );
                        }
                        Tab::Todos => {
                            self.add_todo(
                                input.clone(),
                                "新建待办".to_string(),
                                "默认".to_string(),
                            );
                        }
                        _ => {}
                    }
                }
            } else {
                self.close_modal();
            }
        } else {
            match self.current_tab {
                Tab::Commands => {
                    self.send_enter_to_terminal();
                }
                Tab::Todos => {
                    let _ = self.toggle_todo_status(self.selected_todo_index);
                }
                Tab::FileBrowser => {
                    self.navigate_file_browser_into();
                }
                Tab::Search => {
                    if !self.search.query.is_empty() {
                        self.perform_search();
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_esc(&mut self) {
        if self.show_modal {
            self.close_modal();
        } else if self.input_mode == InputMode::Editing {
            self.input_mode = InputMode::Normal;
            self.input_buffer.clear();
        } else {
            self.note_filter = None;
            self.todo_filter_status = None;
        }
    }

    fn handle_backspace(&mut self) {
        if self.current_tab == Tab::Commands && self.in_terminal_mode {
            self.send_backspace_to_terminal();
        } else if self.input_mode == InputMode::Editing {
            self.input_buffer.pop();
        } else if self.show_modal && self.modal_waiting_input {
            if !self.input_buffer.is_empty() {
                self.input_buffer.pop();
            }
        }
    }

    fn handle_char(&mut self, c: char) {
        if self.show_modal && self.modal_waiting_input {
            if c == '\n' {
                // 处理由 Enter
            } else {
                self.input_buffer.push(c);
            }
        } else if self.current_tab == Tab::Commands && self.in_terminal_mode {
            // Send character to terminal
            self.send_char_to_terminal(c);
        } else if self.input_mode == InputMode::Editing {
            self.input_buffer.push(c);
        } else {
            match c {
                '1'..='8' => {
                    let tabs = Tab::all();
                    let index = c.to_digit(10).unwrap() as usize - 1;
                    if index < tabs.len() {
                        self.current_tab = tabs[index];
                    }
                }
                'a' => match self.current_tab {
                    Tab::Notes => self.show_input_prompt("笔记标题: ".to_string()),
                    Tab::Todos => self.show_input_prompt("待办事项: ".to_string()),
                    _ => {}
                },
                't' => {
                    // Create new terminal tab
                    self.create_terminal_tab();
                }
                'd' => {
                    self.input_mode = InputMode::Editing;
                    match self.current_tab {
                        Tab::Search => self.input_buffer = self.search.query.clone(),
                        _ => {}
                    }
                }
                '/' => {
                    if self.current_tab != Tab::Search {
                        self.note_filter = Some(String::new());
                        self.input_mode = InputMode::Editing;
                        self.input_buffer = String::new();
                    }
                }
                'n' => match self.current_tab {
                    Tab::Todos => self.todo_filter_status = None,
                    _ => {}
                },
                'f' => match self.current_tab {
                    Tab::Todos => self.todo_filter_status = Some(TodoStatus::Completed),
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
