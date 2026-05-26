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

pub(super) fn sanitize_backup_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' => ch,
            _ => '_',
        })
        .collect()
}

pub(super) fn copy_dir_recursive(
    source: &Path,
    target: &Path,
    skip_file_name: Option<&str>,
) -> std::io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        if skip_file_name.is_some_and(|skip| {
            source_path.file_name().and_then(|name| name.to_str()) == Some(skip)
        }) {
            continue;
        }
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path, skip_file_name)?;
        } else if file_type.is_file() {
            fs::copy(source_path, target_path)?;
        }
    }
    Ok(())
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
    read_gzip_named_tag(bytes.as_slice())
}

pub(super) fn corruption_backup_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}

pub(super) fn durable_write_with_backup(
    target: &Path,
    backup: Option<&Path>,
    bytes: &[u8],
) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Some(backup) = backup {
        if target.exists() {
            fs::copy(target, backup)?;
        }
    }

    let tmp = target.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, target)
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
