use crate::worldgen::{
    configured_carver, parse_configured_carver_from_json, CarverShape, ConfiguredCarver,
    FloatProvider, HeightRange, VerticalAnchor, WorldCarverType, CONFIGURED_CARVERS,
};
use std::path::PathBuf;

const CARVERS_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/Carvers.java");

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Carvers.java is missing sentinel: {sentinel}"
        );
    }
}

fn vanilla_data_path(parts: &[&str]) -> PathBuf {
    let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
        panic!("VIBECRAFT_DECOMPILED_SOURCE_ROOT is required for Java-source parity tests");
    };
    parts.iter().fold(PathBuf::from(source_root), |path, part| path.join(part))
}

const EXPECTED_CONFIGURED_CARVERS: [ConfiguredCarver; 4] = [
    ConfiguredCarver {
        id: "minecraft:cave",
        carver_type: WorldCarverType::Cave,
        probability: 0.15,
        y: HeightRange {
            min: VerticalAnchor::AboveBottom(8),
            max: VerticalAnchor::Absolute(180),
        },
        y_scale: FloatProvider::Uniform { min: 0.1, max: 0.9 },
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: crate::worldgen::CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:crimson_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Uniform { min: 0.7, max: 1.4 },
            vertical_radius_multiplier: FloatProvider::Uniform { min: 0.8, max: 1.3 },
            floor_level: FloatProvider::Uniform {
                min: -1.0,
                max: -0.4,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:cave_extra_underground",
        carver_type: WorldCarverType::Cave,
        probability: 0.07,
        y: HeightRange {
            min: VerticalAnchor::AboveBottom(8),
            max: VerticalAnchor::Absolute(47),
        },
        y_scale: FloatProvider::Uniform { min: 0.1, max: 0.9 },
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: crate::worldgen::CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:oak_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Uniform { min: 0.7, max: 1.4 },
            vertical_radius_multiplier: FloatProvider::Uniform { min: 0.8, max: 1.3 },
            floor_level: FloatProvider::Uniform {
                min: -1.0,
                max: -0.4,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:canyon",
        carver_type: WorldCarverType::Canyon,
        probability: 0.01,
        y: HeightRange {
            min: VerticalAnchor::Absolute(10),
            max: VerticalAnchor::Absolute(67),
        },
        y_scale: FloatProvider::Constant(3.0),
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: crate::worldgen::CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:warped_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Canyon {
            vertical_rotation: FloatProvider::Uniform {
                min: -0.125,
                max: 0.125,
            },
            shape: crate::worldgen::CanyonShapeConfiguration {
                distance_factor: FloatProvider::Uniform {
                    min: 0.75,
                    max: 1.0,
                },
                thickness: FloatProvider::Trapezoid {
                    min: 0.0,
                    max: 6.0,
                    plateau: 2.0,
                },
                width_smoothness: 3,
                horizontal_radius_factor: FloatProvider::Uniform {
                    min: 0.75,
                    max: 1.0,
                },
                vertical_radius_default_factor: 1.0,
                vertical_radius_center_factor: 0.0,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:nether_cave",
        carver_type: WorldCarverType::NetherCave,
        probability: 0.2,
        y: HeightRange {
            min: VerticalAnchor::Absolute(0),
            max: VerticalAnchor::BelowTop(1),
        },
        y_scale: FloatProvider::Constant(0.5),
        lava_level: VerticalAnchor::AboveBottom(10),
        debug: crate::worldgen::CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:air",
        },
        replaceable_tag: "#minecraft:nether_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Constant(1.0),
            vertical_radius_multiplier: FloatProvider::Constant(1.0),
            floor_level: FloatProvider::Constant(-0.7),
        },
    },
];

#[test]
fn carvers_java_bootstrap_shape_matches_decompilation() {
    assert_eq!(CARVERS_JAVA.lines().count(), 102);
    assert_source_contains_all(
        CARVERS_JAVA,
        &[
            "public static final ResourceKey<ConfiguredWorldCarver<?>> CAVE = createKey(\"cave\");",
            "public static final ResourceKey<ConfiguredWorldCarver<?>> CAVE_EXTRA_UNDERGROUND = createKey(\"cave_extra_underground\");",
            "public static final ResourceKey<ConfiguredWorldCarver<?>> CANYON = createKey(\"canyon\");",
            "public static final ResourceKey<ConfiguredWorldCarver<?>> NETHER_CAVE = createKey(\"nether_cave\");",
            "return ResourceKey.create(Registries.CONFIGURED_CARVER, Identifier.withDefaultNamespace(name));",
            "HolderGetter<Block> blocks = context.lookup(Registries.BLOCK);",
            "CarverDebugSettings.of(false, Blocks.CRIMSON_BUTTON.defaultBlockState())",
            "CarverDebugSettings.of(false, Blocks.OAK_BUTTON.defaultBlockState())",
            "CarverDebugSettings.of(false, Blocks.WARPED_BUTTON.defaultBlockState())",
            "blocks.getOrThrow(BlockTags.OVERWORLD_CARVER_REPLACEABLES)",
            "blocks.getOrThrow(BlockTags.NETHER_CARVER_REPLACEABLES)",
            "new CanyonCarverConfiguration.CanyonShapeConfiguration(",
        ],
    );

    assert_eq!(
        count_occurrences(
            CARVERS_JAVA,
            "public static final ResourceKey<ConfiguredWorldCarver<?>>"
        ),
        4
    );
    assert_eq!(count_occurrences(CARVERS_JAVA, "context.register("), 4);
    assert_eq!(count_occurrences(CARVERS_JAVA, "WorldCarver.CAVE"), 2);
    assert_eq!(count_occurrences(CARVERS_JAVA, "WorldCarver.CANYON"), 1);
    assert_eq!(
        count_occurrences(CARVERS_JAVA, "WorldCarver.NETHER_CAVE"),
        1
    );
    assert_eq!(
        count_occurrences(CARVERS_JAVA, "CarverDebugSettings.of(false"),
        3
    );
    assert_eq!(
        count_occurrences(CARVERS_JAVA, "BlockTags.OVERWORLD_CARVER_REPLACEABLES"),
        3
    );
    assert_eq!(
        count_occurrences(CARVERS_JAVA, "BlockTags.NETHER_CARVER_REPLACEABLES"),
        1
    );
}

#[test]
fn rust_configured_carvers_match_java_bootstrap_parameters() {
    let expected = EXPECTED_CONFIGURED_CARVERS;
    assert_eq!(CONFIGURED_CARVERS, expected);
    assert_eq!(
        CONFIGURED_CARVERS
            .iter()
            .map(|carver| carver.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:cave",
            "minecraft:cave_extra_underground",
            "minecraft:canyon",
            "minecraft:nether_cave",
        ]
    );

    for expected_carver in expected {
        assert_eq!(
            configured_carver(expected_carver.id),
            Some(&expected_carver)
        );
        assert_eq!(
            configured_carver(expected_carver.id.strip_prefix("minecraft:").unwrap()),
            Some(&expected_carver)
        );
    }
}

#[test]
fn configured_carver_json_files_match_rust_bootstrap_table() {
    let dir = vanilla_data_path(&["data", "minecraft", "worldgen", "configured_carver"]);
    let mut file_names = std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("failed to read configured_carver directory {dir:?}: {err}"))
        .map(|entry| {
            entry
                .unwrap_or_else(|err| panic!("failed to read configured_carver entry: {err}"))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    file_names.sort();
    assert_eq!(
        file_names,
        vec![
            "canyon.json",
            "cave.json",
            "cave_extra_underground.json",
            "nether_cave.json",
        ]
    );

    for expected_carver in EXPECTED_CONFIGURED_CARVERS {
        let path = dir.join(format!(
            "{}.json",
            expected_carver.id.strip_prefix("minecraft:").unwrap()
        ));
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {path:?}: {err}"));
        let json = serde_json::from_str(&raw)
            .unwrap_or_else(|err| panic!("failed to parse configured carver JSON {path:?}: {err}"));
        assert_eq!(
            parse_configured_carver_from_json(expected_carver.id, &json)
                .unwrap_or_else(|err| panic!("failed to decode {path:?}: {err}")),
            expected_carver
        );
    }
}
