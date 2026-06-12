use crate::mob_interaction::*;

#[test]
fn breeze_util_and_shoot_when_stuck_match_java_rules() {
    assert_breeze_util_random_point_and_los_rules();
    assert_breeze_shoot_when_stuck_memory_rules();
    assert_breeze_slide_rules();
}

fn assert_breeze_util_random_point_and_los_rules() {
    assert_eq!(BREEZE_UTIL_MAX_LINE_OF_SIGHT_TEST_RANGE, 50.0);
    assert_eq!(BREEZE_UTIL_BEHIND_TARGET_BASE_DEGREES, 180.0);
    assert_eq!(BREEZE_UTIL_BEHIND_TARGET_SPREAD_DEGREES, 90.0);
    assert_eq!(BREEZE_UTIL_BEHIND_TARGET_MIN_DISTANCE, 4.0);
    assert_eq!(BREEZE_UTIL_BEHIND_TARGET_MAX_DISTANCE, 8.0);

    let target = BreezeVec3::new(10.0, 64.0, -2.0);
    assert_vec3_close(
        breeze_random_point_behind_target(target, 0.0, 0.0, 0.0),
        BreezeVec3::new(10.0, 64.0, 2.0),
    );
    assert_vec3_close(
        breeze_random_point_behind_target(target, 90.0, 0.0, 1.0),
        BreezeVec3::new(2.0, 64.0, -2.0),
    );
    assert!(breeze_has_line_of_sight(
        BreezeVec3::new(0.0, 64.0, 0.0),
        BreezeVec3::new(50.0, 64.0, 0.0),
        16.0,
        true,
    ));
    assert!(!breeze_has_line_of_sight(
        BreezeVec3::new(0.0, 64.0, 0.0),
        BreezeVec3::new(50.1, 64.0, 0.0),
        16.0,
        true,
    ));
    assert!(breeze_has_line_of_sight(
        BreezeVec3::new(0.0, 64.0, 0.0),
        BreezeVec3::new(60.0, 64.0, 0.0),
        64.0,
        true,
    ));
    assert!(!breeze_has_line_of_sight(
        BreezeVec3::new(0.0, 64.0, 0.0),
        BreezeVec3::new(40.0, 64.0, 0.0),
        64.0,
        false,
    ));
}

fn assert_breeze_shoot_when_stuck_memory_rules() {
    assert_eq!(
        BREEZE_SHOOT_WHEN_STUCK_MEMORY_REQUIREMENTS,
        [
            ("attack_target", "value_present"),
            ("breeze_jump_inhaling", "value_absent"),
            ("breeze_jump_target", "value_absent"),
            ("walk_target", "value_absent"),
            ("breeze_shoot", "value_absent"),
        ]
    );
    assert_eq!(BREEZE_SHOOT_WHEN_STUCK_MEMORY_EXPIRY_TICKS, 60);
    assert_eq!(
        breeze_shoot_when_stuck_step(true, false, false),
        BreezeShootWhenStuckStep {
            can_start: true,
            can_still_use: false,
            shoot_memory_expiry_ticks: Some(60),
        }
    );
    assert_eq!(
        breeze_shoot_when_stuck_step(false, true, false),
        BreezeShootWhenStuckStep {
            can_start: true,
            can_still_use: false,
            shoot_memory_expiry_ticks: Some(60),
        }
    );
    assert_eq!(
        breeze_shoot_when_stuck_step(false, false, true),
        BreezeShootWhenStuckStep {
            can_start: true,
            can_still_use: false,
            shoot_memory_expiry_ticks: Some(60),
        }
    );
    assert_eq!(
        breeze_shoot_when_stuck_step(false, false, false),
        BreezeShootWhenStuckStep {
            can_start: false,
            can_still_use: false,
            shoot_memory_expiry_ticks: None,
        }
    );
}

fn assert_breeze_slide_rules() {
    assert_eq!(
        BREEZE_SLIDE_MEMORY_REQUIREMENTS,
        [
            ("attack_target", "value_present"),
            ("walk_target", "value_absent"),
            ("breeze_jump_cooldown", "value_absent"),
            ("breeze_shoot", "value_absent"),
        ]
    );
    assert_eq!(BREEZE_SLIDE_AWAY_HORIZONTAL_RANGE, 5);
    assert_eq!(BREEZE_SLIDE_AWAY_VERTICAL_RANGE, 5);
    assert_eq!(BREEZE_SLIDE_MIDDLE_MIN_DISTANCE, 4.0);
    assert_eq!(BREEZE_SLIDE_MIDDLE_MAX_DISTANCE, 8.0);
    assert_eq!(BREEZE_SLIDE_WALK_TARGET_SPEED, 0.6);
    assert_eq!(BREEZE_SLIDE_WALK_TARGET_CLOSE_ENOUGH_DIST, 1);
    assert!(breeze_slide_can_start(true, false, "standing"));
    assert!(!breeze_slide_can_start(false, false, "standing"));
    assert!(!breeze_slide_can_start(true, true, "standing"));
    assert!(!breeze_slide_can_start(true, false, "shooting"));

    let input = BreezeSlideStartInput {
        breeze_position: BreezeVec3::new(0.0, 64.0, 0.0),
        enemy_position: BreezeVec3::new(10.0, 64.0, 0.0),
        within_inner_ring: true,
        away_candidate: Some(BreezeVec3::new(-6.25, 65.0, 0.5)),
        away_candidate_has_line_of_sight: true,
        random_next_boolean: true,
        behind_target_head_y_rot_degrees: 0.0,
        behind_target_gaussian: 0.0,
        behind_target_random_float: 0.0,
        middle_circle_random_double: 0.0,
    };
    assert_eq!(
        breeze_slide_start(input),
        BreezeSlideWalkTarget {
            position: BreezeVec3::new(-6.25, 65.0, 0.5),
            block_pos: (-7, 65, 0),
            speed_modifier: 0.6,
            close_enough_dist: 1,
        }
    );
    assert_vec3_close(
        breeze_slide_start(BreezeSlideStartInput {
            away_candidate_has_line_of_sight: false,
            ..input
        })
        .position,
        BreezeVec3::new(10.0, 64.0, 4.0),
    );
    assert_vec3_close(
        breeze_slide_random_point_in_middle_circle(
            BreezeVec3::new(0.0, 64.0, 0.0),
            BreezeVec3::new(10.0, 64.0, 0.0),
            0.0,
        ),
        BreezeVec3::new(2.0, 64.0, 0.0),
    );
    assert_vec3_close(
        breeze_slide_start(BreezeSlideStartInput {
            away_candidate: None,
            random_next_boolean: false,
            middle_circle_random_double: 1.0,
            ..input
        })
        .position,
        BreezeVec3::new(6.0, 64.0, 0.0),
    );
}

fn assert_vec3_close(actual: BreezeVec3, expected: BreezeVec3) {
    assert!((actual.x - expected.x).abs() < 1.0e-6, "{actual:?}");
    assert!((actual.y - expected.y).abs() < 1.0e-6, "{actual:?}");
    assert!((actual.z - expected.z).abs() < 1.0e-6, "{actual:?}");
}

#[test]
fn ghast_fireball_spawn_and_movement_gates_match_java_rules() {
    assert_ghast_attributes_spawn_and_targeting();
    assert_ghast_reflected_fireball_damage_rules();
    assert_ghast_shoot_fireball_goal_timing();
    assert_ghast_random_float_and_large_fireball_rules();
}

fn assert_ghast_attributes_spawn_and_targeting() {
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
}

fn assert_ghast_reflected_fireball_damage_rules() {
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
}

fn assert_ghast_shoot_fireball_goal_timing() {
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
}

fn assert_ghast_random_float_and_large_fireball_rules() {
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
    assert_strider_attributes_spawn_and_pathing();
    assert_strider_saddle_riding_and_suffocation();
    assert_strider_lava_float_navigation_and_sounds();
    assert_strider_finalize_spawn_jockey_rules();
}

fn assert_strider_attributes_spawn_and_pathing() {
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
    assert_eq!(
        strider_class_surface(),
        StriderClassSurface {
            blocks_building: true,
            boost_time_default: 0,
            suffocating_default: false,
            can_dispenser_equip_saddle: true,
            saddle_equip_sound: "minecraft:entity.strider.saddle",
            goal_priorities: &[
                (1, "PanicGoal"),
                (2, "BreedGoal"),
                (3, "TemptGoal"),
                (4, "StriderGoToLavaGoal"),
                (5, "FollowParentGoal"),
                (7, "RandomStrollGoal"),
                (8, "LookAtPlayerGoal<Player>"),
                (8, "RandomLookAroundGoal"),
                (9, "LookAtPlayerGoal<Strider>"),
            ],
            ambient_sound: "minecraft:entity.strider.ambient",
            hurt_sound: "minecraft:entity.strider.hurt",
            death_sound: "minecraft:entity.strider.death",
            happy_sound: "minecraft:entity.strider.happy",
            retreat_sound: "minecraft:entity.strider.retreat",
            eat_sound: "minecraft:entity.strider.eat",
            step_sound: "minecraft:entity.strider.step",
            lava_step_sound: "minecraft:entity.strider.step_lava",
            leash_offset_y_eye_height_multiplier: 0.6,
            leash_offset_z_width_multiplier: 0.4,
            ridden_input: (0.0, 0.0, 1.0),
            ridden_pitch_multiplier: 0.5,
            passengers_inherit_malus: true,
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
}

fn assert_strider_saddle_riding_and_suffocation() {
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
}

fn assert_strider_lava_float_navigation_and_sounds() {
    assert_strider_sounds_and_fall_hooks();
    assert_strider_lava_float_rules();
    assert_strider_lava_goal_and_navigation_rules();
}

fn assert_strider_sounds_and_fall_hooks() {
    assert_eq!(STRIDER_HAPPY_SOUND_RANDOM_BOUND, 140);
    assert_eq!(STRIDER_RETREAT_SOUND_RANDOM_BOUND, 60);
    assert_eq!(STRIDER_STEP_DISTANCE_INCREMENT, 0.6);
    assert_eq!(STRIDER_STEP_SOUND_VOLUME, 1.0);
    assert_eq!(STRIDER_STEP_SOUND_PITCH, 1.0);
    assert_eq!(STRIDER_LIQUID_COLLISION_HEIGHT, 8.0);
    assert_eq!(
        strider_ambient_sound(false, false),
        Some("minecraft:entity.strider.ambient")
    );
    assert_eq!(strider_ambient_sound(true, false), None);
    assert_eq!(strider_ambient_sound(false, true), None);
    assert_eq!(strider_step_sound(false), "minecraft:entity.strider.step");
    assert_eq!(
        strider_step_sound(true),
        "minecraft:entity.strider.step_lava"
    );
    assert_eq!(
        strider_eat_sound_on_food_interaction(true, false),
        Some("minecraft:entity.strider.eat")
    );
    assert_eq!(strider_eat_sound_on_food_interaction(true, true), None);
    assert!(strider_fall_damage_resets_in_lava(true));
    assert!(!strider_fall_damage_resets_in_lava(false));
}

fn assert_strider_lava_float_rules() {
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
}

fn assert_strider_lava_goal_and_navigation_rules() {
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
}

fn assert_strider_finalize_spawn_jockey_rules() {
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
    assert_witch_attributes_and_goal_gates();
    assert_witch_drink_potion_selection_and_finish();
    assert_witch_ranged_attack_potion_selection();
    assert_witch_damage_particles_and_raid_flags();
}

fn assert_witch_attributes_and_goal_gates() {
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
    assert_eq!(
        witch_class_surface(),
        WitchClassSurface {
            using_item_default: false,
            ambient_sound: "minecraft:entity.witch.ambient",
            hurt_sound: "minecraft:entity.witch.hurt",
            death_sound: "minecraft:entity.witch.death",
            celebrate_sound: "minecraft:entity.witch.celebrate",
            goal_priorities: &[
                (1, "FloatGoal"),
                (2, "RangedAttackGoal"),
                (2, "WaterAvoidingRandomStrollGoal"),
                (3, "LookAtPlayerGoal"),
                (3, "RandomLookAroundGoal"),
            ],
            target_priorities: &[
                (1, "HurtByTargetGoal"),
                (2, "NearestHealableRaiderTargetGoal"),
                (3, "NearestAttackableWitchTargetGoal"),
            ],
            particle_type: "minecraft:witch",
        }
    );
}

fn assert_witch_drink_potion_selection_and_finish() {
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
            using_time_from_main_hand_use_duration: true,
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
}

fn assert_witch_ranged_attack_potion_selection() {
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
}

fn assert_witch_damage_particles_and_raid_flags() {
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
    assert_guardian_attributes_and_dimensions();
    assert_guardian_spawn_targeting_and_attack_selectors();
    assert_guardian_attack_tick_timing();
    assert_guardian_thorns_movement_and_elder_pulse_rules();
}

fn assert_guardian_attributes_and_dimensions() {
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
}

fn assert_guardian_spawn_targeting_and_attack_selectors() {
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
}

fn assert_guardian_attack_tick_timing() {
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
}

fn assert_guardian_thorns_movement_and_elder_pulse_rules() {
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

include!("tests_e_ravager_zombie.rs");
