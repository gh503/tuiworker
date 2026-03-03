//! Diary module - Calendar diary (placeholder)
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{layout::{Alignment, Rect}, text::Text, widgets::{Block, Borders, Paragraph}, Frame};
use ui::Theme;
use core::{event::Action, module::{Module as CoreModule, Shortcut}};

pub struct DiaryModule { theme: Theme }
impl DiaryModule {
    pub fn new(_dir: std::path::PathBuf) -> Self { Self { theme: Theme::default() } }
    pub fn with_theme(mut self, theme: Theme) -> Self { self.theme = theme; self }
}
impl CoreModule for DiaryModule {
    fn name(&self) -> &str { "diary" }
    fn title(&self) -> &str { "日记" }
    fn update(&mut self, _event: CrosstermEvent) -> Action { Action::None }
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let p = Paragraph::new(Text::from("日记模块 - Coming Soon!"))
            .block(Block::default().title("日历").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }
    fn save(&self) -> anyhow::Result<()> { Ok(()) }
    fn load(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn shortcuts(&self) -> Vec<Shortcut> { vec![] }
    fn init(&mut self) -> anyhow::Result<()> { Ok(()) }
    fn cleanup(&mut self) -> anyhow::Result<()> { Ok(()) }
}
pub use DiaryModule as Diary;
