mod cli;

use core::App;
use log::info;
use logging::init::{init_logging, LogConfig};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    init_logging(&LogConfig::default())?;
    info!("TUIWorker starting...");

    let _args = cli::parse_args();

    let mut app = App::new()?;

    let home_dir = std::env::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

    let filebrowser_module = filebrowser::FileBrowserModule::new(home_dir.clone());
    app.register_module(filebrowser_module);
    info!("Registered FileBrowser module");

    let db_dir = home_dir.join(".local/share/tuiworker");
    std::fs::create_dir_all(&db_dir)?;
    let db = storage::Database::open(&db_dir.join("db"))?;
    let todo_db = db.with_namespace("todo");
    let todo_module = todo::Todo::new(todo_db);
    app.register_module(todo_module);
    info!("Registered Todo module");

    app.run()?;

    info!("TUIWorker shut down gracefully");
    Ok(())
}
