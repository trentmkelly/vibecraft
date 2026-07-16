use super::*;
use crate::registry::feature_flags;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn optional_decompiled_minecraft_data_root() -> Option<PathBuf> {
    let root = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT")?;
    Some(Path::new(root).join("data").join("minecraft"))
}

fn collect_decompiled_minecraft_json_paths(root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    let mut dirs = vec![root.to_path_buf()];

    while let Some(current) = dirs.pop() {
        let read_dir = fs::read_dir(&current).expect("failed to read decompiled data dir");
        for entry in read_dir {
            let entry = entry.expect("failed to read decompiled data entry");
            let path = entry.path();

            if path.is_dir() {
                if current == root
                    && path.file_name().and_then(|name| name.to_str()) == Some("datapacks")
                {
                    continue;
                }
                dirs.push(path);
                continue;
            }

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let relative = path
                .strip_prefix(root)
                .expect("decompiled data path should be under data/minecraft");
            let relative_path = relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join("/");
            paths.push(format!("data/minecraft/{relative_path}"));
        }
    }

    paths
}

#[test]
fn decompiled_minecraft_data_resource_kinds_match_data_folders() {
    let Some(root) = optional_decompiled_minecraft_data_root() else {
        eprintln!("skipping decompiled data resource parity: optional Java source root unavailable");
        return;
    };
    let mut observed_top_levels = root
        .read_dir()
        .expect("failed to read decompiled data root")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(ToString::to_string))
        .filter(|name| name != "datapacks")
        .collect::<Vec<_>>();
    observed_top_levels.sort();

    let known_top_levels = DataResourceKind::ALL
        .iter()
        .map(|kind| kind.path_component().to_string())
        .collect::<std::collections::BTreeSet<_>>();

    for observed in observed_top_levels {
        assert!(
            known_top_levels.contains(&observed),
            "unknown top-level minecraft data folder in decompiled tree: {observed}"
        );
    }
}

#[test]
fn decompiled_minecraft_data_kind_counts_match_index() {
    let Some(root) = optional_decompiled_minecraft_data_root() else {
        eprintln!("skipping decompiled data kind count parity: optional Java source root unavailable");
        return;
    };
    let resource_paths = collect_decompiled_minecraft_json_paths(&root);
    let mut expected_counts = std::collections::BTreeMap::<DataResourceKind, usize>::new();
    for kind in DataResourceKind::ALL {
        expected_counts.insert(*kind, 0);
    }

    for path in &resource_paths {
        let top_level = path
            .split('/')
            .nth(2)
            .expect("data path should include top-level resource folder");
        if let Some(kind) = DataResourceKind::from_path_component(top_level) {
            *expected_counts.entry(kind).or_insert(0) += 1;
        }
    }

    let resources = resource_paths
        .iter()
        .map(|path| (path.as_str(), "{}"))
        .collect::<Vec<_>>();
    let index = DataResourceIndex::from_resources(resources).unwrap();

    for kind in DataResourceKind::ALL {
        let expected = expected_counts.get(kind).copied().unwrap_or(0);
        let observed = index.list("minecraft", *kind).len();
        assert_eq!(
            expected,
            observed,
            "resource kind {} expected count mismatch",
            kind.path_component()
        );
    }
}

#[test]
fn safe_mode_selects_only_vanilla_and_does_not_disable_world_packs() {
    let mut repository = DataPackRepository::new([
        DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
            .with_features(feature_flags::default_flags_26_1_2()),
        DataPack::new("world_pack", PackSource::World),
        DataPack::new("server_pack", PackSource::Server),
    ]);
    let initial = WorldDataConfiguration {
        data_packs: DataPackConfig {
            enabled: vec![
                VANILLA_PACK_ID.to_string(),
                "world_pack".to_string(),
                "server_pack".to_string(),
            ],
            disabled: Vec::new(),
        },
        enabled_features: feature_flags::default_flags_26_1_2(),
    };

    let configured = configure_pack_repository(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: true,
        },
    );

    assert_eq!(configured.data_packs.enabled, vec![VANILLA_PACK_ID]);
    assert!(configured.data_packs.disabled.is_empty());
    assert_eq!(repository.selected_ids(), vec![VANILLA_PACK_ID]);
}

#[test]
fn loads_vanilla_builtin_datapack_from_bundled_resources() {
    let pack = BuiltInDataPack::vanilla_26_1_2();

    assert_eq!(pack.id(), VANILLA_PACK_ID);
    assert_eq!(pack.metadata().description, "dataPack.vanilla.description");
    assert_eq!(
        pack.metadata().requested_features,
        feature_flags::default_flags_26_1_2()
    );
    assert!(pack.contains("pack.mcmeta"));
    assert!(pack.contains("data/minecraft/tags/block/replaceable.json"));
    assert_eq!(pack.namespaces(), vec!["minecraft"]);
    assert_eq!(
        pack.list_prefix("data/minecraft/tags/")
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            "data/minecraft/tags/block/replaceable.json",
            "data/minecraft/tags/item/logs.json"
        ]
    );

    let repository = DataPackRepository::server_repository(Path::new("missing-datapacks"))
        .expect("builtin repository should not require a world datapack dir");
    let vanilla = repository
        .available_packs()
        .into_iter()
        .find(|candidate| candidate.id == VANILLA_PACK_ID)
        .expect("vanilla pack should be present");
    assert_eq!(vanilla.source, PackSource::BuiltIn);
    assert_eq!(
        vanilla.requested_features,
        feature_flags::default_flags_26_1_2()
    );
}

#[test]
fn indexes_all_vanilla_data_resource_roots_by_kind() {
    let resources = DataResourceKind::ALL
        .iter()
        .map(|kind| {
            let id = if *kind == DataResourceKind::Tags {
                "block/example"
            } else {
                "example"
            };
            let path = format!("data/minecraft/{}/{id}.json", kind.path_component());
            (
                path,
                r#"{"loaded":true}"#.to_string(),
                *kind,
                id.to_string(),
            )
        })
        .collect::<Vec<_>>();
    let index = DataResourceIndex::from_resources(
        resources
            .iter()
            .map(|(path, contents, _kind, _id)| (path.as_str(), contents.as_str())),
    )
    .unwrap();

    for (_path, contents, kind, id) in resources {
        assert_eq!(
            index.get("minecraft", kind, &id),
            Some(contents.as_str()),
            "missing indexed resource kind {:?}",
            kind
        );
    }
}

#[test]
fn normal_mode_keeps_enabled_packs_and_disables_inactive_available_packs() {
    let mut repository = DataPackRepository::new([
        DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
            .with_features(feature_flags::default_flags_26_1_2()),
        DataPack::new("world_pack", PackSource::World),
        DataPack::new("disabled_pack", PackSource::World),
    ]);
    let initial = WorldDataConfiguration {
        data_packs: DataPackConfig {
            enabled: vec![VANILLA_PACK_ID.to_string(), "world_pack".to_string()],
            disabled: vec!["disabled_pack".to_string()],
        },
        enabled_features: feature_flags::default_flags_26_1_2(),
    };

    let configured = configure_pack_repository(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: false,
        },
    );

    assert_eq!(
        configured.data_packs.enabled,
        vec![VANILLA_PACK_ID, "world_pack"]
    );
    assert_eq!(configured.data_packs.disabled, vec!["disabled_pack"]);
}

#[test]
fn normal_mode_auto_adds_new_world_packs() {
    let mut repository = DataPackRepository::new([
        DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
            .with_features(feature_flags::default_flags_26_1_2()),
        DataPack::new("new_world_pack", PackSource::World),
    ]);
    let initial = WorldDataConfiguration::default_26_1_2();

    let configured = configure_pack_repository(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: false,
        },
    );

    assert_eq!(
        configured.data_packs.enabled,
        vec![VANILLA_PACK_ID, "new_world_pack"]
    );
    assert!(configured.data_packs.disabled.is_empty());
}

#[test]
fn forced_features_enable_matching_feature_pack() {
    let mut repository = DataPackRepository::new([
        DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
            .with_features(feature_flags::default_flags_26_1_2()),
        DataPack::new("feature/redstone", PackSource::Feature)
            .with_features(FeatureFlagSet::of(&[feature_flags::REDSTONE_EXPERIMENTS])),
    ]);
    let initial = WorldDataConfiguration {
        data_packs: DataPackConfig::default_26_1_2(),
        enabled_features: feature_flags::default_flags_26_1_2()
            .join(FeatureFlagSet::of(&[feature_flags::REDSTONE_EXPERIMENTS])),
    };

    let configured = configure_pack_repository(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: false,
        },
    );

    assert_eq!(
        configured.data_packs.enabled,
        vec![VANILLA_PACK_ID, "feature/redstone"]
    );
    assert!(configured
        .enabled_features
        .contains(feature_flags::REDSTONE_EXPERIMENTS));
}

#[test]
fn datapack_config_from_server_properties_matches_java_comma_splitter() {
    // Java DedicatedServerProperties uses Guava
    // Splitter.on(',').trimResults().splitToList: whitespace is trimmed but
    // empty tokens are preserved.
    assert_eq!(
        DataPackConfig::from_properties("vanilla,file/a,, file/b ", ""),
        DataPackConfig {
            enabled: vec![
                "vanilla".to_string(),
                "file/a".to_string(),
                String::new(),
                "file/b".to_string()
            ],
            disabled: vec![String::new()],
        }
    );
}

#[test]
fn datapack_config_codec_matches_java_record_shape_and_immutable_copy_boundary() {
    let enabled = vec!["vanilla".to_string(), "file/world".to_string()];
    let disabled = vec!["file/disabled".to_string()];
    let config = DataPackConfig::new(enabled.clone(), disabled.clone());

    assert_eq!(config.enabled(), enabled.as_slice());
    assert_eq!(config.disabled(), disabled.as_slice());
    assert_eq!(
        config.to_json().unwrap(),
        r#"{"Enabled":["vanilla","file/world"],"Disabled":["file/disabled"]}"#
    );
    assert_eq!(
        DataPackConfig::from_json(
            r#"{"Enabled":["vanilla","file/world"],"Disabled":["file/disabled"],"Unknown":true}"#
        )
        .unwrap(),
        config
    );
    assert!(DataPackConfig::from_json(r#"{"Enabled":[],"Disabled":[1]}"#).is_err());
    assert!(DataPackConfig::from_json(r#"{"Enabled":[]}"#).is_err());
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn datapack_config_source_matches_java_26_1_2_codec_contract() {
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/world/level/DataPackConfig.java");
    for fragment in [
        "public static final DataPackConfig DEFAULT",
        "Codec.STRING.listOf().fieldOf(\"Enabled\")",
        "Codec.STRING.listOf().fieldOf(\"Disabled\")",
        "this.enabled = ImmutableList.copyOf(enabled)",
        "this.disabled = ImmutableList.copyOf(disabled)",
        "public List<String> getEnabled()",
        "public List<String> getDisabled()",
    ] {
        assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
    }
}

#[test]
fn initial_datapack_property_lists_configure_repository_selection() {
    let mut repository = DataPackRepository::new([
        DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
            .with_features(feature_flags::default_flags_26_1_2()),
        DataPack::new("file/world", PackSource::World),
        DataPack::new("file/disabled", PackSource::World),
    ]);
    let initial = WorldDataConfiguration {
        data_packs: DataPackConfig::from_properties("vanilla,file/world", "file/disabled"),
        enabled_features: feature_flags::default_flags_26_1_2(),
    };

    let configured = configure_pack_repository(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: false,
        },
    );

    assert_eq!(configured.data_packs.enabled, vec![VANILLA_PACK_ID, "file/world"]);
    assert_eq!(configured.data_packs.disabled, vec!["file/disabled"]);
    assert_eq!(repository.selected_ids(), vec![VANILLA_PACK_ID, "file/world"]);
}

#[test]
fn pack_priority_enable_disable_and_reload_rollback_match_repository_rules() {
    let mut repository = DataPackRepository::new([
        DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
            .with_features(feature_flags::default_flags_26_1_2()),
        DataPack::new("low", PackSource::World),
        DataPack::new("high", PackSource::World),
    ]);

    repository.set_selected([VANILLA_PACK_ID, "low"]);
    assert!(repository.enable_pack_highest_priority("high"));
    assert_eq!(
        repository.priority_stack(),
        vec![VANILLA_PACK_ID, "low", "high"]
    );
    assert!(repository.enable_pack_highest_priority("low"));
    assert_eq!(
        repository.priority_stack(),
        vec![VANILLA_PACK_ID, "high", "low"]
    );
    assert!(repository.disable_pack("high"));
    assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "low"]);
    assert!(!repository.enable_pack_highest_priority("missing"));

    let initial = WorldDataConfiguration {
        data_packs: DataPackConfig {
            enabled: vec![VANILLA_PACK_ID.to_string(), "high".to_string()],
            disabled: vec!["low".to_string()],
        },
        enabled_features: feature_flags::default_flags_26_1_2(),
    };
    let err = reload_pack_repository(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: false,
        },
        |packs| {
            assert_eq!(
                packs
                    .iter()
                    .map(|pack| pack.id.as_str())
                    .collect::<Vec<_>>(),
                vec![VANILLA_PACK_ID, "high"]
            );
            Err("reload failed".to_string())
        },
    )
    .unwrap_err();

    assert_eq!(err, "reload failed");
    assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "low"]);

    let configured = reload_pack_repository(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: false,
        },
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(configured.data_packs.enabled, vec![VANILLA_PACK_ID, "high"]);
    assert_eq!(configured.data_packs.disabled, vec!["low"]);
    assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "high"]);
}

#[test]
fn reload_failure_report_is_user_facing_and_names_restored_selection() {
    let mut repository = DataPackRepository::new([
        DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn)
            .with_features(feature_flags::default_flags_26_1_2()),
        DataPack::new("kept", PackSource::World),
        DataPack::new("broken", PackSource::World),
    ]);
    repository.set_selected([VANILLA_PACK_ID, "kept"]);
    let initial = WorldDataConfiguration {
        data_packs: DataPackConfig {
            enabled: vec![VANILLA_PACK_ID.to_string(), "broken".to_string()],
            disabled: vec!["kept".to_string()],
        },
        enabled_features: feature_flags::default_flags_26_1_2(),
    };

    let failure = reload_pack_repository_with_report(
        &mut repository,
        &initial,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: false,
        },
        |_| Err("invalid tag entry in file/broken".to_string()),
    )
    .unwrap_err();

    assert_eq!(repository.priority_stack(), vec![VANILLA_PACK_ID, "kept"]);
    assert_eq!(
        failure.restored_enabled,
        vec![VANILLA_PACK_ID.to_string(), "kept".to_string()]
    );
    assert_eq!(
        failure.user_message(),
        "Failed to reload data packs; keeping previous selection [vanilla,kept]: invalid tag entry in file/broken"
    );
}

#[test]
fn parses_pack_metadata_and_detects_compatibility() {
    let metadata = parse_pack_metadata(
        r#"{
          "pack": {
            "description": "Test pack",
            "pack_format": 101,
            "min_format": [101, 0],
            "max_format": [101, 99]
          },
          "features": {
            "enabled": ["vanilla"]
          }
        }"#,
    )
    .unwrap();

    assert_eq!(metadata.description, "Test pack");
    assert_eq!(
        metadata.supported_formats.min,
        PackFormat {
            major: 101,
            minor: 0
        }
    );
    assert_eq!(metadata.compatibility, PackCompatibility::Compatible);
    assert!(metadata.requested_features.contains(feature_flags::VANILLA));

    let old = parse_pack_metadata(
        r#"{
          "pack": {
            "description": "Old pack",
            "pack_format": 81,
            "supported_formats": [80, 81]
          }
        }"#,
    )
    .unwrap();
    assert_eq!(old.compatibility, PackCompatibility::TooOld);

    assert!(parse_pack_metadata(r#"{"pack":{"description":"bad","pack_format":101}}"#).is_err());
}

#[test]
fn pack_compatibility_source_matches_repository_java_contract() {
    const PACK_COMPATIBILITY_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/repository/PackCompatibility.java");
    for sentinel in [
        "TOO_OLD(\"old\")",
        "TOO_NEW(\"new\")",
        "UNKNOWN(\"unknown\")",
        "COMPATIBLE(\"compatible\")",
        "public static final int UNKNOWN_VERSION = Integer.MAX_VALUE;",
        "Component.translatable(\"pack.incompatible.\" + key).withStyle(ChatFormatting.GRAY)",
        "Component.translatable(\"pack.incompatible.confirm.\" + key)",
        "return this == COMPATIBLE;",
        "packDeclaredVersions.minInclusive().major() == Integer.MAX_VALUE",
        "packDeclaredVersions.maxInclusive().compareTo(gameSupportedVersion) < 0",
        "gameSupportedVersion.compareTo(packDeclaredVersions.minInclusive()) < 0 ? TOO_NEW : COMPATIBLE",
    ] {
        assert!(
            PACK_COMPATIBILITY_JAVA.contains(sentinel),
            "missing PackCompatibility sentinel {sentinel}"
        );
    }
}

#[test]
fn pack_compatibility_range_decisions_match_java_order() {
    let current = PackFormat {
        major: 101,
        minor: 1,
    };
    assert_eq!(
        PackCompatibility::for_version(
            PackFormatRange {
                min: PackFormat {
                    major: PackCompatibility::UNKNOWN_VERSION,
                    minor: 0,
                },
                max: PackFormat {
                    major: PackCompatibility::UNKNOWN_VERSION,
                    minor: 0,
                },
            },
            current,
        ),
        PackCompatibility::Unknown
    );
    assert_eq!(
        PackCompatibility::for_version(
            PackFormatRange {
                min: PackFormat {
                    major: 100,
                    minor: 0,
                },
                max: PackFormat {
                    major: 101,
                    minor: 0,
                },
            },
            current,
        ),
        PackCompatibility::TooOld
    );
    assert_eq!(
        PackCompatibility::for_version(
            PackFormatRange {
                min: PackFormat {
                    major: 101,
                    minor: 2,
                },
                max: PackFormat {
                    major: 102,
                    minor: 0,
                },
            },
            current,
        ),
        PackCompatibility::TooNew
    );
    assert_eq!(
        PackCompatibility::for_version(
            PackFormatRange {
                min: PackFormat {
                    major: 101,
                    minor: 0,
                },
                max: PackFormat {
                    major: 101,
                    minor: 1,
                },
            },
            current,
        ),
        PackCompatibility::Compatible
    );

    assert!(!PackCompatibility::TooOld.is_compatible());
    assert!(!PackCompatibility::TooNew.is_compatible());
    assert!(!PackCompatibility::Unknown.is_compatible());
    assert!(PackCompatibility::Compatible.is_compatible());
}

#[test]
fn pack_compatibility_description_and_confirmation_components_match_java() {
    for (compatibility, key) in [
        (PackCompatibility::TooOld, "old"),
        (PackCompatibility::TooNew, "new"),
        (PackCompatibility::Unknown, "unknown"),
        (PackCompatibility::Compatible, "compatible"),
    ] {
        let description = compatibility.description();
        assert_eq!(description.get_string(), format!("pack.incompatible.{key}"));
        assert_eq!(
            description
                .get_style()
                .get_color()
                .map(crate::chat_component::TextColor::serialize),
            Some("gray".to_string())
        );
        assert_eq!(
            compatibility.confirmation().get_string(),
            format!("pack.incompatible.confirm.{key}")
        );
    }
}

#[test]
fn server_repository_discovers_compatible_directory_world_packs_with_metadata() {
    let temp_dir = std::env::temp_dir().join(format!(
        "vibecraft-packs-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let datapacks = temp_dir.join("datapacks");
    fs::create_dir_all(datapacks.join("dir_pack")).unwrap();
    fs::write(
        datapacks.join("dir_pack").join("pack.mcmeta"),
        r#"{
          "pack": {
            "description": "Directory pack",
            "pack_format": 101,
            "min_format": [101, 0],
            "max_format": [101, 99]
          }
        }"#,
    )
    .unwrap();
    fs::create_dir_all(datapacks.join("dir_pack").join("data/example/tags/item")).unwrap();
    fs::create_dir_all(
        datapacks
            .join("dir_pack")
            .join("data/minecraft/advancement"),
    )
    .unwrap();
    fs::write(
        datapacks
            .join("dir_pack")
            .join("data/example/tags/item/test_items.json"),
        r#"{"replace":false,"values":["minecraft:stick"]}"#,
    )
    .unwrap();
    fs::write(
        datapacks
            .join("dir_pack")
            .join("data/minecraft/advancement/root.json"),
        r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}}}"#,
    )
    .unwrap();
    fs::create_dir_all(datapacks.join("missing_meta")).unwrap();
    fs::create_dir_all(datapacks.join("old_pack")).unwrap();
    fs::write(
        datapacks.join("old_pack").join("pack.mcmeta"),
        r#"{"pack":{"description":"Old","pack_format":81,"supported_formats":[80,81]}}"#,
    )
    .unwrap();
    fs::write(datapacks.join("zip_pack.zip"), []).unwrap();
    fs::write(datapacks.join("notes.txt"), []).unwrap();

    let repository = DataPackRepository::server_repository(&datapacks).unwrap();
    let mut ids = repository.available_ids();
    ids.sort();

    assert_eq!(ids, vec!["file/dir_pack", VANILLA_PACK_ID]);

    let loaded = load_world_data_packs(&datapacks).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].pack.id, "file/dir_pack");
    assert_eq!(
        loaded[0].get("data/example/tags/item/test_items.json"),
        Some(r#"{"replace":false,"values":["minecraft:stick"]}"#)
    );
    assert_eq!(
        loaded[0].list_prefix("data/example/tags/"),
        vec!["data/example/tags/item/test_items.json"]
    );
    let index = loaded[0].data_resources().unwrap();
    assert_eq!(
        index.get("minecraft", DataResourceKind::Advancement, "root"),
        Some(r#"{"criteria":{"tick":{"trigger":"minecraft:tick"}}}"#)
    );
    assert_eq!(
        index
            .list("minecraft", DataResourceKind::Advancement)
            .into_iter()
            .map(|resource| resource.id)
            .collect::<Vec<_>>(),
        vec!["root"]
    );

    fs::remove_dir_all(temp_dir).unwrap();
}
