use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// 终端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalType {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "windows-terminal")]
    WindowsTerminal,
    #[serde(rename = "powershell")]
    PowerShell,
    #[serde(rename = "cmd")]
    Cmd,
    #[serde(rename = "iterm2")]
    Iterm2,
    #[serde(rename = "terminal-app")]
    TerminalApp,
    #[serde(rename = "gnome-terminal")]
    GnomeTerminal,
    #[serde(rename = "konsole")]
    Konsole,
    #[serde(rename = "alacritty")]
    Alacritty,
    #[serde(rename = "kitty")]
    Kitty,
    #[serde(rename = "custom")]
    Custom,
}

impl TerminalType {
    pub fn name(&self) -> &'static str {
        match self {
            TerminalType::Auto => "自动检测",
            TerminalType::WindowsTerminal => "Windows Terminal",
            TerminalType::PowerShell => "PowerShell",
            TerminalType::Cmd => "CMD",
            TerminalType::Iterm2 => "iTerm2",
            TerminalType::TerminalApp => "Terminal.app",
            TerminalType::GnomeTerminal => "GNOME Terminal",
            TerminalType::Konsole => "Konsole",
            TerminalType::Alacritty => "Alacritty",
            TerminalType::Kitty => "Kitty",
            TerminalType::Custom => "自定义",
        }
    }

    pub fn get_name(&self) -> String {
        self.name().to_string()
    }

    pub fn get_shell(&self) -> String {
        match self {
            TerminalType::PowerShell => "powershell.exe".to_string(),
            TerminalType::Cmd => "cmd.exe".to_string(),
            TerminalType::WindowsTerminal => "powershell.exe".to_string(),
            TerminalType::Auto => {
                #[cfg(target_os = "windows")]
                return "powershell.exe".to_string();
                #[cfg(target_os = "macos")]
                return "sh".to_string();
                #[cfg(target_os = "linux")]
                return "bash".to_string();
                #[cfg(not(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "linux"
                )))]
                return "sh".to_string();
            }
            _ => "sh".to_string(),
        }
    }
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            // Windows: 检测可用的终端
            if Self::check_command("wt.exe") {
                return TerminalType::WindowsTerminal;
            }
            if Self::check_command("powershell.exe") {
                return TerminalType::PowerShell;
            }
            return TerminalType::Cmd;
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: 优先使用 iTerm2
            if Self::check_app("iTerm.app") {
                return TerminalType::Iterm2;
            }
            return TerminalType::TerminalApp;
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: 检测可用的终端
            if Self::check_command("alacritty") {
                return TerminalType::Alacritty;
            }
            if Self::check_command("kitty") {
                return TerminalType::Kitty;
            }
            if Self::check_command("konsole") {
                return TerminalType::Konsole;
            }
            if Self::check_command("gnome-terminal") {
                return TerminalType::GnomeTerminal;
            }
            // 默认使用 xterm
            return TerminalType::GnomeTerminal;
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            TerminalType::Auto
        }
    }

    pub fn all() -> [TerminalType; 11] {
        [
            TerminalType::Auto,
            TerminalType::WindowsTerminal,
            TerminalType::PowerShell,
            TerminalType::Cmd,
            TerminalType::Iterm2,
            TerminalType::TerminalApp,
            TerminalType::GnomeTerminal,
            TerminalType::Konsole,
            TerminalType::Alacritty,
            TerminalType::Kitty,
            TerminalType::Custom,
        ]
    }

    #[cfg(not(windows))]
    fn check_command(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(unix)]
    fn check_app(app: &str) -> bool {
        Command::new("mdfind")
            .arg("kMDItemCFBundleIdentifier")
            .arg(app)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    fn check_command(cmd: &str) -> bool {
        Command::new("where")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// 终端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub terminal_type: TerminalType,
    pub custom_path: Option<PathBuf>,
    pub custom_args: Option<Vec<String>>,
    pub shell: Option<String>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            terminal_type: TerminalType::Auto,
            custom_path: None,
            custom_args: None,
            shell: None,
        }
    }
}

impl TerminalConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_effective_terminal(&self) -> TerminalType {
        if self.terminal_type == TerminalType::Auto {
            TerminalType::detect()
        } else {
            self.terminal_type
        }
    }

    /// 在终端中执行命令
    pub fn execute_command(&self, command: &str) -> Result<(), String> {
        let terminal = self.get_effective_terminal();

        #[cfg(target_os = "windows")]
        {
            self.execute_windows(terminal, command)
        }

        #[cfg(target_os = "macos")]
        {
            self.execute_macos(terminal, command)
        }

        #[cfg(target_os = "linux")]
        {
            self.execute_linux(terminal, command)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err("Unsupported platform".to_string())
        }
    }

    #[cfg(target_os = "windows")]
    fn execute_windows(&self, terminal: TerminalType, command: &str) -> Result<(), String> {
        let (program, args) = match terminal {
            TerminalType::WindowsTerminal => {
                // Windows Terminal
                (
                    "wt.exe",
                    vec![
                        "new-tab".to_string(),
                        "powershell.exe".to_string(),
                        "-NoExit".to_string(),
                        "-Command".to_string(),
                        command.to_string(),
                    ],
                )
            }
            TerminalType::PowerShell | TerminalType::Auto => {
                // PowerShell
                (
                    "powershell.exe",
                    vec![
                        "-NoExit".to_string(),
                        "-Command".to_string(),
                        command.to_string(),
                    ],
                )
            }
            TerminalType::Cmd => {
                // CMD
                ("cmd.exe", vec!["/k".to_string(), command.to_string()])
            }
            TerminalType::Custom => {
                let path = self
                    .custom_path
                    .as_ref()
                    .ok_or("Custom terminal path not specified")?;
                let mut args = self.custom_args.clone().unwrap_or_default();
                args.push(command.to_string());
                (path.to_str().unwrap(), args)
            }
            _ => {
                // 其他终端，默认使用 PowerShell
                (
                    "powershell.exe",
                    vec![
                        "-NoExit".to_string(),
                        "-Command".to_string(),
                        command.to_string(),
                    ],
                )
            }
        };

        Command::new(program)
            .args(&args)
            .spawn()
            .map_err(|e| format!("Failed to start terminal: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn execute_macos(&self, terminal: TerminalType, command: &str) -> Result<(), String> {
        let (app_name, args) = match terminal {
            TerminalType::Iterm2 => (
                "iTerm.app",
                vec!["/usr/local/bin/iterm2".to_string(), command.to_string()],
            ),
            TerminalType::TerminalApp | TerminalType::Auto => {
                let shell = self.shell.as_deref().unwrap_or("sh");
                (
                    "/Applications/Utilities/Terminal.app",
                    vec![
                        format!("/bin/{}", shell),
                        "-c".to_string(),
                        command.to_string(),
                    ],
                )
            }
            TerminalType::Alacritty => (
                "Alacritty",
                vec![
                    format!("/bin/{}", self.shell.as_deref().unwrap_or("sh")),
                    "-c".to_string(),
                    command.to_string(),
                ],
            ),
            TerminalType::Kitty => (
                "kitty.app",
                vec![
                    format!("/bin/{}", self.shell.as_deref().unwrap_or("sh")),
                    "-c".to_string(),
                    command.to_string(),
                ],
            ),
            TerminalType::Custom => {
                let path = self
                    .custom_path
                    .as_ref()
                    .ok_or("Custom terminal path not specified")?;
                let mut args = self.custom_args.clone().unwrap_or_default();
                args.push(command.to_string());
                (path.to_str().unwrap(), args)
            }
            _ => {
                // 默认使用 Terminal.app
                (
                    "/Applications/Utilities/Terminal.app",
                    vec![
                        format!("/bin/{}", self.shell.as_deref().unwrap_or("sh")),
                        "-c".to_string(),
                        command.to_string(),
                    ],
                )
            }
        };

        Command::new("open")
            .arg("-a")
            .arg(app_name)
            .args(&args)
            .spawn()
            .map_err(|e| format!("Failed to start terminal: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn execute_linux(&self, terminal: TerminalType, command: &str) -> Result<(), String> {
        let (program, args) = match terminal {
            TerminalType::GnomeTerminal => {
                let shell = self.shell.as_deref().unwrap_or("bash");
                (
                    "gnome-terminal",
                    vec![
                        "--".to_string(),
                        format!("/bin/{}", shell),
                        "-c".to_string(),
                        command.to_string(),
                    ],
                )
            }
            TerminalType::Konsole => {
                let shell = self.shell.as_deref().unwrap_or("bash");
                (
                    "konsole",
                    vec![
                        "-e".to_string(),
                        format!("/bin/{}", shell),
                        "-c".to_string(),
                        command.to_string(),
                    ],
                )
            }
            TerminalType::Alacritty => {
                let shell = self.shell.as_deref().unwrap_or("bash");
                (
                    "alacritty",
                    vec![
                        "-e".to_string(),
                        format!("/bin/{}", shell),
                        "-c".to_string(),
                        command.to_string(),
                    ],
                )
            }
            TerminalType::Kitty => {
                let shell = self.shell.as_deref().unwrap_or("bash");
                (
                    "kitty",
                    vec![
                        format!("/bin/{}", shell),
                        "-c".to_string(),
                        command.to_string(),
                    ],
                )
            }
            TerminalType::Auto | _ => {
                let shell = self.shell.as_deref().unwrap_or("bash");
                (
                    "gnome-terminal",
                    vec![
                        "--".to_string(),
                        format!("/bin/{}", shell),
                        "-c".to_string(),
                        command.to_string(),
                    ],
                )
            }
        };

        Command::new(program)
            .args(&args)
            .spawn()
            .map_err(|e| format!("Failed to start terminal: {}", e))?;

        Ok(())
    }
}
