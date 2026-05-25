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
    let surfaces = [
        (
            LootSurface::BlockBreak,
            LootParamSet::Block,
            LootContextEntityType::Block,
        ),
        (
            LootSurface::EntityDeath,
            LootParamSet::Entity,
            LootContextEntityType::Entity,
        ),
        (
            LootSurface::ChestOpen,
            LootParamSet::Chest,
            LootContextEntityType::Chest,
        ),
        (
            LootSurface::FishingRetrieve,
            LootParamSet::Fishing,
            LootContextEntityType::Fishing,
        ),
        (
            LootSurface::ArchaeologyBrush,
            LootParamSet::Archaeology,
            LootContextEntityType::Archaeology,
        ),
        (
            LootSurface::AdvancementReward,
            LootParamSet::AdvancementReward,
            LootContextEntityType::AdvancementReward,
        ),
        (
            LootSurface::Gift,
            LootParamSet::Gift,
            LootContextEntityType::Gift,
        ),
        (
            LootSurface::PiglinBarter,
            LootParamSet::Barter,
            LootContextEntityType::Barter,
        ),
        (
            LootSurface::Vault,
            LootParamSet::Vault,
            LootContextEntityType::Vault,
        ),
        (
            LootSurface::Command,
            LootParamSet::Command,
            LootContextEntityType::Command,
        ),
        (
            LootSurface::Selector,
            LootParamSet::Selector,
            LootContextEntityType::Selector,
        ),
        (
            LootSurface::AdvancementEntity,
            LootParamSet::AdvancementEntity,
            LootContextEntityType::AdvancementEntity,
        ),
        (
            LootSurface::Equipment,
            LootParamSet::Equipment,
            LootContextEntityType::Equipment,
        ),
    ];
    assert_eq!(surfaces.len(), 13);
    for (surface, param_set, entity_type) in surfaces {
        assert_eq!(surface.param_set(), param_set);
        assert_eq!(surface.entity_type(), entity_type);
        assert_eq!(entity_type.param_set(), param_set);
    }

    let mut params = LootParams::default();
    for value in [
        LootParamValue::InteractingEntity("Steve".to_string()),
        LootParamValue::TargetEntity("Cow".to_string()),
        LootParamValue::BlockState("minecraft:stone".to_string()),
        LootParamValue::BlockEntity("Chest".to_string()),
        LootParamValue::Origin(1.0, 64.0, 2.0),
        LootParamValue::Tool("minecraft:diamond_pickaxe".to_string()),
        LootParamValue::ThisEntity("Zombie".to_string()),
        LootParamValue::LastDamagePlayer("Steve".to_string()),
        LootParamValue::AttackingEntity("Steve".to_string()),
        LootParamValue::DirectAttackingEntity("Arrow".to_string()),
        LootParamValue::KillerEntity("Steve".to_string()),
        LootParamValue::DirectKillerEntity("Arrow".to_string()),
        LootParamValue::ExplosionRadius(2.0),
        LootParamValue::DamageSource("minecraft:player_attack".to_string()),
        LootParamValue::EnchantmentLevel(4),
        LootParamValue::EnchantmentActive(true),
        LootParamValue::AdditionalCostComponentAllowed,
    ] {
        params.insert(value);
    }
    assert_eq!(params.keys().len(), 17);
    assert!(matches!(
        params.get(LootParamKey::DamageSource),
        Some(LootParamValue::DamageSource(id)) if id == "minecraft:player_attack"
    ));

    let mut context = LootContext::new(LootParamSet::Entity, 4);
    context.insert_param(LootParamValue::ThisEntity("Zombie".to_string()));
    context.insert_param(LootParamValue::InteractingEntity("Steve".to_string()));
    context.insert_param(LootParamValue::TargetEntity("Cow".to_string()));
    context.insert_param(LootParamValue::AttackingEntity("Steve".to_string()));
    context.insert_param(LootParamValue::DirectAttackingEntity("Arrow".to_string()));
    context.insert_param(LootParamValue::LastDamagePlayer("Steve".to_string()));
    context.insert_param(LootParamValue::ExplosionRadius(3.0));
    context.insert_param(LootParamValue::AdditionalCostComponentAllowed);
    context.insert_dynamic_param(LootDynamicParamValue::EnchantmentLevel(5));
    context.insert_dynamic_param(LootDynamicParamValue::EnchantmentActive(true));
    context.insert_dynamic_param(LootDynamicParamValue::AttackingEntity("Steve".to_string()));
    context.insert_dynamic_param(LootDynamicParamValue::DirectAttackingEntity(
        "Arrow".to_string(),
    ));

    assert_eq!(context.params.keys().len(), 8);
    assert_eq!(context.enchantment_level, 5);
    assert!(context.enchantment_active);
    assert!(context.killed_by_player);
    assert_eq!(context.explosion_radius, Some(3.0));
    assert_eq!(
        context.entity_properties.get("interacting_entity"),
        Some(&"Steve".to_string())
    );
    assert_eq!(
        context.entity_properties.get("target_entity"),
        Some(&"Cow".to_string())
    );
    assert_eq!(
        context.entity_properties.get("attacking_entity"),
        Some(&"Steve".to_string())
    );
    assert_eq!(
        context.entity_properties.get("direct_attacking_entity"),
        Some(&"Arrow".to_string())
    );
    assert_eq!(
        context
            .entity_properties
            .get("additional_cost_component_allowed"),
        Some(&"true".to_string())
    );
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
    assert!(matches!(
        context
            .dynamic_params
            .get(&LootDynamicParamKey::DirectAttackingEntity),
        Some(LootDynamicParamValue::DirectAttackingEntity(entity)) if entity == "Arrow"
    ));
    assert_eq!(
        context.entity_properties.get("direct_attacking_entity"),
        Some(&"Arrow".to_string())
    );
}

#[test]
fn loot_table_evaluates_pool_list_and_table_functions_in_java_order() {
    let table = LootTable {
        param_set: LootParamSet::Chest,
        random_sequence: None,
        pools: vec![
            LootPool::single(LootEntry::item("minecraft:apple", 1)),
            LootPool::single(LootEntry::item("minecraft:bread", 2)),
        ],
        functions: vec![LootFunction::SetCount(NumberProvider::Constant(5.0))],
    };
    let mut context = LootContext::new(LootParamSet::Chest, 4);

    let drops = table.evaluate(&mut context);

    assert_eq!(
        drops,
        vec![
            LootStack::new("minecraft:apple", 5),
            LootStack::new("minecraft:bread", 5),
        ]
    );
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
fn number_providers_cover_java_26_1_2_provider_registry() {
    let mut context = LootContext::new(LootParamSet::AllParams, 17);
    context.scores.insert("kills".to_string(), 7);
    context
        .storage_numbers
        .insert("minecraft:loot_state:bonus".to_string(), 4.5);
    context
        .environment_attributes
        .insert("minecraft:local_difficulty".to_string(), 2.25);
    context.insert_dynamic_param(LootDynamicParamValue::EnchantmentLevel(3));

    assert_eq!(NumberProvider::Constant(1.6).int(&mut context), 2);
    assert_eq!(
        NumberProvider::UniformProvider {
            min: Box::new(NumberProvider::Constant(3.0)),
            max: Box::new(NumberProvider::Constant(3.0)),
        }
        .int(&mut context),
        3
    );
    assert_eq!(
        NumberProvider::UniformProvider {
            min: Box::new(NumberProvider::Constant(2.5)),
            max: Box::new(NumberProvider::Constant(2.5)),
        }
        .float(&mut context),
        2.5
    );
    assert_eq!(
        NumberProvider::BinomialProvider {
            n: Box::new(NumberProvider::Constant(4.0)),
            p: Box::new(NumberProvider::Constant(1.0)),
        }
        .int(&mut context),
        4
    );
    assert_eq!(
        NumberProvider::Score {
            name: "kills".to_string(),
            scale: 0.5,
        }
        .float(&mut context),
        3.5
    );
    assert_eq!(
        NumberProvider::Storage {
            key: "minecraft:loot_state:bonus".to_string(),
            scale: 2.0,
        }
        .float(&mut context),
        9.0
    );
    assert_eq!(
        NumberProvider::EnchantmentLevel { scale: 2.0 }.float(&mut context),
        6.0
    );
    assert_eq!(
        NumberProvider::Sum(vec![
            NumberProvider::Constant(1.25),
            NumberProvider::Constant(2.5),
            NumberProvider::Score {
                name: "kills".to_string(),
                scale: 0.25,
            },
        ])
        .float(&mut context),
        5.5
    );
    assert_eq!(
        NumberProvider::EnvironmentAttribute {
            attribute: "minecraft:local_difficulty".to_string(),
        }
        .float(&mut context),
        2.25
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
    pool.conditions.push(LootCondition::ValueCheck {
        provider: NumberProvider::Constant(1.0),
        min: 1.0,
        max: 1.0,
    });
    pool.functions
        .push(LootFunction::LimitCount { min: 1, max: 2 });

    let table = table_with_pool(pool);
    let mut context = LootContext::new(LootParamSet::Chest, 11);
    context.luck = 2.0;
    let drops = table.evaluate(&mut context);

    assert_eq!(drops.len(), 4);
    assert!(drops.iter().all(|stack| stack.item == "minecraft:diamond"));
    assert!(drops.iter().all(|stack| stack.count == 2));

    let mut negative_luck = LootContext::new(LootParamSet::Chest, 11);
    negative_luck.luck = -0.25;
    let drops = table.evaluate(&mut negative_luck);

    assert_eq!(drops, vec![LootStack::new("minecraft:stick", 1)]);

    let mut blocked_pool = LootPool::single(LootEntry::item("minecraft:barrier", 1));
    blocked_pool.conditions.push(LootCondition::ValueCheck {
        provider: NumberProvider::Constant(0.0),
        min: 1.0,
        max: 1.0,
    });
    let mut blocked_context = LootContext::new(LootParamSet::Chest, 11);
    assert!(table_with_pool(blocked_pool)
        .evaluate(&mut blocked_context)
        .is_empty());

    let mut engine = LootBehaviorEngine::new();
    let mut pool = LootPool::single(LootEntry::item("minecraft:emerald", 1));
    pool.bonus_rolls = NumberProvider::Constant(1.0);
    engine.insert_table("minecraft:commands/luck_probe", table_with_pool(pool));
    let mut request = LootRequest::new(LootSurface::Command, "minecraft:commands/luck_probe");
    request.luck = 2.0;

    assert_eq!(
        engine.resolve(request, 11).delivery,
        LootDelivery::DropAt(
            (0.0, 0.0, 0.0),
            vec![
                LootStack::new("minecraft:emerald", 1),
                LootStack::new("minecraft:emerald", 1),
                LootStack::new("minecraft:emerald", 1),
            ]
        )
    );
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
fn group_entries_respect_child_conditions_before_generating_stacks() {
    let table = table_with_pool(LootPool::single(LootEntry::Group(vec![
        LootEntry::Item {
            item: "minecraft:blocked".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::RandomChance(0.0)],
            functions: Vec::new(),
        },
        LootEntry::item("minecraft:allowed", 1),
    ])));
    let mut context = LootContext::new(LootParamSet::Chest, 11);

    assert_eq!(
        table.evaluate(&mut context),
        vec![LootStack::new("minecraft:allowed", 1)]
    );
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


mod advanced_tests;
