use std::collections::BTreeMap;

use crate::worldgen::{
    builtin_surface_rule_preset, load_surface_rule, DynSurfaceRule, SurfaceRuleKind,
    BUILTIN_SURFACE_RULE_PRESETS, NETHER_SURFACE_BLOCKS, OVERWORLD_SURFACE_BLOCKS,
};

const SURFACE_RULE_DATA_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/SurfaceRuleData.java"
);
const NOISE_SETTINGS_ROOT: &str =
    "../decompiled-server-26.1.2/data/minecraft/worldgen/noise_settings";

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "SurfaceRuleData.java is missing sentinel: {sentinel}"
        );
    }
}

fn noise_settings_surface_rule_roots() -> BTreeMap<String, String> {
    let root = std::path::Path::new(NOISE_SETTINGS_ROOT);
    let mut roots = BTreeMap::new();
    for entry in std::fs::read_dir(root)
        .unwrap_or_else(|err| panic!("failed to read noise_settings dir {root:?}: {err}"))
    {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read noise_settings entry: {err}"));
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| panic!("invalid noise_settings path {path:?}"));
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read noise_settings JSON {path:?}: {err}"));
        let value = serde_json::from_str::<serde_json::Value>(&raw)
            .unwrap_or_else(|err| panic!("invalid noise_settings JSON {path:?}: {err}"));
        let rule_type = value["surface_rule"]["type"]
            .as_str()
            .unwrap_or_else(|| panic!("noise_settings {id} missing surface_rule.type"));
        roots.insert(format!("minecraft:{id}"), rule_type.to_string());
    }
    roots
}

fn dyn_rule_root_name(rule: &DynSurfaceRule) -> &'static str {
    match rule {
        DynSurfaceRule::Bandlands => "minecraft:bandlands",
        DynSurfaceRule::Block(_) => "minecraft:block",
        DynSurfaceRule::Sequence(_) => "minecraft:sequence",
        DynSurfaceRule::Condition { .. } => "minecraft:condition",
    }
}

#[test]
fn surface_rule_data_java_shape_matches_decompilation() {
    assert_eq!(SURFACE_RULE_DATA_JAVA.lines().count(), 384);
    assert_source_contains_all(
        SURFACE_RULE_DATA_JAVA,
        &[
            "private static final SurfaceRules.RuleSource AIR = makeStateRule(Blocks.AIR);",
            "private static final SurfaceRules.RuleSource ENDSTONE = makeStateRule(Blocks.END_STONE);",
            "public static SurfaceRules.RuleSource overworld() {",
            "return overworldLike(true, false, true);",
            "public static SurfaceRules.RuleSource overworldLike(final boolean doPreliminarySurfaceCheck, final boolean bedrockRoof, final boolean bedrockFloor)",
            "SurfaceRules.verticalGradient(\"bedrock_roof\", VerticalAnchor.belowTop(5), VerticalAnchor.top())",
            "SurfaceRules.verticalGradient(\"bedrock_floor\", VerticalAnchor.bottom(), VerticalAnchor.aboveBottom(5))",
            "SurfaceRules.verticalGradient(\"deepslate\", VerticalAnchor.absolute(0), VerticalAnchor.absolute(8))",
            "SurfaceRules.noiseCondition(Noises.POWDER_SNOW, 0.45, 0.58)",
            "SurfaceRules.noiseCondition(Noises.NETHER_STATE_SELECTOR, 0.0)",
            "SurfaceRules.bandlands()",
            "public static SurfaceRules.RuleSource nether() {",
            "public static SurfaceRules.RuleSource end() {",
            "return ENDSTONE;",
            "public static SurfaceRules.RuleSource air() {",
            "return AIR;",
            "return SurfaceRules.noiseCondition(Noises.SURFACE, threshold / 8.25, Double.MAX_VALUE);",
        ],
    );
    assert_eq!(count_occurrences(SURFACE_RULE_DATA_JAVA, "makeStateRule"), 36);
    assert_eq!(
        count_occurrences(SURFACE_RULE_DATA_JAVA, "SurfaceRules.sequence"),
        44
    );
    assert_eq!(
        count_occurrences(SURFACE_RULE_DATA_JAVA, "SurfaceRules.ifTrue"),
        150
    );
    assert_eq!(
        count_occurrences(SURFACE_RULE_DATA_JAVA, "SurfaceRules.isBiome"),
        35
    );
    assert_eq!(
        count_occurrences(SURFACE_RULE_DATA_JAVA, "SurfaceRules.noiseCondition"),
        20
    );
    assert_eq!(
        count_occurrences(SURFACE_RULE_DATA_JAVA, "SurfaceRules.verticalGradient"),
        5
    );
}

#[test]
fn rust_surface_rule_presets_match_surface_rule_data_methods() {
    assert_eq!(
        BUILTIN_SURFACE_RULE_PRESETS
            .iter()
            .map(|preset| preset.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:overworld",
            "minecraft:caves",
            "minecraft:floating_islands",
            "minecraft:nether",
            "minecraft:end",
            "minecraft:air",
        ]
    );

    assert_eq!(
        builtin_surface_rule_preset("minecraft:overworld").unwrap().rule,
        SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: true,
            bedrock_roof: false,
            bedrock_floor: true,
            deepslate: true,
        }
    );
    assert_eq!(
        builtin_surface_rule_preset("caves").unwrap().rule,
        SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: true,
            bedrock_floor: true,
            deepslate: true,
        }
    );
    assert_eq!(
        builtin_surface_rule_preset("floating_islands").unwrap().rule,
        SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: false,
            bedrock_floor: false,
            deepslate: true,
        }
    );
    assert_eq!(
        builtin_surface_rule_preset("nether").unwrap().rule,
        SurfaceRuleKind::Nether
    );
    assert_eq!(
        builtin_surface_rule_preset("end").unwrap().rule,
        SurfaceRuleKind::State("minecraft:end_stone")
    );
    assert_eq!(
        builtin_surface_rule_preset("air").unwrap().rule,
        SurfaceRuleKind::State("minecraft:air")
    );

    assert!(OVERWORLD_SURFACE_BLOCKS.contains(&"minecraft:grass_block"));
    assert!(OVERWORLD_SURFACE_BLOCKS.contains(&"minecraft:deepslate"));
    assert!(OVERWORLD_SURFACE_BLOCKS.contains(&"minecraft:powder_snow"));
    assert!(OVERWORLD_SURFACE_BLOCKS.contains(&"minecraft:water"));
    assert!(NETHER_SURFACE_BLOCKS.contains(&"minecraft:netherrack"));
    assert!(NETHER_SURFACE_BLOCKS.contains(&"minecraft:warped_nylium"));
    assert!(NETHER_SURFACE_BLOCKS.contains(&"minecraft:crimson_nylium"));
}

#[test]
fn vanilla_noise_settings_surface_rule_roots_parse_through_rust_codec() {
    let roots = noise_settings_surface_rule_roots();
    assert_eq!(
        roots,
        BTreeMap::from([
            ("minecraft:amplified".to_string(), "minecraft:sequence".to_string()),
            ("minecraft:caves".to_string(), "minecraft:sequence".to_string()),
            ("minecraft:end".to_string(), "minecraft:block".to_string()),
            (
                "minecraft:floating_islands".to_string(),
                "minecraft:sequence".to_string(),
            ),
            (
                "minecraft:large_biomes".to_string(),
                "minecraft:sequence".to_string(),
            ),
            ("minecraft:nether".to_string(), "minecraft:sequence".to_string()),
            ("minecraft:overworld".to_string(), "minecraft:sequence".to_string()),
        ])
    );

    for (id, root_type) in roots {
        let rule = load_surface_rule(&id)
            .unwrap_or_else(|| panic!("Rust surface rule codec failed to parse {id}"));
        assert_eq!(dyn_rule_root_name(&rule), root_type);
    }
}
