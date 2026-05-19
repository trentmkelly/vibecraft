use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::GzEncoder;
use flate2::Compression;

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
        rotate_latest_log(dir, &latest_log)?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
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

fn rotate_latest_log(dir: &Path, latest_log: &Path) -> Result<(), String> {
    let metadata = match fs::metadata(latest_log) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(format!(
                "Failed to inspect '{}': {err}",
                latest_log.display()
            ))
        }
    };
    if metadata.len() == 0 {
        return Ok(());
    }

    let date = utc_date_for_log_name(SystemTime::now());
    let mut index = 1;
    let rotated = loop {
        let candidate = dir.join(format!("{date}-{index}.log.gz"));
        if !candidate.exists() {
            break candidate;
        }
        index += 1;
    };

    let bytes = fs::read(latest_log)
        .map_err(|err| format!("Failed to read '{}': {err}", latest_log.display()))?;
    let file = File::create(&rotated)
        .map_err(|err| format!("Failed to create '{}': {err}", rotated.display()))?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder
        .write_all(&bytes)
        .map_err(|err| format!("Failed to write '{}': {err}", rotated.display()))?;
    encoder
        .finish()
        .map_err(|err| format!("Failed to finish '{}': {err}", rotated.display()))?;
    fs::remove_file(latest_log)
        .map_err(|err| format!("Failed to remove '{}': {err}", latest_log.display()))
}

fn utc_date_for_log_name(time: SystemTime) -> String {
    let days = time
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::Logger;
    use flate2::read::GzDecoder;
    use std::fs;
    use std::io::Read;

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

    #[test]
    fn rotates_existing_latest_log_to_numbered_gzip_archive() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-log-rotation-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("latest.log"), "old log line\n").unwrap();

        let logger = Logger::open(&dir).unwrap();
        logger.info("new log line").unwrap();

        let latest = fs::read_to_string(logger.latest_log()).unwrap();
        assert!(latest.contains("new log line"));
        assert!(!latest.contains("old log line"));

        let rotated = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.file_name().to_string_lossy().ends_with("-1.log.gz"))
            .expect("expected rotated gzip log");
        let mut decoded = String::new();
        GzDecoder::new(fs::File::open(rotated.path()).unwrap())
            .read_to_string(&mut decoded)
            .unwrap();
        assert_eq!(decoded, "old log line\n");

        let _ = fs::remove_dir_all(&dir);
    }
}
