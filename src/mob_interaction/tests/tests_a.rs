use crate::mob_interaction::*;

#[test]
fn ageable_mobs_tick_feed_and_age_lock_like_vanilla() {
    let baby = AgeState::baby();
    assert!(baby.is_baby());
    assert_eq!(baby.tick().age, -23_999);
    assert_eq!(speed_up_seconds_when_feeding(24_000), 120);
    assert_eq!(
        baby.age_up(120, true),
        AgeState {
            age: -21_600,
            forced_age: 2_400,
            forced_age_timer: 40,
            age_locked: false,
            age_lock_particle_timer: 0,
        }
    );
    let locked = baby.toggle_age_lock();
    assert!(locked.age_locked);
    assert_eq!(locked.age_lock_particle_timer, 40);
    assert_eq!(locked.tick().age, BABY_START_AGE);
}

#[test]
fn animals_enter_love_age_up_breed_and_reset_parent_state() {
    assert_eq!(
        animal_feed_result(true, 0, 0, false, false),
        AnimalFeedResult::SetInLove
    );
    assert_eq!(
        animal_feed_result(true, -10_000, 0, true, false),
        AnimalFeedResult::AgeUp { seconds: 50 }
    );
    assert_eq!(LoveState::set_in_love(true).tick(0).in_love, 599);
    assert_eq!(LoveState::set_in_love(true).tick(-1).in_love, 0);
    assert!(can_mate(true, false, true, true));
    assert!(!can_mate(false, false, true, true));

    assert_eq!(
        breeding_result(true),
        BreedingResult {
            parent_age: 6_000,
            partner_age: 6_000,
            child_age: -24_000,
            love_reset: true,
            xp_min: 1,
            xp_max_inclusive: 7,
        }
    );
}

#[test]
fn snow_golem_pumpkin_shearing_melting_trail_and_snowball_match_java() {
    assert_eq!(SNOW_GOLEM_MAX_HEALTH, 4.0);
    assert_eq!(SNOW_GOLEM_MOVEMENT_SPEED, 0.2);
    assert_eq!(SNOW_GOLEM_RANGED_ATTACK_INTERVAL_TICKS, 20);
    assert_eq!(SNOW_GOLEM_RANGED_ATTACK_RADIUS, 10.0);

    let mut golem = SnowGolemState::new();
    assert!(golem.has_pumpkin());
    assert!(golem.saved_pumpkin());
    assert!(golem.ready_for_shearing());
    assert_eq!(
        golem.shear(),
        SnowGolemShearResult::Sheared {
            sound: "minecraft:entity.snow_golem.shear",
            game_event: "minecraft:shear",
            loot_table: "minecraft:entities/shear/snow_golem",
            tool_damage: 1,
        }
    );
    assert!(!golem.has_pumpkin());
    assert_eq!(golem.shear(), SnowGolemShearResult::Pass);
    assert!(SnowGolemState::from_saved_pumpkin(true).has_pumpkin());
    assert!(!SnowGolemState::from_saved_pumpkin(false).has_pumpkin());

    let plan = snow_golem_ai_step_plan(10.0, 64.9, 2.0, true, true);
    assert_eq!(plan.melt_damage, Some(1));
    assert_eq!(
        plan.snow_positions,
        [(9, 64, 1), (10, 64, 1), (9, 64, 2), (10, 64, 2)]
    );
    assert_eq!(
        snow_golem_ai_step_plan(10.0, 64.9, 2.0, true, false).snow_positions,
        [(0, 0, 0); 4]
    );
    assert!(snow_golem_snow_placement_allowed("minecraft:air", true));
    assert!(!snow_golem_snow_placement_allowed("minecraft:stone", true));
    assert!(!snow_golem_snow_placement_allowed("minecraft:air", false));

    let attack = snow_golem_ranged_attack_plan(0.0, 0.0, 3.0, 66.6, 4.0, 65.0);
    assert_eq!(attack.item, "minecraft:snowball");
    assert_eq!(attack.velocity, (3.0, 1.5, 4.0));
    assert_eq!(attack.speed, 1.6);
    assert_eq!(attack.inaccuracy, 12.0);
    assert_eq!(attack.sound, "minecraft:entity.snow_golem.shoot");
}

#[test]
fn iron_golem_flags_cracks_repair_flower_and_attack_match_java() {
    assert_iron_golem_constants_and_crackiness();
    assert_iron_golem_player_created_flags();
    assert_iron_golem_flower_events_and_repair();
    assert_iron_golem_attack_rules();
}

fn assert_iron_golem_constants_and_crackiness() {
    assert_eq!(IRON_GOLEM_MAX_HEALTH, 100.0);
    assert_eq!(IRON_GOLEM_MOVEMENT_SPEED, 0.25);
    assert_eq!(IRON_GOLEM_KNOCKBACK_RESISTANCE, 1.0);
    assert_eq!(IRON_GOLEM_ATTACK_DAMAGE, 15.0);
    assert_eq!(IRON_GOLEM_STEP_HEIGHT, 1.0);

    assert_eq!(
        iron_golem_crackiness(100.0, 100.0),
        IronGolemCrackiness::None
    );
    assert_eq!(iron_golem_crackiness(74.9, 100.0), IronGolemCrackiness::Low);
    assert_eq!(
        iron_golem_crackiness(49.9, 100.0),
        IronGolemCrackiness::Medium
    );
    assert_eq!(
        iron_golem_crackiness(24.9, 100.0),
        IronGolemCrackiness::High
    );
}

fn assert_iron_golem_player_created_flags() {
    let mut golem = IronGolemState::new();
    assert!(!golem.is_player_created());
    golem.set_player_created(true);
    assert!(golem.saved_player_created());
    assert!(IronGolemState::from_saved_player_created(true).is_player_created());
    assert!(!IronGolemState::from_saved_player_created(false).is_player_created());
    assert!(iron_golem_block_summon_sets_player_created(
        "minecraft:carved_pumpkin"
    ));
    assert!(iron_golem_block_summon_sets_player_created(
        "minecraft:jack_o_lantern"
    ));
    assert!(!iron_golem_block_summon_sets_player_created(
        "minecraft:pumpkin"
    ));
}

fn assert_iron_golem_flower_events_and_repair() {
    let mut golem = IronGolemState::new();
    assert_eq!(golem.offer_flower(true), 11);
    assert_eq!(golem.offer_flower_tick, 400);
    golem.ai_step();
    assert_eq!(golem.offer_flower_tick, 399);
    golem.handle_entity_event(34);
    assert_eq!(golem.offer_flower_tick, 0);
    golem.handle_entity_event(11);
    assert_eq!(golem.offer_flower_tick, 400);
    golem.handle_entity_event(4);
    assert_eq!(golem.attack_animation_tick, 10);

    golem.health = 80.0;
    assert_eq!(
        golem.repair_with_iron_ingot("minecraft:iron_ingot"),
        IronGolemRepairResult::Repaired {
            consumed: 1,
            healed: 20.0,
            sound: "minecraft:entity.iron_golem.repair",
        }
    );
    assert_eq!(
        golem.repair_with_iron_ingot("minecraft:iron_ingot"),
        IronGolemRepairResult::Pass
    );
    assert_eq!(
        golem.repair_with_iron_ingot("minecraft:gold_ingot"),
        IronGolemRepairResult::Pass
    );
}

fn assert_iron_golem_attack_rules() {
    let attack = iron_golem_attack_plan(15.0, 14, 0.25);
    assert_eq!(
        attack,
        IronGolemAttackPlan {
            event_id: 4,
            attack_animation_tick: 10,
            damage: 21.5,
            target_delta_y: 0.3,
            sound: "minecraft:entity.iron_golem.attack",
        }
    );
    assert_eq!(iron_golem_attack_plan(15.0, 0, 1.0).damage, 7.5);
    assert_eq!(iron_golem_attack_plan(15.0, 0, 1.0).target_delta_y, 0.0);
    assert!(!iron_golem_can_attack_target(true, "minecraft:player"));
    assert!(iron_golem_can_attack_target(false, "minecraft:player"));
    assert!(!iron_golem_can_attack_target(false, "minecraft:creeper"));
    assert!(iron_golem_can_attack_target(true, "minecraft:zombie"));
}

#[test]
fn frog_variant_tongue_and_lay_spawn_match_java() {
    assert_eq!(
        FrogVariantModel::Temperate.registry_id(),
        "minecraft:temperate"
    );
    assert_eq!(
        FrogVariantModel::Warm.texture(),
        "minecraft:entity/frog/frog_warm"
    );
    assert_eq!(
        frog_variant_for_spawn_biome("minecraft:mangrove_swamp"),
        FrogVariantModel::Warm
    );
    assert_eq!(
        frog_variant_for_spawn_biome("minecraft:snowy_taiga"),
        FrogVariantModel::Cold
    );
    assert_eq!(
        frog_variant_for_spawn_biome("minecraft:swamp"),
        FrogVariantModel::Temperate
    );
    assert_eq!(
        frog_variant_for_spawn_biome("minecraft:the_end"),
        FrogVariantModel::Cold
    );

    assert!(frog_can_eat("minecraft:slime", Some(1)));
    assert!(!frog_can_eat("minecraft:slime", Some(2)));
    assert!(frog_can_eat("minecraft:magma_cube", Some(3)));
    assert!(!frog_can_eat("minecraft:zombie", None));

    assert_eq!(
        frog_tongue_catch_plan("minecraft:slime", Some(1), 1.0, false, "standing"),
        Some(FrogTongueCatchPlan {
            tongue_sound: "minecraft:entity.frog.tongue",
            eat_sound: "minecraft:entity.frog.eat",
            pose: "using_tongue",
            target_velocity_scale: 0.75,
            catch_animation_ticks: 6,
            eat_animation_ticks: 10,
            max_eating_distance: 1.75,
        })
    );
    assert_eq!(
        frog_tongue_catch_plan("minecraft:slime", Some(2), 1.0, false, "standing"),
        None
    );
    assert_eq!(
        frog_tongue_catch_plan("minecraft:magma_cube", None, 1.75, false, "standing"),
        None
    );
    assert_eq!(
        frog_tongue_catch_plan("minecraft:magma_cube", None, 1.0, true, "standing"),
        None
    );
    assert_eq!(
        frog_tongue_catch_plan("minecraft:magma_cube", None, 1.0, false, "croaking"),
        None
    );

    assert_eq!(
        frog_breeding_lay_spawn_plan(),
        FrogBreedingPlan {
            memory: "minecraft:is_pregnant",
            activity: "minecraft:lay_spawn",
            placed_block: "minecraft:frogspawn",
            land_search_radius: 8,
        }
    );
}

#[test]
fn fox_flags_variants_trust_items_berries_and_stalking_match_java() {
    assert_fox_flags_and_variants();
    assert_fox_state_flags_and_trusted_players();
    assert_fox_food_and_held_item_rules();
    assert_fox_breeding_rules();
    assert_fox_berry_harvest_rules();
    assert_fox_stalking_rules();
}

fn assert_fox_flags_and_variants() {
    assert_eq!(FOX_FLAG_SITTING, 1);
    assert_eq!(FOX_FLAG_CROUCHING, 4);
    assert_eq!(FOX_FLAG_INTERESTED, 8);
    assert_eq!(FOX_FLAG_POUNCING, 16);
    assert_eq!(FOX_FLAG_SLEEPING, 32);
    assert_eq!(FOX_FLAG_FACEPLANTED, 64);
    assert_eq!(FOX_FLAG_DEFENDING, 128);
    assert_eq!(FoxVariantModel::Red.id(), 0);
    assert_eq!(FoxVariantModel::Snow.serialized_name(), "snow");
    assert_eq!(fox_variant_by_id(99), FoxVariantModel::Red);
    assert_eq!(
        fox_variant_for_spawn_biome("minecraft:snowy_taiga"),
        FoxVariantModel::Snow
    );
    assert_eq!(
        fox_variant_for_spawn_biome("minecraft:taiga"),
        FoxVariantModel::Red
    );
}

fn assert_fox_state_flags_and_trusted_players() {
    let mut fox = FoxState::new();
    fox.set_flag(FOX_FLAG_SITTING, true);
    fox.set_flag(FOX_FLAG_SLEEPING, true);
    fox.set_flag(FOX_FLAG_FACEPLANTED, true);
    assert!(fox.is_sitting());
    assert!(fox.is_sleeping());
    assert!(fox.is_faceplanted());
    assert!(!fox.can_move());
    fox.clear_states();
    assert!(fox.can_move());
    fox.add_trusted("player_a");
    fox.add_trusted("player_b");
    fox.add_trusted("player_c");
    assert_eq!(fox.trusted, [Some("player_a"), Some("player_c")]);
    assert!(fox.trusts("player_c"));
    assert!(!fox.trusts("player_b"));
}

fn assert_fox_food_and_held_item_rules() {
    assert!(fox_food_item("minecraft:sweet_berries"));
    assert!(fox_food_item("minecraft:glow_berries"));
    assert!(!fox_food_item("minecraft:apple"));
    assert!(fox_can_eat_held_item(
        "minecraft:sweet_berries",
        false,
        true,
        false
    ));
    assert!(!fox_can_eat_held_item(
        "minecraft:sweet_berries",
        true,
        true,
        false
    ));
    assert_eq!(
        fox_can_hold_item(None, "minecraft:feather", 0),
        FoxHeldItemDecision::Hold
    );
    assert_eq!(
        fox_can_hold_item(Some("minecraft:feather"), "minecraft:sweet_berries", 1),
        FoxHeldItemDecision::ReplaceAndSpitOld
    );
    assert_eq!(
        fox_can_hold_item(Some("minecraft:feather"), "minecraft:leather", 1),
        FoxHeldItemDecision::Reject
    );
}

fn assert_fox_breeding_rules() {
    assert_eq!(
        fox_breed_plan(
            FoxVariantModel::Red,
            FoxVariantModel::Snow,
            false,
            Some("player_a"),
            Some("player_b")
        ),
        FoxBreedPlan {
            baby_variant: FoxVariantModel::Snow,
            trusted: [Some("player_a"), Some("player_b")],
            parent_age: 6000,
            baby_age: -24000,
            event_id: 18,
            xp_min: 1,
            xp_max_inclusive: 7,
        }
    );
    assert_eq!(
        fox_breed_plan(
            FoxVariantModel::Red,
            FoxVariantModel::Snow,
            true,
            Some("player_a"),
            Some("player_a")
        )
        .trusted,
        [Some("player_a"), None]
    );
}

fn assert_fox_berry_harvest_rules() {
    assert_eq!(
        fox_berry_harvest_plan("minecraft:sweet_berry_bush", 3, None, 1, true, false, 40),
        FoxBerryHarvestPlan::PickSweetBerries {
            held_item: Some("minecraft:sweet_berries"),
            dropped_count: 2,
            new_age: 1,
            sound: "minecraft:block.sweet_berry_bush.pick_berries",
            game_event: "minecraft:block_change",
        }
    );
    assert_eq!(
        fox_berry_harvest_plan(
            "minecraft:sweet_berry_bush",
            2,
            Some("minecraft:feather"),
            0,
            true,
            false,
            40
        ),
        FoxBerryHarvestPlan::PickSweetBerries {
            held_item: Some("minecraft:feather"),
            dropped_count: 1,
            new_age: 1,
            sound: "minecraft:block.sweet_berry_bush.pick_berries",
            game_event: "minecraft:block_change",
        }
    );
    assert_eq!(
        fox_berry_harvest_plan("minecraft:sweet_berry_bush", 3, None, 0, false, false, 40),
        FoxBerryHarvestPlan::None
    );
    assert_eq!(
        fox_berry_harvest_plan("minecraft:sweet_berry_bush", 3, None, 0, true, true, 40),
        FoxBerryHarvestPlan::None
    );
    assert_eq!(
        fox_berry_harvest_plan("minecraft:sweet_berry_bush", 1, None, 0, true, false, 40),
        FoxBerryHarvestPlan::None
    );
    assert_eq!(
        fox_berry_harvest_plan("minecraft:cave_vines", 1, None, 0, true, false, 40),
        FoxBerryHarvestPlan::PickGlowBerry
    );
}

fn assert_fox_stalking_rules() {
    assert_eq!(
        fox_stalk_prey_plan("minecraft:chicken", 49.0, false, false, false, false, true),
        Some(FoxStalkPlan {
            interested: true,
            crouching: true,
            move_speed: None,
        })
    );
    assert_eq!(
        fox_stalk_prey_plan("minecraft:rabbit", 49.0, false, false, false, false, false),
        Some(FoxStalkPlan {
            interested: false,
            crouching: false,
            move_speed: Some(1.5),
        })
    );
    assert_eq!(
        fox_stalk_prey_plan("minecraft:chicken", 36.0, false, false, false, false, true),
        None
    );
    assert_eq!(
        fox_stalk_prey_plan("minecraft:zombie", 49.0, false, false, false, false, true),
        None
    );
    assert_eq!(
        fox_stalk_prey_plan("minecraft:chicken", 49.0, true, false, false, false, true),
        None
    );
}

#[test]
fn panda_genes_flags_interactions_roll_and_sneeze_match_java() {
    assert_panda_constants_and_genes();
    assert_panda_state_flags_attributes_and_food();
    assert_panda_interactions();
    assert_panda_roll_step_rules();
    assert_panda_sneeze_rules();
}

fn assert_panda_constants_and_genes() {
    assert_eq!(PANDA_FLAG_SNEEZE, 2);
    assert_eq!(PANDA_FLAG_ROLL, 4);
    assert_eq!(PANDA_FLAG_SIT, 8);
    assert_eq!(PANDA_FLAG_ON_BACK, 16);
    assert_eq!(PANDA_EAT_TICK_INTERVAL, 5);
    assert_eq!(PANDA_TOTAL_ROLL_STEPS, 32);
    assert_eq!(PANDA_TOTAL_UNHAPPY_TIME, 32);

    assert_eq!(PandaGene::Brown.id(), 4);
    assert_eq!(PandaGene::Aggressive.serialized_name(), "aggressive");
    assert!(PandaGene::Brown.is_recessive());
    assert!(PandaGene::Weak.is_recessive());
    assert!(!PandaGene::Lazy.is_recessive());
    assert_eq!(panda_gene_by_id(99), PandaGene::Normal);
    assert_eq!(panda_random_gene(0), PandaGene::Lazy);
    assert_eq!(panda_random_gene(1), PandaGene::Worried);
    assert_eq!(panda_random_gene(2), PandaGene::Playful);
    assert_eq!(panda_random_gene(4), PandaGene::Aggressive);
    assert_eq!(panda_random_gene(8), PandaGene::Weak);
    assert_eq!(panda_random_gene(10), PandaGene::Brown);
    assert_eq!(panda_random_gene(15), PandaGene::Normal);
    assert_eq!(
        panda_variant_from_genes(PandaGene::Brown, PandaGene::Brown),
        PandaGene::Brown
    );
    assert_eq!(
        panda_variant_from_genes(PandaGene::Brown, PandaGene::Normal),
        PandaGene::Normal
    );
    assert_eq!(
        panda_variant_from_genes(PandaGene::Lazy, PandaGene::Brown),
        PandaGene::Lazy
    );
}

fn assert_panda_state_flags_attributes_and_food() {
    let mut panda = PandaState::new();
    panda.main_gene = PandaGene::Weak;
    panda.hidden_gene = PandaGene::Weak;
    panda.set_flag(PANDA_FLAG_SIT, true);
    panda.set_flag(PANDA_FLAG_ON_BACK, true);
    assert_eq!(panda.variant(), PandaGene::Weak);
    assert!(panda.is_sitting());
    assert!(panda.is_on_back());
    assert!(!panda.can_perform_action(false));
    panda.set_flag(PANDA_FLAG_SIT, false);
    panda.set_flag(PANDA_FLAG_ON_BACK, false);
    assert!(panda.can_perform_action(false));
    assert!(!panda.can_perform_action(true));

    assert_eq!(
        panda_attributes_for_variant(PandaGene::Lazy),
        PandaAttributes {
            movement_speed: 0.07,
            attack_damage: 6.0,
            max_health_override: None,
        }
    );
    assert_eq!(
        panda_attributes_for_variant(PandaGene::Weak).max_health_override,
        Some(10.0)
    );
    assert!(panda_food_item("minecraft:bamboo"));
    assert!(!panda_food_item("minecraft:cake"));
    assert!(panda_eats_from_ground_item("minecraft:bamboo"));
    assert!(panda_eats_from_ground_item("minecraft:cake"));
}

fn assert_panda_interactions() {
    assert_eq!(
        panda_interact_plan(PandaInteractInput {
            item: "minecraft:bamboo",
            scared: true,
            on_back: false,
            target_present: false,
            can_age_up: false,
            baby: false,
            age: 0,
            can_fall_in_love: false,
            sitting: false,
            in_water: false,
            current_held_item: None,
            player_infinite_materials: false,
        }),
        PandaInteractResult::Pass
    );
    assert_eq!(
        panda_interact_plan(PandaInteractInput {
            item: "minecraft:bamboo",
            scared: false,
            on_back: true,
            target_present: false,
            can_age_up: false,
            baby: false,
            age: 0,
            can_fall_in_love: false,
            sitting: false,
            in_water: false,
            current_held_item: None,
            player_infinite_materials: false,
        }),
        PandaInteractResult::Success { on_back: false }
    );
    assert_eq!(
        panda_interact_plan(PandaInteractInput {
            item: "minecraft:bamboo",
            scared: false,
            on_back: false,
            target_present: false,
            can_age_up: false,
            baby: false,
            age: 1,
            can_fall_in_love: false,
            sitting: false,
            in_water: false,
            current_held_item: Some("minecraft:cake"),
            player_infinite_materials: false,
        }),
        PandaInteractResult::SuccessServer {
            consumed: 1,
            sit: true,
            eat: true,
            held_item: Some("minecraft:bamboo"),
        }
    );
    assert_eq!(
        panda_interact_plan(PandaInteractInput {
            item: "minecraft:bamboo",
            scared: false,
            on_back: false,
            target_present: false,
            can_age_up: false,
            baby: true,
            age: 0,
            can_fall_in_love: false,
            sitting: false,
            in_water: false,
            current_held_item: None,
            player_infinite_materials: false,
        }),
        PandaInteractResult::Pass
    );
}

fn assert_panda_roll_step_rules() {
    let first_roll = panda_roll_step(0, false, 0.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), true);
    assert_eq!(
        first_roll,
        PandaRollStep {
            rolling_after_step: true,
            counter: 1,
            delta: (0.0, 0.27, 0.2),
        }
    );
    assert_eq!(
        panda_roll_step(6, false, 0.0, (0.1, 0.0, 0.2), (0.3, 0.0, 0.4), true),
        PandaRollStep {
            rolling_after_step: true,
            counter: 7,
            delta: (0.0, 0.27, 0.0),
        }
    );
    assert!(
        !panda_roll_step(32, false, 0.0, (0.1, 0.2, 0.3), (0.0, 0.0, 0.0), true).rolling_after_step
    );
}

fn assert_panda_sneeze_rules() {
    let mut sneezing = PandaState::new();
    sneezing.set_sneezing(true);
    assert_eq!(
        panda_sneeze_tick(&mut sneezing, true),
        PandaSneezeTick::PreSneezeSound
    );
    sneezing.sneeze_counter = 20;
    assert_eq!(
        panda_sneeze_tick(&mut sneezing, true),
        PandaSneezeTick::Finish {
            sound: "minecraft:entity.panda.sneeze",
            particle: "minecraft:sneeze",
            loot_table: "minecraft:gameplay/panda_sneeze",
        }
    );
    assert!(!sneezing.is_sneezing());
    assert_eq!(sneezing.sneeze_counter, 0);
    assert!(panda_sneeze_goal_can_use(true, true, true, true, false));
    assert!(panda_sneeze_goal_can_use(true, true, false, false, true));
    assert!(!panda_sneeze_goal_can_use(false, true, true, true, true));
    assert!(!panda_sneeze_goal_can_use(true, false, true, true, true));
}

include!("tests_a_parrot_sniffer.rs");
