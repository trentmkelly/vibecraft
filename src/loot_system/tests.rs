use super::*;
use crate::advancement_system::{
    AdvancementDefinition, AdvancementRewards, PlayerAdvancementSet,
};
use crate::registry::Identifier;

fn table_with_pool(pool: LootPool) -> LootTable {
    LootTable {
        param_set: LootParamSet::AllParams,
        random_sequence: None,
        pools: vec![pool],
        functions: Vec::new(),
    }
}

#[test]
fn context_entity_types_params_and_dynamic_params_cover_java_surface() {
    let entity_types = [
        LootContextEntityType::Block,
        LootContextEntityType::Entity,
        LootContextEntityType::Chest,
        LootContextEntityType::Fishing,
        LootContextEntityType::Archaeology,
        LootContextEntityType::AdvancementReward,
        LootContextEntityType::Gift,
        LootContextEntityType::Barter,
        LootContextEntityType::Vault,
        LootContextEntityType::Command,
        LootContextEntityType::Selector,
        LootContextEntityType::AdvancementEntity,
        LootContextEntityType::Equipment,
    ];
    assert_eq!(entity_types.len(), 13);
    assert_eq!(
        LootSurface::Vault.entity_type(),
        LootContextEntityType::Vault
    );
    assert_eq!(
        LootContextEntityType::Equipment.param_set(),
        LootParamSet::Equipment
    );

    let mut params = LootParams::default();
    for value in [
        LootParamValue::BlockState("minecraft:stone".to_string()),
        LootParamValue::BlockEntity("Chest".to_string()),
        LootParamValue::Origin(1.0, 64.0, 2.0),
        LootParamValue::Tool("minecraft:diamond_pickaxe".to_string()),
        LootParamValue::ThisEntity("Zombie".to_string()),
        LootParamValue::LastDamagePlayer("Steve".to_string()),
        LootParamValue::KillerEntity("Steve".to_string()),
        LootParamValue::DirectKillerEntity("Arrow".to_string()),
        LootParamValue::ExplosionRadius(2.0),
        LootParamValue::DamageSource("minecraft:player_attack".to_string()),
    ] {
        params.insert(value);
    }
    assert_eq!(params.keys().len(), 10);
    assert!(matches!(
        params.get(LootParamKey::DamageSource),
        Some(LootParamValue::DamageSource(id)) if id == "minecraft:player_attack"
    ));

    let mut context = LootContext::new(LootParamSet::Entity, 4);
    context.insert_param(LootParamValue::ThisEntity("Zombie".to_string()));
    context.insert_param(LootParamValue::LastDamagePlayer("Steve".to_string()));
    context.insert_param(LootParamValue::ExplosionRadius(3.0));
    context.insert_dynamic_param(LootDynamicParamValue::EnchantmentLevel(5));
    context.insert_dynamic_param(LootDynamicParamValue::EnchantmentActive(true));
    context.insert_dynamic_param(LootDynamicParamValue::AttackingEntity("Steve".to_string()));

    assert_eq!(context.params.keys().len(), 3);
    assert_eq!(context.enchantment_level, 5);
    assert!(context.enchantment_active);
    assert!(context.killed_by_player);
    assert_eq!(context.explosion_radius, Some(3.0));
    assert_eq!(
        NumberProvider::EnchantmentLevel { scale: 2.0 }.float(&mut context),
        10.0
    );
    assert!(matches!(
        context
            .dynamic_params
            .get(&LootDynamicParamKey::AttackingEntity),
        Some(LootDynamicParamValue::AttackingEntity(entity)) if entity == "Steve"
    ));
}

#[test]
fn score_and_nbt_providers_resolve_context_and_storage_values() {
    let mut context = LootContext::new(LootParamSet::AllParams, 9);
    context
        .entity_properties
        .insert("this_entity".to_string(), "Zombie".to_string());
    context.context_nbt.insert(
        "this_entity".to_string(),
        HashMap::from([("CustomName".to_string(), "\"Dinnerbone\"".to_string())]),
    );
    context.storage_nbt.insert(
        "minecraft:loot_state".to_string(),
        HashMap::from([("bonus".to_string(), "enabled".to_string())]),
    );

    assert_eq!(
        ScoreProvider::Context {
            target: "this_entity".to_string()
        }
        .scoreboard_name(&context),
        Some("Zombie".to_string())
    );
    assert_eq!(
        ScoreProvider::Fixed {
            name: "global_counter".to_string()
        }
        .scoreboard_name(&context),
        Some("global_counter".to_string())
    );
    assert_eq!(
        NbtProvider::Context {
            target: "this_entity".to_string(),
            path: "CustomName".to_string(),
        }
        .get(&context),
        Some("\"Dinnerbone\"")
    );
    assert_eq!(
        NbtProvider::Storage {
            key: "minecraft:loot_state".to_string(),
            path: "bonus".to_string(),
        }
        .get(&context),
        Some("enabled")
    );
}

#[test]
fn loot_predicates_cover_java_condition_surface() {
    let mut context = LootContext::new(LootParamSet::AllParams, 13);
    context.insert_param(LootParamValue::BlockState("minecraft:oak_log".to_string()));
    context.insert_param(LootParamValue::Tool("minecraft:diamond_axe".to_string()));
    context.insert_param(LootParamValue::Origin(12.0, 70.0, -4.0));
    context.insert_param(LootParamValue::ThisEntity("minecraft:zombie".to_string()));
    context.insert_param(LootParamValue::LastDamagePlayer("Steve".to_string()));
    context.insert_param(LootParamValue::DamageSource(
        "minecraft:player_attack".to_string(),
    ));
    context.insert_dynamic_param(LootDynamicParamValue::EnchantmentLevel(2));
    context.insert_dynamic_param(LootDynamicParamValue::EnchantmentActive(true));
    context
        .block_state_properties
        .insert("axis".to_string(), "y".to_string());
    context.scores.insert("kills".to_string(), 6);
    context.entity_properties.insert(
        "damage_source.bypasses_armor".to_string(),
        "false".to_string(),
    );
    context
        .entity_properties
        .insert("dimension".to_string(), "minecraft:overworld".to_string());
    context.weather_raining = true;
    context.weather_thundering = false;
    context.game_time = 12_000;
    context
        .condition_references
        .insert("minecraft:ok".to_string());

    let condition = LootCondition::AllOf(vec![
        LootCondition::AnyOf(vec![
            LootCondition::RandomChance(1.0),
            LootCondition::RandomChanceWithEnchantedBonus {
                unenchanted_chance: 0.0,
                enchanted_chance: 1.0,
            },
        ]),
        LootCondition::Inverted(Box::new(LootCondition::RandomChance(0.0))),
        LootCondition::SurvivesExplosion,
        LootCondition::BlockState {
            block: "minecraft:oak_log".to_string(),
        },
        LootCondition::BlockStateProperty {
            property: "axis".to_string(),
            value: "y".to_string(),
        },
        LootCondition::MatchTool {
            item: "minecraft:diamond_axe".to_string(),
        },
        LootCondition::EntityProperty {
            key: "this_entity".to_string(),
            value: "minecraft:zombie".to_string(),
        },
        LootCondition::EntityScore {
            name: "kills".to_string(),
            min: 5,
            max: 10,
        },
        LootCondition::KilledByPlayer,
        LootCondition::RandomChanceWithLooting {
            chance: 1.0,
            looting_multiplier: 0.0,
        },
        LootCondition::DamageSourceProperty {
            key: "type".to_string(),
            value: "minecraft:player_attack".to_string(),
        },
        LootCondition::DamageSourceProperty {
            key: "bypasses_armor".to_string(),
            value: "false".to_string(),
        },
        LootCondition::LocationCheck {
            key: "origin".to_string(),
            value: "12,70,-4".to_string(),
        },
        LootCondition::LocationCheck {
            key: "dimension".to_string(),
            value: "minecraft:overworld".to_string(),
        },
        LootCondition::ValueCheck {
            provider: NumberProvider::Score {
                name: "kills".to_string(),
                scale: 1.0,
            },
            min: 6.0,
            max: 6.0,
        },
        LootCondition::WeatherCheck {
            raining: Some(true),
            thundering: Some(false),
        },
        LootCondition::TimeCheck {
            min: 0,
            max: 24_000,
        },
        LootCondition::Reference("minecraft:ok".to_string()),
        LootCondition::EnchantmentActiveCheck,
        LootCondition::TableBonus {
            chances: vec![0.0, 0.0, 1.0],
        },
    ]);

    assert!(condition.matches(&context));
}

#[test]
fn weighted_entries_rolls_bonus_rolls_and_luck_follow_pool_shape() {
    let mut pool = LootPool::single(LootEntry::item("minecraft:stick", 1));
    pool.entries.push(LootEntry::Item {
        item: "minecraft:diamond".to_string(),
        weight: 1,
        quality: 10,
        conditions: Vec::new(),
        functions: vec![LootFunction::SetCount(NumberProvider::Constant(2.0))],
    });
    pool.rolls = NumberProvider::Constant(2.0);
    pool.bonus_rolls = NumberProvider::Constant(1.0);

    let table = table_with_pool(pool);
    let mut context = LootContext::new(LootParamSet::Chest, 11);
    context.luck = 2.0;
    let drops = table.evaluate(&mut context);

    assert_eq!(drops.len(), 4);
    assert!(drops.iter().all(|stack| stack.item == "minecraft:diamond"));
    assert!(drops.iter().all(|stack| stack.count == 2));
}

#[test]
fn conditions_cover_random_player_explosion_time_tool_score_and_combinators() {
    let condition = LootCondition::AllOf(vec![
        LootCondition::KilledByPlayer,
        LootCondition::SurvivesExplosion,
        LootCondition::TimeCheck { min: 10, max: 30 },
        LootCondition::MatchTool {
            item: "minecraft:diamond_pickaxe".to_string(),
        },
        LootCondition::ValueCheck {
            provider: NumberProvider::Score {
                name: "kills".to_string(),
                scale: 0.5,
            },
            min: 2.0,
            max: 4.0,
        },
        LootCondition::Inverted(Box::new(LootCondition::BlockState {
            block: "minecraft:dirt".to_string(),
        })),
        LootCondition::AnyOf(vec![
            LootCondition::Reference("bonus".to_string()),
            LootCondition::RandomChance(0.0),
        ]),
    ]);

    let mut context = LootContext::new(LootParamSet::Block, 7);
    context.killed_by_player = true;
    context.explosion_radius = None;
    context.game_time = 20;
    context.tool = Some("minecraft:diamond_pickaxe".to_string());
    context.block = Some("minecraft:stone".to_string());
    context.scores.insert("kills".to_string(), 6);
    context.condition_references.insert("bonus".to_string());

    assert!(condition.matches(&context));
}

#[test]
fn entry_types_cover_tags_nested_tables_dynamic_alternatives_sequences_and_groups() {
    let nested = table_with_pool(LootPool::single(LootEntry::item("minecraft:apple", 1)));
    let pool = LootPool {
        entries: vec![LootEntry::Group(vec![
            LootEntry::Tag {
                tag: "minecraft:logs".to_string(),
                expand: true,
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
            },
            LootEntry::NestedTable("minecraft:bonus".to_string()),
            LootEntry::Dynamic("minecraft:sherds".to_string()),
            LootEntry::Alternatives(vec![
                LootEntry::Empty {
                    weight: 0,
                    quality: 0,
                    conditions: Vec::new(),
                },
                LootEntry::item("minecraft:emerald", 1),
            ]),
            LootEntry::Sequence(vec![
                LootEntry::item("minecraft:string", 1),
                LootEntry::item("minecraft:feather", 1),
            ]),
        ])],
        conditions: Vec::new(),
        functions: Vec::new(),
        rolls: NumberProvider::Constant(1.0),
        bonus_rolls: NumberProvider::Constant(0.0),
    };
    let table = table_with_pool(pool);
    let mut context = LootContext::new(LootParamSet::Archaeology, 3);
    context.tables.insert("minecraft:bonus".to_string(), nested);
    context.tags.insert(
        "minecraft:logs".to_string(),
        vec![
            "minecraft:oak_log".to_string(),
            "minecraft:birch_log".to_string(),
        ],
    );
    context.dynamic_drops.insert(
        "minecraft:sherds".to_string(),
        vec![LootStack::new("minecraft:angler_pottery_sherd", 1)],
    );

    let names: Vec<_> = table
        .evaluate(&mut context)
        .into_iter()
        .map(|stack| stack.item)
        .collect();
    assert!(names.contains(&"minecraft:oak_log".to_string()));
    assert!(names.contains(&"minecraft:birch_log".to_string()));
    assert!(names.contains(&"minecraft:apple".to_string()));
    assert!(names.contains(&"minecraft:angler_pottery_sherd".to_string()));
    assert!(names.contains(&"minecraft:emerald".to_string()));
    assert!(names.contains(&"minecraft:string".to_string()));
    assert!(names.contains(&"minecraft:feather".to_string()));
}

#[test]
fn functions_apply_in_entry_pool_and_table_order_with_stack_splitting() {
    let mut pool = LootPool::single(LootEntry::Item {
        item: "minecraft:stone".to_string(),
        weight: 1,
        quality: 0,
        conditions: Vec::new(),
        functions: vec![LootFunction::SetCount(NumberProvider::Constant(130.0))],
    });
    pool.functions
        .push(LootFunction::LimitCount { min: 1, max: 70 });
    let mut table = table_with_pool(pool);
    table
        .functions
        .push(LootFunction::SetItem("minecraft:cobblestone".to_string()));

    let mut context = LootContext::new(LootParamSet::Block, 13);
    let drops = table.evaluate(&mut context);

    assert_eq!(
        drops,
        vec![
            LootStack::new("minecraft:cobblestone", 64),
            LootStack::new("minecraft:cobblestone", 6)
        ]
    );
}

#[test]
fn random_sequences_are_repeatable_and_independent_from_context_seed_stream() {
    let mut table = table_with_pool(LootPool {
        entries: vec![
            LootEntry::item("minecraft:coal", 1),
            LootEntry::item("minecraft:iron_ingot", 1),
        ],
        conditions: Vec::new(),
        functions: Vec::new(),
        rolls: NumberProvider::Constant(8.0),
        bonus_rolls: NumberProvider::Constant(0.0),
    });
    table.random_sequence = Some("minecraft:chests/simple_dungeon".to_string());

    let mut first = LootContext::new(LootParamSet::Chest, 99);
    let mut second = LootContext::new(LootParamSet::Chest, 99);
    let mut different_sequence = table.clone();
    different_sequence.random_sequence =
        Some("minecraft:chests/abandoned_mineshaft".to_string());

    assert_eq!(table.evaluate(&mut first), table.evaluate(&mut second));
    assert_ne!(
        table.evaluate(&mut LootContext::new(LootParamSet::Chest, 99)),
        different_sequence.evaluate(&mut LootContext::new(LootParamSet::Chest, 99))
    );
}

#[test]
fn container_fill_shuffles_once_and_reports_overfill() {
    let mut pool = LootPool::single(LootEntry::Item {
        item: "minecraft:bread".to_string(),
        weight: 1,
        quality: 0,
        conditions: Vec::new(),
        functions: vec![LootFunction::SetCount(NumberProvider::Constant(1.0))],
    });
    pool.rolls = NumberProvider::Constant(4.0);
    let table = table_with_pool(pool);
    let mut context = LootContext::new(LootParamSet::Chest, 5);

    let slots = table.fill_container(&mut context, 2);

    assert_eq!(slots.iter().filter(|slot| slot.is_some()).count(), 2);
    assert_eq!(context.warnings, vec!["Tried to over-fill a container"]);
}

#[test]
fn validation_reports_invalid_pool_and_entry_shapes() {
    let table = table_with_pool(LootPool {
        entries: vec![LootEntry::Tag {
            tag: String::new(),
            expand: false,
            weight: -1,
            quality: 0,
            conditions: Vec::new(),
        }],
        conditions: Vec::new(),
        functions: Vec::new(),
        rolls: NumberProvider::Constant(-1.0),
        bonus_rolls: NumberProvider::Constant(0.0),
    });

    let errors = table.validate();
    assert!(errors.iter().any(|error| error.contains("rolls")));
    assert!(errors.iter().any(|error| error.contains("invalid tag")));
}

#[test]
fn behavior_engine_maps_named_surfaces_to_vanilla_param_sets_and_delivery() {
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:blocks/stone",
        table_with_pool(LootPool::single(LootEntry::Item {
            item: "minecraft:cobblestone".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::AllOf(vec![
                LootCondition::BlockState {
                    block: "minecraft:stone".to_string(),
                },
                LootCondition::MatchTool {
                    item: "minecraft:iron_pickaxe".to_string(),
                },
            ])],
            functions: Vec::new(),
        })),
    );

    let mut request = LootRequest::new(LootSurface::BlockBreak, "minecraft:blocks/stone");
    request.origin = (1.0, 64.0, 2.0);
    request.block = Some("minecraft:stone".to_string());
    request.tool = Some("minecraft:iron_pickaxe".to_string());

    let resolution = engine.resolve(request, 17);

    assert_eq!(resolution.param_set, LootParamSet::Block);
    assert_eq!(
        resolution.delivery,
        LootDelivery::DropAt(
            (1.0, 64.0, 2.0),
            vec![LootStack::new("minecraft:cobblestone", 1)]
        )
    );
}

#[test]
fn behavior_engine_covers_entity_fishing_archaeology_reward_gift_barter_and_command_paths() {
    let mut engine = LootBehaviorEngine::new();
    for (id, item) in [
        ("minecraft:entities/zombie", "minecraft:rotten_flesh"),
        ("minecraft:gameplay/fishing", "minecraft:cod"),
        (
            "minecraft:archaeology/desert_pyramid",
            "minecraft:pottery_sherd",
        ),
        (
            "minecraft:advancements/story/mine_stone",
            "minecraft:emerald",
        ),
        ("minecraft:gameplay/cat_morning_gift", "minecraft:string"),
        ("minecraft:gameplay/piglin_bartering", "minecraft:quartz"),
        ("minecraft:commands/debug", "minecraft:stick"),
    ] {
        engine.insert_table(
            id,
            table_with_pool(LootPool::single(LootEntry::item(item, 1))),
        );
    }

    let mut entity = LootRequest::new(LootSurface::EntityDeath, "minecraft:entities/zombie");
    entity.target_entity = Some("Zombie".to_string());
    entity.damage_source = Some("minecraft:player_attack".to_string());
    entity.killed_by_player = true;
    assert_eq!(engine.resolve(entity, 1).param_set, LootParamSet::Entity);

    let mut fishing =
        LootRequest::new(LootSurface::FishingRetrieve, "minecraft:gameplay/fishing");
    fishing.tool = Some("minecraft:fishing_rod".to_string());
    assert_eq!(engine.resolve(fishing, 1).param_set, LootParamSet::Fishing);

    let mut archaeology = LootRequest::new(
        LootSurface::ArchaeologyBrush,
        "minecraft:archaeology/desert_pyramid",
    );
    archaeology.tool = Some("minecraft:brush".to_string());
    assert_eq!(
        engine.resolve(archaeology, 1).param_set,
        LootParamSet::Archaeology
    );

    let mut advancement = LootRequest::new(
        LootSurface::AdvancementReward,
        "minecraft:advancements/story/mine_stone",
    );
    advancement.actor = Some("Steve".to_string());
    assert_eq!(
        engine.resolve(advancement, 1).delivery,
        LootDelivery::GiveToEntity(
            "Steve".to_string(),
            vec![LootStack::new("minecraft:emerald", 1)]
        )
    );

    let mut gift = LootRequest::new(LootSurface::Gift, "minecraft:gameplay/cat_morning_gift");
    gift.actor = Some("Steve".to_string());
    assert_eq!(engine.resolve(gift, 1).param_set, LootParamSet::Gift);

    let mut barter = LootRequest::new(
        LootSurface::PiglinBarter,
        "minecraft:gameplay/piglin_bartering",
    );
    barter.actor = Some("Piglin".to_string());
    assert_eq!(engine.resolve(barter, 1).param_set, LootParamSet::Barter);

    let mut command = LootRequest::new(LootSurface::Command, "minecraft:commands/debug");
    command.actor = Some("Steve".to_string());
    assert_eq!(
        engine.resolve(command, 1).delivery,
        LootDelivery::GiveToEntity(
            "Steve".to_string(),
            vec![LootStack::new("minecraft:stick", 1)]
        )
    );
}

#[test]
fn entity_death_context_carries_java_kill_params_and_looting_bonus() {
    let mut engine = LootBehaviorEngine::new();
    let mut pool = LootPool::single(LootEntry::Item {
        item: "minecraft:rotten_flesh".to_string(),
        weight: 1,
        quality: 0,
        conditions: vec![
            LootCondition::KilledByPlayer,
            LootCondition::EntityProperty {
                key: "this_entity".to_string(),
                value: "Zombie".to_string(),
            },
            LootCondition::EntityProperty {
                key: "killer_entity".to_string(),
                value: "Steve".to_string(),
            },
            LootCondition::EntityProperty {
                key: "direct_killer_entity".to_string(),
                value: "Arrow".to_string(),
            },
            LootCondition::EntityProperty {
                key: "last_damage_player".to_string(),
                value: "Steve".to_string(),
            },
        ],
        functions: vec![LootFunction::AddLootingBonus {
            per_level: NumberProvider::Constant(2.0),
            limit: Some(5),
        }],
    });
    pool.conditions
        .push(LootCondition::RandomChanceWithLooting {
            chance: 0.0,
            looting_multiplier: 1.0,
        });
    engine.insert_table("minecraft:entities/zombie", table_with_pool(pool));

    let mut request = LootRequest::new(LootSurface::EntityDeath, "minecraft:entities/zombie");
    request.origin = (2.0, 64.0, 3.0);
    request.target_entity = Some("Zombie".to_string());
    request.killer_entity = Some("Steve".to_string());
    request.direct_killer_entity = Some("Arrow".to_string());
    request.last_damage_player = Some("Steve".to_string());
    request.damage_source = Some("minecraft:arrow".to_string());
    request.looting_level = 2;

    let resolution = engine.resolve(request, 11);

    assert_eq!(resolution.param_set, LootParamSet::Entity);
    assert_eq!(
        resolution.delivery,
        LootDelivery::DropAt(
            (2.0, 64.0, 3.0),
            vec![LootStack::new("minecraft:rotten_flesh", 5)]
        )
    );
}

#[test]
fn advancement_reward_loot_grants_xp_and_tables_with_player_context() {
    let mut advancements = PlayerAdvancementSet::default();
    let definition = AdvancementDefinition::all_of(
        "minecraft:story/mine_stone",
        None,
        &["stone"],
        AdvancementRewards {
            experience: 5,
            loot: vec![Identifier::parse("minecraft:advancements/story/mine_stone").unwrap()],
            recipes: Vec::new(),
            function: None,
        },
        None,
    )
    .unwrap();
    let reward = advancements.grant(&definition, "stone", 1).unwrap();
    let mut engine = LootBehaviorEngine::new();
    engine.insert_table(
        "minecraft:advancements/story/mine_stone",
        table_with_pool(LootPool::single(LootEntry::Item {
            item: "minecraft:emerald".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::EntityProperty {
                key: "this_entity".to_string(),
                value: "Steve".to_string(),
            }],
            functions: Vec::new(),
        })),
    );

    let output = resolve_advancement_reward_loot(
        &engine,
        AdvancementRewardLootInput {
            player: "Steve".to_string(),
            origin: (8.0, 65.0, 9.0),
            experience: reward.experience,
            loot_tables: reward.loot.iter().map(ToString::to_string).collect(),
        },
        44,
    );

    assert_eq!(output.experience, 5);
    assert_eq!(output.loot.len(), 1);
    assert_eq!(output.loot[0].param_set, LootParamSet::AdvancementReward);
    assert_eq!(
        output.loot[0].delivery,
        LootDelivery::GiveToEntity(
            "Steve".to_string(),
            vec![LootStack::new("minecraft:emerald", 1)]
        )
    );
}

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
