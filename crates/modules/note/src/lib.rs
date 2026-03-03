//! Note module - Simple note taking (placeholder)

use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
};
use ui::Theme;

pub struct NoteModule {
    theme: Theme,
}

impl NoteModule {
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl CoreModule for NoteModule {
    fn name(&self) -> &str {
        "note"
    }
    fn title(&self) -> &str {
        "笔记"
    }
    fn update(&mut self, _event: CrosstermEvent) -> Action {
        Action::None
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(Text::from("笔记模块 - Coming Soon!"))
            .block(Block::default().title("笔记").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
    fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![]
    }
    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct NoteModuleWrapper {
    inner: NoteModule,
}

impl NoteModuleWrapper {
    pub fn new(_notes_dir: std::path::PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            inner: NoteModule {
                theme: ui::Theme::default(),
            },
        })
    }
}

impl CoreModule for NoteModuleWrapper {
    fn name(&self) -> &str {
        "note"
    }
    fn title(&self) -> &str {
        "Note"
    }
    fn update(&mut self, event: CrosstermEvent) -> Action {
        self.inner.update(event)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.inner.draw(frame, area)
    }
    fn save(&self) -> anyhow::Result<()> {
        self.inner.save()
    }
    fn load(&mut self) -> anyhow::Result<()> {
        self.inner.load()
    }
    fn shortcuts(&self) -> Vec<Shortcut> {
        self.inner.shortcuts()
    }
    fn init(&mut self) -> anyhow::Result<()> {
        self.inner.init()
    }
    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.inner.cleanup()
    }
}

pub use NoteModuleWrapper as Note;
