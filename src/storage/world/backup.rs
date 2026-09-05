//! LevelStorageAccess.makeWorldBackup: streamed, deflated world archives.
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::{Datelike, Timelike};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

pub(super) fn write_world_backup(
    world: &Path,
    backup_dir: &Path,
    level_id: &str,
    stamp: &str,
) -> io::Result<PathBuf> {
    fs::create_dir_all(backup_dir)?;
    let (path, file) = create_archive(backup_dir, &format!("{stamp}_{level_id}"))?;
    let mut archive = ZipWriter::new(BufWriter::new(file));
    write_directory(&mut archive, world, Path::new(level_id))?;
    // Finish the central directory and explicitly flush: Drop cannot report IO errors.
    archive.finish()?.flush()?;
    Ok(path)
}

/// FileUtil's sanitization and collision suffixes for timestamp-prefixed names.
/// Timestamp-prefixed names cannot match reserved Windows device names.
fn create_archive(directory: &Path, prefix: &str) -> io::Result<(PathBuf, File)> {
    let sanitized: String = prefix
        .chars()
        .map(|c| match c {
            '/' | '\n' | '\r' | '\t' | '\0' | '\u{c}' | '`' | '?' | '*' | '\\' | '<' | '>'
            | '|' | '"' | ':' | '.' => '_',
            _ => c,
        })
        .collect();
    let base = java_prefix(&sanitized, 255 - ".zip".len());
    for count in 0_u32.. {
        let name = if count == 0 {
            format!("{base}.zip")
        } else {
            let suffix = format!(" ({count})");
            format!("{}{suffix}.zip", java_prefix(&base, 255 - suffix.len()))
        };
        let path = directory.join(name);
        // Reserve the file atomically so concurrent backups never overwrite one
        // another between FileUtil's name probing and opening the output stream.
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("exhausted world backup names"))
}

fn java_prefix(value: &str, units: usize) -> String {
    String::from_utf16_lossy(&value.encode_utf16().take(units).collect::<Vec<_>>())
}

fn write_directory(
    archive: &mut ZipWriter<BufWriter<File>>,
    source: &Path,
    entry_root: &Path,
) -> io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry_root.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            // Java's visitor emits files only, so empty directories are omitted.
            write_directory(archive, &entry.path(), &entry_path)?;
        } else if entry.file_name() != super::SESSION_LOCK_FILE {
            let mut file = File::open(entry.path())?;
            let now = chrono::Local::now();
            let timestamp = zip::DateTime::from_date_and_time(
                now.year() as u16,
                now.month() as u8,
                now.day() as u8,
                now.hour() as u8,
                now.minute() as u8,
                now.second() as u8,
            )
            .map_err(io::Error::other)?;
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(6))
                .last_modified_time(timestamp)
                .large_file(file.metadata()?.len() >= u32::MAX as u64);
            archive.start_file(entry_path.to_string_lossy().replace('\\', "/"), options)?;
            io::copy(&mut file, archive)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
