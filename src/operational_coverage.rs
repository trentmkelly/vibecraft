#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationalFileSurface {
    pub path: &'static str,
    pub evidence: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedReportSurface {
    pub name: &'static str,
    pub evidence: &'static str,
}

pub const OPERATIONAL_FILE_SURFACES: &[OperationalFileSurface] = &[
    file(
        "eula.txt",
        "eula::Eula load_or_create and main init/refusal tests",
    ),
    file(
        "server.properties",
        "server_properties typed/default/preserve/roundtrip tests",
    ),
    file(
        "ops.json",
        "player_access operator permissions and hot-reload harness coverage",
    ),
    file(
        "whitelist.json",
        "player_access whitelist and auth hot-edit harness coverage",
    ),
    file(
        "banned-players.json",
        "player_access ban/pardon and auth hot-edit harness coverage",
    ),
    file(
        "banned-ips.json",
        "player_access IP ban/pardon and auth hot-edit harness coverage",
    ),
    file(
        "usercache.json",
        "player_online_auth/usercache scenarios and restart coverage",
    ),
    file(
        "session.lock",
        "storage::world WorldLayout session lock path",
    ),
    file("level.dat", "storage::world WorldLayout level data path"),
    file(
        "level.dat_old",
        "crash_recovery_tests level.dat backup recovery coverage",
    ),
    file(
        "region files",
        "storage::region and storage::chunk roundtrip tests",
    ),
    file(
        "entity region files",
        "storage::entities entity region path and NBT coverage",
    ),
    file("POI files", "storage::poi roundtrip tests"),
    file(
        "playerdata",
        "PlayerEntityState save/load and WorldLayout playerdata paths",
    ),
    file(
        "advancements",
        "PlayerAdvancementSet vanilla JSON persistence tests",
    ),
    file("stats", "StatisticsCounter vanilla JSON persistence tests"),
    file(
        "icon",
        "status/favicon resource and server-list metadata coverage",
    ),
    file("crash reports", "crash::CrashReport and panic-hook tests"),
    file("logs", "log::Logger latest.log and rotation tests"),
    file(
        "debug output",
        "runtime profiler/debug sample and debug command tests",
    ),
    file(
        "generated reports",
        "configuration/worldgen report harnesses and generated report manifest",
    ),
];

pub const GENERATED_REPORT_SURFACES: &[GeneratedReportSurface] = &[
    report(
        "registries",
        "configuration_registry_closure_report and raw registry readiness gates",
    ),
    report(
        "tags",
        "configuration registry readiness required tag packet checks",
    ),
    report("commands", "command_tree and command parity report tests"),
    report(
        "packs",
        "datapack reload and pack repository configuration tests",
    ),
    report(
        "worldgen definitions",
        "worldgen_resources and vanilla_worldgen_fixtures report tests",
    ),
];

const fn file(path: &'static str, evidence: &'static str) -> OperationalFileSurface {
    OperationalFileSurface { path, evidence }
}

const fn report(name: &'static str, evidence: &'static str) -> GeneratedReportSurface {
    GeneratedReportSurface { name, evidence }
}

#[cfg(test)]
mod tests {
    use super::{GENERATED_REPORT_SURFACES, OPERATIONAL_FILE_SURFACES};
    use std::collections::BTreeSet;

    #[test]
    fn operational_file_manifest_covers_vanilla_dedicated_server_surfaces() {
        let names = OPERATIONAL_FILE_SURFACES
            .iter()
            .map(|surface| surface.path)
            .collect::<BTreeSet<_>>();

        for required in [
            "eula.txt",
            "server.properties",
            "ops.json",
            "whitelist.json",
            "banned-players.json",
            "banned-ips.json",
            "usercache.json",
            "session.lock",
            "level.dat",
            "level.dat_old",
            "region files",
            "entity region files",
            "POI files",
            "playerdata",
            "advancements",
            "stats",
            "icon",
            "crash reports",
            "logs",
            "debug output",
            "generated reports",
        ] {
            assert!(names.contains(required), "missing {required}");
        }
        assert_eq!(names.len(), OPERATIONAL_FILE_SURFACES.len());
        assert!(OPERATIONAL_FILE_SURFACES
            .iter()
            .all(|surface| !surface.evidence.is_empty()));
    }

    #[test]
    fn generated_report_manifest_covers_comparison_inputs() {
        let names = GENERATED_REPORT_SURFACES
            .iter()
            .map(|surface| surface.name)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            names,
            BTreeSet::from([
                "commands",
                "packs",
                "registries",
                "tags",
                "worldgen definitions",
            ])
        );
        assert!(GENERATED_REPORT_SURFACES
            .iter()
            .all(|surface| !surface.evidence.is_empty()));
    }
}
