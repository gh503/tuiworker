pub mod item;
pub mod todo_impl;

use crate::todo_impl::TodoModule;

use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};
use core::module::Module as CoreModule;
use core::event::Action;
use core::module::Shortcut;

/// Todo 模块实现 Module trait
pub struct TodoModuleWrapper {
    inner: TodoModule,
}

impl TodoModuleWrapper {
    pub fn new(db: storage::NamespacedDatabase) -> Self {
        Self {
            inner: TodoModule::new(db).with_theme(ui::Theme::default()),
        }
    }
}

impl CoreModule for TodoModuleWrapper {
    fn name(&self) -> &str {
        "todo"
    }

    fn title(&self) -> &str {
        "Todo"
    }

    fn update(&mut self, event: CrosstermEvent) -> Action {
        match event {
            CrosstermEvent::Key(key) => self.inner.handle_key_event(key),
            _ => Action::None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.inner.render(frame, area);
    }

    fn save(&self) -> anyhow::Result<()> {
        // Todo 自动保存到数据库，这里不需要操作
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        self.inner.load()
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        self.inner.shortcuts()
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.inner.load()
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

// 重新导出
pub use item::{TodoItem, Priority, TodoStatus};
pub use TodoModuleWrapper as Todo;
