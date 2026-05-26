#[test]
fn parrot_variants_taming_poison_party_shoulder_and_mimic_match_java() {
    assert_parrot_constants_variants_and_food();
    assert_parrot_interactions();
    assert_parrot_party_and_mimic_rules();
    assert_parrot_shoulder_mounting_rules();
}

fn assert_parrot_constants_variants_and_food() {
    assert_eq!(PARROT_MAX_HEALTH, 6.0);
    assert_eq!(PARROT_FLYING_SPEED, 0.4);
    assert_eq!(PARROT_MOVEMENT_SPEED, 0.2);
    assert_eq!(PARROT_ATTACK_DAMAGE, 3.0);
    assert_eq!(PARROT_TAME_ROLL_BOUND, 10);
    assert_eq!(PARROT_POISON_TICKS, 900);
    assert_eq!(PARROT_JUKEBOX_PARTY_DISTANCE, 3.46);
    assert_eq!(PARROT_MIMIC_SCAN_RADIUS, 20.0);
    assert_eq!(PARROT_MIMIC_TICK_ROLL_BOUND, 400);
    assert_eq!(PARROT_MIMIC_SOUND_ROLL_BOUND, 2);
    assert_eq!(SHOULDER_RIDING_COOLDOWN_TICKS, 100);

    assert_eq!(ParrotVariantModel::RedBlue.id(), 0);
    assert_eq!(ParrotVariantModel::Gray.id(), 4);
    assert_eq!(
        ParrotVariantModel::YellowBlue.serialized_name(),
        "yellow_blue"
    );
    assert_eq!(parrot_variant_by_id(-5), ParrotVariantModel::RedBlue);
    assert_eq!(parrot_variant_by_id(99), ParrotVariantModel::Gray);

    assert!(parrot_food_item("minecraft:wheat_seeds"));
    assert!(parrot_food_item("minecraft:melon_seeds"));
    assert!(parrot_food_item("minecraft:pumpkin_seeds"));
    assert!(parrot_food_item("minecraft:beetroot_seeds"));
    assert!(parrot_food_item("minecraft:torchflower_seeds"));
    assert!(parrot_food_item("minecraft:pitcher_pod"));
    assert!(!parrot_food_item("minecraft:cookie"));
    assert!(parrot_poisonous_item("minecraft:cookie"));
}

fn assert_parrot_interactions() {
    assert_eq!(
        parrot_interact_plan("minecraft:wheat_seeds", false, false, false, true, false),
        ParrotInteractResult::TameFood {
            consumed: 1,
            eat_sound: "minecraft:entity.parrot.eat",
            tame_event: 7,
            tamed: true,
        }
    );
    assert_eq!(
        parrot_interact_plan("minecraft:wheat_seeds", false, false, false, false, false),
        ParrotInteractResult::TameFood {
            consumed: 1,
            eat_sound: "minecraft:entity.parrot.eat",
            tame_event: 6,
            tamed: false,
        }
    );
    assert_eq!(
        parrot_interact_plan("minecraft:stick", true, false, true, false, false),
        ParrotInteractResult::ToggleSitting
    );
    assert_eq!(
        parrot_interact_plan("minecraft:stick", true, true, true, false, false),
        ParrotInteractResult::Pass
    );
    assert_eq!(
        parrot_interact_plan("minecraft:cookie", true, false, true, false, false),
        ParrotInteractResult::Poisoned {
            consumed: 1,
            effect: "minecraft:poison",
            duration_ticks: 900,
            lethal_damage: true,
        }
    );
    assert_eq!(
        parrot_interact_plan("minecraft:cookie", true, false, true, false, true),
        ParrotInteractResult::Poisoned {
            consumed: 1,
            effect: "minecraft:poison",
            duration_ticks: 900,
            lethal_damage: false,
        }
    );
}

fn assert_parrot_party_and_mimic_rules() {
    assert!(parrot_party_state_after_ai_step(true, true, 3.45, true));
    assert!(!parrot_party_state_after_ai_step(true, true, 3.46, true));
    assert!(!parrot_party_state_after_ai_step(true, false, 1.0, true));
    assert!(!parrot_party_state_after_ai_step(false, true, 1.0, true));
    assert!(parrot_should_attempt_mimic(true, false, 0));
    assert!(!parrot_should_attempt_mimic(true, false, 1));
    assert!(!parrot_should_attempt_mimic(true, true, 0));

    assert_eq!(
        parrot_mimic_sound("minecraft:cave_spider"),
        Some("minecraft:entity.parrot.imitate.spider")
    );
    assert_eq!(
        parrot_mimic_sound("minecraft:happy_ghast"),
        Some("minecraft:empty")
    );
    assert_eq!(parrot_mimic_sound("minecraft:parrot"), None);
    assert_eq!(
        parrot_mimic_nearby_plan(true, false, true, Some("minecraft:zombie_nautilus"), false),
        Some("minecraft:entity.parrot.imitate.zombie_nautilus")
    );
    assert_eq!(
        parrot_mimic_nearby_plan(true, false, true, Some("minecraft:zombie"), true),
        None
    );
    assert_eq!(
        parrot_mimic_nearby_plan(true, false, false, Some("minecraft:zombie"), false),
        None
    );
}

fn assert_parrot_shoulder_mounting_rules() {
    assert_eq!(
        parrot_shoulder_plan(ParrotShoulderInput {
            has_server_player_owner: true,
            ordered_to_sit: false,
            owner_spectator: false,
            owner_flying: false,
            owner_in_water: false,
            owner_in_powder_snow: false,
            ride_cooldown_counter: 101,
            in_sitting_pose: false,
            leashed: false,
            bounding_boxes_intersect: true,
            owner_is_passenger: false,
            owner_on_ground: true,
        }),
        ParrotShoulderPlan {
            can_use_goal: true,
            can_mount_now: true,
        }
    );
    assert_eq!(
        parrot_shoulder_plan(ParrotShoulderInput {
            has_server_player_owner: true,
            ordered_to_sit: false,
            owner_spectator: false,
            owner_flying: false,
            owner_in_water: false,
            owner_in_powder_snow: false,
            ride_cooldown_counter: 100,
            in_sitting_pose: false,
            leashed: false,
            bounding_boxes_intersect: true,
            owner_is_passenger: false,
            owner_on_ground: true,
        }),
        ParrotShoulderPlan {
            can_use_goal: false,
            can_mount_now: false,
        }
    );
    assert_eq!(
        parrot_shoulder_plan(ParrotShoulderInput {
            has_server_player_owner: true,
            ordered_to_sit: true,
            owner_spectator: false,
            owner_flying: false,
            owner_in_water: false,
            owner_in_powder_snow: false,
            ride_cooldown_counter: 101,
            in_sitting_pose: false,
            leashed: false,
            bounding_boxes_intersect: true,
            owner_is_passenger: false,
            owner_on_ground: true,
        }),
        ParrotShoulderPlan {
            can_use_goal: false,
            can_mount_now: false,
        }
    );
    assert!(
        !parrot_shoulder_plan(ParrotShoulderInput {
            has_server_player_owner: true,
            ordered_to_sit: false,
            owner_spectator: false,
            owner_flying: false,
            owner_in_water: false,
            owner_in_powder_snow: false,
            ride_cooldown_counter: 101,
            in_sitting_pose: false,
            leashed: false,
            bounding_boxes_intersect: true,
            owner_is_passenger: true,
            owner_on_ground: true,
        })
        .can_mount_now
    );
    assert!(
        !parrot_shoulder_plan(ParrotShoulderInput {
            has_server_player_owner: true,
            ordered_to_sit: false,
            owner_spectator: false,
            owner_flying: false,
            owner_in_water: false,
            owner_in_powder_snow: false,
            ride_cooldown_counter: 101,
            in_sitting_pose: false,
            leashed: false,
            bounding_boxes_intersect: true,
            owner_is_passenger: false,
            owner_on_ground: false,
        })
        .can_mount_now
    );
}

#[test]
fn happy_ghast_harness_riding_leash_healing_and_dried_block_match_java() {
    assert_happy_ghast_constants_attributes_and_items();
    assert_happy_ghast_interactions_riding_and_healing();
    assert_happy_ghast_still_timeout_collision_and_leash();
    assert_dried_ghast_hydration_ticks();
}

fn assert_happy_ghast_constants_attributes_and_items() {
    assert_eq!(HAPPY_GHAST_BABY_SCALE, 0.2375);
    assert_eq!(HAPPY_GHAST_WANDER_GROUND_DISTANCE, 16);
    assert_eq!(HAPPY_GHAST_SMALL_RESTRICTION_RADIUS, 32);
    assert_eq!(HAPPY_GHAST_LARGE_RESTRICTION_RADIUS, 64);
    assert_eq!(HAPPY_GHAST_RESTRICTION_RADIUS_BUFFER, 16);
    assert_eq!(HAPPY_GHAST_FAST_HEALING_TICKS, 20);
    assert_eq!(HAPPY_GHAST_SLOW_HEALING_TICKS, 600);
    assert_eq!(HAPPY_GHAST_MAX_PASSENGERS, 4);
    assert_eq!(HAPPY_GHAST_STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD, 60);
    assert_eq!(HAPPY_GHAST_MAX_STILL_TIMEOUT, 10);
    assert_eq!(HAPPY_GHAST_SPEED_MULTIPLIER_WHEN_PANICKING, 2.0);
    assert_eq!(
        happy_ghast_attributes(),
        HappyGhastAttributes {
            max_health: 20.0,
            tempt_range: 16.0,
            flying_speed: 0.05,
            movement_speed: 0.05,
            follow_range: 16.0,
            camera_distance: 8.0,
        }
    );

    assert!(happy_ghast_food_item("minecraft:snowball"));
    assert!(happy_ghast_harness_item("minecraft:white_harness"));
    assert!(happy_ghast_harness_item("minecraft:black_harness"));
    assert!(!happy_ghast_harness_item("minecraft:saddle"));
    assert!(happy_ghast_tempt_item(
        "minecraft:red_harness",
        false,
        false
    ));
    assert!(!happy_ghast_tempt_item(
        "minecraft:red_harness",
        false,
        true
    ));
    assert!(!happy_ghast_tempt_item(
        "minecraft:red_harness",
        true,
        false
    ));
    assert!(happy_ghast_tempt_item("minecraft:snowball", true, true));
    assert!(happy_ghast_can_use_body_slot(true, false));
    assert!(!happy_ghast_can_use_body_slot(true, true));
    assert!(!happy_ghast_can_use_body_slot(false, false));
}

fn assert_happy_ghast_interactions_riding_and_healing() {
    assert_eq!(
        happy_ghast_interact_plan(false, "minecraft:blue_harness", false, false, true),
        HappyGhastInteractPlan::EquipHarness
    );
    assert_eq!(
        happy_ghast_interact_plan(false, "", true, false, false),
        HappyGhastInteractPlan::StartRide
    );
    assert_eq!(
        happy_ghast_interact_plan(false, "", true, true, false),
        HappyGhastInteractPlan::Pass
    );
    assert_eq!(
        happy_ghast_interact_plan(true, "", true, false, false),
        HappyGhastInteractPlan::Pass
    );

    assert!(happy_ghast_can_add_passenger(3));
    assert!(!happy_ghast_can_add_passenger(4));
    assert!(happy_ghast_controlling_passenger(true, false, true));
    assert!(!happy_ghast_controlling_passenger(true, true, true));
    assert!(!happy_ghast_controlling_passenger(false, false, true));
    assert_eq!(happy_ghast_restriction_radius(false, false), 64);
    assert_eq!(happy_ghast_restriction_radius(false, true), 32);
    assert_eq!(happy_ghast_restriction_radius(true, false), 32);
    assert_eq!(happy_ghast_heal_interval_ticks(true), 20);
    assert_eq!(happy_ghast_heal_interval_ticks(false), 600);
}

fn assert_happy_ghast_still_timeout_collision_and_leash() {
    assert_eq!(
        happy_ghast_still_timeout_tick(10, 60, false),
        HappyGhastStillTimeoutStep {
            timeout: 10,
            stays_still: true,
        }
    );
    assert_eq!(
        happy_ghast_still_timeout_tick(10, 61, false),
        HappyGhastStillTimeoutStep {
            timeout: 9,
            stays_still: true,
        }
    );
    assert_eq!(
        happy_ghast_still_timeout_tick(0, 61, true),
        HappyGhastStillTimeoutStep {
            timeout: 10,
            stays_still: true,
        }
    );
    assert_eq!(happy_ghast_still_timeout_after_add_passenger(20, true), 10);
    assert_eq!(happy_ghast_still_timeout_after_add_passenger(8, true), 8);
    assert_eq!(happy_ghast_still_timeout_after_add_passenger(8, false), 0);
    assert_eq!(happy_ghast_still_timeout_after_remove_passenger(), 10);

    assert!(happy_ghast_can_be_collided_with(
        false, true, true, true, false, false, false
    ));
    assert!(happy_ghast_can_be_collided_with(
        false, true, false, false, true, true, false
    ));
    assert!(happy_ghast_can_be_collided_with(
        false, true, false, false, false, false, true
    ));
    assert!(!happy_ghast_can_be_collided_with(
        true, true, true, true, true, true, true
    ));
    assert_eq!(
        happy_ghast_leash_holder_offsets(),
        [
            (-0.03125, 0.4375, 0.46875),
            (0.03125, 0.4375, 0.46875),
            (-0.03125, 0.4375, -0.46875),
            (0.03125, 0.4375, -0.46875),
        ]
    );
    assert_eq!(happy_ghast_notify_leash_holder_time(true), 5);
    assert_eq!(happy_ghast_notify_leash_holder_time(false), 0);
    assert_eq!(HAPPY_GHAST_LEASH_ELASTIC_DISTANCE, 10.0);
    assert_eq!(HAPPY_GHAST_LEASH_SNAP_DISTANCE, 16.0);
}

fn assert_dried_ghast_hydration_ticks() {
    assert_eq!(
        dried_ghast_tick_plan(true, 2),
        DriedGhastTickPlan::Hydrate {
            hydration_level: 3,
            sound: "minecraft:block.dried_ghast.transition",
            game_event: "minecraft:block_change",
        }
    );
    assert_eq!(
        dried_ghast_tick_plan(true, 3),
        DriedGhastTickPlan::SpawnGhastling {
            remove_block: true,
            baby: true,
            sound: "minecraft:entity.ghastling.spawn",
        }
    );
    assert_eq!(
        dried_ghast_tick_plan(false, 2),
        DriedGhastTickPlan::Dehydrate {
            hydration_level: 1,
            game_event: "minecraft:block_change",
        }
    );
}

#[test]
fn sniffer_states_digging_seed_drop_and_egg_hatching_match_java() {
    assert_sniffer_constants_attributes_and_state_ids();
    assert_sniffer_food_blocks_and_sniff_gates();
    assert_sniffer_dig_body_state_rules();
    assert_sniffer_transition_and_ambient_rules();
    assert_sniffer_digging_tick_rules();
    assert_sniffer_memory_mating_and_egg_rules();
}

fn assert_sniffer_constants_attributes_and_state_ids() {
    assert_eq!(SNIFFER_DIGGING_PARTICLES_DELAY_TICKS, 1700);
    assert_eq!(SNIFFER_DIGGING_PARTICLES_DURATION_TICKS, 6000);
    assert_eq!(SNIFFER_DIGGING_PARTICLES_AMOUNT, 30);
    assert_eq!(SNIFFER_DIGGING_DROP_SEED_OFFSET_TICKS, 120);
    assert_eq!(SNIFFER_BABY_START_AGE, -48000);
    assert_eq!(SNIFFER_DIGGING_BB_HEIGHT_OFFSET, 0.4);
    assert_eq!(SNIFFER_EXPLORED_POSITION_LIMIT, 20);
    assert_eq!(SNIFFER_SNIFF_COOLDOWN_TICKS, 9600);
    assert_eq!(SNIFFER_DIGGING_MIN_TICKS, 160);
    assert_eq!(SNIFFER_DIGGING_MAX_TICKS, 180);
    assert_eq!(SNIFFER_FINISHED_DIGGING_TICKS, 40);
    assert_eq!(SNIFFER_SEARCHING_TICKS, 600);
    assert_eq!(SNIFFER_EGG_MAX_HATCH_LEVEL, 2);
    assert_eq!(SNIFFER_EGG_REGULAR_HATCH_TIME_TICKS, 24000);
    assert_eq!(SNIFFER_EGG_BOOSTED_HATCH_TIME_TICKS, 12000);
    assert_eq!(SNIFFER_EGG_RANDOM_HATCH_OFFSET_TICKS, 300);

    assert_eq!(
        sniffer_attributes(),
        SnifferAttributes {
            movement_speed: 0.1,
            max_health: 14.0,
        }
    );
    assert_eq!(SnifferStateModel::Idling.id(), 0);
    assert_eq!(SnifferStateModel::Rising.id(), 6);
    assert_eq!(sniffer_state_by_id(99), SnifferStateModel::Idling);
}

fn assert_sniffer_food_blocks_and_sniff_gates() {
    assert!(sniffer_food_item("minecraft:torchflower_seeds"));
    assert!(!sniffer_food_item("minecraft:wheat_seeds"));
    assert!(sniffer_diggable_block("minecraft:grass_block"));
    assert!(sniffer_diggable_block("minecraft:mud"));
    assert!(sniffer_diggable_block("minecraft:moss_block"));
    assert!(!sniffer_diggable_block("minecraft:stone"));
    assert!(sniffer_egg_hatch_boost_block("minecraft:moss_block"));

    assert!(sniffer_can_sniff(
        false, false, false, false, true, false, false
    ));
    assert!(!sniffer_can_sniff(
        true, false, false, false, true, false, false
    ));
    assert!(!sniffer_can_sniff(
        false, false, true, false, true, false, false
    ));
}

fn assert_sniffer_dig_body_state_rules() {
    let can_dig = SnifferDigBodyStateInput {
        panicking: false,
        tempted: false,
        baby: false,
        in_water: false,
        on_ground: true,
        passenger: false,
        head_block_below_diggable: true,
        explored_position: false,
        path_can_reach: true,
    };
    assert!(sniffer_can_dig_body_state(can_dig));
    assert!(!sniffer_can_dig_body_state(SnifferDigBodyStateInput {
        baby: true,
        ..can_dig
    }));
    assert!(!sniffer_can_dig_body_state(SnifferDigBodyStateInput {
        explored_position: true,
        ..can_dig
    }));
    assert!(!sniffer_can_dig_body_state(SnifferDigBodyStateInput {
        path_can_reach: false,
        ..can_dig
    }));
}

fn assert_sniffer_transition_and_ambient_rules() {
    assert_eq!(
        sniffer_transition_plan(SnifferStateModel::Digging, 1000, false),
        SnifferTransitionPlan {
            state: SnifferStateModel::Digging,
            sound: None,
            drop_seed_at_tick: Some(1120),
            event: Some(63),
        }
    );
    assert_eq!(
        sniffer_transition_plan(SnifferStateModel::Scenting, 0, true).sound,
        Some("minecraft:entity.sniffer.scenting@1.3")
    );
    assert_eq!(
        sniffer_transition_plan(SnifferStateModel::Rising, 0, false).sound,
        Some("minecraft:entity.sniffer.digging_stop")
    );
    assert!(sniffer_can_play_digging_sound(SnifferStateModel::Searching));
    assert_eq!(sniffer_ambient_sound(SnifferStateModel::Digging), None);
    assert_eq!(
        sniffer_ambient_sound(SnifferStateModel::Idling),
        Some("minecraft:entity.sniffer.idle")
    );
}

fn assert_sniffer_digging_tick_rules() {
    assert_eq!(
        sniffer_digging_tick_plan(SnifferStateModel::Digging, 1120, 1120, 1800, true),
        SnifferDiggingTickPlan {
            drop_seed: true,
            seed_loot_table: Some("minecraft:gameplay/sniffer_digging"),
            seed_sound: Some("minecraft:entity.sniffer.drop_seed"),
            particles: 30,
            block_hit_sound: true,
            game_event: true,
        }
    );
    assert_eq!(
        sniffer_digging_tick_plan(SnifferStateModel::Digging, 1121, 1120, 6000, true).particles,
        0
    );
    assert!(
        !sniffer_digging_tick_plan(SnifferStateModel::Searching, 1120, 1120, 1800, true).drop_seed
    );
}

fn assert_sniffer_memory_mating_and_egg_rules() {
    let explored = sniffer_store_explored_position(&(0..25).collect::<Vec<i32>>(), 99);
    assert_eq!(explored[0], 99);
    assert_eq!(explored.len(), 21);
    assert_eq!(*explored.last().unwrap(), 19);
    assert!(sniffer_can_mate_state(
        SnifferStateModel::Idling,
        SnifferStateModel::Scenting
    ));
    assert!(!sniffer_can_mate_state(
        SnifferStateModel::Digging,
        SnifferStateModel::Idling
    ));
    assert_eq!(
        sniffer_breeding_drop_plan(),
        ("minecraft:sniffer_egg", "minecraft:block.sniffer_egg.plop")
    );

    assert_eq!(sniffer_egg_next_tick_delay(false, 299), 8299);
    assert_eq!(sniffer_egg_next_tick_delay(true, 299), 4299);
    assert_eq!(
        sniffer_egg_tick_plan(1),
        SnifferEggTickPlan::Crack {
            hatch_level: 2,
            sound: "minecraft:block.sniffer_egg.crack",
        }
    );
    assert_eq!(
        sniffer_egg_tick_plan(2),
        SnifferEggTickPlan::Hatch {
            destroy_block: true,
            spawn_baby: true,
            sound: "minecraft:block.sniffer_egg.hatch",
        }
    );
}
