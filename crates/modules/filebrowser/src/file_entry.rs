#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub hidden: bool,
}

impl FileEntry {
    pub fn new(path: &std::path::Path, show_hidden: bool) -> anyhow::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let modified = metadata.modified()?;
        let modified = chrono::DateTime::<chrono::Utc>::from(modified);

        Ok(Self {
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
            hidden: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false),
        })
    }

    pub fn display_name(&self) -> String {
        let prefix = if self.is_dir { "/" } else { "" };
        format!(
            "{}{}{}",
            if self.hidden { "." } else { "" },
            self.name,
            prefix
        )
    }

    pub fn format_size(&self) -> String {
        if self.is_dir {
            return "<DIR>".to_string();
        }

        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size < KB {
            format!("{}B", self.size)
        } else if self.size < MB {
            format!("{:.1}KB", self.size as f64 / KB as f64)
        } else if self.size < GB {
            format!("{:.1}MB", self.size as f64 / MB as f64)
        } else {
            format!("{:.1}GB", self.size as f64 / GB as f64)
        }
    }

    pub fn format_modified(&self) -> String {
        self.modified.format("%Y-%m-%d %H:%M").to_string()
    }
}
