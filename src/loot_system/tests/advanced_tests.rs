use super::super::*;
use super::*;

#[test]
fn piglin_barter_accepts_gold_and_uses_barter_context() {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:gameplay/piglin_bartering",
        table_with_pool(LootPool::single(LootEntry::Item {
            item: "minecraft:quartz".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::EntityProperty {
                key: "this_entity".to_string(),
                value: "Piglin".to_string(),
            }],
            functions: Vec::new(),
        })),
    );

    assert!(resolve_piglin_barter_loot(
        &engine,
        "Piglin",
        "minecraft:iron_ingot",
        (0.0, 64.0, 0.0),
        7,
    )
    .is_none());

    let resolution = resolve_piglin_barter_loot(
        &engine,
        "Piglin",
        "minecraft:gold_ingot",
        (4.0, 65.0, 4.0),
        7,
    )
    .unwrap();

    assert_eq!(resolution.param_set, LootParamSet::Barter);
    assert_eq!(
        resolution.delivery,
        LootDelivery::GiveToEntity(
            "Piglin".to_string(),
            vec![LootStack::new("minecraft:quartz", 1)]
        )
    );
}

#[test]
fn fishing_loot_uses_tool_origin_luck_and_category_tables() {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:gameplay/fishing/junk",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:bowl", 1))),
    );
    engine.insert_table(
        "minecraft:gameplay/fishing/fish",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:cod", 1))),
    );
    engine.insert_table(
        "minecraft:gameplay/fishing/treasure",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:name_tag", 1))),
    );
    let fishing_table = LootTable {
        param_set: LootParamSet::Fishing,
        random_sequence: Some("minecraft:gameplay/fishing".to_string()),
        pools: vec![LootPool {
            entries: vec![
                LootEntry::WeightedNestedTable {
                    table: "minecraft:gameplay/fishing/junk".to_string(),
                    weight: 1,
                    quality: -1,
                    conditions: Vec::new(),
                },
                LootEntry::WeightedNestedTable {
                    table: "minecraft:gameplay/fishing/fish".to_string(),
                    weight: 0,
                    quality: 0,
                    conditions: Vec::new(),
                },
                LootEntry::WeightedNestedTable {
                    table: "minecraft:gameplay/fishing/treasure".to_string(),
                    weight: 1,
                    quality: 2,
                    conditions: vec![LootCondition::EntityProperty {
                        key: "in_open_water".to_string(),
                        value: "true".to_string(),
                    }],
                },
            ],
            conditions: Vec::new(),
            functions: Vec::new(),
            rolls: NumberProvider::Constant(1.0),
            bonus_rolls: NumberProvider::Constant(0.0),
        }],
        functions: Vec::new(),
    };
    engine.insert_table("minecraft:gameplay/fishing", fishing_table);

    let no_open_water = resolve_fishing_loot(
        &engine,
        "minecraft:fishing_rod",
        (3.0, 62.0, 4.0),
        0.0,
        0.0,
        false,
        3,
    );
    assert_eq!(no_open_water.param_set, LootParamSet::Fishing);
    assert_eq!(
        no_open_water.delivery,
        LootDelivery::DropAt((3.0, 62.0, 4.0), vec![LootStack::new("minecraft:bowl", 1)])
    );

    let lucky_open_water = resolve_fishing_loot(
        &engine,
        "minecraft:fishing_rod",
        (3.0, 62.0, 4.0),
        2.0,
        8.0,
        true,
        3,
    );
    assert_eq!(
        lucky_open_water.delivery,
        LootDelivery::DropAt(
            (3.0, 62.0, 4.0),
            vec![LootStack::new("minecraft:name_tag", 1)]
        )
    );
}

#[test]
fn block_break_loot_uses_block_entity_tool_gamerule_silk_and_fortune() {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:blocks/diamond_ore",
        table_with_pool(LootPool::single(LootEntry::Item {
            item: "minecraft:diamond".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![
                LootCondition::BlockState {
                    block: "minecraft:diamond_ore".to_string(),
                },
                LootCondition::MatchTool {
                    item: "minecraft:diamond_pickaxe".to_string(),
                },
                LootCondition::EntityProperty {
                    key: "block_entity".to_string(),
                    value: "minecraft:test_block_entity".to_string(),
                },
                LootCondition::AnyOf(vec![
                    LootCondition::EntityProperty {
                        key: "correct_tool".to_string(),
                        value: "true".to_string(),
                    },
                    LootCondition::EntityProperty {
                        key: "silk_touch".to_string(),
                        value: "true".to_string(),
                    },
                ]),
            ],
            functions: vec![LootFunction::ApplyFortuneBonus {
                per_level: NumberProvider::Constant(1.0),
                limit: Some(4),
            }],
        })),
    );

    let mut request = LootRequest::new(LootSurface::BlockBreak, "minecraft:blocks/diamond_ore");
    request.origin = (1.0, 64.0, 2.0);
    request.block = Some("minecraft:diamond_ore".to_string());
    request.block_entity = Some("minecraft:test_block_entity".to_string());
    request.tool = Some("minecraft:diamond_pickaxe".to_string());
    request.fortune_level = 3;

    let resolution = resolve_block_break_loot(&engine, request.clone(), 5);
    assert_eq!(
        resolution.delivery,
        LootDelivery::DropAt(
            (1.0, 64.0, 2.0),
            vec![LootStack::new("minecraft:diamond", 4)]
        )
    );

    request.do_tile_drops = false;
    assert_eq!(
        resolve_block_break_loot(&engine, request.clone(), 5).delivery,
        LootDelivery::DropAt((1.0, 64.0, 2.0), Vec::new())
    );

    request.do_tile_drops = true;
    request.correct_tool = false;
    request.silk_touch = true;
    assert_eq!(
        resolve_block_break_loot(&engine, request, 5).delivery,
        LootDelivery::DropAt(
            (1.0, 64.0, 2.0),
            vec![LootStack::new("minecraft:diamond", 4)]
        )
    );
}

#[test]
fn component_loot_functions_apply_java_item_modifier_surface() {
    let mut context = LootContext::new(LootParamSet::AllParams, 21);
    context.block_on_fire = true;
    context.smelting_results.insert(
        "minecraft:raw_iron".to_string(),
        "minecraft:iron_ingot".to_string(),
    );
    context
        .entity_properties
        .insert("block_entity_name".to_string(), "Loot Chest".to_string());
    context
        .entity_properties
        .insert("last_damage_player".to_string(), "Steve".to_string());
    context
        .block_state_properties
        .insert("facing".to_string(), "north".to_string());
    context.context_nbt.insert(
        "this_entity".to_string(),
        HashMap::from([("CustomName".to_string(), "Dinnerbone".to_string())]),
    );
    context.function_references.insert(
        "minecraft:set_marker".to_string(),
        vec![LootFunction::SetComponents(HashMap::from([(
            "minecraft:marker".to_string(),
            "referenced".to_string(),
        )]))],
    );

    let functions = LootFunction::Sequence(vec![
        LootFunction::SmeltItem,
        LootFunction::SetDamage(NumberProvider::Constant(0.25)),
        LootFunction::SetNbt(HashMap::from([(
            "minecraft:custom_data".to_string(),
            "1b".to_string(),
        )])),
        LootFunction::EnchantWithLevels {
            levels: NumberProvider::Constant(3.0),
            options: vec!["minecraft:fortune".to_string()],
        },
        LootFunction::CopyName {
            source: "block_entity_name".to_string(),
        },
        LootFunction::CopyNbt {
            provider: NbtProvider::Context {
                target: "this_entity".to_string(),
                path: "CustomName".to_string(),
            },
            target: "minecraft:copied_name".to_string(),
        },
        LootFunction::SetContents(vec![LootStack::new("minecraft:apple", 2)]),
        LootFunction::ExplorationMap {
            destination: "minecraft:village".to_string(),
            decoration: "red_x".to_string(),
        },
        LootFunction::FillPlayerHead,
        LootFunction::CopyState(vec!["facing".to_string()]),
        LootFunction::SetAttributes(vec!["generic.attack_damage:+1".to_string()]),
        LootFunction::SetBannerPatterns(vec!["minecraft:stripe_bottom".to_string()]),
        LootFunction::SetBookContents {
            title: "Guide".to_string(),
            author: "Alex".to_string(),
            pages: vec!["Page 1".to_string()],
        },
        LootFunction::SetComponents(HashMap::from([(
            "minecraft:rarity".to_string(),
            "rare".to_string(),
        )])),
        LootFunction::SetInstrument("minecraft:ponder_goat_horn".to_string()),
        LootFunction::SetLore(vec!["Lore".to_string()]),
        LootFunction::SetName("Named".to_string()),
        LootFunction::SetPotion("minecraft:healing".to_string()),
        LootFunction::SetStewEffects(vec!["minecraft:night_vision:160".to_string()]),
        LootFunction::SetWrittenBookPages(vec!["Draft".to_string()]),
        LootFunction::ToggleTooltips(vec!["minecraft:enchantments".to_string()]),
        LootFunction::SetFireworkExplosions(vec!["small_ball:red".to_string()]),
        LootFunction::SetFireworks {
            flight_duration: 2,
            explosions: vec!["small_ball:red".to_string()],
        },
        LootFunction::Reference("minecraft:set_marker".to_string()),
    ]);

    let stack = functions
        .apply(LootStack::new("minecraft:raw_iron", 1), &mut context)
        .unwrap();

    assert_eq!(stack.item, "minecraft:filled_map");
    assert_eq!(stack.components["minecraft:damage_fraction"], "0.25");
    assert_eq!(stack.components["minecraft:custom_data"], "1b");
    assert_eq!(
        stack.components["minecraft:enchantments"],
        "minecraft:fortune:3"
    );
    assert_eq!(stack.components["minecraft:custom_name"], "Named");
    assert_eq!(stack.components["minecraft:copied_name"], "Dinnerbone");
    assert_eq!(stack.components["minecraft:container"], "minecraft:apple:2");
    assert_eq!(stack.components["minecraft:profile"], "Steve");
    assert_eq!(stack.components["minecraft:block_state.facing"], "north");
    assert_eq!(
        stack.components["minecraft:attribute_modifiers"],
        "generic.attack_damage:+1"
    );
    assert_eq!(
        stack.components["minecraft:banner_patterns"],
        "minecraft:stripe_bottom"
    );
    assert_eq!(stack.components["minecraft:written_book_title"], "Guide");
    assert_eq!(stack.components["minecraft:rarity"], "rare");
    assert_eq!(
        stack.components["minecraft:instrument"],
        "minecraft:ponder_goat_horn"
    );
    assert_eq!(stack.components["minecraft:lore"], "Lore");
    assert_eq!(
        stack.components["minecraft:potion_contents"],
        "minecraft:healing"
    );
    assert_eq!(
        stack.components["minecraft:suspicious_stew_effects"],
        "minecraft:night_vision:160"
    );
    assert_eq!(stack.components["minecraft:writable_book_pages"], "Draft");
    assert_eq!(
        stack.components["minecraft:tooltip_hidden"],
        "minecraft:enchantments"
    );
    assert_eq!(
        stack.components["minecraft:firework_explosion"],
        "small_ball:red"
    );
    assert_eq!(stack.components["minecraft:fireworks"], "2:small_ball:red");
    assert_eq!(stack.components["minecraft:marker"], "referenced");

    let copied_name = LootFunction::CopyName {
        source: "block_entity_name".to_string(),
    }
    .apply(LootStack::new("minecraft:chest", 1), &mut context)
    .unwrap();
    assert_eq!(
        copied_name.components["minecraft:custom_name"],
        "Loot Chest"
    );
}

#[test]
fn apply_bonus_function_covers_uniform_binomial_and_ore_drop_formulas() {
    let mut uniform = LootContext::new(LootParamSet::Block, 1);
    uniform.fortune_level = 2;
    let uniform_stack = LootFunction::ApplyBonus(LootBonusFormula::UniformBonusCount {
        bonus_multiplier: 2,
    })
    .apply(LootStack::new("minecraft:lapis_lazuli", 1), &mut uniform)
    .unwrap();
    assert!((1..=5).contains(&uniform_stack.count));

    let mut binomial = LootContext::new(LootParamSet::Block, 2);
    binomial.fortune_level = 3;
    let binomial_stack = LootFunction::ApplyBonus(LootBonusFormula::BinomialWithBonusCount {
        extra: 1,
        probability: 1.0,
    })
    .apply(LootStack::new("minecraft:redstone", 1), &mut binomial)
    .unwrap();
    assert_eq!(binomial_stack.count, 5);

    let mut ore = LootContext::new(LootParamSet::Block, 3);
    ore.fortune_level = 3;
    let ore_stack = LootFunction::ApplyBonus(LootBonusFormula::OreDrops)
        .apply(LootStack::new("minecraft:diamond", 2), &mut ore)
        .unwrap();
    assert!(ore_stack.count >= 2);
}

#[test]
fn vault_loot_resolves_normal_and_ominous_tables_once_per_player() {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:trial_chambers/reward",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:emerald", 1))),
    );
    engine.insert_table(
        "minecraft:trial_chambers/reward_ominous",
        table_with_pool(LootPool::single(LootEntry::item(
            "minecraft:ominous_bottle",
            1,
        ))),
    );

    let mut normal = VaultBlockEntity::default();
    normal.state = crate::block_entity::VaultStateModel::Active;
    assert_eq!(
        resolve_vault_unlock_loot(
            &engine,
            &mut normal,
            "player-a",
            "minecraft:ominous_trial_key",
            (0.0, 64.0, 0.0),
            9,
            20,
        ),
        VaultInsertResult::WrongKey {
            expected: "minecraft:trial_key".to_string()
        }
    );
    assert_eq!(
        resolve_vault_unlock_loot(
            &engine,
            &mut normal,
            "player-a",
            "minecraft:trial_key",
            (0.0, 64.0, 0.0),
            9,
            40,
        ),
        VaultInsertResult::Unlocking { items_to_eject: 1 }
    );
    assert_eq!(
        normal.items_to_eject,
        vec![PotItemStack {
            item_id: "minecraft:emerald".to_string(),
            count: 1
        }]
    );
    assert_eq!(
        resolve_vault_unlock_loot(
            &engine,
            &mut normal,
            "player-a",
            "minecraft:trial_key",
            (0.0, 64.0, 0.0),
            9,
            60,
        ),
        VaultInsertResult::AlreadyRewarded
    );

    let mut ominous = VaultBlockEntity::default();
    ominous.state = crate::block_entity::VaultStateModel::Active;
    ominous.is_ominous = true;
    ominous.config.key_item.item_id = "minecraft:ominous_trial_key".to_string();
    assert_eq!(
        resolve_vault_unlock_loot(
            &engine,
            &mut ominous,
            "player-b",
            "minecraft:ominous_trial_key",
            (0.0, 64.0, 0.0),
            10,
            80,
        ),
        VaultInsertResult::Unlocking { items_to_eject: 1 }
    );
    assert_eq!(
        ominous.items_to_eject,
        vec![PotItemStack {
            item_id: "minecraft:ominous_bottle".to_string(),
            count: 1
        }]
    );
}

#[test]
fn mob_gift_loot_covers_cat_villager_and_wandering_trader_surfaces() {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:gameplay/cat_morning_gift",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:string", 1))),
    );
    engine.insert_table(
        "minecraft:gameplay/hero_of_the_village/farmer_gift",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:bread", 1))),
    );
    engine.insert_table(
        "minecraft:gameplay/hero_of_the_village/baby_gift",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:poppy", 1))),
    );

    let cat = resolve_mob_gift_loot(
        &engine,
        "Cat",
        "minecraft:gameplay/cat_morning_gift",
        (5.0, 65.0, 6.0),
        12,
    );
    assert_eq!(cat.param_set, LootParamSet::Gift);
    assert_eq!(
        cat.delivery,
        LootDelivery::GiveToEntity(
            "Cat".to_string(),
            vec![LootStack::new("minecraft:string", 1)]
        )
    );

    let farmer_table = hero_of_the_village_gift_table(VillagerProfession::Farmer, false);
    let farmer = resolve_mob_gift_loot(&engine, "Farmer", farmer_table, (5.0, 65.0, 6.0), 13);
    assert_eq!(
        farmer.delivery,
        LootDelivery::GiveToEntity(
            "Farmer".to_string(),
            vec![LootStack::new("minecraft:bread", 1)]
        )
    );

    let baby_table = hero_of_the_village_gift_table(VillagerProfession::Toolsmith, true);
    let baby = resolve_mob_gift_loot(&engine, "BabyVillager", baby_table, (5.0, 65.0, 6.0), 14);
    assert_eq!(
        baby.delivery,
        LootDelivery::GiveToEntity(
            "BabyVillager".to_string(),
            vec![LootStack::new("minecraft:poppy", 1)]
        )
    );

    let trader_offers = wandering_trader_reward_offers();
    assert!(!trader_offers.generic.is_empty());
    assert!(!trader_offers.rare.is_empty());
}

#[test]
fn randomizable_container_loot_realizes_table_once_and_preserves_seed() {
    let mut engine = LootBehaviorEngine::new();
    let mut pool = LootPool::single(LootEntry::Item {
        item: "minecraft:bread".to_string(),
        weight: 1,
        quality: 0,
        conditions: Vec::new(),
        functions: vec![LootFunction::SetCount(NumberProvider::Uniform {
            min: 1.0,
            max: 4.0,
        })],
    });
    pool.rolls = NumberProvider::Constant(3.0);
    engine.insert_table("minecraft:chests/simple_dungeon", table_with_pool(pool));

    let mut first = RandomizableContainerLoot {
        loot_table: Some("minecraft:chests/simple_dungeon".to_string()),
        loot_table_seed: 42,
        unpacked: false,
    };
    let mut second = first.clone();

    let first_slots = first.unpack_once(&engine, (0.0, 64.0, 0.0)).unwrap();
    let second_slots = second.unpack_once(&engine, (0.0, 64.0, 0.0)).unwrap();

    assert_eq!(first_slots, second_slots);
    assert!(first.unpack_once(&engine, (0.0, 64.0, 0.0)).is_none());
}

#[test]
fn loot_table_resources_decode_all_vanilla_tables() {
    let root = std::path::Path::new("../decompiled-server-26.1.2/data/minecraft/loot_table");
    let mut paths = Vec::new();
    collect_json_paths(root, &mut paths);
    paths.sort();

    assert_eq!(paths.len(), 1326);
    let mut param_sets = HashSet::new();
    let mut nested_table_count = 0;
    let mut item_entry_count = 0;
    for path in &paths {
        let table = load_loot_table_resource(path).unwrap_or_else(|err| {
            panic!("{} failed to decode: {err}", path.display());
        });
        param_sets.insert(table.param_set);
        for pool in table.pools {
            assert!(
                !pool.entries.is_empty(),
                "{} has empty pool",
                path.display()
            );
            for entry in pool.entries {
                let entry_type = entry
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if entry_type == "minecraft:item" {
                    item_entry_count += 1;
                } else if entry_type == "minecraft:loot_table" {
                    nested_table_count += 1;
                }
            }
        }
    }

    assert!(param_sets.contains("minecraft:block"));
    assert!(param_sets.contains("minecraft:chest"));
    assert!(param_sets.contains("minecraft:entity"));
    assert!(param_sets.contains("minecraft:fishing"));
    assert!(item_entry_count > 1_000);
    assert!(nested_table_count > 0);
}

#[test]
fn loot_table_resource_defaults_match_java_direct_codec() {
    let table = parse_loot_table_resource("{}").unwrap();
    assert_eq!(table.param_set, "minecraft:all_params");
    assert_eq!(table.random_sequence, None);
    assert!(table.pools.is_empty());
    assert!(table.functions.is_empty());

    assert!(parse_loot_table_resource(r#"{"pools":[{"rolls":1.0}]}"#).is_err());
    assert!(parse_loot_table_resource(
        r#"{"pools":[{"rolls":1.0,"entries":[{"type":"minecraft:item"}]}]}"#
    )
    .is_err());
}

fn collect_json_paths(root: &std::path::Path, paths: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_json_paths(&path, paths);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
}
