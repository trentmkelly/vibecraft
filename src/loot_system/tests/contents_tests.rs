//! `SetContainerContents` and `ModifyContainerContents` codec/runtime parity.

use super::*;

fn context(seed: u64) -> LootContext {
    LootContext::new(LootParamSet::AllParams, seed)
}

#[test]
fn decoded_set_contents_expands_entries_splits_stacks_and_preserves_components() {
    let raw = r#"{
        "pools":[{
            "rolls":1,
            "entries":[{
                "type":"minecraft:item",
                "name":"minecraft:barrel",
                "functions":[{
                    "function":"minecraft:set_contents",
                    "component":"minecraft:container",
                    "entries":[{
                        "type":"minecraft:item",
                        "name":"minecraft:ender_pearl",
                        "weight":0,
                        "functions":[
                            {"function":"minecraft:set_count","count":40},
                            {"function":"minecraft:set_name","name":{"text":"kept"}}
                        ]
                    }]
                }]
            }]
        }]
    }"#;
    let table = decode_loot_table(raw).expect("Java-shaped function decodes");
    let mut ctx = context(0x1234);
    let result = table.evaluate(&mut ctx);
    assert_eq!(result.len(), 1);

    let contents: serde_json::Value =
        serde_json::from_str(&result[0].components["minecraft:container"]).unwrap();
    let slots = contents.as_array().expect("sparse slot array");
    assert_eq!(slots.len(), 3);
    let counts: Vec<_> = slots
        .iter()
        .map(|slot| slot["item"]["count"].as_i64().unwrap_or(1))
        .collect();
    assert_eq!(counts, vec![16, 16, 8]);
    for slot in slots {
        assert_eq!(slot["item"]["id"], "minecraft:ender_pearl");
        assert_eq!(
            slot["item"]["components"]["minecraft:custom_name"]["text"],
            "kept"
        );
    }

    // The outer pool has one entry, and the contents expansion ignores weights, so
    // neither path draws from the context random source.
    let actual_next = ctx.random.next_i32(10_000);
    let expected_next = context(0x1234).random.next_i32(10_000);
    assert_eq!(actual_next, expected_next);
}

#[test]
fn decoded_modify_contents_keeps_slots_and_clamps_to_modified_item_maximum() {
    let raw = r#"{
        "pools":[{
            "rolls":1,
            "entries":[{
                "type":"minecraft:item",
                "name":"minecraft:barrel",
                "functions":[{
                    "function":"minecraft:modify_contents",
                    "component":"minecraft:container",
                    "modifier":{"function":"minecraft:set_count","count":70}
                }]
            }]
        }]
    }"#;
    let table = decode_loot_table(raw).expect("modifier decodes");
    let LootEntry::Item { functions, .. } = &table.pools[0].entries[0] else {
        panic!("expected item entry")
    };
    let mut barrel = LootStack::new("minecraft:barrel", 1);
    barrel.components.insert(
        "minecraft:container".to_string(),
        r#"[{"slot":0,"item":{"id":"minecraft:ender_pearl","count":2,"components":{"minecraft:max_stack_size":16}}},{"slot":3,"item":{"id":"minecraft:arrow","count":1}}]"#.to_string(),
    );
    let mut ctx = context(8);
    let barrel = functions
        .iter()
        .try_fold(barrel, |stack, function| function.apply(stack, &mut ctx))
        .expect("modifier keeps the barrel");
    let contents: serde_json::Value =
        serde_json::from_str(&barrel.components["minecraft:container"]).unwrap();
    let slots = contents.as_array().unwrap();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0]["slot"], 0);
    assert_eq!(slots[0]["item"]["count"], 16);
    assert_eq!(slots[1]["slot"], 3);
    assert_eq!(slots[1]["item"]["count"], 64);
}

#[test]
fn charged_projectiles_use_stack_arrays_and_bundle_contents_stay_validated_unmodeled() {
    let raw = r#"{
        "pools":[{
            "rolls":1,
            "entries":[{
                "type":"minecraft:item",
                "name":"minecraft:crossbow",
                "functions":[{
                    "function":"minecraft:set_contents",
                    "component":"minecraft:charged_projectiles",
                    "entries":[{
                        "type":"minecraft:item",
                        "name":"minecraft:arrow"
                    }]
                }]
            }]
        }]
    }"#;
    let table = decode_loot_table(raw).expect("charged projectiles decode");
    let mut ctx = context(3);
    let crossbow = table.evaluate(&mut ctx).remove(0);
    let contents: serde_json::Value =
        serde_json::from_str(&crossbow.components["minecraft:charged_projectiles"]).unwrap();
    assert_eq!(contents, serde_json::json!([{"id":"minecraft:arrow"}]));

    let bundle = r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:bundle","functions":[{"function":"minecraft:set_contents","component":"minecraft:bundle_contents","entries":[]}]}]}]}"#;
    let bundle = decode_loot_table(bundle).expect("registered bundle manipulator validates");
    let LootEntry::Item { functions, .. } = &bundle.pools[0].entries[0] else {
        panic!("expected item entry")
    };
    assert!(matches!(functions[0], LootFunction::Unmodeled { .. }));
    assert!(!bundle.is_fully_modeled());
}

#[test]
fn contents_entry_expansion_keeps_sequence_prefix_and_stops_at_empty_tag() {
    let raw = r#"{
        "pools":[{
            "rolls":1,
            "entries":[{
                "type":"minecraft:item",
                "name":"minecraft:barrel",
                "functions":[{
                    "function":"minecraft:set_contents",
                    "component":"minecraft:container",
                    "entries":[
                        {
                            "type":"minecraft:alternatives",
                            "children":[
                                {"type":"minecraft:tag","name":"minecraft:empty","expand":true},
                                {"type":"minecraft:item","name":"minecraft:gold_ingot"}
                            ]
                        },
                        {
                            "type":"minecraft:sequence",
                            "children":[
                                {"type":"minecraft:item","name":"minecraft:diamond"},
                                {"type":"minecraft:empty","conditions":[{"condition":"minecraft:random_chance","chance":0}]},
                                {"type":"minecraft:item","name":"minecraft:gold_ingot"}
                            ]
                        }
                    ]
                }]
            }]
        }]
    }"#;
    let table = decode_loot_table(raw).expect("contents entry graph decodes");
    let mut ctx = context(0x55aa);
    ctx.tags.insert("minecraft:empty".to_string(), Vec::new());
    let result = table.evaluate(&mut ctx).remove(0);
    let contents: serde_json::Value =
        serde_json::from_str(&result.components["minecraft:container"]).unwrap();
    assert_eq!(
        contents,
        serde_json::json!([{"slot":0,"item":{"id":"minecraft:diamond"}}])
    );
}

#[test]
fn charged_projectile_modify_discards_items_and_missing_component_is_untouched() {
    let raw = r#"{
        "pools":[{
            "rolls":1,
            "entries":[{
                "type":"minecraft:item",
                "name":"minecraft:crossbow",
                "functions":[{
                    "function":"minecraft:modify_contents",
                    "component":"minecraft:charged_projectiles",
                    "modifier":{"function":"minecraft:discard"}
                }]
            }]
        }]
    }"#;
    let table = decode_loot_table(raw).expect("charged modifier decodes");
    let LootEntry::Item { functions, .. } = &table.pools[0].entries[0] else {
        panic!("expected item entry")
    };
    let function = &functions[0];

    let mut crossbow = LootStack::new("minecraft:crossbow", 1);
    crossbow.components.insert(
        "minecraft:charged_projectiles".to_string(),
        r#"[{"id":"minecraft:arrow"}]"#.to_string(),
    );
    let discarded = function
        .apply(crossbow, &mut context(9))
        .expect("function preserves the crossbow");
    assert_eq!(discarded.components["minecraft:charged_projectiles"], "[]");

    let empty_crossbow = LootStack::new("minecraft:crossbow", 1);
    let untouched = function
        .apply(empty_crossbow.clone(), &mut context(9))
        .expect("function preserves the crossbow");
    assert_eq!(untouched, empty_crossbow);
}

#[test]
fn decoded_contents_preserve_generated_empty_slots_and_string_components() {
    let table = decode_loot_table(
        r#"{"pools":[{"rolls":1,"entries":[{
        "type":"minecraft:item","name":"minecraft:barrel","functions":[
            {"function":"minecraft:set_contents","component":"minecraft:container","entries":[
                {"type":"minecraft:dynamic","name":"minecraft:test/gaps"}
            ]},
            {"function":"minecraft:modify_contents","component":"minecraft:container","modifier":[
                {"function":"minecraft:set_count","count":30}
            ]}
        ]
    }]}]}"#,
    )
    .unwrap();
    let mut ctx = context(3);
    let mut arrow = LootStack::new("minecraft:arrow", 1);
    arrow
        .components
        .insert("minecraft:custom_name".into(), "kept".into());
    arrow
        .components
        .insert("minecraft:max_stack_size".into(), "4".into());
    ctx.dynamic_drops.insert(
        "minecraft:test/gaps".into(),
        vec![LootStack::new("minecraft:air", 0), arrow],
    );
    let barrel = table.evaluate(&mut ctx).remove(0);
    let contents: serde_json::Value =
        serde_json::from_str(&barrel.components["minecraft:container"]).unwrap();
    assert_eq!(
        contents,
        serde_json::json!([{"slot":1,"item":{"id":"minecraft:arrow","count":4,"components":{"minecraft:custom_name":"kept","minecraft:max_stack_size":4}}}])
    );
}

#[test]
#[should_panic(expected = "Got 257 items, but maximum is 256")]
fn decoded_set_contents_rejects_container_construction_above_256_slots() {
    let table = decode_loot_table(
        r#"{"pools":[{"rolls":1,"entries":[{
        "type":"minecraft:item","name":"minecraft:barrel","functions":[{
            "function":"minecraft:set_contents","component":"minecraft:container","entries":[{
                "type":"minecraft:item","name":"minecraft:diamond_sword",
                "functions":[{"function":"minecraft:set_count","count":257}]
            }]
        }]
    }]}]}"#,
    )
    .unwrap();
    table.evaluate(&mut context(3));
}

#[test]
fn decoded_contents_expanded_tags_bypass_functions_without_random_draws() {
    let table = decode_loot_table(r#"{"pools":[{"rolls":1,"entries":[{
        "type":"minecraft:item","name":"minecraft:barrel","functions":[{
            "function":"minecraft:set_contents","component":"minecraft:container","entries":[
                {"type":"minecraft:tag","name":"minecraft:test/items","expand":true,
                 "functions":[{"function":"minecraft:set_count","count":{"type":"minecraft:uniform","min":5,"max":10}}]},
                {"type":"minecraft:tag","name":"minecraft:test/items","expand":false,
                 "functions":[{"function":"minecraft:set_count","count":2}]}
            ]
        }]
    }]}]}"#).unwrap();
    let mut ctx = context(19);
    ctx.tags.insert(
        "minecraft:test/items".into(),
        vec!["minecraft:apple".into(), "minecraft:diamond".into()],
    );
    let barrel = table.evaluate(&mut ctx).remove(0);
    let contents: serde_json::Value =
        serde_json::from_str(&barrel.components["minecraft:container"]).unwrap();
    assert_eq!(
        contents,
        serde_json::json!([
            {"slot":0,"item":{"id":"minecraft:apple"}},
            {"slot":1,"item":{"id":"minecraft:diamond"}},
            {"slot":2,"item":{"id":"minecraft:apple","count":2}},
            {"slot":3,"item":{"id":"minecraft:diamond","count":2}}
        ])
    );
    assert_eq!(
        ctx.random.next_i32(10_000),
        context(19).random.next_i32(10_000)
    );
}

#[test]
fn decoded_nested_contents_roundtrip_typed_enchantments_and_block_state() {
    let table = decode_loot_table(r#"{"pools":[{"rolls":1,"entries":[{
        "type":"minecraft:item","name":"minecraft:barrel","functions":[
            {"function":"minecraft:set_contents","component":"minecraft:container","entries":[
                {"type":"minecraft:dynamic","name":"minecraft:test/components"}
            ]},
            {"function":"minecraft:modify_contents","component":"minecraft:container","modifier":
                {"function":"minecraft:set_enchantments","enchantments":{"minecraft:mending":2},"add":true}}
        ]
    }]}]}"#).unwrap();
    let mut ctx = context(3);
    let mut item = LootStack::new("minecraft:diamond_pickaxe", 1);
    item.components.insert(
        "minecraft:enchantments".into(),
        "minecraft:mending:1".into(),
    );
    item.components.insert(
        "minecraft:stored_enchantments".into(),
        "minecraft:unbreaking:2".into(),
    );
    item.components.insert(
        "minecraft:block_state".into(),
        "facing=north,waterlogged=true".into(),
    );
    ctx.dynamic_drops
        .insert("minecraft:test/components".into(), vec![item]);
    let barrel = table.evaluate(&mut ctx).remove(0);
    let contents: serde_json::Value =
        serde_json::from_str(&barrel.components["minecraft:container"]).unwrap();
    let components = &contents[0]["item"]["components"];
    assert_eq!(
        components["minecraft:enchantments"],
        serde_json::json!({"minecraft:mending":3})
    );
    assert_eq!(
        components["minecraft:stored_enchantments"],
        serde_json::json!({"minecraft:unbreaking":2})
    );
    assert_eq!(
        components["minecraft:block_state"],
        serde_json::json!({"facing":"north","waterlogged":"true"})
    );
}
