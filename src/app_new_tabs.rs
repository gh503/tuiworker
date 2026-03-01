// ============ 终端标签页 ============
#[derive(Debug)]
pub struct TerminalTab {
    pub id: usize,
    pub title: String,
    pub command_execution_active: bool,
    pub command_output_buffer: String,
    pub is_active: bool,
}

// ============ 标签页枚举 ============
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Notes,
    Todos,
    Calendar,
    FileBrowser,
    Search,
    Settings,
}

impl Tab {
    pub fn all() -> [Tab; 7] {
        [
            Tab::Dashboard,
            Tab::Notes,
            Tab::Todos,
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
            Tab::Calendar => "5",
            Tab::FileBrowser => "6",
            Tab::Search => "7",
            Tab::Settings => "8",
        }
    }
}
