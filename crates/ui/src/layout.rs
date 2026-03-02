use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// 布局系统
#[derive(Debug, Clone)]
pub struct Layout {
    theme: Option<super::Theme>,
}

impl Default for Layout {
    fn default() -> Self {
        Self { theme: None }
    }
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置主题
    pub fn with_theme(mut self, theme: super::Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// 创建垂直分割布局
    pub fn split_vertical(&self, area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
        RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints(constraints.as_ref())
            .split(area)
            .to_vec()
    }

    /// 创建水平分割布局
    pub fn split_horizontal(&self, area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
        RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.as_ref())
            .split(area)
            .to_vec()
    }

    /// 创建居中布局
    pub fn centered(&self, area: Rect, width: u16, height: u16) -> Rect {
        let x = area
            .x
            .saturating_add((area.width.saturating_sub(width)) / 2);
        let y = area
            .y
            .saturating_add((area.height.saturating_sub(height)) / 2);
        Rect::new(x, y, width.min(area.width), height.min(area.height))
    }
}

/// 便捷的布局约束创建器
pub struct ConstraintBuilder;

impl ConstraintBuilder {
    /// 固定大小
    pub fn fixed(size: u16) -> Constraint {
        Constraint::Length(size)
    }

    /// 比例大小
    pub fn percentage(percent: u16) -> Constraint {
        Constraint::Percentage(percent)
    }

    /// 最小大小
    pub fn min(size: u16) -> Constraint {
        Constraint::Min(size)
    }

    /// 最大大小
    pub fn max(size: u16) -> Constraint {
        Constraint::Max(size)
    }

    /// 剩余空间均分
    pub fn ratio(numerator: u32, denominator: u32) -> Constraint {
        Constraint::Ratio(numerator, denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_vertical() {
        let layout = Layout::new();
        let area = Rect::new(0, 0, 100, 100);
        let constraints = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let result = layout.split_vertical(area, &constraints);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].height, 50);
        assert_eq!(result[1].height, 50);
    }

    #[test]
    fn test_split_horizontal() {
        let layout = Layout::new();
        let area = Rect::new(0, 0, 100, 100);
        let constraints = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let result = layout.split_horizontal(area, &constraints);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].width, 50);
        assert_eq!(result[1].width, 50);
    }

    #[test]
    fn test_centered() {
        let layout = Layout::new();
        let area = Rect::new(0, 0, 100, 100);
        let result = layout.centered(area, 50, 50);

        assert_eq!(result.x, 25);
        assert_eq!(result.y, 25);
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
    }
}
