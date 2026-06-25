#![allow(dead_code)]

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use std::cell::Cell;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const COMPRESS_BUFFER_SIZE: usize = 4096;
const COMPRESSED_EXTENSION: &str = ".gz";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLogDirectory {
    root: PathBuf,
    extension: String,
}

impl EventLogDirectory {
    pub fn open(root: impl AsRef<Path>, extension: &str) -> io::Result<Self> {
        fs::create_dir_all(root.as_ref())?;
        Ok(Self {
            root: root.as_ref().to_path_buf(),
            extension: extension.to_string(),
        })
    }

    pub fn list_files(&self) -> io::Result<FileList> {
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.is_file() {
                if let Some(file) = self.parse_file(&path) {
                    files.push(file);
                }
            }
        }
        Ok(FileList::new(files))
    }

    pub fn create_new_file(&self, date: SimpleDate) -> io::Result<RawFile> {
        let ids = self.list_files()?.ids();
        let mut index = 1;
        loop {
            let id = FileId { date, index };
            if !ids.contains(&id) {
                let file = RawFile {
                    path: self.root.join(id.to_file_name(&self.extension)),
                    id,
                };
                File::create_new(&file.path)?;
                return Ok(file);
            }
            index += 1;
        }
    }

    fn parse_file(&self, path: &Path) -> Option<EventLogFile> {
        let file_name = path.file_name()?.to_str()?;
        let extension_index = file_name.find('.')?;
        let id = FileId::parse(&file_name[..extension_index])?;
        let extension = &file_name[extension_index..];
        if extension == self.extension {
            Some(EventLogFile::Raw(RawFile {
                path: path.to_path_buf(),
                id,
            }))
        } else if extension == format!("{}{}", self.extension, COMPRESSED_EXTENSION) {
            Some(EventLogFile::Compressed(CompressedFile {
                path: path.to_path_buf(),
                id,
            }))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SimpleDate {
    year: i32,
    month: u32,
    day: u32,
}

impl SimpleDate {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, String> {
        if !(1..=12).contains(&month) {
            return Err(format!("invalid month: {month}"));
        }
        let max_day = days_in_month(year, month);
        if day == 0 || day > max_day {
            return Err(format!("invalid day: {day}"));
        }
        Ok(Self { year, month, day })
    }

    pub fn parse_basic_iso(value: &str) -> Option<Self> {
        if value.len() != 8 || !value.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        let year = value[0..4].parse().ok()?;
        let month = value[4..6].parse().ok()?;
        let day = value[6..8].parse().ok()?;
        Self::new(year, month, day).ok()
    }

    fn days_since_epoch(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    fn plus_days(self, days: i32) -> Self {
        civil_from_days(self.days_since_epoch() + i64::from(days))
    }
}

impl std::fmt::Display for SimpleDate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:04}{:02}{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId {
    pub date: SimpleDate,
    pub index: i32,
}

impl FileId {
    pub fn parse(name: &str) -> Option<Self> {
        let separator = name.find('-')?;
        let date = SimpleDate::parse_basic_iso(&name[..separator])?;
        let index = name[separator + 1..].parse().ok()?;
        Some(Self { date, index })
    }

    pub fn to_file_name(self, extension: &str) -> String {
        format!("{self}{extension}")
    }
}

impl std::fmt::Display for FileId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}-{}", self.date, self.index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventLogFile {
    Raw(RawFile),
    Compressed(CompressedFile),
}

impl EventLogFile {
    pub fn path(&self) -> &Path {
        match self {
            Self::Raw(file) => &file.path,
            Self::Compressed(file) => &file.path,
        }
    }

    pub fn id(&self) -> FileId {
        match self {
            Self::Raw(file) => file.id,
            Self::Compressed(file) => file.id,
        }
    }

    pub fn open_reader(&self) -> io::Result<Option<Box<dyn Read>>> {
        match self {
            Self::Raw(file) => file.open_reader(),
            Self::Compressed(file) => file.open_reader(),
        }
    }

    pub fn compress(self) -> io::Result<EventLogFile> {
        match self {
            Self::Raw(file) => Ok(EventLogFile::Compressed(file.compress()?)),
            Self::Compressed(file) => Ok(EventLogFile::Compressed(file)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFile {
    path: PathBuf,
    id: FileId,
}

impl RawFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn open_channel(&self) -> io::Result<File> {
        OpenOptions::new().read(true).write(true).open(&self.path)
    }

    pub fn open_reader(&self) -> io::Result<Option<Box<dyn Read>>> {
        if self.path.exists() {
            Ok(Some(Box::new(File::open(&self.path)?)))
        } else {
            Ok(None)
        }
    }

    pub fn compress(self) -> io::Result<CompressedFile> {
        let compressed_path = self
            .path
            .with_file_name(format!("{}{}", file_name_string(&self.path)?, COMPRESSED_EXTENSION));
        try_compress(&self.path, &compressed_path)?;
        Ok(CompressedFile {
            path: compressed_path,
            id: self.id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedFile {
    path: PathBuf,
    id: FileId,
}

impl CompressedFile {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn open_reader(&self) -> io::Result<Option<Box<dyn Read>>> {
        if self.path.exists() {
            Ok(Some(Box::new(GzDecoder::new(File::open(&self.path)?))))
        } else {
            Ok(None)
        }
    }

    pub fn compress(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileList {
    files: Vec<EventLogFile>,
}

impl FileList {
    fn new(files: Vec<EventLogFile>) -> Self {
        Self { files }
    }

    pub fn prune(mut self, date: SimpleDate, expiry_days: i32) -> Self {
        self.files.retain(|file| {
            let expires_at = file.id().date.plus_days(expiry_days);
            if date.days_since_epoch() >= expires_at.days_since_epoch() {
                fs::remove_file(file.path()).is_err()
            } else {
                true
            }
        });
        self
    }

    pub fn compress_all(mut self) -> Self {
        for index in 0..self.files.len() {
            let file = self.files[index].clone();
            if let Ok(compressed) = file.compress() {
                self.files[index] = compressed;
            }
        }
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &EventLogFile> {
        self.files.iter()
    }

    pub fn ids(&self) -> HashSet<FileId> {
        self.files.iter().map(EventLogFile::id).collect()
    }
}

fn try_compress(raw: &Path, compressed: &Path) -> io::Result<()> {
    if compressed.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Compressed target file already exists: {}", compressed.display()),
        ));
    }

    let mut input = OpenOptions::new().read(true).write(true).open(raw)?;
    let mut output = GzEncoder::new(File::create(compressed)?, Compression::default());
    let mut buffer = [0_u8; COMPRESS_BUFFER_SIZE];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read])?;
    }
    output.finish()?;
    input.set_len(0)?;
    fs::remove_file(raw)
}

fn file_name_string(path: &Path) -> io::Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))
}

#[derive(Debug)]
pub struct JsonEventLog {
    file: Rc<File>,
    reference_count: Rc<Cell<i32>>,
}

impl JsonEventLog {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        Ok(Self {
            file: Rc::new(file),
            reference_count: Rc::new(Cell::new(1)),
        })
    }

    pub fn write(&self, event: &Value) -> io::Result<()> {
        let mut writer = self.file.try_clone()?;
        writer.seek(SeekFrom::End(0))?;
        serde_json::to_writer(&mut writer, event).map_err(io::Error::other)?;
        writer.write_all(b"\n")?;
        writer.flush()
    }

    pub fn open_reader(&self) -> io::Result<JsonEventLogReader> {
        if self.reference_count.get() <= 0 {
            return Err(io::Error::other("Event log has already been closed"));
        }
        self.reference_count.set(self.reference_count.get() + 1);
        Ok(JsonEventLogReader {
            file: Rc::clone(&self.file),
            reference_count: Some(Rc::clone(&self.reference_count)),
            position: 0,
        })
    }

    pub fn close(&self) {
        release_reference(&self.reference_count);
    }
}

#[derive(Debug)]
pub struct JsonEventLogReader {
    file: Rc<File>,
    reference_count: Option<Rc<Cell<i32>>>,
    position: u64,
}

impl JsonEventLogReader {
    pub fn create(file: File) -> Self {
        Self {
            file: Rc::new(file),
            reference_count: None,
            position: 0,
        }
    }

    pub fn next(&mut self) -> io::Result<Option<Value>> {
        let mut file = self.file.try_clone()?;
        file.seek(SeekFrom::Start(self.position))?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        self.position += read as u64;
        if read == 0 {
            return Ok(None);
        }
        let value = serde_json::from_str(line.trim_end()).map_err(io::Error::other)?;
        Ok(Some(value))
    }

    pub fn close(&mut self) {
        if let Some(reference_count) = self.reference_count.take() {
            release_reference(&reference_count);
        }
    }
}

impl Drop for JsonEventLogReader {
    fn drop(&mut self) {
        self.close();
    }
}

fn release_reference(reference_count: &Cell<i32>) {
    reference_count.set(reference_count.get() - 1);
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i64::from(era * 146_097 + doe - 719_468)
}

fn civil_from_days(days: i64) -> SimpleDate {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    SimpleDate {
        year: (y + i64::from(month <= 2)) as i32,
        month: month as u32,
        day: day as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn must_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("expected Ok(..), got Err({err:?})"),
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(err) => panic!("system time before epoch: {err}"),
        };
        std::env::temp_dir().join(format!("vibecraft-{name}-{nanos}"))
    }

    fn date(year: i32, month: u32, day: u32) -> SimpleDate {
        must_ok(SimpleDate::new(year, month, day))
    }

    #[test]
    fn file_id_parse_and_file_matching_follow_java_names() {
        let source = vibecraft_java_source!("/net/minecraft/util/eventlog/EventLogDirectory.java");
        assert!(source.contains("DateTimeFormatter.BASIC_ISO_DATE"));
        assert!(source.contains("fileName.indexOf(46)"));
        assert!(source.contains("extension.equals(this.extension + \".gz\")"));

        assert_eq!(
            FileId::parse("20260625-3"),
            Some(FileId {
                date: date(2026, 6, 25),
                index: 3,
            })
        );
        assert_eq!(FileId::parse("2026-3"), None);
        assert_eq!(
            FileId {
                date: date(2026, 6, 25),
                index: 7,
            }
            .to_file_name(".json"),
            "20260625-7.json"
        );

        let root = temp_dir("eventlog-parse");
        let directory = must_ok(EventLogDirectory::open(&root, ".json"));
        must_ok(fs::write(root.join("20260625-1.json"), b"{}"));
        must_ok(fs::write(root.join("20260625-2.json.gz"), b"not-gzip-yet"));
        must_ok(fs::write(root.join("20260625-3.txt"), b"ignored"));
        must_ok(fs::write(root.join("bad.json"), b"ignored"));

        let mut ids = directory.list_files().map(|files| files.ids()).unwrap_or_else(|err| {
            panic!("failed to list event logs: {err}");
        });
        assert_eq!(ids.len(), 2);
        assert!(ids.remove(&FileId {
            date: date(2026, 6, 25),
            index: 1,
        }));
        assert!(ids.remove(&FileId {
            date: date(2026, 6, 25),
            index: 2,
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn create_new_file_skips_existing_ids_and_compresses_raw_logs() {
        let source = vibecraft_java_source!("/net/minecraft/util/eventlog/EventLogDirectory.java");
        assert!(source.contains("do {\n         id = new EventLogDirectory.FileId(date, index++);"));
        assert!(source.contains("new GZIPOutputStream"));
        assert!(source.contains("channel.truncate(0L);"));
        assert!(source.contains("Files.delete(raw);"));

        let root = temp_dir("eventlog-compress");
        let directory = must_ok(EventLogDirectory::open(&root, ".json"));
        let first = must_ok(directory.create_new_file(date(2026, 6, 25)));
        assert_eq!(first.id().index, 1);
        must_ok(fs::write(first.path(), b"{\"a\":1}\n"));
        let second = must_ok(directory.create_new_file(date(2026, 6, 25)));
        assert_eq!(second.id().index, 2);

        let compressed = must_ok(first.compress());
        assert!(!root.join("20260625-1.json").exists());
        assert!(compressed.path().ends_with("20260625-1.json.gz"));

        let mut text = String::new();
        let mut reader = must_ok(compressed.open_reader()).unwrap_or_else(|| {
            panic!("compressed file should exist");
        });
        must_ok(reader.read_to_string(&mut text));
        assert_eq!(text, "{\"a\":1}\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn file_list_prunes_expired_logs_and_compresses_all_raw_files() {
        let source = vibecraft_java_source!("/net/minecraft/util/eventlog/EventLogDirectory.java");
        assert!(source.contains("LocalDate expiresAt = id.date().plusDays(expiryDays);"));
        assert!(source.contains("if (!date.isBefore(expiresAt))"));
        assert!(source.contains("iterator.set(file.compress())"));

        let root = temp_dir("eventlog-prune");
        let directory = must_ok(EventLogDirectory::open(&root, ".json"));
        let expired = must_ok(directory.create_new_file(date(2026, 1, 1)));
        let fresh = must_ok(directory.create_new_file(date(2026, 1, 10)));
        must_ok(fs::write(expired.path(), b"old"));
        must_ok(fs::write(fresh.path(), b"fresh"));

        let files = must_ok(directory.list_files()).prune(date(2026, 1, 8), 7);
        assert_eq!(files.ids().len(), 1);
        assert!(!expired.path().exists());
        assert!(fresh.path().exists());

        let compressed = must_ok(directory.list_files()).compress_all();
        assert!(compressed.iter().all(|file| matches!(file, EventLogFile::Compressed(_))));
        assert!(root.join("20260110-1.json.gz").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn json_event_log_appends_json_lines_and_readers_track_independent_positions() {
        let source = vibecraft_java_source!("/net/minecraft/util/eventlog/JsonEventLog.java");
        assert!(source.contains("this.channel.position(this.channel.size())"));
        assert!(source.contains("writer.write(10)"));
        assert!(source.contains("private volatile long position"));
        assert!(source.contains("Event log has already been closed"));

        let root = temp_dir("json-eventlog");
        must_ok(fs::create_dir_all(&root));
        let path = root.join("events.json");
        let log = must_ok(JsonEventLog::open(&path));
        must_ok(log.write(&json!({"kind":"start"})));
        must_ok(log.write(&json!({"kind":"stop"})));

        let mut first_reader = must_ok(log.open_reader());
        let mut second_reader = must_ok(log.open_reader());
        assert_eq!(must_ok(first_reader.next()), Some(json!({"kind":"start"})));
        assert_eq!(must_ok(first_reader.next()), Some(json!({"kind":"stop"})));
        assert_eq!(must_ok(first_reader.next()), None);
        assert_eq!(must_ok(second_reader.next()), Some(json!({"kind":"start"})));

        first_reader.close();
        second_reader.close();
        log.close();
        assert_eq!(
            log.open_reader()
                .map(|_| "opened".to_string())
                .map_err(|err| err.to_string()),
            Err("Event log has already been closed".to_string())
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn json_event_log_reader_returns_none_at_eof_and_wraps_parse_errors() {
        let source = vibecraft_java_source!("/net/minecraft/util/eventlog/JsonEventLogReader.java");
        assert!(source.contains("jsonReader.setStrictness(Strictness.LENIENT)"));
        assert!(source.contains("if (!jsonReader.hasNext())"));
        assert!(source.contains("catch (JsonParseException e)"));
        assert!(source.contains("catch (EOFException e)"));

        let root = temp_dir("json-eventlog-reader");
        must_ok(fs::create_dir_all(&root));
        let path = root.join("events.json");
        must_ok(fs::write(&path, "{\"a\":1}\n"));
        let mut reader = JsonEventLogReader::create(must_ok(File::open(&path)));
        assert_eq!(must_ok(reader.next()), Some(json!({"a":1})));
        assert_eq!(must_ok(reader.next()), None);

        let bad_path = root.join("bad.json");
        must_ok(fs::write(&bad_path, "{bad json}\n"));
        let mut bad_reader = JsonEventLogReader::create(must_ok(File::open(&bad_path)));
        assert!(bad_reader.next().is_err());
        let _ = fs::remove_dir_all(root);
    }
}
