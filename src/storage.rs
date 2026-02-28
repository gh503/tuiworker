use crate::models::AppData;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

// ============ 配置模型 ============
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data_file: PathBuf,
    pub backup_enabled: bool,
    pub max_history: usize,
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::config_local_dir().unwrap_or_else(|| PathBuf::from("."));
        let data_dir = config_dir.join("tuiworker");

        Self {
            data_file: data_dir.join("data.json"),
            backup_enabled: true,
            max_history: 100,
        }
    }
}

// ============ 数据存储 ============
pub struct Storage {
    config: Config,
}

impl Storage {
    pub fn new() -> io::Result<Self> {
        let config = Config::default();

        // 确保数据目录存在
        if let Some(parent) = config.data_file.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self { config })
    }

    pub fn from_config(config: Config) -> Self {
        Self { config }
    }

    /// 加载数据
    pub fn load(&self) -> io::Result<AppData> {
        if !self.config.data_file.exists() {
            // 文件不存在，返回空数据
            return Ok(AppData::new());
        }

        let mut file = fs::File::open(&self.config.data_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data: AppData = serde_json::from_str(&contents).unwrap_or_else(|_| {
            // 解析失败，返回空数据
            eprintln!("警告: 数据文件损坏，将创建新数据文件");
            AppData::new()
        });

        Ok(data)
    }

    /// 保存数据
    pub fn save(&self, data: &AppData) -> io::Result<()> {
        let content = serde_json::to_string_pretty(data)?;

        // 如果启用了备份，先创建备份
        if self.config.backup_enabled && self.config.data_file.exists() {
            self.create_backup()?;
        }

        let mut file = fs::File::create(&self.config.data_file)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// 创建备份
    fn create_backup(&self) -> io::Result<()> {
        let backup_path = self.config.data_file.with_extension("json.bak");
        fs::copy(&self.config.data_file, &backup_path)?;
        Ok(())
    }

    /// 获取备份列表
    pub fn list_backups(&self) -> Vec<PathBuf> {
        let mut backups = Vec::new();

        if let Some(parent) = self.config.data_file.parent() {
            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "bak").unwrap_or(false) {
                        backups.push(path);
                    }
                }
            }
        }

        backups.sort();
        backups.reverse();
        backups
    }

    /// 从备份恢复
    pub fn restore_from_backup(&self, backup_path: &PathBuf) -> io::Result<()> {
        fs::copy(backup_path, &self.config.data_file)?;
        Ok(())
    }

    /// 清理旧备份（保留最近 5 个）
    pub fn cleanup_old_backups(&self) -> io::Result<()> {
        let mut backups = self.list_backups();

        // 保留最近的 5 个备份
        while backups.len() > 5 {
            if let Some(old_backup) = backups.pop() {
                fs::remove_file(old_backup)?;
            }
        }

        Ok(())
    }
}


