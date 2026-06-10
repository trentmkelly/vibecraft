pub const TARGET_DATA_VERSION: i32 = 4790;

use std::fs;
use std::io;
use std::path::Path;

use super::chunk::LevelChunk;
use super::entities::ChunkEntities;
use super::nbt::Tag;
use super::region::{ChunkPos, RegionFile, RegionPos, CHUNKS_PER_REGION_AXIS};
use super::world::WorldLayout;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataFixDecision {
    Current,
    Blocked {
        found_data_version: i32,
        target_data_version: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldUpgradeOptions {
    pub force_upgrade: bool,
    pub erase_cache: bool,
    pub recreate_region_files: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldUpgradeStep {
    ScanWorld,
    ValidateDataVersion,
    EraseCachedData,
    RecreateRegionFiles,
    RewriteUpgradedChunks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldUpgradeReport {
    pub decision: DataFixDecision,
    pub steps: Vec<WorldUpgradeStep>,
    pub safe_to_load: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldUpgradeRewriteReport {
    pub plan: WorldUpgradeReport,
    pub chunk_count: usize,
    pub entity_chunk_count: usize,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFixStrategyAction {
    NativeCurrentVersionRewrite,
    ExternalDfuRequired,
    VersionStampedJsonValidation,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFixStrategyFamily {
    pub family: &'static str,
    pub covered_surfaces: &'static [&'static str],
    pub action: DataFixStrategyAction,
}

#[cfg(test)]
pub const DATAFIX_STRATEGY: &[DataFixStrategyFamily] = &[
    DataFixStrategyFamily {
        family: "block_id_renames_and_flattening",
        covered_surfaces: &["level_chunk.block_states", "level_chunk.palette"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "block_entity_renames_and_fields",
        covered_surfaces: &["level_chunk.block_entities", "data/*.dat"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "entity_renames_and_fields",
        covered_surfaces: &["entities/r.*.mca"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "item_renames_and_stack_flattening",
        covered_surfaces: &["playerdata", "entities", "block_entities", "containers"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "chunk_format_upgrades",
        covered_surfaces: &["region/r.*.mca", "entities/r.*.mca"],
        action: DataFixStrategyAction::NativeCurrentVersionRewrite,
    },
    DataFixStrategyFamily {
        family: "poi_creation",
        covered_surfaces: &["poi/r.*.mca", "region/r.*.mca"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "advancements_and_stats",
        covered_surfaces: &["advancements/*.json", "stats/*.json"],
        action: DataFixStrategyAction::VersionStampedJsonValidation,
    },
    DataFixStrategyFamily {
        family: "scoreboards_and_options",
        covered_surfaces: &["data/scoreboard.dat", "options.txt"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "structures_and_text_components",
        covered_surfaces: &["generated/**/*.nbt", "text_components"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "villager_data",
        covered_surfaces: &["entities", "playerdata.trades"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "worldgen_settings",
        covered_surfaces: &["level.dat.WorldGenSettings"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
    DataFixStrategyFamily {
        family: "versioned_registry_renames",
        covered_surfaces: &["level.dat", "chunks", "saved_data"],
        action: DataFixStrategyAction::ExternalDfuRequired,
    },
];

impl WorldUpgradeReport {
    pub fn refusal_message(&self) -> Option<String> {
        match self.decision {
            DataFixDecision::Current => None,
            DataFixDecision::Blocked {
                found_data_version,
                target_data_version,
            } => Some(format!(
                "Unsupported world DataVersion {found_data_version}; RustCraft supports {target_data_version}. Use external DataFixerUpper-compatible tooling before loading this world."
            )),
        }
    }
}

pub fn check_world_data_version(found_data_version: i32) -> DataFixDecision {
    if found_data_version == TARGET_DATA_VERSION {
        DataFixDecision::Current
    } else {
        DataFixDecision::Blocked {
            found_data_version,
            target_data_version: TARGET_DATA_VERSION,
        }
    }
}

pub fn require_current_world_data_version(found_data_version: i32) -> Result<(), String> {
    match check_world_data_version(found_data_version) {
        DataFixDecision::Current => Ok(()),
        DataFixDecision::Blocked {
            found_data_version,
            target_data_version,
        } => Err(format!(
            "Unsupported world DataVersion {found_data_version}; RustCraft currently supports only {target_data_version} and will not perform unsafe migrations"
        )),
    }
}

pub fn tag_data_version(tag: &Tag) -> Option<i32> {
    let Tag::Compound(values) = tag else {
        return None;
    };
    values
        .iter()
        .find_map(|(name, value)| match (name.as_str(), value) {
            ("DataVersion", Tag::Int(version)) => Some(*version),
            _ => None,
        })
}

/// Acceptance policy for a saved-data tag (playerdata, chunk, entity chunk, map,
/// and other `SavedData` surfaces). Mirrors Java `DataFixTypes.updateToCurrentVersion`,
/// which calls `DataFixerUpper.update(type, data, fromVersion, currentVersion)`:
/// when `fromVersion >= currentVersion` the data is returned UNCHANGED (DFU never
/// downgrades), so a tag at or newer than [`TARGET_DATA_VERSION`] loads as-is.
/// Only older tags require datafix upgrades, which RustCraft has not implemented
/// yet, so those are blocked.
///
/// This differs from [`check_world_data_version`] (exact match), which gates the
/// `level.dat` world version where a newer save is deliberately refused.
pub fn check_saved_tag_data_version(found_data_version: i32) -> DataFixDecision {
    if found_data_version >= TARGET_DATA_VERSION {
        DataFixDecision::Current
    } else {
        DataFixDecision::Blocked {
            found_data_version,
            target_data_version: TARGET_DATA_VERSION,
        }
    }
}

pub fn require_current_tag_data_version(surface: &str, tag: &Tag) -> Result<(), String> {
    let version = tag_data_version(tag).ok_or_else(|| format!("{surface} missing DataVersion"))?;
    match check_saved_tag_data_version(version) {
        DataFixDecision::Current => Ok(()),
        DataFixDecision::Blocked {
            found_data_version,
            target_data_version,
        } => Err(format!(
            "Unsupported {surface} DataVersion {found_data_version}; RustCraft supports {target_data_version} or newer and will not perform unsafe downgrade migrations"
        )),
    }
}

pub fn plan_world_upgrade(options: WorldUpgradeOptions) -> Vec<WorldUpgradeStep> {
    if !options.force_upgrade && !options.recreate_region_files {
        return Vec::new();
    }

    let mut steps = vec![
        WorldUpgradeStep::ScanWorld,
        WorldUpgradeStep::ValidateDataVersion,
    ];
    if options.erase_cache {
        steps.push(WorldUpgradeStep::EraseCachedData);
    }
    if options.recreate_region_files {
        steps.push(WorldUpgradeStep::RecreateRegionFiles);
    }
    if options.force_upgrade {
        steps.push(WorldUpgradeStep::RewriteUpgradedChunks);
    }
    steps
}

pub fn plan_compatible_world_upgrade(
    found_data_version: i32,
    options: WorldUpgradeOptions,
) -> WorldUpgradeReport {
    let decision = check_world_data_version(found_data_version);
    let safe_to_load = matches!(decision, DataFixDecision::Current);
    let steps = if safe_to_load {
        plan_world_upgrade(options)
    } else {
        vec![
            WorldUpgradeStep::ScanWorld,
            WorldUpgradeStep::ValidateDataVersion,
        ]
    };
    WorldUpgradeReport {
        decision,
        steps,
        safe_to_load,
    }
}

pub fn run_world_upgrade(
    layout: &WorldLayout,
    found_data_version: i32,
    options: WorldUpgradeOptions,
) -> Result<WorldUpgradeRewriteReport, String> {
    let plan = plan_compatible_world_upgrade(found_data_version, options.clone());
    if !plan.safe_to_load {
        return Err(plan
            .refusal_message()
            .unwrap_or_else(|| "Unsupported world DataVersion".to_string()));
    }
    let mut report = WorldUpgradeRewriteReport {
        plan,
        chunk_count: 0,
        entity_chunk_count: 0,
    };
    if options.erase_cache {
        erase_known_cache_dirs(layout)
            .map_err(|err| format!("Failed to erase world cache: {err}"))?;
    }
    if options.force_upgrade {
        report.chunk_count = rewrite_region_directory(&layout.region_dir(), rewrite_level_chunk)
            .map_err(|err| format!("Failed to rewrite chunk regions: {err}"))?;
        report.entity_chunk_count =
            rewrite_region_directory(&layout.entities_dir(), rewrite_entity_chunk)
                .map_err(|err| format!("Failed to rewrite entity regions: {err}"))?;
    }
    Ok(report)
}

fn erase_known_cache_dirs(layout: &WorldLayout) -> io::Result<()> {
    for relative in ["cache", "data/caches"] {
        let path = layout.root().join(relative);
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

fn rewrite_region_directory(
    dir: &Path,
    rewrite: fn(ChunkPos, Tag) -> Result<Tag, String>,
) -> io::Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let mut rewritten = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("mca") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(region_pos) = parse_region_file_name(file_name) else {
            continue;
        };
        let region = RegionFile::open(dir, region_pos)?;
        for local_z in 0..CHUNKS_PER_REGION_AXIS {
            for local_x in 0..CHUNKS_PER_REGION_AXIS {
                let pos = ChunkPos {
                    x: region_pos.x * CHUNKS_PER_REGION_AXIS + local_x,
                    z: region_pos.z * CHUNKS_PER_REGION_AXIS + local_z,
                };
                let Some((name, tag)) = region.read_chunk_nbt(pos)? else {
                    continue;
                };
                let rewritten_tag = rewrite(pos, tag).map_err(|err| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("{file_name}: {err}"))
                })?;
                region.write_chunk_nbt(pos, &name, &rewritten_tag)?;
                rewritten += 1;
            }
        }
    }
    Ok(rewritten)
}

fn rewrite_level_chunk(pos: ChunkPos, tag: Tag) -> Result<Tag, String> {
    let chunk = LevelChunk::from_nbt(pos, &tag)?;
    Ok(chunk.to_nbt(TARGET_DATA_VERSION))
}

fn rewrite_entity_chunk(pos: ChunkPos, tag: Tag) -> Result<Tag, String> {
    let chunk = ChunkEntities::from_nbt(pos, &tag)?;
    Ok(chunk.to_nbt(TARGET_DATA_VERSION))
}

fn parse_region_file_name(file_name: &str) -> Option<RegionPos> {
    let rest = file_name.strip_prefix("r.")?.strip_suffix(".mca")?;
    let (x, z) = rest.split_once('.')?;
    Some(RegionPos {
        x: x.parse().ok()?,
        z: z.parse().ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        check_saved_tag_data_version, check_world_data_version, plan_compatible_world_upgrade,
        plan_world_upgrade, require_current_tag_data_version, require_current_world_data_version,
        run_world_upgrade, DataFixDecision, DataFixStrategyAction, WorldUpgradeOptions,
        WorldUpgradeStep, DATAFIX_STRATEGY, TARGET_DATA_VERSION,
    };
    use crate::storage::chunk::LevelChunk;
    use crate::storage::entities::ChunkEntities;
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;
    use crate::storage::world::WorldLayout;
    use std::fs;

    #[test]
    fn accepts_current_target_data_version() {
        assert_eq!(
            check_world_data_version(TARGET_DATA_VERSION),
            DataFixDecision::Current
        );
        assert!(require_current_world_data_version(TARGET_DATA_VERSION).is_ok());
    }

    #[test]
    fn blocks_unsupported_migrations() {
        let decision = check_world_data_version(TARGET_DATA_VERSION - 1);
        assert_eq!(
            decision,
            DataFixDecision::Blocked {
                found_data_version: TARGET_DATA_VERSION - 1,
                target_data_version: TARGET_DATA_VERSION,
            }
        );
        let err = require_current_world_data_version(TARGET_DATA_VERSION - 1).unwrap_err();
        assert!(err.contains("will not perform unsafe migrations"));
    }

    #[test]
    fn saved_tags_accept_current_or_newer_and_block_older() {
        // Java mirror: DataFixTypes.updateToCurrentVersion ->
        // DataFixerUpper.update(type, data, fromVersion, currentVersion) returns the
        // data UNCHANGED when fromVersion >= currentVersion (DFU never downgrades), so
        // saved data (playerdata, chunk, entity chunk, map, SavedData) at or newer
        // than the target loads as-is. Only older data needs (unimplemented) upgrades.
        assert_eq!(
            check_saved_tag_data_version(TARGET_DATA_VERSION),
            DataFixDecision::Current
        );
        assert_eq!(
            check_saved_tag_data_version(TARGET_DATA_VERSION + 1),
            DataFixDecision::Current
        );
        assert_eq!(
            check_saved_tag_data_version(TARGET_DATA_VERSION - 1),
            DataFixDecision::Blocked {
                found_data_version: TARGET_DATA_VERSION - 1,
                target_data_version: TARGET_DATA_VERSION,
            }
        );

        // A newer playerdata tag (e.g. a save written by a point release one version
        // ahead) loads without error, exactly like vanilla.
        let newer = Tag::Compound(vec![(
            "DataVersion".to_string(),
            Tag::Int(TARGET_DATA_VERSION + 1),
        )]);
        assert!(require_current_tag_data_version("playerdata", &newer).is_ok());

        let older = Tag::Compound(vec![(
            "DataVersion".to_string(),
            Tag::Int(TARGET_DATA_VERSION - 1),
        )]);
        let err = require_current_tag_data_version("playerdata", &older).unwrap_err();
        assert!(err.contains("Unsupported playerdata DataVersion"));
        assert!(err.contains("will not perform unsafe downgrade migrations"));
    }

    #[test]
    fn datafix_strategy_names_every_legacy_schema_family_and_blocks_unsafe_ones() {
        let families = DATAFIX_STRATEGY
            .iter()
            .map(|family| family.family)
            .collect::<Vec<_>>();
        for required in [
            "block_id_renames_and_flattening",
            "block_entity_renames_and_fields",
            "entity_renames_and_fields",
            "item_renames_and_stack_flattening",
            "chunk_format_upgrades",
            "poi_creation",
            "advancements_and_stats",
            "scoreboards_and_options",
            "structures_and_text_components",
            "villager_data",
            "worldgen_settings",
            "versioned_registry_renames",
        ] {
            assert!(families.contains(&required), "missing {required}");
        }
        assert!(DATAFIX_STRATEGY
            .iter()
            .all(|family| !family.covered_surfaces.is_empty()));
        assert!(DATAFIX_STRATEGY
            .iter()
            .any(|family| family.action == DataFixStrategyAction::NativeCurrentVersionRewrite));
        assert!(DATAFIX_STRATEGY
            .iter()
            .any(|family| family.action == DataFixStrategyAction::ExternalDfuRequired));
    }

    #[test]
    fn plans_force_upgrade_and_region_recreation_workflow() {
        assert_eq!(
            plan_world_upgrade(WorldUpgradeOptions {
                force_upgrade: false,
                erase_cache: true,
                recreate_region_files: false,
            }),
            Vec::<WorldUpgradeStep>::new()
        );
        assert_eq!(
            plan_world_upgrade(WorldUpgradeOptions {
                force_upgrade: true,
                erase_cache: true,
                recreate_region_files: true,
            }),
            vec![
                WorldUpgradeStep::ScanWorld,
                WorldUpgradeStep::ValidateDataVersion,
                WorldUpgradeStep::EraseCachedData,
                WorldUpgradeStep::RecreateRegionFiles,
                WorldUpgradeStep::RewriteUpgradedChunks,
            ]
        );
    }

    #[test]
    fn compatible_upgrade_report_allows_current_worlds_and_refuses_unsafe_migrations() {
        let current = plan_compatible_world_upgrade(
            TARGET_DATA_VERSION,
            WorldUpgradeOptions {
                force_upgrade: true,
                erase_cache: false,
                recreate_region_files: false,
            },
        );
        assert!(current.safe_to_load);
        assert_eq!(
            current.steps,
            vec![
                WorldUpgradeStep::ScanWorld,
                WorldUpgradeStep::ValidateDataVersion,
                WorldUpgradeStep::RewriteUpgradedChunks,
            ]
        );
        assert_eq!(current.refusal_message(), None);

        let old = plan_compatible_world_upgrade(
            TARGET_DATA_VERSION - 10,
            WorldUpgradeOptions {
                force_upgrade: true,
                erase_cache: true,
                recreate_region_files: true,
            },
        );
        assert!(!old.safe_to_load);
        assert_eq!(
            old.steps,
            vec![
                WorldUpgradeStep::ScanWorld,
                WorldUpgradeStep::ValidateDataVersion,
            ]
        );
        assert!(old
            .refusal_message()
            .unwrap()
            .contains("external DataFixerUpper-compatible tooling"));
    }

    #[test]
    fn vanilla_1_20_upgrade_parity_plan_refuses_without_external_dfu() {
        let vanilla_1_20_6_data_version = 3839;
        let report = plan_compatible_world_upgrade(
            vanilla_1_20_6_data_version,
            WorldUpgradeOptions {
                force_upgrade: true,
                erase_cache: false,
                recreate_region_files: true,
            },
        );
        assert!(!report.safe_to_load);
        assert_eq!(
            report.steps,
            vec![
                WorldUpgradeStep::ScanWorld,
                WorldUpgradeStep::ValidateDataVersion,
            ]
        );
        assert!(report
            .refusal_message()
            .unwrap()
            .contains("external DataFixerUpper-compatible tooling"));
    }

    #[test]
    fn force_upgrade_rewrites_current_chunk_and_entity_regions() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-force-upgrade-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let pos = ChunkPos { x: 2, z: -3 };
        let mut chunk = LevelChunk::empty(pos);
        chunk.status = "minecraft:full".to_string();
        layout
            .save_entity_region_chunk(pos, &ChunkEntities::empty(pos).to_nbt(TARGET_DATA_VERSION))
            .unwrap();
        crate::storage::region::RegionFile::open(&layout.region_dir(), pos.region())
            .unwrap()
            .write_chunk_nbt(pos, "", &chunk.to_nbt(TARGET_DATA_VERSION))
            .unwrap();
        fs::create_dir_all(layout.root().join("cache")).unwrap();
        fs::write(layout.root().join("cache").join("stale.bin"), b"stale").unwrap();

        let report = run_world_upgrade(
            &layout,
            TARGET_DATA_VERSION,
            WorldUpgradeOptions {
                force_upgrade: true,
                erase_cache: true,
                recreate_region_files: false,
            },
        )
        .unwrap();

        assert_eq!(report.chunk_count, 1);
        assert_eq!(report.entity_chunk_count, 1);
        assert!(!layout.root().join("cache").exists());
        assert_eq!(
            LevelChunk::from_nbt(
                pos,
                &crate::storage::region::RegionFile::open(&layout.region_dir(), pos.region())
                    .unwrap()
                    .read_chunk_nbt(pos)
                    .unwrap()
                    .unwrap()
                    .1
            )
            .unwrap()
            .status,
            "minecraft:full"
        );

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn force_upgrade_refuses_unsupported_world_versions_before_rewrite() {
        let layout = WorldLayout::new(std::env::temp_dir().join(format!(
            "rustcraft-force-upgrade-refuse-{}",
            std::process::id()
        )));
        let err = run_world_upgrade(
            &layout,
            TARGET_DATA_VERSION - 1,
            WorldUpgradeOptions {
                force_upgrade: true,
                erase_cache: false,
                recreate_region_files: false,
            },
        )
        .unwrap_err();
        assert!(err.contains("Unsupported world DataVersion"));
    }

    #[test]
    fn erase_cache_removes_only_cache_directories_without_world_content_loss() {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-erase-cache-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let layout = WorldLayout::new(&path);
        let level = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(TARGET_DATA_VERSION),
        )]);
        layout.save_level_dat(&level).unwrap();

        let pos = ChunkPos { x: 1, z: 1 };
        let chunk = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        crate::storage::region::RegionFile::open(&layout.region_dir(), pos.region())
            .unwrap()
            .write_chunk_nbt(pos, "", &chunk)
            .unwrap();
        layout
            .save_player_data(
                "00000000-0000-0000-0000-000000000006",
                &crate::storage::nbt::Tag::Compound(Vec::new()),
            )
            .unwrap();

        fs::create_dir_all(layout.root().join("cache")).unwrap();
        fs::write(layout.root().join("cache").join("biome.bin"), b"cache").unwrap();
        fs::create_dir_all(layout.root().join("data").join("caches")).unwrap();
        fs::write(
            layout.root().join("data").join("caches").join("noise.bin"),
            b"cache",
        )
        .unwrap();
        fs::write(
            layout.root().join("data").join("scoreboard.dat"),
            b"content",
        )
        .unwrap();

        let report = run_world_upgrade(
            &layout,
            TARGET_DATA_VERSION,
            WorldUpgradeOptions {
                force_upgrade: false,
                erase_cache: true,
                recreate_region_files: false,
            },
        )
        .unwrap();

        assert_eq!(report.chunk_count, 0);
        assert_eq!(report.entity_chunk_count, 0);
        assert!(!layout.root().join("cache").exists());
        assert!(!layout.root().join("data").join("caches").exists());
        assert_eq!(layout.load_level_dat().unwrap(), level);
        assert!(layout
            .playerdata_dir()
            .join("00000000-0000-0000-0000-000000000006.dat")
            .is_file());
        assert_eq!(
            fs::read(layout.root().join("data").join("scoreboard.dat")).unwrap(),
            b"content"
        );
        assert_eq!(
            crate::storage::region::RegionFile::open(&layout.region_dir(), pos.region())
                .unwrap()
                .read_chunk_nbt(pos)
                .unwrap()
                .unwrap()
                .1,
            chunk
        );

        let _ = fs::remove_dir_all(&path);
    }
}
