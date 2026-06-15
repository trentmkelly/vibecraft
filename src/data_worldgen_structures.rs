use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::worldgen::BUILTIN_STRUCTURES;

const STRUCTURES_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/Structures.java");

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Structures.java is missing sentinel: {sentinel}"
        );
    }
}

fn vanilla_data_path(parts: &[&str]) -> PathBuf {
    let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
        panic!("VIBECRAFT_DECOMPILED_SOURCE_ROOT is required for Java-source parity tests");
    };
    parts.iter().fold(PathBuf::from(source_root), |path, part| path.join(part))
}

fn json_object<'a>(
    value: &'a serde_json::Value,
    label: &str,
) -> &'a serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{label} should be an object"))
}

fn load_structure_json() -> BTreeMap<String, serde_json::Value> {
    let root = vanilla_data_path(&["data", "minecraft", "worldgen", "structure"]);
    let mut structures = BTreeMap::new();
    for entry in std::fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("failed to read structure dir {root:?}: {err}"))
    {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read structure entry: {err}"));
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| panic!("invalid structure path {path:?}"));
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read structure JSON {path:?}: {err}"));
        let value = serde_json::from_str::<serde_json::Value>(&raw)
            .unwrap_or_else(|err| panic!("invalid structure JSON {path:?}: {err}"));
        structures.insert(format!("minecraft:{id}"), value);
    }
    structures
}

fn structure<'a>(
    structures: &'a BTreeMap<String, serde_json::Value>,
    id: &str,
) -> &'a serde_json::Map<String, serde_json::Value> {
    json_object(
        structures
            .get(id)
            .unwrap_or_else(|| panic!("missing vanilla structure {id}")),
        id,
    )
}

fn string_field<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> &'a str {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("missing string field {field}"))
}

fn i64_field(object: &serde_json::Map<String, serde_json::Value>, field: &str) -> i64 {
    object
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_else(|| panic!("missing integer field {field}"))
}

fn optional_str<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Option<&'a str> {
    object.get(field).and_then(serde_json::Value::as_str)
}

fn nested_object<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> &'a serde_json::Map<String, serde_json::Value> {
    json_object(
        object
            .get(field)
            .unwrap_or_else(|| panic!("missing object field {field}")),
        field,
    )
}

#[test]
fn structures_java_bootstrap_shape_matches_decompilation() {
    assert_eq!(STRUCTURES_JAVA.lines().count(), 375);
    assert_source_contains_all(
        STRUCTURES_JAVA,
        &[
            "HolderGetter<Biome> biomes = context.lookup(Registries.BIOME);",
            "HolderGetter<StructureTemplatePool> templates = context.lookup(Registries.TEMPLATE_POOL);",
            "WeightedList.of(new MobSpawnSettings.SpawnerData(EntityType.PILLAGER, 1, 1))",
            "MineshaftStructure.Type.MESA",
            "new OceanRuinStructure(new Structure.StructureSettings(biomes.getOrThrow(BiomeTags.HAS_OCEAN_RUIN_WARM)), OceanRuinStructure.Type.WARM, 0.3F, 0.9F)",
            "templates.getOrThrow(SnowyVillagePools.START)",
            "new RuinedPortalStructure.Setup(RuinedPortalPiece.VerticalPlacement.IN_NETHER, 0.5F, 0.0F, false, false, false, true, 1.0F)",
            "Optional.of(Identifier.withDefaultNamespace(\"city_anchor\"))",
            "new JigsawStructure.MaxDistance(116)",
            "TrialChambersStructurePools.ALIAS_BINDINGS",
            "new DimensionPadding(10)",
            "LiquidSettings.IGNORE_WATERLOGGING",
        ],
    );
    assert_eq!(count_occurrences(STRUCTURES_JAVA, "context.register("), 34);
    assert_eq!(
        count_occurrences(STRUCTURES_JAVA, "new JigsawStructure"),
        12
    );
    assert_eq!(
        count_occurrences(STRUCTURES_JAVA, "new RuinedPortalStructure"),
        16
    );
    assert_eq!(count_occurrences(STRUCTURES_JAVA, "spawnOverrides"), 6);
    assert_eq!(count_occurrences(STRUCTURES_JAVA, "terrainAdapation"), 11);
    assert_eq!(count_occurrences(STRUCTURES_JAVA, "generationStep"), 8);
    assert_eq!(count_occurrences(STRUCTURES_JAVA, "BuiltinStructures."), 34);
    assert_eq!(count_occurrences(STRUCTURES_JAVA, "BiomeTags."), 34);
}

#[test]
fn rust_builtin_structure_order_matches_java_registration_order() {
    assert_eq!(BUILTIN_STRUCTURES.len(), 34);
    assert_eq!(
        BUILTIN_STRUCTURES,
        &[
            "minecraft:pillager_outpost",
            "minecraft:mineshaft",
            "minecraft:mineshaft_mesa",
            "minecraft:mansion",
            "minecraft:jungle_pyramid",
            "minecraft:desert_pyramid",
            "minecraft:igloo",
            "minecraft:shipwreck",
            "minecraft:shipwreck_beached",
            "minecraft:swamp_hut",
            "minecraft:stronghold",
            "minecraft:monument",
            "minecraft:ocean_ruin_cold",
            "minecraft:ocean_ruin_warm",
            "minecraft:fortress",
            "minecraft:nether_fossil",
            "minecraft:end_city",
            "minecraft:buried_treasure",
            "minecraft:bastion_remnant",
            "minecraft:village_plains",
            "minecraft:village_desert",
            "minecraft:village_savanna",
            "minecraft:village_snowy",
            "minecraft:village_taiga",
            "minecraft:ruined_portal",
            "minecraft:ruined_portal_desert",
            "minecraft:ruined_portal_jungle",
            "minecraft:ruined_portal_swamp",
            "minecraft:ruined_portal_mountain",
            "minecraft:ruined_portal_ocean",
            "minecraft:ruined_portal_nether",
            "minecraft:ancient_city",
            "minecraft:trail_ruins",
            "minecraft:trial_chambers",
        ]
    );
}

#[test]
fn vanilla_structure_json_ids_types_steps_and_terrain_match_java_surface() {
    let registry = load_structure_json();
    assert_eq!(registry.len(), BUILTIN_STRUCTURES.len());
    for id in BUILTIN_STRUCTURES {
        assert!(registry.contains_key(*id), "missing structure JSON {id}");
    }

    let mut types = BTreeMap::new();
    let mut steps = BTreeMap::new();
    let mut terrain = BTreeMap::new();
    for value in registry.values() {
        let object = json_object(value, "structure");
        *types
            .entry(string_field(object, "type").to_string())
            .or_insert(0) += 1;
        *steps
            .entry(string_field(object, "step").to_string())
            .or_insert(0) += 1;
        *terrain
            .entry(
                optional_str(object, "terrain_adaptation")
                    .unwrap_or("none")
                    .to_string(),
            )
            .or_insert(0) += 1;
    }

    assert_eq!(
        types,
        BTreeMap::from([
            ("minecraft:buried_treasure".to_string(), 1),
            ("minecraft:desert_pyramid".to_string(), 1),
            ("minecraft:end_city".to_string(), 1),
            ("minecraft:fortress".to_string(), 1),
            ("minecraft:igloo".to_string(), 1),
            ("minecraft:jigsaw".to_string(), 10),
            ("minecraft:jungle_temple".to_string(), 1),
            ("minecraft:mineshaft".to_string(), 2),
            ("minecraft:nether_fossil".to_string(), 1),
            ("minecraft:ocean_monument".to_string(), 1),
            ("minecraft:ocean_ruin".to_string(), 2),
            ("minecraft:ruined_portal".to_string(), 7),
            ("minecraft:shipwreck".to_string(), 2),
            ("minecraft:stronghold".to_string(), 1),
            ("minecraft:swamp_hut".to_string(), 1),
            ("minecraft:woodland_mansion".to_string(), 1),
        ])
    );
    assert_eq!(
        steps,
        BTreeMap::from([
            ("surface_structures".to_string(), 26),
            ("underground_decoration".to_string(), 3),
            ("underground_structures".to_string(), 5),
        ])
    );
    assert_eq!(
        terrain,
        BTreeMap::from([
            ("beard_box".to_string(), 1),
            ("beard_thin".to_string(), 7),
            ("bury".to_string(), 2),
            ("encapsulate".to_string(), 1),
            ("none".to_string(), 23),
        ])
    );
}

#[test]
fn vanilla_structure_json_special_cases_match_java_bootstrap() {
    let registry = load_structure_json();

    let outpost = structure(&registry, "minecraft:pillager_outpost");
    assert_eq!(string_field(outpost, "type"), "minecraft:jigsaw");
    assert_eq!(
        string_field(outpost, "start_pool"),
        "minecraft:pillager_outpost/base_plates"
    );
    assert_eq!(i64_field(outpost, "size"), 7);
    assert_eq!(i64_field(outpost, "max_distance_from_center"), 80);
    assert_eq!(
        optional_str(outpost, "terrain_adaptation"),
        Some("beard_thin")
    );
    let outpost_monster = nested_object(nested_object(outpost, "spawn_overrides"), "monster");
    assert_eq!(string_field(outpost_monster, "bounding_box"), "full");

    let snowy = structure(&registry, "minecraft:village_snowy");
    assert_eq!(
        string_field(snowy, "start_pool"),
        "minecraft:village/snowy/town_centers"
    );
    assert_eq!(i64_field(snowy, "size"), 6);
    assert_eq!(
        optional_str(snowy, "project_start_to_heightmap"),
        Some("WORLD_SURFACE_WG")
    );
    assert_eq!(
        optional_str(snowy, "terrain_adaptation"),
        Some("beard_thin")
    );

    let standard_portal = structure(&registry, "minecraft:ruined_portal");
    let setups = standard_portal
        .get("setups")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("ruined portal should have setups"));
    assert_eq!(setups.len(), 2);
    assert_eq!(
        string_field(json_object(&setups[0], "setup"), "placement"),
        "underground"
    );
    assert_eq!(
        string_field(json_object(&setups[1], "setup"), "placement"),
        "on_land_surface"
    );

    let ancient_city = structure(&registry, "minecraft:ancient_city");
    assert_eq!(
        string_field(ancient_city, "start_pool"),
        "minecraft:ancient_city/city_center"
    );
    assert_eq!(i64_field(ancient_city, "max_distance_from_center"), 116);
    assert_eq!(
        nested_object(ancient_city, "start_height").get("absolute"),
        Some(&serde_json::Value::from(-27))
    );
    assert_eq!(nested_object(ancient_city, "spawn_overrides").len(), 8);

    let trial = structure(&registry, "minecraft:trial_chambers");
    assert_eq!(
        string_field(trial, "start_pool"),
        "minecraft:trial_chambers/chamber/end"
    );
    assert_eq!(i64_field(trial, "dimension_padding"), 10);
    assert_eq!(
        string_field(trial, "liquid_settings"),
        "ignore_waterlogging"
    );
    assert_eq!(i64_field(trial, "size"), 20);
    assert_eq!(i64_field(trial, "max_distance_from_center"), 116);
    assert_eq!(
        trial
            .get("pool_aliases")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(3)
    );
}
