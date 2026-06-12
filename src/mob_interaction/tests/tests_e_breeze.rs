#[test]
fn breeze_util_and_shoot_when_stuck_match_java_rules() {
    assert_breeze_entity_rules();
    assert_breeze_ai_rules();
    assert_breeze_util_random_point_and_los_rules();
    assert_breeze_long_jump_rules();
    assert_breeze_shoot_rules();
    assert_breeze_shoot_when_stuck_memory_rules();
    assert_breeze_slide_rules();
}

fn assert_breeze_entity_rules() {
    assert_breeze_entity_attributes_brain_and_animation();
    assert_breeze_entity_tick_particles_and_sounds();
    assert_breeze_entity_deflection_attack_and_geometry();
}

fn assert_breeze_entity_attributes_brain_and_animation() {
    assert_eq!(
        breeze_attributes(),
        BreezeAttributes {
            movement_speed: 0.63,
            max_health: 30.0,
            follow_range: 24.0,
            attack_damage: 3.0,
            xp_reward: 10,
        }
    );
    assert_eq!(BREEZE_PATHFINDING_MALUS_ON_TRAPDOOR, -1.0);
    assert_eq!(BREEZE_PATHFINDING_MALUS_FIRE, -1.0);
    assert_eq!(
        BREEZE_BRAIN_SENSORS,
        [
            "nearest_living_entities",
            "hurt_by",
            "nearest_players",
            "breeze_attack_entity_sensor",
        ]
    );
    assert_eq!(breeze_make_brain_default_activity(), ("fight", true));
    assert_eq!(
        breeze_pose_animation_update(true, "data_pose", "shooting"),
        BreezePoseAnimationStep {
            reset_animations: true,
            start_animation: Some("shoot"),
        }
    );
    assert_eq!(
        breeze_pose_animation_update(true, "data_pose", "standing"),
        BreezePoseAnimationStep {
            reset_animations: true,
            start_animation: None,
        }
    );
    assert!(!breeze_pose_animation_update(false, "data_pose", "sliding").reset_animations);
    assert_eq!(breeze_reset_animation_stops(), ["shoot", "idle", "inhale", "long_jump"]);
}

fn assert_breeze_entity_tick_particles_and_sounds() {
    assert_eq!(BREEZE_SLIDE_PARTICLES_AMOUNT, 20);
    assert_eq!(BREEZE_IDLE_PARTICLES_AMOUNT, 1);
    assert_eq!(BREEZE_JUMP_TRAIL_PARTICLES_AMOUNT, 3);
    assert_eq!(BREEZE_JUMP_TRAIL_DURATION_TICKS, 5);
    assert_eq!(BREEZE_WHIRL_SOUND_FREQUENCY_MIN, 1);
    assert_eq!(BREEZE_WHIRL_SOUND_FREQUENCY_MAX, 80);
    assert_eq!(
        breeze_tick_step("standing", true, 0, 1, 42),
        BreezeTickStep {
            ground_particles: 1,
            jump_trail_particles: 0,
            reset_jump_trail: true,
            start_idle: true,
            start_long_jump: false,
            start_slide_back: true,
            stop_slide: true,
            next_sound_tick: 0,
            play_whirl_sound: true,
        }
    );
    assert_eq!(breeze_tick_step("sliding", false, 0, 0, 42).ground_particles, 20);
    assert_eq!(
        breeze_tick_step("long_jumping", false, 4, 2, 42).jump_trail_particles,
        3
    );
    assert_eq!(
        breeze_tick_step("long_jumping", false, 5, 2, 42).jump_trail_particles,
        0
    );
    assert_eq!(breeze_emit_ground_particles(false, false, 20), 20);
    assert_eq!(breeze_emit_ground_particles(true, false, 20), 0);
    assert_eq!(breeze_emit_ground_particles(false, true, 20), 0);
    assert!(breeze_play_ambient_sound(false, true));
    assert!(breeze_play_ambient_sound(true, false));
    assert!(!breeze_play_ambient_sound(true, true));
    assert_eq!(breeze_ambient_sound(true), "minecraft:entity.breeze.idle_ground");
    assert_eq!(breeze_ambient_sound(false), "minecraft:entity.breeze.idle_air");
    assert_eq!(
        breeze_whirl_sound(0.5, 0.25),
        BreezeWhirlSound {
            sound: "minecraft:entity.breeze.whirl",
            volume: 0.85,
            pitch: 0.9,
        }
    );
}

fn assert_breeze_entity_deflection_attack_and_geometry() {
    assert_eq!(
        BREEZE_DEFLECT_SOUND,
        ("minecraft:entity.breeze.deflect", 1.0, 1.0)
    );
    assert_eq!(BREEZE_DEATH_SOUND, "minecraft:entity.breeze.death");
    assert_eq!(BREEZE_HURT_SOUND, "minecraft:entity.breeze.hurt");
    assert_eq!(
        breeze_projectile_deflection("minecraft:arrow", true),
        ("reverse", true)
    );
    assert_eq!(
        breeze_projectile_deflection("minecraft:breeze_wind_charge", true),
        ("none", false)
    );
    assert_eq!(BREEZE_MAX_HEAD_Y_ROT, 30);
    assert_eq!(BREEZE_HEAD_ROT_SPEED, 25);
    assert_eq!(BREEZE_MOVEMENT_EMISSION, "events");
    assert!(breeze_can_attack("minecraft:player", true));
    assert!(breeze_can_attack("minecraft:iron_golem", true));
    assert!(!breeze_can_attack("minecraft:zombie", true));
    assert!(breeze_within_inner_circle_range(
        BreezeVec3::new(0.5, 64.5, 0.5),
        BreezeVec3::new(4.49, 74.49, 0.5),
    ));
    assert!(!breeze_within_inner_circle_range(
        BreezeVec3::new(0.5, 64.5, 0.5),
        BreezeVec3::new(4.5, 74.5, 0.5),
    ));
    assert_eq!(breeze_firing_y_position(64.0, 1.77), 65.185);
    assert!(breeze_invulnerable_to(Some("minecraft:breeze"), false));
    assert!(breeze_invulnerable_to(None, true));
    assert_eq!(
        breeze_fall_damage_sound(3.01),
        Some(("minecraft:entity.breeze.land", 1.0, 1.0))
    );
    assert_eq!(breeze_fall_damage_sound(3.0), None);
}

fn assert_breeze_ai_rules() {
    assert_breeze_ai_constants_and_activities();
    assert_breeze_ai_activity_update_and_slide_sink();
}

fn assert_breeze_ai_constants_and_activities() {
    assert_eq!(BREEZE_AI_SPEED_MULTIPLIER_WHEN_SLIDING, 0.6);
    assert_eq!(BREEZE_AI_JUMP_CIRCLE_INNER_RADIUS, 4.0);
    assert_eq!(BREEZE_AI_JUMP_CIRCLE_MIDDLE_RADIUS, 8.0);
    assert_eq!(BREEZE_AI_JUMP_CIRCLE_OUTER_RADIUS, 24.0);
    assert_eq!(BREEZE_AI_TICKS_TO_REMEMBER_SEEN_TARGET, 100);
    assert_eq!(BREEZE_AI_CORE_SWIM_SPEED, 0.8);
    assert_eq!(BREEZE_AI_LOOK_MIN_Y_ROT, 45);
    assert_eq!(BREEZE_AI_LOOK_MAX_X_ROT, 90);
    assert_eq!(BREEZE_AI_SLIDE_TO_TARGET_MIN_TIMEOUT, 20);
    assert_eq!(BREEZE_AI_SLIDE_TO_TARGET_MAX_TIMEOUT, 40);
    assert_eq!(BREEZE_AI_DO_NOTHING_MIN_TICKS, 20);
    assert_eq!(BREEZE_AI_DO_NOTHING_MAX_TICKS, 100);
    assert_eq!(BREEZE_AI_DO_NOTHING_WEIGHT, 1);
    assert_eq!(BREEZE_AI_RANDOM_STROLL_WEIGHT, 2);
    assert_eq!(BREEZE_AI_SLIDE_SHOOT_MEMORY_EXPIRY_TICKS, 60);
    assert_eq!(BREEZE_AI_SLIDE_SOUND, "minecraft:entity.breeze.slide");
    assert_eq!(BREEZE_AI_ACTIVITY_ORDER, ["core", "idle", "fight"]);
    assert_eq!(
        BREEZE_AI_CORE_ACTIVITY,
        [
            BreezeAiActivityStep {
                activity: "core",
                priority: 0,
                behavior: "swim",
            },
            BreezeAiActivityStep {
                activity: "core",
                priority: 0,
                behavior: "look_at_target_sink",
            },
        ]
    );
    assert_eq!(
        BREEZE_AI_IDLE_ACTIVITY,
        [
            BreezeAiActivityStep {
                activity: "idle",
                priority: 0,
                behavior: "start_attacking_nearest_attackable",
            },
            BreezeAiActivityStep {
                activity: "idle",
                priority: 1,
                behavior: "start_attacking_hurt_by_living_entity",
            },
            BreezeAiActivityStep {
                activity: "idle",
                priority: 2,
                behavior: "slide_to_target_sink",
            },
            BreezeAiActivityStep {
                activity: "idle",
                priority: 3,
                behavior: "run_one_do_nothing_or_random_stroll",
            },
        ]
    );
}

fn assert_breeze_ai_activity_update_and_slide_sink() {
    assert_eq!(
        BREEZE_AI_FIGHT_ACTIVITY,
        [
            BreezeAiActivityStep {
                activity: "fight",
                priority: 0,
                behavior: "stop_attacking_if_target_invalid",
            },
            BreezeAiActivityStep {
                activity: "fight",
                priority: 1,
                behavior: "shoot",
            },
            BreezeAiActivityStep {
                activity: "fight",
                priority: 2,
                behavior: "long_jump",
            },
            BreezeAiActivityStep {
                activity: "fight",
                priority: 3,
                behavior: "shoot_when_stuck",
            },
            BreezeAiActivityStep {
                activity: "fight",
                priority: 4,
                behavior: "slide",
            },
        ]
    );
    assert_eq!(
        BREEZE_AI_FIGHT_REQUIREMENTS,
        [
            ("attack_target", "value_present"),
            ("walk_target", "value_absent"),
        ]
    );
    assert_eq!(breeze_ai_update_activity(), ["fight", "idle"]);
    assert!(breeze_ai_stop_attack_when_target_invalid(false));
    assert!(!breeze_ai_stop_attack_when_target_invalid(true));
    assert_eq!(
        breeze_ai_slide_to_target_start(),
        BreezeSlideToTargetSinkStep {
            pose: "sliding",
            sound: Some("minecraft:entity.breeze.slide"),
            shoot_memory_expiry_ticks: None,
        }
    );
    assert_eq!(
        breeze_ai_slide_to_target_stop(true),
        BreezeSlideToTargetSinkStep {
            pose: "standing",
            sound: None,
            shoot_memory_expiry_ticks: Some(60),
        }
    );
    assert_eq!(
        breeze_ai_slide_to_target_stop(false).shoot_memory_expiry_ticks,
        None
    );
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

fn assert_breeze_long_jump_rules() {
    assert_breeze_long_jump_constants_and_memory_requirements();
    assert_breeze_long_jump_can_run_and_position_gates();
    assert_breeze_long_jump_vector_start_and_stop_rules();
    assert_breeze_long_jump_tick_rules();
}

fn assert_breeze_long_jump_constants_and_memory_requirements() {
    assert_eq!(
        BREEZE_LONG_JUMP_MEMORY_REQUIREMENTS,
        [
            ("attack_target", "value_present"),
            ("breeze_jump_cooldown", "value_absent"),
            ("breeze_jump_inhaling", "registered"),
            ("breeze_jump_target", "registered"),
            ("breeze_shoot", "value_absent"),
            ("walk_target", "value_absent"),
            ("breeze_leaving_water", "registered"),
        ]
    );
    assert_eq!(BREEZE_LONG_JUMP_REQUIRED_AIR_BLOCKS_ABOVE, 4);
    assert_eq!(BREEZE_LONG_JUMP_COOLDOWN_TICKS, 10);
    assert_eq!(BREEZE_LONG_JUMP_COOLDOWN_WHEN_HURT_TICKS, 2);
    assert_eq!(BREEZE_LONG_JUMP_INHALING_DURATION_TICKS, 10);
    assert_eq!(BREEZE_LONG_JUMP_DEFAULT_FOLLOW_RANGE, 24.0);
    assert_eq!(BREEZE_LONG_JUMP_DEFAULT_MAX_JUMP_VELOCITY, 1.4);
    assert_eq!(BREEZE_LONG_JUMP_MAX_VELOCITY_MULTIPLIER, 0.058333334);
    assert_eq!(BREEZE_LONG_JUMP_BEHAVIOR_DURATION_TICKS, 200);
    assert_eq!(BREEZE_LONG_JUMP_SNAP_TRACE_DISTANCE, 10.0);
    assert_eq!(BREEZE_LONG_JUMP_MIN_ATTACK_TARGET_DISTANCE, 4.0);
    assert_eq!(BREEZE_LONG_JUMP_SHOOT_MEMORY_EXPIRY_TICKS, 100);
    assert_eq!(BREEZE_LONG_JUMP_ALLOWED_ANGLES, [40, 55, 60, 75, 80]);
    assert_eq!(
        BREEZE_LONG_JUMP_CHARGE_SOUND,
        ("minecraft:entity.breeze.charge", "hostile", 1.0, 1.0)
    );
    assert_eq!(
        BREEZE_LONG_JUMP_JUMP_SOUND,
        ("minecraft:entity.breeze.jump", 1.0, 1.0)
    );
    assert_eq!(
        BREEZE_LONG_JUMP_LAND_SOUND,
        ("minecraft:entity.breeze.land", 1.0, 1.0)
    );
}

fn assert_breeze_long_jump_can_run_and_position_gates() {
    let runnable = BreezeLongJumpCanRunInput {
        on_ground: true,
        in_water: false,
        should_swim: false,
        jump_target_present: false,
        attack_target_present: true,
        out_of_aggro_range: false,
        too_close_for_jump: false,
        can_jump_from_current_position: true,
        snapped_target: Some((4, 65, -2)),
        target_below_dangerous: false,
        line_of_sight_to_target_center: false,
        line_of_sight_to_target_above_four: true,
    };
    assert_eq!(
        breeze_long_jump_can_run(runnable),
        BreezeLongJumpCanRunStep {
            can_run: true,
            erase_attack_target: false,
            set_jump_target: Some((4, 65, -2)),
        }
    );
    assert!(breeze_long_jump_can_run(BreezeLongJumpCanRunInput {
        jump_target_present: true,
        attack_target_present: false,
        ..runnable
    })
    .can_run);
    assert!(breeze_long_jump_can_run(BreezeLongJumpCanRunInput {
        out_of_aggro_range: true,
        ..runnable
    })
    .erase_attack_target);
    assert!(!breeze_long_jump_can_run(BreezeLongJumpCanRunInput {
        on_ground: false,
        in_water: false,
        ..runnable
    })
    .can_run);
    assert!(breeze_long_jump_out_of_aggro_range(24.0, 24.0));
    assert!(!breeze_long_jump_out_of_aggro_range(23.99, 24.0));
    assert!(breeze_long_jump_too_close_for_jump(4.0));
    assert!(!breeze_long_jump_too_close_for_jump(4.01));
    assert!(breeze_long_jump_can_jump_from_current_position(
        false,
        [true, true, true, true],
    ));
    assert!(!breeze_long_jump_can_jump_from_current_position(
        true,
        [true, true, true, true],
    ));
    assert!(!breeze_long_jump_can_jump_from_current_position(
        false,
        [true, false, true, true],
    ));
}

fn assert_breeze_long_jump_vector_start_and_stop_rules() {
    assert_eq!(breeze_long_jump_max_jump_velocity(24.0), 1.4);
    assert_eq!(
        breeze_long_jump_select_vector(
            &[
                (75, None),
                (40, Some(BreezeVec3::new(0.0, 1.2, 0.9))),
                (55, Some(BreezeVec3::new(1.0, 1.0, 0.0))),
            ],
            None,
        ),
        Some(BreezeVec3::new(0.0, 1.2, 0.9))
    );
    assert_vec3_close(
        breeze_long_jump_select_vector(
            &[(40, Some(BreezeVec3::new(0.0, 3.0, 4.0)))],
            Some(0.5),
        )
        .expect("jump boost should preserve selected vector"),
        BreezeVec3::new(0.0, 3.3, 4.0),
    );
    assert_eq!(
        breeze_long_jump_start(true, true),
        BreezeLongJumpStartStep {
            inhaling_memory_expiry_ticks: Some(10),
            pose: "inhaling",
            sound: ("minecraft:entity.breeze.charge", "hostile", 1.0, 1.0),
            look_at_jump_target: true,
        }
    );
    assert!(breeze_long_jump_can_still_use("inhaling", false));
    assert!(!breeze_long_jump_can_still_use("standing", false));
    assert_eq!(
        breeze_long_jump_stop("long_jumping"),
        BreezeLongJumpStopStep {
            pose: Some("standing"),
            erase_jump_target: true,
            erase_inhaling: true,
            erase_leaving_water: true,
        }
    );
}

fn assert_breeze_long_jump_tick_rules() {
    assert_eq!(
        breeze_long_jump_tick(BreezeLongJumpTickInput {
            pose: "inhaling",
            in_water: true,
            on_ground: false,
            leaving_water_memory_present: false,
            inhaling_memory_present: false,
            optimal_jump_vector: Some(BreezeVec3::new(0.2, 1.0, -0.4)),
            hurt_by_memory_present: false,
        }),
        BreezeLongJumpTickStep {
            erase_leaving_water_memory: false,
            set_leaving_water_memory: true,
            pose: Some("long_jumping"),
            sound: Some(("minecraft:entity.breeze.jump", 1.0, 1.0)),
            discard_friction: Some(true),
            delta_movement: Some(BreezeVec3::new(0.2, 1.0, -0.4)),
            y_rot_from_body: true,
            jump_cooldown_expiry_ticks: None,
            shoot_memory_expiry_ticks: None,
        }
    );
    assert_eq!(
        breeze_long_jump_tick(BreezeLongJumpTickInput {
            pose: "inhaling",
            in_water: false,
            on_ground: false,
            leaving_water_memory_present: true,
            inhaling_memory_present: false,
            optimal_jump_vector: None,
            hurt_by_memory_present: false,
        })
        .pose,
        Some("standing")
    );
    assert_eq!(
        breeze_long_jump_tick(BreezeLongJumpTickInput {
            pose: "long_jumping",
            in_water: false,
            on_ground: true,
            leaving_water_memory_present: false,
            inhaling_memory_present: false,
            optimal_jump_vector: None,
            hurt_by_memory_present: true,
        }),
        BreezeLongJumpTickStep {
            erase_leaving_water_memory: false,
            set_leaving_water_memory: false,
            pose: Some("standing"),
            sound: Some(("minecraft:entity.breeze.land", 1.0, 1.0)),
            discard_friction: Some(false),
            delta_movement: None,
            y_rot_from_body: false,
            jump_cooldown_expiry_ticks: Some(2),
            shoot_memory_expiry_ticks: Some(100),
        }
    );
}

fn assert_breeze_shoot_rules() {
    assert_breeze_shoot_constants_and_memory_requirements();
    assert_breeze_shoot_start_stop_gates();
    assert_breeze_shoot_tick_and_projectile();
}

fn assert_breeze_shoot_constants_and_memory_requirements() {
    assert_eq!(
        BREEZE_SHOOT_MEMORY_REQUIREMENTS,
        [
            ("attack_target", "value_present"),
            ("breeze_shoot_cooldown", "value_absent"),
            ("breeze_shoot_charging", "value_absent"),
            ("breeze_shoot_recovering", "value_absent"),
            ("breeze_shoot", "value_present"),
            ("walk_target", "value_absent"),
            ("breeze_jump_target", "value_absent"),
        ]
    );
    assert_eq!(BREEZE_SHOOT_ATTACK_RANGE_MAX_SQR, 256.0);
    assert_eq!(BREEZE_SHOOT_UNCERTAINTY_BASE, 5);
    assert_eq!(BREEZE_SHOOT_UNCERTAINTY_MULTIPLIER, 4);
    assert_eq!(BREEZE_SHOOT_PROJECTILE_MOVEMENT_SCALE, 0.7);
    assert_eq!(BREEZE_SHOOT_INITIAL_DELAY_TICKS, 15);
    assert_eq!(BREEZE_SHOOT_RECOVER_DELAY_TICKS, 4);
    assert_eq!(BREEZE_SHOOT_COOLDOWN_TICKS, 10);
    assert_eq!(BREEZE_SHOOT_BEHAVIOR_DURATION_TICKS, 20);
    assert_eq!(
        BREEZE_SHOOT_INHALE_SOUND,
        ("minecraft:entity.breeze.inhale", 1.0, 1.0)
    );
    assert_eq!(
        BREEZE_SHOOT_SOUND,
        ("minecraft:entity.breeze.shoot", 1.5, 1.0)
    );
    assert_eq!(BREEZE_SHOOT_PROJECTILE_KIND, "minecraft:breeze_wind_charge");
}

fn assert_breeze_shoot_start_stop_gates() {
    assert_eq!(
        breeze_shoot_check_start("standing", true, 255.99),
        BreezeShootStartCheck {
            can_start: true,
            erase_shoot_memory: false,
        }
    );
    assert_eq!(
        breeze_shoot_check_start("standing", true, 256.0),
        BreezeShootStartCheck {
            can_start: false,
            erase_shoot_memory: true,
        }
    );
    assert!(!breeze_shoot_check_start("shooting", true, 10.0).can_start);
    assert!(!breeze_shoot_check_start("standing", false, 10.0).can_start);
    assert!(breeze_shoot_can_still_use(true, true));
    assert!(!breeze_shoot_can_still_use(false, true));
    assert_eq!(
        breeze_shoot_start(true),
        BreezeShootStartStep {
            pose: Some("shooting"),
            charging_memory_expiry_ticks: 15,
            sound: ("minecraft:entity.breeze.inhale", 1.0, 1.0),
        }
    );
    assert_eq!(
        breeze_shoot_stop("shooting"),
        BreezeShootStopStep {
            pose: Some("standing"),
            cooldown_memory_expiry_ticks: 10,
            erase_shoot_memory: true,
        }
    );
    assert_eq!(breeze_shoot_stop("standing").pose, None);
}

fn assert_breeze_shoot_tick_and_projectile() {
    let base_input = BreezeShootTickInput {
        breeze_position: BreezeVec3::new(1.0, 64.0, 2.0),
        breeze_firing_y: 65.2,
        target_position: BreezeVec3::new(5.0, 64.0, -1.0),
        target_height: 1.8,
        target_passenger: false,
        target_present: true,
        charging_memory_present: false,
        recovering_memory_present: false,
        difficulty_id: 2,
    };
    let tick = breeze_shoot_tick(base_input);
    assert!(tick.look_at_target_eyes);
    assert_eq!(tick.recovering_memory_expiry_ticks, Some(4));
    assert_eq!(tick.sound, Some(("minecraft:entity.breeze.shoot", 1.5, 1.0)));
    let projectile = tick.projectile.expect("shoot tick should spawn projectile");
    assert_eq!(projectile.kind, "minecraft:breeze_wind_charge");
    assert_eq!(projectile.movement_scale, 0.7);
    assert_eq!(projectile.uncertainty, -3);
    assert_vec3_close(projectile.direction, BreezeVec3::new(4.0, -0.66, -3.0));
    assert_eq!(
        breeze_shoot_tick(BreezeShootTickInput {
            charging_memory_present: true,
            target_passenger: true,
            difficulty_id: 1,
            ..base_input
        })
        .projectile,
        None
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
