use super::*;
#[cfg(vibecraft_has_decompiled_sources)]
use crate::villager_trade_resources::{
    load_villager_trade_data_root, profession_offers_from_resources,
    wandering_trader_offers_from_resources,
};
#[cfg(vibecraft_has_decompiled_sources)]
use std::collections::BTreeSet;
#[cfg(vibecraft_has_decompiled_sources)]
use std::fs;

#[test]
fn professions_expose_workstations_and_generate_level_offers() {
    let mut farmer = VillagerTradeState::new(VillagerProfession::Farmer);
    assert_eq!(farmer.profession.workstation(), Some("minecraft:composter"));
    assert!(farmer.assign_workstation());
    assert!(!farmer.assign_workstation());

    farmer.generate_level_offers();
    assert_eq!(farmer.offers.len(), 2);
    assert_eq!(farmer.offers[0].base_cost_a.item_id, "minecraft:wheat");
    assert_eq!(farmer.offers[0].base_cost_a.count, 20);
    assert_eq!(farmer.offers[0].result.item_id(), "minecraft:emerald");
    assert_eq!(farmer.offers[1].base_cost_a.item_id, "minecraft:potato");
    assert_eq!(farmer.offers[1].base_cost_a.count, 26);
    assert_eq!(farmer.offers[1].result.item_id(), "minecraft:emerald");

    farmer.release_workstation();
    assert!(farmer.assign_workstation());
}

#[test]
fn villager_profession_assignment_and_job_site_loss_match_java() {
    assert_villager_level_constants_match_java();
    assert_villager_profession_workstations_match_java();
    assert_villager_profession_acquisition_plans_match_java();
    assert_villager_job_site_loss_and_spawn_profession_match_java();
}

fn assert_villager_level_constants_match_java() {
    assert_eq!(VILLAGER_MIN_LEVEL, 1);
    assert_eq!(VILLAGER_MAX_LEVEL, 5);
    assert_eq!(VILLAGER_LEVEL_XP_THRESHOLDS, [0, 10, 70, 150, 250]);
    assert_eq!(
        (
            VILLAGER_ASSIGN_PROFESSION_CLEAR_TICK,
            VILLAGER_UNHAPPY_COUNTER_TICKS
        ),
        (true, 40)
    );
    assert_eq!(VILLAGER_LEVEL_UP_DELAY_TICKS, 40);
    assert_eq!(VILLAGER_LEVEL_UP_REGENERATION_TICKS, 200);
    assert_eq!(VillagerLevel::from_xp(9), VillagerLevel::Novice);
    assert_eq!(VillagerLevel::from_xp(10), VillagerLevel::Apprentice);
    assert_eq!(villager_min_xp_per_level(1), 0);
    assert_eq!(villager_max_xp_per_level(1), 10);
    assert_eq!(villager_max_xp_per_level(5), 0);
    assert!(villager_can_level_up(4));
    assert!(!villager_can_level_up(5));
}

fn assert_villager_profession_workstations_match_java() {
    assert_eq!(
        VillagerProfession::from_workstation("minecraft:lectern"),
        Some(VillagerProfession::Librarian)
    );
    assert_eq!(VillagerProfession::from_workstation("minecraft:bed"), None);
    assert!(VillagerProfession::None.acquirable_job_site_matches("minecraft:composter"));
    assert!(VillagerProfession::Farmer.held_job_site_matches("minecraft:composter"));
    assert!(!VillagerProfession::Farmer.held_job_site_matches("minecraft:lectern"));
    assert_eq!(
        VillagerProfession::Toolsmith.work_sound(),
        Some("minecraft:entity.villager.work_toolsmith")
    );
}

fn assert_villager_profession_acquisition_plans_match_java() {
    let none = VillagerDataModel::default_plains();
    assert_eq!(none.level, 1);
    assert_eq!(
        VillagerDataModel::new("minecraft:desert", VillagerProfession::Farmer, -5).level,
        1
    );

    assert_eq!(
        villager_acquire_profession_from_poi(none, "minecraft:lectern", true, false),
        VillagerProfessionPlan::Change {
            data: none.with_profession(VillagerProfession::Librarian),
            clear_offers: true,
            stop_trading: false,
            acquire_job_site: true,
            release_previous_job_site: false,
        }
    );
    assert_eq!(
        villager_acquire_profession_from_poi(none, "minecraft:lectern", false, false),
        VillagerProfessionPlan::Keep {
            data: none,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        }
    );
    assert_eq!(
        villager_acquire_profession_from_poi(none, "minecraft:lectern", true, true),
        VillagerProfessionPlan::Keep {
            data: none,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        }
    );
}

fn assert_villager_job_site_loss_and_spawn_profession_match_java() {
    let none = VillagerDataModel::default_plains();
    let librarian = none.with_profession(VillagerProfession::Librarian);
    assert_eq!(
        villager_set_profession_plan(librarian, VillagerProfession::Farmer, false),
        VillagerProfessionPlan::Change {
            data: librarian.with_profession(VillagerProfession::Farmer),
            clear_offers: true,
            stop_trading: false,
            acquire_job_site: true,
            release_previous_job_site: true,
        }
    );
    assert_eq!(
        villager_set_profession_plan(librarian, VillagerProfession::Librarian, false),
        VillagerProfessionPlan::Keep {
            data: librarian,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        }
    );
    assert_eq!(
        villager_lose_profession_after_job_site_removed(librarian, Some("minecraft:lectern"), true),
        VillagerProfessionPlan::Keep {
            data: librarian,
            clear_offers: false,
            stop_trading: false,
            release_job_site: false,
        }
    );
    assert_eq!(
        villager_lose_profession_after_job_site_removed(librarian, None, true),
        VillagerProfessionPlan::Change {
            data: librarian.with_profession(VillagerProfession::None),
            clear_offers: true,
            stop_trading: true,
            acquire_job_site: false,
            release_previous_job_site: true,
        }
    );
    assert_eq!(
        villager_finalize_spawn_profession(librarian, "breeding"),
        (librarian.with_profession(VillagerProfession::None), false)
    );
    assert_eq!(
        villager_finalize_spawn_profession(librarian, "structure"),
        (librarian, true)
    );
}

#[test]
fn villager_breeding_food_pickup_and_offspring_type_match_java() {
    assert_villager_breeding_constants_match_java();
    assert_villager_food_points_match_java();
    assert_villager_breeding_gates_match_java();
    assert_villager_food_digest_plan_matches_java();
    assert_villager_food_pickup_rules_match_java();
    assert_villager_offspring_type_rolls_match_java();
}

fn assert_villager_breeding_constants_match_java() {
    assert_eq!(VILLAGER_BREED_FOOD_THRESHOLD, 12);
    assert_eq!(VILLAGER_EXCESS_FOOD_THRESHOLD, 24);
    assert_eq!(VILLAGER_BREED_DIGEST_FOOD_POINTS, 12);
    assert_eq!(VILLAGER_OFFSPRING_BIOME_TYPE_CHANCE, 0.5);
    assert_eq!(VILLAGER_OFFSPRING_FIRST_PARENT_TYPE_CHANCE, 0.25);
}

fn assert_villager_food_points_match_java() {
    assert_eq!(villager_food_points("minecraft:bread"), Some(4));
    assert_eq!(villager_food_points("minecraft:potato"), Some(1));
    assert_eq!(villager_food_points("minecraft:carrot"), Some(1));
    assert_eq!(villager_food_points("minecraft:beetroot"), Some(1));
    assert_eq!(villager_food_points("minecraft:apple"), None);
    assert_eq!(
        villager_inventory_food_points(&[
            ("minecraft:bread", 2),
            ("minecraft:carrot", 3),
            ("minecraft:stone", 99),
        ]),
        11
    );
}

fn assert_villager_breeding_gates_match_java() {
    assert!(villager_can_breed(4, 8, false, 0));
    assert!(!villager_can_breed(4, 7, false, 0));
    assert!(!villager_can_breed(12, 0, true, 0));
    assert!(!villager_can_breed(12, 0, false, -24000));
    assert!(villager_hungry(11));
    assert!(!villager_hungry(12));
    assert!(villager_has_excess_food(24));
    assert!(!villager_has_excess_food(23));
    assert!(villager_wants_more_food(11));
    assert!(!villager_wants_more_food(12));
}

fn assert_villager_food_digest_plan_matches_java() {
    assert_eq!(
        villager_eat_and_digest_plan(
            5,
            &[
                ("minecraft:stone", 64),
                ("minecraft:bread", 2),
                ("minecraft:carrot", 10)
            ]
        ),
        VillagerEatPlan {
            food_level_after_eating: 13,
            consumed_items: 2,
            food_level_after_digest: 1,
        }
    );
    assert_eq!(
        villager_eat_and_digest_plan(12, &[("minecraft:bread", 1)]),
        VillagerEatPlan {
            food_level_after_eating: 12,
            consumed_items: 0,
            food_level_after_digest: 0,
        }
    );
}

fn assert_villager_food_pickup_rules_match_java() {
    assert!(villager_wants_to_pick_up(
        "minecraft:bread",
        VillagerProfession::None,
        true
    ));
    assert!(villager_wants_to_pick_up(
        "minecraft:wheat",
        VillagerProfession::Farmer,
        true
    ));
    assert!(!villager_wants_to_pick_up(
        "minecraft:wheat",
        VillagerProfession::Librarian,
        true
    ));
    assert!(!villager_wants_to_pick_up(
        "minecraft:bread",
        VillagerProfession::None,
        false
    ));
}

fn assert_villager_offspring_type_rolls_match_java() {
    assert_eq!(
        villager_offspring_type(
            "minecraft:savanna",
            "minecraft:plains",
            "minecraft:desert",
            0.49
        ),
        "minecraft:savanna"
    );
    assert_eq!(
        villager_offspring_type(
            "minecraft:savanna",
            "minecraft:plains",
            "minecraft:desert",
            0.5
        ),
        "minecraft:plains"
    );
    assert_eq!(
        villager_offspring_type(
            "minecraft:savanna",
            "minecraft:plains",
            "minecraft:desert",
            0.75
        ),
        "minecraft:desert"
    );
}

#[test]
fn villager_restock_timing_new_day_and_catchup_match_java() {
    assert_eq!(VILLAGER_RESTOCK_COOLDOWN_TICKS, 2400);
    assert_eq!(VILLAGER_RESTOCK_HALF_DAY_TICKS, 12000);
    assert_eq!(VILLAGER_MAX_RESTOCKS_PER_DAY, 2);

    let initial = VillagerRestockState::new();
    assert!(villager_allowed_to_restock(initial, 0));
    let first = villager_record_restock(initial, 100);
    assert_eq!(
        first,
        VillagerRestockState {
            restocks_today: 1,
            last_restock_game_time: 100,
            last_restock_check_day: 0,
        }
    );
    assert!(!villager_allowed_to_restock(first, 2500));
    assert!(villager_allowed_to_restock(first, 2501));
    let second = villager_record_restock(first, 2501);
    assert!(!villager_allowed_to_restock(second, 9999));

    assert_eq!(
        villager_should_restock_plan(first, 2501, 0, true),
        VillagerShouldRestockPlan {
            state: VillagerRestockState {
                restocks_today: 1,
                last_restock_game_time: 100,
                last_restock_check_day: 0,
            },
            should_restock: true,
            reset_for_new_day: false,
        }
    );
    assert!(!villager_should_restock_plan(first, 2501, 0, false).should_restock);
    assert_eq!(
        villager_should_restock_plan(second, 14_502, 0, true),
        VillagerShouldRestockPlan {
            state: VillagerRestockState {
                restocks_today: 0,
                last_restock_game_time: 14_502,
                last_restock_check_day: 0,
            },
            should_restock: true,
            reset_for_new_day: true,
        }
    );
    assert_eq!(
        villager_should_restock_plan(
            VillagerRestockState {
                restocks_today: 2,
                last_restock_game_time: 10_000,
                last_restock_check_day: 1,
            },
            10_100,
            2,
            true,
        ),
        VillagerShouldRestockPlan {
            state: VillagerRestockState {
                restocks_today: 0,
                last_restock_game_time: 10_100,
                last_restock_check_day: 2,
            },
            should_restock: true,
            reset_for_new_day: true,
        }
    );
    assert_eq!(
        villager_catch_up_demand_plan(0),
        VillagerCatchUpDemandPlan {
            missed_updates: 2,
            reset_uses: true,
            demand_updates: 2,
            resend_offers: true,
        }
    );
    assert_eq!(
        villager_catch_up_demand_plan(2),
        VillagerCatchUpDemandPlan {
            missed_updates: 0,
            reset_uses: false,
            demand_updates: 0,
            resend_offers: true,
        }
    );
}

#[test]
fn trading_adds_xp_gossip_demand_and_restock_caps_twice_per_day() {
    let mut villager = VillagerTradeState::new(VillagerProfession::Librarian);
    villager.generate_level_offers();
    villager.offers[0].max_uses = 1;
    villager.offers.push(MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 5),
        None,
        ItemStack::new("minecraft:compass", 1),
        4,
        1,
        0.05,
    ));
    let manual_offer_index = villager.offers.len() - 1;
    villager.offers[manual_offer_index].demand = 6;

    assert_eq!(
        villager.trade(0, "player-a").unwrap().item_id(),
        "minecraft:emerald"
    );
    assert!(villager.offers[0].is_out_of_stock());
    assert_eq!(villager.xp, 2);
    assert_eq!(villager.last_traded_player.as_deref(), Some("player-a"));
    assert_eq!(villager.gossip.reputation("player-a"), 2);

    assert!(villager.restock(1_000));
    assert!(!villager.offers[0].is_out_of_stock());
    assert_eq!(villager.offers[0].demand, 1);
    assert_eq!(villager.offers[manual_offer_index].demand, 2);
    assert!(!villager.restock(1_100));

    villager.trade(0, "player-a");
    assert!(villager.restock(1_200));
    villager.trade(0, "player-a");
    assert!(!villager.restock(1_300));
    assert!(villager.restock(25_000));
}

#[test]
fn trade_xp_grant_sets_delayed_profession_level_up_like_java() {
    let mut villager = VillagerTradeState::new(VillagerProfession::Toolsmith);
    villager.offers.push(MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 1),
        None,
        ItemStack::new("minecraft:stone_shovel", 1),
        4,
        10,
        0.2,
    ));
    villager.offers[0].reward_exp = false;

    // Java Villager.rewardTradeXp always adds offer XP to villager career XP,
    // even when rewardExp=false suppresses the visible experience orb.
    villager.trade(0, "player-a").unwrap();
    assert_eq!(villager.xp, 10);
    assert_eq!(villager.level, VillagerLevel::Novice);
    assert!(villager.should_increase_level());
    assert_eq!(
        villager.update_merchant_timer,
        VILLAGER_LEVEL_UP_DELAY_TICKS
    );
    assert!(villager.increase_profession_level_on_update);

    for _ in 0..VILLAGER_LEVEL_UP_DELAY_TICKS {
        assert!(!villager.tick_level_progression(true));
    }
    assert_eq!(
        villager.update_merchant_timer,
        VILLAGER_LEVEL_UP_DELAY_TICKS
    );
    assert_eq!(villager.level, VillagerLevel::Novice);

    for _ in 1..VILLAGER_LEVEL_UP_DELAY_TICKS {
        assert!(!villager.tick_level_progression(false));
    }
    assert_eq!(villager.level, VillagerLevel::Novice);
    assert!(villager.tick_level_progression(false));
    assert_eq!(villager.update_merchant_timer, 0);
    assert_eq!(villager.level, VillagerLevel::Apprentice);
    assert!(!villager.increase_profession_level_on_update);
    assert!(villager
        .offers
        .iter()
        .any(|offer| offer.result.item_id() == "minecraft:bell"));
    assert!(villager
        .offers
        .iter()
        .any(|offer| offer.result.item_id() == "minecraft:emerald"));
}

#[test]
fn reputation_changes_special_prices_and_level_unlocks_follow_xp_thresholds() {
    let mut villager = VillagerTradeState::new(VillagerProfession::Toolsmith);
    villager.xp = 10;
    villager.level = VillagerLevel::from_xp(villager.xp);
    villager.generate_level_offers();
    villager
        .gossip
        .add("player-a", GossipType::MajorPositive, 20);

    villager.apply_reputation_prices("player-a");

    assert_eq!(villager.level, VillagerLevel::Apprentice);
    let iron_for_emerald = villager
        .offers
        .iter()
        .find(|offer| offer.result.item_id() == "minecraft:emerald")
        .unwrap();
    assert_eq!(iron_for_emerald.special_price_diff, -5);
    assert_eq!(iron_for_emerald.cost_a_count(), 1);

    let bell = villager
        .offers
        .iter()
        .find(|offer| offer.result.item_id() == "minecraft:bell")
        .unwrap();
    assert_eq!(bell.special_price_diff, -20);
    assert_eq!(bell.cost_a_count(), 16);
}

#[test]
fn zombie_villager_cure_duration_and_cured_reputation_discount_values_match_java() {
    assert_eq!(
        crate::mob_interaction::zombie_villager_conversion_time_from_roll(0),
        3600
    );
    assert_eq!(
        crate::mob_interaction::zombie_villager_conversion_time_from_roll(2400),
        6000
    );
    assert!(
        crate::mob_interaction::zombie_villager_finish_conversion(true, true, false)
            .emit_reputation_event
    );

    let player = "curing-player";
    let mut villager = VillagerTradeState::new(VillagerProfession::Toolsmith);
    villager.xp = 10;
    villager.level = VillagerLevel::from_xp(villager.xp);
    villager.generate_level_offers();

    // Villager.java#onReputationEventFrom adds both cure gossips, and
    // updateSpecialPrices applies -floor(reputation * offer.priceMultiplier).
    villager.gossip.add(player, GossipType::MajorPositive, 20);
    villager.gossip.add(player, GossipType::MinorPositive, 25);
    assert_eq!(villager.gossip.reputation(player), 125);

    villager.apply_reputation_prices(player);

    let iron_for_emerald = villager
        .offers
        .iter()
        .find(|offer| offer.result.item_id() == "minecraft:emerald")
        .unwrap();
    assert_eq!(iron_for_emerald.price_multiplier, 0.05);
    assert_eq!(iron_for_emerald.special_price_diff, -6);
    assert_eq!(iron_for_emerald.cost_a_count(), 1);

    let bell = villager
        .offers
        .iter()
        .find(|offer| offer.result.item_id() == "minecraft:bell")
        .unwrap();
    assert_eq!(bell.price_multiplier, 0.2);
    assert_eq!(bell.special_price_diff, -25);
    assert_eq!(bell.cost_a_count(), 11);
}

#[test]
fn wandering_trader_spawn_data_updates_chance_and_sample_trade_groups() {
    let trades = WanderingTraderOffers::vanilla_sample();
    assert!(!trades.buying.is_empty());
    assert!(!trades.uncommon.is_empty());
    assert!(!trades.common.is_empty());

    let data = WanderingTraderData {
        spawn_delay: 1,
        spawn_chance: 50,
    };
    let spawned = wandering_trader_tick(data, true, true, 10);
    assert!(spawned.should_spawn);
    assert_eq!(spawned.next_data.spawn_chance, 25);

    let missed = wandering_trader_tick(data, true, true, 99);
    assert!(!missed.should_spawn);
    assert_eq!(missed.next_data.spawn_chance, 75);
}

#[test]
fn wandering_trader_spawner_timing_attempts_llamas_and_despawn_match_java() {
    assert_wandering_trader_spawner_constants_match_java();
    assert_wandering_trader_idle_and_waiting_ticks_match_java();
    assert_wandering_trader_spawn_success_tick_matches_java();
    assert_wandering_trader_failed_spawn_ticks_match_java();
    assert_wandering_trader_position_and_despawn_helpers_match_java();
}

fn assert_wandering_trader_spawner_constants_match_java() {
    assert_eq!(WANDERING_TRADER_SPAWNER_TICK_DELAY, 1200);
    assert_eq!(WANDERING_TRADER_MIN_SPAWN_CHANCE, 25);
    assert_eq!(WANDERING_TRADER_MAX_SPAWN_CHANCE, 75);
    assert_eq!(WANDERING_TRADER_SPAWN_CHANCE_INCREASE, 25);
    assert_eq!(WANDERING_TRADER_ONE_IN_X_CHANCE, 10);
    assert_eq!(WANDERING_TRADER_SPAWN_ATTEMPTS, 10);
    assert_eq!(WANDERING_TRADER_SPAWN_RADIUS, 48);
    assert_eq!(WANDERING_TRADER_MEETING_POI_RADIUS, 48);
    assert_eq!(WANDERING_TRADER_LLAMA_COUNT, 2);
    assert_eq!(WANDERING_TRADER_LLAMA_SPAWN_RADIUS, 4);
    assert_eq!(WANDERING_TRADER_DESPAWN_DELAY, 48_000);
    assert_eq!(WANDERING_TRADER_HOME_RADIUS, 16);
}

fn assert_wandering_trader_idle_and_waiting_ticks_match_java() {
    let idle = wandering_trader_spawner_tick(
        WanderingTraderSpawnerState::default(),
        WanderingTraderSpawnContext {
            spawn_traders_rule: false,
            random_player_present: true,
            outer_chance_roll_0_to_99: 0,
            one_in_ten_roll: 0,
            spawn_position_found: true,
            has_enough_space: true,
            biome_allows_spawn: true,
        },
    );
    assert_eq!(idle.state.tick_delay, 1200);
    assert!(!idle.spawn_trader);

    let waiting = wandering_trader_spawner_tick(
        WanderingTraderSpawnerState {
            tick_delay: 2,
            data: WanderingTraderData {
                spawn_delay: 24_000,
                spawn_chance: 25,
            },
        },
        WanderingTraderSpawnContext {
            spawn_traders_rule: true,
            random_player_present: true,
            outer_chance_roll_0_to_99: 0,
            one_in_ten_roll: 0,
            spawn_position_found: true,
            has_enough_space: true,
            biome_allows_spawn: true,
        },
    );
    assert_eq!(waiting.state.tick_delay, 1);
    assert_eq!(waiting.state.data.spawn_delay, 24_000);
}

fn assert_wandering_trader_spawn_success_tick_matches_java() {
    let spawned = wandering_trader_spawner_tick(
        WanderingTraderSpawnerState {
            tick_delay: 1,
            data: WanderingTraderData {
                spawn_delay: 1200,
                spawn_chance: 25,
            },
        },
        WanderingTraderSpawnContext {
            spawn_traders_rule: true,
            random_player_present: true,
            outer_chance_roll_0_to_99: 25,
            one_in_ten_roll: 0,
            spawn_position_found: true,
            has_enough_space: true,
            biome_allows_spawn: true,
        },
    );
    assert!(spawned.spawn_trader);
    assert!(spawned.returns_spawn_success);
    assert_eq!(spawned.spawn_llamas, 2);
    assert_eq!(spawned.trader_despawn_delay, Some(48_000));
    assert_eq!(spawned.trader_home_radius, Some(16));
    assert_eq!(spawned.state.tick_delay, 1200);
    assert_eq!(spawned.state.data.spawn_delay, 24_000);
    assert_eq!(spawned.state.data.spawn_chance, 25);
}

fn assert_wandering_trader_failed_spawn_ticks_match_java() {
    let missed_outer_roll = wandering_trader_spawner_tick(
        WanderingTraderSpawnerState {
            tick_delay: 1,
            data: WanderingTraderData {
                spawn_delay: 1200,
                spawn_chance: 25,
            },
        },
        WanderingTraderSpawnContext {
            spawn_traders_rule: true,
            random_player_present: true,
            outer_chance_roll_0_to_99: 26,
            one_in_ten_roll: 0,
            spawn_position_found: true,
            has_enough_space: true,
            biome_allows_spawn: true,
        },
    );
    assert!(!missed_outer_roll.spawn_trader);
    assert_eq!(missed_outer_roll.state.data.spawn_chance, 50);

    let no_player = wandering_trader_spawner_tick(
        WanderingTraderSpawnerState {
            tick_delay: 1,
            data: WanderingTraderData {
                spawn_delay: 1200,
                spawn_chance: 50,
            },
        },
        WanderingTraderSpawnContext {
            spawn_traders_rule: true,
            random_player_present: false,
            outer_chance_roll_0_to_99: 50,
            one_in_ten_roll: 9,
            spawn_position_found: false,
            has_enough_space: false,
            biome_allows_spawn: false,
        },
    );
    assert!(no_player.returns_spawn_success);
    assert!(!no_player.spawn_trader);
    assert_eq!(no_player.state.data.spawn_chance, 25);
}

fn assert_wandering_trader_position_and_despawn_helpers_match_java() {
    assert_eq!(wandering_trader_candidate_offset(0, 48), -48);
    assert_eq!(wandering_trader_candidate_offset(95, 48), 47);
    assert!(wandering_trader_has_enough_space(&[true; 12]));
    assert!(!wandering_trader_has_enough_space(&[true; 11]));
    let mut blocked = [true; 12];
    blocked[6] = false;
    assert!(!wandering_trader_has_enough_space(&blocked));
    assert_eq!(wandering_trader_should_despawn(2, false), (1, false));
    assert_eq!(wandering_trader_should_despawn(1, false), (0, true));
    assert_eq!(wandering_trader_should_despawn(1, true), (1, false));
    assert!(!wandering_trader_remove_when_far_away());
}

#[cfg(vibecraft_has_decompiled_sources)]
fn source_backed_vanilla_data_root() -> std::path::PathBuf {
    let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
        unreachable!("function is gated on vibecraft_has_decompiled_sources");
    };
    std::path::PathBuf::from(source_root)
    .join("data")
    .join("minecraft")
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn villager_trade_resources_decode_all_vanilla_entries() {
    let root = source_backed_vanilla_data_root().join("villager_trade");
    let mut paths = Vec::new();
    collect_json_paths(&root, &mut paths);
    paths.sort();

    assert_eq!(paths.len(), 387);
    let mut result_items = BTreeSet::new();
    let mut modifier_count = 0;
    let mut exploration_map_count = 0;
    let mut additional_cost_count = 0;
    for path in &paths {
        let trade = load_villager_trade_resource(path).unwrap_or_else(|err| {
            panic!("{} failed to decode: {err}", path.display());
        });
        assert!(trade.wants.item_id.starts_with("minecraft:"));
        assert!(trade.gives.item_id.starts_with("minecraft:"));
        assert!(trade.gives.count >= 1);
        if trade.additional_wants.is_some() {
            additional_cost_count += 1;
        }
        if !trade.given_item_modifiers.is_empty() {
            modifier_count += 1;
        }
        if trade.given_item_modifiers.iter().any(|modifier| {
            modifier.get("function").and_then(serde_json::Value::as_str)
                == Some("minecraft:exploration_map")
        }) {
            exploration_map_count += 1;
        }
        result_items.insert(trade.gives.item_id);
    }

    assert!(result_items.contains("minecraft:emerald"));
    assert!(result_items.contains("minecraft:enchanted_book"));
    assert!(result_items.contains("minecraft:map"));
    assert!(additional_cost_count > 0);
    assert!(modifier_count > 0);
    assert!(exploration_map_count > 0);
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn trade_set_resources_decode_all_vanilla_entries() {
    let root = source_backed_vanilla_data_root().join("trade_set");
    let mut paths = Vec::new();
    collect_json_paths(&root, &mut paths);
    paths.sort();

    assert_eq!(paths.len(), 68);
    let mut tag_count = 0;
    let mut wandering_sets = 0;
    for path in &paths {
        let trade_set = load_trade_set_resource(path).unwrap_or_else(|err| {
            panic!("{} failed to decode: {err}", path.display());
        });
        if matches!(trade_set.trades, HolderSetResource::Tag(_)) {
            tag_count += 1;
        }
        assert!(
            trade_set
                .random_sequence
                .as_deref()
                .is_some_and(|id| id.starts_with("minecraft:trade_set/")),
            "{} should define a namespaced random sequence",
            path.display()
        );
        if path.to_string_lossy().contains("wandering_trader") {
            wandering_sets += 1;
        }
    }

    assert_eq!(tag_count, 68);
    assert_eq!(wandering_sets, 3);
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn profession_offer_generation_resolves_vanilla_trade_sets_and_tags() {
    let resources = load_villager_trade_data_root(source_backed_vanilla_data_root()).unwrap();

    let farmer = profession_offers_from_resources(
        &resources,
        VillagerProfession::Farmer,
        VillagerLevel::Novice,
    )
    .unwrap();
    assert_eq!(farmer.len(), 2);
    assert_eq!(farmer[0].base_cost_a.item_id, "minecraft:wheat");
    assert_eq!(farmer[0].base_cost_a.count, 20);
    assert_eq!(farmer[0].result.item_id(), "minecraft:emerald");
    assert_eq!(farmer[0].max_uses, 16);
    assert_eq!(farmer[0].xp, 2);
    assert_eq!(farmer[1].base_cost_a.item_id, "minecraft:potato");
    assert_eq!(farmer[1].result.item_id(), "minecraft:emerald");

    let toolsmith = profession_offers_from_resources(
        &resources,
        VillagerProfession::Toolsmith,
        VillagerLevel::Apprentice,
    )
    .unwrap();
    assert_eq!(toolsmith.len(), 2);
    assert!(toolsmith
        .iter()
        .any(|offer| offer.result.item_id() == "minecraft:bell"));
    assert!(toolsmith
        .iter()
        .any(|offer| offer.result.item_id() == "minecraft:emerald"));
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn wandering_trader_offer_generation_resolves_vanilla_trade_sets() {
    let resources = load_villager_trade_data_root(source_backed_vanilla_data_root()).unwrap();
    let (buying, uncommon, common) = wandering_trader_offers_from_resources(&resources).unwrap();

    assert_eq!(buying.len(), 2);
    assert_eq!(uncommon.len(), 2);
    assert_eq!(common.len(), 5);
    assert!(buying
        .iter()
        .all(|offer| offer.result.item_id() == "minecraft:emerald"));
    assert!(uncommon.iter().any(|offer| offer.xp > 0));
    assert!(common.iter().any(|offer| offer.max_uses > 0));
}

#[test]
fn villager_trade_resource_defaults_match_java_codecs() {
    let trade = parse_villager_trade_resource(
        r#"{"wants":{"id":"minecraft:emerald"},"gives":{"id":"minecraft:bread"}}"#,
    )
    .unwrap();
    assert_eq!(trade.wants.count, NumberProviderResource::Constant(1.0));
    assert_eq!(trade.gives.count, 1);
    assert_eq!(trade.max_uses, NumberProviderResource::Constant(4.0));
    assert_eq!(
        trade.reputation_discount,
        NumberProviderResource::Constant(0.0)
    );
    assert_eq!(trade.xp, NumberProviderResource::Constant(1.0));

    let trade_set =
        parse_trade_set_resource(r##"{"trades":"#minecraft:farmer/level_1","amount":2.0}"##)
            .unwrap();
    assert_eq!(
        trade_set.trades,
        HolderSetResource::Tag("minecraft:farmer/level_1".to_string())
    );
    assert_eq!(trade_set.amount, NumberProviderResource::Constant(2.0));
    assert!(!trade_set.allow_duplicates);
    assert_eq!(trade_set.random_sequence, None);
}

#[test]
fn hero_of_the_village_discount_and_special_price_reset_match_java() {
    let mut villager = VillagerTradeState::new(VillagerProfession::Farmer);
    villager.offers.push(MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 1),
        None,
        ItemStack::new("minecraft:bread", 6),
        16,
        1,
        0.05,
    ));
    villager.offers.push(MerchantOffer::new(
        ItemCost::new("minecraft:wheat", 20),
        None,
        ItemStack::new("minecraft:emerald", 1),
        16,
        2,
        0.05,
    ));

    villager.apply_hero_of_the_village_prices(0);
    assert_eq!(villager.offers[0].special_price_diff, -1);
    assert_eq!(villager.offers[1].special_price_diff, -6);

    villager.apply_hero_of_the_village_prices(2);
    assert_eq!(villager.offers[0].special_price_diff, -2);
    assert_eq!(villager.offers[1].special_price_diff, -14);

    villager.reset_special_prices();
    assert_eq!(villager.offers[0].special_price_diff, 0);
    assert_eq!(villager.offers[1].special_price_diff, 0);
}

#[cfg(vibecraft_has_decompiled_sources)]
fn collect_json_paths(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_json_paths(&path, paths);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
}
