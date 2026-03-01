use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Quit,
    TabNext,
    TabPrevious,
    NewTab,
    CloseTab,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
    Backspace,
    Delete,
    Char(char),
    Other(KeyEvent),
}

impl AppEvent {
    pub fn from_crossterm_event(event: Event) -> Option<Self> {
        match event {
            Event::Key(key_event) => {
                if key_event.kind != KeyEventKind::Press {
                    return None;
                }
                Some(Self::from_key_event(key_event))
            }
            Event::Mouse(_) => None,
            Event::Resize(_, _) => None,
            Event::FocusGained | Event::FocusLost => None,
            Event::Paste(_) => None,
        }
    }

    fn from_key_event(key_event: KeyEvent) -> Self {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => AppEvent::Quit,
            KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    AppEvent::TabPrevious
                } else {
                    AppEvent::TabNext
                }
            }
            KeyCode::Up => AppEvent::Up,
            KeyCode::Down => AppEvent::Down,
            KeyCode::Left => AppEvent::Left,
            KeyCode::Right => AppEvent::Right,
            KeyCode::Enter => AppEvent::Enter,
            KeyCode::Esc => AppEvent::Esc,
            KeyCode::Backspace => AppEvent::Backspace,
            KeyCode::Delete => AppEvent::Delete,
            KeyCode::Char('t') | KeyCode::Char('T') => AppEvent::NewTab,
            KeyCode::Char('w') | KeyCode::Char('W') => AppEvent::CloseTab,
            KeyCode::Char(c) => AppEvent::Char(c),
            _ => AppEvent::Other(key_event),
        }
    }
}
