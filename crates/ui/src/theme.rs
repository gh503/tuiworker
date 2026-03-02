use ratatui::style::{Color, Style};

/// 主题系统
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub color_scheme: ColorScheme,
}

/// 颜色方案
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Default,
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Default,
        }
    }
}

impl Theme {
    /// 从名称创建主题
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dark" => Self {
                color_scheme: ColorScheme::Dark,
            },
            "light" => Self {
                color_scheme: ColorScheme::Light,
            },
            _ => Self::default(),
        }
    }

    /// 获取默认样式
    pub fn default_style(&self) -> Style {
        match self.color_scheme {
            ColorScheme::Default => Style::default().fg(Color::White),
            ColorScheme::Dark => Style::default().fg(Color::Gray),
            ColorScheme::Light => Style::default().fg(Color::Black),
        }
    }

    /// 获取主色调
    pub fn primary(&self) -> Color {
        match self.color_scheme {
            ColorScheme::Default => Color::Cyan,
            ColorScheme::Dark => Color::Blue,
            ColorScheme::Light => Color::DarkGray,
        }
    }

    /// 获取强调色
    pub fn accent(&self) -> Color {
        match self.color_scheme {
            ColorScheme::Default => Color::Yellow,
            ColorScheme::Dark => Color::Green,
            ColorScheme::Light => Color::LightBlue,
        }
    }

    /// 获取警告色
    pub fn warning(&self) -> Color {
        match self.color_scheme {
            ColorScheme::Default => Color::Yellow,
            ColorScheme::Dark => Color::Rgb(255, 165, 0), // Orange
            ColorScheme::Light => Color::Rgb(255, 215, 0), // Gold
        }
    }

    /// 获取错误色
    pub fn error(&self) -> Color {
        match self.color_scheme {
            ColorScheme::Default => Color::Red,
            ColorScheme::Dark => Color::LightRed,
            ColorScheme::Light => Color::Rgb(139, 0, 0), // DarkRed
        }
    }

    /// 获取成功色
    pub fn success(&self) -> Color {
        match self.color_scheme {
            ColorScheme::Default => Color::Green,
            ColorScheme::Dark => Color::LightGreen,
            ColorScheme::Light => Color::Rgb(0, 100, 0),
        }
    }

    /// 获取标签栏样式
    pub fn tab_bar(&self) -> Style {
        Style::default().bg(self.primary()).fg(Color::White)
    }

    /// 获取活动标签样式
    pub fn tab_active(&self) -> Style {
        Style::default().bg(Color::White).fg(self.primary())
    }

    /// 获取状态栏样式
    pub fn status_bar(&self) -> Style {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    }

    /// 获取边框样式
    pub fn border(&self) -> Style {
        Style::default().fg(self.primary())
    }

    /// 获取高亮样式
    pub fn highlight(&self) -> Style {
        match self.color_scheme {
            ColorScheme::Default => Style::default().bg(Color::Rgb(50, 50, 100)),
            ColorScheme::Dark => Style::default().bg(Color::Rgb(30, 30, 60)),
            ColorScheme::Light => Style::default().bg(Color::Rgb(230, 230, 255)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_name() {
        let theme = Theme::from_name("dark");
        assert_eq!(theme.color_scheme, ColorScheme::Dark);

        let theme = Theme::from_name("invalid");
        assert_eq!(theme.color_scheme, ColorScheme::Default);
    }

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.color_scheme, ColorScheme::Default);
        assert_eq!(theme.primary(), Color::Cyan);
    }
}
