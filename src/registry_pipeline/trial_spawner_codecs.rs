//! Codec for `trial_spawner` (`TrialSpawnerConfig.DIRECT_CODEC`) with `SpawnData` and
//! `EquipmentTable`.

use serde_json::json;

use crate::registry_pipeline::codec::{
    custom_field, describe, either, enum_codec, float_codec, float_range, identifier_codec,
    int_codec, int_range, opt, opt_default, parse_identifier, record, req, unbounded_map, Codec,
    Field,
};
use crate::registry_pipeline::worldgen_common::{compound_tag, interval_codec, weighted_list};
use crate::storage::nbt::Tag;

/// `EquipmentSlot` serialised names.
const EQUIPMENT_SLOTS: &[&str] = &[
    "mainhand", "offhand", "feet", "legs", "chest", "head", "body", "saddle",
];

/// `TrialSpawnerConfig.DIRECT_CODEC`.
pub fn trial_spawner_config() -> Codec {
    let non_negative_float = || float_range(0.0, f32::MAX);
    record(vec![
        opt_default("spawn_range", int_range(1, 128), json!(4)),
        opt_default("total_mobs", non_negative_float(), json!(6.0)),
        opt_default("simultaneous_mobs", non_negative_float(), json!(2.0)),
        opt_default(
            "total_mobs_added_per_player",
            non_negative_float(),
            json!(2.0),
        ),
        opt_default(
            "simultaneous_mobs_added_per_player",
            non_negative_float(),
            json!(1.0),
        ),
        opt_default("ticks_between_spawn", int_range(0, i32::MAX), json!(40)),
        opt_default("spawn_potentials", weighted_list(spawn_data()), json!([])),
        opt_default(
            "loot_tables_to_eject",
            weighted_list(identifier_codec()),
            json!([
                {"data": "minecraft:spawners/trial_chamber/consumables", "weight": 1},
                {"data": "minecraft:spawners/trial_chamber/key", "weight": 1}
            ]),
        ),
        opt_default(
            "items_to_drop_when_ominous",
            identifier_codec(),
            json!("minecraft:spawners/trial_chamber/items_to_drop_when_ominous"),
        ),
    ])
}

/// `SpawnData.CODEC`.
fn spawn_data() -> Codec {
    record(vec![
        req("entity", normalized_entity()),
        opt("custom_spawn_rules", custom_spawn_rules()),
        opt("equipment", equipment_table()),
    ])
}

/// `CompoundTag.CODEC` plus the `SpawnData` compact constructor: the entity `id` is
/// normalised to an identifier, or removed when it is not one.
fn normalized_entity() -> Codec {
    let inner = compound_tag();
    Codec::new(move |json, ctx| {
        let Tag::Compound(fields) = inner.parse(json, ctx)? else {
            return Err(format!("Not a compound tag: {}", describe(json)));
        };
        let fields = fields
            .into_iter()
            .filter_map(|(key, value)| {
                if key != "id" {
                    return Some((key, value));
                }
                match &value {
                    Tag::String(text) => parse_identifier(&serde_json::Value::String(text.clone()))
                        .ok()
                        .map(|id| (key, Tag::String(id.to_string()))),
                    _ => None,
                }
            })
            .collect();
        Ok(Tag::Compound(fields))
    })
}

/// `SpawnData.CustomSpawnRules.CODEC`.
fn custom_spawn_rules() -> Codec {
    record(vec![
        light_limit("block_light_limit"),
        light_limit("sky_light_limit"),
    ])
}

/// `InclusiveRange.INT.lenientOptionalFieldOf(name, 0..15).validate(...)`: an
/// undecodable range counts as absent, but a decoded range outside `0..=15` is an error
/// (the validation wraps the whole optional component). Ranges equal to the default are
/// not written.
fn light_limit(name: &'static str) -> Field {
    custom_field(move |object, ctx, out| {
        let Some(value) = object.get(name) else {
            return Ok(());
        };
        let Ok(range) = inclusive_int_range().parse(value, ctx) else {
            return Ok(());
        };
        let (low, high) = match &range {
            Tag::Int(value) => (*value, *value),
            Tag::List(items) => match items.as_slice() {
                [Tag::Int(low), Tag::Int(high)] => (*low, *high),
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };
        if low < 0 || high > 15 {
            return Err("Light values must be withing range InclusiveRange[0, 15]".to_string());
        }
        if (low, high) != (0, 15) {
            out.push((name.to_string(), range));
        }
        Ok(())
    })
}

/// `InclusiveRange.INT`.
fn inclusive_int_range() -> Codec {
    interval_codec(int_codec(), "min_inclusive", "max_inclusive", |min, max| {
        let (Tag::Int(low), Tag::Int(high)) = (min, max) else {
            return Err("InclusiveRange bounds must be ints".to_string());
        };
        if low > high {
            Err("min_inclusive must be less than or equal to max_inclusive".to_string())
        } else {
            Ok((min.clone(), max.clone()))
        }
    })
}

/// `EquipmentTable.CODEC`.
fn equipment_table() -> Codec {
    record(vec![
        req("loot_table", identifier_codec()),
        opt_default("slot_drop_chances", drop_chances(), json!({})),
    ])
}

/// `EquipmentTable.DROP_CHANCES_CODEC`: a float for every slot, or a per-slot map.
/// A map that covers every slot with one value is written back as that float.
fn drop_chances() -> Codec {
    let per_slot = unbounded_map(enum_codec(EQUIPMENT_SLOTS), float_codec());
    either(float_codec(), per_slot).map_tag(|tag| {
        Ok(match tag {
            Tag::Compound(fields) => {
                let all_present = EQUIPMENT_SLOTS
                    .iter()
                    .all(|slot| fields.iter().any(|(key, _)| key == slot));
                let first = fields.first().map(|(_, value)| value.clone());
                let same = first
                    .as_ref()
                    .is_some_and(|first| fields.iter().all(|(_, value)| value == first));
                match first {
                    Some(value) if all_present && same => value,
                    _ => Tag::Compound(fields),
                }
            }
            other => other,
        })
    })
}
