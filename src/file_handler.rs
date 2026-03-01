use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Text,
    Image,
    Audio,
    Video,
    Document,
    Archive,
    Code,
    Unknown,
}

impl FileType {
    pub fn name(&self) -> &'static str {
        match self {
            FileType::Text => "文本",
            FileType::Image => "图片",
            FileType::Audio => "音频",
            FileType::Video => "视频",
            FileType::Document => "文档",
            FileType::Archive => "压缩包",
            FileType::Code => "代码",
            FileType::Unknown => "未知",
        }
    }

    pub fn display_name(&self) -> &'static str {
        self.name()
    }
}

/// 文件扩展名分类
impl FileType {
    fn from_extension(ext: &str) -> Self {
        let ext_lower = ext.to_lowercase();

        // 文本编辑器可编辑的文件
        let text_extensions = [
            "txt",
            "md",
            "rst",
            "adoc",
            "json",
            "xml",
            "yaml",
            "yml",
            "toml",
            "ini",
            "cfg",
            "conf",
            "log",
            "csv",
            "sh",
            "bash",
            "zsh",
            "fish",
            "ps1",
            "bat",
            "cmd",
            "py",
            "js",
            "ts",
            "java",
            "c",
            "cpp",
            "h",
            "hpp",
            "rs",
            "go",
            "swift",
            "makefile",
            "dockerfile",
            "gitignore",
            "gitmodules",
        ];

        // 图片文件
        let image_extensions = [
            "png", "jpg", "jpeg", "gif", "bmp", "tiff", "tif", "svg", "webp", "ico", "raw", "heic",
            "avif",
        ];

        // 音频文件
        let audio_extensions = [
            "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a", "opus", "aiff", "au",
        ];

        // 视频文件
        let video_extensions = [
            "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ogv", "rmvb",
        ];

        // 文档文件
        let doc_extensions = [
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "rtf", "epub",
            "mobi",
        ];

        // 压缩包文件
        let archive_extensions = [
            "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "txz", "iso", "img",
        ];

        // 代码文件（排除已包含的）
        let code_extensions = [
            "html", "css", "scss", "sass", "less", "vue", "jsx", "tsx", "php", "rb", "lua", "sql",
            "pl", "r", "m", "swift", "kt", "scala", "hs", "erl", "ex", "clj", "fs", "fsx", "v",
        ];

        if text_extensions.contains(&ext_lower.as_str()) {
            FileType::Text
        } else if image_extensions.contains(&ext_lower.as_str()) {
            FileType::Image
        } else if audio_extensions.contains(&ext_lower.as_str()) {
            FileType::Audio
        } else if video_extensions.contains(&ext_lower.as_str()) {
            FileType::Video
        } else if doc_extensions.contains(&ext_lower.as_str()) {
            FileType::Document
        } else if archive_extensions.contains(&ext_lower.as_str()) {
            FileType::Archive
        } else if code_extensions.contains(&ext_lower.as_str()) {
            FileType::Code
        } else {
            FileType::Unknown
        }
    }
}

/// 文件操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHandler {
    pub editor: Option<String>,
    pub open_with: Vec<(String, String)>, // (extension, application)
}

impl Default for FileHandler {
    fn default() -> Self {
        Self {
            editor: env::var("EDITOR").or_else(|_| env::var("VISUAL")).ok(),
            open_with: vec![],
        }
    }
}

impl FileHandler {
    pub fn new() -> Self {
        Self::default()
    }

    /// 检测文件类型
    pub fn detect_file_type(path: &Path) -> FileType {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| FileType::from_extension(ext))
            .unwrap_or(FileType::Unknown)
    }

    /// 打开文件
    pub fn open_file(&self, path: &Path) -> Result<(), String> {
        let file_type = Self::detect_file_type(path);

        match file_type {
            // 文本文件和代码文件 - 使用 EDITOR
            FileType::Text | FileType::Code => self.open_with_editor(path),
            // 其他文件 - 使用系统默认应用
            FileType::Image
            | FileType::Audio
            | FileType::Video
            | FileType::Document
            | FileType::Archive
            | FileType::Unknown => self.open_with_default(path),
        }
    }

    /// 用编辑器打开文件
    fn open_with_editor(&self, path: &Path) -> Result<(), String> {
        let editor = self
            .editor
            .as_ref()
            .ok_or("No editor configured. Set EDITOR environment variable.")?;

        #[cfg(target_os = "windows")]
        {
            // Windows: 需要检测编辑器类型
            if editor.to_lowercase().contains("code") {
                // VS Code
                Command::new("code")
                    .arg(path)
                    .spawn()
                    .map_err(|e| format!("Failed to open {} in VS Code: {}", path.display(), e))?;
            } else if editor.to_lowercase().contains("vim") {
                // Vim - 在终端中打开
                Command::new("cmd")
                    .args(["/c", "start", "cmd", "/k", "vim", path.to_str().unwrap()])
                    .spawn()
                    .map_err(|e| format!("Failed to open {} in Vim: {}", path.display(), e))?;
            } else {
                // 默认使用系统默认方式
                Command::new("cmd")
                    .args(["/c", "start", "", editor, path.to_str().unwrap()])
                    .spawn()
                    .map_err(|e| {
                        format!("Failed to open {} with {}: {}", path.display(), editor, e)
                    })?;
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: 检测编辑器类型
            if editor.to_lowercase().contains("code") {
                // VS Code
                Command::new("code")
                    .arg(path)
                    .spawn()
                    .map_err(|e| format!("Failed to open {} in VS Code: {}", path.display(), e))?;
            } else if editor.to_lowercase().contains("vim") || editor.to_lowercase().contains("vi")
            {
                // Vim - 在终端中打开
                Command::new("osascript")
                    .args([
                        "-e",
                        &format!(
                            "tell application \"Terminal\" to do script \"cd {}; vim {}; exit\"",
                            path.parent().unwrap().display(),
                            path.file_name().unwrap().to_str().unwrap()
                        ),
                    ])
                    .spawn()
                    .map_err(|e| format!("Failed to open {} in Vim: {}", path.display(), e))?;
            } else {
                // 默认使用 -a 参数打开
                Command::new("open")
                    .args(["-a", editor, path.to_str().unwrap()])
                    .spawn()
                    .map_err(|e| {
                        format!("Failed to open {} with {}: {}", path.display(), editor, e)
                    })?;
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: 检测编辑器类型
            if editor.to_lowercase().contains("code") {
                // VS Code
                Command::new("code")
                    .arg(path)
                    .spawn()
                    .map_err(|e| format!("Failed to open {} in VS Code: {}", path.display(), e))?;
            } else if editor.to_lowercase().contains("vim") || editor.to_lowercase().contains("vi")
            {
                // Vim - 在终端中打开
                let terminal =
                    env::var("TERMINAL").unwrap_or_else(|_| "gnome-terminal".to_string());
                Command::new(terminal)
                    .args(["--", "vim", path.to_str().unwrap()])
                    .spawn()
                    .map_err(|e| format!("Failed to open {} in Vim: {}", path.display(), e))?;
            } else {
                // 默认使用系统默认方式
                Command::new("xdg-open").arg(path).spawn().map_err(|e| {
                    format!("Failed to open {} with {}: {}", path.display(), editor, e)
                })?;
            }
        }

        Ok(())
    }

    /// 用系统默认应用打开文件
    fn open_with_default(&self, path: &Path) -> Result<(), String> {
        let path_str = path.to_str().ok_or("Invalid path")?;

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", "", path_str])
                .spawn()
                .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(path_str)
                .spawn()
                .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(path_str)
                .spawn()
                .map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
        }

        Ok(())
    }

    /// 在文件夹中打开
    pub fn open_in_explorer(&self, path: &Path) -> Result<(), String> {
        let path_str = path.to_str().ok_or("Invalid path")?;

        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(path_str)
                .spawn()
                .map_err(|e| format!("Failed to open in explorer: {}", e))?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .args(["-R", path_str]) // -R 同时选中并显示
                .spawn()
                .map_err(|e| format!("Failed to open in Finder: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(path_str)
                .spawn()
                .map_err(|e| format!("Failed to open in file manager: {}", e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        assert_eq!(FileType::from_extension("txt"), FileType::Text);
        assert_eq!(FileType::from_extension("py"), FileType::Text);
        assert_eq!(FileType::from_extension("rs"), FileType::Text);
        assert_eq!(FileType::from_extension("md"), FileType::Text);

        assert_eq!(FileType::from_extension("png"), FileType::Image);
        assert_eq!(FileType::from_extension("jpg"), FileType::Image);

        assert_eq!(FileType::from_extension("mp3"), FileType::Audio);
        assert_eq!(FileType::from_extension("mp4"), FileType::Video);

        assert_eq!(FileType::from_extension("pdf"), FileType::Document);
        assert_eq!(FileType::from_extension("zip"), FileType::Archive);

        assert_eq!(FileType::from_extension("unknown_ext"), FileType::Unknown);
    }
}
