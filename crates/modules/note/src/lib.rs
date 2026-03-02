pub mod mod_rs;

use crate::mod_rs::{NoteModule, NoteItem};

use crossterm::event::Event as CrosstermEvent;
use ratatui::{layout::Rect, Frame};
use core::module::Module as CoreModule;
use core::event::Action;
use core::module::Shortcut;

pub struct NoteModuleWrapper {
    inner: NoteModule,
}

impl NoteModuleWrapper {
    pub fn new(notes_dir: std::path::PathBuf) -> anyhow::Result<Self> {
        Ok(Self {
            inner: NoteModule::new(notes_dir)?.with_theme(ui::Theme::default()),
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
        match event {
            CrosstermEvent::Key(key) => self.inner.handle_key_event(key),
            _ => Action::None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.inner.render(frame, area);
    }

    fn save(&self) -> anyhow::Result<()> {
        self.inner.save()
    }

    fn load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        self.inner.shortcuts()
    }

    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub use NoteModuleWrapper as Note;
