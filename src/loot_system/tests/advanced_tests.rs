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
    let engine = fishing_loot_test_engine();
    assert_fishing_category_loot(&engine);
}

fn fishing_loot_test_engine() -> LootBehaviorEngine {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:gameplay/fishing/junk",
        table_with_pool(LootPool::single(LootEntry::item("minecraft:bowl", 1))),
    );
    engine.insert_table(
        "minecraft:gameplay/fishing/fish",
        table_with_pool(LootPool::single(LootEntry::Item {
            item: "minecraft:cod".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::MatchTool {
                item: "minecraft:fishing_rod".to_string(),
            }],
            functions: Vec::new(),
        })),
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
                    weight: 10,
                    quality: -2,
                    conditions: Vec::new(),
                },
                LootEntry::WeightedNestedTable {
                    table: "minecraft:gameplay/fishing/treasure".to_string(),
                    weight: 5,
                    quality: 2,
                    conditions: vec![
                        LootCondition::EntityProperty {
                            key: "this_entity".to_string(),
                            value: "FishingHook".to_string(),
                        },
                        LootCondition::EntityProperty {
                            key: "in_open_water".to_string(),
                            value: "true".to_string(),
                        },
                    ],
                },
                LootEntry::WeightedNestedTable {
                    table: "minecraft:gameplay/fishing/fish".to_string(),
                    weight: 85,
                    quality: -1,
                    conditions: Vec::new(),
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
    engine
}

fn assert_fishing_category_loot(engine: &LootBehaviorEngine) {
    let no_open_water = resolve_fishing_loot(
        engine,
        "minecraft:fishing_rod",
        (3.0, 62.0, 4.0),
        0.0,
        0.0,
        false,
        2,
    );
    assert_eq!(no_open_water.param_set, LootParamSet::Fishing);
    assert_eq!(
        no_open_water.delivery,
        LootDelivery::DropAt((3.0, 62.0, 4.0), vec![LootStack::new("minecraft:bowl", 1)])
    );

    let base_fish = resolve_fishing_loot(
        engine,
        "minecraft:fishing_rod",
        (3.0, 62.0, 4.0),
        0.0,
        0.0,
        false,
        1,
    );
    assert_eq!(
        base_fish.delivery,
        LootDelivery::DropAt((3.0, 62.0, 4.0), vec![LootStack::new("minecraft:cod", 1)])
    );

    let lucky_open_water = resolve_fishing_loot(
        engine,
        "minecraft:fishing_rod",
        (3.0, 62.0, 4.0),
        2.0,
        8.0,
        true,
        4,
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
                LootCondition::EntityProperty {
                    key: "this_entity".to_string(),
                    value: "Steve".to_string(),
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
                LootCondition::SurvivesExplosion,
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
    request.target_entity = Some("Steve".to_string());
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
    request.explosion_radius = Some(f32::MAX);
    assert_eq!(
        resolve_block_break_loot(&engine, request.clone(), 0x1234_5678_9abc_def0).delivery,
        LootDelivery::DropAt((1.0, 64.0, 2.0), Vec::new())
    );

    request.explosion_radius = Some(1.0);
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
    let mut context = java_item_modifier_context();
    let functions = java_item_modifier_sequence();
    let stack = functions
        .apply(LootStack::new("minecraft:raw_iron", 1), &mut context)
        .unwrap();

    assert_java_item_modifier_components(&stack);
    assert_copy_name_reads_block_entity_name(&mut context);
}

fn java_item_modifier_context() -> LootContext {
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
    context
}

fn java_item_modifier_sequence() -> LootFunction {
    LootFunction::Sequence(vec![
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
        LootFunction::SetEnchantments(HashMap::from([(
            "minecraft:sharpness".to_string(),
            2,
        )])),
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
        LootFunction::ModifyContents(vec![LootFunction::SetCount(NumberProvider::Constant(3.0))]),
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
        LootFunction::SetRandomPotion(vec!["minecraft:swiftness".to_string()]),
        LootFunction::SetStewEffects(vec!["minecraft:night_vision:160".to_string()]),
        LootFunction::SetRandomDyes(vec!["minecraft:red".to_string()]),
        LootFunction::SetWrittenBookPages(vec!["Draft".to_string()]),
        LootFunction::SetWritableBookPages(vec!["Writable".to_string()]),
        LootFunction::SetBookCover {
            title: "Cover".to_string(),
            author: "Sam".to_string(),
        },
        LootFunction::ToggleTooltips(vec!["minecraft:enchantments".to_string()]),
        LootFunction::SetFireworkExplosions(vec!["small_ball:red".to_string()]),
        LootFunction::SetFireworks {
            flight_duration: 2,
            explosions: vec!["small_ball:red".to_string()],
        },
        LootFunction::SetOminousBottleAmplifier(NumberProvider::Constant(3.0)),
        LootFunction::SetCustomModelData("model:17".to_string()),
        LootFunction::SetLootTable("minecraft:chests/simple_dungeon".to_string()),
        LootFunction::Reference("minecraft:set_marker".to_string()),
    ])
}

fn assert_java_item_modifier_components(stack: &LootStack) {
    assert_eq!(stack.item, "minecraft:filled_map");
    let expected = [
        ("minecraft:damage_fraction", "0.25"),
        ("minecraft:custom_data", "1b"),
        ("minecraft:enchantments", "minecraft:sharpness:2"),
        ("minecraft:custom_name", "Named"),
        ("minecraft:copied_name", "Dinnerbone"),
        ("minecraft:container", "minecraft:apple:3"),
        ("minecraft:profile", "Steve"),
        ("minecraft:block_state.facing", "north"),
        ("minecraft:attribute_modifiers", "generic.attack_damage:+1"),
        ("minecraft:banner_patterns", "minecraft:stripe_bottom"),
        ("minecraft:written_book_title", "Cover"),
        ("minecraft:written_book_author", "Sam"),
        ("minecraft:rarity", "rare"),
        ("minecraft:instrument", "minecraft:ponder_goat_horn"),
        ("minecraft:lore", "Lore"),
        ("minecraft:potion_contents", "minecraft:swiftness"),
        ("minecraft:suspicious_stew_effects", "minecraft:night_vision:160"),
        ("minecraft:dyed_color", "minecraft:red"),
        ("minecraft:written_book_pages", "Draft"),
        ("minecraft:writable_book_pages", "Writable"),
        ("minecraft:tooltip_hidden", "minecraft:enchantments"),
        ("minecraft:firework_explosion", "small_ball:red"),
        ("minecraft:fireworks", "2:small_ball:red"),
        ("minecraft:ominous_bottle_amplifier", "3"),
        ("minecraft:custom_model_data", "model:17"),
        ("minecraft:container_loot_table", "minecraft:chests/simple_dungeon"),
        ("minecraft:marker", "referenced"),
    ];
    for (key, value) in expected {
        assert_eq!(stack.components[key], value, "component {key}");
    }
}

#[test]
fn discard_function_removes_stack_like_java_discard_item() {
    let mut context = LootContext::new(LootParamSet::AllParams, 9);
    assert_eq!(
        LootFunction::Discard.apply(LootStack::new("minecraft:stick", 1), &mut context),
        None
    );
}

fn assert_copy_name_reads_block_entity_name(context: &mut LootContext) {
    let copied_name = LootFunction::CopyName {
        source: "block_entity_name".to_string(),
    }
    .apply(LootStack::new("minecraft:chest", 1), context)
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
    let engine = vault_loot_test_engine();

    let mut normal = active_vault();
    assert_normal_vault_unlock_sequence(&engine, &mut normal);

    let mut ominous = ominous_active_vault();
    assert_ominous_vault_unlock_sequence(&engine, &mut ominous);
}

fn vault_loot_test_engine() -> LootBehaviorEngine {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:chests/trial_chambers/reward",
        table_with_pool(LootPool::single(LootEntry::Item {
            item: "minecraft:emerald".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::MatchTool {
                item: "minecraft:trial_key".to_string(),
            }],
            functions: Vec::new(),
        })),
    );
    engine.insert_table(
        "minecraft:chests/trial_chambers/reward_ominous",
        table_with_pool(LootPool::single(LootEntry::Item {
            item: "minecraft:ominous_bottle".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::MatchTool {
                item: "minecraft:ominous_trial_key".to_string(),
            }],
            functions: Vec::new(),
        })),
    );
    engine
}

fn active_vault() -> VaultBlockEntity {
    VaultBlockEntity {
        state: crate::block_entity::VaultStateModel::Active,
        ..VaultBlockEntity::default()
    }
}

fn assert_normal_vault_unlock_sequence(engine: &LootBehaviorEngine, normal: &mut VaultBlockEntity) {
    assert_eq!(
        resolve_vault_unlock_loot(
            engine,
            normal,
            VaultUnlockLootRequest::new(
                "player-a",
                "minecraft:ominous_trial_key",
                (0.0, 64.0, 0.0),
                0.0,
                9,
                20,
            ),
        ),
        VaultInsertResult::WrongKey {
            expected: "minecraft:trial_key".to_string()
        }
    );
    assert_eq!(
        resolve_vault_unlock_loot(
            engine,
            normal,
            VaultUnlockLootRequest::new(
                "player-a",
                "minecraft:trial_key",
                (0.0, 64.0, 0.0),
                0.0,
                9,
                40,
            ),
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
            engine,
            normal,
            VaultUnlockLootRequest::new(
                "player-a",
                "minecraft:trial_key",
                (0.0, 64.0, 0.0),
                0.0,
                9,
                60,
            ),
        ),
        VaultInsertResult::AlreadyRewarded
    );
}

fn ominous_active_vault() -> VaultBlockEntity {
    VaultBlockEntity {
        state: crate::block_entity::VaultStateModel::Active,
        is_ominous: true,
        config: crate::block_entity::VaultConfigModel {
            key_item: PotItemStack {
                item_id: "minecraft:ominous_trial_key".to_string(),
                count: 1,
            },
            loot_table: "minecraft:chests/trial_chambers/reward_ominous".to_string(),
            ..crate::block_entity::VaultConfigModel::default()
        },
        ..VaultBlockEntity::default()
    }
}

fn assert_ominous_vault_unlock_sequence(
    engine: &LootBehaviorEngine,
    ominous: &mut VaultBlockEntity,
) {
    assert_eq!(
        resolve_vault_unlock_loot(
            engine,
            ominous,
            VaultUnlockLootRequest::new(
                "player-b",
                "minecraft:ominous_trial_key",
                (0.0, 64.0, 0.0),
                0.0,
                10,
                80,
            ),
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

#[cfg_attr(
    not(vibecraft_has_decompiled_sources),
    ignore = "requires optional Java source root"
)]
#[test]
fn mob_gift_loot_covers_cat_villager_and_wandering_trader_surfaces() {
    let living_entity_source = vibecraft_java_source!("/net/minecraft/world/entity/LivingEntity.java");
    let cat_source = vibecraft_java_source!("/net/minecraft/world/entity/animal/feline/Cat.java");
    let gift_to_hero_source = vibecraft_java_source!("/net/minecraft/world/entity/ai/behavior/GiveGiftToHero.java");
    let wandering_trader_source = vibecraft_java_source!("/net/minecraft/world/entity/npc/wanderingtrader/WanderingTrader.java");
    assert!(living_entity_source.contains("create(LootContextParamSets.GIFT)"));
    assert!(living_entity_source.contains("withParameter(LootContextParams.ORIGIN, this.position())"));
    assert!(living_entity_source.contains("withParameter(LootContextParams.THIS_ENTITY, this)"));
    assert!(cat_source.contains("BuiltInLootTables.CAT_MORNING_GIFT"));
    assert!(gift_to_hero_source.contains("BuiltInLootTables.BABY_VILLAGER_GIFT"));
    assert!(gift_to_hero_source.contains("GIFTS.getOrDefault(profession.get(), BuiltInLootTables.UNEMPLOYED_GIFT)"));
    assert!(wandering_trader_source.contains("this.addOffersFromTradeSet(level, offers, TradeSets.WANDERING_TRADER_BUYING)"));
    assert!(wandering_trader_source.contains("this.addOffersFromTradeSet(level, offers, TradeSets.WANDERING_TRADER_UNCOMMON)"));
    assert!(wandering_trader_source.contains("this.addOffersFromTradeSet(level, offers, TradeSets.WANDERING_TRADER_COMMON)"));

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

    if std::env::var_os("VIBECRAFT_VANILLA_DATA_ROOT").is_none() {
        eprintln!(
            "skipping wandering-trader offer resource parity: VIBECRAFT_VANILLA_DATA_ROOT is not set"
        );
        return;
    }
    let trader_offers = wandering_trader_reward_offers();
    assert!(!trader_offers.buying.is_empty());
    assert!(!trader_offers.uncommon.is_empty());
    assert!(!trader_offers.common.is_empty());
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

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn loot_table_resources_decode_all_vanilla_tables() {
    let Some(source_root) = option_env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT") else {
        unreachable!("test is gated on vibecraft_has_decompiled_sources");
    };
    let root = std::path::PathBuf::from(source_root)
    .join("data")
    .join("minecraft")
    .join("loot_table");
    let mut paths = Vec::new();
    collect_json_paths(&root, &mut paths);
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
