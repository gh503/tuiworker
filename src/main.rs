mod cli;

use log::info;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志
    logging::init_logging(&logging::LogConfig::default())?;
    info!("TUI Workstation starting...");

    // TODO: 初始化配置管理
    // TODO: 初始化数据库
    // TODO: 构建应用实例
    // TODO: 注册模块
    // TODO: 运行应用

    println!("TUI Workstation - Coming Soon!");
    println!("See docs/implementation-plan.md for progress");

    Ok(())
}
