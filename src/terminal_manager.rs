use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

static TERMINAL_ID: AtomicUsize = AtomicUsize::new(0);

pub struct PtySession {
    pub id: usize,
    pub shell: String,
    pub output: Arc<Mutex<String>>,
    pub is_running: bool,
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    read_handle: Option<thread::JoinHandle<()>>,
}

impl PtySession {
    pub fn new(shell: String) -> Self {
        let id = TERMINAL_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            shell,
            output: Arc::new(Mutex::new(String::new())),
            is_running: false,
            master: None,
            writer: None,
            read_handle: None,
        }
    }

    /// 显示欢迎消息（不启动 PTY）
    pub fn show_welcome(&self) {
        let welcome = format!(
            "=== TUI Worker Terminal ===\nShell: {}\n按 Enter 进入交互式终端\n\n",
            self.shell
        );
        if let Ok(mut out) = self.output.lock() {
            out.clear();
            out.push_str(&welcome);
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            return Ok(());
        }

        let pty_system = NativePtySystem::default();
        let size = crossterm::terminal::size().unwrap_or((24, 80));

        let pair = pty_system
            .openpty(PtySize {
                rows: size.0 as u16,
                cols: size.1 as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to create PTY: {}", e))?;

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = CommandBuilder::new(&self.shell);
            if self.shell.ends_with("cmd.exe") || self.shell == "cmd" {
                c.args(["/K"]);
            } else if self.shell.ends_with("powershell.exe") || self.shell == "powershell" {
                c.args(["-NoLogo", "-NoProfile"]);
            }
            c
        } else {
            CommandBuilder::new(&self.shell)
        };

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let mut master = pair.master;
        let writer = master
            .take_writer()
            .map_err(|e| format!("Failed to get writer: {}", e))?;

        let output = self.output.clone();

        let mut reader = master.try_clone_reader().map_err(|e| format!("Failed to get reader: {}", e))?;
        let read_handle = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(text) = String::from_utf8(buffer[..n].to_vec()) {
                            if let Ok(mut out) = output.lock() {
                                out.push_str(&text);
                                if out.len() > 200000 {
                                    let start = out.len() - 100000;
                                    out.drain(..start);
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        self.master = Some(master);
        self.writer = Some(writer);
        self.read_handle = Some(read_handle);
        self.is_running = true;

        Ok(())
    }

    pub fn send_char(&mut self, c: char) -> Result<(), String> {
        if !self.is_running {
            return Err("PTY session not running".to_string());
        }
        if let Some(ref mut writer) = self.writer {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            writer
                .write_all(encoded.as_bytes())
                .map_err(|e| format!("Write error: {}", e))?;
            writer.flush().map_err(|e| format!("Flush error: {}", e))?;
        }
        Ok(())
    }

    pub fn send_enter(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Err("PTY session not running".to_string());
        }
        if let Some(ref mut writer) = self.writer {
            writer
                .write_all(b"\n")
                .map_err(|e| format!("Write error: {}", e))?;
            writer.flush().map_err(|e| format!("Flush error: {}", e))?;
        }
        Ok(())
    }

    pub fn send_backspace(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Err("PTY session not running".to_string());
        }
        if let Some(ref mut writer) = self.writer {
            writer
                .write_all(b"\x7F")
                .map_err(|e| format!("Write error: {}", e))?;
            writer.flush().map_err(|e| format!("Flush error: {}", e))?;
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.read_handle.take() {
            let _ = handle.join();
        }
        self.is_running = false;
    }

    pub fn get_output(&self) -> String {
        if let Ok(output) = self.output.lock() {
            output.clone()
        } else {
            String::new()
        }
    }
}

impl Debug for PtySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtySession")
            .field("id", &self.id)
            .field("shell", &self.shell)
            .field("is_running", &self.is_running)
            .finish()
    }
}
