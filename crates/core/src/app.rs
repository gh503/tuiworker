use crate::event::{Action, Event as AppEvent, Message};
use crossterm::{event, execute, terminal};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::module::Module;

pub struct App {
    modules: Vec<Box<dyn Module>>,
    active_module_index: usize,
    event_sender: mpsc::UnboundedSender<AppEvent>,
    event_receiver: mpsc::UnboundedReceiver<AppEvent>,
    should_quit: bool,
    last_frame_time: Instant,
    tick_rate: Duration,
    status_message: String,
    module_switcher_area: Option<Rect>,
    module_button_areas: Vec<(usize, Rect)>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let (sender, receiver) = mpsc::unbounded_channel();

        Ok(Self {
            modules: Vec::new(),
            active_module_index: 0,
            event_sender: sender,
            event_receiver: receiver,
            should_quit: false,
            last_frame_time: Instant::now(),
            tick_rate: Duration::from_millis(33),
            status_message: "Ready".to_string(),
            module_switcher_area: None,
            module_button_areas: Vec::new(),
        })
    }

    pub fn register_module<M: Module + 'static>(&mut self, module: M) {
        self.modules.push(Box::new(module));
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let _stdout = io::stdout();
        let backend = CrosstermBackend::new(_stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal::enable_raw_mode()?;
        execute!(
            io::stdout(),
            event::EnableMouseCapture,
            terminal::EnterAlternateScreen
        )?;

        let event_sender = self.event_sender.clone();
        std::thread::spawn(move || loop {
            if let Ok(ce) = crossterm::event::read() {
                let app_event = match ce {
                    crossterm::event::Event::Key(key) => AppEvent::Key(key),
                    crossterm::event::Event::Mouse(mouse) => AppEvent::Mouse(mouse),
                    crossterm::event::Event::Resize(x, y) => AppEvent::Resize(x, y),
                    crossterm::event::Event::FocusLost | crossterm::event::Event::FocusGained => {
                        AppEvent::Timer
                    }
                    crossterm::event::Event::Paste(_) => {
                        continue;
                    }
                };

                if event_sender.send(app_event).is_err() {
                    break;
                }
            }
        });

        for module in self.modules.iter_mut() {
            let _ = module.init();
        }

        let result = self.run_inner(&mut terminal);

        terminal::disable_raw_mode()?;
        execute!(
            io::stdout(),
            event::DisableMouseCapture,
            terminal::LeaveAlternateScreen
        )?;

        for module in self.modules.iter_mut() {
            let _ = module.cleanup();
        }

        result
    }

    fn run_inner(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> anyhow::Result<()> {
        loop {
            if self.should_quit {
                return Ok(());
            }

            let elapsed = self.last_frame_time.elapsed();
            if elapsed < self.tick_rate {
                std::thread::sleep(self.tick_rate - elapsed);
            }
            self.last_frame_time = Instant::now();

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
                                if let Err(e) = self.modules[index].init() {
                                    log::error!("Failed to initialize module {}: {:?}", name, e);
                                }
                                if let Some(module) = self.modules.get(index) {
                                    self.status_message =
                                        format!("Switched to: {}", module.title());
                                }
                            }
                        }
                        Action::ShowMessage(msg) => {
                            self.status_message = format!("{:?}", msg);
                        }
                        _ => {}
                    }
                }
            }

            terminal.draw(|frame| {
                self.draw(frame);
            })?;
        }
    }

    fn handle_event(&mut self, event: AppEvent) -> anyhow::Result<Option<Action>> {
        match event {
            AppEvent::Key(key) => {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => return Ok(Some(Action::Quit)),
                    crossterm::event::KeyCode::Char('?') => {
                        let shortcuts = self.active_module_shortcuts();
                        return Ok(Some(Action::ShowMessage(Message::Info(
                            shortcuts.join(" | "),
                        ))));
                    }
                    crossterm::event::KeyCode::Right => {
                        if self.modules.is_empty() {
                            return Ok(None);
                        }
                        self.active_module_index =
                            (self.active_module_index + 1) % self.modules.len();
                        let module_name = self.modules[self.active_module_index].name().to_string();
                        return Ok(Some(Action::SwitchModule(module_name)));
                    }
                    crossterm::event::KeyCode::Left => {
                        if self.modules.is_empty() {
                            return Ok(None);
                        }
                        if self.active_module_index == 0 {
                            self.active_module_index = self.modules.len() - 1;
                        } else {
                            self.active_module_index -= 1;
                        }
                        let module_name = self.modules[self.active_module_index].name().to_string();
                        return Ok(Some(Action::SwitchModule(module_name)));
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        if c.is_ascii_digit() {
                            let digit = c.to_digit(10).unwrap() as usize;
                            if digit == 0 {
                                if self.modules.len() >= 10 {
                                    let module_name = self.modules[9].name().to_string();
                                    self.active_module_index = 9;
                                    return Ok(Some(Action::SwitchModule(module_name)));
                                }
                            } else if digit <= self.modules.len() {
                                let module_name = self.modules[digit - 1].name().to_string();
                                self.active_module_index = digit - 1;
                                return Ok(Some(Action::SwitchModule(module_name)));
                            }
                        }
                    }
                    _ => {}
                }

                let action = self
                    .modules
                    .get_mut(self.active_module_index)
                    .map(|m| m.update(crossterm::event::Event::Key(key)));
                Ok(action)
            }
            AppEvent::Mouse(mouse) => {
                if let crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) =
                    mouse.kind
                {
                    if let Some(module_index) = self.get_module_at_position(mouse.column, mouse.row)
                    {
                        return Ok(Some(Action::SwitchModule(
                            self.modules
                                .get(module_index)
                                .map(|m| m.name().to_string())
                                .unwrap_or_default(),
                        )));
                    }
                }

                let action = self
                    .modules
                    .get_mut(self.active_module_index)
                    .map(|m| m.update(crossterm::event::Event::Mouse(mouse)));
                Ok(action)
            }
            AppEvent::Resize(_, _) => Ok(None),
            AppEvent::Timer => Ok(None),
            AppEvent::TaskComplete(_) => Ok(None),
        }
    }

    fn get_module_at_position(&self, x: u16, y: u16) -> Option<usize> {
        for (module_index, button_area) in &self.module_button_areas {
            if button_area.contains(Position::new(x, y)) {
                return Some(*module_index);
            }
        }
        None
    }

    fn active_module_shortcuts(&self) -> Vec<String> {
        self.modules
            .get(self.active_module_index)
            .map(|m| m.shortcuts())
            .unwrap_or_default()
            .iter()
            .map(|s| format!("{}: {}", s.key, s.description))
            .collect()
    }

    fn draw(&mut self, frame: &mut ratatui::prelude::Frame) {
        let size = frame.size();
        if size.height < 5 || size.width < 10 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(size);

        let active_module = self.active_module_index;
        let module_title = self
            .modules
            .get(active_module)
            .map(|m| m.title().to_string())
            .unwrap_or("No Module".to_string());

        self.draw_top_bar(frame, chunks[0], &module_title);
        self.draw_menu_separator(frame, chunks[1]);

        let work_area = chunks[2];
        let status_bar_area = chunks[3];
        let switcher_area = chunks[4];

        self.module_switcher_area = Some(switcher_area);
        self.draw_work_area(frame, work_area);
        self.draw_status_bar(frame, status_bar_area);
        self.draw_module_switcher(frame, switcher_area);
    }

    fn draw_top_bar(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect, title: &str) {
        let line = Line::from(vec![
            Span::styled(
                "TUIWorker",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(title, Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled("[?]", Style::default().fg(Color::Yellow)),
            Span::raw(" Help | "),
            Span::styled("[q]", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit"),
        ]);

        let paragraph = Paragraph::new(vec![line])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    fn draw_menu_separator(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        let line = vec!['─'; area.width as usize];
        let paragraph = Paragraph::new(line.into_iter().collect::<String>())
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }

    fn draw_work_area(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        if let Some(module) = self.modules.get_mut(self.active_module_index) {
            module.draw(frame, area);
        }
    }

    fn draw_status_bar(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        let paragraph = Paragraph::new(self.status_message.clone())
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn draw_module_switcher(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        self.module_button_areas.clear();

        if self.modules.is_empty() {
            frame.render_widget(
                Paragraph::new("No modules registered")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                    )
                    .alignment(Alignment::Center),
                area,
            );
            return;
        }

        let button_width = 12;
        let gap = 1;
        let max_buttons = ((area.width - 2) / (button_width + gap)) as usize;
        let mut start_index = 0;

        if self.active_module_index >= max_buttons {
            start_index = self.active_module_index - max_buttons + 1;
        }

        let end_index = (start_index + max_buttons).min(self.modules.len());

        let mut spans = Vec::new();
        let mut current_x = area.x + 1;

        for i in start_index..end_index {
            let module = &self.modules[i];
            let is_active = i == self.active_module_index;

            let button_text = format!(" {} ", module.title());

            let style = if is_active {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            };

            let button_area = Rect {
                x: current_x,
                y: area.y + 1,
                width: button_width,
                height: 1,
            };

            self.module_button_areas.push((i, button_area));

            spans.push(Span::styled(button_text, style));
            spans.push(Span::raw(" ".repeat(gap as usize)));

            current_x += button_width + gap;
        }

        let info_text = if start_index > 0 || end_index < self.modules.len() {
            format!(" ({}/{}) ", start_index + 1, self.modules.len())
        } else {
            format!(" ({}) ", self.modules.len())
        };

        spans.push(Span::styled(info_text, Style::default().fg(Color::Gray)));

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Modules:", Style::default().fg(Color::Cyan)),
                Span::raw(" Arrow keys: navigate | Click: select"),
            ]),
            Line::from(spans),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}
