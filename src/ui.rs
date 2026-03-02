use chrono::Datelike;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, InputMode, SearchResult, Tab};

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let size = frame.area();

    // 创建主要布局: 标题栏、中间内容、状态栏
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 标题栏
            Constraint::Min(0),    // 主内容区
            Constraint::Length(3), // 状态栏
        ])
        .split(size);

    // 绘制标题栏
    draw_header(frame, chunks[0], app);

    // 绘制中间内容区：侧边栏 + 主内容
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30), // 侧边栏
            Constraint::Min(0),     // 主内容
        ])
        .split(chunks[1]);

    draw_sidebar(frame, content_chunks[0], app);
    draw_main_content(frame, content_chunks[1], app);

    // 绘制状态栏
    draw_status_bar(frame, chunks[2], app);

    // 如果显示模态对话框，绘制在最上层
    if app.show_modal {
        draw_modal_dialog(frame, size, app);
    }

    // 如果正在编辑，绘制编辑器
    if app.input_mode == InputMode::Editing && !app.show_modal {
        draw_input_editor(frame, size, app);
    }
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled(
            "TUI Worker",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled("办公工作入口", Style::default().fg(Color::Gray)),
    ]);

    let right_info = format!(
        "{} | 笔记: {} | 待办: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        app.data.notes.len(),
        app.data
            .todos
            .iter()
            .filter(|t| t.status != crate::models::TodoStatus::Completed)
            .count()
    );

    let title_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(40)])
        .split(area);

    let paragraph = Paragraph::new(vec![title])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);

    let right_paragraph = Paragraph::new(right_info).alignment(Alignment::Right);

    frame.render_widget(paragraph, title_area[0]);
    frame.render_widget(right_paragraph, title_area[1]);
}

fn draw_sidebar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let tabs = Tab::all();

    let items: Vec<ListItem> = tabs
        .iter()
        .map(|tab| {
            let style = if *tab == app.current_tab {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Gray)
            };

            let text = format!("{}. {}", tab.shortcut(), tab.name());
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("导航")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(list, area);
}

fn draw_main_content(frame: &mut Frame<'_>, area: Rect, app: &App) {
    match app.current_tab {
        Tab::Dashboard => draw_dashboard(frame, area, app),
        Tab::Notes => draw_notes(frame, area, app),
        Tab::Todos => draw_todos(frame, area, app),
        Tab::Commands => draw_commands(frame, area, app),
        Tab::Calendar => draw_calendar(frame, area, app),
        Tab::FileBrowser => draw_file_browser(frame, area, app),
        Tab::Search => draw_search(frame, area, app),
        Tab::Settings => draw_settings(frame, area, app),
        Tab::Logs => draw_logs(frame, area, app),
    }
}

fn draw_dashboard(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let content = vec![
        Line::from("欢迎使用 TUI Worker"),
        Line::from(""),
        Line::from("这是一个纯命令行操作的工作入口"),
        Line::from(""),
        Line::from("快捷键:"),
        Line::from("  Tab       - 切换标签页"),
        Line::from("  1-8       - 直接跳转到标签页"),
        Line::from("  ↑↓        - 上下导航"),
        Line::from("  q         - 退出程序"),
        Line::from("  a         - 添加新项目"),
        Line::from("  d         - 删除选中项目"),
        Line::from("  /         - 搜索/过滤"),
        Line::from(""),
        Line::from("统计数据:"),
        Line::from(format!("  笔记总数: {}", app.data.notes.len())),
        Line::from(format!("  待办事项: {}", app.data.todos.len())),
        Line::from(format!(
            "  已完成: {}",
            app.data
                .todos
                .iter()
                .filter(|t| t.status == crate::models::TodoStatus::Completed)
                .count()
        )),
        Line::from(format!("  命令历史: {}", app.data.command_history.len())),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().title("仪表板").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

fn draw_notes(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // 左侧：笔记列表
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(area);

    let notes = app.get_notes();
    let items: Vec<ListItem> = notes
        .iter()
        .enumerate()
        .map(|(i, note)| {
            let style = if i == app.selected_note_index {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let preview = if note.content.len() > 30 {
                format!("{}...", &note.content[..30])
            } else {
                note.content.clone()
            };

            ListItem::new(format!("\n{}\n  {}", note.title, preview)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("笔记列表")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(list, chunks[0]);

    // 右侧：笔记详情
    if let Some(note) = notes.get(app.selected_note_index) {
        let content = vec![
            Line::from(vec![
                Span::styled("标题: ", Style::default().fg(Color::Yellow)),
                Span::styled(&note.title, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("分类: ", Style::default().fg(Color::Yellow)),
                Span::styled(&note.category, Style::default()),
            ]),
            Line::from(vec![
                Span::styled("标签: ", Style::default().fg(Color::Yellow)),
                Span::styled(note.tags.join(", "), Style::default()),
            ]),
            Line::from(vec![
                Span::styled("创建时间: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    note.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default(),
                ),
            ]),
            Line::from(""),
            Line::from("内容:"),
            Line::from(""),
        ];

        let note_lines: Vec<Line> = note.content.lines().map(Line::from).collect();
        let full_content: Vec<Line> = content.into_iter().chain(note_lines).collect();

        let paragraph = Paragraph::new(full_content)
            .block(Block::default().title("笔记详情").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, chunks[1]);
    }
}

fn draw_todos(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let todos = app.get_todos();

    if todos.is_empty() {
        let paragraph = Paragraph::new(
            "暂无待办事项\n\n按 'a' 添加新的待办事项\n'n': 显示全部\n'f': 只显示已完成",
        )
        .block(Block::default().title("待办列表").borders(Borders::ALL))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    }

    let rows: Vec<Row> = todos
        .iter()
        .enumerate()
        .map(|(i, todo)| {
            let _status_style = match todo.status {
                crate::models::TodoStatus::Pending => Style::default().fg(Color::Yellow),
                crate::models::TodoStatus::InProgress => Style::default().fg(Color::Blue),
                crate::models::TodoStatus::Completed => Style::default().fg(Color::Green),
                crate::models::TodoStatus::Cancelled => Style::default().fg(Color::Red),
            };

            let status_text = match todo.status {
                crate::models::TodoStatus::Pending => "待办",
                crate::models::TodoStatus::InProgress => "进行中",
                crate::models::TodoStatus::Completed => "已完成",
                crate::models::TodoStatus::Cancelled => "已取消",
            };

            let row_style = if i == app.selected_todo_index {
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let preview = if todo.description.len() > 20 {
                format!("{}...", &todo.description[..20])
            } else {
                todo.description.clone()
            };

            Row::new(vec![
                todo.title.clone(),
                status_text.to_string(),
                todo.category.clone(),
                preview,
            ])
            .style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Min(0),
        ],
    )
    .header(
        Row::new(vec!["任务", "状态", "分类", "描述"]).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(Block::default().title("待办列表").borders(Borders::ALL));

    frame.render_widget(table, area);
}

fn draw_commands(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // 左侧：终端标签页列表
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(area);

    let tab_items: Vec<ListItem> = app
        .terminal_tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let style = if tab.is_active {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(tab.title.as_str()).style(style)
        })
        .collect();

    let tab_list = List::new(tab_items)
        .block(
            Block::default()
                .title("终端标签")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(tab_list, chunks[0]);

    // 右侧：当前终端输出
    if let Some(tab) = app.get_current_terminal_tab() {
        let output_text: Vec<Line> = tab.command_output_buffer.lines().map(Line::from).collect();

        let paragraph = Paragraph::new(output_text)
            .block(
                Block::default()
                    .title(tab.title.as_str())
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, chunks[1]);
    } else {
        let paragraph = Paragraph::new("没有打开的终端\n\n按 't' 创建新终端标签")
            .block(Block::default().title("终端").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, chunks[1]);
    }
}

fn draw_calendar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let current_date = chrono::Local::now().date_naive();
    let year = current_date.year();
    let month = current_date.month();

    let header = format!("{}年{}月", year, month);

    // 日历表头
    let weekday_names = ["日", "一", "二", "三", "四", "五", "六"];
    let header_row = Row::new(weekday_names.to_vec()).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    // 计算日历天数
    let first_day = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let weekday = first_day.weekday().num_days_from_sunday();
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 31,
    };

    // 构建日历行
    let mut calendar_rows = vec![header_row];
    let mut current_day = 1;

    while current_day <= days_in_month {
        let mut row_data = vec![];
        for i in 0..7 {
            if (current_day == 1 && i < weekday as usize) || current_day > days_in_month {
                row_data.push(Span::raw("  ".to_string()));
            } else {
                let day_str = format!("{:2}", current_day);
                let is_today = current_day == current_date.day();
                let has_events = app.data.calendar_events.iter().any(|e| {
                    e.date.year() == year && e.date.month() == month && e.date.day() == current_day
                });

                let style = if is_today {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else if has_events {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };

                row_data.push(Span::styled(day_str, style));
                current_day += 1;
            }
        }
        calendar_rows.push(Row::new(row_data));
    }

    let table = Table::new(
        calendar_rows,
        [
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ],
    )
    .block(Block::default().title(header.clone()).borders(Borders::ALL));

    frame.render_widget(table, area);
}

fn draw_file_browser(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // 顶部：当前路径
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let path_str = app.file_browser.current_path.display().to_string();
    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled("路径: ", Style::default().fg(Color::Cyan)),
        Span::styled(path_str, Style::default()),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Left);

    frame.render_widget(paragraph, chunks[0]);

    // 底部：文件列表
    let items: Vec<ListItem> = app
        .file_browser
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.file_browser.selected_index {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            let icon = if entry.is_dir { "[DIR]" } else { "[FILE]" };
            let size_str = if entry.is_dir {
                "".to_string()
            } else {
                format!(" ({} bytes)", entry.file_size)
            };

            let text = format!("{} {} {}{}", icon, entry.name, size_str, " ".repeat(10));
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("文件列表")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(list, chunks[1]);
}

fn draw_search(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // 顶部：搜索输入
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let input_prompt = Line::from(vec![
        Span::styled("搜索: ", Style::default().fg(Color::Yellow)),
        Span::styled(app.search.query.as_str(), Style::default()),
    ]);
    let input_paragraph = Paragraph::new(input_prompt)
        .block(Block::default().title("全局搜索").borders(Borders::ALL))
        .alignment(Alignment::Left);

    frame.render_widget(input_paragraph, chunks[0]);

    // 底部：搜索结果
    if app.search.results.is_empty() {
        let paragraph = Paragraph::new("没有找到匹配的结果\n\n输入搜索关键词后按 Enter 搜索")
            .block(Block::default().title("搜索结果").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .search
            .results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == app.search.selected_index {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let (prefix, text) = match result {
                    SearchResult::Note { title, .. } => ("[笔记]", title),
                    SearchResult::Todo { title, .. } => ("[待办]", title),
                };

                ListItem::new(format!("{} {}", prefix, text)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!("搜索结果 ({})", app.search.results.len()))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(list, chunks[1]);
    }
}

fn draw_settings(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let content = vec![
        Line::from(vec![Span::styled(
            "设置",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("配置选项（开发中）"),
        Line::from(""),
        Line::from("• 主题设置"),
        Line::from("• 键盘映射"),
        Line::from("• 窗口布局"),
        Line::from("• 插件管理"),
        Line::from("• 数据导出"),
        Line::from("• 数据导入"),
        Line::from(""),
        Line::from("数据统计:"),
        Line::from(format!("  笔记总数: {}", app.data.notes.len())),
        Line::from(format!("  待办总数: {}", app.data.todos.len())),
        Line::from(format!("  命令历史: {}", app.data.command_history.len())),
        Line::from(format!("  日程事件: {}", app.data.calendar_events.len())),
    ];

    let paragraph = Paragraph::new(content)
        .block(Block::default().title("设置").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}
fn draw_logs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    // Get logs from buffer
    let logs = &app.log_buffer;

    if logs.is_empty() {
        let paragraph = Paragraph::new(
            "暂无日志\n\n日志将自动记录到 tuiworker.log 文件\n\n按 'c' 清空日志缓冲区"
        )
        .block(Block::default().title("日志").borders(Borders::ALL))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    }

    // Show last 100 log lines
    let log_lines: Vec<Line> = logs
        .iter()
        .rev()
        .take(100)
        .rev()
        .map(|line| {
            let (level, msg) = line.split_once("] ").unwrap_or(("INFO", line));
            
            let style = match level {
                "ERROR" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                "WARN" => Style::default().fg(Color::Yellow),
                "DEBUG" => Style::default().fg(Color::Gray),
                _ => Style::default().fg(Color::Cyan),
            };
            
            Line::from(vec![Span::styled(level, style), Span::raw(": "), Span::styled(msg, Style::default())])
        })
        .collect();

    let header = Line::from(vec![
        Span::styled(
            format!("日志 (共 {} 条, 显示最近 100 条) - 按 'c' 清空", logs.len()),
            Style::default().fg(Color::Yellow),
        ),
    ]);
    
    // Combine header and logs
    let mut content = vec![header];
    content.extend(log_lines);

    let content_len = content.len();
    let paragraph = Paragraph::new(content)
        .block(Block::default().title("日志").borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left)
        .scroll((0, content_len as u16 - 2));
    frame.render_widget(paragraph, area);
}



fn draw_status_bar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let mut status_parts = vec![format!("当前标签: {}", app.current_tab.name())];

    match app.current_tab {
        Tab::Notes => {
            status_parts.push(format!("笔记数: {}", app.data.notes.len()));
        }
        Tab::Todos => {
            status_parts.push(format!("待办数: {}", app.data.todos.len()));
            if let Some(filter) = &app.todo_filter_status {
                status_parts.push(format!("过滤: {:?}", filter));
            }
        }
        Tab::Commands => {
            status_parts.push(format!("标签: {}", app.terminal_tabs.len()));
        }
        Tab::Search => {
            status_parts.push(format!("结果: {}", app.search.results.len()));
        }
        _ => {}
    }

    let status = status_parts.join(" | ");
    let status_text = Line::from(vec![
        Span::styled(status, Style::default()),
        Span::raw(" | 按 'q' 退出"),
    ]);

    let paragraph = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

fn draw_modal_dialog(frame: &mut Frame<'_>, size: Rect, app: &App) {
    // 计算居中位置
    let width = size.width.min(50);
    let height = size.height.min(10);
    let x = (size.width - width) / 2;
    let y = (size.height - height) / 2;

    let dialog_area = Rect::new(x, y, width, height);

    let content = if app.modal_waiting_input {
        vec![
            Line::from(app.input_prompt.as_str()),
            Line::from(""),
            Line::from(app.input_buffer.as_str()),
            Line::from(""),
            Line::from(vec![Span::styled(
                "按 Enter 确认，Esc 取消",
                Style::default().fg(Color::Cyan),
            )]),
        ]
    } else {
        vec![
            Line::from(app.modal_message.as_str()),
            Line::from(vec![Span::styled(
                "按 Enter 继续",
                Style::default().fg(Color::Cyan),
            )]),
        ]
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title("提示")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, dialog_area);
}

fn draw_input_editor(frame: &mut Frame<'_>, size: Rect, app: &App) {
    // 底部输入提示栏
    let input_line = Rect::new(0, size.height - 1, size.width, 1);

    let input_prompt = format!("> {}", app.input_buffer);
    let paragraph = Paragraph::new(input_prompt).style(Style::default().fg(Color::Yellow));

    frame.render_widget(paragraph, input_line);
}
