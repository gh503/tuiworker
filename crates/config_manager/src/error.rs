use thiserror::Error;

/// 配置错误类型
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    FileNotFound(std::path::PathBuf),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("Invalid config value: {0}")]
    InvalidValue(String),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
