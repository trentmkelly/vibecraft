pub const TARGET_DATA_VERSION: i32 = 4790;

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

#[cfg(test)]
mod tests {
    use super::{
        check_world_data_version, plan_world_upgrade, require_current_world_data_version,
        DataFixDecision, WorldUpgradeOptions, WorldUpgradeStep, TARGET_DATA_VERSION,
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
}
