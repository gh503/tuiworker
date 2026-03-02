use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

pub struct Logger {
    log_file: Mutex<Option<File>>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            log_file: Mutex::new(None),
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let log_path = "tuiworker.log";

        // Write separator for new session and init log file
        {
            let mut file = self.log_file.lock().unwrap();
            // If log file already exists, just use it
            if file.is_none() {
                let mut temp_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_path)?;
                let separator = format!(
                    "\n=== New Session Started at {} ===\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S")
                );
                let _ = temp_file.write_all(separator.as_bytes());
                *file = Some(temp_file);
            }
        }

        self.log("INFO", "=== App Started ===");
        Ok(())
    }

    pub fn log(&self, level: &str, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!("[{} {}] {}\n", timestamp, level, message);

        // Write to file
        if let Some(ref mut file) = self.log_file.lock().unwrap().as_mut() {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }

        // Also print to stderr for immediate visibility
        eprintln!("{}", log_line.trim_end());
    }

    pub fn debug(&self, message: &str) {
        self.log("DEBUG", message);
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn warn(&self, message: &str) {
        self.log("WARN", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
}

// Global logger instance
static mut LOGGER: Option<Logger> = None;

pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        LOGGER = Some(Logger::new());
        if let Some(ref logger) = LOGGER {
            logger.init()?;
        }
    }
    Ok(())
}

pub fn log_debug(message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.debug(message);
        }
    }
}

pub fn log_info(message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.info(message);
        }
    }
}

pub fn log_warn(message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.warn(message);
        }
    }
}

pub fn log_error(message: &str) {
    unsafe {
        if let Some(ref logger) = LOGGER {
            logger.error(message);
        }
    }
}
