use anyhow::Result;
use chrono::Local;
use fern;
use log::{Level, LevelFilter};
use std::path::PathBuf;

/// 应用程序配置中的日志部分
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub log_level: String,
    pub log_to_file: bool,
    pub log_file: PathBuf,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_to_file: true,
            log_file: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("tui-workstation")
                .join("logs")
                .join("app.log"),
        }
    }
}

/// 初始化日志系统
pub fn init_logging(config: &LogConfig) -> Result<()> {
    // 解析日志级别
    let level = parse_log_level(&config.log_level);

    let mut dispatch = fern::Dispatch::new()
        .level(level)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                style_level(record.level()),
                record.target(),
                Local::now().format("%H:%M:%S"),
                message
            ))
        });

    // 文件日志
    if config.log_to_file {
        // 创建日志目录
        if let Some(parent) = config.log_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        dispatch = dispatch.chain(fern::log_file(config.log_file.clone())?);
    }

    dispatch.apply()?;

    log::info!("Logging initialized at level: {:?}", level);

    Ok(())
}

/// 解析日志级别字符串
fn parse_log_level(s: &str) -> LevelFilter {
    match s.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    }
}

/// 为日志级别添加 ANSI 颜色代码
fn style_level(level: Level) -> &'static str {
    match level {
        Level::Trace => "\x1b[90m", // 灰色
        Level::Debug => "\x1b[36m", // 青色
        Level::Info => "\x1b[32m",  // 绿色
        Level::Warn => "\x1b[33m",  // 黄色
        Level::Error => "\x1b[31m", // 红色
    }
}
