#[test]
fn ravager_attack_stun_roar_and_leaf_griefing_match_java() {
    assert_ravager_attributes_targeting_and_controls();
    assert_ravager_ai_step_leaf_collision_and_immobility();
    assert_ravager_blocking_roar_and_spawn_rules();
}

fn assert_ravager_attributes_targeting_and_controls() {
    assert_eq!(
        ravager_attributes(),
        RavagerAttributes {
            max_health: 100.0,
            movement_speed: 0.3,
            knockback_resistance: 0.75,
            attack_damage: 12.0,
            attack_knockback: 1.5,
            follow_range: 32.0,
            step_height: 1.0,
            xp_reward: 20,
        }
    );
    assert_eq!(
        ravager_entity_type_surface(),
        RavagerEntityTypeSurface {
            width: 1.95,
            height: 2.2,
            passenger_attachment_y: 2.2625,
            passenger_attachment_z: -0.0625,
            client_tracking_range: 10,
            not_in_peaceful: true,
        }
    );
    assert_eq!(RAVAGER_LEAVES_PATHFINDING_MALUS, 0.0);
    assert_eq!(RAVAGER_MAX_HEAD_Y_ROT, 45);
    assert_eq!(RAVAGER_ATTACK_BB_DEFLATE_XZ, 0.05);
    assert_eq!(RAVAGER_RANDOM_STROLL_SPEED, 0.4);
    assert_eq!(RAVAGER_LOOK_AT_PLAYER_RANGE, 6.0);
    assert_eq!(RAVAGER_LOOK_AT_MOB_RANGE, 8.0);

    assert!(ravager_target_selector_matches("minecraft:player", false));
    assert!(ravager_target_selector_matches("minecraft:villager", false));
    assert!(!ravager_target_selector_matches("minecraft:villager", true));
    assert!(ravager_target_selector_matches(
        "minecraft:iron_golem",
        false
    ));
    assert!(!ravager_target_selector_matches("minecraft:zombie", false));

    assert_eq!(
        ravager_control_flags_enabled(false, false, false),
        (true, true, true, true)
    );
    assert_eq!(
        ravager_control_flags_enabled(true, true, false),
        (true, true, true, true)
    );
    assert_eq!(
        ravager_control_flags_enabled(true, false, false),
        (false, false, false, false)
    );
    assert_eq!(
        ravager_control_flags_enabled(true, false, true),
        (false, false, false, false)
    );
}

fn assert_ravager_ai_step_leaf_collision_and_immobility() {
    let ravager_step = RavagerAiStepInput {
        base_movement_speed: 0.3,
        has_target: false,
        immobile: false,
        attack_tick: 0,
        stunned_tick: 0,
        roar_tick: 0,
        horizontal_collision: false,
        mob_griefing: false,
        destroyed_leaves: false,
        on_ground: false,
    };
    assert_eq!(
        ravager_ai_step(RavagerAiStepInput {
            has_target: true,
            attack_tick: 10,
            ..ravager_step
        })
        .movement_speed,
        0.305
    );
    assert_eq!(
        ravager_ai_step(RavagerAiStepInput {
            base_movement_speed: 0.35,
            ..ravager_step
        })
        .movement_speed,
        0.345
    );
    assert_eq!(
        ravager_ai_step(RavagerAiStepInput {
            has_target: true,
            immobile: true,
            ..ravager_step
        })
        .movement_speed,
        0.0
    );
    assert!(
        ravager_ai_step(RavagerAiStepInput {
            roar_tick: 11,
            ..ravager_step
        })
        .roar_now
    );
    assert_eq!(
        ravager_ai_step(RavagerAiStepInput {
            attack_tick: 3,
            stunned_tick: 1,
            ..ravager_step
        }),
        RavagerAiStep {
            movement_speed: 0.3,
            attack_tick: 2,
            stunned_tick: 0,
            roar_tick: 20,
            roar_now: false,
            start_roar_sound: true,
            should_jump_after_leaf_collision: false,
        }
    );
    assert!(
        ravager_ai_step(RavagerAiStepInput {
            horizontal_collision: true,
            mob_griefing: true,
            on_ground: true,
            ..ravager_step
        })
        .should_jump_after_leaf_collision
    );
    assert!(
        !ravager_ai_step(RavagerAiStepInput {
            horizontal_collision: true,
            mob_griefing: true,
            destroyed_leaves: true,
            on_ground: true,
            ..ravager_step
        })
        .should_jump_after_leaf_collision
    );
    assert!(ravager_is_immobile(false, 1, 0, 0));
    assert!(ravager_is_immobile(false, 0, 1, 0));
    assert!(ravager_is_immobile(false, 0, 0, 1));
    assert!(!ravager_is_immobile(false, 0, 0, 0));
    assert!(!ravager_has_line_of_sight_allowed(1, 0, true));
    assert!(!ravager_has_line_of_sight_allowed(0, 1, true));
    assert!(ravager_has_line_of_sight_allowed(0, 0, true));
}

fn assert_ravager_blocking_roar_and_spawn_rules() {
    assert_eq!(
        ravager_blocked_by_item(0, true),
        RavagerBlockedByItem {
            stunned_tick: 40,
            roar_tick: 0,
            stun_event: Some(39),
            strong_knockback: false,
            defender_hurt_marked: true,
        }
    );
    assert_eq!(
        ravager_blocked_by_item(0, false),
        RavagerBlockedByItem {
            stunned_tick: 0,
            roar_tick: 0,
            stun_event: None,
            strong_knockback: true,
            defender_hurt_marked: true,
        }
    );
    assert!(!ravager_blocked_by_item(5, true).defender_hurt_marked);

    assert_eq!(
        ravager_roar_effect("minecraft:player", true, true),
        Some(RavagerRoarEffect {
            damage: Some(6.0),
            strong_knockback: false,
            include_armor_stand: false,
            event: Some(69),
        })
    );
    assert_eq!(
        ravager_roar_effect("minecraft:vindicator", true, true).and_then(|effect| effect.damage),
        None
    );
    assert_eq!(ravager_roar_effect("minecraft:ravager", true, true), None);
    assert_eq!(
        ravager_roar_effect("minecraft:armor_stand", true, false),
        None
    );
    assert_eq!(
        ravager_roar_effect("minecraft:armor_stand", true, true)
            .map(|effect| effect.include_armor_stand),
        Some(true)
    );
    assert_eq!(
        ravager_do_hurt_target_event(),
        (RAVAGER_ATTACK_DURATION, RAVAGER_ATTACK_EVENT_ID)
    );
    assert!(ravager_can_spawn_without_obstruction(false));
    assert!(!ravager_can_spawn_without_obstruction(true));
    assert!(!ravager_can_be_raid_leader());
}

#[test]
fn shulker_attach_peek_teleport_and_bullet_rules_match_java() {
    assert_shulker_surfaces_and_constants();
    assert_shulker_peek_color_and_scale_rules();
    assert_shulker_hurt_teleport_and_spawn_split_rules();
    assert_shulker_attack_and_bullet_rules();
}

fn assert_shulker_surfaces_and_constants() {
    assert_eq!(
        shulker_attributes(),
        ShulkerAttributes {
            max_health: 30.0,
            covered_armor_bonus: 20.0,
            xp_reward: 5,
        }
    );
    assert_eq!(
        shulker_entity_type_surface(),
        ShulkerEntityTypeSurface {
            width: 1.0,
            height: 1.0,
            eye_height: 0.5,
            client_tracking_range: 10,
            fire_immune: true,
            can_spawn_far_from_player: true,
        }
    );
    assert_eq!(
        shulker_bullet_surface(),
        ShulkerBulletSurface {
            width: 0.3125,
            height: 0.3125,
            client_tracking_range: 8,
            no_loot_table: true,
            no_physics: true,
        }
    );
    assert_eq!(SHULKER_DEFAULT_ATTACH_FACE, ShulkerDirection::Down);
    assert_eq!(SHULKER_DEFAULT_PEEK, 0);
    assert_eq!(SHULKER_DEFAULT_COLOR, 16);
    assert_eq!(SHULKER_NO_COLOR, 16);
    assert_eq!(SHULKER_TELEPORT_STEPS, 6);
    assert_eq!(SHULKER_MAX_TELEPORT_DISTANCE, 8);
    assert_eq!(SHULKER_TELEPORT_ATTEMPTS, 5);
    assert_eq!(SHULKER_OTHER_SCAN_RADIUS, 8.0);
    assert_eq!(SHULKER_LOOK_AT_PLAYER_RANGE, 8.0);
    assert_eq!(SHULKER_LOOK_AT_PLAYER_PROBABILITY, 0.02);
    assert_eq!(SHULKER_MAX_HEAD_X_ROT, 180);
    assert_eq!(SHULKER_MAX_HEAD_Y_ROT, 180);
    assert_eq!(SHULKER_RENDER_DISTANCE_SQR, 16384.0);
}

fn assert_shulker_peek_color_and_scale_rules() {
    assert_eq!(shulker_update_peek_amount(0.0, 30), 0.05);
    assert!((shulker_update_peek_amount(0.35, 30) - 0.3).abs() < f32::EPSILON);
    assert_eq!(shulker_update_peek_amount(1.0, 100), 1.0);
    assert_eq!(shulker_raw_peek_armor_bonus(0), Some(20.0));
    assert_eq!(shulker_raw_peek_armor_bonus(1), None);
    assert_eq!(shulker_color_from_data(0), Some(0));
    assert_eq!(shulker_color_from_data(15), Some(15));
    assert_eq!(shulker_color_from_data(16), None);
    assert_eq!(shulker_color_from_data(99), None);
    assert_eq!(shulker_sanitized_scale(2.5), 2.5);
    assert_eq!(shulker_sanitized_scale(4.0), 3.0);
}

fn assert_shulker_hurt_teleport_and_spawn_split_rules() {
    assert!(!shulker_hurt_allowed(0, "minecraft:arrow"));
    assert!(shulker_hurt_allowed(1, "minecraft:arrow"));
    assert!(shulker_hurt_allowed(0, "minecraft:trident"));
    assert!(shulker_should_teleport_after_hurt(14.9, 30.0, 0));
    assert!(!shulker_should_teleport_after_hurt(15.0, 30.0, 0));
    assert!(!shulker_should_teleport_after_hurt(14.9, 30.0, 1));

    assert_eq!(
        shulker_hit_by_bullet(1, true, 1, 0.0),
        ShulkerHitByBullet {
            should_spawn_baby: true,
            failure_chance: 0.0,
        }
    );
    assert_eq!(
        shulker_hit_by_bullet(1, true, 6, 0.99),
        ShulkerHitByBullet {
            should_spawn_baby: false,
            failure_chance: 1.0,
        }
    );
    assert!(!shulker_hit_by_bullet(0, true, 1, 0.0).should_spawn_baby);
    assert!(!shulker_hit_by_bullet(1, false, 1, 0.0).should_spawn_baby);
}

fn assert_shulker_attack_and_bullet_rules() {
    assert!(shulker_attack_can_use(true, false));
    assert!(!shulker_attack_can_use(true, true));
    assert!(!shulker_attack_can_use(false, false));
    assert_eq!(
        shulker_attack_tick(1, false, true, 399.9, 3),
        ShulkerAttackTick {
            attack_time: 35,
            raw_peek: 100,
            shoot_bullet: true,
            clear_target: false,
        }
    );
    assert_eq!(
        shulker_attack_tick(20, false, true, 400.0, 0),
        ShulkerAttackTick {
            attack_time: 19,
            raw_peek: 100,
            shoot_bullet: false,
            clear_target: true,
        }
    );
    assert_eq!(
        shulker_defense_search_inflate(ShulkerDirection::East, 16.0),
        (4.0, 16.0, 16.0)
    );
    assert_eq!(
        shulker_defense_search_inflate(ShulkerDirection::North, 16.0),
        (16.0, 16.0, 4.0)
    );
    assert_eq!(
        shulker_defense_search_inflate(ShulkerDirection::Up, 16.0),
        (16.0, 4.0, 16.0)
    );

    assert_eq!(shulker_bullet_on_hit_entity(true, true), Some((4.0, 200)));
    assert_eq!(shulker_bullet_on_hit_entity(false, true), None);
    assert_eq!(SHULKER_BULLET_SPEED, 0.15);
    assert_eq!(SHULKER_BULLET_GRAVITY, 0.04);
    assert!(shulker_bullet_discards_in_peaceful(true));
    assert!(!shulker_bullet_discards_in_peaceful(false));
}

#[test]
fn giant_attributes_dimensions_and_spawn_surface_match_java_rules() {
    assert_eq!(
        giant_attributes(),
        GiantAttributes {
            max_health: 100.0,
            movement_speed: 0.5,
            attack_damage: 50.0,
            camera_distance: 16.0,
        }
    );
    assert_eq!(
        giant_entity_type_surface(),
        GiantEntityTypeSurface {
            width: 3.6,
            height: 12.0,
            eye_height: 10.44,
            riding_offset: -3.75,
            client_tracking_range: 10,
            not_in_peaceful: true,
            natural_spawn: false,
            custom_ai_goals: 0,
        }
    );
    assert_eq!(giant_walk_target_value(0.42), 0.42);
}

#[test]
fn zombie_conversion_baby_reinforcement_and_spawn_gates_match_java_rules() {
    assert_zombie_attributes_baby_and_water_conversion();
    assert_zombie_reinforcement_item_and_villager_conversion_rules();
    assert_zombie_finalize_spawn_rules();
}

fn assert_zombie_attributes_baby_and_water_conversion() {
    assert_eq!(
        zombie_attributes(),
        ZombieAttributes {
            follow_range: 35.0,
            movement_speed: 0.23,
            attack_damage: 3.0,
            armor: 2.0,
            baby_speed_modifier: 0.5,
        }
    );
    assert_eq!(
        zombie_baby_dimensions(),
        ZombieBabyDimensions {
            width: 0.49,
            height: 0.99,
            eye_height: 0.775,
            vehicle_attachment_y: 0.1875,
        }
    );
    assert!(zombie_is_sun_sensitive());
    assert_eq!(zombie_baby_xp_reward(5, true), 12);
    assert_eq!(zombie_baby_xp_reward(5, false), 5);
    assert!(zombie_spawn_as_baby(0.049));
    assert!(!zombie_spawn_as_baby(0.05));

    assert_eq!(
        zombie_water_conversion_tick(false, true, false, -1, 599, true),
        (600, 300, true, None)
    );
    assert_eq!(
        zombie_water_conversion_tick(false, true, true, 0, 600, true),
        (600, -1, true, Some("minecraft:drowned"))
    );
    assert_eq!(
        zombie_water_conversion_tick(false, true, false, -1, 4, false),
        (-1, -1, false, None)
    );
    assert_eq!(
        zombie_water_conversion_tick(true, true, false, -1, 599, true),
        (599, -1, false, None)
    );
    assert_eq!(zombie_drowned_conversion_event(false), Some(1040));
    assert_eq!(zombie_drowned_conversion_event(true), None);
    assert_eq!(
        zombie_fire_on_hit_seconds(true, true, true, 0.59, 2.0),
        Some(4)
    );
    assert_eq!(zombie_fire_on_hit_seconds(true, true, true, 0.6, 2.0), None);
    assert_eq!(
        zombie_fire_on_hit_seconds(true, false, true, 0.0, 3.0),
        None
    );
}

fn assert_zombie_reinforcement_item_and_villager_conversion_rules() {
    assert_eq!(
        zombie_reinforcement_attempt(true, true, true, 0.04, 0.05, true, true),
        ZombieReinforcementOutcome {
            attempts: 50,
            spawned: true,
            caller_reinforcement_delta: -0.05,
            callee_reinforcement_delta: -0.05,
        }
    );
    assert_eq!(
        zombie_reinforcement_attempt(true, true, false, 0.0, 1.0, true, true).attempts,
        0
    );
    assert_eq!(ZOMBIE_REINFORCEMENT_RANGE_MIN, 7);
    assert_eq!(ZOMBIE_REINFORCEMENT_RANGE_MAX, 40);
    assert_eq!(
        zombie_default_main_hand_item(false, 0.009, 0),
        Some("minecraft:iron_sword")
    );
    assert_eq!(
        zombie_default_main_hand_item(true, 0.049, 1),
        Some("minecraft:iron_spear")
    );
    assert_eq!(
        zombie_default_main_hand_item(true, 0.049, 2),
        Some("minecraft:iron_shovel")
    );
    assert_eq!(zombie_default_main_hand_item(false, 0.01, 0), None);
    assert!(!zombie_can_hold_item("minecraft:egg", true, true));
    assert!(zombie_can_hold_item("minecraft:egg", true, false));
    assert!(!zombie_wants_to_pick_up("minecraft:glow_ink_sac"));
    assert!(zombie_wants_to_pick_up("minecraft:rotten_flesh"));

    assert_eq!(
        zombie_killed_villager_outcome("easy", true, false, true, false),
        ZombieVillagerConversionOutcome::NoConversion
    );
    assert_eq!(
        zombie_killed_villager_outcome("normal", true, true, true, false),
        ZombieVillagerConversionOutcome::PerishedNormally
    );
    assert_eq!(
        zombie_killed_villager_outcome("hard", true, false, true, false),
        ZombieVillagerConversionOutcome::Converted {
            preserve_villager_data: true,
            preserve_gossips: true,
            preserve_trade_offers: true,
            preserve_xp: true,
            level_event: Some(1026),
        }
    );
}

fn assert_zombie_finalize_spawn_rules() {
    let spawn_input = ZombieFinalizeSpawnInput {
        spawn_reason_conversion: false,
        spawn_reason_load_or_dimension_travel: false,
        group_data_present: false,
        group_baby: false,
        group_can_spawn_jockey: true,
        spawn_baby_random_float: 0.049,
        loot_random_float: 0.54,
        difficulty_special_multiplier: 1.0,
        existing_chicken_random_float: 0.06,
        existing_chicken_available: true,
        new_chicken_random_float: 0.04,
        door_random_float: 0.09,
        halloween: true,
        head_empty: true,
        halloween_head_random_float: 0.24,
        jack_o_lantern_random_float: 0.09,
        reinforcement_base_random_double: 0.5,
        knockback_random_double: 0.5,
        follow_range_random_double: 1.0,
        leader_random_float: 0.04,
        leader_reinforcement_random_double: 1.0,
        leader_health_random_double: 1.0,
    };
    let spawned = zombie_finalize_spawn_outcome(spawn_input);
    assert_eq!(spawned.can_pick_up_loot, Some(true));
    assert!(spawned.is_baby);
    assert!(!spawned.tried_existing_chicken_jockey);
    assert!(spawned.spawned_new_chicken_jockey);
    assert!(spawned.can_break_doors);
    assert_eq!(spawned.halloween_head, Some("minecraft:jack_o_lantern"));
    assert_eq!(spawned.halloween_head_drop_chance, Some(0.0));
    assert_eq!(spawned.reinforcement_base_chance, 0.05);
    assert_eq!(spawned.knockback_resistance_bonus, 0.025);
    assert_eq!(spawned.follow_range_bonus, Some(1.5));
    assert_eq!(spawned.leader_reinforcement_bonus, Some(0.75));
    assert_eq!(spawned.leader_max_health_bonus, Some(4.0));
    assert!(spawned.reset_health_to_max);
    let conversion_spawn = zombie_finalize_spawn_outcome(ZombieFinalizeSpawnInput {
        spawn_reason_conversion: true,
        group_data_present: true,
        group_baby: true,
        group_can_spawn_jockey: false,
        door_random_float: 1.0,
        halloween: false,
        leader_random_float: 1.0,
        leader_reinforcement_random_double: 0.0,
        leader_health_random_double: 0.0,
        ..spawn_input
    });
    assert_eq!(conversion_spawn.can_pick_up_loot, None);
    assert!(conversion_spawn.is_baby);
    assert!(!conversion_spawn.can_break_doors);
    assert!(!conversion_spawn.reset_health_to_max);
}

#[test]
fn zombie_villager_cure_timer_preservation_and_reputation_match_java_rules() {
    assert_eq!(
        zombie_villager_baby_dimensions(),
        ZombieVillagerBabyDimensions {
            width: 0.49,
            height: 0.99,
            eye_height: 0.67,
            vehicle_attachment_y: 0.125,
        }
    );
    assert!(zombie_villager_finalize_spawn_sets_biome_type(false));
    assert!(!zombie_villager_finalize_spawn_sets_biome_type(true));
    assert_eq!(zombie_villager_conversion_time_from_roll(0), 3600);
    assert_eq!(zombie_villager_conversion_time_from_roll(2400), 6000);
    assert_eq!(ZOMBIE_VILLAGER_NOT_CONVERTING, -1);
    assert_eq!(
        zombie_villager_interact("minecraft:stick", true, true, 2, 3600),
        ZombieVillagerInteraction::PassToZombie
    );
    assert_eq!(
        zombie_villager_interact("minecraft:golden_apple", false, true, 2, 3600),
        ZombieVillagerInteraction::ConsumeGoldenApple
    );
    assert_eq!(
        zombie_villager_interact("minecraft:golden_apple", true, true, 3, 4200),
        ZombieVillagerInteraction::StartConversion {
            consumed_golden_apple: true,
            remove_weakness: true,
            strength_effect_ticks: 4200,
            strength_amplifier: 0,
            broadcast_event: 16,
        }
    );

    assert!(zombie_villager_remove_when_far_away(false, 0));
    assert!(!zombie_villager_remove_when_far_away(true, 0));
    assert!(!zombie_villager_remove_when_far_away(false, 1));
    assert_eq!(zombie_villager_conversion_progress(0.01, 14, 14), 1);
    assert_eq!(zombie_villager_conversion_progress(0.009, 20, 20), 15);
    assert_eq!(
        zombie_villager_conversion_tick(true, true, false, 3, 2),
        ZombieVillagerConversionTick {
            conversion_time: 1,
            finished: false,
        }
    );
    assert_eq!(
        zombie_villager_conversion_tick(true, true, false, 1, 2),
        ZombieVillagerConversionTick {
            conversion_time: -1,
            finished: true,
        }
    );
    assert_eq!(
        zombie_villager_conversion_tick(true, true, true, 1, 2),
        ZombieVillagerConversionTick {
            conversion_time: 1,
            finished: false,
        }
    );

    assert_eq!(
        zombie_villager_finish_conversion(true, true, false),
        ZombieVillagerFinishConversion {
            target_entity: "minecraft:villager",
            copy_position_motion_vehicle_passengers: false,
            preserve_non_binding_equipment: true,
            preserve_villager_data: true,
            preserve_gossips: true,
            copy_trade_offers: true,
            preserve_xp: true,
            finalize_spawn_reason: "conversion",
            refresh_brain: true,
            trigger_cured_advancement: true,
            emit_reputation_event: true,
            nausea_ticks: 200,
            level_event: Some(1027),
        }
    );
    assert_eq!(
        zombie_villager_finish_conversion(true, false, true).level_event,
        None
    );
    assert!(zombie_villager_set_villager_data_clears_offers(true, true));
    assert!(!zombie_villager_set_villager_data_clears_offers(
        false, true
    ));
}
