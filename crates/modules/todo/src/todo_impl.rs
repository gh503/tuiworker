use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::item::{TodoItem, Priority, TodoStatus};

pub struct TodoModule {
    items: Vec<TodoItem>,
    selected_index: usize,
    db: storage::NamespacedDatabase,
    show_completed: bool,
    filter_tag: Option<String>,
    sort_by: SortBy,
    theme: ui::Theme,
}

#[derive(Debug, Clone, Copy)]
enum SortBy {
    Created,
    Priority,
    Due,
}

impl TodoModule {
    pub fn new(db: storage::NamespacedDatabase) -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            db,
            show_completed: true,
            filter_tag: None,
            sort_by: SortBy::Created,
            theme: ui::Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: ui::Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        self.items.clear();

        // 从数据库加载所有待办事项
        let db_items: Vec<TodoItem> = self
            .db
            .iter()
            .filter_map(|(key, value)| match String::from_utf8(key) {
                Ok(k) if k.starts_with("item:") => serde_json::from_slice(&value).ok(),
                _ => None,
            })
            .collect();

        self.items = db_items;
        self.filter_and_sort();
        self.selected_index = self.selected_index.min(self.items.len().saturating_sub(1));

        Ok(())
    }

    pub fn add(&mut self, item: TodoItem) -> anyhow::Result<()> {
        let key = format!("item:{}", item.id);
        self.db.insert_json(key.as_bytes(), &item)?;
        self.items.push(item);
        self.filter_and_sort();
        Ok(())
    }

    pub fn update(&mut self, id: uuid::Uuid, item: TodoItem) -> anyhow::Result<()> {
        let key = format!("item:{}", id);
        self.db.insert_json(key.as_bytes(), &item)?;
        if let Some(existing_item) = self.items.iter_mut().find(|i| i.id == id) {
            *existing_item = item;
        }
        self.filter_and_sort();
        Ok(())
    }

    pub fn delete(&mut self, id: uuid::Uuid) -> anyhow::Result<()> {
        let key = format!("item:{}", id);
        self.db.remove(key.as_bytes())?;
        self.items.retain(|i| i.id != id);
        self.selected_index = self.selected_index.min(self.items.len().saturating_sub(1));
        Ok(())
    }

    pub fn toggle_complete(&mut self, id: uuid::Uuid) -> anyhow::Result<()> {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            let item_clone = item.clone();  // 提前克隆避免借用冲突
            if item.status == TodoStatus::Completed {
                item.status = TodoStatus::Pending;
                item.completed_at = None;
            } else {
                item.status = TodoStatus::Completed;
                item.completed_at = Some(chrono::Utc::now());
                self.add_to_tag_history(&item_clone);
            }
            self.update(id, item_clone)?;
            self.filter_and_sort();
        }
        Ok(())
    }
    fn add_to_tag_history(&self, item: &TodoItem) {
        for tag in &item.tags {
            // 更新标签完成历史（简化实现）
            let _ = self.db.insert(
                format!("tag:completed:{}:{}", tag, item.id).as_bytes(),
                chrono::Utc::now().timestamp().to_string().as_bytes(),
            );
        }
    }

    pub fn filter_by_tag(&mut self, tag: Option<String>) {
        self.filter_tag = tag;
        self.filter_and_sort();
    }

    pub fn toggle_completed(&mut self) {
        self.show_completed = !self.show_completed;
        self.filter_and_sort();
    }

    pub fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Created => SortBy::Priority,
            SortBy::Priority => SortBy::Due,
            SortBy::Due => SortBy::Created,
        };
        self.filter_and_sort();
    }

    fn filter_and_sort(&mut self) {
        // 过滤
        let mut filtered: Vec<TodoItem> = self
            .items
            .iter()
            .filter(|item| {
                if item.status == TodoStatus::Completed && !self.show_completed {
                    return false;
                }
                if let Some(ref tag) = self.filter_tag {
                    return item.tags.contains(tag);
                }
                true
            })
            .cloned()
            .collect();

        // 排序
        match self.sort_by {
            SortBy::Created => {
                filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            SortBy::Priority => {
                filtered.sort_by(|a, b| {
                    let priority_order = |p: &Priority| match p {
                        Priority::High => 0,
                        Priority::Medium => 1,
                        Priority::Low => 2,
                    };
                    priority_order(&a.priority)
                        .cmp(&priority_order(&b.priority))
                        .then_with(|| a.created_at.cmp(&b.created_at))
                });
            }
            SortBy::Due => {
                filtered.sort_by(|a, b| match (&a.due_date, &b.due_date) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (Some(d1), Some(d2)) => d1.cmp(d2).reverse(),
                });
            }
        }

        self.items = filtered;
        self.selected_index = self.selected_index.min(self.items.len().saturating_sub(1));
    }

    pub fn get_selected(&self) -> Option<&TodoItem> {
        self.items.get(self.selected_index)
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        self.render_todo_list(frame, chunks[0]);
        self.render_info_panel(frame, chunks[1]);
    }

    fn render_todo_list(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let prefix = if i == self.selected_index { ">" } else { " " };

                let priority = item.priority.symbol();
                let status = item.status.symbol();
                let title = if item.status == TodoStatus::Completed {
                    format!("{} [{status}] ~~{}~~", prefix, item.title,)
                } else {
                    format!("{} [{}] {}{}", prefix, status, priority, item.title,)
                };

                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else if item.status == TodoStatus::Completed {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(title).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("Todos ({})", self.items.len()))
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .highlight_style(self.theme.highlight());

        frame.render_widget(list, area);
    }

    fn render_info_panel(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![Line::from("Todo Info"), Line::from(""), Line::from("")];

        if let Some(item) = self.get_selected() {
            let priority_style = Style::default().fg(item.priority.display_color());
            let status_text = match item.status {
                TodoStatus::Pending => "Pending",
                TodoStatus::InProgress => "In Progress",
                TodoStatus::Completed => "Completed",
            };

            let mut info = vec![
                Line::from(vec![
                    Span::styled("Title: ", Style::default().fg(Color::Gray)),
                    Span::raw(&item.title),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Gray)),
                    Span::styled(status_text, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::styled("Priority: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{:?}", item.priority), priority_style),
                ]),
                Line::from(vec![
                    Span::styled("Created: ", Style::default().fg(Color::Gray)),
                    Span::raw(item.created_at.format("%Y-%m-%d %H:%M").to_string()),
                ]),
            ];

            if let Some(desc) = &item.description {
                info.push(Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Gray)),
                    Span::raw(desc),
                ]));
            }

            if !item.tags.is_empty() {
                let tags_str = item.tags.join(", ");
                info.push(Line::from(vec![
                    Span::styled("Tags: ", Style::default().fg(Color::Gray)),
                    Span::raw(tags_str),
                ]));
            }

            if let Some(due) = item.due_date {
                info.push(Line::from(vec![
                    Span::styled("Due: ", Style::default().fg(Color::Gray)),
                    Span::raw(due.format("%Y-%m-%d %H:%M").to_string()),
                ]));
            }

            lines.extend(info);

            // 显示快捷键提示
            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                " shortcuts:",
                Style::default().fg(Color::Cyan),
            )]));
            lines.push(Line::from("n: Add | d: Delete | e: Edit"));
        } else {
            lines.push(Line::from("No todo selected"));
            lines.push(Line::from(""));
            lines.push(Line::from("Press 'n' to create new todo"));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("Info")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> core::event::Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                core::event::Action::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.items.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                core::event::Action::Consumed
            }
            KeyCode::Char('x') => {
                if let Some(item) = self.get_selected() {
                    let _ = self.toggle_complete(item.id);
                }
                core::event::Action::Consumed
            }
            KeyCode::Char('c') => {
                self.toggle_completed();
                core::event::Action::Consumed
            }
            KeyCode::Char('s') => {
                self.cycle_sort();
                core::event::Action::Consumed
            }
            KeyCode::Char('?') => core::event::Action::ShowMessage(core::event::Message::Info(
                "j/k: Navigate | x: Toggle complete | c: Toggle filter | s: Sort".to_string(),
            )),
            _ => core::event::Action::None,
        }
    }

    pub fn shortcuts(&self) -> Vec<core::module::Shortcut> {
        vec![
            core::module::Shortcut {
                key: "j/k",
                description: "Navigate",
            },
            core::module::Shortcut {
                key: "x",
                description: "Toggle complete",
            },
            core::module::Shortcut {
                key: "c",
                description: "Toggle filter",
            },
            core::module::Shortcut {
                key: "s",
                description: "Change sort",
            },
        ]
    }
}
