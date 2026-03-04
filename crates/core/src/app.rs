use crate::event::{Action, Event as AppEvent, Message};
use crossterm::{event, execute, terminal};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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
    module_buttons: Vec<(usize, Rect)>,
    log_panel_collapsed: bool,
    log_panel_height: u16,
    log_messages: Vec<(log::Level, String)>,
    dialog_visible: bool,
    dialog_message: Option<String>,
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
            module_buttons: Vec::new(),
            log_panel_collapsed: false,
            log_panel_height: 6,
            log_messages: vec![
                (log::Level::Info, "TUIWorker initialized".to_string()),
                (
                    log::Level::Info,
                    "Modules loaded. Arrow keys to switch.".to_string(),
                ),
                (
                    log::Level::Info,
                    "Press 'q' to quit, '?' for help".to_string(),
                ),
            ],
            dialog_visible: false,
            dialog_message: None,
        })
    }

    pub fn register_module<M: Module + 'static>(&mut self, module: M) {
        self.modules.push(Box::new(module));
    }

    pub fn toggle_log_panel(&mut self) {
        self.log_panel_collapsed = !self.log_panel_collapsed;
        self.status_message = if self.log_panel_collapsed {
            "Log panel collapsed".to_string()
        } else {
            "Log panel expanded".to_string()
        };
    }

    pub fn adjust_log_panel_height(&mut self, delta: i32) {
        if self.log_panel_collapsed {
            return;
        }

        let new_height = self.log_panel_height as i32 + delta;
        self.log_panel_height = new_height.max(3).min(20) as u16;
        self.status_message = format!("Log panel height: {}", self.log_panel_height);
    }

    pub fn add_log_message(&mut self, level: log::Level, message: String) {
        self.log_messages.push((level, message));
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
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
                            let message_str = match msg {
                                Message::Info(s) => s,
                                Message::Warning(s) => s,
                                Message::Error(s) => s,
                            };
                            self.dialog_visible = true;
                            self.dialog_message = Some(message_str.clone());
                            self.log_messages.push((log::Level::Info, message_str));
                            if self.log_messages.len() > 100 {
                                self.log_messages.remove(0);
                            }
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
                if self.dialog_visible
                    && (key.code == crossterm::event::KeyCode::Esc
                        || key.code == crossterm::event::KeyCode::Enter)
                {
                    self.dialog_visible = false;
                    self.dialog_message = None;
                    return Ok(None);
                }

                if self.dialog_visible {
                    return Ok(None);
                }

                match key.code {
                    crossterm::event::KeyCode::Char('q') => return Ok(Some(Action::Quit)),
                    crossterm::event::KeyCode::Char('l') => {
                        self.toggle_log_panel();
                        return Ok(None);
                    }
                    crossterm::event::KeyCode::Up => {
                        if !self.log_panel_collapsed {
                            self.adjust_log_panel_height(1);
                            return Ok(None);
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        if !self.log_panel_collapsed {
                            self.adjust_log_panel_height(-1);
                            return Ok(None);
                        }
                    }
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
                    _ => {}
                }

                let action = self
                    .modules
                    .get_mut(self.active_module_index)
                    .map(|m| m.update(crossterm::event::Event::Key(key)));
                Ok(action)
            }
            AppEvent::Mouse(mouse) => {
                if mouse.kind
                    == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
                {
                    for (module_index, button_area) in &self.module_buttons {
                        if button_area.contains(Position::new(mouse.column, mouse.row)) {
                            return Ok(Some(Action::SwitchModule(
                                self.modules
                                    .get(*module_index)
                                    .map(|m| m.name().to_string())
                                    .unwrap_or_default(),
                            )));
                        }
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

        let top_section_height = 3 + 1;
        if size.height < top_section_height + 5 || size.width < 20 {
            return;
        }

        let top_section = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(size);

        let app_bar_area = top_section[0];
        let separator_area = top_section[1];

        let main_section = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(separator_area);

        let work_status_area = main_section[0];
        let status_bar_area = main_section[1];

        let (work_area, log_area) = if !self.log_panel_collapsed {
            let log_constraint = Constraint::Length(self.log_panel_height);
            let work_log_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), log_constraint])
                .split(work_status_area);
            (work_log_split[0], Some(work_log_split[1]))
        } else {
            (work_status_area, None)
        };

        self.draw_app_bar(frame, app_bar_area);
        self.draw_separator(frame, separator_area);
        self.draw_work_area(frame, work_area);
        self.draw_status_bar(frame, status_bar_area);

        if let Some(log_area) = log_area {
            self.draw_log_panel(frame, log_area);
        }

        if self.dialog_visible {
            self.draw_dialog(frame, size);
        }
    }

    fn draw_app_bar(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        let left_width = 30;
        let right_width = 40;
        let middle_width = area.width.saturating_sub(left_width + right_width);

        if middle_width < 10 {
            return;
        }

        let horizontal_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_width),
                Constraint::Min(10),
                Constraint::Length(right_width),
            ])
            .split(area);

        let modules_area = horizontal_split[0];
        let _middle_area = horizontal_split[1];
        let help_area = horizontal_split[2];

        let modules_list: Vec<_> = self.modules.iter().map(|m| m.title()).collect();
        let total_modules = modules_list.len();

        let display_count = if total_modules > 0 {
            (left_width as usize / 4).min(total_modules.max(1))
        } else {
            1
        };

        if total_modules > 0 {
            let start_index = if display_count >= total_modules {
                0
            } else {
                let half = display_count / 2;
                if self.active_module_index + half >= total_modules {
                    total_modules.saturating_sub(self.active_module_index + half)
                } else if self.active_module_index > half {
                    self.active_module_index - half
                } else {
                    0
                }
            };

            let end_index = (start_index + display_count).min(total_modules);

            let mut current_x = modules_area.x + 1;
            self.module_buttons.clear();

            for i in start_index..end_index {
                let module_index = i;
                let is_active = module_index == self.active_module_index;
                let title = modules_list[module_index].clone();
                let title_len = title.len();

                let button_text = if is_active {
                    format!("[{}]", &title[0..title_len.min(1)].to_uppercase())
                } else {
                    format!(" {} ", &title[0..title_len.min(2)])
                };

                let button_width = button_text.len() as u16;

                if current_x + button_width <= modules_area.x + modules_area.width {
                    let button_area = Rect {
                        x: current_x,
                        y: area.y + 1,
                        width: button_width,
                        height: 1,
                    };

                    self.module_buttons.push((module_index, button_area));

                    let style = if is_active {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(ratatui::style::Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White).bg(Color::DarkGray)
                    };

                    let paragraph = Paragraph::new(button_text).style(style);
                    frame.render_widget(paragraph, button_area);
                }

                current_x += button_width + 1;
            }
        }

        let help_text = "[?] Help | [l] Logs ↓↑ | [q] Quit";

        let paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(paragraph, help_area);

        if total_modules > display_count {
            let nav_text = format!(" ({}/{})", self.active_module_index + 1, total_modules);
            let nav_paragraph = Paragraph::new(nav_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Cyan));
            frame.render_widget(nav_paragraph, modules_area);
        }

        let border = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(border, area);
    }

    fn draw_separator(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        let line = vec!['─'; area.width as usize];
        let paragraph = Paragraph::new(line.into_iter().collect::<String>())
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }

    fn draw_status_bar(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        let paragraph = Paragraph::new(self.status_message.clone())
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn draw_work_area(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        if let Some(module) = self.modules.get_mut(self.active_module_index) {
            module.draw(frame, area);
        }
    }

    fn draw_dialog(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        let overlay_width = 60.min(area.width - 4);
        let overlay_height = 20.min(area.height - 4);

        let overlay_area = Rect {
            x: area.x + (area.width.saturating_sub(overlay_width)) / 2,
            y: area.y + (area.height.saturating_sub(overlay_height)) / 2,
            width: overlay_width,
            height: overlay_height,
        };

        let content = if let Some(msg) = &self.dialog_message {
            vec![
                Line::from("Message"),
                Line::from(""),
                Line::from(msg.clone()),
                Line::from(""),
                Line::from("[Enter/Esc] Close"),
            ]
        } else {
            vec![Line::from("")]
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Dialog")
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    ),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(Clear, overlay_area);
        frame.render_widget(paragraph, overlay_area);
    }

    fn draw_log_panel(&mut self, frame: &mut ratatui::prelude::Frame, area: Rect) {
        if self.log_panel_collapsed {
            let indicator = " Logs »";
            let paragraph = Paragraph::new(Line::from(Span::styled(
                indicator,
                Style::default().fg(Color::Yellow),
            )))
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Right);

            frame.render_widget(paragraph, area);
        } else {
            let log_lines: Vec<Line> = self
                .log_messages
                .iter()
                .map(|(level, msg)| {
                    let level_str = match level {
                        log::Level::Error => "ERR",
                        log::Level::Warn => "WRN",
                        log::Level::Info => "INF",
                        log::Level::Debug => "DBG",
                        log::Level::Trace => "TRC",
                    };

                    let level_color = match level {
                        log::Level::Error => Color::Red,
                        log::Level::Warn => Color::Yellow,
                        log::Level::Info => Color::Green,
                        log::Level::Debug => Color::Cyan,
                        log::Level::Trace => Color::Gray,
                    };

                    Line::from(vec![
                        Span::styled(
                            format!("[{}] ", level_str),
                            Style::default().fg(level_color),
                        ),
                        Span::styled(msg, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let indicator = "« Logs";
            let title = format!("Logs (l) ↑↓{}", indicator);

            let paragraph = Paragraph::new(log_lines)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .alignment(Alignment::Left)
                .scroll((
                    (self
                        .log_messages
                        .len()
                        .saturating_sub(area.height.saturating_sub(2) as usize))
                        as u16,
                    0,
                ));

            frame.render_widget(paragraph, area);
        }
    }
}
