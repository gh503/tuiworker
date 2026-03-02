use std::path::{Path, PathBuf};

use crate::app_config::AppConfig;
use crate::error::ConfigResult;

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    /// 加载配置文件
    ///
    /// 按以下优先级查找配置：
    /// 1. 指定的路径
    /// 2. 环境变量 TUI_WORKSTATION_CONFIG
    /// 3. XDG 配置目录 ~/.config/tui-workstation/config.toml
    pub fn load(config_path: Option<PathBuf>) -> ConfigResult<Self> {
        let config_path = Self::resolve_config_path(config_path)?;

        if !config_path.exists() {
            log::warn!("Config file not found at {:?}, using defaults", config_path);
            return Ok(Self {
                config_path,
                config: AppConfig::default(),
            });
        }

        let settings = config::Config::builder()
            .add_source(config::File::from(config_path.as_path()))
            .build()?;

        let config: AppConfig = settings.try_deserialize()?;

        log::info!("Loaded config from {:?}", config_path);

        Ok(Self {
            config_path,
            config,
        })
    }

    /// 保存配置文件
    pub fn save(&self) -> ConfigResult<()> {
        // 创建父目录
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let toml = toml::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, toml)?;

        log::info!("Saved config to {:?}", self.config_path);
        Ok(())
    }

    /// 获取配置引用
    pub fn get(&self) -> &AppConfig {
        &self.config
    }

    /// 获取配置可变引用
    pub fn get_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// 热重载配置
    pub fn reload(&mut self) -> ConfigResult<()> {
        let config = Self::load(Some(self.config_path.clone()))?;
        self.config = config.config;
        log::info!("Reloaded config from {:?}", self.config_path);
        Ok(())
    }

    /// 解析配置文件路径
    fn resolve_config_path(config_path: Option<PathBuf>) -> ConfigResult<PathBuf> {
        if let Some(path) = config_path {
            return Ok(path);
        }

        // 检查环境变量
        if let Ok(env_path) = std::env::var("TUI_WORKSTATION_CONFIG") {
            return Ok(PathBuf::from(env_path));
        }

        // 使用 XDG 配置目录
        let config_dir = Self::get_config_dir();
        Ok(config_dir.join("config.toml"))
    }

    /// 获取配置文件路径
    pub fn config_file_path(&self) -> &Path {
        &self.config_path
    }

    /// 生成默认配置文件（保存到指定路径）
    pub fn generate_default_config(target_path: &Path) -> ConfigResult<()> {
        let default_config = AppConfig::default();
        let toml = toml::to_string_pretty(&default_config)?;

        // 创建父目录
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(target_path, toml)?;

        log::info!("Generated default config at {:?}", target_path);
        Ok(())
    }

    /// 获取 XDG 数据目录
    pub fn get_data_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from(".local/share"))
            .join("tui-workstation")
    }

    /// 获取 XDG 配置目录
    pub fn get_config_dir() -> PathBuf {
        dirs::config_local_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("tui-workstation")
    }

    /// 获取缓存目录
    pub fn get_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("tui-workstation")
    }

    /// 默认日志文件路径
    pub fn default_log_file() -> PathBuf {
        Self::get_data_dir().join("logs").join("app.log")
    }

    /// 默认笔记目录
    pub fn default_notes_dir() -> PathBuf {
        Self::get_data_dir().join("notes")
    }

    /// 默认日记目录
    pub fn default_diary_dir() -> PathBuf {
        Self::get_data_dir().join("diary")
    }

    /// 默认数据库路径
    pub fn default_db_path() -> PathBuf {
        Self::get_data_dir().join("db")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.general.log_level, "info");
        assert_eq!(config.shortcuts.global_quit, "q");
        assert!(!config.modules.enabled.is_empty());
    }

    #[test]
    fn test_temp_dir_paths() {
        let data_dir = ConfigManager::get_data_dir();
        let config_dir = ConfigManager::get_config_dir();
        let cache_dir = ConfigManager::get_cache_dir();

        assert!(data_dir.ends_with("tui-workstation"));
        assert!(config_dir.ends_with("tui-workstation"));
        assert!(cache_dir.ends_with("tui-workstation"));
    }
}
