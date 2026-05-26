use crate::mob_interaction::*;

#[test]
fn sheep_wool_shearing_eating_and_color_rules_match_java() {
    assert_sheep_constants_and_dye_color_ids();
    let mut sheep = assert_sheep_state_color_and_shearing_flags();
    assert_sheep_interaction_eating_and_animation(&mut sheep);
    assert_sheep_spawn_and_offspring_colors();
}

fn assert_sheep_constants_and_dye_color_ids() {
    assert_eq!(SHEEP_EAT_ANIMATION_TICKS, 40);
    assert_eq!(SHEEP_SHEARED_FLAG, 16);
    assert_eq!(SHEEP_COLOR_MASK, 15);
    assert_eq!(SHEEP_MAX_HEALTH, 8.0);
    assert_eq!(SHEEP_MOVEMENT_SPEED, 0.23);
    assert_eq!(SHEEP_PANIC_SPEED, 1.25);
    assert_eq!(SHEEP_BREED_SPEED, 1.0);
    assert_eq!(SHEEP_TEMPT_SPEED, 1.1);
    assert_eq!(SHEEP_FOLLOW_PARENT_SPEED, 1.1);
    assert_eq!(SHEEP_STROLL_SPEED, 1.0);
    assert_eq!(SHEEP_LOOK_AT_PLAYER_DISTANCE, 6.0);
    assert_eq!(SHEEP_ATE_AGE_UP_SECONDS, 60);

    let colors = [
        DyeColorModel::White,
        DyeColorModel::Orange,
        DyeColorModel::Magenta,
        DyeColorModel::LightBlue,
        DyeColorModel::Yellow,
        DyeColorModel::Lime,
        DyeColorModel::Pink,
        DyeColorModel::Gray,
        DyeColorModel::LightGray,
        DyeColorModel::Cyan,
        DyeColorModel::Purple,
        DyeColorModel::Blue,
        DyeColorModel::Brown,
        DyeColorModel::Green,
        DyeColorModel::Red,
        DyeColorModel::Black,
    ];
    for (id, color) in colors.into_iter().enumerate() {
        assert_eq!(color.id(), id as u8);
        assert_eq!(DyeColorModel::by_id(id as u8), color);
    }
    assert_eq!(DyeColorModel::by_id(99), DyeColorModel::White);
}

fn assert_sheep_state_color_and_shearing_flags() -> SheepState {
    let mut sheep = SheepState::new();
    assert_eq!(sheep.color(), DyeColorModel::White);
    assert!(!sheep.is_sheared());
    sheep.set_color(DyeColorModel::Blue);
    assert_eq!(sheep.wool_data, 11);
    sheep.set_sheared(true);
    assert_eq!(sheep.wool_data, 27);
    assert_eq!(sheep.color(), DyeColorModel::Blue);
    assert!(sheep.is_sheared());
    assert!(!sheep.ready_for_shearing(true, false));
    sheep.set_sheared(false);
    assert_eq!(sheep.wool_data, 11);
    assert!(sheep.ready_for_shearing(true, false));
    assert!(!sheep.ready_for_shearing(false, false));
    assert!(!sheep.ready_for_shearing(true, true));
    sheep
}

fn assert_sheep_interaction_eating_and_animation(sheep: &mut SheepState) {
    assert_eq!(
        sheep_interaction("minecraft:shears", true, true),
        SheepInteraction::ShearServer
    );
    assert_eq!(
        sheep_interaction("minecraft:shears", false, true),
        SheepInteraction::ConsumeClientOrNotReady
    );
    assert_eq!(
        sheep_interaction("minecraft:wheat", true, true),
        SheepInteraction::Delegate
    );

    sheep.set_sheared(true);
    assert_eq!(sheep.ate(true), Some(60));
    assert!(!sheep.is_sheared());
    sheep.set_sheared(true);
    assert_eq!(sheep.ate(false), None);
    assert!(!sheep.is_sheared());
    assert!(sheep.handle_entity_event(10));
    assert_eq!(sheep.eat_animation_tick, 40);
    assert!(!sheep.handle_entity_event(9));
    sheep.client_ai_step();
    assert_eq!(sheep.eat_animation_tick, 39);
    sheep.eat_animation_tick = 0;
    assert_eq!(sheep.head_eat_position_scale(0.0), 0.0);
    sheep.eat_animation_tick = 3;
    assert!((sheep.head_eat_position_scale(1.0) - 0.5).abs() < 0.00001);
    sheep.eat_animation_tick = 20;
    assert_eq!(sheep.head_eat_position_scale(0.0), 1.0);
    sheep.eat_animation_tick = 39;
    assert!((sheep.head_eat_position_scale(1.0) - 0.5).abs() < 0.00001);
    assert!(sheep.head_eat_angle_scale(0.0, 30.0) > 0.0);
    sheep.eat_animation_tick = 0;
    assert!((sheep.head_eat_angle_scale(0.0, 30.0) - std::f32::consts::PI / 6.0).abs() < 0.00001);
}

fn assert_sheep_spawn_and_offspring_colors() {
    assert_eq!(sheep_spawn_color(false, false, 0, 0), DyeColorModel::Black);
    assert_eq!(sheep_spawn_color(false, false, 5, 0), DyeColorModel::Gray);
    assert_eq!(
        sheep_spawn_color(false, false, 10, 0),
        DyeColorModel::LightGray
    );
    assert_eq!(sheep_spawn_color(false, false, 15, 0), DyeColorModel::Brown);
    assert_eq!(sheep_spawn_color(false, false, 18, 0), DyeColorModel::White);
    assert_eq!(
        sheep_spawn_color(false, false, 18, 499),
        DyeColorModel::Pink
    );
    assert_eq!(sheep_spawn_color(true, false, 18, 0), DyeColorModel::Brown);
    assert_eq!(sheep_spawn_color(false, true, 18, 0), DyeColorModel::Black);
    assert_eq!(
        sheep_offspring_color(
            Some(DyeColorModel::Purple),
            DyeColorModel::Red,
            DyeColorModel::Blue,
            true,
        ),
        DyeColorModel::Purple
    );
    assert_eq!(
        sheep_offspring_color(None, DyeColorModel::Red, DyeColorModel::Blue, false),
        DyeColorModel::Blue
    );
}

#[test]
fn squid_and_glow_squid_ink_flee_and_dark_ticks_match_java_rules() {
    assert_squid_constants();
    assert_squid_state_events();
    assert_squid_flee_vectors_and_bubbles();
    assert_glow_squid_dark_ticks_and_spawn_rules();
}

fn assert_squid_constants() {
    assert_eq!(SQUID_MAX_HEALTH, 10.0);
    assert_eq!(SQUID_DEFAULT_GRAVITY, 0.08);
    assert_eq!(SQUID_SOUND_VOLUME, 0.4);
    assert_eq!(SQUID_BABY_WIDTH, 0.5);
    assert_eq!(SQUID_BABY_HEIGHT, 0.63);
    assert_eq!(SQUID_BABY_EYE_HEIGHT, 0.37);
    assert_eq!(SQUID_INK_PARTICLE_COUNT, 30);
    assert_eq!(SQUID_INK_BABY_OFFSET_SCALE, 0.1);
    assert_eq!(SQUID_INK_ADULT_OFFSET_SCALE, 0.3);
    assert_eq!(SQUID_FLEE_SPEED, 3.0);
    assert_eq!(SQUID_FLEE_MIN_DISTANCE, 5.0);
    assert_eq!(SQUID_FLEE_MAX_DISTANCE, 10.0);
    assert_eq!(SQUID_FLEE_DISTANCE_SQUARED, 100.0);
    assert_eq!(SQUID_FLEE_VECTOR_SCALE, 20.0);
    assert_eq!(SQUID_BUBBLE_INTERVAL_TICKS, 10);
    assert_eq!(SQUID_BUBBLE_PHASE_TICK, 5);
    assert_eq!(GLOW_SQUID_DEFAULT_DARK_TICKS_REMAINING, 0);
    assert_eq!(GLOW_SQUID_DARK_TICKS_ON_HURT, 100);
    assert_eq!(GLOW_SQUID_SPAWN_SEA_LEVEL_OFFSET, 33);
}

fn assert_squid_state_events() {
    let mut squid = SquidState::new(0.0);
    assert_eq!(squid.tentacle_speed, 0.2);
    assert!(!squid.has_movement_vector());
    squid.movement_vector = (0.01, 0.0, 0.0);
    assert!(squid.has_movement_vector());
    squid.tentacle_movement = 2.0;
    assert!(squid.handle_entity_event(19));
    assert_eq!(squid.tentacle_movement, 0.0);
    assert!(!squid.handle_entity_event(18));

    assert!(squid_hurt_spawns_ink(true, true));
    assert!(!squid_hurt_spawns_ink(true, false));
    assert!(!squid_hurt_spawns_ink(false, true));
}

fn assert_squid_flee_vectors_and_bubbles() {
    assert!(squid_flee_can_use(true, true, 99.99));
    assert!(!squid_flee_can_use(true, true, 100.0));
    assert!(!squid_flee_can_use(false, true, 1.0));

    let flee = squid_flee_vector((0.0, 0.0, 0.0), (-5.0, 0.0, 0.0), true, false)
        .expect("water target produces flee vector");
    assert!((flee.0 - 0.15).abs() < 0.00001);
    assert_eq!(flee.1, 0.0);
    assert_eq!(flee.2, 0.0);
    let far_flee = squid_flee_vector((0.0, 0.0, 0.0), (-10.0, 0.0, 0.0), true, false)
        .expect("10 block target still has a Java flee vector");
    assert!((far_flee.0 - 0.1).abs() < 0.00001);
    assert_eq!(far_flee.1, 0.0);
    assert_eq!(far_flee.2, 0.0);
    let air_flee = squid_flee_vector((0.0, 0.0, 0.0), (0.0, -5.0, 0.0), false, true)
        .expect("air target is allowed but clamps vertical vector");
    assert_eq!(air_flee.1, 0.0);
    assert_eq!(
        squid_flee_vector((0.0, 0.0, 0.0), (-5.0, 0.0, 0.0), false, false),
        None
    );
    assert!(squid_flee_emits_bubble(5));
    assert!(squid_flee_emits_bubble(15));
    assert!(!squid_flee_emits_bubble(6));
}

fn assert_glow_squid_dark_ticks_and_spawn_rules() {
    let mut glow = GlowSquidState::new();
    assert_eq!(glow.dark_ticks_remaining, 0);
    glow.on_hurt(true);
    assert_eq!(glow.dark_ticks_remaining, 100);
    glow.ai_step();
    assert_eq!(glow.dark_ticks_remaining, 99);
    glow.on_hurt(false);
    assert_eq!(glow.dark_ticks_remaining, 99);
    assert!(glow_squid_spawn_allowed(30, 63, 0, true));
    assert!(!glow_squid_spawn_allowed(31, 63, 0, true));
    assert!(!glow_squid_spawn_allowed(30, 63, 1, true));
    assert!(!glow_squid_spawn_allowed(30, 63, 0, false));
}

#[test]
fn creeper_fuse_ignition_power_and_cloud_match_java_rules() {
    assert_creeper_state_save_and_igniter_rules();
    assert_creeper_fuse_tick_and_lingering_cloud();
    assert_creeper_powered_loot_and_target_rules();
}

fn assert_creeper_state_save_and_igniter_rules() {
    let mut creeper = CreeperState::new();
    assert_eq!(creeper.swell_dir, CREEPER_DEFAULT_SWELL_DIR);
    assert_eq!(creeper.max_swell, 30);
    assert_eq!(creeper.explosion_radius, 3);
    assert_eq!(CREEPER_MOVEMENT_SPEED, 0.25);
    assert_eq!(CREEPER_CAT_AVOID_DISTANCE, 6.0);
    assert_eq!(CREEPER_LOOK_AT_PLAYER_DISTANCE, 8.0);

    creeper.cause_fall_damage(40.0);
    assert_eq!(creeper.swell, 25);
    assert_eq!(creeper.swelling(0.5), 12.5 / 28.0);

    let loaded = CreeperState::read_save_data(true, Some(40), Some(5), true);
    assert!(loaded.powered);
    assert!(loaded.ignited);
    assert_eq!(loaded.max_swell, 40);
    assert_eq!(loaded.explosion_radius, 5);
    assert_eq!(loaded.effective_explosion_radius(), 10.0);

    assert_eq!(
        creeper_igniter_use(false, false),
        CreeperIgniterUse::NotIgniter
    );
    assert_eq!(
        creeper_igniter_use(true, false),
        CreeperIgniterUse::ConsumeOne
    );
    assert_eq!(
        creeper_igniter_use(true, true),
        CreeperIgniterUse::DamageOne
    );
}

fn assert_creeper_fuse_tick_and_lingering_cloud() {
    let mut ticking = CreeperState::new();
    ticking.ignite();
    let first = ticking.tick(0);
    assert!(first.primed_sound);
    assert!(first.prime_fuse_game_event);
    assert_eq!(ticking.swell_dir, 1);
    assert_eq!(ticking.swell, 1);
    let second = ticking.tick(0);
    assert!(!second.primed_sound);
    assert_eq!(ticking.swell, 2);

    ticking.swell = ticking.max_swell - 1;
    let exploded = ticking.tick(2);
    assert_eq!(ticking.swell, ticking.max_swell);
    assert!(!ticking.alive);
    assert_eq!(exploded.explosion_radius, Some(3.0));
    assert_eq!(exploded.lingering_cloud, Some(creeper_lingering_cloud()));
    assert_eq!(
        exploded.lingering_cloud.unwrap(),
        CreeperLingeringCloud {
            radius: 2.5,
            radius_on_use: -0.5,
            wait_time: 10,
            duration: 300,
            potion_duration_scale: 0.25,
            radius_per_tick: -2.5 / 300.0,
        }
    );
}

fn assert_creeper_powered_loot_and_target_rules() {
    let mut powered = CreeperState::new();
    powered.thunder_hit();
    assert_eq!(powered.effective_explosion_radius(), 6.0);
    assert!(powered.killed_entity_drops_charged_creeper_loot(true));
    assert!(!powered.killed_entity_drops_charged_creeper_loot(true));
    assert!(!CreeperState::new().killed_entity_drops_charged_creeper_loot(true));
    assert!(powered.can_target(false));
    assert!(!powered.can_target(true));
}

#[test]
fn slime_and_magma_cube_size_split_spawn_and_jump_match_java_rules() {
    assert_slime_constants_size_and_basic_state();
    assert_slime_jump_move_control_and_split_rules();
    assert_slime_spawn_rules();
    assert_magma_cube_attributes_jump_and_spawn_rules();
}

fn assert_slime_constants_size_and_basic_state() {
    assert_eq!(clamp_slime_size(0), 1);
    assert_eq!(clamp_slime_size(200), 127);
    assert_eq!(SLIME_MAX_NATURAL_SIZE, 4);
    assert_eq!(SLIME_ATTACK_TARGET_VERTICAL_RANGE, 4.0);
    assert_eq!(SLIME_ATTACK_GROW_TIRED_TICKS, 300);
    assert_eq!(SLIME_RANDOM_DIRECTION_MIN_TICKS, 40);
    assert_eq!(SLIME_RANDOM_DIRECTION_RANDOM_BOUND, 60);
    assert_eq!(SLIME_FLOAT_WANTED_MOVEMENT, 1.2);
    assert_eq!(SLIME_KEEP_JUMPING_WANTED_MOVEMENT, 1.0);

    assert_eq!(slime_finalize_spawn_size(0, 0.0, 0.0), 1);
    assert_eq!(slime_finalize_spawn_size(0, 1.0, 0.49), 2);
    assert_eq!(slime_finalize_spawn_size(1, 1.0, 0.49), 4);
    assert_eq!(slime_finalize_spawn_size(2, 1.0, 0.0), 4);

    let slime = SlimeFamilyState::new(SlimeFamilyKind::Slime, 4);
    assert_eq!(slime.serialized_size(), 3);
    assert_eq!(
        slime.attributes(),
        SlimeFamilyAttributes {
            max_health: 16.0,
            movement_speed: 0.6,
            attack_damage: 4.0,
            armor: 0.0,
            xp_reward: 4,
        }
    );
    assert!(slime.deals_damage(true));
    assert!(!SlimeFamilyState::new(SlimeFamilyKind::Slime, 1).deals_damage(true));
}

fn assert_slime_jump_move_control_and_split_rules() {
    let slime = SlimeFamilyState::new(SlimeFamilyKind::Slime, 4);
    assert_eq!(slime.jump_delay(19, false), 29);
    assert_eq!(slime.jump_delay(19, true), 9);
    assert_eq!(
        slime.float_goal_step(true, false, true, 0.79),
        SlimeFloatGoalStep {
            can_use: true,
            jump: true,
            wanted_movement: 1.2,
        }
    );
    assert_eq!(
        slime.float_goal_step(false, false, true, 0.0),
        SlimeFloatGoalStep {
            can_use: false,
            jump: false,
            wanted_movement: 0.0,
        }
    );
    assert!(slime.keep_on_jumping_can_use(false));
    assert!(!slime.keep_on_jumping_can_use(true));
    assert!(slime.random_direction_can_use(false, false, true, false, false, true));
    assert!(!slime.random_direction_can_use(true, true, false, false, false, true));
    let moving_slime = SlimeMoveControlInput {
        operation_move_to: true,
        on_ground: true,
        speed_modifier: 1.2,
        movement_speed_attribute: 0.6,
        jump_delay: 0,
        random_0_to_19: 5,
        aggressive: true,
    };
    assert_eq!(
        slime.move_control_step(moving_slime),
        SlimeMoveControlStep {
            speed: 0.72,
            jump: true,
            play_jump_sound: true,
            next_jump_delay: 5,
            zero_strafe: false,
        }
    );
    assert_eq!(
        slime.move_control_step(SlimeMoveControlInput {
            jump_delay: 3,
            aggressive: false,
            ..moving_slime
        }),
        SlimeMoveControlStep {
            speed: 0.0,
            jump: false,
            play_jump_sound: false,
            next_jump_delay: 2,
            zero_strafe: true,
        }
    );
    assert_eq!(
        slime
            .move_control_step(SlimeMoveControlInput {
                on_ground: false,
                jump_delay: 3,
                aggressive: false,
                ..moving_slime
            })
            .speed,
        0.72
    );
    assert_eq!(slime.sound_volume(), 1.6);
    assert_eq!(slime.passenger_attachment_y(2.04, 1.0), 1.9775);

    let children = slime.split_children(2);
    assert_eq!(children.len(), 4);
    assert_eq!(
        children[0],
        SlimeSplitChild {
            size: 2,
            x_offset: -1.0,
            y_offset: 0.5,
            z_offset: -1.0,
        }
    );
    assert_eq!(children[3].size, 2);
    assert!(SlimeFamilyState::new(SlimeFamilyKind::Slime, 1)
        .split_children(2)
        .is_empty());

    let mut squish = SlimeFamilyState::read_save_data(SlimeFamilyKind::Slime, Some(3), Some(false));
    assert_eq!(squish.size, 4);
    assert_eq!(squish.tick_squish(true), 128);
    assert_eq!(squish.target_squish, -0.3);
    assert_eq!(squish.tick_squish(false), 0);
    assert_eq!(squish.target_squish, 0.6);
}

fn assert_slime_spawn_rules() {
    let surface = SlimeSpawnRuleInput {
        peaceful: false,
        spawner_reason: false,
        mob_spawn_rules_pass: true,
        allows_surface_slime_spawns_biome: true,
        y: 60,
        surface_slime_spawn_chance: 0.25,
        surface_random_float: 0.24,
        max_local_raw_brightness: 3,
        brightness_random_bound_8: 3,
        worldgen_level: false,
        slime_chunk: false,
        underground_random_bound_10: 9,
    };
    assert!(slime_spawn_allowed(surface));
    assert!(!slime_spawn_allowed(SlimeSpawnRuleInput {
        peaceful: true,
        ..surface
    }));
    assert!(slime_spawn_allowed(SlimeSpawnRuleInput {
        spawner_reason: true,
        allows_surface_slime_spawns_biome: false,
        mob_spawn_rules_pass: true,
        ..surface
    }));
    assert!(slime_spawn_allowed(SlimeSpawnRuleInput {
        allows_surface_slime_spawns_biome: false,
        y: 20,
        surface_random_float: 1.0,
        worldgen_level: true,
        slime_chunk: true,
        underground_random_bound_10: 0,
        ..surface
    }));
}

fn assert_magma_cube_attributes_jump_and_spawn_rules() {
    let magma = SlimeFamilyState::new(SlimeFamilyKind::MagmaCube, 4);
    assert_eq!(
        magma.attributes(),
        SlimeFamilyAttributes {
            max_health: 16.0,
            movement_speed: 0.6,
            attack_damage: 6.0,
            armor: 12.0,
            xp_reward: 4,
        }
    );
    assert_eq!(MAGMA_CUBE_CREATE_ATTRIBUTES_MOVEMENT_SPEED, 0.2);
    assert!(SlimeFamilyState::new(SlimeFamilyKind::MagmaCube, 1).deals_damage(true));
    assert_eq!(magma.jump_delay(0, false), 40);
    assert_eq!(magma.ground_jump_y_velocity(0.42), 0.82);
    assert_eq!(magma.lava_jump_y_velocity(), Some(0.42000002));
    assert!(magma_cube_spawn_allowed(false));
    assert!(!magma_cube_spawn_allowed(true));
    assert!(!magma_cube_is_on_fire());
}

#[test]
fn phantom_size_anchor_swoop_and_cat_gates_match_java_rules() {
    assert_phantom_size_anchor_and_flap_rules();
    assert_phantom_target_anchor_and_strategy_rules();
    assert_phantom_swoop_cat_and_loot_rules();
}

fn assert_phantom_size_anchor_and_flap_rules() {
    let mut phantom = PhantomState::new();
    phantom.set_size(200);
    assert_eq!(phantom.size, 64);
    phantom.set_size(-5);
    assert_eq!(phantom.size, 0);

    let finalized = PhantomState::finalize_spawn(PhantomBlockPos { x: 3, y: 70, z: -2 });
    assert_eq!(finalized.size, 0);
    assert_eq!(
        finalized.anchor_point,
        Some(PhantomBlockPos { x: 3, y: 75, z: -2 })
    );
    assert_eq!(
        PhantomState::read_save_data(Some(4), Some(PhantomBlockPos { x: 1, y: 2, z: 3 }))
            .attributes(),
        PhantomAttributes {
            attack_damage: 10.0,
            xp_reward: 5,
            dimensions_scale: 1.6,
        }
    );

    assert_eq!(PHANTOM_FLAP_DEGREES_PER_TICK, 7.448451);
    assert_eq!(PHANTOM_TICKS_PER_FLAP, 25);
    assert_eq!(phantom_unique_flap_tick_offset(7), 21);
    assert!(finalized.is_flapping(7, 4));
    assert_eq!(phantom_target_scan_tick(2), Some(1));
    assert_eq!(phantom_target_scan_tick(0), None);
    assert_eq!(phantom_target_scan_reset_ticks(), 60);
    assert_eq!(PHANTOM_TARGET_RANGE, 64.0);
    assert_eq!(PHANTOM_TARGET_BOX_INFLATE_XZ, 16.0);
    assert_eq!(PHANTOM_TARGET_BOX_INFLATE_Y, 64.0);
}

fn assert_phantom_target_anchor_and_strategy_rules() {
    let target = PhantomBlockPos {
        x: 10,
        y: 50,
        z: -10,
    };
    assert_eq!(
        phantom_anchor_above_target(target, 19, 80),
        PhantomBlockPos {
            x: 10,
            y: 89,
            z: -10
        }
    );
    assert_eq!(
        phantom_stop_anchor_after_heightmap(4, 63, 5, 19),
        PhantomBlockPos { x: 4, y: 92, z: 5 }
    );

    let started = phantom_attack_strategy_start(target, 0, 63);
    assert_eq!(started.attack_phase, PhantomAttackPhase::Circle);
    assert_eq!(started.next_sweep_tick, 10);
    assert_eq!(
        started.anchor_point,
        Some(PhantomBlockPos {
            x: 10,
            y: 70,
            z: -10
        })
    );

    let waiting = phantom_attack_strategy_tick(PhantomAttackPhase::Circle, 2, target, 0, 3, 63);
    assert_eq!(waiting.next_sweep_tick, 1);
    assert_eq!(waiting.attack_phase, PhantomAttackPhase::Circle);
    assert!(!waiting.played_swoop_sound);

    let swoop = phantom_attack_strategy_tick(PhantomAttackPhase::Circle, 1, target, 3, 3, 63);
    assert_eq!(swoop.attack_phase, PhantomAttackPhase::Swoop);
    assert_eq!(swoop.next_sweep_tick, 220);
    assert!(swoop.played_swoop_sound);
}

fn assert_phantom_swoop_cat_and_loot_rules() {
    assert_eq!(
        phantom_can_continue_swoop(true, true, false, false, PhantomAttackPhase::Swoop, true),
        PhantomSwoopContinuation::StopScaredOfCat
    );
    assert_eq!(
        phantom_can_continue_swoop(true, true, true, false, PhantomAttackPhase::Swoop, false),
        PhantomSwoopContinuation::StopCreativeOrSpectatorPlayer
    );
    assert_eq!(
        phantom_can_continue_swoop(true, true, false, false, PhantomAttackPhase::Swoop, false),
        PhantomSwoopContinuation::Continue
    );
    assert_eq!(
        phantom_swoop_tick(true, false, false, false),
        PhantomSwoopTick::HitTarget {
            level_event: Some(1039)
        }
    );
    assert_eq!(
        phantom_swoop_tick(false, true, false, false),
        PhantomSwoopTick::CancelledToCircle
    );
    assert_eq!(PHANTOM_SWEEP_CAT_SEARCH_TICK_DELAY, 20);
    assert_eq!(PHANTOM_CAT_AVOID_INFLATE, 16.0);
    assert_eq!(PHANTOM_LOOT_ITEM, "minecraft:phantom_membrane");
    assert!(!phantom_burns_in_daylight());
    assert!(!phantom_uses_nearest_players_memory());
    assert_eq!(phantom_membrane_loot_roll(false, 1, 3), 0);
    assert_eq!(phantom_membrane_loot_roll(true, 1, 2), 3);
}

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

    let flags = vex_set_charging(0, true);
    assert!(vex_is_charging(flags));
    assert!(!vex_is_charging(vex_set_charging(flags, false)));
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
    assert!(vex_charge_attack_can_use(true, true, false, 0, 4.1));
    assert!(!vex_charge_attack_can_use(true, true, false, 1, 4.1));
    assert!(!vex_charge_attack_can_use(true, true, true, 0, 4.1));
    assert!(!vex_charge_attack_can_use(true, true, false, 0, 4.0));
    assert!(vex_charge_attack_can_continue(true, true, true, true));
    assert!(!vex_charge_attack_can_continue(true, false, true, true));
    assert_eq!(vex_charge_attack_tick(true, 16.0), (true, false));
    assert_eq!(vex_charge_attack_tick(false, 8.9), (false, true));
    assert_eq!(vex_charge_attack_tick(false, 9.0), (false, false));

    assert!(vex_copy_owner_target_can_use(true, true, true));
    assert!(!vex_copy_owner_target_can_use(true, false, true));
    assert!(vex_random_move_can_use(false, 0));
    assert!(!vex_random_move_can_use(false, 1));
    assert_eq!(VEX_RANDOM_MOVE_ATTEMPTS, 3);
    assert_eq!(VEX_RANDOM_MOVE_XZ_RANDOM_BOUND, 15);
    assert_eq!(VEX_RANDOM_MOVE_Y_RANDOM_BOUND, 11);
    assert_eq!(VEX_RANDOM_MOVE_XZ_OFFSET, 7);
    assert_eq!(VEX_RANDOM_MOVE_Y_OFFSET, 5);
    assert_eq!(VEX_RANDOM_MOVE_SPEED, 0.25);
    assert_eq!(VEX_MOVE_ACCELERATION, 0.05);
    assert_eq!(VEX_MOVE_CLOSE_DAMPING, 0.5);
    assert_eq!(VEX_OWNER_TARGET_RANGE, 16.0);
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
