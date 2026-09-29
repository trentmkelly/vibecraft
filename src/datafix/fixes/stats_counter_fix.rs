//! Port of `net.minecraft.util.datafix.fixes.StatsCounterFix`: flat `stat.*`
//! counters become the structured `stats` object, and scoreboard objectives get
//! structured criteria.

use crate::datafix::decode::decode;
use crate::datafix::dynamic::{as_f64, get_str};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::schemas::v1451_6::pack_namespaced_with_dot;
use crate::datafix::template::Tmpl;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::block_state_data::upgrade_block_name;
use super::item_stack_the_flattening_fix::update_item;
use super::stats_counter_fix_tables::{
    CUSTOM_MAP, ENTITIES, ENTITY_KEYS, ITEM_KEYS, SKIP, SPECIAL_OBJECTIVE_CRITERIA,
};

/// `StatsCounterFix.StatType`.
struct StatType {
    kind: String,
    type_key: String,
}

fn lookup(table: &[(&str, &str)], key: &str) -> Option<String> {
    table
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| (*value).to_string())
}

/// `StringUtils.ordinalIndexOf(str, ".", 2)`.
fn second_dot(key: &str) -> Option<usize> {
    let first = key.find('.')?;
    key[first + 1..].find('.').map(|index| first + 1 + index)
}

/// `StatsCounterFix.unpackLegacyKey`.
fn unpack_legacy_key(key: &str) -> Option<StatType> {
    if SKIP.contains(&key) {
        return None;
    }
    if let Some(custom_key) = lookup(CUSTOM_MAP, key) {
        return Some(StatType {
            kind: "minecraft:custom".to_string(),
            type_key: custom_key,
        });
    }
    let split_index = second_dot(key)?;
    let prefix = &key[..split_index];
    let rest = key[split_index + 1..].replace('.', ":");
    if prefix == "stat.mineBlock" {
        return Some(StatType {
            kind: "minecraft:mined".to_string(),
            type_key: upgrade_block_name(&rest),
        });
    }
    if let Some(item_key) = lookup(ITEM_KEYS, prefix) {
        let type_key = update_item(&rest, 0).unwrap_or(rest);
        return Some(StatType {
            kind: item_key,
            type_key,
        });
    }
    if let Some(entity_key) = lookup(ENTITY_KEYS, prefix) {
        let type_key = lookup(ENTITIES, &rest).unwrap_or(rest);
        return Some(StatType {
            kind: entity_key,
            type_key,
        });
    }
    None
}

/// `new StatsCounterFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::sequence("StatsCounterFix", vec![stat_fixer(), objective_fixer()])
}

/// `makeStatFixer`: the whole stats object is rebuilt from the flat counters.
fn stat_fixer() -> Fix {
    Fix::everywhere_with(
        "StatsCounterFix",
        Target::Type(r::STATS),
        |ctx, stats_tag| {
            let mut stats: Vec<(String, Tag)> = Vec::new();
            if let Tag::Compound(entries) = stats_tag {
                for (key, value) in entries.iter() {
                    if as_f64(value).is_none() {
                        continue;
                    }
                    let Some(stat_type) = unpack_legacy_key(key) else {
                        continue;
                    };
                    let index = match stats.iter().position(|(kind, _)| *kind == stat_type.kind) {
                        Some(index) => index,
                        None => {
                            stats.push((stat_type.kind.clone(), Tag::Compound(Vec::new())));
                            stats.len() - 1
                        }
                    };
                    crate::datafix::dynamic::set(
                        &mut stats[index].1,
                        &stat_type.type_key,
                        value.clone(),
                    );
                }
            }
            let mut rebuilt = Tag::Compound(vec![("stats".to_string(), Tag::Compound(stats))]);
            if decode(ctx.output, &Tmpl::Ref(r::STATS), &mut rebuilt).is_ok() {
                *stats_tag = rebuilt;
            }
        },
    )
}

/// `makeObjectiveFixer`: criteria names of the old flat form are repacked.
fn objective_fixer() -> Fix {
    Fix::everywhere_with(
        "ObjectiveStatFix",
        Target::Type(r::OBJECTIVE),
        |ctx, objective| {
            let mut updated = objective.clone();
            if let Some(key) = get_str(objective, "CriteriaName").map(str::to_string) {
                let name = if SPECIAL_OBJECTIVE_CRITERIA.contains(&key.as_str()) {
                    key
                } else {
                    match unpack_legacy_key(&key) {
                        None => "dummy".to_string(),
                        Some(stat_type) => format!(
                            "{}:{}",
                            pack_namespaced_with_dot(&stat_type.kind),
                            pack_namespaced_with_dot(&stat_type.type_key)
                        ),
                    }
                };
                crate::datafix::dynamic::set(&mut updated, "CriteriaName", Tag::String(name));
            }
            if decode(ctx.output, &Tmpl::Ref(r::OBJECTIVE), &mut updated).is_ok() {
                *objective = updated;
            }
        },
    )
}
