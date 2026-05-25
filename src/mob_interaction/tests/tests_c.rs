use crate::mob_interaction::*;

#[test]
fn anger_and_conversion_preserve_server_visible_state() {
    let angry = AngerState {
        anger_end_time: NO_ANGER_END_TIME,
        target_present: true,
    }
    .set_time_to_remain_angry(100, 40);
    assert!(angry.is_angry(120));
    assert!(!angry.is_angry(140));
    assert!(should_stop_anger_for_player(false, false, true));
    assert_eq!(angry.stop_being_angry().anger_end_time, NO_ANGER_END_TIME);

    assert_eq!(
        conversion_copy_plan(ConversionParamsModel {
            conversion_type: ConversionTypeModel::Single,
            keep_equipment: true,
            preserve_can_pick_up_loot: true,
            team_present: true,
        }),
        ConversionCopyPlan {
            copy_position_motion_passenger_vehicle: true,
            keep_equipment: true,
            copy_effects_absorption_age_anger_and_flags: true,
            preserve_can_pick_up_loot: true,
            move_scoreboard_team: true,
            discard_original: true,
        }
    );
    assert!(!ConversionTypeModel::SplitOnDeath.discard_after_conversion());
}

#[test]
fn axolotl_variants_match_java_ids_default_and_rare_flag() {
    assert_eq!(DEFAULT_AXOLOTL_VARIANT_ID, 0);
    assert_eq!(AXOLOTL_RARE_VARIANT_CHANCE, 1200);
    assert_eq!(
        AXOLOTL_VARIANTS,
        &[
            AxolotlVariantModel {
                name: "lucy",
                id: 0,
                common_spawn: true,
            },
            AxolotlVariantModel {
                name: "wild",
                id: 1,
                common_spawn: true,
            },
            AxolotlVariantModel {
                name: "gold",
                id: 2,
                common_spawn: true,
            },
            AxolotlVariantModel {
                name: "cyan",
                id: 3,
                common_spawn: true,
            },
            AxolotlVariantModel {
                name: "blue",
                id: 4,
                common_spawn: false,
            },
        ]
    );
    assert_eq!(axolotl_variant_by_id(-1).name, "lucy");
    assert_eq!(axolotl_variant_by_id(99).name, "lucy");
    assert_eq!(axolotl_variant_by_name("blue").unwrap().id, 4);

    let common = axolotl_spawn_variants(true);
    assert_eq!(common.len(), 4);
    assert!(common.iter().all(|variant| variant.common_spawn));

    let rare = axolotl_spawn_variants(false);
    assert_eq!(rare, vec![AXOLOTL_VARIANTS[4]]);
}

#[test]
fn axolotl_play_dead_and_air_rules_match_java_gates() {
    let mut axolotl = AxolotlState::new();
    assert_eq!(AXOLOTL_MAX_HEALTH, 14.0);
    assert_eq!(AXOLOTL_MOVEMENT_SPEED, 1.0);
    assert_eq!(AXOLOTL_ATTACK_DAMAGE, 2.0);
    assert_eq!(AXOLOTL_STEP_HEIGHT, 1.0);
    assert_eq!(axolotl_dimensions(false), (0.75, 0.42, 0.2751));
    assert_eq!(axolotl_dimensions(true), (0.5, 0.25, 0.2));
    assert_eq!(axolotl.air_supply, AXOLOTL_MAX_AIR_SUPPLY);
    assert!(axolotl.should_play_ambient_sound());
    assert!(axolotl.can_be_seen_as_enemy(true));
    assert_eq!(
        axolotl_bucket_pickup_result("minecraft:water_bucket", true),
        BucketPickupResult::FilledBucketAndDiscardEntity
    );
    assert_eq!(
        axolotl_bucket_pickup_result("minecraft:bucket", true),
        BucketPickupResult::NotApplicable
    );
    assert!(axolotl_requires_custom_persistence(false, true));
    assert!(!axolotl_requires_custom_persistence(false, false));
    assert!(!axolotl_remove_when_far_away(true, false));
    assert!(!axolotl_remove_when_far_away(false, true));
    assert!(axolotl_remove_when_far_away(false, false));
    assert_eq!(
        axolotl_bucket_saved_keys(false),
        vec!["Variant", "Age", "AgeLocked"]
    );
    assert_eq!(
        axolotl_bucket_saved_keys(true),
        vec!["Variant", "Age", "AgeLocked", "HuntingCooldown"]
    );

    axolotl.update_playing_dead_from_memory(Some(AXOLOTL_PLAY_DEAD_TICKS), false);
    assert!(axolotl.playing_dead);
    assert!(!axolotl.should_play_ambient_sound());
    assert!(!axolotl.can_be_seen_as_enemy(true));

    axolotl.update_playing_dead_from_memory(Some(AXOLOTL_PLAY_DEAD_TICKS), true);
    assert!(axolotl.playing_dead);
    axolotl.update_playing_dead_from_memory(None, false);
    assert!(!axolotl.playing_dead);

    axolotl.air_supply = 5000;
    axolotl.rehydrate();
    assert_eq!(axolotl.air_supply, AXOLOTL_MAX_AIR_SUPPLY);
    axolotl.air_supply = 1000;
    axolotl.rehydrate();
    assert_eq!(axolotl.air_supply, 2800);

    let triggering = AxolotlHurtContext {
        no_ai: false,
        random_one_in_three: true,
        random_damage_gate: 1,
        damage: 2.0,
        current_health: 10.0,
        max_health: 14.0,
        in_water: true,
        source_entity_present: true,
        direct_entity_present: false,
    };
    assert_eq!(
        axolotl_play_dead_memory_on_hurt(triggering),
        Some(AXOLOTL_PLAY_DEAD_TICKS)
    );

    assert_eq!(
        axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
            in_water: false,
            ..triggering
        }),
        None
    );
    assert_eq!(
        axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
            current_health: 2.0,
            damage: 2.0,
            ..triggering
        }),
        None
    );
    assert_eq!(
        axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
            source_entity_present: false,
            direct_entity_present: false,
            ..triggering
        }),
        None
    );
    assert_eq!(
        axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
            random_one_in_three: false,
            ..triggering
        }),
        None
    );
    assert_eq!(AXOLOTL_DRY_OUT_DAMAGE, 2.0);
    assert!(axolotl_can_attack_target(
        "minecraft:drowned",
        true,
        64.0,
        true,
        true
    ));
    assert!(axolotl_can_attack_target(
        "minecraft:cod",
        true,
        64.0,
        false,
        true
    ));
    assert!(!axolotl_can_attack_target(
        "minecraft:cod",
        true,
        64.0,
        true,
        true
    ));
    assert!(!axolotl_can_attack_target(
        "minecraft:cod",
        false,
        64.0,
        false,
        true
    ));
    assert!(!axolotl_can_attack_target(
        "minecraft:cod",
        true,
        64.1,
        false,
        true
    ));
    assert!(!axolotl_can_attack_target(
        "minecraft:zombie",
        true,
        10.0,
        false,
        true
    ));
    assert!(!axolotl_can_attack_target(
        "minecraft:drowned",
        true,
        10.0,
        false,
        false
    ));
    assert_eq!(
        axolotl_find_attack_target(false, Some("minecraft:cod")),
        Some("minecraft:cod")
    );
    assert_eq!(
        axolotl_find_attack_target(true, Some("minecraft:cod")),
        None
    );
    assert_eq!(AXOLOTL_HUNTING_COOLDOWN_TICKS, 2400);
    assert_eq!(AXOLOTL_PLAYER_REGEN_DETECTION_RANGE, 20.0);
    assert_eq!(
        axolotl_on_stop_attacking_effects(true, true, true, None, true),
        (Some(100), false)
    );
    assert_eq!(
        axolotl_on_stop_attacking_effects(true, true, true, Some(2350), true),
        (Some(2400), false)
    );
    assert_eq!(
        axolotl_on_stop_attacking_effects(true, true, true, Some(2400), true),
        (Some(2400), false)
    );
    assert_eq!(
        axolotl_on_stop_attacking_effects(true, false, true, Some(20), true),
        (Some(20), true)
    );
}

#[test]
fn chicken_egg_flap_jockey_and_dimensions_match_java_rules() {
    assert_eq!(chicken_next_egg_time(-1), CHICKEN_EGG_TIME_MIN);
    assert_eq!(chicken_next_egg_time(0), 6000);
    assert_eq!(chicken_next_egg_time(5999), 11999);
    assert_eq!(chicken_next_egg_time(6000), 11999);
    assert_eq!(chicken_dimensions(false), (0.4, 0.7, None));
    assert_eq!(chicken_dimensions(true), (0.3, 0.4, Some(0.28)));

    let mut chicken = ChickenState::new(0);
    chicken.egg_time = 1;
    chicken.delta_y = -1.0;
    assert_eq!(
        chicken.tick(false, true, false, true, 42),
        ChickenTickEvent::LayEgg
    );
    assert_eq!(chicken.egg_time, 6042);
    assert_eq!(chicken.flap_speed, 1.0);
    assert_eq!(chicken.flapping, 0.9);
    assert_eq!(chicken.delta_y, -0.6);
    assert_eq!(chicken.flap, 1.8);

    let mut baby = ChickenState::new(0);
    baby.egg_time = 1;
    assert_eq!(baby.tick(true, true, true, true, 0), ChickenTickEvent::None);
    assert_eq!(baby.egg_time, 1);

    let mut jockey = ChickenState::new(0);
    jockey.is_chicken_jockey = true;
    jockey.egg_time = 1;
    assert_eq!(
        jockey.tick(true, true, false, true, 0),
        ChickenTickEvent::None
    );
    assert_eq!(jockey.egg_time, 1);
    assert!(jockey.remove_when_far_away());
    assert_eq!(
        jockey.base_experience_reward(1),
        CHICKEN_JOCKEY_BASE_EXPERIENCE
    );
}

#[test]
fn cow_milking_dimensions_breeding_and_mooshroom_mutation_match_java_rules() {
    assert!(cow_is_food("minecraft:wheat"));
    assert!(!cow_is_food("minecraft:hay_block"));
    assert_eq!(
        cow_interaction("minecraft:bucket", false),
        CowInteraction::FillMilkBucket
    );
    assert_eq!(
        cow_interaction("minecraft:bucket", true),
        CowInteraction::Delegate
    );
    assert_eq!(
        cow_interaction("minecraft:bowl", false),
        CowInteraction::Delegate
    );
    assert_eq!(cow_dimensions(false), (0.9, 1.4, None));
    assert_eq!(cow_dimensions(true), (0.45, 0.7, Some(0.665)));

    assert_eq!(
        cow_breed_variant("minecraft:warm", "minecraft:cold", true),
        "minecraft:warm"
    );
    assert_eq!(
        cow_breed_variant("minecraft:warm", "minecraft:cold", false),
        "minecraft:cold"
    );

    assert_eq!(
        mooshroom_thunder_variant(MooshroomVariant::Red, None, "bolt-a"),
        MooshroomVariant::Brown
    );
    assert_eq!(
        mooshroom_thunder_variant(MooshroomVariant::Brown, Some("bolt-a"), "bolt-a"),
        MooshroomVariant::Brown
    );
    assert_eq!(MOOSHROOM_MUTATE_CHANCE, 1024);
    assert_eq!(
        mooshroom_offspring_variant(MooshroomVariant::Red, MooshroomVariant::Red, true, true),
        MooshroomVariant::Brown
    );
    assert_eq!(
        mooshroom_offspring_variant(MooshroomVariant::Brown, MooshroomVariant::Brown, true, true),
        MooshroomVariant::Red
    );
    assert_eq!(
        mooshroom_offspring_variant(MooshroomVariant::Red, MooshroomVariant::Brown, false, false),
        MooshroomVariant::Brown
    );
}

#[test]
fn dolphin_moistness_feeding_treasure_and_grace_match_java_rules() {
    let mut dolphin = DolphinState::new();
    assert_eq!(dolphin.air_supply, DOLPHIN_TOTAL_AIR_SUPPLY);
    assert_eq!(dolphin.moistness, DOLPHIN_TOTAL_MOISTNESS_LEVEL);
    assert!(!dolphin.got_fish);
    assert_eq!(DOLPHIN_BABY_SCALE, 0.65);
    assert_eq!(DOLPHIN_TREASURE_SEARCH_RADIUS, 50);
    assert_eq!(DOLPHIN_TREASURE_STOP_DISTANCE, 4.0);

    dolphin.moistness = 10;
    assert_eq!(
        dolphin.tick_moistness(false, true, false),
        DolphinLandTickOutcome {
            dry_out_damage: None,
            jump_delta_y: None,
            needs_sync: false,
        }
    );
    assert_eq!(dolphin.moistness, DOLPHIN_TOTAL_MOISTNESS_LEVEL);

    dolphin.moistness = 1;
    assert_eq!(
        dolphin.tick_moistness(false, false, true),
        DolphinLandTickOutcome {
            dry_out_damage: Some(DOLPHIN_DRY_OUT_DAMAGE),
            jump_delta_y: Some(0.5),
            needs_sync: true,
        }
    );
    assert_eq!(dolphin.moistness, 0);

    dolphin.air_supply = 1;
    let no_ai_outcome = dolphin.tick_moistness(true, false, true);
    assert_eq!(dolphin.air_supply, DOLPHIN_TOTAL_AIR_SUPPLY);
    assert_eq!(no_ai_outcome.dry_out_damage, None);

    assert_eq!(
        dolphin_feed_result(false, false, 24_000),
        DolphinFeedResult::NotFish
    );
    assert_eq!(
        dolphin_feed_result(true, true, 24_000),
        DolphinFeedResult::AgeUp { seconds: 120 }
    );
    assert_eq!(
        dolphin_feed_result(true, false, 24_000),
        DolphinFeedResult::SetGotFish
    );

    dolphin.got_fish = true;
    dolphin.air_supply = 99;
    assert!(!dolphin.can_start_treasure_goal());
    dolphin.air_supply = 100;
    assert!(dolphin.can_start_treasure_goal());
    assert!(dolphin.should_clear_got_fish_on_treasure_stop(false, true, false));
    assert!(dolphin.should_clear_got_fish_on_treasure_stop(false, false, true));
    assert!(!dolphin.should_clear_got_fish_on_treasure_stop(false, false, false));

    assert_eq!(DOLPHIN_SWIM_WITH_PLAYER_RANGE, 10.0);
    assert_eq!(DOLPHIN_SWIM_WITH_PLAYER_CONTINUE_DISTANCE_SQUARED, 256.0);
    assert_eq!(dolphin_grace_refresh_ticks(true, 0), Some(100));
    assert_eq!(dolphin_grace_refresh_ticks(true, 5), None);
    assert_eq!(dolphin_grace_refresh_ticks(false, 0), None);
}

#[test]
fn bee_flags_sting_hive_and_pollination_counters_match_java_rules() {
    let mut bee = BeeState::new();
    assert_eq!(bee.flags, 0);
    assert!(!bee.has_nectar());
    assert!(!bee.has_stung());
    assert!(!bee.is_rolling());
    assert_eq!(BEE_FLAG_ROLL, 2);
    assert_eq!(BEE_FLAG_HAS_STUNG, 4);
    assert_eq!(BEE_FLAG_HAS_NECTAR, 8);
    assert_eq!(BEE_STING_DEATH_COUNTDOWN, 1200);
    assert_eq!(BEE_TICKS_BEFORE_GOING_TO_KNOWN_FLOWER, 600);
    assert_eq!(BEE_TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME, 3600);
    assert_eq!(BEE_MAX_CROPS_GROWABLE, 10);
    assert_eq!(BEE_GROW_CROP_CHANCE_BOUND, 30);
    assert_eq!(BEE_TOO_FAR_DISTANCE, 48);
    assert_eq!(BEE_HIVE_CLOSE_ENOUGH_DISTANCE, 2);
    assert_eq!(BEE_HIVE_SEARCH_DISTANCE, 20);
    assert_eq!(BEE_HIVE_MAX_OCCUPANTS, 3);
    assert_eq!(bee_hive_min_ticks_in_hive(true), 2400);
    assert_eq!(bee_hive_min_ticks_in_hive(false), 600);
    assert!(bee_hive_can_add_occupant(2));
    assert!(!bee_hive_can_add_occupant(3));
    assert_eq!(BEE_COOLDOWN_BEFORE_LOCATING_NEW_HIVE, 200);
    assert_eq!(BEE_MIN_FIND_FLOWER_RETRY_COOLDOWN, 20);
    assert_eq!(BEE_MAX_FIND_FLOWER_RETRY_COOLDOWN, 60);
    assert_eq!(BEE_PERSISTENT_ANGER_MIN_TICKS, 400);
    assert_eq!(BEE_PERSISTENT_ANGER_MAX_TICKS, 780);
    assert_eq!(BEE_MAX_HEALTH, 10.0);
    assert_eq!(BEE_FLYING_SPEED, 0.6);
    assert_eq!(BEE_MOVEMENT_SPEED, 0.3);
    assert_eq!(BEE_ATTACK_DAMAGE, 2.0);

    bee.ticks_without_nectar_since_exiting_hive = 12;
    bee.set_has_nectar(true);
    assert!(bee.has_nectar());
    assert_eq!(bee.ticks_without_nectar_since_exiting_hive, 0);
    bee.set_has_nectar(false);
    assert!(!bee.has_nectar());

    bee.set_has_nectar(true);
    assert!(bee.can_grow_crop(0.3, true));
    assert!(!bee.can_grow_crop(0.29, true));
    assert!(!bee.can_grow_crop(0.3, false));
    bee.crops_grown_since_pollination = BEE_MAX_CROPS_GROWABLE;
    assert!(!bee.can_grow_crop(0.3, true));
    bee.crops_grown_since_pollination = 2;
    assert!(bee.grow_crop(0, false, true));
    assert_eq!(bee.crops_grown_since_pollination, 3);
    assert!(!bee.grow_crop(1, true, true));
    bee.drop_off_nectar();
    assert!(!bee.has_nectar());
    assert_eq!(bee.crops_grown_since_pollination, 0);

    bee.stay_out_of_hive_countdown = 1;
    assert!(!bee.wants_to_enter_hive(false, false, true, false));
    bee.stay_out_of_hive_countdown = 0;
    assert!(bee.wants_to_enter_hive(false, false, true, false));
    assert!(!bee.wants_to_enter_hive(false, false, true, true));
    bee.set_has_stung(true);
    assert!(!bee.wants_to_enter_hive(false, false, true, false));

    bee.set_has_stung(false);
    bee.tick_ai_step(true, true, 3.99);
    assert!(bee.is_rolling());
    bee.tick_ai_step(true, true, 4.0);
    assert!(!bee.is_rolling());

    for _ in 0..20 {
        assert_eq!(
            bee.tick_server_ai(true, false).drown_damage,
            None,
            "bee only takes drown damage after more than 20 underwater ticks"
        );
    }
    assert_eq!(
        bee.tick_server_ai(true, false).drown_damage,
        Some(BEE_DROWN_DAMAGE)
    );
    assert_eq!(bee.tick_server_ai(false, false).drown_damage, None);

    bee.set_has_stung(true);
    bee.time_since_sting = 4;
    let sting_outcome = bee.tick_server_ai(false, true);
    assert!(sting_outcome.sting_death_damage);
    assert_eq!(bee.time_since_sting, 5);
    assert_eq!(bee_poison_duration_ticks("peaceful"), None);
    assert_eq!(bee_poison_duration_ticks("normal"), Some(200));
    assert_eq!(bee_poison_duration_ticks("hard"), Some(360));

    assert_eq!(
        bee_hive_use(5, "minecraft:shears", true, false, true),
        BeeHiveUseResult::ShearHoneycombAndReset
    );
    assert_eq!(
        bee_hive_use(5, "minecraft:glass_bottle", false, false, true),
        BeeHiveUseResult::FillHoneyBottleAndReset
    );
    assert_eq!(
        bee_hive_use(4, "minecraft:shears", true, false, true),
        BeeHiveUseResult::Delegate
    );
    assert!(bee_hive_should_anger_nearby_bees(true, false, true));
    assert!(!bee_hive_should_anger_nearby_bees(true, true, true));
    assert!(!bee_hive_should_anger_nearby_bees(true, false, false));
    let hive_release = BeeHiveReleaseInput {
        bees_stay_in_hive_environment: false,
        emergency: false,
        front_blocked: false,
        honey_delivered: true,
        honey_level: 0,
        honey_level_bonus_roll_hits: false,
        bee_has_saved_flower: false,
        hive_has_saved_flower: false,
        copy_flower_roll_hits: false,
    };
    assert_eq!(
        bee_hive_release_occupant(BeeHiveReleaseInput {
            bees_stay_in_hive_environment: true,
            ..hive_release
        }),
        BeeHiveReleaseResult::StayInHive
    );
    assert_eq!(
        bee_hive_release_occupant(BeeHiveReleaseInput {
            front_blocked: true,
            ..hive_release
        }),
        BeeHiveReleaseResult::FrontBlocked
    );
    assert_eq!(
        bee_hive_release_occupant(BeeHiveReleaseInput {
            honey_level: 4,
            honey_level_bonus_roll_hits: true,
            hive_has_saved_flower: true,
            copy_flower_roll_hits: true,
            ..hive_release
        }),
        BeeHiveReleaseResult::Release {
            honey_level: 5,
            bee_has_nectar: false,
            bee_crops_grown: 0,
            copied_flower_pos: true,
        }
    );
    assert_eq!(
        bee_hive_release_occupant(BeeHiveReleaseInput {
            emergency: true,
            front_blocked: true,
            honey_delivered: false,
            honey_level: 2,
            hive_has_saved_flower: true,
            copy_flower_roll_hits: true,
            ..hive_release
        }),
        BeeHiveReleaseResult::Release {
            honey_level: 2,
            bee_has_nectar: false,
            bee_crops_grown: 0,
            copied_flower_pos: true,
        }
    );
}

#[test]
fn camel_dash_pose_and_passenger_offsets_match_java_rules() {
    assert_eq!(CAMEL_BABY_SCALE, 0.6);
    assert_eq!(CAMEL_DASH_COOLDOWN_TICKS, 55);
    assert_eq!(CAMEL_MAX_HEAD_Y_ROT, 30);
    assert_eq!(CAMEL_RUNNING_SPEED_BONUS, 0.1);
    assert_eq!(CAMEL_DASH_VERTICAL_MOMENTUM, 1.4285);
    assert_eq!(CAMEL_DASH_HORIZONTAL_MOMENTUM, 22.2222);
    assert_eq!(CAMEL_DASH_MINIMUM_DURATION_TICKS, 5);
    assert_eq!(CAMEL_SITDOWN_DURATION_TICKS, 40);
    assert_eq!(CAMEL_STANDUP_DURATION_TICKS, 52);
    assert_eq!(CAMEL_IDLE_MINIMAL_DURATION_TICKS, 80);
    assert_eq!(CAMEL_SITTING_HEIGHT_DIFFERENCE, 1.43);
    assert_eq!(CAMEL_SITTING_EYE_HEIGHT, 0.845);

    let mut camel = CamelState::new_standing(100);
    assert_eq!(camel.last_pose_change_tick, 47);
    assert!(!camel.is_in_pose_transition(100));
    assert!(!camel.refuse_to_move(100));
    assert!(camel.can_start_dash(true, true));
    assert!(!camel.can_start_dash(false, true));
    assert!(!camel.can_start_dash(true, false));

    camel.start_dash();
    assert!(camel.dashing);
    assert_eq!(camel.dash_cooldown, CAMEL_DASH_COOLDOWN_TICKS);
    assert!(!camel.tick(true, false, false).dash_ready_sound);
    assert!(
        camel.dashing,
        "dash remains visible through the first five ticks"
    );
    for _ in 0..4 {
        camel.tick(true, false, false);
    }
    assert_eq!(camel.dash_cooldown, 50);
    camel.tick(true, false, false);
    assert!(camel.dashing);
    assert_eq!(camel.dash_cooldown, 49);
    camel.tick(true, false, false);
    assert!(!camel.dashing);
    for _ in 0..47 {
        assert!(!camel.tick(false, false, false).dash_ready_sound);
    }
    assert!(camel.tick(false, false, false).dash_ready_sound);
    assert_eq!(camel.dash_cooldown, 0);
    assert_eq!(camel.ridden_speed(0.09, true), 0.19);

    assert!(camel.sit_down(200));
    assert!(camel.is_sitting());
    assert_eq!(camel.last_pose_change_tick, -200);
    assert!(camel.refuse_to_move(239));
    assert!(
        camel.refuse_to_move(240),
        "sitting camels refuse movement after transition too"
    );
    assert!(camel.stand_up(300));
    assert!(!camel.is_sitting());
    assert!(camel.is_in_pose_transition(351));
    assert!(!camel.is_in_pose_transition(352));
    camel.stand_up_instantly(400);
    assert_eq!(camel.last_pose_change_tick, 347);

    let (horizontal, vertical) = camel_dash_impulse(1.0, 0.09, 1.0, 0.42);
    assert!((horizontal - 1.999998).abs() < 0.00001);
    assert!((vertical - 0.59997).abs() < 0.00001);

    let passenger_attachment = CamelPassengerAttachmentInput {
        passenger_index: 0,
        passenger_count: 2,
        passenger_is_animal: false,
        camel_sitting: false,
        removed: false,
        dimensions_width: 1.7,
        dimensions_height: 2.375,
        scale: 1.0,
    };
    let driver = camel_passenger_attachment_point(passenger_attachment);
    assert_eq!(driver.x, 0.0);
    assert!((driver.y - 2.0).abs() < 0.00001);
    assert!((driver.z - 0.5).abs() < 0.00001);
    let rear_animal = camel_passenger_attachment_point(CamelPassengerAttachmentInput {
        passenger_index: 1,
        passenger_is_animal: true,
        ..passenger_attachment
    });
    assert!((rear_animal.z - -0.5).abs() < 0.00001);
    assert!(CamelState::can_add_passenger(2));
    assert!(!CamelState::can_add_passenger(3));
}

#[test]
fn goat_milking_horns_ram_and_head_lowering_match_java_rules() {
    assert_eq!(GOAT_SCREAMING_CHANCE_DENOMINATOR, 50);
    assert_eq!(GOAT_INITIAL_MISSING_HORN_CHANCE_DENOMINATOR, 10);
    assert_eq!(GOAT_RAM_PREPARE_TIME, 20);
    assert_eq!(GOAT_RAM_MIN_DISTANCE, 4);
    assert_eq!(GOAT_RAM_MAX_DISTANCE, 7);
    assert_eq!(GOAT_TIME_BETWEEN_RAMS_MIN, 600);
    assert_eq!(GOAT_TIME_BETWEEN_RAMS_MAX, 6000);
    assert_eq!(GOAT_TIME_BETWEEN_RAMS_SCREAMER_MIN, 100);
    assert_eq!(GOAT_TIME_BETWEEN_RAMS_SCREAMER_MAX, 300);
    assert_eq!(GOAT_TIME_BETWEEN_LONG_JUMPS_MIN, 600);
    assert_eq!(GOAT_TIME_BETWEEN_LONG_JUMPS_MAX, 1200);
    assert_eq!(GOAT_MAX_LONG_JUMP_HEIGHT, 5);
    assert_eq!(GOAT_MAX_LONG_JUMP_WIDTH, 5);
    assert_eq!(GOAT_MAX_JUMP_VELOCITY_MULTIPLIER, 3.5714288);
    assert_eq!(GOAT_PREPARE_RAM_SPEED_MULTIPLIER, 1.25);
    assert_eq!(GOAT_RAMMING_SPEED_MULTIPLIER, 3.0);
    assert_eq!(GOAT_ADULT_RAM_KNOCKBACK_FORCE, 2.5);
    assert_eq!(GOAT_BABY_RAM_KNOCKBACK_FORCE, 1.0);
    assert!((GOAT_LONG_JUMPING_WIDTH - 0.63).abs() < 0.00001);
    assert!((GOAT_LONG_JUMPING_HEIGHT - 0.91).abs() < 0.00001);

    assert_eq!(
        goat_interaction("minecraft:bucket", false),
        GoatInteraction::Milk
    );
    assert_eq!(
        goat_interaction("minecraft:bucket", true),
        GoatInteraction::Delegate
    );
    assert_eq!(
        goat_interaction("minecraft:wheat", false),
        GoatInteraction::Delegate
    );
    assert_eq!(goat_ram_knockback_force(false), 2.5);
    assert_eq!(goat_ram_knockback_force(true), 1.0);

    let normal = GoatState::finalize_spawn(false, true, false, true);
    assert!(!normal.is_screaming);
    assert!(normal.has_left_horn);
    assert!(normal.has_right_horn);
    assert_eq!(normal.ram_cooldown_range(), (600, 6000));

    let mut screaming_missing_left = GoatState::finalize_spawn(true, true, true, true);
    assert!(screaming_missing_left.is_screaming);
    assert!(!screaming_missing_left.has_left_horn);
    assert!(screaming_missing_left.has_right_horn);
    assert_eq!(screaming_missing_left.ram_cooldown_range(), (100, 300));
    assert_eq!(
        screaming_missing_left.drop_horn(false, true),
        GoatHornDrop::Right
    );
    assert!(!screaming_missing_left.has_right_horn);
    assert_eq!(
        screaming_missing_left.drop_horn(false, true),
        GoatHornDrop::None
    );

    let baby_missing_roll = GoatState::finalize_spawn(true, false, true, false);
    assert!(baby_missing_roll.has_left_horn);
    assert!(baby_missing_roll.has_right_horn);
    let mut both_horns = GoatState::new();
    assert_eq!(both_horns.drop_horn(true, true), GoatHornDrop::None);
    assert_eq!(both_horns.drop_horn(false, false), GoatHornDrop::Right);
    assert!(both_horns.has_left_horn);
    assert!(!both_horns.has_right_horn);

    let mut lowering = GoatState::new();
    for _ in 0..25 {
        lowering.tick_lower_head(true);
    }
    assert_eq!(lowering.lower_head_tick, GOAT_MAX_LOWER_HEAD_TICK);
    let adult_rot = lowering.ramming_x_head_rot_radians(false);
    let baby_rot = lowering.ramming_x_head_rot_radians(true);
    assert!((adult_rot - std::f32::consts::PI / 6.0).abs() < 0.00001);
    assert!((baby_rot - (52.5_f32 * std::f32::consts::PI / 180.0)).abs() < 0.00001);
    lowering.tick_lower_head(false);
    assert_eq!(lowering.lower_head_tick, 18);
    for _ in 0..20 {
        lowering.tick_lower_head(false);
    }
    assert_eq!(lowering.lower_head_tick, 0);
}

#[test]
fn pig_saddle_boost_food_on_a_stick_and_lightning_match_java_rules() {
    assert_eq!(PIG_MAX_HEALTH, 10.0);
    assert_eq!(PIG_MOVEMENT_SPEED, 0.25);
    assert_eq!(PIG_RIDDEN_SPEED_FACTOR, 0.225);
    assert_eq!(PIG_BOOST_MIN_TIME, 140);
    assert_eq!(PIG_BOOST_MAX_TIME, 980);
    assert_eq!(PIG_BOOST_RANDOM_BOUND, 841);
    assert_eq!(PIG_BOOST_SPEED_AMPLIFIER, 1.15);
    assert_eq!(PIG_CARROT_ON_A_STICK_DURABILITY, 25);
    assert_eq!(PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST, 7);
    assert_eq!(PIG_TEMPT_SPEED, 1.2);
    assert_eq!(PIG_PANIC_SPEED, 1.25);
    assert_eq!(PIG_FOLLOW_PARENT_SPEED, 1.1);
    assert_eq!(PIG_LEASH_EYE_HEIGHT_FACTOR, 0.6);
    assert_eq!(PIG_LEASH_WIDTH_FACTOR, 0.4);

    let mut pig = PigState::new();
    assert!(!pig.controlling_passenger(true, true));
    pig.saddled = true;
    assert!(pig.controlling_passenger(true, true));
    assert!(!pig.controlling_passenger(false, true));
    assert!(!pig.controlling_passenger(true, false));
    assert!(PigState::can_use_saddle_slot(true, false));
    assert!(!PigState::can_use_saddle_slot(false, false));
    assert!(!PigState::can_use_saddle_slot(true, true));

    assert_eq!(
        pig_interaction(false, true, false, false, false, false),
        PigInteraction::StartRide
    );
    assert_eq!(
        pig_interaction(true, true, false, false, true, false),
        PigInteraction::DelegateToAnimal
    );
    assert_eq!(
        pig_interaction(false, true, true, false, false, true),
        PigInteraction::EquipSaddle
    );
    assert_eq!(
        pig_interaction(false, false, false, false, false, false),
        PigInteraction::Pass
    );

    assert!(pig.boost(0));
    assert_eq!(pig.boost_time_total, PIG_BOOST_MIN_TIME);
    assert_eq!(pig.boost_time, 0);
    assert!(!pig.boost(840), "ItemBasedSteering rejects nested boosts");
    pig.boost_time = pig.boost_time_total / 2;
    assert!((pig.boost_factor() - 2.15).abs() < 0.00001);
    assert!((pig.ridden_speed(PIG_MOVEMENT_SPEED) - 0.1209375).abs() < 0.00001);
    pig.boost_time = pig.boost_time_total;
    pig.tick_boost();
    assert!(
        pig.boosting,
        "Java stops only after old boostTime is greater than total"
    );
    pig.tick_boost();
    assert!(!pig.boosting);
    assert_eq!(pig.boost_factor(), 1.0);

    let mut max_boost = PigState::new();
    assert!(max_boost.boost(840));
    assert_eq!(max_boost.boost_time_total, PIG_BOOST_MAX_TIME);
    assert!(max_boost.boost(999) == false);

    assert_eq!(
        pig_food_on_a_stick_use(true, true, true, true, 0),
        PigBoostUseResult::BoostStarted {
            damage: 7,
            converts_to_fishing_rod: false,
        }
    );
    assert_eq!(
        pig_food_on_a_stick_use(true, true, true, true, 18),
        PigBoostUseResult::BoostStarted {
            damage: 7,
            converts_to_fishing_rod: true,
        }
    );
    assert_eq!(
        pig_food_on_a_stick_use(false, true, true, true, 0),
        PigBoostUseResult::Pass
    );

    assert!(!pig_thunder_converts_to_zombified_piglin("peaceful"));
    assert!(pig_thunder_converts_to_zombified_piglin("easy"));
    assert_eq!(
        pig_offspring_variant("minecraft:temperate", "minecraft:cold", true),
        "minecraft:temperate"
    );
    assert_eq!(
        pig_offspring_variant("minecraft:temperate", "minecraft:cold", false),
        "minecraft:cold"
    );
}

#[test]
fn polar_bear_cub_targeting_standing_warning_and_swim_match_java_rules() {
    assert_eq!(POLAR_BEAR_MAX_HEALTH, 30.0);
    assert_eq!(POLAR_BEAR_FOLLOW_RANGE, 20.0);
    assert_eq!(POLAR_BEAR_PLAYER_ATTACK_FOLLOW_DISTANCE_FACTOR, 0.5);
    assert_eq!(
        POLAR_BEAR_FOLLOW_RANGE * POLAR_BEAR_PLAYER_ATTACK_FOLLOW_DISTANCE_FACTOR,
        10.0
    );
    assert_eq!(POLAR_BEAR_MOVEMENT_SPEED, 0.25);
    assert_eq!(POLAR_BEAR_ATTACK_DAMAGE, 6.0);
    assert_eq!(POLAR_BEAR_MELEE_SPEED, 1.25);
    assert_eq!(POLAR_BEAR_PANIC_SPEED, 2.0);
    assert_eq!(POLAR_BEAR_FOLLOW_PARENT_SPEED, 1.25);
    assert_eq!(POLAR_BEAR_RANDOM_STROLL_SPEED, 1.0);
    assert_eq!(POLAR_BEAR_LOOK_AT_PLAYER_DISTANCE, 6.0);
    assert_eq!(POLAR_BEAR_STAND_ANIMATION_TICKS, 6.0);
    assert_eq!(POLAR_BEAR_WARNING_SOUND_COOLDOWN_TICKS, 40);
    assert_eq!(POLAR_BEAR_WARNING_ATTACK_TICKS, 10);
    assert_eq!(POLAR_BEAR_CUB_ALERT_XZ_RANGE, 8.0);
    assert_eq!(POLAR_BEAR_CUB_ALERT_Y_RANGE, 4.0);
    assert_eq!(POLAR_BEAR_PERSISTENT_ANGER_MIN_TICKS, 400);
    assert_eq!(POLAR_BEAR_PERSISTENT_ANGER_MAX_TICKS, 780);
    assert_eq!(POLAR_BEAR_WATER_SLOWDOWN, 0.98);

    assert!(polar_bear_should_attack_player(false, true, true));
    assert!(!polar_bear_should_attack_player(true, true, true));
    assert!(!polar_bear_should_attack_player(false, false, true));
    assert!(!polar_bear_should_attack_player(false, true, false));
    assert!(polar_bear_should_attack_fox(false));
    assert!(!polar_bear_should_attack_fox(true));
    assert!(polar_bear_should_alert_other_on_hurt(true, false));
    assert!(!polar_bear_should_alert_other_on_hurt(true, true));
    assert!(!polar_bear_should_alert_other_on_hurt(false, false));

    let mut bear = PolarBearState::new();
    assert!(bear.play_warning_sound());
    assert_eq!(bear.warning_sound_ticks, 40);
    assert!(!bear.play_warning_sound());
    bear.tick(false);
    assert_eq!(bear.warning_sound_ticks, 39);
    bear.warning_sound_ticks = 0;
    assert!(bear.play_warning_sound());

    bear.standing = true;
    for _ in 0..10 {
        bear.tick(true);
    }
    assert_eq!(
        bear.client_stand_animation,
        POLAR_BEAR_STAND_ANIMATION_TICKS
    );
    assert_eq!(bear.dimensions_height_scale(), 2.0);
    bear.standing = false;
    bear.tick(true);
    assert_eq!(bear.client_stand_animation, 5.0);
    assert!((bear.standing_animation_scale(0.5) - (5.5 / 6.0)).abs() < 0.00001);

    assert_eq!(
        polar_bear_melee_signal(true, 100.0, 0.6, 20, false),
        PolarBearMeleeSignal::AttackAndStopStanding
    );
    assert_eq!(
        polar_bear_melee_signal(false, 12.0, 0.6, 10, false),
        PolarBearMeleeSignal::StandAndWarn
    );
    assert_eq!(
        polar_bear_melee_signal(false, 12.0, 0.6, 11, true),
        PolarBearMeleeSignal::StopStanding
    );
    assert_eq!(
        polar_bear_melee_signal(false, 13.0, 0.6, 10, false),
        PolarBearMeleeSignal::StopStanding
    );
}

#[test]
fn rabbit_variants_garden_raid_and_jump_timing_match_java_rules() {
    assert_eq!(RABBIT_MAX_HEALTH, 3.0);
    assert_eq!(RABBIT_MOVEMENT_SPEED, 0.3);
    assert_eq!(RABBIT_ATTACK_DAMAGE, 3.0);
    assert_eq!(RABBIT_EVIL_ATTACK_POWER_INCREMENT, 5.0);
    assert_eq!(RABBIT_EVIL_ARMOR_VALUE, 8.0);
    assert_eq!(RABBIT_STROLL_SPEED_MOD, 0.6);
    assert_eq!(RABBIT_BREED_SPEED_MOD, 0.8);
    assert_eq!(RABBIT_FOLLOW_SPEED_MOD, 1.0);
    assert_eq!(RABBIT_FLEE_SPEED_MOD, 2.2);
    assert_eq!(RABBIT_ATTACK_SPEED_MOD, 1.4);
    assert_eq!(RABBIT_BABY_JUMP_HEIGHT, 0.5);
    assert_eq!(RABBIT_ADULT_JUMP_HEIGHT, 1.5);
    assert_eq!(RABBIT_JUMP_DELAY_TICKS, 10);
    assert_eq!(RABBIT_PANIC_JUMP_DELAY_TICKS, 3);
    assert_eq!(RABBIT_JUMP_DURATION_TICKS, 15);
    assert_eq!(RABBIT_MORE_CARROTS_DELAY, 40);
    assert_eq!(RABBIT_IDLE_MINIMAL_DURATION_TICKS, 180);
    assert_eq!(RABBIT_BABY_WIDTH, 0.24);
    assert_eq!(RABBIT_BABY_HEIGHT, 0.4);
    assert_eq!(RABBIT_BABY_EYE_HEIGHT, 0.39);

    let variants = [
        (RabbitVariant::Brown, 0, "brown"),
        (RabbitVariant::White, 1, "white"),
        (RabbitVariant::Black, 2, "black"),
        (RabbitVariant::WhiteSplotched, 3, "white_splotched"),
        (RabbitVariant::Gold, 4, "gold"),
        (RabbitVariant::Salt, 5, "salt"),
        (RabbitVariant::Evil, 99, "evil"),
    ];
    for (variant, id, name) in variants {
        assert_eq!(variant.id(), id);
        assert_eq!(variant.name(), name);
        assert_eq!(RabbitVariant::by_id(id), variant);
    }
    assert_eq!(RabbitVariant::by_id(12345), RabbitVariant::Brown);
    assert!(RabbitVariant::Evil.is_evil());
    assert!(!RabbitVariant::Brown.is_evil());

    assert_eq!(rabbit_random_variant(true, false, 79), RabbitVariant::White);
    assert_eq!(
        rabbit_random_variant(true, false, 80),
        RabbitVariant::WhiteSplotched
    );
    assert_eq!(rabbit_random_variant(false, true, 99), RabbitVariant::Gold);
    assert_eq!(
        rabbit_random_variant(false, false, 49),
        RabbitVariant::Brown
    );
    assert_eq!(rabbit_random_variant(false, false, 89), RabbitVariant::Salt);
    assert_eq!(
        rabbit_random_variant(false, false, 90),
        RabbitVariant::Black
    );

    assert_eq!(
        rabbit_offspring_variant(
            RabbitVariant::Gold,
            RabbitVariant::Brown,
            RabbitVariant::Salt,
            0,
            true,
        ),
        RabbitVariant::Gold
    );
    assert_eq!(
        rabbit_offspring_variant(
            RabbitVariant::Gold,
            RabbitVariant::Brown,
            RabbitVariant::Salt,
            1,
            true,
        ),
        RabbitVariant::Salt
    );
    assert_eq!(
        rabbit_offspring_variant(
            RabbitVariant::Gold,
            RabbitVariant::Brown,
            RabbitVariant::Salt,
            1,
            false,
        ),
        RabbitVariant::Brown
    );

    let mut rabbit = RabbitState::new();
    assert!(rabbit.wants_more_food());
    rabbit.more_carrot_ticks = 40;
    assert!(!rabbit.wants_more_food());
    rabbit.tick_more_carrots(2);
    assert_eq!(rabbit.more_carrot_ticks, 38);
    rabbit.more_carrot_ticks = 1;
    rabbit.tick_more_carrots(2);
    assert_eq!(rabbit.more_carrot_ticks, 0);

    assert_eq!(
        rabbit_raid_garden(true, true, true, Some(0)),
        RabbitRaidGardenResult::DestroyCrop
    );
    assert_eq!(
        rabbit_raid_garden(true, true, true, Some(7)),
        RabbitRaidGardenResult::ReduceCarrotAge(6)
    );
    assert_eq!(
        rabbit_raid_garden(false, true, true, Some(7)),
        RabbitRaidGardenResult::Noop
    );
    assert!(rabbit_should_avoid_entity(RabbitVariant::Brown, true));
    assert!(!rabbit_should_avoid_entity(RabbitVariant::Evil, true));

    rabbit.start_jumping();
    assert_eq!(rabbit.jump_duration, 15);
    assert_eq!(rabbit.jump_ticks, 0);
    rabbit.jump_ticks = 7;
    assert!((rabbit.jump_completion(0.5) - 0.5).abs() < 0.00001);
    rabbit.jump_ticks = rabbit.jump_duration;
    rabbit.ai_step_jump();
    assert_eq!(rabbit.jump_ticks, 0);
    assert_eq!(rabbit.jump_duration, 0);
    rabbit.set_landing_delay(0.6);
    assert_eq!(rabbit.jump_delay_ticks, 10);
    rabbit.set_landing_delay(2.2);
    assert_eq!(rabbit.jump_delay_ticks, 3);
}
