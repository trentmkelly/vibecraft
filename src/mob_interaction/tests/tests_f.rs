use crate::mob_interaction::*;

#[test]
fn zombified_piglin_anger_alert_spawn_and_portal_gates_match_java_rules() {
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

#[test]
fn enderman_carry_stare_anger_and_teleport_gates_match_java() {
    assert_eq!(
        enderman_attributes(),
        EndermanAttributes {
            max_health: 40.0,
            movement_speed: 0.3,
            attacking_speed_bonus: 0.15,
            attack_damage: 7.0,
            follow_range: 64.0,
            step_height: 1.0,
        }
    );
    assert_eq!(
        enderman_entity_type_surface(),
        EndermanEntityTypeSurface {
            width: 0.6,
            height: 2.9,
            eye_height: 2.55,
            passenger_attachment_y: 2.80625,
            client_tracking_range: 8,
            not_in_peaceful: true,
        }
    );
    assert_eq!(ENDERMAN_WATER_PATHFINDING_MALUS, -1.0);
    assert_eq!(ENDERMAN_STARE_SOUND_COOLDOWN, 400);
    assert_eq!(ENDERMAN_MIN_DEAGGRESSION_TIME, 600);
    assert_eq!(ENDERMAN_PERSISTENT_ANGER_MIN_SECONDS, 20);
    assert_eq!(ENDERMAN_PERSISTENT_ANGER_MAX_SECONDS, 39);
    assert_eq!(ENDERMAN_LOOK_AT_PLAYER_RANGE, 8.0);
    assert_eq!(ENDERMAN_STARE_DOT_THRESHOLD, 0.025);
    assert_eq!(ENDERMAN_RANDOM_TELEPORT_HORIZONTAL_RANGE, 64.0);
    assert_eq!(ENDERMAN_RANDOM_TELEPORT_VERTICAL_RANGE, 64);
    assert_eq!(ENDERMAN_TELEPORT_TOWARDS_DISTANCE, 16.0);
    assert_eq!(ENDERMAN_TELEPORT_TOWARDS_RANDOM_HORIZONTAL, 8.0);
    assert_eq!(ENDERMAN_TELEPORT_TOWARDS_RANDOM_VERTICAL, 16);
    assert_eq!(ENDERMAN_PROJECTILE_TELEPORT_ATTEMPTS, 64);

    assert_eq!(
        enderman_set_target_state(true, 123),
        EndermanTargetState {
            target_change_time: 123,
            creepy: true,
            stared_at: false,
            speed_modifier_present: true,
        }
    );
    assert_eq!(
        enderman_set_target_state(false, 123),
        EndermanTargetState {
            target_change_time: 0,
            creepy: false,
            stared_at: false,
            speed_modifier_present: false,
        }
    );
    assert!(enderman_stare_sound_allowed(400, 0));
    assert!(!enderman_stare_sound_allowed(399, 0));
    assert!(enderman_should_daylight_deaggro_and_teleport(
        true, 700, 100, 0.6, true, 0.0
    ));
    assert!(!enderman_should_daylight_deaggro_and_teleport(
        true, 699, 100, 0.6, true, 0.0
    ));
    assert!(!enderman_should_daylight_deaggro_and_teleport(
        true, 700, 100, 0.5, true, 0.0
    ));

    assert_eq!(
        enderman_hurt_response(false, false, false, true, 1),
        EndermanHurtResponse::NormalHurt
    );
    assert_eq!(
        enderman_hurt_response(false, false, false, false, 1),
        EndermanHurtResponse::NormalHurtAndMaybeTeleport
    );
    assert_eq!(
        enderman_hurt_response(false, false, false, false, 0),
        EndermanHurtResponse::NormalHurt
    );
    assert_eq!(
        enderman_hurt_response(true, false, false, false, 0),
        EndermanHurtResponse::ProjectileTryTeleport64
    );
    assert_eq!(
        enderman_hurt_response(false, true, true, false, 0),
        EndermanHurtResponse::WaterPotionHurtAndTryTeleport64
    );

    assert!(enderman_freeze_when_looked_at(true, 256.0, true));
    assert!(!enderman_freeze_when_looked_at(true, 256.1, true));
    assert!(!enderman_freeze_when_looked_at(false, 1.0, true));
    assert_eq!(enderman_look_goal_starts_aggro(true), Some(5));
    assert_eq!(enderman_look_goal_starts_aggro(false), None);
    assert_eq!(
        enderman_look_goal_tick(Some(1), false, false, 0.0, 0, false),
        (Some(0), true, 0, false)
    );
    assert_eq!(
        enderman_look_goal_tick(None, true, true, 15.9, 9, false),
        (None, false, 0, true)
    );
    assert_eq!(
        enderman_look_goal_tick(None, true, false, 257.0, 30, false),
        (None, false, 31, true)
    );
    assert_eq!(
        enderman_look_goal_tick(None, true, false, 257.0, 29, false),
        (None, false, 30, false)
    );

    assert!(enderman_take_block_can_use(false, true, 0));
    assert!(!enderman_take_block_can_use(true, true, 0));
    assert!(!enderman_take_block_can_use(false, false, 0));
    assert!(!enderman_take_block_can_use(false, true, 1));
    assert!(enderman_leave_block_can_use(true, true, 0));
    assert!(!enderman_leave_block_can_use(false, true, 0));
    assert!(!enderman_leave_block_can_use(true, false, 0));
    assert!(enderman_can_place_carried_block(
        true, false, false, true, true, true
    ));
    assert!(!enderman_can_place_carried_block(
        true, true, false, true, true, true
    ));
    assert!(!enderman_can_place_carried_block(
        true, false, true, true, true, true
    ));
    assert!(!enderman_can_place_carried_block(
        true, false, false, false, true, true
    ));
    assert!(enderman_requires_custom_persistence(false, true));
    assert!(!enderman_requires_custom_persistence(false, false));
}

#[test]
fn skeleton_family_ranged_conversion_and_variant_gates_match_java() {
    assert_eq!(ABSTRACT_SKELETON_MOVEMENT_SPEED, 0.25);
    assert_eq!(ABSTRACT_SKELETON_BOW_SPEED, 1.0);
    assert_eq!(ABSTRACT_SKELETON_BOW_RANGE, 15.0);
    assert_eq!(ABSTRACT_SKELETON_MELEE_SPEED, 1.2);
    assert_eq!(ABSTRACT_SKELETON_WOLF_AVOID_DISTANCE, 6.0);
    assert_eq!(ABSTRACT_SKELETON_LOOK_AT_PLAYER_RANGE, 8.0);
    assert_eq!(ABSTRACT_SKELETON_TURTLE_TARGET_INTERVAL, 10);
    assert_eq!(ABSTRACT_SKELETON_DEFAULT_MAINHAND, "minecraft:bow");
    assert_eq!(ABSTRACT_SKELETON_STEP_SOUND_VOLUME, 0.15);
    assert_eq!(ABSTRACT_SKELETON_STEP_SOUND_PITCH, 1.0);

    assert_eq!(
        skeleton_entity_type_surface("minecraft:skeleton"),
        SkeletonEntityTypeSurface {
            width: 0.6,
            height: 1.99,
            eye_height: 1.74,
            riding_offset: -0.7,
            client_tracking_range: 8,
            not_in_peaceful: true,
            fire_immune: false,
            immune_to_powder_snow: false,
            immune_to_wither_rose: false,
        }
    );
    assert_eq!(
        skeleton_entity_type_surface("minecraft:stray"),
        SkeletonEntityTypeSurface {
            width: 0.6,
            height: 1.99,
            eye_height: 1.74,
            riding_offset: -0.7,
            client_tracking_range: 8,
            not_in_peaceful: true,
            fire_immune: false,
            immune_to_powder_snow: true,
            immune_to_wither_rose: false,
        }
    );
    assert_eq!(
        skeleton_entity_type_surface("minecraft:wither_skeleton"),
        SkeletonEntityTypeSurface {
            width: 0.7,
            height: 2.4,
            eye_height: 2.1,
            riding_offset: -0.875,
            client_tracking_range: 8,
            not_in_peaceful: true,
            fire_immune: true,
            immune_to_powder_snow: false,
            immune_to_wither_rose: true,
        }
    );
    assert_eq!(
        skeleton_entity_type_surface("minecraft:bogged").height,
        1.99
    );
    assert_eq!(
        skeleton_entity_type_surface("minecraft:parched").height,
        1.99
    );
    assert!(!skeleton_entity_type_surface("minecraft:parched").fire_immune);
    assert_eq!(
        abstract_skeleton_weapon_goal(true, true, false),
        SkeletonWeaponGoal::Bow {
            min_attack_interval: 20,
        }
    );
    assert_eq!(
        abstract_skeleton_weapon_goal(true, false, false),
        SkeletonWeaponGoal::Bow {
            min_attack_interval: 40,
        }
    );
    assert_eq!(
        abstract_skeleton_weapon_goal(true, true, true),
        SkeletonWeaponGoal::Bow {
            min_attack_interval: 50,
        }
    );
    assert_eq!(
        abstract_skeleton_weapon_goal(true, false, true),
        SkeletonWeaponGoal::Bow {
            min_attack_interval: 70,
        }
    );
    assert_eq!(
        abstract_skeleton_weapon_goal(false, true, false),
        SkeletonWeaponGoal::Melee
    );
    assert_eq!(
        abstract_skeleton_ranged_shot(0),
        AbstractSkeletonRangedShot {
            speed: 1.6,
            inaccuracy: 14,
            vertical_lead_multiplier: 0.2,
        }
    );
    assert_eq!(abstract_skeleton_ranged_shot(3).inaccuracy, 2);
    assert!(abstract_skeleton_can_pick_up_loot(0.54, 1.0));
    assert!(!abstract_skeleton_can_pick_up_loot(0.55, 1.0));
    assert_eq!(
        abstract_skeleton_halloween_head(0.249, 0.099, true, true),
        Some("minecraft:jack_o_lantern")
    );
    assert_eq!(
        abstract_skeleton_halloween_head(0.249, 0.1, true, true),
        Some("minecraft:carved_pumpkin")
    );
    assert_eq!(
        abstract_skeleton_halloween_head(0.1, 0.0, false, true),
        None
    );

    assert_eq!(
        skeleton_freeze_tick(true, false, 139, 0, true, false),
        SkeletonFreezeTick {
            in_powder_snow_time: 140,
            conversion_time: 300,
            freeze_converting: true,
            convert_to_stray: false,
        }
    );
    assert_eq!(
        skeleton_freeze_tick(true, true, 140, 0, true, false),
        SkeletonFreezeTick {
            in_powder_snow_time: 140,
            conversion_time: -1,
            freeze_converting: true,
            convert_to_stray: true,
        }
    );
    assert_eq!(
        skeleton_freeze_tick(false, true, 140, 20, true, false),
        SkeletonFreezeTick {
            in_powder_snow_time: -1,
            conversion_time: 20,
            freeze_converting: false,
            convert_to_stray: false,
        }
    );
    assert_eq!(
        skeleton_freeze_tick(true, true, 140, 20, true, true),
        SkeletonFreezeTick {
            in_powder_snow_time: 140,
            conversion_time: 20,
            freeze_converting: true,
            convert_to_stray: false,
        }
    );
    assert_eq!(
        skeleton_freeze_tick(true, true, 140, 20, false, false),
        SkeletonFreezeTick {
            in_powder_snow_time: 140,
            conversion_time: 20,
            freeze_converting: true,
            convert_to_stray: false,
        }
    );
    assert_eq!(skeleton_save_stray_conversion_time(true, 42), 42);
    assert_eq!(skeleton_save_stray_conversion_time(false, 42), -1);
    assert_eq!(SKELETON_STRAY_CONVERSION_EVENT, 1048);

    assert!(stray_spawn_allowed(true, true, false));
    assert!(stray_spawn_allowed(true, false, true));
    assert!(!stray_spawn_allowed(false, true, true));
    assert!(!stray_spawn_allowed(true, false, false));
    assert_eq!(
        skeleton_arrow_effect("minecraft:stray"),
        Some(("minecraft:slowness", 600))
    );
    assert_eq!(
        skeleton_arrow_effect("minecraft:bogged"),
        Some(("minecraft:poison", 100))
    );
    assert_eq!(
        skeleton_arrow_effect("minecraft:parched"),
        Some(("minecraft:weakness", 600))
    );
    assert_eq!(skeleton_arrow_effect("minecraft:skeleton"), None);
    assert_eq!(
        wither_skeleton_melee_effect(true),
        Some(("minecraft:wither", 200))
    );
    assert_eq!(wither_skeleton_melee_effect(false), None);
    assert_eq!(WITHER_SKELETON_ATTACK_DAMAGE, 4.0);
    assert_eq!(WITHER_SKELETON_ARROW_FIRE_SECONDS, 100.0);
    assert_eq!(WITHER_SKELETON_LAVA_PATHFINDING_MALUS, 8.0);
    assert!(!wither_skeleton_can_be_affected("minecraft:wither"));
    assert!(wither_skeleton_can_be_affected("minecraft:slowness"));
    assert_eq!((BOGGED_MAX_HEALTH, BOGGED_DEFAULT_SHEARED), (16.0, false));
    assert!(bogged_ready_for_shearing(false, true));
    assert!(!bogged_ready_for_shearing(true, true));
    assert!(!bogged_ready_for_shearing(false, false));
    assert!(bogged_shear_sets_sheared(true));
    assert_eq!(PARCHED_MAX_HEALTH, 16.0);
    assert!(!parched_can_be_affected("minecraft:weakness"));
    assert!(parched_can_be_affected("minecraft:poison"));
}

#[test]
fn cave_spider_dimensions_poison_and_inherited_spider_gates_match_java_rules() {
    assert_eq!(
        cave_spider_attributes(),
        CaveSpiderAttributes {
            max_health: 12.0,
            movement_speed: 0.3,
        }
    );
    assert_eq!(
        cave_spider_entity_type_surface(),
        CaveSpiderEntityTypeSurface {
            width: 0.7,
            height: 0.5,
            eye_height: 0.45,
            client_tracking_range: 8,
            not_in_peaceful: true,
        }
    );

    assert_eq!(cave_spider_poison_duration_ticks("peaceful", true), None);
    assert_eq!(cave_spider_poison_duration_ticks("easy", true), None);
    assert_eq!(cave_spider_poison_duration_ticks("normal", true), Some(140));
    assert_eq!(cave_spider_poison_duration_ticks("hard", true), Some(300));
    assert_eq!(cave_spider_poison_duration_ticks("hard", false), None);
    assert_eq!(CAVE_SPIDER_POISON_AMPLIFIER, 0);

    assert_eq!(
        cave_spider_finalize_spawn_preserves_group_data(Some(7)),
        Some(7)
    );
    assert_eq!(
        cave_spider_vehicle_attachment_y(0.7, 0.7, 1.0),
        Some(0.21875)
    );
    assert_eq!(cave_spider_vehicle_attachment_y(0.8, 0.7, 1.0), None);

    let flags = spider_set_climbing_flags(0, true);
    assert!(spider_is_climbing(flags));
    assert!(!spider_is_climbing(spider_set_climbing_flags(flags, false)));
    assert!(spider_is_climbing(spider_tick_climbing_flags(0, true)));
    assert!(!spider_is_climbing(spider_tick_climbing_flags(
        flags, false
    )));
    assert!(spider_attack_goal_can_use(true, false));
    assert!(!spider_attack_goal_can_use(true, true));
    assert!(spider_target_goal_can_use(0.49, true));
    assert!(!spider_target_goal_can_use(0.5, true));
    assert!(spider_avoids_armadillo(false));
    assert!(!spider_avoids_armadillo(true));
    assert_eq!(
        (SPIDER_POISON_IMMUNE, spider_can_be_affected("minecraft:poison")),
        (true, false)
    );
    assert!(!spider_can_be_affected("minecraft:poison"));
    assert!(spider_can_be_affected("minecraft:speed"));
    assert!(spider_jockey_from_finalize_spawn(0));
    assert!(!spider_jockey_from_finalize_spawn(1));
    assert!(spider_should_drop_target_in_light(0.5, 0));
    assert!(!spider_should_drop_target_in_light(0.49, 0));
    assert!(!spider_should_drop_target_in_light(0.5, 1));
    assert_eq!(SPIDER_SPECIAL_EFFECT_CHANCE, 0.1);
    assert_eq!(SPIDER_VEHICLE_ATTACHMENT_Y, 0.3125);
    assert_eq!(SPIDER_AVOID_ARMADILLO_DISTANCE, 6.0);
    assert_eq!(SPIDER_AVOID_ARMADILLO_WALK_SPEED, 1.0);
    assert_eq!(SPIDER_AVOID_ARMADILLO_SPRINT_SPEED, 1.2);
    assert_eq!(SPIDER_LEAP_AT_TARGET_POWER, 0.4);
    assert_eq!(SPIDER_RANDOM_STROLL_SPEED, 0.8);
    assert_eq!(SPIDER_LOOK_AT_PLAYER_RANGE, 8.0);
    assert_eq!(
        spider_effect_from_group_data_selection(0),
        "minecraft:speed"
    );
    assert_eq!(
        spider_effect_from_group_data_selection(1),
        "minecraft:speed"
    );
    assert_eq!(
        spider_effect_from_group_data_selection(2),
        "minecraft:strength"
    );
    assert_eq!(
        spider_effect_from_group_data_selection(3),
        "minecraft:regeneration"
    );
    assert_eq!(
        spider_effect_from_group_data_selection(4),
        "minecraft:invisibility"
    );
    assert!(spider_should_roll_special_effect("hard", 0.09, 1.0));
    assert!(!spider_should_roll_special_effect("normal", 0.0, 1.0));
    assert!(!spider_should_roll_special_effect("hard", 0.11, 1.0));
}

#[test]
fn pufferfish_puff_timing_and_contact_effects_match_java_thresholds() {
    let mut fish = PufferfishState::new();
    fish.start_inflating();
    fish.tick(true, true);
    assert_eq!(fish.puff_state, PufferfishState::MID);
    assert_eq!(fish.inflate_counter, 2);
    for _ in 0..39 {
        fish.tick(true, true);
    }
    assert_eq!(fish.puff_state, PufferfishState::MID);
    assert_eq!(fish.inflate_counter, 41);
    fish.tick(true, true);
    assert_eq!(fish.puff_state, PufferfishState::FULL);
    assert_eq!(
        fish.contact_effect(true, true),
        Some(PufferfishContactEffect {
            damage: 3,
            poison_effect: "minecraft:poison",
            poison_duration_ticks: 120,
            poison_amplifier: 0,
        })
    );
    assert_eq!(fish.contact_effect(true, false), None);

    fish.stop_inflating();
    for _ in 0..61 {
        fish.tick(true, true);
    }
    assert_eq!(fish.puff_state, PufferfishState::FULL);
    fish.tick(true, true);
    assert_eq!(fish.puff_state, PufferfishState::MID);
    assert_eq!(
        fish.contact_effect(true, true),
        Some(PufferfishContactEffect {
            damage: 2,
            poison_effect: "minecraft:poison",
            poison_duration_ticks: 60,
            poison_amplifier: 0,
        })
    );

    while fish.deflate_timer <= 100 {
        fish.tick(true, true);
    }
    fish.tick(true, true);
    assert_eq!(fish.puff_state, PufferfishState::SMALL);
    assert_eq!(fish.contact_effect(true, true), None);
}

#[test]
fn salmon_size_variants_match_java_ids_scales_defaults_and_weights() {
    assert_eq!(DEFAULT_SALMON_VARIANT_ID, 1);
    assert_eq!(
        SALMON_VARIANTS,
        &[
            SalmonVariantModel {
                name: "small",
                id: 0,
                bounding_box_scale: 0.5,
                spawn_weight: 30,
            },
            SalmonVariantModel {
                name: "medium",
                id: 1,
                bounding_box_scale: 1.0,
                spawn_weight: 50,
            },
            SalmonVariantModel {
                name: "large",
                id: 2,
                bounding_box_scale: 1.5,
                spawn_weight: 15,
            },
        ]
    );
    assert_eq!(salmon_variant_by_id(-1).name, "small");
    assert_eq!(salmon_variant_by_id(99).name, "large");
    assert_eq!(salmon_variant_by_name("medium").unwrap().id, 1);
    assert_eq!(salmon_spawn_weight_total(), 95);
}

#[test]
fn tropical_fish_packed_variants_match_java_pattern_and_color_layout() {
    assert_eq!(DEFAULT_TROPICAL_FISH_VARIANT_PACKED_ID, 0);
    assert_eq!(TROPICAL_FISH_PATTERNS.len(), 12);
    let kob = TROPICAL_FISH_PATTERNS[0];
    let flopper = TROPICAL_FISH_PATTERNS[6];
    let stripey = TROPICAL_FISH_PATTERNS[7];
    let clayfish = TROPICAL_FISH_PATTERNS[11];
    assert_eq!(tropical_fish_pattern_packed_id(kob), 0);
    assert_eq!(tropical_fish_pattern_packed_id(flopper), 1);
    assert_eq!(tropical_fish_pattern_packed_id(stripey), 257);
    assert_eq!(tropical_fish_pattern_packed_id(clayfish), 1281);

    let packed = tropical_fish_pack_variant(stripey, 1, 7);
    assert_eq!(packed, 117_506_305);
    assert_eq!(tropical_fish_pattern_from_variant(packed), stripey);
    assert_eq!(tropical_fish_base_color_id(packed), 1);
    assert_eq!(tropical_fish_pattern_color_id(packed), 7);
    assert_eq!(tropical_fish_pattern_by_packed_id(99_999), kob);
    let common = tropical_fish_common_variants();
    assert_eq!(common.len(), 22);
    assert_eq!(common[0].pattern.name, "stripey");
    assert_eq!(
        tropical_fish_pack_variant(
            common[0].pattern,
            common[0].base_color_id,
            common[0].pattern_color_id
        ),
        packed
    );
    assert_eq!(common[21].pattern.name, "flopper");
    assert_eq!(common[21].base_color_id, 4);
    assert_eq!(common[21].pattern_color_id, 4);
}

#[test]
fn abstract_fish_bucket_and_schooling_gates_match_java_rules() {
    assert_eq!(
        abstract_fish_attributes(),
        AbstractFishAttributes {
            max_health: 3.0,
            panic_speed_modifier: 1.25,
            avoid_player_distance: 8.0,
            avoid_player_near_speed: 1.6,
            avoid_player_far_speed: 1.4,
            random_swim_speed_modifier: 1.0,
            random_swim_interval_ticks: 40,
            water_drag: 0.9,
            no_target_gravity: -0.005,
        }
    );
    assert_eq!(ABSTRACT_FISH_MAX_SPAWN_CLUSTER_SIZE, 8);
    assert_eq!(SALMON_MAX_SCHOOL_SIZE, 5);
    assert_eq!(ABSTRACT_FISH_EYE_WATER_BOOST_Y, 0.005);
    assert_eq!(ABSTRACT_FISH_FLOP_JUMP_Y, 0.4);
    assert_eq!(ABSTRACT_FISH_FLOP_RANDOM_XZ_SCALE, 0.05);
    assert_eq!(SCHOOLING_FISH_NEIGHBOR_SCAN_RANGE, 8.0);
    assert_eq!(SCHOOLING_FISH_LEADER_RANGE_SQR, 121.0);

    assert!(fish_requires_custom_persistence(false, true));
    assert!(fish_requires_custom_persistence(true, false));
    assert!(!fish_requires_custom_persistence(false, false));
    assert!(fish_remove_when_far_away(false, false));
    assert!(!fish_remove_when_far_away(true, false));
    assert!(!fish_remove_when_far_away(false, true));
    assert!(fish_can_random_swim(false));
    assert!(!fish_can_random_swim(true));

    assert_eq!(
        fish_flop_step(false, true, true),
        FishFlopStep {
            jump: true,
            sync_needed: true,
            play_flop_sound: true,
        }
    );
    assert!(!fish_flop_step(true, true, true).jump);
    assert_eq!(fish_travel_y_delta(false, 0.0), -0.005);
    assert_eq!(fish_travel_y_delta(true, 0.1), 0.1);

    assert!(schooling_fish_is_follower(true, true));
    assert!(!schooling_fish_is_follower(true, false));
    assert!(schooling_fish_can_be_followed(SchoolingFishState {
        school_size: 2,
        max_school_size: 8,
        leader_alive: true,
    }));
    assert!(!schooling_fish_can_be_followed(SchoolingFishState {
        school_size: 1,
        max_school_size: 8,
        leader_alive: true,
    }));
    assert_eq!(schooling_fish_followers_added(3, 8, 10), 5);
    assert!(schooling_fish_should_reset_size(1, 1));
    assert!(!schooling_fish_should_reset_size(2, 1));
    assert!(!schooling_fish_should_reset_size(1, 2));

    assert_eq!(
        fish_bucket_model("minecraft:cod", true),
        Some(FishBucketModel {
            bucket_item: "minecraft:cod_bucket",
            pickup_sound: "minecraft:item.bucket.fill_fish",
            from_bucket_save_field: "FromBucket",
            requires_persistence_from_bucket: true,
            discard_on_pickup: true,
        })
    );
    assert_eq!(
        fish_bucket_model("minecraft:salmon", false)
            .expect("salmon bucket model")
            .bucket_item,
        "minecraft:salmon_bucket"
    );
    assert_eq!(
        fish_bucket_model("minecraft:tropical_fish", false)
            .expect("tropical fish bucket model")
            .bucket_item,
        "minecraft:tropical_fish_bucket"
    );
    assert_eq!(
        fish_bucket_model("minecraft:pufferfish", false)
            .expect("pufferfish bucket model")
            .bucket_item,
        "minecraft:pufferfish_bucket"
    );
    assert_eq!(fish_bucket_model("minecraft:squid", false), None);
}

#[test]
fn mob_interaction_checklist_surface_is_covered() {
    let covered: Vec<&str> = MOB_INTERACTION_COVERAGE
        .iter()
        .flat_map(|entry| entry.covered_rules.iter().copied())
        .collect();
    for expected in [
        "tameable",
        "breedable",
        "rideable",
        "shearable",
        "bucketable",
        "variant",
        "ageable",
        "trading",
        "anger",
        "conversion",
        "transformation",
    ] {
        assert!(covered.contains(&expected), "missing {expected}");
    }
}
