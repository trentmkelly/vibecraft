//! Tests for the Brigadier-exact value parser, the `game_rules` saved data codec and the
//! live change-log store.

use super::{
    deserialize_game_rule_value, game_rule_definition, GameRuleArgumentError, GameRuleValue,
    GameRules, LiveGameRules,
};
use crate::storage::nbt::Tag;

fn parse(rule: &str, raw: &str) -> Result<GameRuleValue, GameRuleArgumentError> {
    deserialize_game_rule_value(raw, game_rule_definition(rule).unwrap())
}

#[test]
fn bool_argument_matches_brigadier_reader() {
    assert_eq!(parse("pvp", "true"), Ok(GameRuleValue::Bool(true)));
    assert_eq!(parse("pvp", "\"false\""), Ok(GameRuleValue::Bool(false)));
    assert_eq!(parse("pvp", ""), Err(GameRuleArgumentError::ExpectedBool));
    assert_eq!(
        parse("pvp", "TRUE"),
        Err(GameRuleArgumentError::InvalidBool("TRUE".to_string()))
    );
    assert_eq!(
        parse("pvp", "true "),
        Err(GameRuleArgumentError::TrailingData)
    );
    assert_eq!(
        GameRuleArgumentError::InvalidBool("x".to_string()).message(),
        "Invalid bool, expected true or false but found 'x'"
    );
}

#[test]
fn integer_argument_matches_brigadier_reader_and_bounds() {
    assert_eq!(parse("random_tick_speed", "7"), Ok(GameRuleValue::Int(7)));
    assert_eq!(
        parse("fire_spread_radius_around_player", "-1"),
        Ok(GameRuleValue::Int(-1))
    );
    assert_eq!(
        parse("random_tick_speed", "abc"),
        Err(GameRuleArgumentError::ExpectedInteger)
    );
    assert_eq!(
        parse("random_tick_speed", "1.5"),
        Err(GameRuleArgumentError::InvalidInteger("1.5".to_string()))
    );
    assert_eq!(
        parse("random_tick_speed", "+5"),
        Err(GameRuleArgumentError::ExpectedInteger)
    );
    assert_eq!(
        parse("random_tick_speed", "-1"),
        Err(GameRuleArgumentError::IntegerTooLow { found: -1, min: 0 })
    );
    assert_eq!(
        parse("max_snow_accumulation_height", "9"),
        Err(GameRuleArgumentError::IntegerTooHigh { found: 9, max: 8 })
    );
    assert_eq!(
        GameRuleArgumentError::IntegerTooLow { found: -1, min: 0 }.message(),
        "Integer must not be less than 0, found -1"
    );
    assert_eq!(
        parse("random_tick_speed", "99999999999"),
        Err(GameRuleArgumentError::InvalidInteger("99999999999".to_string()))
    );
}

#[test]
fn saved_tag_uses_identifier_keys_with_byte_and_int_values() {
    let mut rules = GameRules::new(false);
    rules.set("keep_inventory", "true").unwrap();
    rules.set("random_tick_speed", "9").unwrap();
    let Tag::Compound(entries) = rules.to_saved_tag() else {
        panic!("compound expected");
    };
    let get = |key: &str| entries.iter().find(|(name, _)| name == key).map(|(_, tag)| tag);
    assert_eq!(get("minecraft:keep_inventory"), Some(&Tag::Byte(1)));
    assert_eq!(get("minecraft:random_tick_speed"), Some(&Tag::Int(9)));
    assert_eq!(get("minecraft:max_minecart_speed"), None, "feature-gated rule is absent");
    assert_eq!(entries.len(), 58);
}

#[test]
fn saved_tag_round_trips_and_repairs_bad_entries() {
    let mut rules = GameRules::new(true);
    rules.set("max_minecart_speed", "20").unwrap();
    rules.set("pvp", "false").unwrap();
    assert_eq!(GameRules::from_saved_tag(&rules.to_saved_tag(), true), rules);

    // Unknown ids, wrong tag kinds and out-of-range ints fall back to defaults; a disabled
    // feature's rule is dropped.
    let tag = Tag::Compound(vec![
        ("minecraft:bogus".to_string(), Tag::Byte(1)),
        ("minecraft:pvp".to_string(), Tag::String("false".to_string())),
        ("minecraft:random_tick_speed".to_string(), Tag::Int(-5)),
        ("minecraft:max_minecart_speed".to_string(), Tag::Int(20)),
        ("minecraft:keep_inventory".to_string(), Tag::Byte(1)),
    ]);
    let loaded = GameRules::from_saved_tag(&tag, false);
    assert_eq!(loaded.get("pvp"), Some(GameRuleValue::Bool(true)));
    assert_eq!(loaded.get("random_tick_speed"), Some(GameRuleValue::Int(3)));
    assert_eq!(loaded.get("max_minecart_speed"), None);
    assert_eq!(loaded.get("keep_inventory"), Some(GameRuleValue::Bool(true)));
}

#[test]
fn rules_persist_in_data_minecraft_game_rules_dat() {
    let dir = std::env::temp_dir().join(format!("vibecraft-game-rules-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        GameRules::load_from_world(&dir, false).unwrap(),
        GameRules::new(false),
        "a world without the file gets defaults"
    );
    let mut rules = GameRules::new(false);
    rules.set("immediate_respawn", "true").unwrap();
    rules.save_to_world(&dir).unwrap();
    assert!(dir.join("data/minecraft/game_rules.dat").is_file());
    assert_eq!(GameRules::load_from_world(&dir, false).unwrap(), rules);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn live_store_logs_every_set_and_replays_only_newer_changes() {
    let mut live = LiveGameRules::new(GameRules::new(false));
    assert_eq!(live.current_seq(), 0);
    assert!(!live.take_dirty());

    live.set("keep_inventory", "true", false).unwrap();
    // Setting the same value again is still a `GameRules.set` and is logged.
    live.set("keep_inventory", "true", false).unwrap();
    live.set_from_client(
        game_rule_definition("immediate_respawn").unwrap(),
        GameRuleValue::Bool(true),
    );
    assert!(live.take_dirty());
    assert!(!live.take_dirty());

    let all = live.changes_since(0);
    assert_eq!(all.len(), 3);
    assert_eq!(all[2].rule, "immediate_respawn");
    assert!(all[2].announce_to_operators && all[2].applied);
    assert!(!all[0].announce_to_operators);
    assert_eq!(live.changes_since(2).len(), 1);
    assert!(live.bool("immediate_respawn"));
}

#[test]
fn set_from_client_announces_but_ignores_feature_disabled_rules() {
    let mut live = LiveGameRules::new(GameRules::new(false));
    live.set_from_client(
        game_rule_definition("max_minecart_speed").unwrap(),
        GameRuleValue::Int(20),
    );
    assert_eq!(live.get("max_minecart_speed"), None);
    let change = &live.changes_since(0)[0];
    assert!(!change.applied && change.announce_to_operators);
    assert!(!live.take_dirty(), "an ignored set does not dirty the saved data");
}
