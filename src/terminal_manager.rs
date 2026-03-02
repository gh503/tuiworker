use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static TERMINAL_ID: AtomicUsize = AtomicUsize::new(0);

pub struct PtySession {
    pub id: usize,
    pub shell: String,
    output: Arc<Mutex<Vec<u8>>>,
    is_running: bool,
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
            output: Arc::new(Mutex::new(Vec::new())),
            is_running: false,
            master: None,
            writer: None,
            read_handle: None,
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
                // PowerShell 参数：禁用 LOGO，禁用配置文件，设置输出为行缓冲
                c.args(["-NoLogo", "-NoProfile", "-Command", "$OutputEncoding = [Console]::OutputEncoding = [Console]::InputEncoding = [System.Text.Encoding]::UTF8"]);
            }
            c
        } else {
            CommandBuilder::new(&self.shell)
        };

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let master = pair.master;
        let writer = master
            .take_writer()
            .map_err(|e| format!("Failed to get writer: {}", e))?;

        let output = self.output.clone();

        let mut reader = master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get reader: {}", e))?;
        let read_handle = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // Reader closed
                        break;
                    }
                    Ok(n) => {
                        if let Ok(mut out) = output.lock() {
                            out.extend_from_slice(&buffer[..n]);
                            // 限制缓冲区大小
                            if out.len() > 500000 {
                                let drain_amount = out.len() - 200000;
                                out.drain(..drain_amount);
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                        // Interrupted is not a fatal error, retry
                        continue;
                    }
                    Err(_) => {
                        // Other errors mean the reader is dead
                        break;
                    }
                }
            }
        });

        self.master = Some(master);
        self.writer = Some(writer);
        self.read_handle = Some(read_handle);
        self.is_running = true;

        // 给 shell 一些时间启动
        std::thread::sleep(Duration::from_millis(100));

        Ok(())
    }

    /// 发送字符到 PTY，并立即回显到本地输出
    pub fn send_char(&mut self, c: char) -> Result<(), String> {
        if !self.is_running {
            return Err("PTY session not running".to_string());
        }

        // 立即回显到本地输出
        if let Ok(mut out) = self.output.lock() {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            out.extend_from_slice(encoded.as_bytes());
        }

        // 发送到 PTY
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

    /// 发送 Enter 到 PTY，并添加换行符到本地输出
    pub fn send_enter(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Err("PTY session not running".to_string());
        }

        // 立即回显换行到本地输出
        if let Ok(mut out) = self.output.lock() {
            out.push(b'\n');
        }

        // 发送到 PTY
        if let Some(ref mut writer) = self.writer {
            writer
                .write_all(b"\n")
                .map_err(|e| format!("Write error: {}", e))?;
            writer.flush().map_err(|e| format!("Flush error: {}", e))?;
        }
        Ok(())
    }

    /// 发送 Backspace 到 PTY，并从本地输出删除最后一个字符
    pub fn send_backspace(&mut self) -> Result<(), String> {
        if !self.is_running {
            return Err("PTY session not running".to_string());
        }

        // 从本地输出删除最后一个字符（如果有的话）
        if let Ok(mut out) = self.output.lock() {
            // 删除最后一个 UTF-8 字符
            if let Some((index, _)) = out.len().checked_sub(1).and_then(|i| {
                if out.is_empty() {
                    return None;
                }
                // 从最后一个字符向后搜索有效的 UTF-8 序列的开始
                let mut pos = i;
                while pos > 0 {
                    let byte = out[pos];
                    // UTF-8 字符的首字节：0xxxxxxx, 110xxxxx, 1110xxxx, 11110xxx
                    if (byte & 0b11000000) != 0b10000000 {
                        break;
                    }
                    pos -= 1;
                }
                Some((pos, i))
            }) {
                out.drain(index..);
            }
        }

        // 发送到 PTY (发送 Backspace 控制字符)
        if let Some(ref mut writer) = self.writer {
            // 在 Unix 系统上使用 DEL (0x7F)，在 Windows 上可能需要其他方式
            let bs_byte = if cfg!(target_os = "windows") {
                b'\x08' // Backspace
            } else {
                b'\x7f' // DEL
            };
            writer
                .write_all(&[bs_byte])
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

    /// 获取输出为 UTF-8 字符串
    pub fn get_output(&self) -> String {
        if let Ok(out) = self.output.lock() {
            // 只取有效的 UTF-8 部分
            String::from_utf8_lossy(&out).to_string()
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
