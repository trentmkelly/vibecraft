#![cfg(test)]

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Component, Path, PathBuf};
    use std::process::Command;

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

    fn collect_code_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(root)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
        {
            let entry = entry.unwrap_or_else(|err| {
                panic!("failed to read directory entry in {}: {err}", root.display())
            });
            let path = entry.path();
            if path.is_dir() {
                if matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("node_modules" | "target" | ".git" | "artifacts")
                ) {
                    continue;
                }
                collect_code_files(&path, files);
                continue;
            }
            if let Some("java" | "js" | "mjs" | "py" | "rs" | "sh" | "toml" | "yaml" | "yml") =
                path.extension().and_then(|ext| ext.to_str())
            {
                files.push(path);
            }
        }
    }

    fn normalize_repo_path(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    normalized.pop();
                }
                other => normalized.push(other.as_os_str()),
            }
        }
        normalized
    }

    fn extract_quoted_literal(text: &str) -> Option<&str> {
        let start = text.find('"')? + 1;
        let rest = &text[start..];
        let end = rest.find('"')?;
        Some(&rest[..end])
    }

    fn include_macro_targets(contents: &str) -> Vec<(&str, &str)> {
        let mut targets = Vec::new();
        for macro_name in ["include_str!(", "include_bytes!("] {
            let mut remaining = contents;
            while let Some(index) = remaining.find(macro_name) {
                let after_macro = &remaining[index + macro_name.len()..];
                remaining = after_macro;
                let Some(end) = after_macro.find(')') else {
                    break;
                };
                let invocation = &after_macro[..end];
                if invocation.contains("env!(") || invocation.contains("option_env!(") {
                    continue;
                }
                if let Some(target) = extract_quoted_literal(invocation) {
                    if !target.is_empty()
                        && target.trim() == target
                        && !target.contains(',')
                        && !target.contains('\n')
                    {
                        targets.push((macro_name.trim_end_matches('('), target));
                    }
                }
            }
        }
        targets
    }

    fn cargo_manifest_include_targets(contents: &str) -> Vec<&str> {
        let mut targets = Vec::new();
        let mut remaining = contents;
        let needle = "env!(\"CARGO_MANIFEST_DIR\")";
        while let Some(index) = remaining.find(needle) {
            let after_env = &remaining[index + needle.len()..];
            remaining = after_env;
            if !after_env.trim_start().starts_with(',') {
                continue;
            }
            if let Some(target) = extract_quoted_literal(after_env) {
                targets.push(target);
            }
        }
        targets
    }

    fn assert_tracked_by_git(manifest_dir: &Path, path: &Path) {
        let relative = path.strip_prefix(manifest_dir).unwrap_or_else(|err| {
            panic!(
                "compile-time include target {} is outside repo {}: {err}",
                path.display(),
                manifest_dir.display()
            )
        });
        let output = Command::new("git")
            .arg("ls-files")
            .arg("--error-unmatch")
            .arg(relative)
            .current_dir(manifest_dir)
            .output()
            .unwrap_or_else(|err| panic!("failed to run git ls-files: {err}"));
        assert!(
            output.status.success(),
            "compile-time include target {} must be tracked by git so fresh clones include it",
            relative.display()
        );
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
    fn code_does_not_hardcode_decompiled_data_roots() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let developer_home = ["/home", "trent"].join("/");
        let relative_decompiled_root = ["..", "decompiled-server-26.1.2"].join("/");
        let relative_parent_server_jar = ["..", "server.jar"].join("/");
        let bare_server_jar_command = ["java -jar", "server.jar"].join(" ");
        let workspace_server_jar_default = "workspace.join(\"server.jar\")";
        let mut files = vec![manifest_dir.join("build.rs")];
        collect_code_files(&manifest_dir.join("src"), &mut files);
        collect_code_files(&manifest_dir.join("harness").join("mineflayer"), &mut files);
        collect_code_files(&manifest_dir.join("tools"), &mut files);
        collect_code_files(&manifest_dir.join(".github").join("workflows"), &mut files);

        for path in files {
            let contents = fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("failed to read {}: {err}", path.display());
            });
            assert!(
                !contents.contains(&developer_home),
                "{} must not hard-code a developer-local home path",
                path.display()
            );
            assert!(
                !contents.contains(&relative_decompiled_root),
                "{} must use VIBECRAFT_DECOMPILED_SOURCE_ROOT or vendored data instead of a relative decompiled source path",
                path.display()
            );
            assert!(
                !contents.contains(&["", "..", "decompiled-server-26.1.2"].join("/")),
                "{} must not embed parent-directory decompiled source paths",
                path.display()
            );
            assert!(
                !contents.contains(&relative_parent_server_jar),
                "{} must use VIBECRAFT_OFFICIAL_SERVER_JAR or an explicit option instead of a parent-directory server.jar",
                path.display()
            );
            assert!(
                !contents.contains(&bare_server_jar_command),
                "{} must use VIBECRAFT_OFFICIAL_SERVER_JAR or an explicit option instead of assuming an untracked server.jar in the working directory",
                path.display()
            );
            assert!(
                !contents.contains(workspace_server_jar_default),
                "{} must not default official-server parity runs to an untracked workspace server.jar",
                path.display()
            );
        }
    }

    #[test]
    fn compile_time_file_includes_are_in_repo_and_present() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let normalized_manifest_dir = normalize_repo_path(manifest_dir);
        let mut files = Vec::new();
        collect_code_files(&manifest_dir.join("src"), &mut files);
        collect_code_files(&manifest_dir.join("harness").join("mineflayer"), &mut files);
        collect_code_files(&manifest_dir.join("tools"), &mut files);
        collect_code_files(&manifest_dir.join(".github").join("workflows"), &mut files);

        for path in files {
            let contents = fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("failed to read {}: {err}", path.display());
            });

            for (macro_name, target) in include_macro_targets(&contents) {
                let resolved = normalize_repo_path(&path.parent().unwrap_or(manifest_dir).join(target));
                assert!(
                    resolved.starts_with(&normalized_manifest_dir),
                    "{} includes out-of-repo file {target:?} through {macro_name}",
                    path.display()
                );
                assert!(
                    resolved.is_file(),
                    "{} includes missing file {} through {macro_name}",
                    path.display(),
                    resolved.display()
                );
                assert_tracked_by_git(manifest_dir, &resolved);
            }

            for target in cargo_manifest_include_targets(&contents) {
                let resolved = normalize_repo_path(&manifest_dir.join(target.trim_start_matches('/')));
                assert!(
                    resolved.starts_with(&normalized_manifest_dir),
                    "{} builds an out-of-repo CARGO_MANIFEST_DIR include target {target:?}",
                    path.display()
                );
                assert!(
                    resolved.is_file(),
                    "{} builds a missing CARGO_MANIFEST_DIR include target {}",
                    path.display(),
                    resolved.display()
                );
                assert_tracked_by_git(manifest_dir, &resolved);
            }
        }
    }

    #[test]
    fn generated_block_protocol_assets_are_vendored_in_repo() {
        for parts in [
            &["vanilla-data", "reports", "blocks_26_1_2.json"][..],
            &["vanilla-data", "reports", "block_registry_26_1_2.json"][..],
            &["vanilla-data", "reports", "block_properties_26_1_2.json.gz"][..],
            &["vanilla-data", "reports", "light_occlusion_26_1_2.json.gz"][..],
            &["src", "block_states", "state_data_a.rs"][..],
            &["src", "block_states", "state_data_b.rs"][..],
            &["src", "block_states", "state_data_c.rs"][..],
            &["src", "block_states", "state_data_d.rs"][..],
            &["src", "block_states", "state_data_e.rs"][..],
            &["src", "block_states", "state_data_f.rs"][..],
            &["src", "block_states", "state_data_g.rs"][..],
            &["src", "block_states", "state_data_h.rs"][..],
            &["src", "block_states", "state_data_i.rs"][..],
        ] {
            let path = parts
                .iter()
                .fold(Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf(), |path, part| {
                    path.join(part)
                });
            let metadata = fs::metadata(&path).unwrap_or_else(|err| {
                panic!(
                    "fresh clones must include generated block protocol asset {}: {err}",
                    path.display()
                );
            });
            assert!(
                metadata.is_file() && metadata.len() > 0,
                "generated block protocol asset {} must be a non-empty file",
                path.display()
            );
            assert_tracked_by_git(Path::new(env!("CARGO_MANIFEST_DIR")), &path);
        }

        assert_eq!(crate::block_states::VANILLA_BLOCK_STATE_COUNT_26_1_2, 29_873);
        assert_eq!(crate::block_states::block_state_entries().len(), 1_168);
        assert_eq!(
            crate::block_states::default_state_network_id("minecraft:stone"),
            Some(1)
        );
        assert_eq!(
            crate::block_states::default_state_network_id("minecraft:smooth_stone"),
            Some(13_480)
        );
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
