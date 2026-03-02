pub mod file_browser;
pub mod file_entry;

use crate::file_browser::FileBrowser;

use core::event::Action;
use core::module::Module as CoreModule;
use core::module::Shortcut;
use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};

/// FileBrowser 模块实现 Module trait
pub struct FileBrowserModule {
    browser: FileBrowser,
    initialized: bool,
}

impl FileBrowserModule {
    pub fn new(start_path: std::path::PathBuf) -> Self {
        Self {
            browser: FileBrowser::new(start_path).with_theme(ui::Theme::default()),
            initialized: false,
        }
    }
}

impl CoreModule for FileBrowserModule {
    fn name(&self) -> &str {
        "filebrowser"
    }

    fn title(&self) -> &str {
        "File Browser"
    }

    fn update(&mut self, event: CrosstermEvent) -> Action {
        match event {
            CrosstermEvent::Key(key) => self.browser.handle_key_event(key),
            _ => Action::None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.browser.render(frame, area);
    }

    fn save(&self) -> anyhow::Result<()> {
        // FileBrowser 不需要保存状态
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        // FileBrowser 不需要加载状态
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        self.browser.shortcuts()
    }

    fn init(&mut self) -> anyhow::Result<()> {
        if !self.initialized {
            let _ = self.browser.refresh();
            self.initialized = true;
        }
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        // FileBrowser 不需要清理资源
        Ok(())
    }
}

// 重新导出
pub use FileBrowserModule;
