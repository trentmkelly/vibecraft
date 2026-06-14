#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    fn harness_file(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("harness")
            .join("mineflayer")
            .join(name);
        fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", path.display());
        })
    }

    fn doc_file(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join(name);
        fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", path.display());
        })
    }

    fn assert_contains_all(contents: &str, expected_fragments: &[&str]) {
        for fragment in expected_fragments {
            assert!(
                contents.contains(fragment),
                "expected file to contain {fragment:?}"
            );
        }
    }

    fn repo_file(parts: &[&str]) -> String {
        let path = parts
            .iter()
            .fold(Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf(), |path, part| {
                path.join(part)
            });
        fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", path.display());
        })
    }

    #[test]
    fn mineflayer_runner_targets_vibecraft_and_official_server() {
        let runner = harness_file("runner.mjs");

        assert!(runner.contains("startVibeCraft"));
        assert!(runner.contains("startOfficialServer"));
        assert!(runner.contains("runParityScenario"));
        assert!(runner.contains("serverKind: 'official'"));
        assert!(runner.contains("serverKind: 'vibecraft'"));
        assert!(runner.contains("normalizeArtifacts"));
        assert!(runner.contains("diffArtifacts"));
    }

    #[test]
    fn validation_sources_policy_names_each_allowed_evidence_family() {
        let policy = doc_file("VALIDATION_SOURCES.md");
        let package_lock = harness_file("package-lock.json");
        let protocol_manifest = harness_file("protocol_packet_manifest.mjs");
        let runner = harness_file("runner.mjs");
        let datapack = harness_file("datapack_scenarios.mjs");
        let raw_probe = harness_file("raw_26_1_2_join_probe.mjs");
        let generated_reports = repo_file(&["src", "generated_reports.rs"]);
        let vanilla_recipe = repo_file(&[
            "vanilla-data",
            "data",
            "minecraft",
            "recipe",
            "stick.json",
        ]);

        assert_contains_all(
            &policy,
            &[
                "Black-box tests",
                "Public protocol references",
                "Vanilla datapacks and resources",
                "Generated assets",
                "Observed behavior",
                "official `server.jar`",
            ],
        );
        assert!(runner.contains("runParityScenario"));
        assert!(package_lock.contains("node_modules/minecraft-data"));
        assert!(package_lock.contains("node_modules/prismarine-registry"));
        assert!(protocol_manifest.contains("createClientboundGoldenCoverage"));
        assert!(datapack.contains("datapack-reload"));
        assert!(raw_probe.contains("protocolVersion"));
        assert!(generated_reports.contains("registries.json"));
        assert!(generated_reports.contains("worldgen_chunks.json"));
        assert!(vanilla_recipe.contains("\"minecraft:stick\""));
    }

    #[test]
    fn offline_baseline_fixture_contract_is_shared_and_deterministic() {
        let runner = harness_file("runner.mjs");
        let fixtures = harness_file("fixtures.mjs");

        assert!(runner.contains("offlineUuid"));
        assert!(runner.contains("'online-mode': 'false'"));
        assert!(runner.contains("'enforce-secure-profile': 'false'"));
        assert!(runner.contains("'level-seed'"));
        assert!(runner.contains("'server-port'"));
        assert!(runner.contains("writeOfflineServerFiles"));

        assert!(fixtures.contains("DEFAULT_FIXTURE_SEED"));
        assert!(fixtures.contains("deterministicProfile"));
        assert!(fixtures.contains("profileSet"));
        assert!(fixtures.contains("createScenarioFixture"));
        assert!(fixtures.contains("readFixtureFiles"));
        assert!(fixtures.contains("writeFixtureManifest"));
    }

    #[test]
    fn offline_login_contract_and_preflight_gate_cover_readiness_and_play_entry() {
        let login_session = harness_file("login_session.mjs");
        let login_gate = harness_file("login_gate.mjs");
        let ci_shard = harness_file("ci_login_shard.mjs");

        assert!(login_session.contains("runObservedOfflineLogin"));
        assert!(login_session.contains("waitForPort"));
        assert!(login_session.contains("connectObservedOfflineBot"));
        assert!(login_session.contains("onceWithTimeout(bot, 'spawn'"));
        assert!(login_session.contains("profile:"));
        assert!(login_session.contains("expectedUuid"));
        assert!(login_session.contains("actualUuid"));

        assert!(login_gate.contains("runRequiredLoginGate"));
        assert!(login_gate.contains("shouldRunLoginGate"));
        assert!(login_gate.contains("runPacketFlowSmoke"));

        assert!(ci_shard.contains("tcpReadiness"));
        assert!(ci_shard.contains("configurationOrdering"));
        assert!(ci_shard.contains("firstSpawn"));
        assert!(ci_shard.contains("unexpectedDisconnect"));
    }

    #[test]
    fn live_runtime_loaders_do_not_hardcode_local_or_decompiled_paths() {
        let developer_home = ["/home", "trent"].join("/");
        let decompiled_parent_path = ["..", "decompiled-server-26.1.2"].join("/");
        for parts in [
            &["src", "worldgen", "surface_rule_runtime.rs"][..],
            &["src", "villager_trade_resources.rs"][..],
            &["src", "villager_system.rs"][..],
            &["harness", "mineflayer", "vanilla_client_xephyr.test.mjs"][..],
        ] {
            let contents = repo_file(parts);
            let path = parts.join("/");
            assert!(
                !contents.contains(&decompiled_parent_path),
                "{path} must not hard-code an out-of-repo decompiled data path"
            );
            assert!(
                !contents.contains(&developer_home),
                "{path} must not hard-code a developer-local home path"
            );
        }
    }

    #[test]
    fn login_artifacts_snapshots_and_diagnostics_are_captured_for_failures() {
        let runner = harness_file("runner.mjs");
        let login_session = harness_file("login_session.mjs");
        let debug_bundle = harness_file("debug_bundle.mjs");
        let normalizer = harness_file("artifact_normalizer.mjs");
        let spawn_diagnostics = harness_file("spawn_timeout_diagnostics.mjs");
        let quarantine = harness_file("quarantine_report.mjs");

        assert_contains_all(
            &runner,
            &[
                "collectArtifacts",
                "serverProperties",
                "eula",
                "events: [...events]",
                "logs: logs.map",
            ],
        );
        assert_contains_all(
            &login_session,
            &["packetTrace", "serverLogs", "endpoint", "server", "bot"],
        );
        assert_contains_all(
            &debug_bundle,
            &[
                "profile.json",
                "timeline.json",
                "packet-trace.json",
                "server.log",
                "normalized-artifacts.json",
                "parity-diff.json",
                "server.properties",
                "eula.txt",
            ],
        );
        assert_contains_all(
            &normalizer,
            &[
                "normalizeLoginArtifacts",
                "<run-dir>",
                "<port>",
                "<username>",
            ],
        );
        assert_contains_all(
            &spawn_diagnostics,
            &[
                "lastReceivedChunk",
                "entityId",
                "dimension",
                "position",
                "playPacketIds",
            ],
        );
        assert_contains_all(
            &quarantine,
            &[
                "rawDisconnectPackets",
                "classification",
                "prismarineDependencySnapshot",
            ],
        );
    }

    #[test]
    fn matrix_freshness_sanity_and_oracle_policy_are_explicit() {
        let profile_matrix = harness_file("profile_file_matrix.mjs");
        let transcript_wrapper = harness_file("transcript_scenario_wrapper.mjs");
        let fixtures = harness_file("fixtures.mjs");
        let runner = harness_file("runner.mjs");

        assert!(profile_matrix.contains("fresh-profile"));
        assert!(profile_matrix.contains("returning-profile"));
        assert!(profile_matrix.contains("banned-profile"));
        assert!(profile_matrix.contains("whitelisted-profile"));
        assert!(profile_matrix.contains("operator-profile"));
        assert!(profile_matrix.contains("profile-files-use-offline-uuid"));
        assert!(profile_matrix.contains("usercache"));
        assert!(profile_matrix.contains("playerdata"));

        assert!(fixtures.contains("offlineUuid"));
        assert!(fixtures.contains("deterministicProfile"));
        assert!(runner.contains("offlineUuid"));

        assert!(transcript_wrapper.contains("TARGET_MINECRAFT_VERSION = '26.1.2'"));
        assert!(transcript_wrapper.contains("TARGET_PROTOCOL_VERSION = 775"));
        assert!(transcript_wrapper.contains("prismarineSupportsTargetProtocol"));
        assert!(transcript_wrapper.contains("runner: 'raw-26.1.2'"));
        assert!(transcript_wrapper.contains("runner: 'mineflayer'"));
    }

    #[test]
    fn unmodified_vanilla_client_compatibility_oracle_is_documented_and_tested() {
        let compatibility = harness_file("vanilla_compatibility.mjs");
        let client_smoke = harness_file("vanilla_client_connection_smoke.mjs");
        let client_xephyr = harness_file("vanilla_client_xephyr.mjs");
        let docs = doc_file("COMPATIBILITY.md");

        assert!(compatibility.contains("vanilla-26.1.2-compatibility"));
        assert!(compatibility.contains("targetServer: 'minecraft_server.26.1.2'"));
        assert!(compatibility.contains("client-join"));
        assert!(compatibility.contains("survival-play"));
        assert!(compatibility.contains("death-respawn"));
        assert!(compatibility.contains("dimension-travel"));
        assert!(compatibility.contains("reconnecting"));

        assert!(client_smoke.contains("vanilla-client-xephyr-connection-smoke"));
        assert!(client_smoke.contains("wait for terrain to render"));
        assert!(client_smoke.contains("capture joined-world screenshot"));
        assert!(client_smoke.contains("collect latest.log and crash reports"));
        assert!(client_xephyr.contains("vanilla-client-xephyr-oracle"));
        assert!(client_xephyr
            .contains("no full argv logging because vanilla launch args contain auth material"));

        assert!(docs.contains("Target: Minecraft Java Edition 26.1.2."));
        assert!(docs.contains("does not affect vanilla client protocol compatibility"));
        assert!(docs.contains("Before any `CHECKLIST.md` item is checked"));
    }

    #[test]
    fn vanilla_datapack_and_resource_pack_compatibility_is_covered_by_resource_harnesses() {
        let datapack = harness_file("datapack_scenarios.mjs");
        let registry = harness_file("registry_scenarios.mjs");
        let fixture_linter = harness_file("fixture_linter.mjs");
        let docs = doc_file("COMPATIBILITY.md");

        assert!(datapack.contains("datapack-reload"));
        assert!(datapack.contains("feature-flag-datapack-mismatch"));
        assert!(datapack.contains("changed-datapack-registry-contents"));
        assert!(datapack.contains("vanilla-compatible-success-or-disconnect"));
        assert!(datapack.contains("disconnect-component-parity"));

        assert!(registry.contains("compares-registry-ids"));
        assert!(registry.contains("compares-tag-contents"));
        assert!(registry.contains("compares-known-packs"));
        assert!(registry.contains("compares-enabled-feature-order"));
        assert!(registry.contains("official-server-oracle"));

        assert!(fixture_linter.contains("vanillaComparison"));
        assert!(fixture_linter.contains("comparison mode must be required, optional, or disabled"));
        assert!(docs.contains("vanilla datapacks"));
        assert!(docs.contains("vanilla resources"));
    }
}
