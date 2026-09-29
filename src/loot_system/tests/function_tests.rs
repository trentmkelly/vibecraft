//! Behaviour of the component, predicate and inline-table parts of the loot model,
//! checked against the Java semantics of the corresponding classes.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;
use crate::loot_system::world_view::{EntityFlags, LootBlockState, RaiderState};

fn function(json: &str) -> LootFunction {
    let table = format!(
        r#"{{"pools":[{{"rolls":1,"entries":[{{"type":"minecraft:item","name":"minecraft:stone","functions":[{json}]}}]}}]}}"#
    );
    match &decode_loot_table(&table).expect("decodes").pools[0].entries[0] {
        LootEntry::Item { functions, .. } => functions[0].clone(),
        other => panic!("{other:?}"),
    }
}

fn condition(json: &str) -> LootCondition {
    let table = format!(
        r#"{{"pools":[{{"rolls":1,"conditions":[{json}],"entries":[]}}]}}"#
    );
    decode_loot_table(&table).expect("decodes").pools[0].conditions[0].clone()
}

fn context() -> LootContext {
    let mut context = LootContext::new(LootParamSet::AllParams, 7);
    context.insert_param(LootParamValue::Origin(10.5, 64.0, -3.5));
    context
}

#[test]
fn set_damage_follows_set_item_damage_function() {
    let sword = LootStack::new("minecraft:diamond_sword", 1);
    let f = function(r#"{"function":"minecraft:set_damage","damage":0.25}"#);
    // 1 - 0.25 = 0.75 of 1561 durability, floored.
    assert_eq!(f.apply(sword.clone(), &mut context()).unwrap().damage_value(), 1170);
    // `add` starts from the existing damage fraction (100/1561 used).
    let mut used = sword.clone();
    used.set_damage_value(100);
    let f = function(r#"{"function":"minecraft:set_damage","damage":0.5,"add":true}"#);
    let out = f.apply(used, &mut context()).unwrap();
    let base = 1.0 - 100.0_f32 / 1561.0;
    assert_eq!(out.damage_value(), ((1.0 - (0.5 + base).clamp(0.0, 1.0)) * 1561.0).floor() as i32);
    // Non-damageable items are left alone.
    let stone = function(r#"{"function":"minecraft:set_damage","damage":0.5}"#)
        .apply(LootStack::new("minecraft:stone", 1), &mut context())
        .unwrap();
    assert!(!stone.components.contains_key("minecraft:damage"));
}

#[test]
fn set_enchantments_converts_books_and_clamps_levels() {
    let f = function(
        r#"{"function":"minecraft:set_enchantments","enchantments":{"minecraft:sharpness":300,"minecraft:mending":0}}"#,
    );
    let book = f
        .apply(LootStack::new("minecraft:book", 2), &mut context())
        .unwrap();
    assert_eq!(book.item, "minecraft:enchanted_book");
    assert_eq!(book.count, 2);
    assert_eq!(book.components["minecraft:stored_enchantments"], "minecraft:sharpness:255");
    let add = function(
        r#"{"function":"minecraft:set_enchantments","add":true,"enchantments":{"minecraft:sharpness":2}}"#,
    );
    let mut sword = LootStack::new("minecraft:diamond_sword", 1);
    sword.enchant("minecraft:sharpness", 3);
    let sword = add.apply(sword, &mut context()).unwrap();
    assert_eq!(sword.components["minecraft:enchantments"], "minecraft:sharpness:5");
}

#[test]
fn set_stew_effect_scales_durations_of_non_instant_effects() {
    let f = function(
        r#"{"function":"minecraft:set_stew_effect","effects":[{"type":"minecraft:saturation","duration":7}]}"#,
    );
    let stew = f
        .apply(LootStack::new("minecraft:suspicious_stew", 1), &mut context())
        .unwrap();
    // Saturation is instantaneous: the duration is not multiplied by 20.
    assert_eq!(stew.components["minecraft:suspicious_stew_effects"], "minecraft:saturation:7");
    let f = function(
        r#"{"function":"minecraft:set_stew_effect","effects":[{"type":"minecraft:poison","duration":7}]}"#,
    );
    let stew = f
        .apply(LootStack::new("minecraft:suspicious_stew", 1), &mut context())
        .unwrap();
    assert_eq!(stew.components["minecraft:suspicious_stew_effects"], "minecraft:poison:140");
    let other = f.apply(LootStack::new("minecraft:stone", 1), &mut context()).unwrap();
    assert!(other.components.is_empty());
}

#[test]
fn duplicate_stew_effects_are_rejected() {
    let table = r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone","functions":[{"function":"minecraft:set_stew_effect","effects":[{"type":"minecraft:poison","duration":1},{"type":"minecraft:poison","duration":2}]}]}]}]}"#;
    assert!(decode_loot_table(table).is_err());
}

#[test]
fn set_ominous_bottle_amplifier_clamps_to_four() {
    let out = function(r#"{"function":"minecraft:set_ominous_bottle_amplifier","amplifier":9}"#)
        .apply(LootStack::new("minecraft:ominous_bottle", 1), &mut context())
        .unwrap();
    assert_eq!(out.components["minecraft:ominous_bottle_amplifier"], "4");
}

#[test]
fn set_instrument_draws_from_the_option_set() {
    let f = function(
        r#"{"function":"minecraft:set_instrument","options":["minecraft:ponder_goat_horn"]}"#,
    );
    let out = f.apply(LootStack::new("minecraft:goat_horn", 1), &mut context()).unwrap();
    assert_eq!(out.components["minecraft:instrument"], "minecraft:ponder_goat_horn");
    let empty = function(r#"{"function":"minecraft:set_instrument","options":[]}"#)
        .apply(LootStack::new("minecraft:goat_horn", 1), &mut context())
        .unwrap();
    assert!(empty.components.is_empty());
}

#[test]
fn set_name_writes_the_selected_component() {
    let f = function(
        r#"{"function":"minecraft:set_name","name":{"translate":"x.y"},"target":"item_name"}"#,
    );
    let out = f.apply(LootStack::new("minecraft:map", 1), &mut context()).unwrap();
    assert_eq!(out.components["minecraft:item_name"], r#"{"translate":"x.y"}"#);
    // A component that needs an entity to resolve stays unmodeled.
    let table = r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone","functions":[{"function":"minecraft:set_name","entity":"this","name":{"selector":"@s"}}]}]}]}"#;
    assert!(!decode_loot_table(table).unwrap().is_fully_modeled());
}

#[test]
fn copy_state_copies_only_present_block_properties() {
    let f = function(
        r#"{"function":"minecraft:copy_state","block":"minecraft:beehive","properties":["honey_level","not_a_property"]}"#,
    );
    let LootFunction::CopyState { properties, .. } = &f else {
        panic!("{f:?}")
    };
    assert_eq!(properties, &["honey_level".to_string()]);
    let mut ctx = context();
    assert_eq!(f.apply(LootStack::new("minecraft:beehive", 1), &mut ctx).unwrap().components, Default::default());
    ctx.insert_param(LootParamValue::BlockState("minecraft:beehive".to_string()));
    ctx.block_state_properties.insert("honey_level".into(), "5".into());
    let out = f.apply(LootStack::new("minecraft:beehive", 1), &mut ctx).unwrap();
    assert_eq!(out.components["minecraft:block_state"], "honey_level=5");
}

#[test]
fn copy_components_honours_include_exclude_and_source_presence() {
    let f = function(
        r#"{"function":"minecraft:copy_components","source":"block_entity","include":["minecraft:custom_name","minecraft:lock"],"exclude":["minecraft:lock"]}"#,
    );
    let mut ctx = context();
    assert!(f.apply(LootStack::new("minecraft:chest", 1), &mut ctx).unwrap().components.is_empty());
    ctx.insert_param(LootParamValue::BlockEntity("minecraft:chest".to_string()));
    ctx.block_entity_components = BTreeMap::from([
        ("minecraft:custom_name".to_string(), "\"Box\"".to_string()),
        ("minecraft:lock".to_string(), "key".to_string()),
        ("minecraft:rarity".to_string(), "rare".to_string()),
    ]);
    let out = f.apply(LootStack::new("minecraft:chest", 1), &mut ctx).unwrap();
    assert_eq!(out.components.len(), 1);
    assert_eq!(out.components["minecraft:custom_name"], "\"Box\"");
    assert!(decode_loot_table(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone","functions":[{"function":"minecraft:copy_components","source":"block_entity","include":["minecraft:nope"]}]}]}]}"#
    )
    .is_err());
}

struct TestLevel;

impl LootLevel for TestLevel {
    fn dimension(&self) -> String {
        "minecraft:overworld".to_string()
    }
    fn is_loaded(&self, _: i32, y: i32, _: i32) -> bool {
        y < 100
    }
    fn biome(&self, _: i32, _: i32, _: i32) -> String {
        "minecraft:jungle".to_string()
    }
    fn block_state(&self, _: i32, y: i32, _: i32) -> LootBlockState {
        LootBlockState {
            block: "minecraft:tall_grass".to_string(),
            properties: BTreeMap::from([(
                "half".to_string(),
                if y == 63 { "lower" } else { "upper" }.to_string(),
            )]),
        }
    }
}

#[test]
fn location_check_uses_origin_offset_biomes_and_block_state() {
    let block = condition(
        r#"{"condition":"minecraft:location_check","offsetY":-1,"predicate":{"block":{"blocks":"minecraft:tall_grass","state":{"half":"lower"}}}}"#,
    );
    let biome = condition(
        r#"{"condition":"minecraft:location_check","predicate":{"biomes":["minecraft:jungle","minecraft:sparse_jungle"]}}"#,
    );
    let mut ctx = context();
    assert!(!block.matches(&ctx), "no level attached: nothing is loaded");
    ctx.level = Some(Arc::new(TestLevel));
    assert!(block.matches(&ctx));
    assert!(biome.matches(&ctx));
    ctx.insert_param(LootParamValue::Origin(0.0, 200.0, 0.0));
    assert!(!biome.matches(&ctx), "unloaded position fails the biome check");
}

#[test]
fn entity_properties_match_components_flags_vehicle_and_type_specific() {
    let mut ctx = context();
    let sheep = LootEntity::new("minecraft:sheep")
        .living()
        .with_component("minecraft:sheep/color", &serde_json::json!("black"))
        .with_sheared(false);
    ctx.insert_entity(LootEntityTarget::This, sheep);
    let wool = condition(
        r#"{"condition":"minecraft:entity_properties","entity":"this","predicate":{"components":{"minecraft:sheep/color":"black"},"type_specific":{"type":"minecraft:sheep","sheared":false}}}"#,
    );
    assert!(wool.matches(&ctx));
    let white = condition(
        r#"{"condition":"minecraft:entity_properties","entity":"this","predicate":{"components":{"minecraft:sheep/color":"white"}}}"#,
    );
    assert!(!white.matches(&ctx));

    let zombie = LootEntity::new("minecraft:zombie")
        .living()
        .with_flags(EntityFlags {
            on_fire: true,
            baby: true,
            ..EntityFlags::default()
        })
        .with_vehicle(LootEntity::new("minecraft:chicken"));
    ctx.insert_entity(LootEntityTarget::Attacker, zombie);
    for json in [
        r#"{"flags":{"is_on_fire":true,"is_baby":true},"vehicle":{"type":"minecraft:chicken"}}"#,
        r#"{"type":"minecraft:zombie"}"#,
    ] {
        let c = condition(&format!(
            r#"{{"condition":"minecraft:entity_properties","entity":"attacker","predicate":{json}}}"#
        ));
        assert!(c.matches(&ctx), "{json}");
    }
    let wrong_vehicle = condition(
        r#"{"condition":"minecraft:entity_properties","entity":"attacker","predicate":{"vehicle":{"type":"minecraft:zombie_horse"}}}"#,
    );
    assert!(!wrong_vehicle.matches(&ctx));
    // An entity that is not in the context never matches a predicate.
    let missing = condition(
        r#"{"condition":"minecraft:entity_properties","entity":"direct_attacker","predicate":{}}"#,
    );
    assert!(!missing.matches(&ctx));

    ctx.insert_entity(
        LootEntityTarget::DirectAttacker,
        LootEntity::new("minecraft:pillager").living().with_raider(RaiderState {
            has_raid: false,
            is_captain: true,
        }),
    );
    let captain = condition(
        r#"{"condition":"minecraft:entity_properties","entity":"direct_attacker","predicate":{"type_specific":{"type":"minecraft:raider","is_captain":true}}}"#,
    );
    assert!(captain.matches(&ctx));
}

#[test]
fn entity_equipment_predicate_reads_enchantments_of_the_held_item() {
    let mut ctx = context();
    let mut tags = RegistryTags::default();
    tags.insert("enchantment", "minecraft:smelts_loot", vec!["minecraft:fire_aspect".into()]);
    ctx.registry_tags = Arc::new(tags);
    let mut sword = LootStack::new("minecraft:iron_sword", 1);
    sword.enchant("minecraft:fire_aspect", 1);
    ctx.insert_entity(
        LootEntityTarget::DirectAttacker,
        LootEntity::new("minecraft:player").living().with_equipment("mainhand", sword),
    );
    let c = condition(
        r##"{"condition":"minecraft:entity_properties","entity":"direct_attacker","predicate":{"equipment":{"mainhand":{"predicates":{"minecraft:enchantments":[{"enchantments":"#minecraft:smelts_loot"}]}}}}}"##,
    );
    assert!(c.matches(&ctx));
    ctx.insert_entity(
        LootEntityTarget::DirectAttacker,
        LootEntity::new("minecraft:player").living(),
    );
    assert!(!c.matches(&ctx));
}

#[test]
fn damage_source_predicate_checks_tags_and_entities() {
    let mut ctx = context();
    let mut tags = RegistryTags::default();
    tags.insert("damage_type", "minecraft:is_lightning", vec!["minecraft:lightning_bolt".into()]);
    ctx.registry_tags = Arc::new(tags);
    let lightning = condition(
        r#"{"condition":"minecraft:damage_source_properties","predicate":{"tags":[{"id":"minecraft:is_lightning","expected":true}]}}"#,
    );
    assert!(!lightning.matches(&ctx), "no damage source in the context");
    ctx.insert_damage_source(LootDamageSource {
        damage_type: "minecraft:lightning_bolt".into(),
        ..LootDamageSource::default()
    });
    assert!(lightning.matches(&ctx));
    let frog = condition(
        r#"{"condition":"minecraft:damage_source_properties","predicate":{"source_entity":{"type":"minecraft:frog"}}}"#,
    );
    assert!(!frog.matches(&ctx));
    ctx.insert_damage_source(LootDamageSource {
        damage_type: "minecraft:mob_attack".into(),
        source_entity: Some(LootEntity::new("minecraft:frog")),
        ..LootDamageSource::default()
    });
    assert!(frog.matches(&ctx));
    assert!(!lightning.matches(&ctx));
}

#[test]
fn inline_loot_table_entries_run_their_table() {
    let table = decode_loot_table(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:apple"}]}]},"functions":[{"function":"minecraft:set_count","count":3}]}]}]}"#,
    )
    .expect("decodes");
    assert!(table.is_fully_modeled());
    let out = table.evaluate(&mut context());
    assert_eq!(out, vec![LootStack::new("minecraft:apple", 3)]);
}

#[test]
fn split_stacks_keeps_components_and_uses_the_item_stack_limit() {
    let mut stack = LootStack::new("minecraft:ender_pearl", 20);
    stack.components.insert("minecraft:custom_name".into(), "\"x\"".into());
    let split = context::split_stacks(vec![stack]);
    assert_eq!(split.iter().map(|s| s.count).collect::<Vec<_>>(), vec![16, 4]);
    assert!(split.iter().all(|s| s.components.contains_key("minecraft:custom_name")));
}

#[test]
fn lone_valid_pool_entry_is_chosen_without_drawing() {
    let table = decode_loot_table(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:apple","weight":0},{"type":"minecraft:item","name":"minecraft:stick"}]}]}"#,
    )
    .expect("decodes");
    let mut ctx = context();
    assert_eq!(table.evaluate(&mut ctx), vec![LootStack::new("minecraft:stick", 1)]);
    // No random number was consumed by the pool.
    let fresh = context();
    assert_eq!(ctx.random.next_i32(1000), fresh.random.next_i32(1000));
}
