# TUI Workstation API 设计文档

## 核心接口定义

### 1. Module Trait

所有功能模块必须实现的核心接口：

```rust
use ratatui::{Frame, layout::Rect};
use crate::core::event::Event;
use crate::core::action::Action;

pub trait Module {
    /// 模块名称（唯一标识符）
    fn name(&self) -> &str;

    /// 显示标题（用于标签栏）
    fn title(&self) -> &str;

    /// 处理输入事件
    /// 返回 Action 指定应用层面的操作（切换模块、退出等）
    fn update(&mut self, event: Event) -> Action;

    /// 绘制 UI
    fn draw(&mut self, frame: &mut Frame, area: Rect);

    /// 保存状态到持久化存储
    fn save(&self) -> Result<(), StorageError>;

    /// 从持久化存储加载状态
    fn load(&mut self) -> Result<(), StorageError>;

    /// 模块快捷键列表（用于帮助显示）
    fn shortcuts(&self) -> Vec<Shortcut>;

    /// 模块初始化（首次激活时调用）
    fn init(&mut self) -> Result<(), InitError> {
        Ok(())
    }

    /// 模块清理（切换到其他模块时调用）
    fn cleanup(&mut self) -> Result<(), CleanupError> {
        Ok(())
    }

    /// 是否支持异步任务
    fn has_async_tasks(&self) -> bool {
        false
    }

    /// 获取异步任务（如果有）
    fn get_async_task(&mut self) -> Option<BoxFuture<'static, ()>> {
        None
    }
}

/// 快捷键定义
pub struct Shortcut {
    pub key: &'static str,        // "q", "Ctrl+C", "Enter"
    pub description: &'static str,
}
```

### 2. 事件系统

```rust
use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};

/// 应用级事件类型
pub enum Event {
    /// 键盘事件
    Key(KeyEvent),

    /// 鼠标事件
    Mouse(MouseEvent),

    /// 窗口大小变化
    Resize(u16, u16),

    /// 定时器事件（用于刷新/动画）
    Timer,

    /// 自定义事件（模块间通信）
    Custom(Box<dyn CustomEvent>),

    /// 异步任务完成
    TaskComplete(TaskId),
}

/// 自定义事件 trait
pub trait CustomEvent: Send + Sync {
    fn source_module(&self) -> &str;
    fn target_module(&self) -> Option<&str>;
}

/// 应用操作（由模块 update 返回）
pub enum Action {
    /// 无操作
    None,

    /// 处理了事件
    Consumed,

    /// 切换到指定模块
    SwitchModule(String),

    /// 退出应用
    Quit,

    /// 显示消息（状态栏提示）
    ShowMessage(Message),

    /// 执行命令
    ExecuteCommand(Command),
}

pub enum Message {
    Info(String),
    Warning(String),
    Error(String),
}

pub struct Command {
    pub command: String,
    pub args: Vec<String>,
}
```

### 3. 数据存储 API

```rust
use sled::{Db, Batch};

/// 数据库封装
pub struct Database {
    db: Db,
    namespace_prefix: Vec<u8>,
}

impl Database {
    /// 初始化数据库
    pub fn open(path: &Path) -> Result<Self, DatabaseError>;

    /// 创建命名空间（模块专用）
    pub fn with_namespace(&self, namespace: &str) -> NamespacedDatabase;

    /// 基本键值操作
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError>;
    pub fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError>;
    pub fn remove(&self, key: &[u8]) -> Result<(), DatabaseError>;

    /// 批量操作
    pub fn batch(&self) -> Batch;

    /// 遍历键值对
    pub fn iter(&self) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)>;

    /// 集合操作（用于列表/标签）
    pub fn add_to_set(&self, set_key: &[u8], value: &[u8]) -> Result<()>;
    pub fn remove_from_set(&self, set_key: &[u8], value: &[u8]) -> Result<()>;
    pub fn get_set(&self, set_key: &[u8]) -> Result<HashSet<Vec<u8>>>;

    /// 事务支持
    pub fn transaction<F, R>(&self, f: F) -> Result<R, TransactionError>
    where
        F: Fn(&Db) -> Result<R, sled::Error>;
}

/// 带命名空间的数据库（模块使用）
pub struct NamespacedDatabase {
    db: Db,
    prefix: Vec<u8>,
}
```

### 4. 配置管理 API

```rust
use serde::Deserialize;
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub shortcuts: ShortcutConfig,
    pub modules: ModulesConfig,
    #[serde(skip)]
    pub filebrowser: FileBrowserConfig,
    #[serde(skip)]
    pub todo: TodoConfig,
    // ... 其他模块配置
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    pub log_level: String,
    pub log_to_file: bool,
    pub log_file: PathBuf,
    pub theme: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShortcutConfig {
    pub global_quit: String,
    pub switch_tab_next: String,
    pub switch_tab_prev: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModulesConfig {
    pub enabled: Vec<String>,
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// 加载配置文件
    pub fn load() -> Result<Self, ConfigError>;

    /// 保存配置文件
    pub fn save(&self) -> Result<(), ConfigError>;

    /// 获取配置引用
    pub fn get(&self) -> &AppConfig;

    /// 热重载配置
    pub fn reload(&mut self) -> Result<(), ConfigError>;

    /// 获取 XDG 目录
    pub fn get_data_dir() -> PathBuf;
    pub fn get_config_dir() -> PathBuf;
}
```

### 5. 日志 API

```rust
use log::{Level, LevelFilter};

/// 日志初始化
pub fn init_logging(config: &GeneralConfig) -> Result<(), LogError> {
    // 配置控制台日志
    // 配置文件日志
    // 设置日志级别
}

/// 日志 facade（由内部模块使用）
// 使用标准 log crate 宏：
// log::trace!("...");
// log::debug!("...");
// log::info!("...");
// log::warn!("...");
// log::error!("...);

/// 自定义 Logger 实现
pub struct AppLogger {
    console: bool,
    file: bool,
    file_path: PathBuf,
    level: LevelFilter,
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool;
    fn log(&self, record: &log::Record);
    fn flush(&self);
}
```

### 6. 应用程序 API

```rust
pub struct App {
    modules: Vec<Box<dyn Module>>,
    active_module_index: usize,
    event_sender: mpsc::Sender<Event>,
    event_receiver: mpsc::Receiver<Event>,
    should_quit: bool,
    config: AppConfig,
}

impl App {
    /// 创建新应用
    pub fn new() -> Result<Self, InitError>;

    /// 添加模块
    pub fn register_module<M: Module + 'static>(&mut self, module: M) {
        self.modules.push(Box::new(module));
    }

    /// 运行主循环
    pub fn run(&mut self) -> Result<(), RunError>;

    /// 处理事件（内部）
    fn handle_event(&mut self, event: Event) -> Result<(), RunError>;

    /// 渲染帧
    fn draw(&mut self) -> Result<(), RunError>;

    /// 模块间通信
    pub fn send_event(&self, event: Event) -> Result<(), SendError>;
}
```

## 模块特定 API

### 7. FileBrowser 模块

```rust
pub struct FileBrowser {
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    selected_index: usize,
    show_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: SystemTime,
}

impl FileBrowser {
    pub fn new(start_path: PathBuf) -> Self;

    /// 刷新目录
    pub fn refresh(&mut self) -> Result<(), IOError>;

    /// 进入目录
    pub fn enter(&mut self) -> Result<(), IOError>;

    /// 返回上层目录
    pub fn go_up(&mut self);

    /// 预览文件
    pub fn preview(&self) -> Option<String>;

    /// 打开文件（使用系统默认应用）
    pub fn open_file(&self) -> Result<(), OpenError>;

    /// 导航到指定路径
    pub fn navigate_to(&mut self, path: PathBuf) -> Result<(), IOError>;
}
```

### 8. Todo 模块

```rust
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: TodoStatus,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

pub struct TodoModule {
    items: Vec<TodoItem>,
    selected_index: usize,
    db: NamespacedDatabase,
}

impl TodoModule {
    pub fn new(db: NamespacedDatabase) -> Self;

    /// 添加待办
    pub fn add(&mut self, item: TodoItem) -> Result<(), StorageError>;

    /// 更新待办
    pub fn update(&mut self, id: Uuid, item: TodoItem) -> Result<(), StorageError>;

    /// 删除待办
    pub fn delete(&mut self, id: Uuid) -> Result<(), StorageError>;

    /// 切换完成状态
    pub fn toggle_complete(&mut self, id: Uuid) -> Result<(), StorageError>;

    /// 按标签筛选
    pub fn filter_by_tag(&mut self, tag: &str);

    /// 按优先级排序
    pub fn sort_by_priority(&mut self);
}
```

### 9. Note 模块

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NoteModule {
    notes: Vec<Note>,
    selected_index: usize,
    editing_index: Option<usize>,
    db: NamespacedDatabase,
    notes_dir: PathBuf,
}

impl NoteModule {
    pub fn new(db: NamespacedDatabase, notes_dir: PathBuf) -> Self;

    /// 创建新笔记
    pub fn create(&mut self, title: String) -> Result<(), NoteError>;

    /// 编辑笔记
    pub fn edit(&mut self, id: Uuid);

    /// 保存笔记
    pub fn save_note(&mut self) -> Result<(), NoteError>;

    /// 删除笔记
    pub fn delete(&mut self, id: Uuid) -> Result<(), NoteError>;

    /// 搜索笔记
    pub fn search(&self, query: &str) -> Vec<Note>;

    /// 按标签筛选
    pub fn filter_by_tag(&self, tag: &str) -> Vec<Note>;
}
```

### 10. Diary 模块

```rust
use chrono::{Date, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryEntry {
    pub date: Date<Utc>,
    pub content: String,
}

pub struct DiaryModule {
    entries: HashMap<Date<Utc>, DiaryEntry>,
    selected_date: Date<Utc>,
    editing: bool,
    diary_dir: PathBuf,
}

impl DiaryModule {
    pub fn new(diary_dir: PathBuf) -> Self;

    /// 切换到指定日期
    pub fn select_date(&mut self, date: Date<Utc>);

    /// 写入日记
    pub fn write_entry(&mut self, content: String) -> Result<(), DiaryError>;

    /// 加载日记
    pub fn load_entries(&mut self) -> Result<(), DiaryError>;

    /// 获取农历信息
    pub fn get_lunar_info(&self, date: Date<Utc>) -> Option<LunarDate>;

    /// 判断是否为节假日
    pub fn is_holiday(&self, date: Date<Utc>) -> bool;
}

#[derive(Debug)]
pub struct LunarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub zodiac: String,
}
```

### 11. Terminal 模块

```rust
pub struct TerminalModule {
    tabs: Vec<TerminalTab>,
    active_tab_index: usize,
}

#[derive(Clone)]
pub struct TerminalTab {
    pub id: Uuid,
    pub title: String,
    pub shell: String,
}

pub struct TerminalInstance {
    pty: Box<dyn portable_pty::MasterPty + Send>,
    reader: Box<dyn portable_pty::Child std::os::raw::c_int>>,
    writer: Box<dyn portable_pty::Child std::os::raw::c_int>>,
}

impl TerminalModule {
    pub fn new() -> Self;

    /// 创建新标签
    pub fn new_tab(&mut self, shell: Option<String>) -> Result<(), TerminalError>;

    /// 切换标签
    pub fn switch_tab(&mut self, index: usize);

    /// 关闭标签
    pub fn close_tab(&mut self, index: usize);

    /// 发送输入到终端
    pub fn send_input(&mut self, input: &str);

    /// 读取终端输出
    pub async fn read_output(&mut self) -> Vec<u8>;
}
```

### 12. Git 模块

```rust
use git2::{Repository, Status};

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
    pub untracked: Vec<String>,
}

pub struct GitModule {
    repo: Option<Repository>,
    current_dir: PathBuf,
}

impl GitModule {
    pub fn new(start_dir: PathBuf) -> Self;

    /// 打开 Git 仓库
    pub fn open_repo(&mut self) -> Result<(), GitError>;

    /// 获取状态
    pub fn get_status(&self) -> Result<GitStatus, GitError>;

    /// 提交更改
    pub fn commit(&self, message: &str) -> Result<Oid, GitError>;

    /// 获取日志
    pub fn get_log(&self, limit: usize) -> Result<Vec<Commit>, GitError>;

    /// 切换分支
    pub fn checkout(&self, branch: &str) -> Result<(), GitError>;

    /// 拉取
    pub async fn pull(&self) -> Result<(), GitError>;

    /// 推送
    pub async fn push(&self) -> Result<(), GitError>;
}
```

### 13. Music 模块

```rust
use symphonia::core::audio::AudioBuffer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Duration,
}

pub struct MusicModule {
    playlist: Vec<Track>,
    current_index: usize,
    is_playing: bool,
    volume: f32,
    player: Option<Player>,
}

impl MusicModule {
    pub fn new() -> Self;

    /// 加载播放列表
    pub fn load_playlist(&mut self, dir: PathBuf) -> Result<(), MusicError>;

    /// 播放
    pub fn play(&mut self) -> Result<(), MusicError>;

    /// 暂停
    pub fn pause(&mut self);

    /// 下一首
    pub fn next(&mut self);

    /// 上一首
    pub fn previous(&mut self);

    /// 设置音量
    pub fn set_volume(&mut self, volume: f32);

    /// 跳转到指定时间
    pub fn seek(&mut self, position: Duration);
}
```

### 14. Project 模块

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub progress: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Planning,
    Active,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub due_date: DateTime<Utc>,
    pub status: MilestoneStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MilestoneStatus {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: Uuid,
    pub description: String,
    pub probability: RiskLevel,
    pub impact: RiskLevel,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ProjectModule {
    projects: Vec<Project>,
    selected_index: usize,
    db: NamespacedDatabase,
}

impl ProjectModule {
    pub fn new(db: NamespacedDatabase) -> Self;

    /// 创建项目
    pub fn create_project(&mut self, project: Project) -> Result<(), StorageError>;

    /// 更新项目
    pub fn update_project(&mut self, id: Uuid, project: Project) -> Result<(), StorageError>;

    /// 添加里程碑
    pub fn add_milestone(&mut self, project_id: Uuid, milestone: Milestone) -> Result<(), StorageError>;

    /// 添加风险
    pub fn add_risk(&mut self, project_id: Uuid, risk: Risk) -> Result<(), StorageError>;

    /// 更新进度
    pub fn update_progress(&mut self, id: Uuid, progress: f32) -> Result<(), StorageError>;
}
```

### 15. Mail 模块

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailHeader {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub date: DateTime<Utc>,
    pub seen: bool,
    pub answered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailBody {
    pub uid: u32,
    pub plain_text: String,
    pub html: Option<String>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub size: u64,
    pub content_type: String,
}

pub struct MailModule {
    config: MailConfig,
    folders: Vec<Folder>,
    current_folder: Option<String>,
    headers: Vec<MailHeader>,
    selected_index: usize,
}

pub struct MailConfig {
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String, // 存储在 keyring
}

impl MailModule {
    pub fn new(config: MailConfig) -> Self;

    /// 连接到服务器
    pub async fn connect(&mut self) -> Result<(), MailError>;

    /// 获取文件夹列表
    pub async fn fetch_folders(&mut self) -> Result<(), MailError>;

    /// 获取邮件头
    pub async fn fetch_headers(&mut self, folder: &str) -> Result<(), MailError>;

    /// 获取邮件正文
    pub async fn fetch_body(&mut self, uid: u32) -> Result<MailBody, MailError>;

    /// 发送邮件
    pub async fn send_mail(&self, to: &str, subject: &str, body: &str) -> Result<(), MailError>;

    /// 标记已读
    pub async fn mark_seen(&mut self, uid: u32) -> Result<(), MailError>;

    /// 删除邮件
    pub async fn delete_mail(&mut self, uid: u32) -> Result<(), MailError>;
}
```

## 错误处理

```rust
/// 应用错误类型
#[derive(Debug)]
pub enum AppError {
    Storage(StorageError),
    Terminal(TerminalError),
    IO(std::io::Error),
    Config(ConfigError),
    Module(String),
}

/// 存储错误
#[derive(Debug)]
pub enum StorageError {
    Database(sled::Error),
    Serialization(serde_json::Error),
    NotFound,
    InvalidData,
}

/// 终端错误
#[derive(Debug)]
pub enum TerminalError {
    PTY(String),
    IO(std::io::Error),
}

/// 配置错误
#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    Parse(config::ConfigError),
    Serialization(serde_json::Error),
}

// 所有模块实现 From trait 以便错误转换
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IO(err)
    }
}

// ... 其他转换实现
```

## 测试 API

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_trait() {
        struct DummyModule;
        impl Module for DummyModule {
            fn name(&self) -> &str { "dummy" }
            fn title(&self) -> &str { "Dummy" }
            fn update(&mut self, _event: Event) -> Action { Action::None }
            fn draw(&mut self, _frame: &mut Frame, _area: Rect) {}
            fn save(&self) -> Result<(), StorageError> { Ok(()) }
            fn load(&mut self) -> Result<(), StorageError> { Ok(()) }
            fn shortcuts(&self) -> Vec<Shortcut> { vec![] }
        }

        let module = DummyModule;
        assert_eq!(module.name(), "dummy");
    }

    #[test]
    fn test_database_operations() {
        // 测试数据库 CRUD 操作
    }

    #[test]
    fn test_config_loading() {
        // 测试配置加载
    }
}
```

---

**文档版本：** 1.0
**更新日期：** 2025-03-02
