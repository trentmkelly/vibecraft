//! Loader glue for the DataFixer: upgrades old saved tags on read.
//!
//! Java's storage classes call `DataFixTypes.updateToCurrentVersion(fixer, tag,
//! dataVersion)` right after reading a tag (`ChunkStorage.upgradeChunkTag`,
//! `PlayerDataStorage.load`, `SavedDataStorage.readTagFromDisk`,
//! `LevelStorageSource.readLevelDataTagFixed`, ...). [`upgrade_saved_tag`] is that
//! call on top of the [`crate::datafix`] port.
//!
//! While the fixer chain is incomplete (see
//! [`crate::datafix::chain::COMPLETE_THROUGH`]) tags older than the current
//! `DataVersion` are still refused with the historical error message rather than
//! being upgraded by an incomplete chain.

use std::borrow::Cow;

use crate::datafix::chain::data_fixer;
use crate::datafix::dynamic::{remove, set};
use crate::datafix::fixer::DataFixer;
use crate::datafix::references::{self as refs, TypeReference};
use crate::storage::nbt::Tag;

use super::datafix::{require_current_tag_data_version, tag_data_version, TARGET_DATA_VERSION};

/// Whether the ported chain reaches the current data version, i.e. whether older
/// data can be upgraded instead of refused.
pub fn chain_reaches_current_version() -> bool {
    data_fixer().complete_through() >= TARGET_DATA_VERSION
}

/// `DataFixTypes.updateToCurrentVersion` for a tag carrying its own `DataVersion`.
/// Tags at or above the current version are returned as they are.
pub fn upgrade_saved_tag(surface: &str, reference: TypeReference, tag: Tag) -> Result<Tag, String> {
    upgrade_saved_tag_with(data_fixer(), surface, reference, tag)
}

/// [`upgrade_saved_tag`] with an explicit fixer.
fn upgrade_saved_tag_with(
    fixer: &DataFixer,
    surface: &str,
    reference: TypeReference,
    tag: Tag,
) -> Result<Tag, String> {
    let version = tag_data_version(&tag).ok_or_else(|| format!("{surface} missing DataVersion"))?;
    if version >= TARGET_DATA_VERSION {
        return Ok(tag);
    }
    let mut body = tag.clone();
    remove(&mut body, "DataVersion");
    match fixer.update(reference, &body, version, TARGET_DATA_VERSION) {
        Ok(mut upgraded) => {
            set(&mut upgraded, "DataVersion", Tag::Int(TARGET_DATA_VERSION));
            Ok(upgraded)
        }
        // The chain cannot reach the current version yet: report the same
        // refusal as before the DataFixer existed.
        Err(_) => match require_current_tag_data_version(surface, &tag) {
            Err(refusal) => Err(refusal),
            Ok(()) => Err(format!("{surface} could not be upgraded")),
        },
    }
}

/// Borrowing variant of [`upgrade_saved_tag`] for readers that only hold `&Tag`.
pub fn upgraded_saved_tag<'a>(
    surface: &str,
    reference: TypeReference,
    tag: &'a Tag,
) -> Result<Cow<'a, Tag>, String> {
    match tag_data_version(tag) {
        Some(version) if version >= TARGET_DATA_VERSION => Ok(Cow::Borrowed(tag)),
        _ => upgrade_saved_tag(surface, reference, tag.clone()).map(Cow::Owned),
    }
}

/// Whether a `level.dat` of `data_version` can be loaded: the current version, or
/// an older one that the fixer chain can upgrade. Newer worlds are refused.
pub fn check_level_data_version(data_version: i32) -> Result<(), String> {
    if data_version < TARGET_DATA_VERSION && chain_reaches_current_version() {
        return Ok(());
    }
    super::datafix::require_current_world_data_version(data_version)
}

/// `level.dat` is `{Data: <level>}`; the `Data` compound is the `LEVEL` type
/// (`LevelStorageSource.readLevelDataTagFixed`).
pub fn upgrade_level_dat(root: Tag) -> Result<Tag, String> {
    let mut root = root;
    let Some(data) = crate::datafix::dynamic::get(&root, "Data").cloned() else {
        return Ok(root);
    };
    let upgraded = upgrade_saved_tag("level.dat", refs::LEVEL, data)?;
    set(&mut root, "Data", upgraded);
    Ok(root)
}

/// The `DataFixTypes` of a saved data file (`SavedDataType.dataFixType`), by the
/// path of its id, or `None` for files that are not vanilla saved data.
pub fn saved_data_fix_type(path: &str) -> Option<TypeReference> {
    let reference = match path {
        "scoreboard" => refs::SAVED_DATA_SCOREBOARD,
        "chunk_tickets" => refs::SAVED_DATA_TICKETS,
        "world_border" => refs::SAVED_DATA_WORLD_BORDER,
        "world_gen_settings" => refs::SAVED_DATA_WORLD_GEN_SETTINGS,
        "wandering_trader" => refs::SAVED_DATA_WANDERING_TRADER,
        "weather" => refs::SAVED_DATA_WEATHER,
        "game_rules" => refs::SAVED_DATA_GAME_RULES,
        "ender_dragon_fight" => refs::SAVED_DATA_ENDER_DRAGON_FIGHT,
        "scheduled_events" => refs::SAVED_DATA_SCHEDULED_EVENTS,
        "world_clocks" => refs::SAVED_DATA_WORLD_CLOCKS,
        "custom_boss_events" => refs::SAVED_DATA_CUSTOM_BOSS_EVENTS,
        "stopwatches" => refs::SAVED_DATA_STOPWATCHES,
        "random_sequences" => refs::SAVED_DATA_RANDOM_SEQUENCES,
        "command_storage" => refs::SAVED_DATA_COMMAND_STORAGE,
        "maps/last_id" | "idcounts" => refs::SAVED_DATA_MAP_INDEX,
        other if other == "raids" || other.starts_with("raids_") => refs::SAVED_DATA_RAIDS,
        other if other.starts_with("maps/") || other.starts_with("map_") => {
            refs::SAVED_DATA_MAP_DATA
        }
        _ => return None,
    };
    Some(reference)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafix::dynamic::{compound, get, get_i32_or, get_str};
    use crate::datafix::fixer::DataFixerBuilder;
    use crate::datafix::fixes::item_id_fix;
    use crate::datafix::schemas::{v102, v99};

    /// A two-schema fixer (99 -> 102 with `ItemIdFix`) that claims to reach the
    /// current version, to exercise the upgrade path of the loader glue.
    fn item_fixer() -> DataFixer {
        let mut builder = DataFixerBuilder::new();
        builder.add_schema(99, 0, v99::build);
        let v102 = builder.add_schema(102, 0, v102::build);
        builder.add_fixer(v102, true, item_id_fix::fix());
        builder.build(TARGET_DATA_VERSION)
    }

    fn old_player() -> Tag {
        compound(vec![
            ("DataVersion", Tag::Int(99)),
            (
                "Inventory",
                Tag::List(vec![compound(vec![
                    ("id", Tag::Short(5)),
                    ("Count", Tag::Byte(1)),
                ])]),
            ),
        ])
    }

    #[test]
    fn current_and_newer_tags_are_returned_unchanged() {
        for version in [TARGET_DATA_VERSION, TARGET_DATA_VERSION + 1] {
            let tag = compound(vec![("DataVersion", Tag::Int(version))]);
            assert_eq!(
                upgrade_saved_tag("playerdata", refs::PLAYER, tag.clone()).unwrap(),
                tag
            );
        }
    }

    #[test]
    fn tags_without_data_version_are_rejected() {
        let error = upgrade_saved_tag("playerdata", refs::PLAYER, compound(vec![])).unwrap_err();
        assert_eq!(error, "playerdata missing DataVersion");
    }

    #[test]
    fn old_tags_are_refused_while_the_chain_is_incomplete() {
        assert!(!chain_reaches_current_version());
        let error = upgrade_saved_tag("playerdata", refs::PLAYER, old_player()).unwrap_err();
        assert!(
            error.contains("Unsupported playerdata DataVersion 99"),
            "{error}"
        );
    }

    #[test]
    fn old_tags_are_upgraded_and_stamped_when_the_chain_reaches_current() {
        let fixer = item_fixer();
        let upgraded =
            upgrade_saved_tag_with(&fixer, "playerdata", refs::PLAYER, old_player()).unwrap();
        assert_eq!(get_i32_or(&upgraded, "DataVersion", 0), TARGET_DATA_VERSION);
        let Some(Tag::List(items)) = get(&upgraded, "Inventory") else {
            panic!("inventory missing");
        };
        assert_eq!(get_str(&items[0], "id"), Some("minecraft:planks"));
    }

    #[test]
    fn borrowed_tags_at_the_current_version_are_not_copied() {
        let tag = compound(vec![("DataVersion", Tag::Int(TARGET_DATA_VERSION))]);
        let upgraded = upgraded_saved_tag("chunk", refs::CHUNK, &tag).unwrap();
        assert!(matches!(upgraded, Cow::Borrowed(_)));
    }

    #[test]
    fn level_dat_versions_follow_the_chain() {
        assert!(check_level_data_version(TARGET_DATA_VERSION).is_ok());
        assert!(check_level_data_version(TARGET_DATA_VERSION + 1).is_err());
        assert!(check_level_data_version(1343).is_err());
    }

    #[test]
    fn saved_data_files_map_to_their_fix_types() {
        assert_eq!(
            saved_data_fix_type("scoreboard"),
            Some(refs::SAVED_DATA_SCOREBOARD)
        );
        assert_eq!(
            saved_data_fix_type("command_storage"),
            Some(refs::SAVED_DATA_COMMAND_STORAGE)
        );
        assert_eq!(
            saved_data_fix_type("maps/5"),
            Some(refs::SAVED_DATA_MAP_DATA)
        );
        assert_eq!(
            saved_data_fix_type("maps/last_id"),
            Some(refs::SAVED_DATA_MAP_INDEX)
        );
        assert_eq!(
            saved_data_fix_type("raids_end"),
            Some(refs::SAVED_DATA_RAIDS)
        );
        assert_eq!(saved_data_fix_type("mods/custom"), None);
    }
}
