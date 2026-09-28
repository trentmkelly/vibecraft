//! Upgrade of pre-26.1.2 world folder layouts.
//!
//! Java performs this in `net.minecraft.util.filefix.fixes.DimensionStorageFileFix`
//! (region/entities/poi) and `PlayerStorageFileFix` (`advancements`, `playerdata`,
//! `stats` -> `players/...`) through `FileFixOperations.move`/`groupMove`, which
//! only move a source that exists. The per-dimension `data` (SavedData) renames of
//! `DimensionStorageFileFix` are not performed here; see the TODO in
//! [`WorldLayout::migrate_legacy_layout`].

use std::fs;
use std::io;
use std::path::Path;

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

impl WorldLayout {
    /// Moves legacy folders to their 26.1.2 locations. Idempotent. A legacy folder
    /// whose destination already holds data is a conflict and aborts with
    /// `AlreadyExists` instead of merging, so no world data is overwritten.
    // TODO(dimension-data-file-fix): move `data/{chunks,raids,raids_end,
    // world_border,scoreboard,stopwatches,idcounts,map_N,command_storage_*,
    // random_sequences}.dat` to the namespaced per-dimension/level data layout.
    pub fn migrate_legacy_layout(&self) -> io::Result<()> {
        for (legacy, target) in LEGACY_DIMENSION_FOLDERS {
            for sub in DIMENSION_SUBFOLDERS {
                let source = self.root.join(legacy).join(sub);
                move_folder(&source, &self.root.join(target).join(sub))?;
            }
            if !legacy.is_empty() {
                // FileFixOperations.delete("DIM-1"/"DIM1"): only when now empty of
                // region data; leftover unmigrated data files keep it alive.
                let _ = fs::remove_dir(self.root.join(legacy));
            }
        }
        for (source, target) in PLAYER_FOLDERS {
            move_folder(&self.root.join(source), &self.root.join(target))?;
        }
        Ok(())
    }
}

/// Renames `source` to `target` if `source` is a directory.
fn move_folder(source: &Path, target: &Path) -> io::Result<()> {
    if !source.is_dir() {
        return Ok(());
    }
    if target.exists() && fs::read_dir(target)?.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "cannot upgrade legacy world folder {} because {} already exists",
                source.display(),
                target.display()
            ),
        ));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    if target.exists() {
        fs::remove_dir(target)?;
    }
    fs::rename(source, target)
}
