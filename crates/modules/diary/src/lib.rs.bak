//! Diary module - Calendar-based journaling with lunar calendar and holiday support

use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use chrono::{Datelike, Days, Local, NaiveDate, Weekday};

use core::{
    event::Action,
    module::{Module as CoreModule, Shortcut},
};

/// Diary entry structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiaryEntry {
    pub date: NaiveDate,
    pub content: String,
}

/// Lunar date information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LunarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub zodiac: String,
    pub lunar_date: String, // Formatted string e.g., "八月十五"
}

/// Result of holiday check
#[derive(Debug, Clone)]
pub enum HolidayType {
    None,
    PublicHoliday(String),      // Chinese public holiday name
    TraditionalHoliday(String), // Traditional festival name
    weekend,
}

/// Main diary module
pub struct DiaryModule {
    entries: HashMap<NaiveDate, DiaryEntry>,
    selected_date: NaiveDate,
    current_month: NaiveDate,
    editing: bool,
    edit_content: String,
    diary_dir: PathBuf,
    theme: ui::Theme,
}

impl DiaryModule {
    /// Create a new diary module
    pub fn new(diary_dir: PathBuf) -> Self {
        let today = Local::now().date_naive();
        Self {
            entries: HashMap::new(),
            selected_date: today,
            current_month: today,
            editing: false,
            edit_content: String::new(),
            diary_dir,
            theme: ui::Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: ui::Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Select a specific date
    pub fn select_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
        self.current_month = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);

        // Load entry if exists
        if self.entries.contains_key(&date) {
            self.edit_content = self.entries[&date].content.clone();
        } else {
            // Try to load from file
            self.load_entry_file(date);
        }
    }

    /// Write diary entry
    pub fn write_entry(&mut self, content: String) -> anyhow::Result<()> {
        let entry = DiaryEntry {
            date: self.selected_date,
            content,
        };

        self.entries.insert(self.selected_date, entry.clone());
        self.save_entry_file(&entry)?;

        Ok(())
    }

    /// Load all diary entries
    pub fn load_entries(&mut self) -> anyhow::Result<()> {
        if !self.diary_dir.exists() {
            fs::create_dir_all(&self.diary_dir)?;
        }

        let entries_iter = fs::read_dir(&self.diary_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"));

        for entry in entries_iter {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(diary_entry) = serde_json::from_str::<DiaryEntry>(&content) {
                    self.entries.insert(diary_entry.date, diary_entry);
                }
            }
        }

        Ok(())
    }

    /// Get lunar date information (simplified implementation)
    pub fn get_lunar_date(&self, date: NaiveDate) -> LunarDate {
        // Simplified lunar calendar calculation
        // In production, use a proper library like chinese-calendar
        let lunar_year = date.year() - 1984 + 1;
        let lunar_month = ((date.month() % 12) + 1) as u8;
        let lunar_day = ((date.day() % 29) + 1) as u8;

        LunarDate {
            year: lunar_year,
            month: lunar_month,
            day: lunar_day,
            zodiac: self.get_zodiac(lunar_year),
            lunar_date: format!("{}月{}", lunar_month, lunar_day),
        }
    }

    /// Check if date is a holiday
    pub fn is_holiday(&self, date: NaiveDate) -> HolidayType {
        let month = date.month();
        let day = date.day();
        let weekday = date.weekday();

        // Check if weekend
        if weekday == Weekday::Sat || weekday == Weekday::Sun {
            return HolidayType::weekend;
        }

        // Fixed date holidays
        match (month, day) {
            (1, 1) => return HolidayType::PublicHoliday("元旦".to_string()),
            (5, 1) => return HolidayType::PublicHoliday("劳动节".to_string()),
            (10, 1) => return HolidayType::PublicHoliday("国庆节".to_string()),
            (12, 25) => return HolidayType::TraditionalHoliday("圣诞节".to_string()),
            // Add more fixed holidays as needed
            _ => {}
        }

        // Approximate traditional holidays (simplified)
        // In production, use proper lunar calendar library
        match (month, day) {
            (1, 1) | (1, 2) | (1, 3) => HolidayType::TraditionalHoliday("春节".to_string()),
            (1, 15) => HolidayType::TraditionalHoliday("元宵节".to_string()),
            (4, 4) | (4, 5) | (4, 6) => HolidayType::TraditionalHoliday("清明节".to_string()),
            (6, 22) => HolidayType::TraditionalHoliday("端午节".to_string()),
            (8, 15) => HolidayType::TraditionalHoliday("中秋节".to_string()),
            _ => HolidayType::None,
        }
    }

    /// Get zodiac animal based on lunar year
    fn get_zodiac(&self, year: i32) -> String {
        let zodiacs = vec![
            "鼠", "牛", "虎", "兔", "龙", "蛇", "马", "羊", "猴", "鸡", "狗", "猪",
        ];
        let index = (year - 4).rem_euclid(12) as usize;
        zodiacs[index].to_string()
    }

    /// Load a single entry from file
    fn load_entry_file(&mut self, date: NaiveDate) {
        let filename = format!("{}{:02}{:02}.json", date.year(), date.month(), date.day());
        let filepath = self.diary_dir.join(filename);

        if let Ok(content) = fs::read_to_string(&filepath) {
            if let Ok(entry) = serde_json::from_str::<DiaryEntry>(&content) {
                self.edit_content = entry.content;
            }
        }
    }

    /// Save entry to file
    fn save_entry_file(&self, entry: &DiaryEntry) -> anyhow::Result<()> {
        if !self.diary_dir.exists() {
            fs::create_dir_all(&self.diary_dir)?;
        }

        let filename = format!(
            "{}{:02}{:02}.json",
            entry.date.year(),
            entry.date.month(),
            entry.date.day()
        );
        let filepath = self.diary_dir.join(filename);

        let json = serde_json::to_string_pretty(entry)?;
        let mut file = File::create(filepath)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Get days in month
    fn days_in_month(&self, year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Build calendar widget
    fn build_calendar(&self, area: Rect) -> Vec<Line> {
        let mut lines = vec![];

        // Month and year header
        let month_names = vec![
            "",
            "一月",
            "二月",
            "三月",
            "四月",
            "五月",
            "六月",
            "七月",
            "八月",
            "九月",
            "十月",
            "十一月",
            "十二月",
        ];
        let header = format!(
            "  {} {}  ",
            self.current_month.year(),
            month_names[self.current_month.month() as usize]
        );
        lines.push(Line::from(vec![Span::styled(
            header,
            Style::default()
                .fg(self.theme.primary())
                .add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::default());

        // Weekday headers
        let weekdays = ["日", "一", "二", "三", "四", "五", "六"];
        let weekday_spans: Vec<Span> = weekdays
            .iter()
            .map(|w| {
                Span::styled(
                    format!(" {:<2}", w),
                    Style::default()
                        .fg(self.theme.muted())
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        lines.push(Line::from(weekday_spans));
        lines.push(Line::default());

        // Calendar grid
        let first_day =
            NaiveDate::from_ymd_opt(self.current_month.year(), self.current_month.month(), 1)
                .unwrap();
        let start_weekday = first_day.weekday().num_days_from_sunday() as u32;
        let days_in_month =
            self.days_in_month(self.current_month.year(), self.current_month.month());

        let mut current_day = 1;
        for _week in 0..6 {
            let mut day_cells = Vec::new();

            for weekday in 0..7 {
                if weekday < start_weekday || current_day > days_in_month {
                    day_cells.push(Span::styled("   ", Style::default()));
                } else {
                    let date = NaiveDate::from_ymd_opt(
                        self.current_month.year(),
                        self.current_month.month(),
                        current_day,
                    )
                    .unwrap();

                    let is_selected = date == self.selected_date;
                    let is_today = date == Local::now().date_naive();
                    let has_entry = self.entries.contains_key(&date);
                    let is_holiday = !matches!(self.is_holiday(date), HolidayType::None);

                    let mut style = Style::default().fg(self.theme.text());

                    if is_selected {
                        style = style.bg(self.theme.primary()).fg(Color::Black);
                    } else if is_today {
                        style = style.fg(self.theme.primary()).add_modifier(Modifier::BOLD);
                    } else if is_holiday {
                        style = style.fg(Color::Red);
                    } else {
                        style = style.fg(self.theme.text());
                    }

                    let marker = if has_entry {
                        "*"
                    } else if is_today {
                        "•"
                    } else {
                        " "
                    };

                    day_cells.push(Span::styled(
                        format!("{: >2}{}", current_day, marker),
                        style,
                    ));

                    current_day += 1;
                }
            }

            lines.push(Line::from(day_cells));

            if current_day > days_in_month {
                break;
            }
        }

        lines
    }

    /// Draw entry content
    fn draw_entry(&self, frame: &mut Frame, area: Rect) {
        let has_entry = self.entries.contains_key(&self.selected_date);

        if self.editing {
            // Edit mode
            let edit_content = if self.edit_content.is_empty() {
                "正在编辑... (ESC 退出, Ctrl+S 保存)"
            } else {
                &self.edit_content
            };

            let paragraph = Paragraph::new(edit_content)
                .block(
                    Block::default()
                        .title("编辑日记")
                        .title_style(
                            Style::default()
                                .fg(self.theme.primary())
                                .add_modifier(Modifier::BOLD),
                        )
                        .borders(Borders::ALL)
                        .border_style(self.theme.border()),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(paragraph, area);
        } else if has_entry {
            // View mode
            let entry = &self.entries[&self.selected_date];
            let lines: Vec<Line> = entry
                .content
                .lines()
                .map(|l| Line::from(l.to_string()))
                .collect();

            let paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title("日记内容")
                        .title_style(
                            Style::default()
                                .fg(self.theme.primary())
                                .add_modifier(Modifier::BOLD),
                        )
                        .borders(Borders::ALL)
                        .border_style(self.theme.border()),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(paragraph, area);
        } else {
            // No entry
            let paragraph = Paragraph::new("按 Enter 开始写日记")
                .block(
                    Block::default()
                        .title("日记内容")
                        .title_style(
                            Style::default()
                                .fg(self.theme.primary())
                                .add_modifier(Modifier::BOLD),
                        )
                        .borders(Borders::ALL)
                        .border_style(self.theme.border()),
                )
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
        }
    }

    /// Draw lunar date info
    fn draw_lunar_info(&self, frame: &mut Frame, area: Rect) {
        let lunar = self.get_lunar_date(self.selected_date);
        let holiday = self.is_holiday(self.selected_date);

        let lines = vec![
            Line::from(vec![
                Span::styled("农历: ", Style::default().fg(self.theme.muted())),
                Span::styled(
                    format!("{}年{}月{}日", lunar.year, lunar.month, lunar.day),
                    Style::default().fg(self.theme.primary()),
                ),
            ]),
            Line::from(vec![
                Span::styled("生肖: ", Style::default().fg(self.theme.muted())),
                Span::styled(lunar.zodiac, Style::default().fg(self.theme.primary())),
            ]),
            Line::from(vec![
                Span::styled("农历: ", Style::default().fg(self.theme.muted())),
                Span::styled(lunar.lunar_date, Style::default().fg(self.theme.primary())),
            ]),
            Line::default(),
            match holiday {
                HolidayType::PublicHoliday(name) => Line::from(vec![
                    Span::styled("节日: ", Style::default().fg(self.theme.muted())),
                    Span::styled(
                        format!("🎉 {}", name),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]),
                HolidayType::TraditionalHoliday(name) => Line::from(vec![
                    Span::styled("节日: ", Style::default().fg(self.theme.muted())),
                    Span::styled(format!("🎋 {}", name), Style::default().fg(Color::Yellow)),
                ]),
                HolidayType::weekend => Line::from(vec![Span::styled(
                    "休息日",
                    Style::default().fg(Color::Green),
                )]),
                HolidayType::None => Line::from(vec![Span::styled("")]),
            },
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("农历信息")
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('h') => {
                // Previous month
                if let Some(new_month) = self.current_month.checked_sub_days(Days::new(1)) {
                    self.current_month =
                        NaiveDate::from_ymd_opt(new_month.year(), new_month.month(), 1).unwrap();
                }
                Action::None
            }
            KeyCode::Char('l') => {
                // Next month
                if let Some(new_month) = self.current_month.checked_add_days(Days::new(32)) {
                    self.current_month =
                        NaiveDate::from_ymd_opt(new_month.year(), new_month.month(), 1).unwrap();
                }
                Action::None
            }
            KeyCode::Char('j') => {
                // Previous week
                if let Some(new_date) = self.selected_date.checked_sub_days(Days::new(7)) {
                    self.select_date(new_date);
                }
                Action::None
            }
            KeyCode::Char('k') => {
                // Next week
                if let Some(new_date) = self.selected_date.checked_add_days(Days::new(7)) {
                    self.select_date(new_date);
                }
                Action::None
            }
            KeyCode::Char('J') => {
                // Previous day
                if let Some(new_date) = self.selected_date.checked_sub_days(Days::new(1)) {
                    self.select_date(new_date);
                }
                Action::None
            }
            KeyCode::Char('K') => {
                // Next day
                if let Some(new_date) = self.selected_date.checked_add_days(Days::new(1)) {
                    self.select_date(new_date);
                }
                Action::None
            }
            KeyCode::Enter => {
                // Toggle edit mode
                if !self.editing {
                    self.editing = true;
                    if self.edit_content.is_empty()
                        && self.entries.contains_key(&self.selected_date)
                    {
                        self.edit_content = self.entries[&self.selected_date].content.clone();
                    }
                } else {
                    self.editing = false;
                }
                Action::None
            }
            KeyCode::Esc => {
                // Exit edit mode
                if self.editing {
                    self.editing = false;
                } else {
                    return Action::Quit;
                }
                Action::None
            }
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                // Save entry
                if self.editing {
                    if let Err(e) = self.write_entry(self.edit_content.clone()) {
                        log::error!("Failed to save diary entry: {}", e);
                    } else {
                        self.editing = false;
                    }
                }
                Action::None
            }
            _ => Action::None,
        }
    }
}

impl CoreModule for DiaryModule {
    fn name(&self) -> &str {
        "diary"
    }

    fn title(&self) -> &str {
        "日记"
    }

    fn update(&mut self, event: Event) -> Action {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            _ => Action::None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Month header
                Constraint::Min(10),   // Calendar
                Constraint::Min(5),    // Entry
                Constraint::Length(8), // Lunar info
            ])
            .split(area);

        // Draw calendar
        let calendar_lines = self.build_calendar(layout[1]);
        let calendar = Paragraph::new(Text::from(calendar_lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.theme.border()),
            )
            .alignment(Alignment::Left);
        frame.render_widget(calendar, layout[1]);

        // Draw entry
        self.draw_entry(frame, layout[2]);

        // Draw lunar info
        self.draw_lunar_info(frame, layout[3]);

        // Draw help bar
        let help = Paragraph::new(
            "h:上月 l:下月 J:上日 K:下日 j:上周 k:下周 Enter:编辑 Ctrl+S:保存 Esc:退出/取消编辑",
        )
        .style(Style::default().fg(self.theme.muted()));
        frame.render_widget(help, layout[0]);
    }

    fn save(&self) -> anyhow::Result<()> {
        for entry in self.entries.values() {
            self.save_entry_file(entry)?;
        }
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        self.load_entries()
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut {
                key: "h",
                description: "上月",
            },
            Shortcut {
                key: "l",
                description: "下月",
            },
            Shortcut {
                key: "J",
                description: "上一天",
            },
            Shortcut {
                key: "K",
                description: "下一天",
            },
            Shortcut {
                key: "j",
                description: "上一周",
            },
            Shortcut {
                key: "k",
                description: "下一周",
            },
            Shortcut {
                key: "Enter",
                description: "编辑",
            },
            Shortcut {
                key: "Ctrl+S",
                description: "保存",
            },
        ]
    }

    fn init(&mut self) -> anyhow::Result<()> {
        self.load_entries()?;
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.save()?;
        Ok(())
    }
}

pub use DiaryModule as Diary;
