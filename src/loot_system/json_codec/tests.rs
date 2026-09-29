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
fn an_empty_table_decodes_and_a_table_with_unsupported_parts_is_rejected() {
    let empty = decode(r#"{"pools":[]}"#).unwrap_or_else(|e| panic!("{e}"));
    assert!(empty.pools.is_empty() && empty.functions.is_empty());

    for (json, needle) in [
        (
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone",
               "functions":[{"function":"minecraft:enchant_randomly"}]}]}]}"#,
            "unsupported loot function minecraft:enchant_randomly",
        ),
        (
            r#"{"pools":[{"rolls":1,"conditions":[{"condition":"minecraft:match_tool"}],"entries":[]}]}"#,
            "unsupported loot condition minecraft:match_tool",
        ),
        (
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:composter"}]}]}"#,
            "unsupported loot entry type minecraft:composter",
        ),
        (r#"{"pools":[{"entries":[]}]}"#, "missing rolls"),
        (r#"{"pools":{}}"#, "pools is not a list"),
        ("[]", "loot table is not a JSON object"),
        ("{", "invalid JSON"),
    ] {
        let error = decode(json).expect_err(json);
        assert!(error.contains(needle), "{json}: {error}");
    }
}
