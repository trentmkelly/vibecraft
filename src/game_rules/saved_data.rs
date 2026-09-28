//! `GameRuleMap.TYPE` saved data (`data/minecraft/game_rules.dat`).
//!
//! 26.1.2 no longer stores rules in `level.dat`; they live in the `game_rules` `SavedData`
//! whose codec is `Codec.dispatchedMap(GAME_RULE.byNameCodec(), GameRule::valueCodec)`: a
//! compound keyed by the rule identifier (`minecraft:keep_inventory`) whose values are NBT
//! bytes (`Codec.BOOL`) or ints (`Codec.intRange`).

use std::io;
use std::path::Path;

use super::{
    game_rule_definition, vanilla_game_rules, GameRuleType, GameRuleValue, GameRules,
};
use crate::storage::nbt::Tag;
use crate::storage::saved_data::{ResourceLocation, SavedDataStorage};

/// Saved-data id of `GameRuleMap.TYPE`.
pub const GAME_RULES_SAVED_DATA_ID: (&str, &str) = ("minecraft", "game_rules");

fn saved_data_id() -> ResourceLocation {
    ResourceLocation {
        namespace: GAME_RULES_SAVED_DATA_ID.0.to_string(),
        path: GAME_RULES_SAVED_DATA_ID.1.to_string(),
    }
}

impl GameRules {
    /// Encodes the rules with `GameRuleMap.CODEC` (registry order).
    pub fn to_saved_tag(&self) -> Tag {
        Tag::Compound(
            vanilla_game_rules()
                .iter()
                .filter_map(|definition| {
                    let value = self.values.get(definition.name)?;
                    let tag = match value {
                        GameRuleValue::Bool(value) => Tag::Byte(i8::from(*value)),
                        GameRuleValue::Int(value) => Tag::Int(*value),
                    };
                    Some((format!("minecraft:{}", definition.name), tag))
                })
                .collect(),
        )
    }

    /// Decodes `GameRuleMap.CODEC` and then applies the `GameRules(FeatureFlagSet, GameRuleMap)`
    /// constructor: missing rules are reset to their defaults and rules whose feature is
    /// disabled are dropped. Entries that fail to decode (unknown id, wrong type, out of
    /// range) are ignored, so the affected rule falls back to its default.
    pub fn from_saved_tag(tag: &Tag, minecart_improvements: bool) -> Self {
        let mut rules = Self::new(minecart_improvements);
        let Tag::Compound(entries) = tag else {
            return rules;
        };
        for (key, value) in entries {
            let name = key.strip_prefix("minecraft:").unwrap_or(key);
            let Some(definition) = game_rule_definition(name) else {
                continue;
            };
            if !rules.values.contains_key(definition.name) {
                continue;
            }
            if let Some(decoded) = decode_value(value, definition) {
                rules.values.insert(definition.name, decoded);
            }
        }
        rules
    }

    /// Loads `data/minecraft/game_rules.dat` from `world_root`, or the defaults when absent.
    pub fn load_from_world(world_root: &Path, minecart_improvements: bool) -> io::Result<Self> {
        let data_folder = world_root.join("data");
        if !data_folder.join("minecraft/game_rules.dat").is_file() {
            return Ok(Self::new(minecart_improvements));
        }
        let mut storage = SavedDataStorage::new(data_folder);
        Ok(match storage.get(&saved_data_id())? {
            Some(tag) => Self::from_saved_tag(tag, minecart_improvements),
            None => Self::new(minecart_improvements),
        })
    }

    /// Writes `data/minecraft/game_rules.dat` under `world_root` (`SavedData` wrapper with
    /// `DataVersion`).
    pub fn save_to_world(&self, world_root: &Path) -> io::Result<()> {
        let data_folder = world_root.join("data");
        // `SavedDataStorage` validates paths against the (existing) data folder.
        std::fs::create_dir_all(&data_folder)?;
        let mut storage = SavedDataStorage::new(data_folder);
        let tag = self.to_saved_tag();
        storage.compute_if_absent(saved_data_id(), || tag)?;
        storage.schedule_save().map(|_| ())
    }
}

/// Decodes one NBT value with `Codec.BOOL` / `Codec.intRange(min, max)`.
fn decode_value(tag: &Tag, definition: &super::GameRuleDefinition) -> Option<GameRuleValue> {
    let number = match tag {
        Tag::Byte(value) => i64::from(*value),
        Tag::Short(value) => i64::from(*value),
        Tag::Int(value) => i64::from(*value),
        Tag::Long(value) => *value,
        _ => return None,
    };
    match definition.rule_type {
        GameRuleType::Bool => Some(GameRuleValue::Bool(number != 0)),
        GameRuleType::Int => {
            let value = i32::try_from(number).ok()?;
            let in_range = definition.min.is_none_or(|min| value >= min)
                && definition.max.is_none_or(|max| value <= max);
            in_range.then_some(GameRuleValue::Int(value))
        }
    }
}
