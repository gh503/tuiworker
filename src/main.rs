mod app;
mod event;
mod models;
mod storage;
mod terminal;
mod terminal_manager;
mod ui;

use app::App;
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::AppEvent;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};
use storage::Storage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 初始化存储
    let storage = Storage::new()?;

    // 加载数据
    let data = storage.load()?;

    // 创建应用实例并加载数据
    let mut app = App::new().with_data(data);
    app.refresh_file_browser();

    // 设置重绘时间间隔（120FPS）
    let tick_rate = Duration::from_millis(8);
    let last_tick = Instant::now();
    let last_output_update = Instant::now();
    let output_update_rate = Duration::from_millis(50); // 每50ms更新一次终端输出

    // 主循环
    let result = run_app(
        &mut terminal,
        &mut app,
        &storage,
        tick_rate,
        output_update_rate,
        last_tick,
        last_output_update,
    );

    // 保存数据
    if let Err(e) = storage.save(&app.data) {
        eprintln!("保存数据失败: {}", e);
    }

    // 恢复终端状态
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    storage: &Storage,
    tick_rate: Duration,
    output_update_rate: Duration,
    mut last_tick: Instant,
    mut last_output_update: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Update terminal output periodically (only when in Commands tab)
        if app.current_tab == app::Tab::Commands
            && last_output_update.elapsed() >= output_update_rate
        {
            app.update_current_terminal_output();
            last_output_update = Instant::now();
        }

        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // 超时时间 = 下一次 tick 的时间
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // 等待事件
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = read()? {
                if let Some(app_event) = AppEvent::from_crossterm_event(Event::Key(key)) {
                    app.handle_event(app_event);

                    // 如果需要退出，保存数据并返回
                    if app.should_quit {
                        storage.save(&app.data)?;
                        return Ok(());
                    }
                }
            }
        }

        // 更新 tick 时间
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}
