use crate::event::{Action, Event as AppEvent};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute, terminal,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::module::Module;

/// 主应用程序
pub struct App {
    modules: Vec<Box<dyn Module>>,
    active_module_index: usize,
    event_sender: mpsc::UnboundedSender<AppEvent>,
    event_receiver: mpsc::UnboundedReceiver<AppEvent>,
    should_quit: bool,
    last_frame_time: Instant,
    tick_rate: Duration,
}

impl App {
    /// 创建新应用
    pub fn new() -> anyhow::Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();

        Ok(Self {
            modules: Vec::new(),
            active_module_index: 0,
            event_sender: sender,
            event_receiver: receiver,
            should_quit: false,
            last_frame_time: Instant::now(),
            tick_rate: Duration::from_millis(33), // ~30 FPS
        })
    }

    /// 添加模块
    pub fn register_module<M: Module + 'static>(&mut self, module: M) {
        self.modules.push(Box::new(module));
    }

    /// 运行主循环
    pub fn run(&mut self) -> anyhow::Result<()> {
        // 初始化终端
        let _stdout = io::stdout();
        let backend = CrosstermBackend::new(_stdout);
        let mut terminal = Terminal::new(backend)?;

        // 设置终端
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), event::EnableMouseCapture)?;

        // 初始化所有模块
        for module in self.modules.iter_mut() {
            let _ = module.init(); // 忽略初始化错误，不中断应用启动
        }

        let result = self.run_inner(&mut terminal);

        // 清理
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), event::DisableMouseCapture)?;

        // 清理所有模块
        for module in self.modules.iter_mut() {
            let _ = module.cleanup();
        }

        result
    }

    /// 内部运行循环
    fn run_inner(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> anyhow::Result<()> {
        loop {
            // 检查是否需要退出
            if self.should_quit {
                return Ok(());
            }

            // 控制帧率
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < self.tick_rate {
                std::thread::sleep(self.tick_rate - elapsed);
            }
            self.last_frame_time = Instant::now();

            // 处理事件
            while let Ok(event) = self.event_receiver.try_recv() {
                if let Some(action) = self.handle_event(event)? {
                    match action {
                        Action::Quit => {
                            self.should_quit = true;
                        }
                        Action::SwitchModule(name) => {
                            if let Some(index) = self.modules.iter().position(|m| m.name() == name)
                            {
                                self.active_module_index = index;
                                // 切换模块时初始化新模块
                                if let Err(e) = self.modules[index].init() {
                                    log::error!("Failed to initialize module {}: {:?}", name, e);
                                }
                            }
                        }
                        Action::ShowMessage(msg) => {
                            log::info!("{:?}", msg);
                        }
                        _ => {}
                    }
                }
            }

            // 渲染
            terminal.draw(|frame| {
                self.draw(frame);
            })?;
        }
    }

    /// 处理事件
    fn handle_event(&mut self, event: AppEvent) -> anyhow::Result<Option<Action>> {
        match event {
            AppEvent::Key(key) => {
                // 全局快捷键检查可以在这里进行
                // 然后传递给当前活动模块
                let action = self
                    .modules
                    .get_mut(self.active_module_index)
                    .map(|m| m.update(crossterm::event::Event::Key(key)));
                Ok(action)
            }
            AppEvent::Mouse(mouse) => {
                let action = self
                    .modules
                    .get_mut(self.active_module_index)
                    .map(|m| m.update(crossterm::event::Event::Mouse(mouse)));
                Ok(action)
            }
            AppEvent::Resize(_, _) => {
                // 窗口大小变化，重新渲染
                Ok(None)
            }
            AppEvent::Timer => Ok(None),
            AppEvent::TaskComplete(_) => Ok(None),
        }
    }

    /// 绘制帧
    fn draw(&mut self, frame: &mut ratatui::prelude::Frame) {
        let size = frame.size();

        // 绘制当前活动模块
        if let Some(module) = self.modules.get_mut(self.active_module_index) {
            module.draw(frame, size);
        }
    }
}
