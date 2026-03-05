//! E2E Tests for TUIWorker FileBrowser Module

use core::event::Action;
use core::module::Module;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use filebrowser::FileBrowserModule;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_filebrowser_creation() {
    let temp_dir = TempDir::new().unwrap();
    let module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    assert_eq!(module.name(), "filebrowser");
    assert_eq!(module.title(), "File Browser");
}

#[test]
fn test_filebrowser_init() {
    let temp_dir = TempDir::new().unwrap();
    create_test_file(&temp_dir.path(), "test.txt", "Hello");

    let mut module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    module.init().unwrap();

    let status = module.get_status();
    assert!(!status.is_empty());
}

#[test]
fn test_keyboard_navigation() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir.path(), "aaa.txt", "A");
    create_test_file(&temp_dir.path(), "bbb.txt", "B");
    create_test_file(&temp_dir.path(), "ccc.txt", "C");

    let mut module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    module.init().unwrap();

    // Press j to move down
    let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let status = module.get_status();
    assert!(!status.is_empty());
}

#[test]
fn test_file_open_and_close() {
    let temp_dir = TempDir::new().unwrap();
    create_test_file(&temp_dir.path(), "test.txt", "Content");

    let mut module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    module.init().unwrap();

    // Open file
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    // Close with Escape
    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let action = module.update(Event::Key(key));

    assert!(matches!(action, Action::Consumed | Action::None));
}

#[test]
fn test_tab_switch() {
    let temp_dir = TempDir::new().unwrap();
    create_test_file(&temp_dir.path(), "test.txt", "Content");

    let mut module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    module.init().unwrap();

    // Open file first
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    // Press Tab to switch focus
    let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    let action = module.update(Event::Key(key));

    assert!(matches!(action, Action::Consumed | Action::None));
}

#[test]
fn test_scrolling_in_file() {
    let temp_dir = TempDir::new().unwrap();
    let mut content = String::new();
    for i in 1..=100 {
        content.push_str(&format!("This is line {}\n", i));
    }
    create_test_file(&temp_dir.path(), "long.txt", &content);

    let mut module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    module.init().unwrap();

    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let status_before = module.get_status();

    for _ in 0..10 {
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let _ = module.update(Event::Key(key));
    }

    let status_after = module.get_status();
    assert_ne!(status_before, status_after);
    assert!(status_after.contains("Line"));
}

#[test]
fn test_multiple_file_navigation() {
    let temp_dir = TempDir::new().unwrap();

    create_test_file(&temp_dir.path(), "file1.txt", "File 1 content");
    create_test_file(&temp_dir.path(), "file2.txt", "File 2 content");
    create_test_file(&temp_dir.path(), "file3.txt", "File 3 content");

    let mut module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    module.init().unwrap();

    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let _ = module.update(Event::Key(key));

    let status = module.get_status();
    assert!(!status.is_empty());
}

#[test]
fn test_mouse_events() {
    let temp_dir = TempDir::new().unwrap();
    create_test_file(&temp_dir.path(), "test.txt", "Content");

    let mut module = FileBrowserModule::new(temp_dir.path().to_path_buf());
    module.init().unwrap();

    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 10,
        modifiers: KeyModifiers::NONE,
    };

    let action = module.update(Event::Mouse(mouse_event));
    assert!(matches!(action, Action::Consumed | Action::None));
}
