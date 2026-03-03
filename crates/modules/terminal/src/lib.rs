//! Terminal module - Terminal emulator (placeholder)
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{layout::{Alignment, Rect}, text::Text, widgets::{Block, Borders, Paragraph}, Frame};
use ui::Theme;
use core::{event::Action, module::{Module as CoreModule, Shortcut}};

pub struct TerminalModule { theme: Theme }
impl TerminalModule {
    pub fn new() -> Self { Self { theme: Theme::default() } }
    pub fn with_theme(mut self, theme: Theme) -> Self { self.theme = theme; self }
}
impl CoreModule for TerminalModule {
    fn name(&self) -> &str { "terminal" }
    fn title(&self) -> &str { "终端" }
    fn update(&mut self, _event: CrosstermEvent) -> Action { Action::None }
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let p = Paragraph::new(Text::from("终端模拟器 - Coming Soon!"))
            .block(Block::default().title("终端").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }
    fn save(&self) -> anyhow::Result<()> { Ok(()) }
    fn load(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn shortcuts(&self) -> Vec<Shortcut> { vec![] }
    fn init(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> anyhow::Result<()> { Ok(()) }
}
pub use TerminalModule as Terminal;
