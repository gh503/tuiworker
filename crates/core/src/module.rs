use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use ratatui::{layout::Rect, Frame};

use crate::event::Action;

/// 所有功能模块必须实现的核心接口
pub trait Module {
    /// 模块名称（唯一标识符）
    fn name(&self) -> &str;

    /// 显示标题（用于标签栏）
    fn title(&self) -> &str;

    /// 处理输入事件
    /// 返回 Action 指定应用层面的操作（切换模块、退出等）
    fn update(&mut self, event: CrosstermEvent) -> Action;

    /// 绘制 UI
    fn draw(&mut self, frame: &mut Frame, area: Rect);

    /// 保存状态到持久化存储
    fn save(&self) -> anyhow::Result<()>;

    /// 从持久化存储加载状态
    fn load(&mut self) -> anyhow::Result<()>;

    /// 模块快捷键列表（用于帮助显示）
    fn shortcuts(&self) -> Vec<Shortcut>;

    /// 获取模块状态字符串（用于状态栏显示）
    fn get_status(&self) -> String {
        self.title().to_string()
    }

    /// 模块初始化（首次激活时调用）
    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// 模块清理（切换到其他模块时调用）
    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 快捷键定义
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub key: &'static str,
    pub description: &'static str,
}
