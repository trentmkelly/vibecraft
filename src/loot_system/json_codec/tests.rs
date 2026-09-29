//! `LootTable.DIRECT_CODEC` for JSON: the supported subset decodes to the runtime
//! model, everything else is rejected loudly.

use super::*;

fn decode(json: &str) -> Result<LootTable, String> {
    decode_loot_table(json)
}

#[test]
fn a_vanilla_shaped_block_table_decodes_conditions_functions_and_number_providers() {
    let table = decode(
        r#"{
          "type": "minecraft:block",
          "random_sequence": "minecraft:blocks/redstone_ore",
          "pools": [{
            "bonus_rolls": 0.0,
            "conditions": [{"condition": "minecraft:survives_explosion"}],
            "entries": [{
              "type": "minecraft:item",
              "name": "minecraft:redstone",
              "weight": 3,
              "functions": [
                {"function": "minecraft:set_count", "count": {"type": "minecraft:uniform", "min": 4.0, "max": 5.0}},
                {"function": "minecraft:limit_count", "limit": {"min": 1, "max": 9}},
                {"function": "minecraft:explosion_decay"}
              ]
            }],
            "rolls": 1.0
          }]
        }"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(table.param_set, LootParamSet::Block);
    assert_eq!(
        table.random_sequence.as_deref(),
        Some("minecraft:blocks/redstone_ore")
    );
    let pool = &table.pools[0];
    assert_eq!(pool.conditions, vec![LootCondition::SurvivesExplosion]);
    assert_eq!(pool.rolls, NumberProvider::Constant(1.0));
    let LootEntry::Item {
        item,
        weight,
        functions,
        ..
    } = &pool.entries[0]
    else {
        panic!("item entry");
    };
    assert_eq!((item.as_str(), *weight), ("minecraft:redstone", 3));
    assert_eq!(
        functions,
        &vec![
            LootFunction::SetCount(NumberProvider::UniformProvider {
                min: Box::new(NumberProvider::Constant(4.0)),
                max: Box::new(NumberProvider::Constant(5.0)),
            }),
            LootFunction::LimitCount { min: 1, max: 9 },
            LootFunction::ApplyExplosionDecay,
        ]
    );
}

#[test]
fn composite_entries_and_nested_conditions_decode() {
    let table = decode(
        r#"{"pools":[{"rolls":{"type":"minecraft:binomial","n":3,"p":0.5},"entries":[{
          "type": "minecraft:alternatives",
          "children": [
            {"type": "minecraft:item", "name": "minecraft:stone",
             "conditions": [{"condition": "minecraft:inverted",
                             "term": {"condition": "minecraft:random_chance", "chance": 0.25}}]},
            {"type": "minecraft:empty"}
          ]}]}]}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    // A missing or unknown `type` is the lenient default `ALL_PARAMS`.
    assert_eq!(table.param_set, LootParamSet::AllParams);
    assert!(matches!(
        table.pools[0].rolls,
        NumberProvider::BinomialProvider { .. }
    ));
    let LootEntry::Alternatives(children) = &table.pools[0].entries[0] else {
        panic!("alternatives");
    };
    assert_eq!(children.len(), 2);
    let LootEntry::Item { conditions, .. } = &children[0] else {
        panic!("item");
    };
    assert_eq!(
        conditions,
        &vec![LootCondition::Inverted(Box::new(
            LootCondition::RandomChance(0.25)
        ))]
    );
}

#[test]
fn block_state_property_becomes_the_block_plus_each_property() {
    let table = decode(
        r#"{"pools":[{"rolls":1,"conditions":[{"condition":"minecraft:block_state_property",
            "block":"minecraft:wheat","properties":{"age":"7"}}],
            "entries":[{"type":"minecraft:item","name":"minecraft:wheat"}]}]}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        table.pools[0].conditions,
        vec![LootCondition::AllOf(vec![
            LootCondition::BlockState {
                block: "minecraft:wheat".to_string()
            },
            LootCondition::BlockStateProperty {
                property: "age".to_string(),
                value: "7".to_string()
            },
        ])]
    );
}

#[test]
fn an_empty_table_decodes_and_malformed_tables_are_rejected() {
    let empty = decode(r#"{"pools":[]}"#).unwrap_or_else(|e| panic!("{e}"));
    assert!(empty.pools.is_empty() && empty.functions.is_empty());

    for (json, needle) in [
        (
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone",
               "functions":[{"function":"minecraft:no_such_function"}]}]}]}"#,
            "Unknown function type 'minecraft:no_such_function'",
        ),
        (
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone",
               "functions":[{"function":"minecraft:enchanted_count_increase"}]}]}]}"#,
            "missing field enchantment",
        ),
        (
            r#"{"pools":[{"rolls":1,"conditions":[{"condition":"minecraft:table_bonus",
               "enchantment":"minecraft:fortune","chances":[]}],"entries":[]}]}"#,
            "non-empty",
        ),
        (
            r#"{"pools":[{"rolls":1,"conditions":[{"condition":"minecraft:bogus"}],"entries":[]}]}"#,
            "Unknown condition type 'minecraft:bogus'",
        ),
        (
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:composter"}]}]}"#,
            "Unknown loot entry type 'minecraft:composter'",
        ),
        (
            r#"{"pools":[{"rolls":{"type":"minecraft:bogus"},"entries":[]}]}"#,
            "Unknown number provider type 'minecraft:bogus'",
        ),
        (
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:tag","name":"minecraft:logs"}]}]}"#,
            "missing field expand",
        ),
        (r#"{"pools":[{"entries":[]}]}"#, "missing field rolls"),
        (r#"{"pools":{}}"#, "pools is not a list"),
        ("[]", "loot table is not a JSON object"),
        ("{", "invalid JSON"),
    ] {
        let error = decode(json).expect_err(json);
        assert!(error.contains(needle), "{json}: {error}");
    }
}

#[test]
fn parts_the_runtime_cannot_evaluate_decode_as_unmodeled_and_flag_the_table() {
    let modeled = decode(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"stone"}]}]}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(modeled.is_fully_modeled());

    for json in [
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone",
            "conditions":[{"condition":"minecraft:match_tool","predicate":{"count":1}}]}]}]}"#,
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone",
            "functions":[{"function":"minecraft:set_name","name":"x"}]}]}]}"#,
        r#"{"pools":[{"rolls":{"type":"minecraft:score","target":"this","score":"s"},"entries":[]}]}"#,
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:slots","slot_source":{}}]}]}"#,
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":{"pools":[]}}]}]}"#,
    ] {
        let table = decode(json).unwrap_or_else(|e| panic!("{json}: {e}"));
        assert!(!table.is_fully_modeled(), "{json}");
    }
}

#[test]
fn inline_lists_number_providers_and_entry_extras_follow_the_java_codecs() {
    let table = decode(
        r#"{"type":"minecraft:block_interact","functions":[[{"function":"minecraft:discard"}]],
          "pools":[{"rolls":{"min":1,"max":2},"conditions":[[{"condition":"minecraft:killed_by_player"}]],
            "entries":[
              {"type":"minecraft:alternatives","conditions":[{"condition":"minecraft:killed_by_player"}],
               "children":[{"type":"minecraft:dynamic","name":"contents","weight":3}]},
              {"type":"minecraft:loot_table","value":"minecraft:chests/x",
               "functions":[{"function":"minecraft:explosion_decay",
                             "conditions":[{"condition":"minecraft:survives_explosion"}]}]}
            ]}]}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(table.param_set, LootParamSet::BlockInteract);
    assert_eq!(
        table.functions,
        vec![LootFunction::Sequence(vec![LootFunction::Discard])]
    );
    let pool = &table.pools[0];
    // A typeless number provider object is a uniform generator.
    assert!(matches!(pool.rolls, NumberProvider::UniformProvider { .. }));
    assert_eq!(
        pool.conditions,
        vec![LootCondition::AllOf(vec![LootCondition::KilledByPlayer])]
    );
    let LootEntry::Conditional { conditions, entry } = &pool.entries[0] else {
        panic!("composite conditions");
    };
    assert_eq!(conditions, &vec![LootCondition::KilledByPlayer]);
    let LootEntry::Alternatives(children) = entry.as_ref() else {
        panic!("alternatives");
    };
    assert!(matches!(
        &children[0],
        LootEntry::Dynamic { name, weight: 3, .. } if name == "minecraft:contents"
    ));
    let LootEntry::WeightedNestedTable { functions, .. } = &pool.entries[1] else {
        panic!("nested table");
    };
    assert_eq!(
        functions,
        &vec![LootFunction::Filtered {
            condition: LootCondition::AllOf(vec![LootCondition::SurvivesExplosion]),
            function: Box::new(LootFunction::ApplyExplosionDecay),
        }]
    );
    assert!(table.is_fully_modeled());
}

#[test]
fn silk_touch_match_tool_and_bonus_functions_evaluate_against_tool_enchantments() {
    let table = decode(
        r#"{"type":"minecraft:block","pools":[{"rolls":1,"entries":[{
          "type":"minecraft:alternatives","children":[
            {"type":"minecraft:item","name":"minecraft:stone","conditions":[{
              "condition":"minecraft:match_tool","predicate":{"predicates":{
                "minecraft:enchantments":[{"enchantments":"minecraft:silk_touch","levels":{"min":1}}]}}}]},
            {"type":"minecraft:item","name":"minecraft:cobblestone","functions":[{
              "function":"minecraft:apply_bonus","enchantment":"minecraft:fortune",
              "formula":"minecraft:uniform_bonus_count","parameters":{"bonusMultiplier":0}},
              {"function":"minecraft:set_count","count":2,"add":true}]}
          ]}]}]}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(table.is_fully_modeled());

    let mut silk = LootContext::new(LootParamSet::Block, 1);
    silk.entity_properties
        .insert("silk_touch".to_string(), "true".to_string());
    assert_eq!(table.evaluate(&mut silk), vec![LootStack::new("minecraft:stone", 1)]);

    let mut plain = LootContext::new(LootParamSet::Block, 1);
    assert_eq!(
        table.evaluate(&mut plain),
        vec![LootStack::new("minecraft:cobblestone", 3)]
    );
}

#[test]
fn enchanted_count_increase_and_level_based_values_follow_java() {
    let table = decode(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:bone",
          "functions":[{"function":"minecraft:enchanted_count_increase",
            "enchantment":"minecraft:looting","count":1.5,"limit":4}]}]}]}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let evaluate = |looting: i32| {
        let mut context = LootContext::new(LootParamSet::Entity, 5);
        context.looting_level = looting;
        table.evaluate(&mut context)[0].count
    };
    // No level leaves the count alone; `round(level * 1.5)` is added; the limit caps it.
    assert_eq!((evaluate(0), evaluate(1), evaluate(2), evaluate(3)), (1, 3, 4, 4));

    let linear = LevelBasedValue::Linear {
        base: 0.1,
        per_level_above_first: 0.05,
    };
    assert!((linear.calculate(3) - 0.2).abs() < 1e-6);
    let lookup = LevelBasedValue::Lookup {
        values: vec![1.0, 2.0],
        fallback: Box::new(LevelBasedValue::LevelsSquared(1.0)),
    };
    assert_eq!((lookup.calculate(2), lookup.calculate(3)), (2.0, 10.0));
}
