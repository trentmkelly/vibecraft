use crate::registry::{registries, BuiltInRegistries, Identifier};

const DIMENSION_TYPES_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/DimensionTypes.java");

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "DimensionTypes.java is missing sentinel: {sentinel}"
        );
    }
}

fn dimension_type_json(name: &str) -> serde_json::Value {
    let path = std::path::Path::new(env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT"))
        .join("data")
        .join("minecraft")
        .join("dimension_type")
        .join(format!("{name}.json"));
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read dimension type JSON {path:?}: {err}"));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("failed to parse dimension type JSON {path:?}: {err}"))
}

fn json_path<'a>(value: &'a serde_json::Value, path: &[&str]) -> &'a serde_json::Value {
    let mut current = value;
    for segment in path {
        current = current
            .get(*segment)
            .unwrap_or_else(|| panic!("missing JSON path segment {segment:?} in {path:?}"));
    }
    current
}

fn json_string(value: &serde_json::Value, path: &[&str]) -> String {
    json_path(value, path)
        .as_str()
        .unwrap_or_else(|| panic!("JSON path {path:?} is not a string"))
        .to_string()
}

fn json_bool(value: &serde_json::Value, path: &[&str]) -> bool {
    json_path(value, path)
        .as_bool()
        .unwrap_or_else(|| panic!("JSON path {path:?} is not a bool"))
}

fn json_i64(value: &serde_json::Value, path: &[&str]) -> i64 {
    json_path(value, path)
        .as_i64()
        .unwrap_or_else(|| panic!("JSON path {path:?} is not an integer"))
}

fn json_f64(value: &serde_json::Value, path: &[&str]) -> f64 {
    json_path(value, path)
        .as_f64()
        .unwrap_or_else(|| panic!("JSON path {path:?} is not a number"))
}

#[test]
fn dimension_types_java_bootstrap_shape_matches_decompilation() {
    assert_eq!(DIMENSION_TYPES_JAVA.lines().count(), 153);
    assert_source_contains_all(
        DIMENSION_TYPES_JAVA,
        &[
            "public static void bootstrap(final BootstrapContext<DimensionType> context)",
            "HolderGetter<Timeline> timelines = context.lookup(Registries.TIMELINE);",
            "HolderGetter<WorldClock> clocks = context.lookup(Registries.WORLD_CLOCK);",
            "EnvironmentAttributeMap overworldAttributes = EnvironmentAttributeMap.builder()",
            "context.register(\n         BuiltinDimensionTypes.OVERWORLD,",
            "context.register(\n         BuiltinDimensionTypes.NETHER,",
            "context.register(\n         BuiltinDimensionTypes.END,",
            "context.register(\n         BuiltinDimensionTypes.OVERWORLD_CAVES,",
            "BlockTags.INFINIBURN_OVERWORLD",
            "BlockTags.INFINIBURN_NETHER",
            "BlockTags.INFINIBURN_END",
            "new DimensionType.MonsterSettings(UniformInt.of(0, 7), 0)",
            "new DimensionType.MonsterSettings(ConstantInt.of(7), 15)",
            "new DimensionType.MonsterSettings(ConstantInt.of(15), 0)",
            "DimensionType.Skybox.OVERWORLD",
            "DimensionType.Skybox.NONE",
            "DimensionType.Skybox.END",
            "CardinalLighting.Type.NETHER",
            "timelines.getOrThrow(TimelineTags.IN_OVERWORLD)",
            "Optional.of(clocks.getOrThrow(WorldClocks.THE_END))",
        ],
    );

    assert_eq!(
        count_occurrences(DIMENSION_TYPES_JAVA, "context.register("),
        4
    );
    assert_eq!(
        count_occurrences(DIMENSION_TYPES_JAVA, "new DimensionType("),
        4
    );
    assert_eq!(
        count_occurrences(DIMENSION_TYPES_JAVA, "new DimensionType.MonsterSettings("),
        4
    );
    assert_eq!(
        count_occurrences(DIMENSION_TYPES_JAVA, "Optional.of(clocks.getOrThrow("),
        3
    );
}

#[test]
fn builtin_dimension_type_registry_order_matches_java_bootstrap_order() {
    let builtins = BuiltInRegistries::bootstrap_26_1_2().expect("builtins should bootstrap");
    assert_eq!(
        builtins.dimension_types.registry_id(),
        &Identifier::parse(registries::DIMENSION_TYPE).unwrap()
    );
    assert_eq!(
        builtins
            .dimension_types
            .iter()
            .map(|entry| (entry.id(), entry.key().location().to_string()))
            .collect::<Vec<_>>(),
        vec![
            (0, "minecraft:overworld".to_string()),
            (1, "minecraft:the_nether".to_string()),
            (2, "minecraft:the_end".to_string()),
            (3, "minecraft:overworld_caves".to_string()),
        ]
    );
}

#[test]
fn overworld_dimension_type_json_matches_java_constructor_values() {
    let overworld = dimension_type_json("overworld");
    assert!(!json_bool(&overworld, &["has_ceiling"]));
    assert!(json_bool(&overworld, &["has_skylight"]));
    assert_eq!(json_f64(&overworld, &["coordinate_scale"]), 1.0);
    assert_eq!(json_i64(&overworld, &["min_y"]), -64);
    assert_eq!(json_i64(&overworld, &["height"]), 384);
    assert_eq!(json_i64(&overworld, &["logical_height"]), 384);
    assert_eq!(
        json_string(&overworld, &["infiniburn"]),
        "#minecraft:infiniburn_overworld"
    );
    assert_eq!(
        json_string(&overworld, &["timelines"]),
        "#minecraft:in_overworld"
    );
    assert_eq!(
        json_string(&overworld, &["default_clock"]),
        "minecraft:overworld"
    );
    assert_eq!(
        json_i64(&overworld, &["monster_spawn_light_level", "min_inclusive"]),
        0
    );
    assert_eq!(
        json_i64(&overworld, &["monster_spawn_light_level", "max_inclusive"]),
        7
    );

    let overworld_caves = dimension_type_json("overworld_caves");
    assert!(json_bool(&overworld_caves, &["has_ceiling"]));
    assert!(json_bool(&overworld_caves, &["has_skylight"]));
    assert_eq!(json_i64(&overworld_caves, &["min_y"]), -64);
    assert_eq!(json_i64(&overworld_caves, &["height"]), 384);
    assert_eq!(
        json_string(&overworld_caves, &["default_clock"]),
        "minecraft:overworld"
    );
}

#[test]
fn nether_dimension_type_json_matches_java_constructor_values() {
    let nether = dimension_type_json("the_nether");
    assert!(json_bool(&nether, &["has_ceiling"]));
    assert!(!json_bool(&nether, &["has_skylight"]));
    assert_eq!(json_f64(&nether, &["coordinate_scale"]), 8.0);
    assert_eq!(json_i64(&nether, &["height"]), 256);
    assert_eq!(json_i64(&nether, &["logical_height"]), 128);
    assert_eq!(json_f64(&nether, &["ambient_light"]), 0.1);
    assert_eq!(
        json_string(&nether, &["infiniburn"]),
        "#minecraft:infiniburn_nether"
    );
    assert_eq!(json_string(&nether, &["skybox"]), "none");
    assert_eq!(json_string(&nether, &["cardinal_light"]), "nether");
    assert_eq!(json_i64(&nether, &["monster_spawn_light_level"]), 7);
    assert_eq!(json_i64(&nether, &["monster_spawn_block_light_limit"]), 15);
}

#[test]
fn end_dimension_type_json_matches_java_constructor_values() {
    let end = dimension_type_json("the_end");
    assert!(json_bool(&end, &["has_ender_dragon_fight"]));
    assert!(json_bool(&end, &["has_fixed_time"]));
    assert_eq!(json_f64(&end, &["ambient_light"]), 0.25);
    assert_eq!(json_string(&end, &["skybox"]), "end");
    assert_eq!(json_string(&end, &["default_clock"]), "minecraft:the_end");
    assert_eq!(json_i64(&end, &["monster_spawn_light_level"]), 15);
    assert_eq!(
        json_string(
            &end,
            &[
                "attributes",
                "minecraft:audio/background_music",
                "default",
                "sound"
            ]
        ),
        "minecraft:music.end"
    );
}
