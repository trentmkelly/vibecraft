use super::*;

#[test]
fn gamerule_command_supports_canonical_names_qualified_ids_and_integer_ranges() {
    // GameRuleCommand.register only creates id / namespaced-id literals; legacy names are unknown.
    let mut legacy = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(&mut legacy, LevelBasedPermissionSet::GAMEMASTER, "gamerule keepInventory true"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(&mut legacy, LevelBasedPermissionSet::GAMEMASTER, "gamerule minecraft:keep_inventory true")
            .unwrap()
            .success_count,
        1
    );
    let mut state = ServerCommandState::default();
    let canonical = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamerule keep_inventory true",
    )
    .unwrap();
    assert_eq!(canonical.success_count, 1);
    assert_eq!(
        super::game_rule_value(&state, "keep_inventory").unwrap(),
        super::GameRuleValue::Bool(true)
    );

    let bounded = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamerule max_snow_accumulation_height 8",
    )
    .unwrap();
    assert_eq!(bounded.success_count, 8);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamerule max_snow_accumulation_height 9"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamerule random_tick_speed -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamerule fire_spread_radius_around_player -1"
        )
        .unwrap()
        .success_count,
        -1
    );
}

#[test]
fn gamerule_command_toggles_named_client_observable_rules() {
    let mut state = ServerCommandState::default();
    let cases = [
        ("keep_inventory", "true", "minecraft:keep_inventory", 1),
        (
            "immediate_respawn",
            "true",
            "minecraft:immediate_respawn",
            1,
        ),
        (
            "send_command_feedback",
            "false",
            "minecraft:send_command_feedback",
            0,
        ),
        ("advance_time", "false", "minecraft:advance_time", 0),
        ("mob_griefing", "false", "minecraft:mob_griefing", 0),
    ];

    for (rule, value, sync_rule, success_count) in cases {
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            &format!("gamerule {rule} {value}"),
        )
        .unwrap();
        assert_eq!(result.success_count, success_count);
        assert_eq!(result.feedback_key, "commands.gamerule.set");
        assert!(result.broadcast_to_admins);
        assert_eq!(
            state.game_rule_syncs.last().unwrap(),
            &super::GameRuleSyncEvent {
                rule: sync_rule.to_string(),
                value: value.to_string(),
            }
        );
    }

    assert_eq!(
        super::game_rule_value(&state, "keep_inventory").unwrap(),
        super::GameRuleValue::Bool(true)
    );
    assert_eq!(
        super::game_rule_value(&state, "immediate_respawn").unwrap(),
        super::GameRuleValue::Bool(true)
    );
    assert_eq!(
        super::game_rule_value(&state, "send_command_feedback").unwrap(),
        super::GameRuleValue::Bool(false)
    );
    assert_eq!(
        super::game_rule_value(&state, "advance_time").unwrap(),
        super::GameRuleValue::Bool(false)
    );
    assert_eq!(
        super::game_rule_value(&state, "mob_griefing").unwrap(),
        super::GameRuleValue::Bool(false)
    );
}
