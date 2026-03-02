use crate::logger::{log_debug, log_error, log_info, log_warn};
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static TERMINAL_ID: AtomicUsize = AtomicUsize::new(0);

// ANSI escape sequence patterns
const ANSI_ESCAPE: u8 = 0x1B;
const CSI_START: u8 = b'[';
const OSC_START: u8 = b']';

pub struct PtySession {
    pub id: usize,
    pub shell: String,
    output: Arc<Mutex<Vec<u8>>>,
    is_running: bool,
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    read_handle: Option<thread::JoinHandle<()>>,
}

/// Strip ANSI escape sequences from byte buffer
fn strip_ansi_sequences(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if input[i] == ANSI_ESCAPE {
            // Start of ANSI escape sequence
            i += 1;
            if i >= input.len() {
                break;
            }

            if input[i] == CSI_START {
                // CSI sequence (ESC [ ... )
                i += 1;
                while i < input.len() {
                    let byte = input[i];
                    i += 1;
                    // CSI ends with @-~ (bytes 0x40-0x7E)
                    if byte >= 0x40 && byte <= 0x7E {
                        break;
                    }
                }
            } else if input[i] == OSC_START {
                // OSC sequence (ESC ] ... ST)
                i += 1;
                while i < input.len() {
                    let byte = input[i];
                    i += 1;
                    // OSC ends with BEL (0x07) or ST (ESC \)
                    if byte == 0x07 || (byte == 0x1B && i < input.len() && input[i] == b'\\') {
                        if byte == 0x1B {
                            i += 1;
                        }
                        break;
                    }
                }
            } else {
                // Other escape sequence, skip up to 2 more bytes
                i = (i + 2).min(input.len());
            }
        } else {
            // Normal character
            result.push(input[i]);
            i += 1;
        }
    }

    result
}

impl PtySession {
    pub fn new(shell: String) -> Self {
        let id = TERMINAL_ID.fetch_add(1, Ordering::Relaxed);
        log_debug(&format!(
            "Creating new PtySession with id: {}, shell: {}",
            id, shell
        ));

        Self {
            id,
            shell: shell.clone(),
            output: Arc::new(Mutex::new(Vec::new())),
            is_running: false,
            master: None,
            writer: None,
            read_handle: None,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running {
            log_warn(&format!("PtySession {} already running", self.id));
            return Ok(());
        }

        log_info(&format!(
            "Starting PTY session {} with shell: {}",
            self.id, self.shell
        ));

        let pty_system = NativePtySystem::default();
        let size = crossterm::terminal::size().unwrap_or((24, 80));
        log_debug(&format!("Terminal size: {}x{}", size.1, size.0));

        let pair = pty_system
            .openpty(PtySize {
                rows: size.0 as u16,
                cols: size.1 as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                log_error(&format!("Failed to create PTY: {}", e));
                format!("Failed to create PTY: {}", e)
            })?;

        log_debug("PTY created successfully");

        let cmd = if cfg!(target_os = "windows") {
            let mut c = CommandBuilder::new(&self.shell);
            if self.shell.ends_with("cmd.exe") || self.shell == "cmd" {
                c.args(["/K"]);
                log_debug("Using CMD with /K flag");
            } else if self.shell.ends_with("powershell.exe") || self.shell == "powershell" {
                // PowerShell 参数：禁用 LOGO，禁用配置文件
                c.args(["-NoLogo", "-NoProfile"]);
                log_debug("Using PowerShell with -NoLogo -NoProfile flags");
            } else {
                log_debug(&format!(
                    "Using shell without special flags: {}",
                    self.shell
                ));
            }
            c
        } else {
            log_debug("Using default shell (Unix)");
            CommandBuilder::new(&self.shell)
        };

        let _child = pair.slave.spawn_command(cmd).map_err(|e| {
            log_error(&format!("Failed to spawn shell: {}", e));
            format!("Failed to spawn shell: {}", e)
        })?;

        log_info(&format!(
            "Shell spawned successfully for PTY session {}",
            self.id
        ));

        let master = pair.master;
        let writer = master.take_writer().map_err(|e| {
            log_error(&format!("Failed to get writer: {}", e));
            format!("Failed to get writer: {}", e)
        })?;

        log_debug("Writer obtained successfully");

        let output = self.output.clone();
        let session_id = self.id;

        let mut reader = master.try_clone_reader().map_err(|e| {
            log_error(&format!("Failed to get reader: {}", e));
            format!("Failed to get reader: {}", e)
        })?;

        log_debug("Reader obtained successfully, starting read thread");

        let read_handle = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            let mut total_bytes = 0usize;
            let mut read_count = 0usize;

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        log_debug(&format!(
                            "PTY session {} reader closed after {} bytes in {} reads",
                            session_id, total_bytes, read_count
                        ));
                        break;
                    }
                    Ok(n) => {
                        total_bytes += n;
                        read_count += 1;

                        if read_count % 10 == 0 {
                            log_debug(&format!(
                                "PTY session {}: Read {} bytes in {} iterations (total: {})",
                                session_id, n, read_count, total_bytes
                            ));
                        }

                        // Strip ANSI sequences
                        let clean_bytes = strip_ansi_sequences(&buffer[..n]);

                        if !clean_bytes.is_empty() {
                            if let Ok(mut out) = output.lock() {
                                out.extend_from_slice(&clean_bytes);

                                // 限制缓冲区大小
                                if out.len() > 500000 {
                                    let drain_amount = out.len() - 200000;
                                    log_debug(&format!(
                                        "PTY session {} output buffer trimmed by {} bytes",
                                        session_id, drain_amount
                                    ));
                                    out.drain(..drain_amount);
                                }
                            }
                        } else {
                            log_debug(&format!(
                                "PTY session {}: ANSI sequence stripped, {} bytes -> 0 bytes",
                                session_id, n
                            ));
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {
                        continue;
                    }
                    Err(e) => {
                        log_error(&format!("PTY session {} read error: {}", session_id, e));
                        break;
                    }
                }
            }

            log_info(&format!(
                "PTY session {} read thread terminated",
                session_id
            ));
        });

        self.master = Some(master);
        self.writer = Some(writer);
        self.read_handle = Some(read_handle);
        self.is_running = true;

        // 给 shell 一些时间启动
        log_debug("Waiting for shell to initialize (100ms)...");
        std::thread::sleep(Duration::from_millis(100));

        log_info(&format!("PTY session {} started successfully", self.id));
        Ok(())
    }

    /// 发送字符到 PTY，并立即回显到本地输出
    pub fn send_char(&mut self, c: char) -> Result<(), String> {
        if !self.is_running {
            log_warn(&format!(
                "PTY session {} not running, ignoring send_char: {:?}",
                self.id, c
            ));
            return Err("PTY session not running".to_string());
        }

        log_debug(&format!(
            "PTY session {} sending char: {:?} (U+{:04X})",
            self.id, c, c as u32
        ));

        // 立即回显到本地输出
        if let Ok(mut out) = self.output.lock() {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            let bytes = encoded.as_bytes();
            out.extend_from_slice(bytes);
            log_debug(&format!(
                "Locally echoed {} bytes: {:?}",
                bytes.len(),
                String::from_utf8_lossy(bytes)
            ));
        }

        // 发送到 PTY
        if let Some(ref mut writer) = self.writer {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            let bytes = encoded.as_bytes();

            writer.write_all(bytes).map_err(|e| {
                log_error(&format!("PTY session {} write error: {}", self.id, e));
                format!("Write error: {}", e)
            })?;
            writer.flush().map_err(|e| {
                log_error(&format!("PTY session {} flush error: {}", self.id, e));
                format!("Flush error: {}", e)
            })?;

            log_debug(&format!("Sent {} bytes to PTY", bytes.len()));
        }
        Ok(())
    }

    /// 发送 Enter 到 PTY，并添加换行符到本地输出
    pub fn send_enter(&mut self) -> Result<(), String> {
        if !self.is_running {
            log_warn(&format!(
                "PTY session {} not running, ignoring send_enter",
                self.id
            ));
            return Err("PTY session not running".to_string());
        }

        log_debug(&format!("PTY session {} sending Enter (\\n)", self.id));

        // 立即回显换行到本地输出
        if let Ok(mut out) = self.output.lock() {
            out.push(b'\n');
        }

        // 发送到 PTY
        if let Some(ref mut writer) = self.writer {
            writer.write_all(b"\n").map_err(|e| {
                log_error(&format!("PTY session {} write error: {}", self.id, e));
                format!("Write error: {}", e)
            })?;
            writer.flush().map_err(|e| {
                log_error(&format!("PTY session {} flush error: {}", self.id, e));
                format!("Flush error: {}", e)
            })?;
        }
        Ok(())
    }

    /// 发送 Backspace 到 PTY，并从本地输出删除最后一个字符
    pub fn send_backspace(&mut self) -> Result<(), String> {
        if !self.is_running {
            log_warn(&format!(
                "PTY session {} not running, ignoring send_backspace",
                self.id
            ));
            return Err("PTY session not running".to_string());
        }

        log_debug(&format!("PTY session {} sending Backspace", self.id));

        // 从本地输出删除最后一个字符（如果有的话）
        if let Ok(mut out) = self.output.lock() {
            let original_len = out.len();

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
                log_debug(&format!(
                    "Locally deleted {} bytes from output",
                    original_len - index
                ));
            } else {
                log_debug("Nothing to delete from output buffer");
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

            writer.write_all(&[bs_byte]).map_err(|e| {
                log_error(&format!("PTY session {} write error: {}", self.id, e));
                format!("Write error: {}", e)
            })?;
            writer.flush().map_err(|e| {
                log_error(&format!("PTY session {} flush error: {}", self.id, e));
                format!("Flush error: {}", e)
            })?;

            log_debug(&format!("Sent Backspace byte: 0x{:02X}", bs_byte));
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        log_info(&format!("Stopping PTY session {}", self.id));
        if let Some(handle) = self.read_handle.take() {
            let _ = handle.join();
        }
        self.is_running = false;
    }

    /// 获取输出为 UTF-8 字符串
    pub fn get_output(&self) -> String {
        if let Ok(out) = self.output.lock() {
            let output_len = out.len();
            let result = String::from_utf8_lossy(&out).to_string();
            log_debug(&format!(
                "PTY session {} get_output: {} bytes -> {} chars",
                self.id,
                output_len,
                result.len()
            ));
            result
        } else {
            log_warn(&format!(
                "PTY session {} failed to lock output buffer",
                self.id
            ));
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
