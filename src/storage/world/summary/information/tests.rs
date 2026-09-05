use super::*;
use serde_json::{json, Value};

fn summary(features: Tag) -> LevelSummary {
    let root = Tag::Compound(vec![(
        "Data".to_owned(),
        Tag::Compound(vec![
            ("version".to_owned(), Tag::Int(19133)),
            ("DataVersion".to_owned(), Tag::Int(4790)),
            (
                "Version".to_owned(),
                Tag::Compound(vec![
                    ("Id".to_owned(), Tag::Int(4790)),
                    ("Name".to_owned(), Tag::String("26.1.2".to_owned())),
                ]),
            ),
            ("enabled_features".to_owned(), features),
        ]),
    )]);
    LevelSummary::from_level_dat(&LevelDirectory::new("world"), &root, false).unwrap()
}

fn json_info(summary: &LevelSummary) -> Value {
    serde_json::from_str(&summary.info().to_json()).unwrap()
}

#[test]
fn summary_flags_tolerate_unknown_entries_without_discarding_known_experiments() {
    for name in [
        "trade_rebalance",
        "redstone_experiments",
        "minecart_improvements",
    ] {
        assert!(
            summary(Tag::List(vec![
                Tag::String("unknown:flag".to_owned()),
                Tag::String("Invalid!".to_owned()),
                Tag::String(format!("minecraft:{name}")),
            ]))
            .experimental
        );
    }
    for input in [
        Tag::Int(1),
        Tag::IntArray(vec![1]),
        Tag::List(vec![]),
        Tag::List(vec![
            Tag::String("vanilla".to_owned()),
            Tag::String("unknown:flag".to_owned()),
        ]),
    ] {
        assert!(!summary(input).experimental);
    }
}

#[test]
fn warning_priority_and_incompatible_version_arguments_match_java() {
    let mut value = summary(Tag::List(vec![]));
    value.locked = true;
    value.requires_manual_conversion = true;
    value.version.minecraft_version.series = "other".to_owned();
    assert_eq!(
        json_info(&value),
        json!({"translate":"selectWorld.locked","color":"red"})
    );
    value.locked = false;
    assert_eq!(
        json_info(&value),
        json!({"translate":"selectWorld.conversion","color":"red"})
    );
    value.requires_manual_conversion = false;
    value.version.minecraft_version_name.clear();
    assert_eq!(
        json_info(&value),
        json!({"translate":"selectWorld.incompatible.info","color":"red",
        "with":[{"translate":"selectWorld.versionUnknown"}]})
    );
}

#[test]
fn normal_info_preserves_component_tree_colors_and_backup_version_style() {
    let mut value = summary(Tag::List(vec![Tag::String("trade_rebalance".to_owned())]));
    value.hardcore = true;
    value.cheats = true;
    value.version.minecraft_version.id = 4791;
    assert_eq!(
        json_info(&value),
        json!({"text":"", "extra":[
            {"translate":"gameMode.hardcore","color":"#FF0000"},
            {"text":", "},{"translate":"selectWorld.commands"},
            {"text":", "},{"translate":"selectWorld.experimental","color":"yellow"},
            {"text":", ","extra":[{"translate":"selectWorld.version"},{"text":" "},{"text":"26.1.2","color":"red"}]}
        ]})
    );
    value.version.minecraft_version.id = 4772;
    assert_eq!(
        json_info(&value)["extra"][5]["extra"][2],
        json!({"text":"26.1.2","italic":true})
    );
    value.version.minecraft_version.id = 4790;
    assert_eq!(
        json_info(&value)["extra"][5]["extra"][2],
        json!({"text":"26.1.2"})
    );
}

#[test]
fn game_mode_and_primary_action_components_use_translation_keys() {
    let mut value = summary(Tag::List(vec![]));
    for (mode, name) in [
        (LevelGameType::Survival, "survival"),
        (LevelGameType::Creative, "creative"),
        (LevelGameType::Adventure, "adventure"),
        (LevelGameType::Spectator, "spectator"),
    ] {
        value.game_type = mode;
        assert_eq!(json_info(&value)["translate"], format!("gameMode.{name}"));
    }
    assert_eq!(
        value.primary_action_message().to_json(),
        "{\"translate\":\"selectWorld.select\"}"
    );
    value.requires_file_fixing = true;
    assert_eq!(
        value.primary_action_message().to_json(),
        "{\"translate\":\"selectWorld.upgrade_and_play\"}"
    );
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_summary_info_and_experimental_contracts_are_present() {
    const SUMMARY: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/LevelSummary.java");
    const FLAGS: &str = vibecraft_java_source!("/net/minecraft/world/flag/FeatureFlags.java");
    assert!(FLAGS.contains("return !features.isSubsetOf(VANILLA_SET)"));
    for fragment in [
        "this.isLocked()",
        "this.requiresManualConversion()",
        "selectWorld.incompatible.info",
        "gameMode.hardcore",
        "withColor(-65536)",
        "selectWorld.experimental",
        "ChatFormatting.YELLOW",
        "this.isDowngrade() ? ChatFormatting.RED : ChatFormatting.ITALIC",
    ] {
        assert!(
            SUMMARY.contains(fragment),
            "missing Java summary info contract: {fragment}"
        );
    }
}
