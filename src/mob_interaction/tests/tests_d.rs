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
    assert_slime_goal_and_sound_surfaces();
    assert_slime_synced_size_and_attack_surfaces();
    assert_slime_jump_move_control_and_split_rules();
    assert_slime_spawn_rules();
    assert_magma_cube_attributes_jump_and_spawn_rules();
}

fn assert_slime_constants_size_and_basic_state() {
    assert_eq!(SLIME_DEFAULT_SIZE, 1);
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

fn assert_slime_goal_and_sound_surfaces() {
    assert_eq!(
        SLIME_GOALS,
        [
            SlimeGoalRegistration {
                selector: SlimeGoalSelector::Goal,
                priority: 1,
                goal: SlimeGoalKind::Float,
            },
            SlimeGoalRegistration {
                selector: SlimeGoalSelector::Goal,
                priority: 2,
                goal: SlimeGoalKind::Attack,
            },
            SlimeGoalRegistration {
                selector: SlimeGoalSelector::Goal,
                priority: 3,
                goal: SlimeGoalKind::RandomDirection,
            },
            SlimeGoalRegistration {
                selector: SlimeGoalSelector::Goal,
                priority: 5,
                goal: SlimeGoalKind::KeepOnJumping,
            },
            SlimeGoalRegistration {
                selector: SlimeGoalSelector::Target,
                priority: 1,
                goal: SlimeGoalKind::NearestPlayer,
            },
            SlimeGoalRegistration {
                selector: SlimeGoalSelector::Target,
                priority: 3,
                goal: SlimeGoalKind::NearestIronGolem,
            },
        ]
    );
    assert_eq!(SLIME_SOUND_SOURCE, "hostile");
    assert_eq!(SLIME_PARTICLE_TYPE, "minecraft:item_slime");
    assert_eq!(SLIME_MAX_HEAD_X_ROT, 0);
    assert_eq!(SLIME_ATTACK_SOUND, "minecraft:entity.slime.attack");
    assert_eq!(SLIME_ATTACK_SOUND_VOLUME, 1.0);
    assert_eq!(SLIME_MOVE_CONTROL_ROT_LERP_DEGREES, 90.0);
    assert_eq!(SLIME_SPLIT_CONVERSION_TYPE, "split_on_death");
    assert_eq!(SLIME_SPLIT_SPAWN_REASON, "triggered");

    let big = SlimeFamilyState::new(SlimeFamilyKind::Slime, 4);
    assert_eq!(
        big.sound_set(),
        SlimeSoundSet {
            hurt: "minecraft:entity.slime.hurt",
            death: "minecraft:entity.slime.death",
            squish: "minecraft:entity.slime.squish",
            jump: "minecraft:entity.slime.jump",
        }
    );
    assert_eq!(
        SlimeFamilyState::new(SlimeFamilyKind::Slime, 1).sound_set(),
        SlimeSoundSet {
            hurt: "minecraft:entity.slime.hurt_small",
            death: "minecraft:entity.slime.death_small",
            squish: "minecraft:entity.slime.squish_small",
            jump: "minecraft:entity.slime.jump_small",
        }
    );
    assert_eq!(big.sound_pitch_multiplier(), 0.8);
    assert_eq!(
        SlimeFamilyState::new(SlimeFamilyKind::Slime, 1).sound_pitch_multiplier(),
        1.4
    );
}

fn assert_slime_synced_size_and_attack_surfaces() {
    let big = SlimeFamilyState::new(SlimeFamilyKind::Slime, 4);
    assert_eq!(big.default_dimension_scale(0.52, 0.52), (2.08, 2.08));
    assert_eq!(
        slime_synced_size_update(true, 0),
        SlimeSyncedSizeUpdate {
            refresh_dimensions: true,
            y_rot_from_head: true,
            body_rot_from_head: true,
            water_splash: true,
        }
    );
    assert!(!slime_synced_size_update(true, 1).water_splash);
    assert!(!slime_synced_size_update(false, 0).water_splash);
    assert!(slime_target_player_allowed(64.0, 68.0));
    assert!(!slime_target_player_allowed(64.0, 68.01));

    assert_eq!(big.attack_goal_start_timer(), 300);
    assert_eq!(
        big.attack_goal_step(true, true, true, 300, true),
        SlimeAttackGoalStep {
            can_use: true,
            can_continue: true,
            next_grow_tired_timer: 299,
            look_at_target: true,
            set_direction: true,
            aggressive: true,
        }
    );
    assert_eq!(
        big.attack_goal_step(true, true, true, 1, true),
        SlimeAttackGoalStep {
            can_use: true,
            can_continue: false,
            next_grow_tired_timer: 0,
            look_at_target: true,
            set_direction: true,
            aggressive: true,
        }
    );
    assert!(!big.attack_goal_step(false, true, true, 300, true).can_use);
}

fn assert_slime_jump_move_control_and_split_rules() {
    let slime = SlimeFamilyState::new(SlimeFamilyKind::Slime, 4);
    assert_eq!(slime.jump_delay(19, false), 29);
    assert_eq!(slime.jump_delay(19, true), 9);
    assert_eq!(slime.random_direction_next_time(59), 99);
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
    assert_phantom_class_client_and_control_surface();
    assert_phantom_size_anchor_and_flap_rules();
    assert_phantom_target_anchor_and_strategy_rules();
    assert_phantom_circle_and_move_control_rules();
    assert_phantom_swoop_cat_and_loot_rules();
}

fn assert_phantom_class_client_and_control_surface() {
    assert_eq!(
        phantom_goal_surface(),
        PhantomGoalSurface {
            goal_priorities: &[
                (1, "PhantomAttackStrategyGoal"),
                (2, "PhantomSweepAttackGoal"),
                (3, "PhantomCircleAroundAnchorGoal"),
            ],
            target_priorities: &[(1, "PhantomAttackPlayerTargetGoal")],
            move_control: "PhantomMoveControl",
            look_control: "PhantomLookControl",
            body_control: "PhantomBodyRotationControl",
            sound_source: "hostile",
            ambient_sound: "minecraft:entity.phantom.ambient",
            hurt_sound: "minecraft:entity.phantom.hurt",
            death_sound: "minecraft:entity.phantom.death",
            should_render_at_any_distance: true,
            on_climbable: false,
        }
    );
    assert_eq!(
        phantom_client_tick_surface(),
        PhantomClientTickSurface {
            flap_sound: "minecraft:entity.phantom.flap",
            flap_sound_volume_min: 0.95,
            flap_sound_volume_random_span: 0.05,
            flap_sound_pitch_min: 0.95,
            flap_sound_pitch_random_span: 0.05,
            particle: "minecraft:mycelium",
            particle_count: 2,
            particle_width_multiplier: 1.48,
            particle_height_base: 0.3,
            particle_height_anim_multiplier: 0.45,
            particle_height_scale: 2.5,
        }
    );
    assert_eq!(
        phantom_body_rotation_client_tick(45.0, 90.0),
        PhantomBodyRotation {
            y_head_rot: 45.0,
            y_body_rot: 90.0,
        }
    );
    assert_eq!(PHANTOM_TRAVEL_FLYING_FRICTION, 0.2);
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
    assert_eq!(PHANTOM_SWOOP_SOUND, "minecraft:entity.phantom.swoop");
    assert_eq!(PHANTOM_SWOOP_SOUND_VOLUME, 10.0);
    assert_eq!(PHANTOM_SWOOP_SOUND_PITCH_MIN, 0.95);
    assert_eq!(PHANTOM_SWOOP_SOUND_PITCH_RANDOM_SPAN, 0.1);
}

fn assert_phantom_circle_and_move_control_rules() {
    assert_eq!(
        phantom_circle_start(1.0, 0.0, true),
        PhantomCircleStart {
            distance: 15.0,
            height: -4.0,
            clockwise: 1.0,
        }
    );
    assert_eq!(
        phantom_circle_start(0.0, 1.0, false),
        PhantomCircleStart {
            distance: 5.0,
            height: 5.0,
            clockwise: -1.0,
        }
    );
    assert_eq!(phantom_circle_distance_tick(15.0, 1.0), (5.0, -1.0));
    assert_eq!(phantom_circle_distance_tick(6.0, -1.0), (7.0, -1.0));
    assert_eq!(
        phantom_circle_height_after_block_check(-2.0, true, true, false, false),
        1.0
    );
    assert_eq!(
        phantom_circle_height_after_block_check(2.0, false, false, true, true),
        -1.0
    );
    assert!(phantom_circle_touching_target(3.999));
    assert!(!phantom_circle_touching_target(4.0));
    assert_eq!(PHANTOM_CIRCLE_HEIGHT_RESELECT_TICKS, 350);
    assert_eq!(PHANTOM_CIRCLE_DISTANCE_RESELECT_TICKS, 250);
    assert_eq!(PHANTOM_CIRCLE_ANGLE_RESELECT_TICKS, 450);
    assert_eq!(PHANTOM_CIRCLE_ANGLE_STEP_DEGREES, 15.0);

    assert_eq!(
        phantom_move_control_surface(),
        PhantomMoveControlSurface {
            initial_speed: 0.1,
            horizontal_collision_yaw_flip: 180.0,
            turn_step_degrees: 4.0,
            fast_speed: 1.8,
            slow_speed: 0.2,
            fast_turn_threshold_degrees: 3.0,
            fast_approach_base: 0.005,
            slow_approach: 0.025,
            delta_movement_approach: 0.2,
            y_relative_scale: 0.7,
            horizontal_distance_epsilon: 1.0E-5,
        }
    );
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

include!("tests_d_vex_silverfish.rs");
