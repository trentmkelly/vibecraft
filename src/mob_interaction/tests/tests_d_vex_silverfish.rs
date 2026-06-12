#[test]
fn vex_lifetime_charge_and_evoker_summon_gates_match_java_rules() {
    assert_vex_attributes_lifetime_and_charging_flags();
    assert_vex_charge_copy_target_and_random_move_rules();
    assert_evoker_vex_summon_rules();
}

fn assert_vex_attributes_lifetime_and_charging_flags() {
    assert_eq!(
        vex_attributes(),
        VexAttributes {
            max_health: 14.0,
            attack_damage: 4.0,
            xp_reward: 3,
        }
    );
    assert_eq!(VEX_FLAP_DEGREES_PER_TICK, 45.836624);
    assert_eq!(VEX_TICKS_PER_FLAP, 4);
    assert!(vex_is_flapping(8));
    assert!(!vex_is_flapping(9));
    assert_eq!(VEX_DEFAULT_MAINHAND_ITEM, "minecraft:iron_sword");
    assert_eq!(VEX_MAINHAND_DROP_CHANCE, 0.0);
    assert_eq!(VEX_LIGHT_LEVEL_MAGIC_VALUE, 1.0);
    assert_eq!(
        vex_class_surface(),
        VexClassSurface {
            data_flags_default: 0,
            ambient_sound: "minecraft:entity.vex.ambient",
            death_sound: "minecraft:entity.vex.death",
            hurt_sound: "minecraft:entity.vex.hurt",
            light_level_magic_value: 1.0,
            default_mainhand_item: "minecraft:iron_sword",
            mainhand_drop_chance: 0.0,
        }
    );
    assert_eq!(
        vex_goal_surface(),
        VexGoalSurface {
            float_goal_priority: 0,
            charge_attack_priority: 4,
            random_move_priority: 8,
            look_at_player_priority: 9,
            look_at_player_range: 3.0,
            look_at_player_probability: 1.0,
            look_at_mob_priority: 10,
            look_at_mob_range: 8.0,
            hurt_by_target_priority: 1,
            hurt_by_excluded_class: "Raider",
            hurt_by_alerts_others: true,
            copy_owner_target_priority: 2,
            nearest_player_target_priority: 3,
            nearest_player_must_see: true,
        }
    );
    assert!(vex_is_affected_by_blocks(false));
    assert!(!vex_is_affected_by_blocks(true));

    let flags = vex_set_charging(0, true);
    assert!(vex_is_charging(flags));
    assert!(!vex_is_charging(vex_set_charging(flags, false)));
    assert_eq!(
        vex_save_state(true, true, 77, true),
        VexSaveState {
            bound_origin_present: true,
            life_ticks_written: Some(77),
            owner_present: true,
        }
    );
    assert_eq!(
        vex_save_state(false, false, 77, false),
        VexSaveState {
            bound_origin_present: false,
            life_ticks_written: None,
            owner_present: false,
        }
    );
    assert!(vex_restore_owner(true, true));
    assert!(!vex_restore_owner(false, true));
    assert_eq!(
        vex_tick(true, 2),
        VexTickOutcome {
            no_physics_during_tick: true,
            no_gravity_after_tick: true,
            limited_life_ticks: 1,
            starve_damage: false,
        }
    );
    assert_eq!(
        vex_tick(true, 1),
        VexTickOutcome {
            no_physics_during_tick: true,
            no_gravity_after_tick: true,
            limited_life_ticks: 20,
            starve_damage: true,
        }
    );
    assert!(!vex_tick(false, 0).starve_damage);
}

fn assert_vex_charge_copy_target_and_random_move_rules() {
    assert_vex_charge_attack_rules();
    assert_vex_copy_owner_target_rules();
    assert_vex_move_control_rules();
    assert_vex_random_move_rules();
}

fn assert_vex_charge_attack_rules() {
    assert!(vex_charge_attack_can_use(true, true, false, 0, 4.1));
    assert!(!vex_charge_attack_can_use(true, true, false, 1, 4.1));
    assert!(!vex_charge_attack_can_use(true, true, true, 0, 4.1));
    assert!(!vex_charge_attack_can_use(true, true, false, 0, 4.0));
    assert!(vex_charge_attack_can_continue(true, true, true, true));
    assert!(!vex_charge_attack_can_continue(true, false, true, true));
    assert_eq!(vex_charge_attack_tick(true, 16.0), (true, false));
    assert_eq!(vex_charge_attack_tick(false, 8.9), (false, true));
    assert_eq!(vex_charge_attack_tick(false, 9.0), (false, false));
    assert_eq!(
        vex_charge_attack_start(Some(VexVec3 {
            x: 1.0,
            y: 65.0,
            z: -2.0,
        })),
        (
            Some(VexVec3 {
                x: 1.0,
                y: 65.0,
                z: -2.0,
            }),
            true,
            "minecraft:entity.vex.charge",
        )
    );
}

fn assert_vex_copy_owner_target_rules() {
    assert!(vex_copy_owner_target_can_use(true, true, true));
    assert!(!vex_copy_owner_target_can_use(true, false, true));
    assert_eq!(VEX_OWNER_TARGET_RANGE, 16.0);
}

fn assert_vex_move_control_rules() {
    assert_eq!(VEX_MOVE_ACCELERATION, 0.05);
    assert_eq!(VEX_MOVE_CLOSE_DAMPING, 0.5);
    assert_eq!(
        vex_move_control_tick(VexMoveControlInput {
            move_to_operation: true,
            wanted: VexVec3 {
                x: 0.2,
                y: 0.0,
                z: 0.0,
            },
            position: VexVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            delta_movement: VexVec3 {
                x: 0.4,
                y: 0.0,
                z: 0.0,
            },
            speed_modifier: 1.0,
            bounding_box_size: 0.4,
            target_position: None,
        }),
        VexMoveControlTick {
            wait_operation: true,
            delta_movement: VexVec3 {
                x: 0.2,
                y: 0.0,
                z: 0.0,
            },
            y_rot: None,
            y_body_rot: None,
        }
    );
    assert_eq!(
        vex_move_control_tick(VexMoveControlInput {
            move_to_operation: true,
            wanted: VexVec3 {
                x: 0.0,
                y: 0.0,
                z: 10.0,
            },
            position: VexVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            delta_movement: VexVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            speed_modifier: 1.0,
            bounding_box_size: 0.4,
            target_position: Some(VexVec3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            }),
        }),
        VexMoveControlTick {
            wait_operation: false,
            delta_movement: VexVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.05,
            },
            y_rot: Some(-90.0),
            y_body_rot: Some(-90.0),
        }
    );
}

fn assert_vex_random_move_rules() {
    assert!(vex_random_move_can_use(false, 0));
    assert!(!vex_random_move_can_use(false, 1));
    assert_eq!(VEX_RANDOM_MOVE_ATTEMPTS, 3);
    assert_eq!(VEX_RANDOM_MOVE_XZ_RANDOM_BOUND, 15);
    assert_eq!(VEX_RANDOM_MOVE_Y_RANDOM_BOUND, 11);
    assert_eq!(VEX_RANDOM_MOVE_XZ_OFFSET, 7);
    assert_eq!(VEX_RANDOM_MOVE_Y_OFFSET, 5);
    assert_eq!(VEX_RANDOM_MOVE_SPEED, 0.25);
    assert_eq!(
        vex_random_move_candidate(
            Some(VexBlockPos { x: 10, y: 64, z: -3 }),
            VexBlockPos { x: 0, y: 70, z: 0 },
            14,
            10,
            0,
            true,
            false,
        ),
        VexRandomMoveCandidate {
            wanted_position: Some(VexVec3 {
                x: 17.5,
                y: 69.5,
                z: -9.5,
            }),
            look_at: Some(VexVec3 {
                x: 17.5,
                y: 69.5,
                z: -9.5,
            }),
            speed: 0.25,
        }
    );
    assert_eq!(
        vex_random_move_candidate(
            None,
            VexBlockPos { x: 0, y: 70, z: 0 },
            7,
            5,
            7,
            false,
            true,
        )
        .wanted_position,
        None
    );
}

fn assert_evoker_vex_summon_rules() {
    assert!(evoker_vex_summon_can_use(true, 2, 3));
    assert!(!evoker_vex_summon_can_use(true, 3, 3));
    assert!(!evoker_vex_summon_can_use(false, 0, 8));
    assert_eq!(evoker_vex_limited_life_ticks(0), 600);
    assert_eq!(evoker_vex_limited_life_ticks(89), 2380);
    assert_eq!(
        evoker_vex_summon_plan(true, 2, 3, 10, true),
        VexSummonPlan {
            can_summon: true,
            count: 3,
            limited_life_ticks: 800,
            y_offset: 1,
            horizontal_random_bound: 5,
            copy_evoker_team: true,
            game_event: "minecraft:entity_place",
        }
    );
    assert_eq!(EVOKER_VEX_SUMMON_CASTING_TIME, 100);
    assert_eq!(EVOKER_VEX_SUMMON_INTERVAL, 340);
}

#[test]
fn silverfish_infested_merge_and_wake_rules_match_java_rules() {
    assert_silverfish_attributes_spawn_and_infested_mappings();
    assert_silverfish_merge_and_hurt_wake_rules();
    assert_silverfish_wake_scan_and_step_rules();
}

fn assert_silverfish_attributes_spawn_and_infested_mappings() {
    assert_eq!(
        silverfish_attributes(),
        SilverfishAttributes {
            max_health: 8.0,
            movement_speed: 0.25,
            attack_damage: 1.0,
        }
    );
    assert!(silverfish_spawn_allowed(true, true, true));
    assert!(silverfish_spawn_allowed(true, false, false));
    assert!(!silverfish_spawn_allowed(true, false, true));
    assert!(!silverfish_spawn_allowed(false, true, false));
    assert_eq!(SILVERFISH_NEAR_PLAYER_SPAWN_BLOCK_RANGE, 5.0);
    assert_eq!(SILVERFISH_STEP_SOUND_VOLUME, 0.15);
    assert_eq!(SILVERFISH_STEP_SOUND_PITCH, 1.0);
    assert_eq!(
        silverfish_class_surface(),
        SilverfishClassSurface {
            movement_emission: "events",
            ambient_sound: "minecraft:entity.silverfish.ambient",
            hurt_sound: "minecraft:entity.silverfish.hurt",
            death_sound: "minecraft:entity.silverfish.death",
            step_sound: "minecraft:entity.silverfish.step",
            step_sound_volume: 0.15,
            step_sound_pitch: 1.0,
        }
    );
    assert_eq!(
        silverfish_goal_surface(),
        SilverfishGoalSurface {
            float_goal_priority: 1,
            powder_snow_goal_priority: 1,
            wake_friends_priority: 3,
            melee_attack_priority: 4,
            melee_attack_speed: 1.0,
            melee_attack_follow_even_if_not_seen: false,
            merge_with_stone_priority: 5,
            merge_with_stone_speed: 1.0,
            merge_with_stone_interval: 10,
            hurt_by_target_priority: 1,
            hurt_by_alerts_others: true,
            nearest_player_target_priority: 2,
            nearest_player_must_see: true,
        }
    );
    assert_eq!(
        silverfish_tick_rotation(25.0),
        SilverfishRotationTick {
            y_rot: 25.0,
            y_body_rot: 25.0,
        }
    );
    assert_eq!(
        silverfish_set_y_body_rot(10.0, 70.0),
        SilverfishRotationTick {
            y_rot: 70.0,
            y_body_rot: 70.0,
        }
    );

    assert_eq!(
        silverfish_infested_block_for_host("minecraft:stone"),
        Some("minecraft:infested_stone")
    );
    assert_eq!(
        silverfish_infested_block_for_host("minecraft:cobblestone"),
        Some("minecraft:infested_cobblestone")
    );
    assert_eq!(
        silverfish_infested_block_for_host("minecraft:stone_bricks"),
        Some("minecraft:infested_stone_bricks")
    );
    assert_eq!(
        silverfish_infested_block_for_host("minecraft:mossy_stone_bricks"),
        Some("minecraft:infested_mossy_stone_bricks")
    );
    assert_eq!(
        silverfish_infested_block_for_host("minecraft:cracked_stone_bricks"),
        Some("minecraft:infested_cracked_stone_bricks")
    );
    assert_eq!(
        silverfish_infested_block_for_host("minecraft:chiseled_stone_bricks"),
        Some("minecraft:infested_chiseled_stone_bricks")
    );
    assert_eq!(
        silverfish_infested_block_for_host("minecraft:deepslate"),
        Some("minecraft:infested_deepslate")
    );
    assert_eq!(silverfish_infested_block_for_host("minecraft:dirt"), None);
    assert_eq!(
        silverfish_host_block_for_infested("minecraft:infested_deepslate"),
        Some("minecraft:deepslate")
    );
}

fn assert_silverfish_merge_and_hurt_wake_rules() {
    assert_eq!(
        silverfish_walk_target_value("minecraft:stone", 0.25),
        SILVERFISH_WALK_TARGET_HOST_VALUE
    );
    assert_eq!(silverfish_walk_target_value("minecraft:dirt", 0.25), 0.25);
    assert!(silverfish_merge_can_use(
        false,
        true,
        true,
        0,
        "minecraft:stone"
    ));
    assert!(!silverfish_merge_can_use(
        true,
        true,
        true,
        0,
        "minecraft:stone"
    ));
    assert!(!silverfish_merge_can_use(
        false,
        false,
        true,
        0,
        "minecraft:stone"
    ));
    assert!(!silverfish_merge_can_use(
        false,
        true,
        false,
        0,
        "minecraft:stone"
    ));
    assert!(!silverfish_merge_can_use(
        false,
        true,
        true,
        1,
        "minecraft:stone"
    ));
    assert!(!silverfish_merge_can_use(
        false,
        true,
        true,
        0,
        "minecraft:dirt"
    ));
    assert_eq!(SILVERFISH_MERGE_SPEED, 1.0);
    assert_eq!(SILVERFISH_MERGE_INTERVAL_TICKS, 10);

    assert_eq!(
        silverfish_notify_hurt_delay(0, true, false),
        SILVERFISH_WAKE_DELAY_TICKS
    );
    assert_eq!(
        silverfish_notify_hurt_delay(0, false, true),
        SILVERFISH_WAKE_DELAY_TICKS
    );
    assert_eq!(silverfish_notify_hurt_delay(7, true, false), 7);
    assert_eq!(silverfish_notify_hurt_delay(0, false, false), 0);

    assert!(silverfish_infested_break_spawns_silverfish(true, false));
    assert!(!silverfish_infested_break_spawns_silverfish(false, false));
    assert!(!silverfish_infested_break_spawns_silverfish(true, true));
}

fn assert_silverfish_wake_scan_and_step_rules() {
    let offsets = silverfish_wake_scan_offsets();
    assert_eq!(offsets.len(), 11 * 21 * 21);
    assert_eq!(offsets[0], (0, 0, 0));
    assert_eq!(offsets[1], (0, 0, 1));
    assert_eq!(offsets[2], (0, 0, -1));
    assert_eq!(offsets[21], (1, 0, 0));
    assert_eq!(offsets[21 * 21], (0, 1, 0));
    assert_eq!(*offsets.last().unwrap(), (-10, -5, -10));

    assert_eq!(
        silverfish_wake_step((1, 0, -1), "minecraft:infested_stone", true),
        Some(SilverfishWakeStep {
            offset: (1, 0, -1),
            action: SilverfishWakeAction::DestroyInfestedBlock,
        })
    );
    assert_eq!(
        silverfish_wake_step((1, 0, -1), "minecraft:infested_stone", false),
        Some(SilverfishWakeStep {
            offset: (1, 0, -1),
            action: SilverfishWakeAction::RestoreHostBlock,
        })
    );
    assert_eq!(
        silverfish_wake_step((0, 0, 0), "minecraft:stone", true),
        None
    );

    let mut scan = vec!["minecraft:air"; offsets.len()];
    scan[2] = "minecraft:infested_stone";
    scan[21] = "minecraft:infested_deepslate";
    assert_eq!(
        silverfish_wake_steps_until_random_stop(&scan, true, &[false, true]),
        vec![
            SilverfishWakeStep {
                offset: (0, 0, -1),
                action: SilverfishWakeAction::DestroyInfestedBlock,
            },
            SilverfishWakeStep {
                offset: (1, 0, 0),
                action: SilverfishWakeAction::DestroyInfestedBlock,
            },
        ]
    );
}

#[test]
fn zoglin_attack_target_and_hoglin_conversion_rules_match_java_rules() {
    assert_zoglin_attributes_and_constants();
    assert_zoglin_targeting_and_attack_rules();
    assert_hoglin_attributes_and_spawn_rules();
    assert_hoglin_target_and_hurt_response_rules();
    assert_hoglin_social_ai_and_activity_rules();
    assert_hoglin_constants();
    assert_hoglin_conversion_rules();
    assert_abstract_piglin_conversion_rules();
    assert_piglin_brute_rules();
    assert_hoglin_base_attack_damage_and_throw_rules();
}

fn assert_zoglin_attributes_and_constants() {
    assert_eq!(
        zoglin_attributes(),
        ZoglinAttributes {
            max_health: 40.0,
            movement_speed: 0.3,
            knockback_resistance: 0.6,
            attack_knockback: 1.0,
            attack_damage: 6.0,
            xp_reward: 5,
        }
    );
    assert_eq!(zoglin_attack_damage(false), 6.0);
    assert_eq!(zoglin_attack_damage(true), 0.5);
    assert_eq!(zoglin_attack_interval_ticks(false), 40);
    assert_eq!(zoglin_attack_interval_ticks(true), 15);
    assert!(zoglin_finalize_spawn_is_baby(0.199));
    assert!(!zoglin_finalize_spawn_is_baby(0.2));
    assert_eq!(ZOGLIN_IDLE_SPEED_MULTIPLIER, 0.4);
    assert_eq!(ZOGLIN_FIGHTING_MOVEMENT_SPEED, 0.3);
    assert_eq!(ZOGLIN_LOOK_TARGET_RANGE, 8.0);
    assert_eq!(ZOGLIN_LOOK_INTERVAL_MIN_TICKS, 30);
    assert_eq!(ZOGLIN_LOOK_INTERVAL_MAX_TICKS, 60);
    assert_eq!(ZOGLIN_DO_NOTHING_MIN_TICKS, 30);
    assert_eq!(ZOGLIN_DO_NOTHING_MAX_TICKS, 60);
}

fn assert_zoglin_targeting_and_attack_rules() {
    assert!(zoglin_valid_attack_target("minecraft:player", true));
    assert!(!zoglin_valid_attack_target("minecraft:zoglin", true));
    assert!(!zoglin_valid_attack_target("minecraft:creeper", true));
    assert!(!zoglin_valid_attack_target("minecraft:player", false));
    assert_eq!(
        zoglin_ambient_sound(false, false),
        Some("minecraft:entity.zoglin.ambient")
    );
    assert_eq!(
        zoglin_ambient_sound(true, false),
        Some("minecraft:entity.zoglin.angry")
    );
    assert_eq!(zoglin_ambient_sound(true, true), None);
    assert!(zoglin_on_hurt_should_retarget(true, true, true, false));
    assert!(!zoglin_on_hurt_should_retarget(true, true, true, true));
    assert_eq!(
        zoglin_event_attack_animation_ticks(4),
        Some(ZOGLIN_ATTACK_ANIMATION_DURATION_TICKS)
    );
    assert_eq!(zoglin_event_attack_animation_ticks(3), None);
    assert_eq!(zoglin_next_attack_animation_ticks(10), 9);
    assert_eq!(zoglin_next_attack_animation_ticks(0), 0);
    assert!(zoglin_blocked_by_item_throws_target(false));
    assert!(!zoglin_blocked_by_item_throws_target(true));
    assert_eq!(zoglin_save_is_baby_key(), "IsBaby");
    assert!(zoglin_is_immune_to_regular_zombification());
    assert_eq!(ZOGLIN_HURT_RETARGET_DISTANCE_MARGIN, 4.0);
    assert_eq!(ZOGLIN_STEP_SOUND_VOLUME, 0.15);
    assert_eq!(ZOGLIN_STEP_SOUND_PITCH, 1.0);
}

fn assert_hoglin_attributes_and_spawn_rules() {
    assert_eq!(
        hoglin_attributes(),
        HoglinAttributes {
            max_health: 40.0,
            movement_speed: 0.3,
            knockback_resistance: 0.6,
            attack_knockback: 1.0,
            attack_damage: 6.0,
            xp_reward: 5,
        }
    );
    assert_eq!(
        hoglin_entity_type_surface(),
        HoglinEntityTypeSurface {
            width: 1.3964844,
            height: 1.4,
            passenger_attachment_y: 1.49375,
            client_tracking_range: 8,
        }
    );
    assert_eq!(hoglin_attack_damage(false), 6.0);
    assert_eq!(hoglin_attack_damage(true), 0.5);
    assert_eq!(hoglin_xp_reward(false), 5);
    assert_eq!(hoglin_xp_reward(true), 3);
    assert_eq!(hoglin_attack_interval_ticks(false), 40);
    assert_eq!(hoglin_attack_interval_ticks(true), 15);
    assert!(hoglin_finalize_spawn_is_baby(0.199));
    assert!(!hoglin_finalize_spawn_is_baby(0.2));
    assert!(!hoglin_spawn_allowed("minecraft:nether_wart_block"));
    assert!(hoglin_spawn_allowed("minecraft:crimson_nylium"));
    assert_eq!(
        hoglin_walk_target_value(true, "minecraft:crimson_nylium"),
        -1.0
    );
    assert_eq!(
        hoglin_walk_target_value(false, "minecraft:crimson_nylium"),
        10.0
    );
    assert_eq!(hoglin_walk_target_value(false, "minecraft:netherrack"), 0.0);
    assert!(hoglin_can_be_hunted(true, false));
    assert!(!hoglin_can_be_hunted(false, false));
    assert!(!hoglin_can_be_hunted(true, true));
    assert!(hoglin_can_fall_in_love(false, true));
    assert!(!hoglin_can_fall_in_love(true, true));
    assert!(hoglin_piglins_outnumber_hoglins(false, 3, 1));
    assert!(!hoglin_piglins_outnumber_hoglins(false, 2, 1));
    assert!(!hoglin_piglins_outnumber_hoglins(true, 3, 1));
}

fn assert_hoglin_target_and_hurt_response_rules() {
    assert_eq!(
        hoglin_on_hit_target_action(false, "minecraft:piglin", true),
        HoglinAiAction::BroadcastRetreat
    );
    assert_eq!(
        hoglin_on_hit_target_action(false, "minecraft:player", false),
        HoglinAiAction::BroadcastAttackTarget
    );
    assert_eq!(
        hoglin_on_hit_target_action(true, "minecraft:player", false),
        HoglinAiAction::None
    );
    assert_eq!(
        hoglin_was_hurt_action(true, "minecraft:player", false, false, true),
        HoglinAiAction::SetAvoidTarget
    );
    assert_eq!(
        hoglin_was_hurt_action(false, "minecraft:piglin", true, false, true),
        HoglinAiAction::None
    );
    assert_eq!(
        hoglin_was_hurt_action(false, "minecraft:player", false, false, true),
        HoglinAiAction::SetAttackTarget
    );
    assert_eq!(
        hoglin_was_hurt_action(false, "minecraft:hoglin", false, false, true),
        HoglinAiAction::None
    );
    assert!(hoglin_find_nearest_valid_attack_target(false, false, true));
    assert!(!hoglin_find_nearest_valid_attack_target(true, false, true));
    assert!(!hoglin_find_nearest_valid_attack_target(false, true, true));
}

fn assert_hoglin_social_ai_and_activity_rules() {
    assert_eq!(
        hoglin_activity_sound("avoid", false, false, false),
        Some("minecraft:entity.hoglin.retreat")
    );
    assert_eq!(
        hoglin_activity_sound("fight", false, false, false),
        Some("minecraft:entity.hoglin.angry")
    );
    assert_eq!(
        hoglin_activity_sound("idle", false, true, false),
        Some("minecraft:entity.hoglin.retreat")
    );
    assert_eq!(
        hoglin_activity_sound("idle", false, false, false),
        Some("minecraft:entity.hoglin.ambient")
    );
    assert_eq!(hoglin_activity_sound("fight", false, false, true), None);
    assert_eq!(hoglin_event_attack_animation_ticks(4), Some(10));
    assert_eq!(hoglin_event_attack_animation_ticks(3), None);
    assert_eq!(hoglin_next_attack_animation_ticks(10), 9);
    assert_eq!(hoglin_next_attack_animation_ticks(0), 0);
    assert!(hoglin_blocked_by_item_throws_target(false));
    assert!(!hoglin_blocked_by_item_throws_target(true));
}

fn assert_hoglin_constants() {
    assert_eq!(HOGLIN_REPELLENT_DETECTION_HORIZONTAL, 8);
    assert_eq!(HOGLIN_REPELLENT_DETECTION_VERTICAL, 4);
    assert_eq!(HOGLIN_REPELLENT_PACIFY_TIME, 200);
    assert_eq!(HOGLIN_RETREAT_MIN_SECONDS, 5);
    assert_eq!(HOGLIN_RETREAT_MAX_SECONDS, 20);
    assert_eq!(HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_IDLING, 8);
    assert_eq!(HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_RETREATING, 15);
    assert_eq!(HOGLIN_AVOID_REPELLENT_SPEED, 1.0);
    assert_eq!(HOGLIN_RETREAT_SPEED, 1.3);
    assert_eq!(HOGLIN_BREEDING_SPEED, 0.6);
    assert_eq!(HOGLIN_IDLE_SPEED, 0.4);
    assert_eq!(HOGLIN_BABY_FOLLOW_ADULT_SPEED, 0.6);
    assert_eq!(HOGLIN_ADULT_FOLLOW_RANGE_MIN, 5);
    assert_eq!(HOGLIN_ADULT_FOLLOW_RANGE_MAX, 16);
    assert_eq!(HOGLIN_LOOK_TARGET_RANGE, 8.0);
    assert_eq!(HOGLIN_LOOK_INTERVAL_MIN_TICKS, 30);
    assert_eq!(HOGLIN_LOOK_INTERVAL_MAX_TICKS, 60);
    assert_eq!(HOGLIN_DO_NOTHING_MIN_TICKS, 30);
    assert_eq!(HOGLIN_DO_NOTHING_MAX_TICKS, 60);
    assert_eq!(HOGLIN_STEP_SOUND_VOLUME, 0.15);
    assert_eq!(HOGLIN_STEP_SOUND_PITCH, 1.0);
}

fn assert_hoglin_conversion_rules() {
    assert_eq!(
        hoglin_conversion_tick(299, false, false, true),
        HoglinConversionTick {
            time_in_overworld: 300,
            convert_to_zoglin: false,
            nausea_ticks: 0,
        }
    );
    assert_eq!(
        hoglin_conversion_tick(300, false, false, true),
        HoglinConversionTick {
            time_in_overworld: 301,
            convert_to_zoglin: true,
            nausea_ticks: 200,
        }
    );
    assert_eq!(
        hoglin_conversion_tick(42, true, false, true),
        HoglinConversionTick {
            time_in_overworld: 0,
            convert_to_zoglin: false,
            nausea_ticks: 0,
        }
    );
    assert_eq!(
        hoglin_conversion_tick(42, false, true, true).time_in_overworld,
        0
    );
    assert_eq!(
        hoglin_conversion_tick(42, false, false, false).time_in_overworld,
        0
    );
}

fn assert_abstract_piglin_conversion_rules() {
    assert!(abstract_piglin_is_converting(false, false, true));
    assert!(!abstract_piglin_is_converting(true, false, true));
    assert!(!abstract_piglin_is_converting(false, true, true));
    assert!(!abstract_piglin_is_converting(false, false, false));
    assert_eq!(
        abstract_piglin_conversion_tick(299, false, false, true),
        AbstractPiglinConversionTick {
            time_in_overworld: 300,
            convert_to_zombified_piglin: false,
            nausea_ticks: 0,
            keep_equipment: true,
            preserve_can_pick_up_loot: true,
        }
    );
    assert_eq!(
        abstract_piglin_conversion_tick(300, false, false, true),
        AbstractPiglinConversionTick {
            time_in_overworld: 301,
            convert_to_zombified_piglin: true,
            nausea_ticks: 200,
            keep_equipment: true,
            preserve_can_pick_up_loot: true,
        }
    );
    assert_eq!(
        abstract_piglin_conversion_tick(42, false, true, true).time_in_overworld,
        0
    );
    assert_eq!(abstract_piglin_save_defaults(), (false, true, 0));
    assert_eq!(
        piglin_finish_conversion_plan(),
        PiglinFinishConversionPlan {
            cancel_admiring: true,
            drop_inventory: true,
            target_entity: "minecraft:zombified_piglin",
            conversion_type: ConversionTypeModel::Single,
            nausea_ticks: 200,
        }
    );
}

fn assert_piglin_brute_rules() {
    assert_eq!(
        piglin_brute_attributes(),
        PiglinBruteAttributes {
            max_health: 50.0,
            movement_speed: 0.35,
            attack_damage: 7.0,
            follow_range: 12.0,
            xp_reward: 20,
        }
    );
    assert_eq!(
        piglin_brute_default_main_hand_item(),
        "minecraft:golden_axe"
    );
    assert!(!piglin_brute_can_hunt());
    assert!(piglin_brute_wants_to_pick_up("minecraft:golden_axe", true));
    assert!(!piglin_brute_wants_to_pick_up("minecraft:gold_ingot", true));
    assert!(!piglin_brute_wants_to_pick_up(
        "minecraft:golden_axe",
        false
    ));
    assert_eq!(
        piglin_brute_ai_constants(),
        PiglinBruteAiConstants {
            anger_duration_ticks: 600,
            melee_attack_cooldown_ticks: 20,
            activity_sound_likelihood_per_tick: 0.0125,
            max_look_dist: 8.0,
            interaction_range: 8,
            idle_speed_multiplier: 0.6,
            home_close_enough_distance: 2,
            home_too_far_distance: 100,
            home_stroll_around_distance: 5,
        }
    );
    assert_eq!(
        piglin_brute_target_choice(true, true, true),
        PiglinBruteTargetChoice::AngryAt
    );
    assert_eq!(
        piglin_brute_target_choice(false, true, true),
        PiglinBruteTargetChoice::NearestVisibleAttackablePlayer
    );
    assert_eq!(
        piglin_brute_target_choice(false, false, true),
        PiglinBruteTargetChoice::NearestVisibleNemesis
    );
    assert_eq!(
        piglin_brute_target_choice(false, false, false),
        PiglinBruteTargetChoice::None
    );
    assert!(piglin_brute_retaliates_against(false));
    assert!(!piglin_brute_retaliates_against(true));
    assert_eq!(
        piglin_brute_arm_pose(true, true),
        "attacking_with_melee_weapon"
    );
    assert_eq!(piglin_brute_arm_pose(true, false), "default");
}

fn assert_hoglin_base_attack_damage_and_throw_rules() {
    assert_eq!(hoglin_base_attack_damage(false, 6.0, 5), 8.0);
    assert_eq!(hoglin_base_attack_damage(false, 6.0, 6), 3.0);
    assert_eq!(hoglin_base_attack_damage(true, 0.5, 0), 0.5);
    assert_eq!(hoglin_base_attack_damage(false, 0.0, 0), 0.0);

    let blocked_throw = HoglinThrowTargetInput {
        body_x: 0.0,
        body_z: 0.0,
        target_x: 4.0,
        target_z: 0.0,
        attack_knockback: 1.0,
        target_knockback_resistance: 1.0,
        random_y_rot_minus_10_to_10: 0.0,
        random_float_0_to_1_for_horizontal: 0.0,
        random_float_0_to_1_for_vertical: 0.0,
    };
    assert_eq!(hoglin_base_throw_target(blocked_throw), None);
    assert_eq!(
        hoglin_base_throw_target(HoglinThrowTargetInput {
            target_knockback_resistance: 0.25,
            random_float_0_to_1_for_vertical: 1.0,
            ..blocked_throw
        }),
        Some(HoglinBaseThrowVector {
            x: 0.15000000000000002,
            y: 0.375,
            z: 0.0,
            hurt_marked: true,
        })
    );
}
