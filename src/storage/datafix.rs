pub const TARGET_DATA_VERSION: i32 = 4790;

use super::nbt::Tag;

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

pub fn require_current_tag_data_version(surface: &str, tag: &Tag) -> Result<(), String> {
    let version = tag_data_version(tag).ok_or_else(|| format!("{surface} missing DataVersion"))?;
    require_current_world_data_version(version)
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

#[cfg(test)]
mod tests {
    use super::{
        check_world_data_version, plan_compatible_world_upgrade, plan_world_upgrade,
        require_current_world_data_version, DataFixDecision, WorldUpgradeOptions, WorldUpgradeStep,
        TARGET_DATA_VERSION,
    };

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
}
