use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Logger {
    latest_log: PathBuf,
    file: Mutex<File>,
}

impl Logger {
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, String> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)
            .map_err(|err| format!("Failed to create log directory '{}': {err}", dir.display()))?;
        let latest_log = dir.join("latest.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&latest_log)
            .map_err(|err| format!("Failed to open '{}': {err}", latest_log.display()))?;
        Ok(Self {
            latest_log,
            file: Mutex::new(file),
        })
    }

    pub fn latest_log(&self) -> &Path {
        &self.latest_log
    }

    pub fn info(&self, message: &str) -> Result<(), String> {
        self.write("INFO", message)
    }

    pub fn warn(&self, message: &str) -> Result<(), String> {
        self.write("WARN", message)
    }

    pub fn error(&self, message: &str) -> Result<(), String> {
        self.write("ERROR", message)
    }

    fn write(&self, level: &str, message: &str) -> Result<(), String> {
        let line = format!("[{}] [{}]: {}\n", timestamp(), level, message);
        print!("{line}");
        let mut file = self
            .file
            .lock()
            .map_err(|_| "log mutex poisoned".to_string())?;
        file.write_all(line.as_bytes())
            .map_err(|err| format!("Failed to write '{}': {err}", self.latest_log.display()))
    }
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::Logger;
    use std::fs;

    #[test]
    fn writes_lifecycle_messages_to_latest_log() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-logs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let logger = Logger::open(&dir).unwrap();
        logger.info("Starting RustCraft").unwrap();
        logger.warn("Safe mode active").unwrap();
        logger.error("Stopping RustCraft").unwrap();

        let contents = fs::read_to_string(logger.latest_log()).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(contents.contains("[INFO]: Starting RustCraft"));
        assert!(contents.contains("[WARN]: Safe mode active"));
        assert!(contents.contains("[ERROR]: Stopping RustCraft"));
    }
}
