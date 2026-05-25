use crate::mob_interaction::*;

#[test]
fn ghast_fireball_spawn_and_movement_gates_match_java_rules() {
    assert_eq!(
        ghast_attributes(),
        GhastAttributes {
            max_health: 10.0,
            follow_range: 100.0,
            camera_distance: 8.0,
            flying_speed: 0.06,
            xp_reward: 5,
        }
    );
    assert_eq!(
        ghast_entity_type_surface(),
        GhastEntityTypeSurface {
            width: 4.0,
            height: 4.0,
            eye_height: 2.6,
            passenger_attachment_y: 4.0625,
            riding_offset: 0.5,
            client_tracking_range: 10,
            fire_immune: true,
            not_in_peaceful: true,
        }
    );
    assert!(ghast_spawn_allowed(false, 0, true));
    assert!(!ghast_spawn_allowed(true, 0, true));
    assert!(!ghast_spawn_allowed(false, 1, true));
    assert!(!ghast_spawn_allowed(false, 0, false));
    assert!(ghast_target_predicate_matches(4.0));
    assert!(!ghast_target_predicate_matches(4.01));
    assert_eq!(GHAST_SOUND_VOLUME, 5.0);
    assert_eq!(GHAST_DEFAULT_EXPLOSION_POWER, 1);
    assert_eq!(GHAST_LEASH_ELASTIC_DISTANCE, 10.0);
    assert_eq!(GHAST_LEASH_SNAP_DISTANCE, 16.0);

    assert!(ghast_is_reflected_fireball(
        "minecraft:fireball",
        "minecraft:player"
    ));
    assert!(!ghast_is_reflected_fireball(
        "minecraft:small_fireball",
        "minecraft:player"
    ));
    assert_eq!(ghast_hurt_damage(true, true, 1.0), Some(1000.0));
    assert_eq!(ghast_hurt_damage(false, true, 6.0), None);
    assert_eq!(ghast_hurt_damage(false, false, 6.0), Some(6.0));
    assert_eq!(GHAST_FIREBALL_ENTITY_DAMAGE, 6.0);

    assert_eq!(
        ghast_shoot_fireball_tick(9, true, 4095.9, true, false, 1),
        GhastShootTick {
            charge_time: 10,
            charging: false,
            warn_level_event: Some(1015),
            shoot_level_event: None,
            fireball: None,
        }
    );
    assert!(ghast_shoot_fireball_tick(10, true, 4095.9, true, false, 1).charging);
    assert_eq!(
        ghast_shoot_fireball_tick(19, true, 4095.9, true, false, 3),
        GhastShootTick {
            charge_time: -40,
            charging: false,
            warn_level_event: None,
            shoot_level_event: Some(1016),
            fireball: Some(GhastFireballPlan {
                spawn_offset: 4.0,
                y_offset_from_ghast_mid: 0.5,
                explosion_power: 3,
            }),
        }
    );
    assert_eq!(
        ghast_shoot_fireball_tick(19, true, 4095.9, true, true, 1).shoot_level_event,
        None
    );
    assert_eq!(
        ghast_shoot_fireball_tick(3, true, 4096.0, true, false, 1).charge_time,
        2
    );
    assert_eq!(
        ghast_shoot_fireball_tick(0, true, 4096.0, true, false, 1).charge_time,
        0
    );
    assert_eq!(
        ghast_shoot_fireball_tick(12, false, 0.0, true, false, 1),
        GhastShootTick {
            charge_time: 12,
            charging: false,
            warn_level_event: None,
            shoot_level_event: None,
            fireball: None,
        }
    );

    assert!(ghast_random_float_can_use(false, 100.0));
    assert!(ghast_random_float_can_use(true, 0.99));
    assert!(ghast_random_float_can_use(true, 3600.01));
    assert!(!ghast_random_float_can_use(true, 1.0));
    assert!(!ghast_random_float_can_use(true, 3600.0));
    assert_eq!(GHAST_RANDOM_FLOAT_MAX_ATTEMPTS, 64);
    assert_eq!(
        ghast_random_float_target((10.0, 20.0, 30.0), 0.0, 0.5, 1.0),
        (-6.0, 20.0, 46.0)
    );
    assert_eq!(ghast_move_float_duration_tick(0, 4), 5);
    assert_eq!(GHAST_MOVE_ACCELERATION_SCALE, 5.0 / 3.0);
    assert_eq!(large_fireball_hit_outcome(true, 2), (2.0, true, true));
    assert_eq!(large_fireball_hit_outcome(false, 1), (1.0, false, true));
}

#[test]
fn strider_lava_saddle_suffocation_and_jockey_rules_match_java() {
    assert_eq!(
        strider_attributes(),
        StriderAttributes {
            movement_speed: 0.175
        }
    );
    assert_eq!(
        strider_entity_type_surface(),
        StriderEntityTypeSurface {
            width: 0.9,
            height: 1.7,
            client_tracking_range: 10,
            fire_immune: true,
        }
    );
    assert_eq!(STRIDER_WATER_PATHFINDING_MALUS, -1.0);
    assert_eq!(STRIDER_LAVA_PATHFINDING_MALUS, 0.0);
    assert_eq!(STRIDER_FIRE_PATHFINDING_MALUS, 0.0);
    assert!(strider_can_stand_on_fluid("minecraft:lava"));
    assert!(!strider_can_stand_on_fluid("minecraft:water"));
    assert!(strider_spawn_allowed(true));
    assert!(!strider_spawn_allowed(false));
    assert!(strider_can_use_saddle_slot(true, false));
    assert!(!strider_can_use_saddle_slot(true, true));
    assert!(!strider_can_use_saddle_slot(false, false));

    assert!(strider_controlling_passenger(true, true, true));
    assert!(!strider_controlling_passenger(true, true, false));
    assert_eq!(strider_ridden_speed(0.175, false, 1.0), 0.09625);
    assert!((strider_ridden_speed(0.175, true, 1.0) - 0.06125).abs() < 0.000001);
    assert_eq!(STRIDER_SUFFOCATE_STEERING_MODIFIER, 0.35);
    assert_eq!(STRIDER_STEERING_MODIFIER, 0.55);
    assert_eq!(STRIDER_SUFFOCATING_MODIFIER, -0.34);

    assert_eq!(
        strider_suffocation_state(false, false, false, false, false),
        Some(StriderSuffocationState {
            suffocating: true,
            movement_speed_modifier: Some(-0.34),
        })
    );
    assert_eq!(
        strider_suffocation_state(false, false, false, true, false),
        Some(StriderSuffocationState {
            suffocating: false,
            movement_speed_modifier: None,
        })
    );
    assert_eq!(
        strider_suffocation_state(false, false, false, false, true).map(|state| state.suffocating),
        Some(false)
    );
    assert_eq!(
        strider_suffocation_state(true, false, false, false, false),
        None
    );
    assert_eq!(strider_walk_target_value(true, false), 10.0);
    assert_eq!(strider_walk_target_value(false, false), 0.0);
    assert_eq!(strider_walk_target_value(false, true), f32::NEG_INFINITY);

    assert!(strider_can_add_passenger(false, false));
    assert!(!strider_can_add_passenger(true, false));
    assert!(!strider_can_add_passenger(false, true));
    assert!(strider_interaction_starts_riding(false, true, false, false));
    assert!(!strider_interaction_starts_riding(true, true, false, false));
    assert_eq!(STRIDER_HAPPY_SOUND_RANDOM_BOUND, 140);
    assert_eq!(STRIDER_RETREAT_SOUND_RANDOM_BOUND, 60);
    assert_eq!(STRIDER_STEP_DISTANCE_INCREMENT, 0.6);
    assert_eq!(STRIDER_STEP_SOUND_VOLUME, 1.0);
    assert_eq!(STRIDER_STEP_SOUND_PITCH, 1.0);
    assert_eq!(STRIDER_LIQUID_COLLISION_HEIGHT, 8.0);
    assert_eq!(
        strider_float_in_lava(true, true, false, (0.2, -0.1, 0.4)),
        (true, (0.2, -0.1, 0.4))
    );
    assert_eq!(
        strider_float_in_lava(true, false, false, (0.2, -0.1, 0.4)),
        (false, (0.1, 0.0, 0.2))
    );
    assert_eq!(
        strider_float_in_lava(false, false, false, (0.2, -0.1, 0.4)),
        (false, (0.2, -0.1, 0.4))
    );

    assert!(strider_go_to_lava_can_use(false, true));
    assert!(!strider_go_to_lava_can_use(true, true));
    assert!(strider_go_to_lava_can_continue(false, true));
    assert!(!strider_go_to_lava_can_continue(true, true));
    assert!(strider_go_to_lava_valid_target("minecraft:lava", true));
    assert!(!strider_go_to_lava_valid_target("minecraft:lava", false));
    assert_eq!(STRIDER_GO_TO_LAVA_SEARCH_RANGE, 8);
    assert_eq!(STRIDER_GO_TO_LAVA_VERTICAL_SEARCH_RANGE, 2);
    assert_eq!(STRIDER_RANDOM_STROLL_INTERVAL, 60);
    assert!(strider_navigation_valid_path_type("lava", false));
    assert!(strider_navigation_valid_path_type("fire", false));
    assert!(strider_navigation_valid_path_type(
        "fire_in_neighbor",
        false
    ));
    assert!(!strider_navigation_valid_path_type("water", false));
    assert!(strider_navigation_valid_path_type("water", true));

    assert_eq!(
        strider_finalize_spawn(false, 0, 9),
        StriderFinalizeSpawn::ZombifiedPiglinJockey {
            jockey_holds: "minecraft:warped_fungus_on_a_stick",
            strider_saddle: "minecraft:saddle",
            guaranteed_saddle_drop: true,
        }
    );
    assert_eq!(
        strider_finalize_spawn(false, 1, 0),
        StriderFinalizeSpawn::BabyStriderJockey { baby_age: -24000 }
    );
    assert_eq!(
        strider_finalize_spawn(false, 1, 1),
        StriderFinalizeSpawn::AgeableGroup { baby_chance: 50 }
    );
    assert_eq!(
        strider_finalize_spawn(true, 0, 0),
        StriderFinalizeSpawn::None
    );
    assert!(strider_is_sensitive_to_water());
    assert!(!strider_is_on_fire());
}

#[test]
fn witch_drinking_throwing_and_raid_gates_match_java_rules() {
    assert_eq!(
        witch_attributes(),
        WitchAttributes {
            max_health: 26.0,
            movement_speed: 0.25,
        }
    );
    assert_eq!(WITCH_RANGED_ATTACK_SPEED, 1.0);
    assert_eq!(WITCH_RANGED_ATTACK_INTERVAL_TICKS, 60);
    assert_eq!(WITCH_RANGED_ATTACK_RADIUS, 10.0);
    assert_eq!(WITCH_RANDOM_STROLL_SPEED, 1.0);
    assert_eq!(WITCH_LOOK_AT_PLAYER_RANGE, 8.0);
    assert!(witch_heal_raiders_goal_enabled(true, "minecraft:pillager"));
    assert!(!witch_heal_raiders_goal_enabled(true, "minecraft:witch"));
    assert!(!witch_heal_raiders_goal_enabled(
        false,
        "minecraft:pillager"
    ));
    assert!(witch_attack_players_enabled(0));
    assert!(!witch_attack_players_enabled(1));

    let no_drink = WitchDrinkPotionInput {
        water_roll: 1.0,
        fire_roll: 1.0,
        heal_roll: 1.0,
        speed_roll: 1.0,
        eye_in_water: false,
        has_water_breathing: false,
        on_fire_or_fire_damage: false,
        has_fire_resistance: false,
        health: 26.0,
        max_health: 26.0,
        target_present: false,
        has_speed: false,
        target_distance_sqr: 0.0,
    };
    assert_eq!(
        witch_select_drink_potion(WitchDrinkPotionInput {
            water_roll: 0.149,
            eye_in_water: true,
            ..no_drink
        }),
        Some("minecraft:water_breathing")
    );
    assert_eq!(
        witch_select_drink_potion(WitchDrinkPotionInput {
            fire_roll: 0.149,
            on_fire_or_fire_damage: true,
            ..no_drink
        }),
        Some("minecraft:fire_resistance")
    );
    assert_eq!(
        witch_select_drink_potion(WitchDrinkPotionInput {
            heal_roll: 0.049,
            health: 25.0,
            ..no_drink
        }),
        Some("minecraft:healing")
    );
    assert_eq!(
        witch_select_drink_potion(WitchDrinkPotionInput {
            speed_roll: 0.499,
            target_present: true,
            target_distance_sqr: 121.1,
            ..no_drink
        }),
        Some("minecraft:swiftness")
    );
    assert_eq!(
        witch_select_drink_potion(WitchDrinkPotionInput {
            water_roll: 0.15,
            fire_roll: 0.15,
            heal_roll: 0.05,
            speed_roll: 0.5,
            eye_in_water: true,
            on_fire_or_fire_damage: true,
            health: 25.0,
            target_present: true,
            target_distance_sqr: 122.0,
            ..no_drink
        }),
        None
    );
    assert_eq!(
        witch_start_drinking(Some("minecraft:healing"), false),
        Some(WitchDrinkStart {
            potion: "minecraft:healing",
            using_item: true,
            speed_modifier: -0.25,
            drink_sound: Some("minecraft:entity.witch.drink"),
        })
    );
    assert_eq!(witch_start_drinking(None, false), None);
    assert_eq!(
        witch_finish_drinking(true, true),
        WitchDrinkFinish {
            using_item: false,
            clear_main_hand: true,
            apply_potion_effects: true,
            game_event: "minecraft:drink",
            remove_speed_modifier: true,
        }
    );
    assert!(!witch_finish_drinking(true, false).apply_potion_effects);
    let ranged_attack = WitchRangedAttackInput {
        drinking_potion: false,
        target_is_raider: false,
        target_health: 20.0,
        target_has_slowness: false,
        target_has_poison: false,
        target_has_weakness: false,
        horizontal_distance: 10.0,
        weakness_roll: 0.0,
        silent: false,
    };
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            drinking_potion: true,
            ..ranged_attack
        }),
        None
    );
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            target_is_raider: true,
            target_health: 4.0,
            horizontal_distance: 1.0,
            weakness_roll: 1.0,
            ..ranged_attack
        }),
        Some(WitchRangedAttack {
            potion: "minecraft:healing",
            clear_target: true,
            velocity: 0.45,
            inaccuracy: 8.0,
            throw_sound: Some("minecraft:entity.witch.throw"),
        })
    );
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            target_is_raider: true,
            target_health: 4.1,
            horizontal_distance: 4.0,
            weakness_roll: 1.0,
            silent: true,
            ..ranged_attack
        })
        .map(|attack| attack.potion),
        Some("minecraft:regeneration")
    );
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            horizontal_distance: 8.0,
            weakness_roll: 1.0,
            ..ranged_attack
        })
        .map(|attack| attack.potion),
        Some("minecraft:slowness")
    );
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            target_health: 8.0,
            target_has_slowness: true,
            horizontal_distance: 4.0,
            weakness_roll: 1.0,
            ..ranged_attack
        })
        .map(|attack| attack.potion),
        Some("minecraft:poison")
    );
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            target_health: 7.9,
            target_has_slowness: true,
            target_has_poison: true,
            horizontal_distance: 3.0,
            weakness_roll: 0.249,
            ..ranged_attack
        })
        .map(|attack| attack.potion),
        Some("minecraft:weakness")
    );
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            target_health: 7.9,
            target_has_slowness: true,
            target_has_poison: true,
            horizontal_distance: 3.0,
            weakness_roll: 0.25,
            ..ranged_attack
        })
        .map(|attack| attack.potion),
        Some("minecraft:harming")
    );
    assert_eq!(
        witch_ranged_attack(WitchRangedAttackInput {
            target_has_slowness: true,
            horizontal_distance: 2.0,
            weakness_roll: 1.0,
            ..ranged_attack
        })
        .map(|attack| attack.velocity),
        Some(0.45)
    );
    assert_eq!(witch_projectile_y_adjustment(8.0), 1.6);
    assert_eq!(witch_damage_after_magic_absorb(true, true, 10.0), 0.0);
    assert_eq!(witch_damage_after_magic_absorb(false, true, 10.0), 1.5);
    assert_eq!(witch_damage_after_magic_absorb(false, false, 10.0), 10.0);
    assert_eq!(WITCH_PARTICLE_EVENT_ID, 15);
    assert_eq!(WITCH_PARTICLE_CHANCE, 7.5E-4);
    assert_eq!(witch_particle_count(0), 10);
    assert_eq!(witch_particle_count(34), 44);
    assert_eq!(
        (WITCH_CAN_BE_RAID_LEADER, WITCH_RAID_BUFFS_APPLIED),
        (false, false)
    );
    assert!(!witch_finalize_can_join_raid(true));
    assert!(witch_finalize_can_join_raid(false));
    assert!(!witch_ravager_rider_in_java_26_1_2());
}

#[test]
fn guardian_beam_thorns_and_elder_curse_rules_match_java() {
    assert_eq!(
        guardian_attributes(),
        GuardianAttributes {
            max_health: 30.0,
            movement_speed: 0.5,
            attack_damage: 6.0,
            xp_reward: 10,
        }
    );
    assert_eq!(
        elder_guardian_attributes(),
        GuardianAttributes {
            max_health: 80.0,
            movement_speed: 0.3,
            attack_damage: 8.0,
            xp_reward: 10,
        }
    );
    assert_eq!(
        guardian_entity_type_surface(),
        GuardianEntityTypeSurface {
            width: 0.85,
            height: 0.85,
            eye_height: 0.425,
            passenger_attachment_y: 0.975,
            client_tracking_range: 8,
            not_in_peaceful: true,
        }
    );
    assert_eq!(
        elder_guardian_entity_type_surface(),
        GuardianEntityTypeSurface {
            width: 1.9975,
            height: 1.9975,
            eye_height: 0.99875,
            passenger_attachment_y: 2.350625,
            client_tracking_range: 10,
            not_in_peaceful: true,
        }
    );
    assert_eq!(GUARDIAN_WATER_PATHFINDING_MALUS, 0.0);
    assert_eq!(GUARDIAN_AMBIENT_SOUND_INTERVAL, 160);
    assert_eq!(GUARDIAN_MAX_HEAD_X_ROT, 180);
    assert_eq!(GUARDIAN_ATTACK_START_TICKS, -10);
    assert_eq!(GUARDIAN_ATTACK_DURATION_TICKS, 80);
    assert_eq!(ELDER_GUARDIAN_ATTACK_DURATION_TICKS, 60);
    assert!(guardian_spawn_allowed(0, true, false, false, true, true));
    assert!(guardian_spawn_allowed(1, false, false, false, true, true));
    assert!(guardian_spawn_allowed(0, true, false, true, false, true));
    assert!(!guardian_spawn_allowed(1, true, false, true, false, true));
    assert!(!guardian_spawn_allowed(1, true, false, false, true, true));
    assert!(!guardian_spawn_allowed(0, true, true, false, true, true));
    assert!(!guardian_spawn_allowed(0, true, false, false, false, true));
    assert!(!guardian_spawn_allowed(0, true, false, false, true, false));

    assert!(guardian_attack_selector_matches("minecraft:player", 9.1));
    assert!(guardian_attack_selector_matches("minecraft:squid", 10.0));
    assert!(guardian_attack_selector_matches("minecraft:axolotl", 10.0));
    assert!(!guardian_attack_selector_matches("minecraft:zombie", 10.0));
    assert!(!guardian_attack_selector_matches("minecraft:player", 9.0));
    assert!(guardian_attack_can_continue(false, true, 9.1, true));
    assert!(!guardian_attack_can_continue(false, true, 9.0, true));
    assert!(guardian_attack_can_continue(true, false, 0.0, true));

    assert_eq!(
        guardian_attack_tick(-1, 42, true, false, false, false),
        GuardianAttackTick {
            attack_time: 0,
            active_attack_target: Some(42),
            broadcast_event: Some(21),
            magic_damage: None,
            melee_hit: false,
            clear_target: false,
        }
    );
    assert_eq!(
        guardian_attack_tick(79, 42, true, false, true, false),
        GuardianAttackTick {
            attack_time: 80,
            active_attack_target: None,
            broadcast_event: None,
            magic_damage: Some(3.0),
            melee_hit: true,
            clear_target: true,
        }
    );
    assert_eq!(
        guardian_attack_tick(59, 42, true, false, true, true).magic_damage,
        Some(5.0)
    );
    assert!(guardian_attack_tick(5, 42, false, false, false, false).clear_target);

    assert_eq!(guardian_thorns_damage(false, false, false, true), Some(2.0));
    assert_eq!(guardian_thorns_damage(true, false, false, true), None);
    assert_eq!(guardian_thorns_damage(false, true, false, true), None);
    assert_eq!(guardian_thorns_damage(false, false, true, true), None);
    assert_eq!(guardian_thorns_damage(false, false, false, false), None);
    assert_eq!(guardian_walk_target_value(true, 0.25, 0.0), 10.25);
    assert_eq!(guardian_walk_target_value(false, 0.25, 0.5), 0.5);
    assert_eq!(GUARDIAN_AIR_SUPPLY_IN_WATER, 300);
    assert_eq!(GUARDIAN_LAND_FLOP_Y_PUSH, 0.5);
    assert_eq!(GUARDIAN_LAND_FLOP_XZ_SCALE, 0.4);
    assert_eq!(GUARDIAN_TRAVEL_WATER_RELATIVE, 0.1);
    assert_eq!(GUARDIAN_TRAVEL_WATER_DAMPING, 0.9);
    assert_eq!(GUARDIAN_IDLE_SINKING_Y, -0.005);

    assert_eq!(
        elder_guardian_effect_pulse(1199, 1, false),
        Some(ElderGuardianEffectPulse {
            mining_fatigue_ticks: 6000,
            amplifier: 2,
            radius: 50.0,
            display_limit_ticks: 1200,
            game_event_strength: 1.0,
        })
    );
    assert_eq!(
        elder_guardian_effect_pulse(1199, 1, true).map(|pulse| pulse.game_event_strength),
        Some(0.0)
    );
    assert_eq!(elder_guardian_effect_pulse(1198, 1, false), None);
    assert_eq!(elder_guardian_sets_home_when_missing(false), Some(16));
    assert_eq!(elder_guardian_sets_home_when_missing(true), None);
    assert_eq!(ELDER_GUARDIAN_RANDOM_STROLL_INTERVAL, 400);
}

#[test]
fn ravager_attack_stun_roar_and_leaf_griefing_match_java() {
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
