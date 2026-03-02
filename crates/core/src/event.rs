use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use uuid::Uuid;

/// 应用级事件类型
#[derive(Debug)]
pub enum Event {
    /// 键盘事件
    Key(KeyEvent),

    /// 鼠标事件
    Mouse(MouseEvent),

    /// 窗口大小变化
    Resize(u16, u16),

    /// 定时器事件（用于刷新/动画）
    Timer,

    /// 异步任务完成
    TaskComplete(Uuid),
}

/// 应用操作（由模块 update 返回）
#[derive(Debug, PartialEq)]
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

/// 消息类型
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Info(String),
    Warning(String),
    Error(String),
}

/// 命令类型
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub command: String,
    pub args: Vec<String>,
}
