use crate::mob_interaction::*;

#[test]
fn zombified_piglin_anger_alert_spawn_and_portal_gates_match_java_rules() {
    assert_zombified_piglin_attributes_and_anger_setup();
    assert_zombified_piglin_ai_alert_and_spawn_rules();
    assert_zombified_piglin_portal_and_pickup_rules();
}

fn assert_zombified_piglin_attributes_and_anger_setup() {
    assert_eq!(
        zombified_piglin_attributes(),
        ZombifiedPiglinAttributes {
            follow_range: 35.0,
            movement_speed: 0.23,
            attack_damage: 5.0,
            armor: 2.0,
            spawn_reinforcements_chance: 0.0,
            attacking_speed_modifier: 0.05,
            lava_pathfinding_malus: 8.0,
        }
    );
    assert_eq!(
        zombified_piglin_baby_dimensions(),
        ZombifiedPiglinBabyDimensions {
            width: 0.49,
            height: 0.99,
            eye_height: 0.78,
            vehicle_attachment_y: 0.1875,
        }
    );
    assert_eq!(zombified_piglin_start_persistent_anger_time(399), 400);
    assert_eq!(zombified_piglin_start_persistent_anger_time(781), 780);
    assert_eq!(
        zombified_piglin_set_target_delays(false, true, 7, 99),
        Some((7, 99))
    );
    assert_eq!(zombified_piglin_set_target_delays(true, true, 7, 99), None);
}

fn assert_zombified_piglin_ai_alert_and_spawn_rules() {
    let piglin_step = ZombifiedPiglinAiStepInput {
        angry: true,
        baby: false,
        has_attacking_speed_modifier: false,
        play_first_anger_sound_in: 1,
        target_present: true,
        ticks_until_next_alert: 0,
        has_line_of_sight_to_target: true,
        sampled_next_alert_interval: 80,
    };
    assert_eq!(
        zombified_piglin_ai_step(piglin_step),
        ZombifiedPiglinAiStep {
            has_attacking_speed_modifier: true,
            play_first_anger_sound_in: 0,
            played_first_anger_sound: true,
            ticks_until_next_alert: 80,
            alert_others: true,
        }
    );
    assert!(
        !zombified_piglin_ai_step(ZombifiedPiglinAiStepInput {
            angry: false,
            has_attacking_speed_modifier: true,
            play_first_anger_sound_in: 0,
            target_present: false,
            has_line_of_sight_to_target: false,
            ..piglin_step
        })
        .has_attacking_speed_modifier
    );
    assert!(
        !zombified_piglin_ai_step(ZombifiedPiglinAiStepInput {
            baby: true,
            play_first_anger_sound_in: 0,
            target_present: false,
            has_line_of_sight_to_target: false,
            ..piglin_step
        })
        .has_attacking_speed_modifier
    );
    assert!(zombified_piglin_alerts_other(false, false, false, true));
    assert!(!zombified_piglin_alerts_other(true, false, false, true));
    assert!(!zombified_piglin_alerts_other(false, true, false, true));
    assert_eq!(ZOMBIFIED_PIGLIN_ALERT_RANGE_Y, 10.0);

    assert!(zombified_piglin_spawn_allowed(false, false));
    assert!(!zombified_piglin_spawn_allowed(true, false));
    assert!(!zombified_piglin_spawn_allowed(false, true));
    assert!(zombified_piglin_spawn_obstruction(true, false));
    assert!(!zombified_piglin_spawn_obstruction(true, true));
    assert_eq!(
        zombified_piglin_default_main_hand_item(0),
        "minecraft:golden_spear"
    );
    assert_eq!(
        zombified_piglin_default_main_hand_item(1),
        "minecraft:golden_sword"
    );
}

fn assert_zombified_piglin_portal_and_pickup_rules() {
    assert_eq!(
        zombified_piglin_portal_spawn(true, true, 1, 2, true, true, true),
        ZombifiedPiglinPortalSpawn {
            spawn: true,
            spawn_pos_above_portal_floor: true,
            set_entity_portal_cooldown: true,
            set_vehicle_portal_cooldown: true,
        }
    );
    assert!(!zombified_piglin_portal_spawn(true, true, 2, 2, true, true, false).spawn);
    assert!(!zombified_piglin_portal_spawn(false, true, 0, 3, true, true, false).spawn);
    assert!(zombified_piglin_prevents_player_rest(true));
    assert!(!zombified_piglin_prevents_player_rest(false));
    assert!(zombified_piglin_wants_to_pick_up(true));
    assert!(!zombified_piglin_wants_to_pick_up(false));
}

#[test]
fn blaze_attack_hover_fire_and_loot_gates_match_java_rules() {
    assert_blaze_attributes_hover_and_fire_state();
    assert_blaze_attack_goal_start_and_fireball_burst();
    assert_blaze_attack_goal_finish_close_range_and_loot();
}

fn assert_blaze_attributes_hover_and_fire_state() {
    assert_eq!(
        blaze_attributes(),
        BlazeAttributes {
            attack_damage: 6.0,
            movement_speed: 0.23,
            follow_range: 48.0,
            water_pathfinding_malus: -1.0,
            lava_pathfinding_malus: 8.0,
            fire_neighbor_pathfinding_malus: 0.0,
            fire_pathfinding_malus: 0.0,
            xp_reward: 10,
        }
    );
    assert!(blaze_attack_goal_can_use(true, true, true));
    assert!(!blaze_attack_goal_can_use(true, false, true));
    assert!(blaze_is_sensitive_to_water());
    assert_eq!(blaze_light_level_dependent_magic_value(), 1.0);
    assert!(blaze_is_on_fire(true));
    assert!(!blaze_is_on_fire(false));

    assert_eq!(
        blaze_ai_step(false, -0.2, 10, 3.0, false, 0.0, false),
        BlazeAiStep {
            delta_y: -0.120000005,
            needs_sync: false,
            allowed_height_offset: 0.5,
            next_height_offset_change_tick: 9,
        }
    );
    assert_eq!(
        blaze_ai_step(true, 0.0, 1, 4.25, true, 5.0, true),
        BlazeAiStep {
            delta_y: 0.09,
            needs_sync: true,
            allowed_height_offset: 4.25,
            next_height_offset_change_tick: 100,
        }
    );
}

fn assert_blaze_attack_goal_start_and_fireball_burst() {
    let state = blaze_attack_goal_start();
    assert_eq!(state.attack_step, 0);
    let (state, action) =
        blaze_attack_goal_tick(state, true, true, 16.0, BLAZE_FOLLOW_RANGE, false);
    assert_eq!(state.attack_step, 1);
    assert!(state.charged);
    assert_eq!(action, BlazeAttackAction::Charge { charge_ticks: 60 });

    let mut state = BlazeAttackState {
        attack_step: 1,
        attack_time: 0,
        last_seen: 0,
        charged: true,
    };
    for expected_step in 2..=4 {
        let (next_state, action) =
            blaze_attack_goal_tick(state, true, true, 16.0, BLAZE_FOLLOW_RANGE, false);
        assert_eq!(next_state.attack_step, expected_step);
        assert!(next_state.charged);
        assert_eq!(
            action,
            BlazeAttackAction::ShootSmallFireball {
                cooldown_ticks: 6,
                level_event: 1018,
            }
        );
        state = BlazeAttackState {
            attack_time: 0,
            ..next_state
        };
    }
}

fn assert_blaze_attack_goal_finish_close_range_and_loot() {
    let state = BlazeAttackState {
        attack_step: 4,
        attack_time: 0,
        last_seen: 0,
        charged: true,
    };
    let (state, action) = blaze_attack_goal_tick(state, true, true, 16.0, BLAZE_FOLLOW_RANGE, true);
    assert_eq!(state.attack_step, 0);
    assert!(!state.charged);
    assert_eq!(
        action,
        BlazeAttackAction::Cooldown {
            cooldown_ticks: 100,
        }
    );

    let close_state = BlazeAttackState {
        attack_step: 0,
        attack_time: 0,
        last_seen: 0,
        charged: false,
    };
    assert_eq!(
        blaze_attack_goal_tick(close_state, true, true, 3.0, BLAZE_FOLLOW_RANGE, false).1,
        BlazeAttackAction::Melee { cooldown_ticks: 20 }
    );
    assert_eq!(
        blaze_attack_goal_tick(close_state, true, false, 3.0, BLAZE_FOLLOW_RANGE, false).1,
        BlazeAttackAction::None
    );
    assert_eq!(
        blaze_attack_goal_tick(close_state, true, false, 400.0, BLAZE_FOLLOW_RANGE, false).1,
        BlazeAttackAction::MoveTowardTarget
    );

    let stopped = blaze_attack_goal_stop(BlazeAttackState {
        attack_step: 2,
        attack_time: 6,
        last_seen: 3,
        charged: true,
    });
    assert!(!stopped.charged);
    assert_eq!(stopped.last_seen, 0);
    assert_eq!(blaze_fireball_spread(16.0), 1.0);
    assert_eq!(BLAZE_FIREBALL_INACCURACY, 2.297);
    assert_eq!(BLAZE_LOOT_ITEM, "minecraft:blaze_rod");
    assert_eq!(blaze_loot_roll(false, 1, 3), 0);
    assert_eq!(blaze_loot_roll(true, 1, 2), 3);
}

#[test]
fn drowned_spawn_equipment_swim_and_trident_gates_match_java_rules() {
    assert_drowned_attributes_dimensions_and_spawn_rules();
    assert_drowned_equipment_finalize_and_pickup_rules();
    assert_drowned_target_swim_trident_and_water_goals();
}

fn assert_drowned_attributes_dimensions_and_spawn_rules() {
    assert_eq!(
        drowned_attributes(),
        DrownedAttributes {
            follow_range: 35.0,
            movement_speed: 0.23,
            attack_damage: 3.0,
            armor: 2.0,
            step_height: 1.0,
        }
    );
    assert_eq!(
        drowned_entity_type_surface(),
        DrownedEntityTypeSurface {
            width: 0.6,
            height: 1.95,
            eye_height: 1.74,
            passenger_attachment_y: 2.0125,
            riding_offset: -0.7,
            client_tracking_range: 8,
            not_in_peaceful: true,
            amphibious_navigation: true,
            water_pathfinding_malus: 0.0,
            can_spawn_in_liquids: true,
        }
    );
    assert_eq!(
        drowned_baby_dimensions(),
        DrownedBabyDimensions {
            width: 0.49,
            height: 0.99,
            eye_height: 0.775,
            vehicle_attachment_y: 0.1875,
        }
    );

    let drowned_spawn = DrownedSpawnInput {
        below_is_water: true,
        pos_is_water: true,
        spawner_reason: false,
        reinforcement_reason: false,
        ignores_light_requirements: false,
        difficulty_peaceful: false,
        dark_enough_to_spawn: true,
        more_frequent_drowned_biome: false,
        random_roll: 0,
        y: 57,
        sea_level: 63,
    };
    assert!(!drowned_spawn_allowed(DrownedSpawnInput {
        below_is_water: false,
        ignores_light_requirements: true,
        y: 50,
        ..drowned_spawn
    }));
    assert!(drowned_spawn_allowed(DrownedSpawnInput {
        below_is_water: false,
        spawner_reason: true,
        ignores_light_requirements: true,
        random_roll: 39,
        y: 80,
        ..drowned_spawn
    }));
    assert!(drowned_spawn_allowed(DrownedSpawnInput {
        reinforcement_reason: true,
        random_roll: 39,
        y: 80,
        ..drowned_spawn
    }));
    assert!(drowned_spawn_allowed(DrownedSpawnInput {
        more_frequent_drowned_biome: true,
        y: 80,
        ..drowned_spawn
    }));
    assert!(!drowned_spawn_allowed(DrownedSpawnInput {
        more_frequent_drowned_biome: true,
        random_roll: 1,
        y: 80,
        ..drowned_spawn
    }));
    assert!(drowned_spawn_allowed(drowned_spawn));
    assert!(!drowned_spawn_allowed(DrownedSpawnInput {
        y: 58,
        ..drowned_spawn
    }));
    assert!(!drowned_spawn_allowed(DrownedSpawnInput {
        difficulty_peaceful: true,
        ..drowned_spawn
    }));
    assert_eq!(drowned_default_main_hand_item(0.9, 0), None);
    assert_eq!(
        drowned_default_main_hand_item(0.91, 9),
        Some("minecraft:trident")
    );
    assert_eq!(
        drowned_default_main_hand_item(0.91, 10),
        Some("minecraft:fishing_rod")
    );
}

fn assert_drowned_equipment_finalize_and_pickup_rules() {
    let drowned_finalize = DrownedFinalizeSpawnInput {
        offhand_empty: true,
        nautilus_random_float: 0.029,
        natural_or_structure_spawn: true,
        structure_spawn: true,
        main_hand_trident: true,
        zombie_nautilus_random_float: 0.49,
        baby: false,
        more_frequent_drowned_biome: false,
    };
    let finalize = drowned_finalize_spawn_outcome(drowned_finalize);
    assert_eq!(
        finalize,
        DrownedFinalizeSpawnOutcome {
            offhand_nautilus_shell: true,
            guaranteed_offhand_drop: true,
            spawned_zombie_nautilus_jockey: true,
            zombie_nautilus_persistent: true,
        }
    );
    assert!(
        !drowned_finalize_spawn_outcome(DrownedFinalizeSpawnInput {
            nautilus_random_float: 0.03,
            structure_spawn: false,
            zombie_nautilus_random_float: 0.0,
            ..drowned_finalize
        })
        .offhand_nautilus_shell
    );
    assert!(
        !drowned_finalize_spawn_outcome(DrownedFinalizeSpawnInput {
            nautilus_random_float: 0.0,
            structure_spawn: false,
            zombie_nautilus_random_float: 0.0,
            baby: true,
            ..drowned_finalize
        })
        .spawned_zombie_nautilus_jockey
    );
    assert!(
        !drowned_finalize_spawn_outcome(DrownedFinalizeSpawnInput {
            nautilus_random_float: 0.0,
            structure_spawn: false,
            zombie_nautilus_random_float: 0.0,
            more_frequent_drowned_biome: true,
            ..drowned_finalize
        })
        .spawned_zombie_nautilus_jockey
    );

    assert!(!drowned_can_replace_current_item(
        "minecraft:nautilus_shell"
    ));
    assert!(drowned_can_replace_current_item("minecraft:stick"));
    assert!(!drowned_wants_to_pick_up("minecraft:trident"));
    assert!(!drowned_wants_to_pick_up("minecraft:iron_spear"));
    assert!(drowned_wants_to_pick_up("minecraft:rotten_flesh"));
}

fn assert_drowned_target_swim_trident_and_water_goals() {
    assert!(drowned_ok_target(true, false, false));
    assert!(drowned_ok_target(true, true, true));
    assert!(!drowned_ok_target(true, true, false));
    assert!(!drowned_ok_target(false, false, true));
    assert!(drowned_wants_to_swim(true, false, false));
    assert!(drowned_wants_to_swim(false, true, true));
    assert!(!drowned_wants_to_swim(false, true, false));
    assert!(drowned_should_update_swimming(
        true, true, true, false, false
    ));
    assert!(!drowned_should_update_swimming(
        false, true, true, false, false
    ));

    assert!(drowned_trident_attack_can_use(true, true));
    assert!(!drowned_trident_attack_can_use(true, false));
    assert_eq!(
        drowned_trident_shot(2, true),
        DrownedTridentShot {
            item: "minecraft:trident",
            power: 1.6,
            inaccuracy: 6,
            y_lead_scale: 0.2,
            sound: "minecraft:entity.drowned.shoot",
        }
    );

    assert!(drowned_go_to_water_goal_can_use(true, false, true));
    assert!(!drowned_go_to_water_goal_can_use(false, false, true));
    assert!(drowned_go_to_beach_goal_can_use(true, false, true, 60, 63));
    assert!(!drowned_go_to_beach_goal_can_use(true, true, true, 60, 63));
    assert!(drowned_swim_up_goal_can_use(false, true, 60, 63));
    assert!(!drowned_swim_up_goal_can_use(true, true, 60, 63));
    assert_eq!(DROWNED_WATER_SEARCH_ATTEMPTS, 10);
    assert_eq!(DROWNED_WATER_SEARCH_XZ_RANGE, 10);
    assert_eq!(DROWNED_WATER_SEARCH_Y_UP, 2);
    assert_eq!(DROWNED_WATER_SEARCH_Y_DOWN, 5);
}

#[test]
fn husk_daylight_hunger_conversion_and_camel_spawn_match_java_rules() {
    assert_husk_dimensions_hunger_and_conversion();
    assert_husk_loot_and_camel_spawn_rules();
}

fn assert_husk_dimensions_hunger_and_conversion() {
    assert_eq!(
        husk_entity_type_surface(),
        HuskEntityTypeSurface {
            width: 0.6,
            height: 1.95,
            eye_height: 1.74,
            passenger_attachment_y: 2.075,
            riding_offset: -0.7,
            client_tracking_range: 8,
            not_in_peaceful: true,
        }
    );
    assert_eq!(
        husk_baby_dimensions(),
        HuskBabyDimensions {
            width: 0.49,
            height: 0.99,
            eye_height: 0.825,
            vehicle_attachment_y: 0.1875,
        }
    );
    assert!(!husk_is_sun_sensitive());
    assert_eq!(husk_hunger_duration_ticks(2.0, true, true, true), Some(280));
    assert_eq!(husk_hunger_duration_ticks(3.0, true, true, true), Some(420));
    assert_eq!(husk_hunger_duration_ticks(3.0, false, true, true), None);
    assert_eq!(husk_hunger_duration_ticks(3.0, true, false, true), None);
    assert_eq!(husk_hunger_duration_ticks(3.0, true, true, false), None);
    assert_eq!(HUSK_HUNGER_EFFECT_ID, "minecraft:hunger");
    assert_eq!(HUSK_HUNGER_AMPLIFIER, 0);

    assert_eq!(
        (HUSK_CONVERTS_IN_WATER, HUSK_UNDERWATER_CONVERSION_TARGET),
        (true, "minecraft:zombie")
    );
    assert_eq!(husk_underwater_conversion_event(false), Some(1041));
    assert_eq!(husk_underwater_conversion_event(true), None);
}

fn assert_husk_loot_and_camel_spawn_rules() {
    assert_eq!(husk_should_pick_up_loot(true, 0.0, 1.0), None);
    assert_eq!(husk_should_pick_up_loot(false, 0.54, 1.0), Some(true));
    assert_eq!(husk_should_pick_up_loot(false, 0.55, 1.0), Some(false));

    let camel_spawn = husk_finalize_spawn_outcome(true, false, 0.0, 1.0, true, 0.09);
    assert_eq!(
        camel_spawn,
        HuskFinalizeSpawnOutcome {
            can_pick_up_loot: Some(true),
            tried_to_spawn_camel_husk: true,
            spawned_camel_husk: true,
            spawned_parched_passenger: true,
            equipped_iron_spear: true,
        }
    );
    assert_eq!(HUSK_CAMEL_HUSK_RIDER_ITEM, "minecraft:iron_spear");

    let skipped_by_spawn_reason = husk_finalize_spawn_outcome(false, true, 0.0, 1.0, true, 0.0);
    assert_eq!(skipped_by_spawn_reason.can_pick_up_loot, None);

    let non_natural = husk_finalize_spawn_outcome(false, false, 0.0, 1.0, true, 0.0);
    assert!(non_natural.tried_to_spawn_camel_husk);
    assert!(!non_natural.spawned_camel_husk);

    let natural_blocked = husk_finalize_spawn_outcome(true, false, 0.0, 1.0, false, 0.0);
    assert!(!natural_blocked.tried_to_spawn_camel_husk);
    assert!(!natural_blocked.spawned_camel_husk);

    let natural_miss = husk_finalize_spawn_outcome(true, false, 0.0, 1.0, true, 0.1);
    assert!(natural_miss.tried_to_spawn_camel_husk);
    assert!(!natural_miss.spawned_camel_husk);
}

#[test]
fn endermite_lifetime_spawn_and_pearl_gates_match_java_rules() {
    assert_eq!(
        endermite_attributes(),
        EndermiteAttributes {
            max_health: 8.0,
            movement_speed: 0.25,
            attack_damage: 2.0,
            xp_reward: 3,
        }
    );
    assert_eq!(ENDERMITE_MAX_LIFE_TICKS, 2400);
    assert_eq!(ENDERMITE_CLIENT_PORTAL_PARTICLES_PER_TICK, 2);
    assert_eq!(ENDERMITE_LOOK_AT_PLAYER_RANGE, 8.0);
    assert_eq!(ENDERMITE_STEP_SOUND_VOLUME, 0.15);
    assert_eq!(ENDERMITE_STEP_SOUND_PITCH, 1.0);

    let mut mite = EndermiteState::read_save_data(Some(2399), false);
    assert_eq!(
        mite.ai_step(),
        EndermiteTickOutcome {
            life: 2400,
            discard: true,
        }
    );

    let mut persistent = EndermiteState::read_save_data(Some(2399), true);
    assert_eq!(
        persistent.ai_step(),
        EndermiteTickOutcome {
            life: 2399,
            discard: false,
        }
    );

    assert!(endermite_spawn_allowed(true, true, true));
    assert!(!endermite_spawn_allowed(true, false, true));
    assert!(endermite_spawn_allowed(true, false, false));
    assert!(!endermite_spawn_allowed(false, true, false));
    assert!(endermite_from_ender_pearl(0.049, true));
    assert!(!endermite_from_ender_pearl(0.05, true));
    assert!(!endermite_from_ender_pearl(0.0, false));
    assert!(enderman_targets_endermite());
}

include!("tests_f_enderman_skeleton.rs");
