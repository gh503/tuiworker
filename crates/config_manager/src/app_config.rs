use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::manager::ConfigManager;

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub shortcuts: ShortcutConfig,
    pub modules: ModulesConfig,
    #[serde(skip)] // 动态加载，不序列化
    pub filebrowser: FileBrowserConfig,
    #[serde(skip)]
    pub todo: TodoConfig,
    #[serde(skip)]
    pub note: NoteConfig,
    #[serde(skip)]
    pub diary: DiaryConfig,
    #[serde(skip)]
    pub terminal: TerminalConfig,
    #[serde(skip)]
    pub git: GitConfig,
    #[serde(skip)]
    pub music: MusicConfig,
    #[serde(skip)]
    pub mail: MailConfig,
    #[serde(skip)]
    pub project: ProjectConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            shortcuts: ShortcutConfig::default(),
            modules: ModulesConfig::default(),
            filebrowser: FileBrowserConfig::default(),
            todo: TodoConfig::default(),
            note: NoteConfig::default(),
            diary: DiaryConfig::default(),
            terminal: TerminalConfig::default(),
            git: GitConfig::default(),
            music: MusicConfig::default(),
            mail: MailConfig::default(),
            project: ProjectConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// 日志级别: trace, debug, info, warn, error
    pub log_level: String,
    /// 是否记录日志到文件
    pub log_to_file: bool,
    /// 日志文件路径
    pub log_file: PathBuf,
    /// 主题名称: default, dark, light
    pub theme: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_to_file: true,
            log_file: ConfigManager::default_log_file(),
            theme: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// 全局快捷键：退出
    pub global_quit: String,
    /// 切换到下一个模块
    pub switch_tab_next: String,
    /// 切换到上一个模块
    pub switch_tab_prev: String,
    /// 显示帮助
    pub show_help: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            global_quit: "q".to_string(),
            switch_tab_next: "Ctrl+Right".to_string(),
            switch_tab_prev: "Ctrl+Left".to_string(),
            show_help: "?".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    /// 启用的模块列表
    pub enabled: Vec<String>,
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                "filebrowser".to_string(),
                "todo".to_string(),
                "note".to_string(),
                "diary".to_string(),
                "terminal".to_string(),
                "git".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBrowserConfig {
    /// 是否显示隐藏文件
    pub show_hidden: bool,
    /// 排序方式: name, size, modified
    pub sort_by: String,
}

impl Default for FileBrowserConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_by: "name".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoConfig {
    /// 默认优先级
    pub default_priority: String,
}

impl Default for TodoConfig {
    fn default() -> Self {
        Self {
            default_priority: "medium".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteConfig {
    /// 笔记目录
    pub notes_dir: PathBuf,
}

impl Default for NoteConfig {
    fn default() -> Self {
        Self {
            notes_dir: ConfigManager::default_notes_dir(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryConfig {
    /// 日记目录
    pub diary_dir: PathBuf,
}

impl Default for DiaryConfig {
    fn default() -> Self {
        Self {
            diary_dir: ConfigManager::default_diary_dir(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// 默认 shell: default, bash, zsh, powershell, cmd
    pub default_shell: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            default_shell: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// 默认编辑器: default, vim, nano, emacs
    pub default_editor: String,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_editor: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicConfig {
    /// 音乐目录
    pub music_dir: PathBuf,
    /// 默认音量 (0.0 - 1.0)
    pub default_volume: f32,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            music_dir: dirs::audio_dir()
                .unwrap_or_else(|| PathBuf::from("~/Music"))
                .join("TUI_Workstation"),
            default_volume: 0.8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailConfig {
    pub imap_server: String,
    pub imap_port: u16,
    pub use_imaps: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub use_smtps: bool,
    pub username: String,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            imap_server: "imap.example.com".to_string(),
            imap_port: 993,
            use_imaps: true,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            use_smtps: false,
            username: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 项目数据存储在 sled 数据库中
    #[serde(default)]
    _phantom: std::marker::PhantomData<()>,
}

impl Default for ProjectConfig {
fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
}

}
