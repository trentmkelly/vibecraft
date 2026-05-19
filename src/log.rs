use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::GzEncoder;
use flate2::Compression;

/// Severity level for log filtering.
///
/// Variants are ordered so that numeric comparisons express "at least as verbose
/// as" relationships: `level >= LogLevel::Debug` is true when debug output is
/// enabled.  The discriminant values are intentional — do not reorder them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Normal operational messages.  Always emitted.
    Info = 0,
    /// Developer-facing diagnostic messages.  Emitted when level ≥ Debug.
    Debug = 1,
    /// Fine-grained protocol traces including hex dumps.  Emitted when level ≥ Trace.
    Trace = 2,
}

impl LogLevel {
    /// Parses a case-insensitive level name ("info", "debug", "trace").
    ///
    /// Returns an error string when `s` is not a recognised variant name.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(format!("Unknown log level: {s:?}. Expected info, debug, or trace.")),
        }
    }
}

// Implement the standard FromStr trait so callers can use `.parse::<LogLevel>()`.
impl FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LogLevel::from_str(s)
    }
}

// ---------------------------------------------------------------------------
// Global logger
// ---------------------------------------------------------------------------

/// The process-wide logger instance.  Set exactly once by [`init`].
static GLOBAL_LOGGER: OnceLock<Arc<Logger>> = OnceLock::new();

/// Moves `logger` into an [`Arc`], registers it as the process-wide global, and
/// returns the [`Arc`] so the caller can keep using the same instance.
///
/// Calling this a second time is a no-op — the first call wins.  The returned
/// [`Arc`] always points at the active global logger.
pub fn init(logger: Logger) -> Arc<Logger> {
    let arc = Arc::new(logger);
    // If already set, discard our new arc and return the existing one.
    let _ = GLOBAL_LOGGER.set(arc.clone());
    GLOBAL_LOGGER.get().expect("just set").clone()
}

/// Returns the current global log level.  Defaults to [`LogLevel::Info`] when
/// the global logger has not yet been initialised.
pub fn global_level() -> LogLevel {
    GLOBAL_LOGGER
        .get()
        .map(|l| l.level)
        .unwrap_or(LogLevel::Info)
}

// ---------------------------------------------------------------------------
// Free-function log helpers (route through the global, silently no-op when uninitialised)
// ---------------------------------------------------------------------------

/// Logs a message at INFO level via the global logger.
pub fn log_info(message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        let _ = logger.info(message);
    }
}

/// Logs a message at DEBUG level via the global logger.  No-ops when the global
/// level is below Debug.
pub fn log_debug(message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        let _ = logger.debug(message);
    }
}

/// Logs a message at TRACE level via the global logger.  No-ops when the global
/// level is below Trace.
pub fn log_trace(message: &str) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        let _ = logger.trace(message);
    }
}

/// Logs an outbound packet at TRACE level via the global logger.
///
/// `packet_id` is the numeric packet identifier.  `body` is the raw packet body
/// bytes — i.e. everything *after* the leading packet-ID VarInt.
pub fn log_packet_send(packet_id: i32, body: &[u8]) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        let _ = logger.packet_send(packet_id, body);
    }
}

// ---------------------------------------------------------------------------
// Logger struct
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Logger {
    /// Minimum level for messages to be written.
    pub level: LogLevel,
    latest_log: PathBuf,
    file: Mutex<File>,
}

impl Logger {
    /// Opens the log directory and creates `latest.log`, rotating any existing
    /// non-empty log file.  Defaults to [`LogLevel::Info`] filtering.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, String> {
        Self::open_with_level(dir, LogLevel::Info)
    }

    /// Opens the log directory with an explicit minimum log level.
    ///
    /// Messages below `level` are silently dropped.
    pub fn open_with_level(dir: impl AsRef<Path>, level: LogLevel) -> Result<Self, String> {
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
            level,
            latest_log,
            file: Mutex::new(file),
        })
    }

    pub fn latest_log(&self) -> &Path {
        &self.latest_log
    }

    /// Writes a message at INFO level.  Always emitted regardless of configured level.
    pub fn info(&self, message: &str) -> Result<(), String> {
        self.write("INFO", message)
    }

    /// Writes a message at WARN level.  Always emitted regardless of configured level.
    pub fn warn(&self, message: &str) -> Result<(), String> {
        self.write("WARN", message)
    }

    /// Writes a message at ERROR level.  Always emitted regardless of configured level.
    pub fn error(&self, message: &str) -> Result<(), String> {
        self.write("ERROR", message)
    }

    /// Writes a message at DEBUG level.  Filtered out when `self.level < Debug`.
    pub fn debug(&self, message: &str) -> Result<(), String> {
        if self.level >= LogLevel::Debug {
            self.write("DEBUG", message)
        } else {
            Ok(())
        }
    }

    /// Writes a message at TRACE level.  Filtered out when `self.level < Trace`.
    pub fn trace(&self, message: &str) -> Result<(), String> {
        if self.level >= LogLevel::Trace {
            self.write("TRACE", message)
        } else {
            Ok(())
        }
    }

    /// Logs an outbound packet at TRACE level.
    ///
    /// Emits the packet ID as hex, the body byte-count, and a formatted hex dump
    /// of `body` (every byte after the leading packet-ID VarInt).  No-ops when
    /// `self.level < Trace`.
    ///
    /// Example output:
    /// ```text
    /// SEND 0x26 (37 bytes)
    ///   0000  01 02 03 04 05 06 07 08  09 0a 0b 0c 0d 0e 0f 10  |................|
    /// ```
    pub fn packet_send(&self, packet_id: i32, body: &[u8]) -> Result<(), String> {
        if self.level < LogLevel::Trace {
            return Ok(());
        }
        let header = format!("SEND 0x{:02x} ({} bytes)", packet_id, body.len());
        let dump = hex_dump(body);
        let message = if dump.is_empty() {
            header
        } else {
            format!("{header}\n{dump}")
        };
        self.write("TRACE", &message)
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

// ---------------------------------------------------------------------------
// Hex-dump helper
// ---------------------------------------------------------------------------

/// Formats `bytes` as an annotated hex dump.
///
/// Each line contains up to 16 bytes rendered as pairs of hex digits separated
/// by a mid-group space at offset 8, and a printable-ASCII sidebar.  Columns
/// are aligned so that lines with fewer than 16 bytes are padded to keep the
/// sidebar at a fixed position.
///
/// Example (16-byte input):
/// ```text
///   0000  01 02 03 04 05 06 07 08  09 0a 0b 0c 0d 0e 0f 10  |................|
/// ```
fn hex_dump(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for (chunk_idx, chunk) in bytes.chunks(16).enumerate() {
        let offset = chunk_idx * 16;
        // Build the hex column.  Bytes 0–7 and 8–15 are separated by an extra
        // space to make it easier to count to a specific byte visually.
        let mut hex_col = String::new();
        for (i, b) in chunk.iter().enumerate() {
            if i == 8 {
                hex_col.push(' ');
            }
            hex_col.push_str(&format!("{b:02x} "));
        }
        // Pad to the width of a full 16-byte row so the ASCII sidebar lines up.
        // Full row: 8×3 + 1 + 8×3 = 24 + 1 + 24 = 49 chars (each byte is "xx " = 3).
        let full_hex_width = 49;
        while hex_col.len() < full_hex_width {
            hex_col.push(' ');
        }

        // ASCII sidebar: printable bytes verbatim, others replaced with '.'.
        let ascii_col: String = chunk
            .iter()
            .map(|&b| if (0x20..0x7f).contains(&b) { b as char } else { '.' })
            .collect();

        out.push_str(&format!("  {offset:04x}  {hex_col} |{ascii_col}|\n"));
    }
    // Remove trailing newline — callers add their own.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

// ---------------------------------------------------------------------------
// Timestamp / rotation helpers (unchanged from original)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{hex_dump, Logger, LogLevel};
    use flate2::read::GzDecoder;
    use std::fs;
    use std::io::Read;

    // -- Original tests (must remain passing) --------------------------------

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

    // -- Level filtering tests -----------------------------------------------

    #[test]
    fn debug_is_filtered_when_level_is_info() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-logs-debug-filter-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let logger = Logger::open_with_level(&dir, LogLevel::Info).unwrap();
        logger.debug("should not appear").unwrap();
        logger.info("should appear").unwrap();

        let contents = fs::read_to_string(logger.latest_log()).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(!contents.contains("should not appear"), "debug message leaked through Info filter");
        assert!(contents.contains("should appear"));
    }

    #[test]
    fn trace_is_filtered_when_level_is_debug() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-logs-trace-filter-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let logger = Logger::open_with_level(&dir, LogLevel::Debug).unwrap();
        logger.trace("trace should not appear").unwrap();
        logger.debug("debug should appear").unwrap();

        let contents = fs::read_to_string(logger.latest_log()).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(!contents.contains("trace should not appear"), "trace message leaked through Debug filter");
        assert!(contents.contains("debug should appear"));
    }

    // -- packet_send / hex_dump tests ----------------------------------------

    #[test]
    fn packet_send_produces_correct_hex_output() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-logs-packet-send-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let logger = Logger::open_with_level(&dir, LogLevel::Trace).unwrap();
        // 4 bytes so we get a single short hex-dump row.
        let body = [0x01u8, 0x02, 0xab, 0xff];
        logger.packet_send(0x26, &body).unwrap();

        let contents = fs::read_to_string(logger.latest_log()).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(contents.contains("SEND 0x26 (4 bytes)"), "missing SEND header; got:\n{contents}");
        assert!(contents.contains("0000"), "missing offset column");
        assert!(contents.contains("01 02 ab ff"), "missing hex bytes");
        assert!(contents.contains("|...."), "missing ASCII sidebar");
    }

    #[test]
    fn packet_send_is_suppressed_below_trace_level() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-logs-packet-send-filter-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let logger = Logger::open_with_level(&dir, LogLevel::Info).unwrap();
        logger.packet_send(0x10, &[0x00, 0x01]).unwrap();

        let contents = fs::read_to_string(logger.latest_log()).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert!(!contents.contains("SEND"), "packet_send leaked through Info filter");
    }

    #[test]
    fn hex_dump_formats_correctly() {
        // Single row, 4 bytes.
        let dump = hex_dump(&[0x01, 0x02, 0x41, 0x7e]);
        assert!(dump.contains("0000"), "missing offset");
        assert!(dump.contains("01 02 41 7e"), "wrong hex bytes");
        // 0x41 = 'A', 0x7e = '~' — both printable.
        assert!(dump.contains("|..A~|"), "wrong ASCII sidebar");
    }

    #[test]
    fn hex_dump_splits_at_16_bytes() {
        let data: Vec<u8> = (0u8..20).collect();
        let dump = hex_dump(&data);
        // Should have two rows.
        assert!(dump.contains("0000"), "missing first offset");
        assert!(dump.contains("0010"), "missing second offset");
    }

    #[test]
    fn hex_dump_empty_input_returns_empty_string() {
        assert_eq!(hex_dump(&[]), "");
    }

    // -- LogLevel::from_str tests --------------------------------------------

    #[test]
    fn log_level_from_str_parses_all_variants() {
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
    }

    #[test]
    fn log_level_from_str_is_case_insensitive() {
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("Debug").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("TRACE").unwrap(), LogLevel::Trace);
    }

    #[test]
    fn log_level_from_str_rejects_unknown() {
        assert!(LogLevel::from_str("verbose").is_err());
        assert!(LogLevel::from_str("").is_err());
    }
}
