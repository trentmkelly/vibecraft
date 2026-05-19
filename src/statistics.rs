use std::collections::{BTreeMap, BTreeSet};

use crate::network::play::{AwardedStat, ClientboundAwardStatsPacket};
use crate::registry::Identifier;

pub const STAT_TYPE_CATEGORIES_26_1_2: &[&str] = &[
    "minecraft:mined",
    "minecraft:crafted",
    "minecraft:used",
    "minecraft:broken",
    "minecraft:picked_up",
    "minecraft:dropped",
    "minecraft:killed",
    "minecraft:killed_by",
    "minecraft:custom",
];

pub const CUSTOM_STATS_26_1_2: &[&str] = &[
    "minecraft:leave_game",
    "minecraft:play_time",
    "minecraft:total_world_time",
    "minecraft:time_since_death",
    "minecraft:time_since_rest",
    "minecraft:sneak_time",
    "minecraft:walk_one_cm",
    "minecraft:crouch_one_cm",
    "minecraft:sprint_one_cm",
    "minecraft:walk_on_water_one_cm",
    "minecraft:fall_one_cm",
    "minecraft:climb_one_cm",
    "minecraft:fly_one_cm",
    "minecraft:walk_under_water_one_cm",
    "minecraft:minecart_one_cm",
    "minecraft:boat_one_cm",
    "minecraft:pig_one_cm",
    "minecraft:happy_ghast_one_cm",
    "minecraft:horse_one_cm",
    "minecraft:aviate_one_cm",
    "minecraft:swim_one_cm",
    "minecraft:strider_one_cm",
    "minecraft:nautilus_one_cm",
    "minecraft:jump",
    "minecraft:drop",
    "minecraft:damage_dealt",
    "minecraft:damage_dealt_absorbed",
    "minecraft:damage_dealt_resisted",
    "minecraft:damage_taken",
    "minecraft:damage_blocked_by_shield",
    "minecraft:damage_absorbed",
    "minecraft:damage_resisted",
    "minecraft:deaths",
    "minecraft:mob_kills",
    "minecraft:animals_bred",
    "minecraft:player_kills",
    "minecraft:fish_caught",
    "minecraft:talked_to_villager",
    "minecraft:traded_with_villager",
    "minecraft:eat_cake_slice",
    "minecraft:fill_cauldron",
    "minecraft:use_cauldron",
    "minecraft:clean_armor",
    "minecraft:clean_banner",
    "minecraft:clean_shulker_box",
    "minecraft:interact_with_brewingstand",
    "minecraft:interact_with_beacon",
    "minecraft:inspect_dropper",
    "minecraft:inspect_hopper",
    "minecraft:inspect_dispenser",
    "minecraft:play_noteblock",
    "minecraft:tune_noteblock",
    "minecraft:pot_flower",
    "minecraft:trigger_trapped_chest",
    "minecraft:open_enderchest",
    "minecraft:enchant_item",
    "minecraft:play_record",
    "minecraft:interact_with_furnace",
    "minecraft:interact_with_crafting_table",
    "minecraft:open_chest",
    "minecraft:sleep_in_bed",
    "minecraft:open_shulker_box",
    "minecraft:open_barrel",
    "minecraft:interact_with_blast_furnace",
    "minecraft:interact_with_smoker",
    "minecraft:interact_with_lectern",
    "minecraft:interact_with_campfire",
    "minecraft:interact_with_cartography_table",
    "minecraft:interact_with_loom",
    "minecraft:interact_with_stonecutter",
    "minecraft:bell_ring",
    "minecraft:raid_trigger",
    "minecraft:raid_win",
    "minecraft:interact_with_anvil",
    "minecraft:interact_with_grindstone",
    "minecraft:target_hit",
    "minecraft:interact_with_smithing_table",
];

pub const CUSTOM_STAT_TYPE_NETWORK_ID_26_1_2: i32 = 8;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatKey {
    pub category: Identifier,
    pub value: Identifier,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatisticsCounter {
    values: BTreeMap<StatKey, i32>,
    dirty: BTreeSet<StatKey>,
}

impl StatKey {
    pub fn new(category: &str, value: &str) -> Result<Self, String> {
        let category = normalize_identifier(category)?;
        let value = normalize_identifier(value)?;
        if !STAT_TYPE_CATEGORIES_26_1_2
            .iter()
            .any(|known| *known == category.to_string())
        {
            return Err(format!("unknown stat category {category}"));
        }
        Ok(Self { category, value })
    }

    pub fn custom(value: &str) -> Result<Self, String> {
        Self::new("minecraft:custom", value)
    }

    pub fn objective_name(&self) -> String {
        format!(
            "{}:{}",
            self.category.to_string().replace(':', "."),
            self.value.to_string().replace(':', ".")
        )
    }
}

impl StatisticsCounter {
    pub fn increment(&mut self, stat: StatKey, count: i32) {
        let current = self.get(&stat);
        let result =
            i32::try_from((i64::from(current) + i64::from(count)).min(i64::from(i32::MAX)))
                .unwrap_or(i32::MIN);
        self.set(stat, result);
    }

    pub fn set(&mut self, stat: StatKey, value: i32) {
        self.values.insert(stat.clone(), value);
        self.dirty.insert(stat);
    }

    pub fn get(&self, stat: &StatKey) -> i32 {
        self.values.get(stat).copied().unwrap_or(0)
    }

    pub fn mark_all_dirty(&mut self) {
        self.dirty.extend(self.values.keys().cloned());
    }

    pub fn drain_dirty_packet(&mut self) -> ClientboundAwardStatsPacket {
        let stats = self
            .dirty
            .iter()
            .filter_map(|stat| {
                self.values
                    .get(stat)
                    .and_then(|value| stat_network_value(stat, *value))
            })
            .collect();
        self.dirty.clear();
        ClientboundAwardStatsPacket { stats }
    }

    pub fn to_vanilla_json(&self, data_version: i32) -> String {
        let mut grouped: BTreeMap<String, Vec<(String, i32)>> = BTreeMap::new();
        for (stat, value) in &self.values {
            grouped
                .entry(stat.category.to_string())
                .or_default()
                .push((stat.value.to_string(), *value));
        }

        let mut out = String::from("{\"stats\":{");
        for (category_index, (category, values)) in grouped.iter().enumerate() {
            if category_index > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(category);
            out.push_str("\":{");
            for (value_index, (value, count)) in values.iter().enumerate() {
                if value_index > 0 {
                    out.push(',');
                }
                out.push('"');
                out.push_str(value);
                out.push_str("\":");
                out.push_str(&count.to_string());
            }
            out.push('}');
        }
        out.push_str("},\"DataVersion\":");
        out.push_str(&data_version.to_string());
        out.push('}');
        out
    }

    pub fn from_vanilla_json(json: &str) -> Result<Self, String> {
        let stats_start = json
            .find("\"stats\"")
            .ok_or_else(|| "missing stats root".to_string())?;
        let object_start = json[stats_start..]
            .find('{')
            .map(|index| stats_start + index)
            .ok_or_else(|| "missing stats object".to_string())?;
        let object_end = matching_brace(json, object_start)?;
        parse_stats_object(&json[object_start + 1..object_end])
    }
}

fn normalize_identifier(value: &str) -> Result<Identifier, String> {
    let value = if value.contains(':') {
        value.to_string()
    } else {
        format!("minecraft:{value}")
    };
    Identifier::parse(&value)
}

fn stat_network_value(stat: &StatKey, value: i32) -> Option<AwardedStat> {
    if stat.category.to_string() != "minecraft:custom" {
        return None;
    }
    let stat_value_id = CUSTOM_STATS_26_1_2
        .iter()
        .position(|known| *known == stat.value.to_string())? as i32;
    Some(AwardedStat {
        stat_type_id: CUSTOM_STAT_TYPE_NETWORK_ID_26_1_2,
        stat_value_id,
        value,
    })
}

fn matching_brace(input: &str, open: usize) -> Result<usize, String> {
    let mut depth = 0_i32;
    for (index, byte) in input.bytes().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    Err("unterminated stats object".to_string())
}

fn parse_stats_object(input: &str) -> Result<StatisticsCounter, String> {
    let mut counter = StatisticsCounter::default();
    let mut rest = input.trim();
    while !rest.is_empty() {
        let (category, after_category) = parse_json_key(rest)?;
        let category_start = after_category
            .trim_start()
            .strip_prefix(':')
            .ok_or_else(|| "missing category separator".to_string())?
            .trim_start();
        let nested_start = input.len() - category_start.len();
        if !category_start.starts_with('{') {
            return Err("expected category object".to_string());
        }
        let nested_end = matching_brace(input, nested_start)?;
        let entries = &input[nested_start + 1..nested_end];
        for (value, count) in parse_stat_values(entries)? {
            counter
                .values
                .insert(StatKey::new(&category, &value)?, count);
        }
        rest = input[nested_end + 1..].trim_start();
        rest = rest.strip_prefix(',').unwrap_or(rest).trim_start();
    }
    Ok(counter)
}

fn parse_stat_values(input: &str) -> Result<Vec<(String, i32)>, String> {
    let mut values = Vec::new();
    let mut rest = input.trim();
    while !rest.is_empty() {
        let (key, after_key) = parse_json_key(rest)?;
        let after_colon = after_key
            .trim_start()
            .strip_prefix(':')
            .ok_or_else(|| "missing stat separator".to_string())?
            .trim_start();
        let end = after_colon.find([',', '}']).unwrap_or(after_colon.len());
        let count = after_colon[..end]
            .trim()
            .parse::<i32>()
            .map_err(|err| err.to_string())?;
        values.push((key, count));
        rest = after_colon[end..]
            .trim_start()
            .strip_prefix(',')
            .unwrap_or("")
            .trim_start();
    }
    Ok(values)
}

fn parse_json_key(input: &str) -> Result<(String, &str), String> {
    let input = input.trim_start();
    let input = input
        .strip_prefix('"')
        .ok_or_else(|| "expected quoted key".to_string())?;
    let end = input
        .find('"')
        .ok_or_else(|| "unterminated quoted key".to_string())?;
    Ok((input[..end].to_string(), &input[end + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_categories_and_custom_stats_match_decompiled_surface() {
        assert_eq!(
            STAT_TYPE_CATEGORIES_26_1_2,
            [
                "minecraft:mined",
                "minecraft:crafted",
                "minecraft:used",
                "minecraft:broken",
                "minecraft:picked_up",
                "minecraft:dropped",
                "minecraft:killed",
                "minecraft:killed_by",
                "minecraft:custom",
            ]
        );
        assert_eq!(CUSTOM_STATS_26_1_2.len(), 77);
        assert!(CUSTOM_STATS_26_1_2.contains(&"minecraft:play_time"));
        assert!(CUSTOM_STATS_26_1_2.contains(&"minecraft:happy_ghast_one_cm"));
        assert!(CUSTOM_STATS_26_1_2.contains(&"minecraft:interact_with_smithing_table"));
    }

    #[test]
    fn stat_counter_saturates_marks_dirty_and_builds_award_packet() {
        let mut counter = StatisticsCounter::default();
        let jumps = StatKey::custom("jump").unwrap();
        counter.increment(jumps.clone(), 2);
        counter.increment(jumps.clone(), i32::MAX);
        assert_eq!(counter.get(&jumps), i32::MAX);

        let packet = counter.drain_dirty_packet();
        assert_eq!(packet.stats.len(), 1);
        assert_eq!(
            packet.stats[0],
            AwardedStat {
                stat_type_id: CUSTOM_STAT_TYPE_NETWORK_ID_26_1_2,
                stat_value_id: 23,
                value: i32::MAX,
            }
        );
        assert!(counter.drain_dirty_packet().stats.is_empty());
        counter.mark_all_dirty();
        assert_eq!(counter.drain_dirty_packet().stats.len(), 1);
    }

    #[test]
    fn stat_persistence_uses_vanilla_grouped_json_shape() {
        let mut counter = StatisticsCounter::default();
        counter.set(StatKey::custom("play_time").unwrap(), 20);
        counter.set(
            StatKey::new("minecraft:mined", "minecraft:stone").unwrap(),
            3,
        );

        let json = counter.to_vanilla_json(4189);
        assert_eq!(
            json,
            "{\"stats\":{\"minecraft:custom\":{\"minecraft:play_time\":20},\"minecraft:mined\":{\"minecraft:stone\":3}},\"DataVersion\":4189}"
        );

        let loaded = StatisticsCounter::from_vanilla_json(&json).unwrap();
        assert_eq!(loaded.get(&StatKey::custom("play_time").unwrap()), 20);
        assert_eq!(
            loaded.get(&StatKey::new("minecraft:mined", "minecraft:stone").unwrap()),
            3
        );
        assert!(loaded.dirty.is_empty());
    }

    #[test]
    fn stat_keys_expose_vanilla_scoreboard_objective_names() {
        let mined = StatKey::new("mined", "stone").unwrap();
        assert_eq!(mined.objective_name(), "minecraft.mined:minecraft.stone");
        assert!(StatKey::new("bad", "stone").is_err());
    }
}
