use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::{Theme, ColorScheme};

/// 渲染部件 trait
pub trait RenderWidget {
    fn render(&self, frame: &mut Frame, area: Rect);
}

/// 标签栏
pub struct TabBar {
    tabs: Vec<String>,
    active_index: usize,
    theme: Theme,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
            theme: Theme { color_scheme: ColorScheme::Default },
        }
    }

    pub fn with_tabs(mut self, tabs: Vec<String>) -> Self {
        self.tabs = tabs;
        self
    }

    pub fn with_active(mut self, index: usize) -> Self {
        self.active_index = index.clamp(0, self.tabs.len().saturating_sub(1));
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderWidget for TabBar {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let tabs: Vec<Line> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if i == self.active_index {
                    self.theme.tab_active()
                } else {
                    self.theme.tab_bar()
                };

                Line::from(vec![Span::styled(tab, style)])
            })
            .collect();

        let paragraph = Paragraph::new(tabs)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(self.theme.border()),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

/// 状态栏
pub struct StatusBar {
    left: String,
    center: String,
    right: String,
    theme: Theme,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            left: String::new(),
            center: String::new(),
            right: String::new(),
            theme: Theme { color_scheme: ColorScheme::Default },
        }
    }

    pub fn with_left(mut self, text: String) -> Self {
        self.left = text;
        self
    }

    pub fn with_center(mut self, text: String) -> Self {
        self.center = text;
        self
    }

    pub fn with_right(mut self, text: String) -> Self {
        self.right = text;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderWidget for StatusBar {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let spans = vec![
            Span::styled(&self.left, Style::default()),
            Span::raw(" "),
            Span::styled(&self.center, Style::default()),
        ];

        let paragraph = Paragraph::new(Line::from(spans))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(self.theme.border()),
            )
            .alignment(Alignment::Left);

        // 只绘制左边部分，右边的需要另外绘制
        frame.render_widget(paragraph, area);

        // 在右侧绘制
        if !self.right.is_empty() {
            let right_area = Rect {
                x: area
                    .x
                    .saturating_add(area.width.saturating_sub(self.right.len() as u16 + 1)),
                y: area.y,
                width: self.right.len() as u16 + 1,
                height: area.height,
            };
            let right_paragraph = Paragraph::new(self.right.as_str()).alignment(Alignment::Left);
            frame.render_widget(right_paragraph, right_area);
        }
    }
}

/// 消息列表 widget
pub struct MessageList {
    items: Vec<String>,
    selected_index: usize,
    theme: Theme,
}

impl MessageList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            theme: Theme { color_scheme: ColorScheme::Default },
        }
    }

    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn with_selected(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderWidget for MessageList {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected_index {
                    self.theme.highlight()
                } else {
                    Style::default()
                };
                ListItem::new(item.as_str()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .highlight_style(self.theme.highlight());

        frame.render_widget(list, area);
    }
}

/// 边框 widget
pub struct BorderedBlock {
    title: Option<String>,
    theme: Theme,
}

impl BorderedBlock {
    pub fn new() -> Self {
        Self {
            title: None,
            theme: Theme { color_scheme: ColorScheme::Default },
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderWidget for BorderedBlock {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let mut block = Block::default()
            .border_type(BorderType::Plain)
            .border_style(self.theme.border());

        if let Some(title) = &self.title {
            block = block.title(title.as_str());
        }

        frame.render_widget(block, area);
    }
}

/// 帮助弹窗
pub struct HelpModal {
    shortcuts: Vec<(String, String)>,
    theme: Theme,
}

impl HelpModal {
    pub fn new() -> Self {
        Self {
            shortcuts: Vec::new(),
            theme: Theme { color_scheme: ColorScheme::Default },
        }
    }

    pub fn with_shortcuts(mut self, shortcuts: Vec<(String, String)>) -> Self {
        self.shortcuts = shortcuts;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

impl RenderWidget for HelpModal {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let mut text = String::from("Shortcuts:\n\n");

        for (key, desc) in &self.shortcuts {
            text.push_str(&format!("  {:<20} {}\n", key, desc));
        }

        text.push_str("\nPress ? to close");

        let paragraph = Paragraph::new(text.as_str())
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar() {
        let bar = TabBar::new()
            .with_tabs(vec!["Tab1".to_string(), "Tab2".to_string()])
            .with_active(0);

        assert_eq!(bar.tabs.len(), 2);
        assert_eq!(bar.active_index, 0);
    }

    #[test]
    fn test_status_bar() {
        let bar = StatusBar::new()
            .with_left("FileBrowser".to_string())
            .with_right("Press ? for help".to_string());

        assert_eq!(bar.left, "FileBrowser");
        assert_eq!(bar.right, "Press ? for help");
    }
}
