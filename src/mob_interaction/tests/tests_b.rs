use crate::mob_interaction::*;

#[test]
fn turtle_home_egg_laying_growth_and_egg_block_match_java() {
    assert_turtle_constants_attributes_and_state();
    assert_turtle_food_breeding_and_lay_step_rules();
    assert_turtle_growth_spawn_navigation_and_damage_rules();
    assert_turtle_egg_destruction_tick_and_placement_rules();
}

fn assert_turtle_constants_attributes_and_state() {
    assert_eq!(TURTLE_BABY_SCALE, 0.3);
    assert_eq!(TURTLE_AMBIENT_SOUND_INTERVAL, 200);
    assert_eq!(TURTLE_BREED_PARENT_AGE, 6000);
    assert_eq!(TURTLE_LAY_EGG_DELAY_TICKS, 200);
    assert_eq!(TURTLE_LAY_EGG_PARTICLE_INTERVAL_TICKS, 5);
    assert_eq!(TURTLE_GO_HOME_RANDOM_INTERVAL, 700);
    assert_eq!(TURTLE_GO_HOME_GIVE_UP_TICKS, 600);
    assert_eq!(TURTLE_GO_TO_WATER_GIVE_UP_TICKS, 1200);
    assert_eq!(TURTLE_GO_TO_WATER_RECALC_INTERVAL_TICKS, 160);
    assert_eq!(TURTLE_TRAVEL_XZ_RANGE, 512);
    assert_eq!(TURTLE_TRAVEL_Y_RANGE, 4);
    assert_eq!(TURTLE_EGG_MAX_HATCH_LEVEL, 2);
    assert_eq!(TURTLE_EGG_MIN_EGGS, 1);
    assert_eq!(TURTLE_EGG_MAX_EGGS, 4);
    assert_eq!(TURTLE_EGG_STEP_RANDOM_BOUND, 100);
    assert_eq!(TURTLE_EGG_FALL_RANDOM_BOUND, 3);
    assert_eq!(TURTLE_HATCHLING_AGE, -24000);
    assert_eq!(
        turtle_attributes(),
        TurtleAttributes {
            max_health: 30.0,
            movement_speed: 0.25,
            step_height: 1.0,
        }
    );

    let mut turtle = TurtleStateModel::new((10, 63, -4));
    assert_eq!(turtle.home_pos, (10, 63, -4));
    turtle.set_laying_egg(true);
    assert!(turtle.laying_egg);
    assert_eq!(turtle.lay_egg_counter, 1);
    turtle.set_laying_egg(false);
    assert_eq!(turtle.lay_egg_counter, 0);
}

fn assert_turtle_food_breeding_and_lay_step_rules() {
    assert!(turtle_food_item("minecraft:seagrass"));
    assert!(!turtle_food_item("minecraft:kelp"));
    assert!(turtle_can_fall_in_love(true, false));
    assert!(!turtle_can_fall_in_love(true, true));
    assert_eq!(
        turtle_breed_plan(true, true),
        TurtleBreedPlan {
            has_egg_after: true,
            parent_age: 6000,
            reset_love: true,
            bred_animals_stat: true,
            xp_min: 1,
            xp_max: 7,
        }
    );
    assert_eq!(turtle_breed_plan(false, false).xp_max, 0);

    assert_eq!(
        turtle_lay_egg_step(true, false, 0, false, true, true, 2),
        TurtleLayEggStep::StartLaying
    );
    assert_eq!(
        turtle_lay_egg_step(true, true, 5, false, true, true, 2),
        TurtleLayEggStep::DigParticles {
            level_event: 2001,
            game_event: "minecraft:entity_action",
        }
    );
    assert_eq!(
        turtle_lay_egg_step(true, true, 201, false, true, true, 2),
        TurtleLayEggStep::PlaceEggs {
            egg_count: 3,
            sound: "minecraft:entity.turtle.lay_egg",
            game_event: "minecraft:block_place",
            has_egg_after: false,
            laying_after: false,
            in_love_time: 600,
        }
    );
    assert_eq!(
        turtle_lay_egg_step(true, true, 201, true, true, true, 2),
        TurtleLayEggStep::None
    );
}

fn assert_turtle_growth_spawn_navigation_and_damage_rules() {
    assert_eq!(
        turtle_grow_drop_loot(true, true),
        Some("minecraft:gameplay/turtle_grow")
    );
    assert_eq!(turtle_grow_drop_loot(true, false), None);
    assert!(turtle_spawn_allowed(66, 63, true, true));
    assert!(!turtle_spawn_allowed(67, 63, true, true));

    assert!(turtle_go_home_can_use(false, true, 1.0, false));
    assert!(turtle_go_home_can_use(false, false, 64.0, true));
    assert!(!turtle_go_home_can_use(true, true, 100.0, true));
    assert!(turtle_lay_egg_goal_can_use(true, 8.99, true));
    assert!(!turtle_lay_egg_goal_can_use(true, 9.0, true));
    assert!(turtle_go_to_water_can_use(true, false, true, true, true));
    assert!(turtle_go_to_water_can_use(false, false, false, false, true));
    assert!(!turtle_go_to_water_can_use(false, false, false, true, true));
    assert!(turtle_travel_can_use(false, false, true));
    assert!(!turtle_travel_can_use(true, false, true));
    assert_eq!(
        turtle_water_travel_sinking(false, false, 0.0),
        Some((0.0, -0.005, 0.0))
    );
    assert_eq!(turtle_water_travel_sinking(false, true, 19.0), None);
    assert!(!turtle_can_be_leashed());
    assert_eq!(turtle_lightning_damage(), f32::MAX);
}

fn assert_turtle_egg_destruction_tick_and_placement_rules() {
    assert_eq!(
        turtle_egg_destroy_plan(2, false, false, true, true, false, true),
        TurtleEggDestroyPlan::DecreaseEggs {
            eggs_after: 1,
            sound: "minecraft:entity.turtle.egg_break",
            destroy_block: false,
            game_event: Some("minecraft:block_destroy"),
        }
    );
    assert_eq!(
        turtle_egg_destroy_plan(1, false, false, true, false, true, true),
        TurtleEggDestroyPlan::DecreaseEggs {
            eggs_after: 0,
            sound: "minecraft:entity.turtle.egg_break",
            destroy_block: true,
            game_event: None,
        }
    );
    assert_eq!(
        turtle_egg_destroy_plan(2, true, false, true, true, true, true),
        TurtleEggDestroyPlan::None
    );
    assert_eq!(
        turtle_egg_destroy_plan(2, false, false, true, false, false, true),
        TurtleEggDestroyPlan::None
    );
    assert_eq!(
        turtle_egg_random_tick_plan(true, true, 1, 3),
        TurtleEggRandomTickPlan::Crack {
            hatch: 2,
            sound: "minecraft:entity.turtle.egg_crack",
            game_event: "minecraft:block_change",
        }
    );
    assert_eq!(
        turtle_egg_random_tick_plan(true, true, 2, 3),
        TurtleEggRandomTickPlan::Hatch {
            hatchlings: 3,
            hatchling_age: -24000,
            sound: "minecraft:entity.turtle.egg_hatch",
            game_event: "minecraft:block_destroy",
        }
    );
    assert_eq!(
        turtle_egg_random_tick_plan(false, true, 2, 3),
        TurtleEggRandomTickPlan::None
    );
    assert!(turtle_egg_can_be_replaced(false, true, 3));
    assert!(!turtle_egg_can_be_replaced(false, true, 4));
    assert_eq!(turtle_egg_placement_eggs(None), 1);
    assert_eq!(turtle_egg_placement_eggs(Some(3)), 4);
    assert_eq!(turtle_egg_placement_eggs(Some(4)), 4);
}

#[test]
fn armadillo_roll_scute_brush_and_scare_gates_match_java() {
    assert_armadillo_constants_and_attributes();
    assert_armadillo_state_model_rules();
    assert_armadillo_food_spawn_and_scute_drop_rules();
    assert_armadillo_scare_roll_and_hurt_rules();
    assert_armadillo_interactions_sounds_and_animation_ticks();
}

fn assert_armadillo_constants_and_attributes() {
    assert_eq!(ARMADILLO_BABY_SCALE, 0.6);
    assert_eq!(ARMADILLO_MAX_HEAD_ROTATION_EXTENT, 32.5);
    assert_eq!(ARMADILLO_SCARE_CHECK_INTERVAL, 80);
    assert_eq!(ARMADILLO_SCARE_DISTANCE_HORIZONTAL, 7.0);
    assert_eq!(ARMADILLO_SCARE_DISTANCE_VERTICAL, 2.0);
    assert_eq!(ARMADILLO_SCUTE_DROP_MIN_TICKS, 6000);
    assert_eq!(ARMADILLO_SCUTE_DROP_RANDOM_BOUND, 6000);
    assert_eq!(ARMADILLO_BRUSH_DAMAGE, 16);
    assert_eq!(ARMADILLO_BALL_UP_STAY_IN_STATE_TICKS, 6000);
    assert_eq!(ARMADILLO_DANGER_DELAY_TICKS, 5);
    assert_eq!(ARMADILLO_DANGER_THRESHOLD_TICKS, 75);
    assert_eq!(ARMADILLO_PEEK_EVENT, 64);
    assert_eq!(
        armadillo_attributes(),
        ArmadilloAttributes {
            max_health: 12.0,
            movement_speed: 0.14,
        }
    );
}

fn assert_armadillo_state_model_rules() {
    assert_eq!(ArmadilloStateModel::Idle.id(), 0);
    assert_eq!(ArmadilloStateModel::Unrolling.id(), 3);
    assert_eq!(ArmadilloStateModel::Scared.serialized_name(), "scared");
    assert_eq!(armadillo_state_by_id(99), ArmadilloStateModel::Idle);
    assert_eq!(ArmadilloStateModel::Rolling.animation_duration(), 10);
    assert_eq!(ArmadilloStateModel::Scared.animation_duration(), 50);
    assert!(!ArmadilloStateModel::Idle.is_threatened());
    assert!(ArmadilloStateModel::Rolling.is_threatened());
    assert!(!ArmadilloStateModel::Rolling.should_hide_in_shell(5));
    assert!(ArmadilloStateModel::Rolling.should_hide_in_shell(6));
    assert!(ArmadilloStateModel::Scared.should_hide_in_shell(0));
    assert!(ArmadilloStateModel::Unrolling.should_hide_in_shell(25));
    assert!(!ArmadilloStateModel::Unrolling.should_hide_in_shell(26));
}

fn assert_armadillo_food_spawn_and_scute_drop_rules() {
    assert!(armadillo_food_item("minecraft:spider_eye"));
    assert!(!armadillo_food_item("minecraft:carrot"));
    assert!(armadillo_spawnable_on("minecraft:grass_block"));
    assert!(armadillo_spawnable_on("minecraft:red_sand"));
    assert!(armadillo_spawnable_on("minecraft:orange_terracotta"));
    assert!(!armadillo_spawnable_on("minecraft:stone"));
    assert!(armadillo_spawn_allowed("minecraft:coarse_dirt", true));
    assert!(!armadillo_spawn_allowed("minecraft:coarse_dirt", false));
    assert_eq!(armadillo_pick_next_scute_drop_time(0), 6000);
    assert_eq!(armadillo_pick_next_scute_drop_time(5999), 11999);
    assert_eq!(
        armadillo_scute_drop_tick(true, 1, true, true, 7),
        ArmadilloScuteDropPlan {
            next_scute_time: 6007,
            loot_table: Some("minecraft:gameplay/armadillo_shed"),
            sound: Some("minecraft:entity.armadillo.scute_drop"),
            game_event: Some("minecraft:entity_place"),
        }
    );
    assert_eq!(
        armadillo_scute_drop_tick(true, 2, true, true, 7).next_scute_time,
        1
    );
}

fn assert_armadillo_scare_roll_and_hurt_rules() {
    assert!(armadillo_is_scared_by(
        true, true, false, false, false, false, false
    ));
    assert!(armadillo_is_scared_by(
        true, false, true, false, false, false, false
    ));
    assert!(armadillo_is_scared_by(
        true, false, false, true, false, true, false
    ));
    assert!(armadillo_is_scared_by(
        true, false, false, true, false, false, true
    ));
    assert!(!armadillo_is_scared_by(
        true, false, false, true, true, true, true
    ));
    assert!(!armadillo_is_scared_by(
        false, true, true, true, false, true, true
    ));
    assert_eq!(
        armadillo_roll_up_plan(ArmadilloStateModel::Idle),
        Some(ArmadilloRollPlan {
            state: ArmadilloStateModel::Rolling,
            sound: "minecraft:entity.armadillo.roll",
            game_event: "minecraft:entity_action",
            stop_in_place: true,
            reset_love: true,
        })
    );
    assert_eq!(armadillo_roll_up_plan(ArmadilloStateModel::Scared), None);
    assert_eq!(
        armadillo_roll_out_plan(ArmadilloStateModel::Scared)
            .unwrap()
            .state,
        ArmadilloStateModel::Idle
    );
    assert!(armadillo_can_stay_rolled_up(
        false, false, false, false, false
    ));
    assert!(!armadillo_can_stay_rolled_up(
        true, false, false, false, false
    ));
    assert_eq!(
        armadillo_damage_after_shell(ArmadilloStateModel::Scared, 9.0),
        4.0
    );
    assert_eq!(
        armadillo_hurt_reaction(false, false, true, false, true),
        ArmadilloHurtReaction::DangerMemory {
            ticks: 80,
            roll_up: true,
        }
    );
    assert_eq!(
        armadillo_hurt_reaction(false, false, false, true, true),
        ArmadilloHurtReaction::RollOutEnvironmental
    );
    assert_eq!(
        armadillo_hurt_reaction(true, false, true, true, true),
        ArmadilloHurtReaction::None
    );
}

fn assert_armadillo_interactions_sounds_and_animation_ticks() {
    assert_eq!(
        armadillo_interact_plan("minecraft:brush", false, false),
        ArmadilloInteractPlan::Brush {
            loot_table: "minecraft:gameplay/armadillo_brush",
            sound: "minecraft:entity.armadillo.brush",
            game_event: "minecraft:entity_interact",
            tool_damage: 16,
        }
    );
    assert_eq!(
        armadillo_interact_plan("minecraft:brush", true, false),
        ArmadilloInteractPlan::Delegate
    );
    assert_eq!(
        armadillo_interact_plan("minecraft:stick", false, true),
        ArmadilloInteractPlan::FailScared
    );
    assert!(armadillo_can_fall_in_love(true, false));
    assert!(!armadillo_can_fall_in_love(true, true));
    assert_eq!(
        armadillo_ambient_sound(false),
        Some("minecraft:entity.armadillo.ambient")
    );
    assert_eq!(armadillo_ambient_sound(true), None);
    assert_eq!(
        armadillo_hurt_sound(true),
        "minecraft:entity.armadillo.hurt_reduced"
    );
    assert_eq!(armadillo_max_head_y_rot(true), 0);
    assert_eq!(armadillo_max_head_y_rot(false), 32);

    assert_eq!(
        armadillo_ball_up_tick_plan(ArmadilloStateModel::Rolling, 11, true, 80, 1),
        ArmadilloBallUpTickPlan::SwitchToScared {
            sound: Some("minecraft:entity.armadillo.land"),
        }
    );
    assert_eq!(
        armadillo_ball_up_tick_plan(ArmadilloStateModel::Scared, 50, true, 80, 0),
        ArmadilloBallUpTickPlan::Peek {
            event: 64,
            next_peek_timer_min: 150,
            next_peek_timer_max: 450,
        }
    );
    assert_eq!(
        armadillo_ball_up_tick_plan(ArmadilloStateModel::Scared, 50, true, 29, 1),
        ArmadilloBallUpTickPlan::StartUnrolling {
            sound: "minecraft:entity.armadillo.unroll_start",
        }
    );
    assert_eq!(
        armadillo_ball_up_tick_plan(ArmadilloStateModel::Unrolling, 5, true, 31, 1),
        ArmadilloBallUpTickPlan::ReturnToScared
    );
}

#[test]
fn allay_item_pickup_noteblock_dancing_and_duplication_match_java() {
    assert_allay_constants_and_attributes();
    assert_allay_item_pickup_rules();
    assert_allay_interactions_and_duplication_cooldown();
    assert_allay_dancing_and_noteblock_memory();
    assert_allay_deposit_vibration_and_sound_rules();
}

fn assert_allay_constants_and_attributes() {
    assert_eq!(ALLAY_ITEM_PICKUP_REACH, (1, 1, 1));
    assert_eq!(ALLAY_LIFTING_ITEM_ANIMATION_DURATION, 5);
    assert_eq!(ALLAY_DANCING_LOOP_DURATION, 55.0);
    assert_eq!(ALLAY_SPINNING_ANIMATION_DURATION, 15.0);
    assert_eq!(ALLAY_DEFAULT_DUPLICATION_COOLDOWN, 0);
    assert_eq!(ALLAY_DUPLICATION_COOLDOWN_TICKS, 6000);
    assert_eq!(ALLAY_NUM_DUPLICATION_HEARTS, 3);
    assert_eq!(ALLAY_MAX_NOTEBLOCK_DISTANCE, 1024);
    assert_eq!(ALLAY_VIBRATION_LISTENER_RANGE, 16);
    assert_eq!(ALLAY_TIME_TO_FORGET_NOTEBLOCK, 600);
    assert_eq!(ALLAY_DISTANCE_TO_WANTED_ITEM, 32);
    assert_eq!(ALLAY_GIVE_ITEM_TIMEOUT_DURATION, 20);
    assert_eq!(ALLAY_LIKED_PLAYER_DISTANCE, 64.0);
    assert_eq!(ALLAY_DUPLICATION_EVENT, 18);
    assert_eq!(
        allay_attributes(),
        AllayAttributes {
            max_health: 20.0,
            flying_speed: 0.1,
            movement_speed: 0.1,
            attack_damage: 2.0,
        }
    );
}

fn assert_allay_item_pickup_rules() {
    assert!(allay_duplicate_item("minecraft:amethyst_shard"));
    assert!(!allay_duplicate_item("minecraft:diamond"));
    assert!(allay_can_pick_up_loot(false, true));
    assert!(!allay_can_pick_up_loot(true, true));
    assert!(!allay_can_pick_up_loot(false, false));
    assert!(allay_considers_item_equal(
        "minecraft:potion",
        "minecraft:potion",
        Some("minecraft:healing"),
        Some("minecraft:healing")
    ));
    assert!(!allay_considers_item_equal(
        "minecraft:potion",
        "minecraft:potion",
        Some("minecraft:healing"),
        Some("minecraft:swiftness")
    ));
    assert!(allay_wants_to_pick_up(
        "minecraft:diamond",
        "minecraft:diamond",
        None,
        None,
        true,
        true
    ));
    assert!(!allay_wants_to_pick_up(
        "minecraft:diamond",
        "minecraft:emerald",
        None,
        None,
        true,
        true
    ));
    assert!(!allay_wants_to_pick_up(
        "minecraft:diamond",
        "minecraft:diamond",
        None,
        None,
        false,
        true
    ));
}

fn assert_allay_interactions_and_duplication_cooldown() {
    assert_eq!(
        allay_interact_plan("minecraft:amethyst_shard", true, true, true, true, false),
        AllayInteractPlan::Duplicate {
            consumed: 1,
            parent_cooldown: 6000,
            child_cooldown: 6000,
            event: 18,
            hearts: 3,
            sound: "minecraft:block.amethyst_block.chime",
        }
    );
    assert_eq!(
        allay_interact_plan("minecraft:cookie", true, false, false, true, false),
        AllayInteractPlan::GiveItem {
            consumed: 1,
            held_count: 1,
            remember_liked_player: true,
            sound: "minecraft:entity.allay.item_given",
        }
    );
    assert_eq!(
        allay_interact_plan("", true, true, false, true, true),
        AllayInteractPlan::TakeItem {
            clear_liked_player: true,
            return_held_item: true,
            throw_inventory: true,
            sound: "minecraft:entity.allay.item_taken",
        }
    );
    assert_eq!(
        allay_interact_plan("minecraft:amethyst_shard", true, true, true, false, false),
        AllayInteractPlan::Delegate
    );
    assert_eq!(allay_duplication_cooldown_tick(2, false), (1, false));
    assert_eq!(allay_duplication_cooldown_tick(1, false), (0, true));
    assert_eq!(allay_duplication_cooldown_tick(2, true), (2, false));
}

fn assert_allay_dancing_and_noteblock_memory() {
    assert_eq!(
        allay_set_jukebox_playing(None, (1, 2, 3), true, false),
        (Some((1, 2, 3)), true)
    );
    assert_eq!(
        allay_set_jukebox_playing(Some((1, 2, 3)), (1, 2, 3), false, true),
        (None, false)
    );
    assert_eq!(
        allay_set_jukebox_playing(Some((1, 2, 3)), (4, 5, 6), false, true),
        (Some((1, 2, 3)), true)
    );
    assert!(allay_should_stop_dancing(None, 0.0, true, 32.0));
    assert!(allay_should_stop_dancing(Some((0, 0, 0)), 32.0, true, 32.0));
    assert!(allay_should_stop_dancing(Some((0, 0, 0)), 1.0, false, 32.0));
    assert!(!allay_should_stop_dancing(
        Some((0, 0, 0)),
        31.9,
        true,
        32.0
    ));
    assert!(allay_set_dancing_allowed(false, true, true, false));
    assert!(!allay_set_dancing_allowed(false, true, true, true));
    assert!(allay_set_dancing_allowed(false, true, false, true));
    assert!(allay_is_spinning(14.9));
    assert!(!allay_is_spinning(15.0));
    assert!(allay_is_spinning(55.0));

    assert_eq!(
        allay_hear_noteblock(None, (7, 8, 9)),
        AllayNoteBlockPlan {
            liked_noteblock: Some((7, 8, 9)),
            cooldown_ticks: Some(600),
        }
    );
    assert_eq!(
        allay_hear_noteblock(Some((7, 8, 9)), (7, 8, 9)).cooldown_ticks,
        Some(600)
    );
    assert_eq!(
        allay_hear_noteblock(Some((1, 2, 3)), (7, 8, 9)),
        AllayNoteBlockPlan {
            liked_noteblock: Some((1, 2, 3)),
            cooldown_ticks: None,
        }
    );
}

fn assert_allay_deposit_vibration_and_sound_rules() {
    assert_eq!(
        allay_deposit_target(Some((1, 2, 3)), true, true, true, true),
        AllayDepositTarget::NoteBlockAbove
    );
    assert_eq!(
        allay_deposit_target(Some((1, 2, 3)), true, false, true, true),
        AllayDepositTarget::LikedPlayer
    );
    assert_eq!(
        allay_deposit_target(None, false, false, false, true),
        AllayDepositTarget::LikedPlayer
    );
    assert_eq!(
        allay_deposit_target(None, false, false, false, false),
        AllayDepositTarget::None
    );
    assert!(allay_liked_player_available(true, true, 63.9));
    assert!(!allay_liked_player_available(true, true, 64.0));
    assert!(!allay_liked_player_available(true, false, 1.0));
    assert!(allay_can_receive_note_vibration(
        false,
        None,
        (1, 2, 3),
        false
    ));
    assert!(allay_can_receive_note_vibration(
        false,
        Some((1, 2, 3)),
        (1, 2, 3),
        true
    ));
    assert!(!allay_can_receive_note_vibration(
        false,
        Some((1, 2, 3)),
        (4, 5, 6),
        true
    ));
    assert!(!allay_can_receive_note_vibration(
        true,
        None,
        (1, 2, 3),
        true
    ));

    assert!(!allay_hurt_allowed(true));
    assert!(allay_hurt_allowed(false));
    assert_eq!(
        allay_ambient_sound(true),
        "minecraft:entity.allay.ambient_with_item"
    );
    assert_eq!(
        allay_ambient_sound(false),
        "minecraft:entity.allay.ambient_without_item"
    );
    assert!(!allay_remove_when_far_away());
    let leash_offset = allay_leash_offset(0.36, 0.35);
    assert_eq!(leash_offset.0, 0.0);
    assert!((leash_offset.1 - 0.216).abs() < f32::EPSILON * 2.0);
    assert_eq!(leash_offset.2, 0.035);
    assert!(allay_throw_sound_can_play(14, true));
    assert!(!allay_throw_sound_can_play(15, true));
    assert!(!allay_throw_sound_can_play(14, false));
}

#[test]
fn feline_cat_ocelot_variants_trust_gifts_and_sitting_match_java() {
    assert_feline_constants_and_attributes();
    assert_feline_variants_collars_and_food();
    assert_feline_pose_cat_and_ocelot_interactions();
    assert_feline_trust_tempt_mate_and_removal_rules();
    assert_cat_owner_relax_gift_and_block_sitting_rules();
    assert_cat_and_ocelot_spawn_rules();
}

fn assert_feline_constants_and_attributes() {
    assert_eq!(FELINE_CROUCH_SPEED_MOD, 0.6);
    assert_eq!(FELINE_WALK_SPEED_MOD, 0.8);
    assert_eq!(FELINE_SPRINT_SPEED_MOD, 1.33);
    assert_eq!(FELINE_PLAYER_AVOID_DISTANCE, 16.0);
    assert_eq!(FELINE_REMOVE_WHEN_FAR_TICKS, 2400);
    assert_eq!(CAT_AMBIENT_SOUND_INTERVAL, 120);
    assert_eq!(OCELOT_AMBIENT_SOUND_INTERVAL, 900);
    assert_eq!(CAT_OWNER_RELAX_DISTANCE_SQR, 100.0);
    assert_eq!(CAT_LIE_ON_OWNER_DISTANCE_SQR, 2.5);
    assert_eq!(CAT_ON_BED_RELAX_TICKS, 16);
    assert_eq!(CAT_BEG_SOUND_INTERVAL_TICKS, 100);
    assert_eq!(CAT_TEMPT_SELECT_INTERVAL_TICKS, 600);
    assert_eq!(CAT_TEMPT_FORGET_INTERVAL_TICKS, 500);
    assert_eq!(CAT_TAME_ROLL_BOUND, 3);
    assert_eq!(OCELOT_TRUST_ROLL_BOUND, 3);
    assert_eq!(CAT_STRAY_SPAWNER_TICK_DELAY, 1200);
    assert_eq!(CAT_VILLAGE_HOME_POI_RADIUS, 48);
    assert_eq!(CAT_VILLAGE_MIN_OCCUPIED_HOMES, 5);
    assert_eq!(CAT_VILLAGE_MAX_CATS, 5);
    assert_eq!(CAT_HUT_CAT_RADIUS, 16);
    assert_eq!(OCELOT_SPAWN_ROLL_BOUND, 3);

    assert_eq!(
        feline_attributes(),
        FelineAttributes {
            max_health: 10.0,
            movement_speed: 0.3,
            attack_damage: 3.0,
        }
    );
}

fn assert_feline_variants_collars_and_food() {
    assert_eq!(cat_default_variant(), CatVariantModel::Black);
    assert_eq!(CatVariantModel::AllBlack.serialized_name(), "all_black");
    assert_eq!(
        cat_spawn_variant(true, false, CatVariantModel::Tabby),
        CatVariantModel::AllBlack
    );
    assert_eq!(
        cat_spawn_variant(false, true, CatVariantModel::Calico),
        CatVariantModel::AllBlack
    );
    assert_eq!(
        cat_spawn_variant(false, false, CatVariantModel::Calico),
        CatVariantModel::Calico
    );
    assert_eq!(cat_default_collar_color_id(), 14);
    assert_eq!(cat_collar_dye_color_id("minecraft:blue_dye"), Some(11));
    assert_eq!(cat_collar_dye_color_id("minecraft:stick"), None);
    assert!(cat_food_item("minecraft:cod"));
    assert!(cat_food_item("minecraft:salmon"));
    assert!(!cat_food_item("minecraft:tropical_fish"));
    assert!(ocelot_food_item("minecraft:cod"));
    assert!(!ocelot_food_item("minecraft:chicken"));
}

fn assert_feline_pose_cat_and_ocelot_interactions() {
    assert_eq!(
        feline_pose_for_move(true, FELINE_CROUCH_SPEED_MOD),
        (FelineMovePose::Crouching, false)
    );
    assert_eq!(
        feline_pose_for_move(true, FELINE_SPRINT_SPEED_MOD),
        (FelineMovePose::Standing, true)
    );
    assert_eq!(
        feline_pose_for_move(false, FELINE_SPRINT_SPEED_MOD),
        (FelineMovePose::Standing, false)
    );

    assert_eq!(
        cat_interact_plan(
            "minecraft:blue_dye",
            true,
            true,
            cat_default_collar_color_id(),
            true,
            false,
            false,
        ),
        CatInteractPlan::DyeCollar {
            color_id: 11,
            consumed: 1,
            persist: true,
        }
    );
    assert_eq!(
        cat_interact_plan("minecraft:cod", true, true, 14, true, false, false),
        CatInteractPlan::Heal {
            consumed: 1,
            heal_min: 1,
            heal_max: 1,
        }
    );
    assert_eq!(
        cat_interact_plan("minecraft:stick", true, true, 14, false, false, false),
        CatInteractPlan::ToggleSit
    );
    assert_eq!(
        cat_interact_plan("minecraft:cod", false, false, 14, false, false, true),
        CatInteractPlan::TameFood {
            consumed: 1,
            tame_event: 7,
            tamed: true,
            ordered_to_sit: true,
            persist: true,
            eat_sound: "minecraft:entity.cat.eat",
        }
    );
    assert_eq!(
        cat_interact_plan("minecraft:cod", false, false, 14, false, false, false),
        CatInteractPlan::TameFood {
            consumed: 1,
            tame_event: 6,
            tamed: false,
            ordered_to_sit: false,
            persist: true,
            eat_sound: "minecraft:entity.cat.eat",
        }
    );

    assert_eq!(
        ocelot_interact_plan("minecraft:salmon", true, false, 8.99, true),
        OcelotInteractPlan::TrustFood {
            consumed: 1,
            event: 41,
            trusting: true,
        }
    );
    assert_eq!(
        ocelot_interact_plan("minecraft:salmon", true, false, 8.99, false),
        OcelotInteractPlan::TrustFood {
            consumed: 1,
            event: 40,
            trusting: false,
        }
    );
    assert_eq!(
        ocelot_interact_plan("minecraft:salmon", true, false, 9.0, true),
        OcelotInteractPlan::Delegate
    );
}

fn assert_feline_trust_tempt_mate_and_removal_rules() {
    assert!(feline_should_avoid_player(false, false));
    assert!(!feline_should_avoid_player(true, false));
    assert!(!feline_should_avoid_player(false, true));
    assert!(cat_tempt_can_use(true, false));
    assert!(!cat_tempt_can_use(true, true));
    assert!(!cat_tempt_can_scare(true, true));
    assert!(cat_tempt_can_scare(false, true));
    assert!(cat_should_play_beg_sound(true, false, 200));
    assert!(!cat_should_play_beg_sound(true, false, 201));
    assert!(cat_can_mate(true, true, true, true));
    assert!(!cat_can_mate(false, true, true, true));
    assert!(cat_remove_when_far_away(false, 2401));
    assert!(!cat_remove_when_far_away(true, 2401));
    assert!(ocelot_remove_when_far_away(false, 2401));
    assert!(!ocelot_remove_when_far_away(true, 2401));
}

fn assert_cat_owner_relax_gift_and_block_sitting_rules() {
    assert!(cat_relax_on_owner_can_use(
        true, false, true, true, 100.0, true, false,
    ));
    assert!(!cat_relax_on_owner_can_use(
        true, false, true, true, 100.1, true, false,
    ));
    assert_eq!(cat_relax_on_owner_tick(2.49, 17), (true, false));
    assert_eq!(cat_relax_on_owner_tick(2.49, 16), (false, true));
    assert_eq!(cat_relax_on_owner_tick(2.5, 17), (false, false));
    assert_eq!(
        cat_morning_gift_plan(100, true, false),
        Some(("minecraft:gameplay/cat_morning_gift", false))
    );
    assert_eq!(cat_morning_gift_plan(99, true, false), None);
    assert_eq!(cat_morning_gift_plan(100, false, false), None);
    assert!(cat_lie_on_bed_can_use(true, false, false, true));
    assert!(!cat_lie_on_bed_can_use(true, true, false, true));
    assert!(cat_sit_on_block_can_use(true, false, true));
    assert!(!cat_sit_on_block_can_use(false, false, true));
    assert!(cat_sit_on_block_target_valid(
        "minecraft:chest",
        true,
        0,
        false,
        false
    ));
    assert!(!cat_sit_on_block_target_valid(
        "minecraft:chest",
        true,
        1,
        false,
        false
    ));
    assert!(cat_sit_on_block_target_valid(
        "minecraft:furnace",
        true,
        0,
        true,
        false
    ));
    assert!(cat_sit_on_block_target_valid(
        "minecraft:red_bed",
        true,
        0,
        false,
        false
    ));
    assert!(!cat_sit_on_block_target_valid(
        "minecraft:red_bed",
        true,
        0,
        false,
        true
    ));
    assert!(cat_lie_on_bed_target_valid("minecraft:black_bed", true));
    assert!(!cat_lie_on_bed_target_valid("minecraft:chest", true));
}

fn assert_cat_and_ocelot_spawn_rules() {
    assert!(cat_spawner_should_try_spawn(0, true));
    assert!(!cat_spawner_should_try_spawn(1, true));
    assert_eq!(cat_spawner_offset(23, true), -31);
    assert_eq!(cat_spawner_offset(0, false), 8);
    assert!(cat_village_spawn_allowed(5, 4));
    assert!(!cat_village_spawn_allowed(4, 4));
    assert!(!cat_village_spawn_allowed(5, 5));
    assert!(cat_hut_spawn_allowed(0));
    assert!(!cat_hut_spawn_allowed(1));
    assert!(!ocelot_spawn_rules(0));
    assert!(ocelot_spawn_rules(1));
    assert!(ocelot_spawn_obstruction(
        true,
        false,
        63,
        63,
        "minecraft:grass_block"
    ));
    assert!(ocelot_spawn_obstruction(
        true,
        false,
        64,
        63,
        "minecraft:oak_leaves"
    ));
    assert!(!ocelot_spawn_obstruction(
        true,
        false,
        62,
        63,
        "minecraft:grass_block"
    ));
    assert_eq!(ocelot_leash_offset(0.7, 0.6), (0.0, 0.35, 0.24000001));
}

#[test]
fn tamable_flags_owner_sitting_and_teleport_distance_match_vanilla() {
    let state = TamableState {
        flags: 0,
        owner_present: true,
        ordered_to_sit: false,
    }
    .set_tame(true)
    .set_sitting_pose(true);
    assert!(state.is_tame());
    assert!(state.is_sitting());
    assert!(state.ordered_to_sit);
    assert!(should_tamable_teleport_to_owner(true, 144));
    assert!(!should_tamable_teleport_to_owner(true, 143));
    assert_eq!(TAMABLE_TELEPORT_ATTEMPTS, 10);
    assert_eq!(TAMABLE_TELEPORT_MIN_HORIZONTAL, 2);
    assert_eq!(TAMABLE_TELEPORT_MAX_HORIZONTAL, 3);
    assert_eq!(TAMABLE_TELEPORT_MAX_VERTICAL, 1);
}

#[test]
fn bucketable_shearable_and_mooshroom_transformations_follow_interaction_gates() {
    assert_eq!(
        bucket_pickup_result("minecraft:water_bucket", true),
        BucketPickupResult::FilledBucketAndDiscardEntity
    );
    assert_eq!(
        save_default_bucket_data(true, true, false, true, false),
        BucketEntityData {
            no_ai: true,
            silent: true,
            no_gravity: false,
            glowing: true,
            invulnerable: false,
            health_saved: true,
        }
    );
    assert!(ready_for_shearing(true, false, false));
    assert!(!ready_for_shearing(true, true, false));
    assert_eq!(
        mooshroom_interaction("minecraft:bowl", false, false, false, false),
        MooshroomInteraction::FillStew
    );
    assert_eq!(
        mooshroom_interaction("minecraft:bowl", false, true, true, false),
        MooshroomInteraction::FillSuspiciousStew
    );
    assert_eq!(
        mooshroom_interaction("minecraft:shears", false, false, false, false),
        MooshroomInteraction::ShearIntoCow
    );
    assert_eq!(
        mooshroom_interaction("minecraft:poppy", false, true, false, true),
        MooshroomInteraction::StoreSuspiciousEffects
    );
}

#[test]
fn horse_riding_saddle_temper_and_villager_trading_slots_match_vanilla() {
    assert_horse_saddle_temper_and_constants();
    assert_horse_variant_and_offspring_rules();
    assert_chested_horse_inventory_and_llama_constants();
    assert_llama_spawn_offspring_inventory_and_villager_slots();
}

fn assert_horse_saddle_temper_and_constants() {
    let horse = HorseState {
        flags: HORSE_FLAG_TAME,
        temper: 5,
        max_temper: 100,
        baby: false,
        alive: true,
    };
    assert!(horse.can_use_saddle_slot());
    assert_eq!(horse.modify_temper(200).temper, 100);
    assert_eq!(HORSE_CHEST_SLOT_OFFSET, 499);
    assert_eq!(HORSE_INVENTORY_SLOT_OFFSET, 500);
    assert_eq!(HORSE_BREEDING_CROSS_FACTOR, 0.15);
    assert_eq!(HORSE_INVENTORY_ROWS, 3);
    assert_eq!(HORSE_FLAG_BRED, 8);
    assert_eq!(HORSE_FLAG_EATING, 16);
    assert_eq!(HORSE_FLAG_STANDING, 32);
    assert_eq!(HORSE_FLAG_OPEN_MOUTH, 64);
    assert_eq!(HORSE_BABY_SCALE, 0.7);
    assert_eq!(CHESTED_HORSE_BABY_SCALE, 0.5);
    assert_eq!(CHESTED_HORSE_MOVEMENT_SPEED, 0.175);
    assert_eq!(CHESTED_HORSE_JUMP_STRENGTH, 0.5);
}

fn assert_horse_variant_and_offspring_rules() {
    assert_eq!(
        horse_type_variant(HorseVariant::DarkBrown, HorseMarkings::BlackDots),
        1030
    );
    assert_eq!(horse_variant_from_type(1030), HorseVariant::DarkBrown);
    assert_eq!(horse_markings_from_type(1030), HorseMarkings::BlackDots);
    assert_eq!(HorseVariant::by_id(7), HorseVariant::White);
    assert_eq!(HorseMarkings::by_id(5), HorseMarkings::None);
    assert_eq!(
        horse_offspring_variant(HorseVariant::White, HorseVariant::Black, 3, 6),
        HorseVariant::White
    );
    assert_eq!(
        horse_offspring_variant(HorseVariant::White, HorseVariant::Black, 7, 6),
        HorseVariant::Black
    );
    assert_eq!(
        horse_offspring_variant(HorseVariant::White, HorseVariant::Black, 8, 6),
        HorseVariant::DarkBrown
    );
    assert_eq!(
        horse_offspring_markings(HorseMarkings::White, HorseMarkings::BlackDots, 1, 2),
        HorseMarkings::White
    );
    assert_eq!(
        horse_offspring_markings(HorseMarkings::White, HorseMarkings::BlackDots, 3, 2),
        HorseMarkings::BlackDots
    );
    assert_eq!(
        horse_offspring_markings(HorseMarkings::White, HorseMarkings::BlackDots, 4, 2),
        HorseMarkings::WhiteField
    );
}

fn assert_chested_horse_inventory_and_llama_constants() {
    assert_eq!(chested_horse_inventory_columns(false), 0);
    assert_eq!(chested_horse_inventory_columns(true), 5);
    assert!(chested_horse_can_equip_chest(
        false,
        true,
        false,
        "minecraft:chest"
    ));
    assert!(!chested_horse_can_equip_chest(
        true,
        true,
        false,
        "minecraft:chest"
    ));
    assert_eq!(LLAMA_MAX_STRENGTH, 5);
    assert_eq!(LLAMA_COMMON_MAX_STRENGTH, 3);
    assert_eq!(LLAMA_RARE_MAX_STRENGTH_CHANCE, 0.04);
    assert_eq!(LLAMA_BREED_STRENGTH_BONUS_CHANCE, 0.03);
    assert_eq!(LLAMA_MAX_TEMPER, 30);
    assert_eq!(LLAMA_RANGED_ATTACK_INTERVAL_TICKS, 40);
    assert_eq!(LLAMA_RANGED_ATTACK_RADIUS, 20.0);
    assert_eq!(LLAMA_SPIT_SPEED, 1.5);
    assert_eq!(LLAMA_SPIT_INACCURACY, 10.0);
    assert_eq!(LlamaVariant::by_id(-1), LlamaVariant::Creamy);
    assert_eq!(LlamaVariant::by_id(9), LlamaVariant::Gray);
    assert_eq!(LlamaVariant::Brown.id(), 2);
}

fn assert_llama_spawn_offspring_inventory_and_villager_slots() {
    assert_eq!(llama_strength_from_spawn(false, 2), 3);
    assert_eq!(llama_strength_from_spawn(true, 4), 5);
    assert_eq!(llama_inventory_columns(false, 5), 0);
    assert_eq!(llama_inventory_columns(true, 5), 5);
    assert_eq!(llama_inventory_columns(true, 9), 5);
    assert_eq!(llama_offspring_strength(2, 4, 3, false), 4);
    assert_eq!(llama_offspring_strength(2, 4, 3, true), 5);
    assert_eq!(
        llama_offspring_variant(LlamaVariant::White, LlamaVariant::Gray, true),
        LlamaVariant::White
    );
    assert_eq!(
        llama_offspring_variant(LlamaVariant::White, LlamaVariant::Gray, false),
        LlamaVariant::Gray
    );
    assert_eq!(villager_slot_index(300), Some(0));
    assert_eq!(villager_slot_index(307), Some(7));
    assert_eq!(villager_slot_index(308), None);
    assert_eq!(VILLAGER_INVENTORY_SIZE, 8);
}
