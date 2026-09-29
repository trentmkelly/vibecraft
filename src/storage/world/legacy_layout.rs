//! Upgrade of pre-26.1.2 world folder layouts.
//!
//! Ports `net.minecraft.util.filefix.fixes.DimensionStorageFileFix` (region,
//! entities, poi and the `data/*.dat` SavedData renames) and
//! `PlayerStorageFileFix` (`advancements`, `playerdata`, `stats` ->
//! `players/...`). Operations follow `FileFixOperations`/`FileFixUtil`: a move
//! whose source is missing is a no-op, and a move whose target already exists is
//! skipped (Java logs a warning) rather than merged or overwritten.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::WorldLayout;

/// `DimensionStorageFileFix` groupMove: (legacy folder, dimension folder).
const LEGACY_DIMENSION_FOLDERS: [(&str, &str); 3] = [
    ("", "dimensions/minecraft/overworld"),
    ("DIM-1", "dimensions/minecraft/the_nether"),
    ("DIM1", "dimensions/minecraft/the_end"),
];
/// Folders moved for each dimension.
const DIMENSION_SUBFOLDERS: [&str; 3] = ["region", "entities", "poi"];
/// `PlayerStorageFileFix` moves.
const PLAYER_FOLDERS: [(&str, &str); 3] = [
    ("advancements", "players/advancements"),
    ("playerdata", "players/data"),
    ("stats", "players/stats"),
];
/// Legacy per-dimension SavedData: (legacy data folder, new dimension data folder).
const LEGACY_DIMENSION_DATA: [(&str, &str); 3] = [
    ("data", "dimensions/minecraft/overworld/data/minecraft"),
    (
        "DIM-1/data",
        "dimensions/minecraft/the_nether/data/minecraft",
    ),
    ("DIM1/data", "dimensions/minecraft/the_end/data/minecraft"),
];

impl WorldLayout {
    /// Applies `DimensionStorageFileFix` then `PlayerStorageFileFix` to the world
    /// folder, in Java's operation order. Idempotent.
    pub fn migrate_legacy_layout(&self) -> io::Result<()> {
        let root = self.root.as_path();
        // applyInFolders(DIMENSIONS_DATA, ...): renames inside dimension data folders.
        for dimension in discover_dimensions(root) {
            let data = dimension.join("data");
            move_file(&data, "chunks.dat", "minecraft/chunk_tickets.dat")?;
            move_file(&data, "raids.dat", "minecraft/raids.dat")?;
            move_file(&data, "world_border.dat", "minecraft/world_border.dat")?;
        }
        // groupMove of the legacy `data` folders.
        for (legacy, target) in LEGACY_DIMENSION_DATA {
            move_file(
                root,
                &format!("{legacy}/chunks.dat"),
                &format!("{target}/chunk_tickets.dat"),
            )?;
            move_file(
                root,
                &format!("{legacy}/raids.dat"),
                &format!("{target}/raids.dat"),
            )?;
            move_file(
                root,
                &format!("{legacy}/raids_end.dat"),
                &format!("{target}/raids.dat"),
            )?;
            move_file(
                root,
                &format!("{legacy}/world_border.dat"),
                &format!("{target}/world_border.dat"),
            )?;
        }
        delete_file_or_empty_directory(root, "DIM1/data")?;
        delete_file_or_empty_directory(root, "DIM-1/data")?;
        // applyInFolders(DATA, ...): level-wide SavedData renames.
        let data = root.join("data");
        move_file(&data, "scoreboard.dat", "minecraft/scoreboard.dat")?;
        move_file(&data, "stopwatches.dat", "minecraft/stopwatches.dat")?;
        move_regex(&data, match_command_storage)?;
        move_file(&data, "idcounts.dat", "minecraft/maps/last_id.dat")?;
        move_regex(&data, match_map)?;
        move_file(
            &data,
            "random_sequences.dat",
            "minecraft/random_sequences.dat",
        )?;
        for (legacy, target) in LEGACY_DIMENSION_FOLDERS {
            for sub in DIMENSION_SUBFOLDERS {
                // Java's `"" + "/" + "region"` resolves to the root-relative `region`.
                let from = if legacy.is_empty() {
                    sub.to_string()
                } else {
                    format!("{legacy}/{sub}")
                };
                move_file(root, &from, &format!("{target}/{sub}"))?;
            }
        }
        delete_file_or_empty_directory(root, "DIM-1")?;
        delete_file_or_empty_directory(root, "DIM1")?;
        for (source, target) in PLAYER_FOLDERS {
            move_file(root, source, target)?;
        }
        Ok(())
    }
}

/// `FileRelation.DIMENSIONS`: every directory under `dimensions/<namespace>/`,
/// or the three default dimensions when none exist.
fn discover_dimensions(root: &Path) -> Vec<PathBuf> {
    let defaults = || {
        LEGACY_DIMENSION_FOLDERS
            .iter()
            .map(|(_, dimension)| root.join(dimension))
            .collect::<Vec<_>>()
    };
    let Ok(namespaces) = fs::read_dir(root.join("dimensions")) else {
        return defaults();
    };
    let mut found = Vec::new();
    for namespace in namespaces.flatten().filter(|e| e.path().is_dir()) {
        if let Ok(entries) = fs::read_dir(namespace.path()) {
            found.extend(entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()));
        }
    }
    if found.is_empty() {
        defaults()
    } else {
        found.sort();
        found
    }
}

/// `FileFixUtil.moveFile`: moves `base/from` to `base/to` when the source exists
/// and the target does not (an existing target is skipped, not overwritten).
fn move_file(base: &Path, from: &str, to: &str) -> io::Result<()> {
    let source = base.join(from);
    if !source.exists() {
        return Ok(());
    }
    let target = base.join(to);
    if target.exists() {
        return Ok(());
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, target)
}

/// `RegexMove.fix`: renames every entry of `base` for which `rename` yields a
/// new relative path (full-match semantics are inside the matcher).
fn move_regex(base: &Path, rename: fn(&str) -> Option<String>) -> io::Result<()> {
    if !base.is_dir() {
        return Ok(());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(base)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    for name in names {
        if let Some(new_name) = rename(&name) {
            move_file(base, &name, &new_name)?;
        }
    }
    Ok(())
}

/// `command_storage_([a-z0-9_.-]+)\.dat` -> `$1/command_storage.dat`.
fn match_command_storage(name: &str) -> Option<String> {
    let namespace = name
        .strip_prefix("command_storage_")?
        .strip_suffix(".dat")?;
    let valid = !namespace.is_empty()
        && namespace
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-'));
    valid.then(|| format!("{namespace}/command_storage.dat"))
}

/// `map_(\d+)\.dat` -> `minecraft/maps/$1.dat` (`\d` is ASCII in Java).
fn match_map(name: &str) -> Option<String> {
    let id = name.strip_prefix("map_")?.strip_suffix(".dat")?;
    let valid = !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit());
    valid.then(|| format!("minecraft/maps/{id}.dat"))
}

/// `FileFixUtil.deleteFileOrEmptyDirectory`: removes a file, or a directory only
/// when empty (a lone `.DS_Store` is removed first).
fn delete_file_or_empty_directory(base: &Path, file: &str) -> io::Result<()> {
    let target = base.join(file);
    if !target.exists() {
        return Ok(());
    }
    if target.is_dir() {
        let entries: Vec<_> = match fs::read_dir(&target) {
            Ok(read) => read.flatten().collect(),
            Err(_) => return Ok(()),
        };
        if entries.len() == 1 && entries[0].file_name() == ".DS_Store" {
            let _ = fs::remove_file(entries[0].path());
        }
        if fs::read_dir(&target)
            .map(|mut r| r.next().is_some())
            .unwrap_or(true)
        {
            return Ok(());
        }
        return fs::remove_dir(&target);
    }
    fs::remove_file(&target)
}
