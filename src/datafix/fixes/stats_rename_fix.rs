//! Port of `net.minecraft.util.datafix.fixes.StatsRenameFix`: renames custom
//! statistics in stats files and in scoreboard objective criteria.

use crate::datafix::dynamic::get_str;
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

fn renamed(renames: &[(&str, &str)], value: &str) -> String {
    renames
        .iter()
        .find(|(old, _)| *old == value)
        .map_or(value, |(_, new)| *new)
        .to_string()
}

/// `new StatsRenameFix(schema, name, renames)`.
pub fn stats_rename_fix(name: &str, renames: &'static [(&'static str, &'static str)]) -> Fix {
    Fix::sequence(
        name,
        vec![
            // `createStatRule`: keys of `stats["minecraft:custom"]`.
            Fix::everywhere(name, Target::Type(r::STATS), move |stats| {
                let Some(stats) = typed_mut(stats, "stats") else {
                    return;
                };
                let Some(Tag::Compound(custom)) = typed_mut(stats, "minecraft:custom") else {
                    return;
                };
                for (key, _) in custom.iter_mut() {
                    *key = renamed(renames, key);
                }
            }),
            // `createCriteriaRule`: `id` of `minecraft:custom` criteria.
            Fix::everywhere(name, Target::Type(r::OBJECTIVE), move |objective| {
                let Some(criteria_type) = typed_mut(objective, "CriteriaType") else {
                    return;
                };
                if get_str(criteria_type, "type") != Some("minecraft:custom") {
                    return;
                }
                if let Some(Tag::String(id)) = typed_mut(criteria_type, "id") {
                    *id = renamed(renames, id);
                }
            }),
        ],
    )
}
