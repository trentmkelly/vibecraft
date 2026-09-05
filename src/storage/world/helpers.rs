use std::fs::{self, File};
use std::path::{Component, Path};

use crate::storage::nbt::{read_gzip_named_tag, read_named_tag, Tag};

use crate::storage::datafix::{
    require_current_tag_data_version, require_current_world_data_version, TARGET_DATA_VERSION,
};

pub(super) fn lock_file_exclusive_nonblocking(
    file: &File,
    lock_path: &Path,
) -> std::io::Result<()> {
    try_lock_file_exclusive_nonblocking(file).map_err(|err| {
        if is_would_block_lock_error(&err) {
            std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                format!(
                    "{}: already locked (possibly by other Minecraft instance?)",
                    lock_path.display()
                ),
            )
        } else {
            err
        }
    })
}

pub(super) fn try_lock_file_exclusive_nonblocking(file: &File) -> std::io::Result<()> {
    Ok(file.try_lock()?)
}

pub(super) fn unlock_file(file: &File) -> std::io::Result<()> {
    file.unlock()
}

pub(super) fn is_would_block_lock_error(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
        || err.raw_os_error() == Some(libc::EWOULDBLOCK)
        || err.raw_os_error() == Some(libc::EAGAIN)
}

pub(super) fn level_dat_data_compound(tag: &Tag) -> Option<&[(String, Tag)]> {
    let Tag::Compound(values) = tag else {
        return None;
    };
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            ("Data", Tag::Compound(data_values)) => Some(data_values.as_slice()),
            _ => None,
        })
        .or(Some(values.as_slice()))
}

pub(super) fn compound_tag<'a>(
    values: &'a [(String, Tag)],
    key: &str,
) -> Option<&'a [(String, Tag)]> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::Compound(values)) if name == key => Some(values.as_slice()),
            _ => None,
        })
}

pub(super) fn compound_i32(values: &[(String, Tag)], key: &str) -> Option<i32> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::Int(value)) if name == key => Some(*value),
            _ => None,
        })
}

pub(super) fn compound_i8(values: &[(String, Tag)], key: &str) -> Option<i8> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::Byte(value)) if name == key => Some(*value),
            (name, Tag::Int(value)) if name == key => i8::try_from(*value).ok(),
            _ => None,
        })
}

pub(super) fn compound_i64(values: &[(String, Tag)], key: &str) -> Option<i64> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::Long(value)) if name == key => Some(*value),
            (name, Tag::Int(value)) if name == key => Some(i64::from(*value)),
            _ => None,
        })
}

pub(super) fn compound_f32(values: &[(String, Tag)], key: &str) -> Option<f32> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::Float(value)) if name == key => Some(*value),
            _ => None,
        })
}

pub(super) fn compound_string<'a>(values: &'a [(String, Tag)], key: &str) -> Option<&'a str> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::String(value)) if name == key => Some(value.as_str()),
            _ => None,
        })
}

pub(super) fn compound_bool(values: &[(String, Tag)], key: &str) -> Option<bool> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::Byte(value)) if name == key => Some(*value != 0),
            _ => None,
        })
}

pub(super) fn compound_clone(values: &[(String, Tag)], key: &str) -> Option<Tag> {
    values
        .iter()
        .find_map(|(name, value)| (name == key).then(|| value.clone()))
}

pub(super) fn compound_string_list(values: &[(String, Tag)], key: &str) -> Vec<String> {
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            (name, Tag::List(values)) if name == key => Some(
                values
                    .iter()
                    .filter_map(|tag| match tag {
                        Tag::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

pub(super) fn string_list_tag<'a>(values: impl Iterator<Item = &'a String>) -> Tag {
    Tag::List(values.cloned().map(Tag::String).collect())
}

pub(super) fn empty_compound_tag() -> Tag {
    Tag::Compound(Vec::new())
}

pub(super) fn empty_list_tag() -> Tag {
    Tag::List(Vec::new())
}

pub(super) fn validate_resource_location_namespace(value: &str) -> std::io::Result<()> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| !matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-'))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid resource location namespace",
        ));
    }
    Ok(())
}

pub(super) fn validate_resource_location_path(value: &str) -> std::io::Result<()> {
    if value.is_empty()
        || value.starts_with('/')
        || value.split('/').any(|segment| {
            segment.is_empty()
                || segment == "."
                || segment == ".."
                || segment
                    .bytes()
                    .any(|byte| !matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-'))
        })
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid resource location path",
        ));
    }
    Ok(())
}

pub(super) fn validate_level_id(value: &str) -> std::io::Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid level id",
        ));
    }
    Ok(())
}

pub(super) fn reject_symlinks_recursive(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "symlinks are not allowed",
        ));
    }
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            reject_symlinks_recursive(&entry?.path())?;
        }
    }
    Ok(())
}

pub(super) fn put_level_name(tag: &mut Tag, new_name: &str) {
    let Tag::Compound(values) = tag else {
        return;
    };
    if let Some(index) = values
        .iter()
        .position(|(name, value)| name == "Data" && matches!(value, Tag::Compound(_)))
    {
        if let Tag::Compound(data) = &mut values[index].1 {
            put_compound_string(data, "LevelName", new_name);
        }
    } else {
        put_compound_string(values, "LevelName", new_name);
    }
}

pub(super) fn put_compound_string(values: &mut Vec<(String, Tag)>, key: &str, value: &str) {
    if let Some((_, tag)) = values.iter_mut().find(|(name, _)| name == key) {
        *tag = Tag::String(value.to_string());
    } else {
        values.push((key.to_string(), Tag::String(value.to_string())));
    }
}

pub(super) fn remove_dir_recursive_except(path: &Path, except: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path == except {
            continue;
        }
        if entry_path.is_dir() {
            remove_dir_recursive_except(&entry_path, except)?;
            let _ = fs::remove_dir(&entry_path);
        } else {
            fs::remove_file(entry_path)?;
        }
    }
    Ok(())
}

pub(super) fn read_named_tag_file(path: &Path) -> std::io::Result<(String, Tag)> {
    let bytes = fs::read(path)?;
    read_named_tag(&mut bytes.as_slice())
}

pub(super) fn read_gzip_named_tag_file(path: &Path) -> std::io::Result<(String, Tag)> {
    let bytes = fs::read(path)?;
    let (name, tag) = read_gzip_named_tag(bytes.as_slice())?;
    // NbtIo.readCompressed delegates to NbtIo.read, which rejects non-compound
    // roots. Do this before data-version validation so PlayerDataStorage can
    // preserve the corrupt file and try .dat_old, just as Java does.
    if !matches!(tag, Tag::Compound(_)) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Root tag must be a named compound tag",
        ));
    }
    Ok((name, tag))
}

/// Read `level.dat`, auto-detecting the format. Vanilla writes gzip-compressed
/// NBT (`NbtIo.writeCompressed`, gzip magic `1f 8b`); this also accepts the
/// legacy uncompressed format VibeCraft previously wrote so existing worlds keep
/// loading after the move to vanilla-compatible gzip output.
pub(super) fn read_level_dat_file(path: &Path) -> std::io::Result<(String, Tag)> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(&[0x1f, 0x8b]) {
        read_gzip_named_tag(bytes.as_slice())
    } else {
        read_named_tag(&mut bytes.as_slice())
    }
}

/// Matches Java `FileNameDateFormatter.FORMATTER`: `yyyy-MM-dd_HH-mm-ss`
/// using the system local timezone (same as Java `ZonedDateTime.now()`).
pub(super) fn corruption_backup_stamp() -> String {
    std::process::Command::new("date")
        .arg("+%Y-%m-%d_%H-%M-%S")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            secs.to_string()
        })
}

/// Mirrors Java `Util.safeReplaceFile(target, newFile, backup)`:
/// 1. Rename existing target → backup (if target exists and backup is given)
/// 2. Delete target (if it still exists after rename)
/// 3. Rename temp → target
/// 4. On failure, attempt rollback from backup → target
///
/// Each step retries up to 10 times to match Java's `runWithRetries`.
pub(super) fn durable_write_with_backup(
    target: &Path,
    backup: Option<&Path>,
    bytes: &[u8],
) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = target.with_extension("tmp");
    fs::write(&tmp, bytes)?;

    if let Some(backup) = backup {
        if target.exists() {
            let _ = fs::remove_file(backup);
            retry_io(10, || fs::rename(target, backup))?;
        }
    }

    if target.exists() {
        retry_io(10, || fs::remove_file(target))?;
    }

    if let Err(err) = retry_io(10, || fs::rename(&tmp, target)) {
        if let Some(backup) = backup {
            if backup.exists() {
                let _ = fs::rename(backup, target);
            }
        }
        return Err(err);
    }

    Ok(())
}

fn retry_io<F: FnMut() -> std::io::Result<()>>(max_retries: u32, mut f: F) -> std::io::Result<()> {
    let mut last_err = None;
    for _ in 0..max_retries {
        match f() {
            Ok(()) => return Ok(()),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap_or_else(|| std::io::Error::other("retry exhausted with no attempts")))
}

pub(super) fn tag_with_data_version(tag: &Tag) -> Tag {
    let mut tag = tag.clone();
    if let Tag::Compound(values) = &mut tag {
        match values
            .iter_mut()
            .find(|(name, _)| name == "DataVersion")
            .map(|(_, value)| value)
        {
            Some(value) => *value = Tag::Int(TARGET_DATA_VERSION),
            None => values.push(("DataVersion".to_string(), Tag::Int(TARGET_DATA_VERSION))),
        }
    }
    tag
}

pub(super) fn checked_saved_tag(surface: &str, tag: Tag) -> std::io::Result<Tag> {
    require_current_tag_data_version(surface, &tag)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    Ok(tag)
}

pub(super) fn json_with_data_version(surface: &str, json: &str) -> std::io::Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(json).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{surface} JSON is invalid: {err}"),
        )
    })?;
    let Some(object) = value.as_object_mut() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{surface} JSON root must be an object"),
        ));
    };
    object.insert(
        "DataVersion".to_string(),
        serde_json::Value::Number(TARGET_DATA_VERSION.into()),
    );
    serde_json::to_string(&value).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{surface} JSON cannot be serialized: {err}"),
        )
    })
}

pub(super) fn checked_json_data_version(surface: &str, json: &str) -> std::io::Result<()> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{surface} JSON is invalid: {err}"),
        )
    })?;
    let version = value
        .as_object()
        .and_then(|object| object.get("DataVersion"))
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{surface} missing DataVersion"),
            )
        })?;
    require_current_world_data_version(version as i32)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}
