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
        assert_eq!(golem.saved_pumpkin(), true);
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

        assert_eq!(
            panda_interact_plan(
                "minecraft:bamboo",
                true,
                false,
                false,
                false,
                false,
                0,
                false,
                false,
                false,
                None,
                false
            ),
            PandaInteractResult::Pass
        );
        assert_eq!(
            panda_interact_plan(
                "minecraft:bamboo",
                false,
                true,
                false,
                false,
                false,
                0,
                false,
                false,
                false,
                None,
                false
            ),
            PandaInteractResult::Success { on_back: false }
        );
        assert_eq!(
            panda_interact_plan(
                "minecraft:bamboo",
                false,
                false,
                false,
                false,
                false,
                1,
                false,
                false,
                false,
                Some("minecraft:cake"),
                false
            ),
            PandaInteractResult::SuccessServer {
                consumed: 1,
                sit: true,
                eat: true,
                held_item: Some("minecraft:bamboo"),
            }
        );
        assert_eq!(
            panda_interact_plan(
                "minecraft:bamboo",
                false,
                false,
                false,
                false,
                true,
                0,
                false,
                false,
                false,
                None,
                false
            ),
            PandaInteractResult::Pass
        );

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
        assert_eq!(
            panda_roll_step(32, false, 0.0, (0.1, 0.2, 0.3), (0.0, 0.0, 0.0), true)
                .rolling_after_step,
            false
        );

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

    #[test]
    fn parrot_variants_taming_poison_party_shoulder_and_mimic_match_java() {
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

        assert_eq!(
            parrot_shoulder_plan(
                true, false, false, false, false, false, 101, false, false, true, false, true,
            ),
            ParrotShoulderPlan {
                can_use_goal: true,
                can_mount_now: true,
            }
        );
        assert_eq!(
            parrot_shoulder_plan(
                true, false, false, false, false, false, 100, false, false, true, false, true,
            ),
            ParrotShoulderPlan {
                can_use_goal: false,
                can_mount_now: false,
            }
        );
        assert_eq!(
            parrot_shoulder_plan(
                true, true, false, false, false, false, 101, false, false, true, false, true,
            ),
            ParrotShoulderPlan {
                can_use_goal: false,
                can_mount_now: false,
            }
        );
        assert_eq!(
            parrot_shoulder_plan(
                true, false, false, false, false, false, 101, false, false, true, true, true,
            )
            .can_mount_now,
            false
        );
        assert_eq!(
            parrot_shoulder_plan(
                true, false, false, false, false, false, 101, false, false, true, false, false,
            )
            .can_mount_now,
            false
        );
    }

    #[test]
    fn happy_ghast_harness_riding_leash_healing_and_dried_block_match_java() {
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
        assert!(sniffer_can_dig_body_state(
            false, false, false, false, true, false, true, false, true,
        ));
        assert!(!sniffer_can_dig_body_state(
            false, false, true, false, true, false, true, false, true,
        ));
        assert!(!sniffer_can_dig_body_state(
            false, false, false, false, true, false, true, true, true,
        ));
        assert!(!sniffer_can_dig_body_state(
            false, false, false, false, true, false, true, false, false,
        ));

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
        assert_eq!(
            sniffer_digging_tick_plan(SnifferStateModel::Searching, 1120, 1120, 1800, true)
                .drop_seed,
            false
        );

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

