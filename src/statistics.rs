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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatTypeDef {
    pub category: &'static str,
    pub registry_key: &'static str,
    pub display_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatFormatterKind {
    Default,
    DivideByTen,
    Distance,
    Time,
}

pub const STAT_TYPE_DEFS_26_1_2: &[StatTypeDef] = &[
    stat_type_def("minecraft:mined", "minecraft:block", "stat_type.minecraft.mined"),
    stat_type_def(
        "minecraft:crafted",
        "minecraft:item",
        "stat_type.minecraft.crafted",
    ),
    stat_type_def("minecraft:used", "minecraft:item", "stat_type.minecraft.used"),
    stat_type_def("minecraft:broken", "minecraft:item", "stat_type.minecraft.broken"),
    stat_type_def(
        "minecraft:picked_up",
        "minecraft:item",
        "stat_type.minecraft.picked_up",
    ),
    stat_type_def(
        "minecraft:dropped",
        "minecraft:item",
        "stat_type.minecraft.dropped",
    ),
    stat_type_def(
        "minecraft:killed",
        "minecraft:entity_type",
        "stat_type.minecraft.killed",
    ),
    stat_type_def(
        "minecraft:killed_by",
        "minecraft:entity_type",
        "stat_type.minecraft.killed_by",
    ),
    stat_type_def(
        "minecraft:custom",
        "minecraft:custom_stat",
        "stat_type.minecraft.custom",
    ),
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatKey {
    pub category: Identifier,
    pub value: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatModel {
    key: StatKey,
    formatter: StatFormatterKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatTypeModel {
    category: Identifier,
    registry_key: Identifier,
    display_name: String,
    stats: BTreeMap<Identifier, StatModel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StatsCounterModel {
    values: BTreeMap<StatKey, i32>,
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

impl StatFormatterKind {
    pub fn format(self, value: i32) -> String {
        match self {
            Self::Default => format_integer_us(value),
            Self::DivideByTen => decimal_format(f64::from(value) * 0.1),
            Self::Distance => format_distance(value),
            Self::Time => format_time(value),
        }
    }
}

pub fn custom_stat_formatter_26_1_2(id: &str) -> Option<StatFormatterKind> {
    let id = id.strip_prefix("minecraft:").unwrap_or(id);
    match id {
        "play_time" | "total_world_time" | "time_since_death" | "time_since_rest"
        | "sneak_time" => Some(StatFormatterKind::Time),
        "walk_one_cm" | "crouch_one_cm" | "sprint_one_cm" | "walk_on_water_one_cm"
        | "fall_one_cm" | "climb_one_cm" | "fly_one_cm" | "walk_under_water_one_cm"
        | "minecart_one_cm" | "boat_one_cm" | "pig_one_cm" | "happy_ghast_one_cm"
        | "horse_one_cm" | "aviate_one_cm" | "swim_one_cm" | "strider_one_cm"
        | "nautilus_one_cm" => Some(StatFormatterKind::Distance),
        "damage_dealt" | "damage_dealt_absorbed" | "damage_dealt_resisted" | "damage_taken"
        | "damage_blocked_by_shield" | "damage_absorbed" | "damage_resisted" => {
            Some(StatFormatterKind::DivideByTen)
        }
        _ if CUSTOM_STATS_26_1_2
            .iter()
            .any(|known| *known == format!("minecraft:{id}")) =>
        {
            Some(StatFormatterKind::Default)
        }
        _ => None,
    }
}

impl StatModel {
    pub fn new(key: StatKey, formatter: StatFormatterKind) -> Self {
        Self { key, formatter }
    }

    pub fn stat_type(&self) -> &Identifier {
        &self.key.category
    }

    pub fn value(&self) -> &Identifier {
        &self.key.value
    }

    pub fn name(&self) -> String {
        self.key.objective_name()
    }

    pub fn format(&self, value: i32) -> String {
        self.formatter.format(value)
    }

    pub fn description(&self) -> String {
        format!(
            "Stat{{name={}, formatter={:?}}}",
            self.name(),
            self.formatter
        )
    }
}

const fn stat_type_def(
    category: &'static str,
    registry_key: &'static str,
    display_name: &'static str,
) -> StatTypeDef {
    StatTypeDef {
        category,
        registry_key,
        display_name,
    }
}

impl StatTypeModel {
    pub fn new(category: &str, registry_key: &str, display_name: &str) -> Result<Self, String> {
        let category = normalize_identifier(category)?;
        let registry_key = normalize_identifier(registry_key)?;
        Ok(Self {
            category,
            registry_key,
            display_name: display_name.to_string(),
            stats: BTreeMap::new(),
        })
    }

    pub fn registry_key(&self) -> &Identifier {
        &self.registry_key
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn contains(&self, value: &Identifier) -> bool {
        self.stats.contains_key(value)
    }

    pub fn get(
        &mut self,
        value: &str,
        formatter: StatFormatterKind,
    ) -> Result<&StatModel, String> {
        let value = normalize_identifier(value)?;
        if !self.stats.contains_key(&value) {
            let key = StatKey {
                category: self.category.clone(),
                value: value.clone(),
            };
            self.stats
                .insert(value.clone(), StatModel::new(key, formatter));
        }
        self.stats
            .get(&value)
            .ok_or_else(|| format!("missing cached stat {value}"))
    }

    pub fn get_default(&mut self, value: &str) -> Result<&StatModel, String> {
        self.get(value, StatFormatterKind::Default)
    }

    pub fn iter(&self) -> impl Iterator<Item = &StatModel> {
        self.stats.values()
    }
}

impl StatsCounterModel {
    pub fn increment(&mut self, stat: &StatModel, count: i32) {
        let result = (i64::from(self.get_value(stat)) + i64::from(count)).min(i64::from(i32::MAX));
        self.set_value(stat, result as i32);
    }

    pub fn set_value(&mut self, stat: &StatModel, count: i32) {
        self.values.insert(stat.key.clone(), count);
    }

    pub fn get_typed_value(&self, stat_type: &StatTypeModel, key: &Identifier) -> i32 {
        if !stat_type.contains(key) {
            return 0;
        }
        let stat_key = StatKey {
            category: stat_type.category.clone(),
            value: key.clone(),
        };
        self.values.get(&stat_key).copied().unwrap_or(0)
    }

    pub fn get_value(&self, stat: &StatModel) -> i32 {
        self.values.get(&stat.key).copied().unwrap_or(0)
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

fn format_distance(cm: i32) -> String {
    let meters = f64::from(cm) / 100.0;
    let kilometers = meters / 1000.0;
    if kilometers > 0.5 {
        format!("{} km", decimal_format(kilometers))
    } else if meters > 0.5 {
        format!("{} m", decimal_format(meters))
    } else {
        format!("{cm} cm")
    }
}

fn format_time(value: i32) -> String {
    let seconds = f64::from(value) / 20.0;
    let minutes = seconds / 60.0;
    let hours = minutes / 60.0;
    let days = hours / 24.0;
    let years = days / 365.0;
    if years > 0.5 {
        format!("{} y", decimal_format(years))
    } else if days > 0.5 {
        format!("{} d", decimal_format(days))
    } else if hours > 0.5 {
        format!("{} h", decimal_format(hours))
    } else if minutes > 0.5 {
        format!("{} min", decimal_format(minutes))
    } else {
        format!("{} s", java_double_text(seconds))
    }
}

fn decimal_format(value: f64) -> String {
    format!("{value:.2}")
}

fn java_double_text(value: f64) -> String {
    let formatted = value.to_string();
    if formatted.contains(['.', 'E', 'e']) {
        formatted
    } else {
        format!("{formatted}.0")
    }
}

fn format_integer_us(value: i32) -> String {
    let magnitude = if value < 0 {
        -(i64::from(value))
    } else {
        i64::from(value)
    };
    let mut digits = magnitude.to_string();
    let mut grouped = String::new();
    while digits.len() > 3 {
        let tail = digits.split_off(digits.len() - 3);
        if grouped.is_empty() {
            grouped = tail;
        } else {
            grouped = format!("{tail},{grouped}");
        }
    }
    if grouped.is_empty() {
        grouped = digits;
    } else {
        grouped = format!("{digits},{grouped}");
    }
    if value < 0 {
        format!("-{grouped}")
    } else {
        grouped
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

    const STAT_FORMATTER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/stats/StatFormatter.java");
    const STAT_JAVA: &str = vibecraft_java_source!("/net/minecraft/stats/Stat.java");
    const STAT_TYPE_JAVA: &str = vibecraft_java_source!("/net/minecraft/stats/StatType.java");
    const STATS_JAVA: &str = vibecraft_java_source!("/net/minecraft/stats/Stats.java");
    const STATS_COUNTER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/stats/StatsCounter.java");

    #[test]
    fn stat_formatters_match_java_thresholds_and_units() {
        for sentinel in [
            "new DecimalFormat(\"########0.00\", DecimalFormatSymbols.getInstance(Locale.ROOT))",
            "NumberFormat.getIntegerInstance(Locale.US)::format",
            "value -> DECIMAL_FORMAT.format(value * 0.1)",
            "double meters = cm / 100.0;",
            "double kilometers = meters / 1000.0;",
            "if (kilometers > 0.5)",
            "return meters > 0.5 ? DECIMAL_FORMAT.format(meters) + \" m\" : cm + \" cm\";",
            "double seconds = value / 20.0;",
            "double years = days / 365.0;",
            "return minutes > 0.5 ? DECIMAL_FORMAT.format(minutes) + \" min\" : seconds + \" s\";",
        ] {
            assert!(
                STAT_FORMATTER_JAVA.contains(sentinel),
                "StatFormatter.java is missing sentinel: {sentinel}"
            );
        }

        assert_eq!(StatFormatterKind::Default.format(1_234_567), "1,234,567");
        assert_eq!(StatFormatterKind::Default.format(i32::MIN), "-2,147,483,648");
        assert_eq!(StatFormatterKind::DivideByTen.format(123), "12.30");

        assert_eq!(StatFormatterKind::Distance.format(50), "50 cm");
        assert_eq!(StatFormatterKind::Distance.format(51), "0.51 m");
        assert_eq!(StatFormatterKind::Distance.format(50_000), "500.00 m");
        assert_eq!(StatFormatterKind::Distance.format(50_001), "0.50 km");

        assert_eq!(StatFormatterKind::Time.format(10), "0.5 s");
        assert_eq!(StatFormatterKind::Time.format(20), "1.0 s");
        assert_eq!(StatFormatterKind::Time.format(601), "0.50 min");
        assert_eq!(StatFormatterKind::Time.format(36_001), "0.50 h");
        assert_eq!(StatFormatterKind::Time.format(864_001), "0.50 d");
        assert_eq!(StatFormatterKind::Time.format(315_360_001), "0.50 y");
    }

    #[test]
    fn stat_model_matches_java_name_format_and_identity_rules() {
        for sentinel in [
            "public static final StreamCodec<RegistryFriendlyByteBuf, Stat<?>> STREAM_CODEC = ByteBufCodecs.registry(Registries.STAT_TYPE)",
            ".dispatch(Stat::getType, StatType::streamCodec)",
            "super(buildName(type, value));",
            "return locationToKey(BuiltInRegistries.STAT_TYPE.getKey(type)) + \":\" + locationToKey(type.getRegistry().getKey(value));",
            "return location.toString().replace(':', '.');",
            "return this.type;",
            "return this.value;",
            "return this.formatter.format(value);",
            "Objects.equals(this.getName(), ((Stat)o).getName())",
            "return this.getName().hashCode();",
            "\"Stat{name=\" + this.getName() + \", formatter=\" + this.formatter + \"}\"",
        ] {
            assert!(
                STAT_JAVA.contains(sentinel),
                "Stat.java is missing sentinel: {sentinel}"
            );
        }

        let key = StatKey::new("minecraft:custom", "minecraft:walk_one_cm").unwrap();
        let stat = StatModel::new(key.clone(), StatFormatterKind::Distance);
        assert_eq!(stat.stat_type().to_string(), "minecraft:custom");
        assert_eq!(stat.value().to_string(), "minecraft:walk_one_cm");
        assert_eq!(stat.name(), "minecraft.custom:minecraft.walk_one_cm");
        assert_eq!(stat.format(123), "1.23 m");
        assert_eq!(
            stat.description(),
            "Stat{name=minecraft.custom:minecraft.walk_one_cm, formatter=Distance}"
        );
        assert_eq!(stat, StatModel::new(key, StatFormatterKind::Distance));
    }

    #[test]
    fn stat_type_model_matches_java_cache_and_accessors() {
        for sentinel in [
            "private final Registry<T> registry;",
            "private final Map<T, Stat<T>> map = new IdentityHashMap<>();",
            "private final Component displayName;",
            "this.streamCodec = ByteBufCodecs.registry(registry.key()).map(this::get, Stat::getValue);",
            "public StreamCodec<RegistryFriendlyByteBuf, Stat<T>> streamCodec()",
            "return this.map.containsKey(key);",
            "return this.map.computeIfAbsent(argument, t -> new Stat<>(this, (T)t, formatter));",
            "return this.registry;",
            "return this.map.values().iterator();",
            "return this.get(argument, StatFormatter.DEFAULT);",
            "return this.displayName;",
        ] {
            assert!(
                STAT_TYPE_JAVA.contains(sentinel),
                "StatType.java is missing sentinel: {sentinel}"
            );
        }

        let mut stat_type = StatTypeModel::new(
            "minecraft:custom",
            "minecraft:custom_stat",
            "stat_type.minecraft.custom",
        )
        .unwrap();
        let walk = Identifier::parse("minecraft:walk_one_cm").unwrap();
        assert_eq!(stat_type.registry_key().to_string(), "minecraft:custom_stat");
        assert_eq!(stat_type.display_name(), "stat_type.minecraft.custom");
        assert!(!stat_type.contains(&walk));

        let first_name = stat_type
            .get("minecraft:walk_one_cm", StatFormatterKind::Distance)
            .unwrap()
            .name();
        assert_eq!(first_name, "minecraft.custom:minecraft.walk_one_cm");
        assert!(stat_type.contains(&walk));
        let cached_name = stat_type
            .get("minecraft:walk_one_cm", StatFormatterKind::Default)
            .unwrap()
            .name();
        assert_eq!(cached_name, first_name);
        assert_eq!(stat_type.iter().count(), 1);

        let jump = stat_type.get_default("jump").unwrap();
        assert_eq!(jump.name(), "minecraft.custom:minecraft.jump");
        assert_eq!(jump.format(1_234), "1,234");
        assert_eq!(stat_type.iter().count(), 2);
    }

    #[test]
    fn stats_registry_and_custom_stat_formatters_match_java() {
        for sentinel in [
            "public static final StatType<Block> BLOCK_MINED = makeRegistryStatType(\"mined\", BuiltInRegistries.BLOCK);",
            "public static final StatType<Identifier> CUSTOM = makeRegistryStatType(\"custom\", BuiltInRegistries.CUSTOM_STAT);",
            "public static final Identifier PLAY_TIME = makeCustomStat(\"play_time\", StatFormatter.TIME);",
            "public static final Identifier WALK_ONE_CM = makeCustomStat(\"walk_one_cm\", StatFormatter.DISTANCE);",
            "public static final Identifier DAMAGE_DEALT = makeCustomStat(\"damage_dealt\", StatFormatter.DIVIDE_BY_TEN);",
            "public static final Identifier INTERACT_WITH_SMITHING_TABLE = makeCustomStat(\"interact_with_smithing_table\", StatFormatter.DEFAULT);",
            "Identifier location = Identifier.withDefaultNamespace(id);",
            "Registry.register(BuiltInRegistries.CUSTOM_STAT, id, location);",
            "CUSTOM.get(location, formatter);",
            "Component displayName = Component.translatable(\"stat_type.minecraft.\" + name);",
            "Registry.register(BuiltInRegistries.STAT_TYPE, name, new StatType<>(registry, displayName));",
        ] {
            assert!(
                STATS_JAVA.contains(sentinel),
                "Stats.java is missing sentinel: {sentinel}"
            );
        }

        assert_eq!(
            STAT_TYPE_DEFS_26_1_2
                .iter()
                .map(|definition| definition.category)
                .collect::<Vec<_>>(),
            STAT_TYPE_CATEGORIES_26_1_2
        );
        assert_eq!(
            STAT_TYPE_DEFS_26_1_2
                .iter()
                .map(|definition| (definition.registry_key, definition.display_name))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:block", "stat_type.minecraft.mined"),
                ("minecraft:item", "stat_type.minecraft.crafted"),
                ("minecraft:item", "stat_type.minecraft.used"),
                ("minecraft:item", "stat_type.minecraft.broken"),
                ("minecraft:item", "stat_type.minecraft.picked_up"),
                ("minecraft:item", "stat_type.minecraft.dropped"),
                ("minecraft:entity_type", "stat_type.minecraft.killed"),
                ("minecraft:entity_type", "stat_type.minecraft.killed_by"),
                ("minecraft:custom_stat", "stat_type.minecraft.custom"),
            ]
        );

        let java_custom_stats = java_custom_stat_formatters();
        assert_eq!(java_custom_stats.len(), CUSTOM_STATS_26_1_2.len());
        assert_eq!(
            java_custom_stats
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>(),
            CUSTOM_STATS_26_1_2
        );
        for (id, formatter) in java_custom_stats {
            assert_eq!(
                custom_stat_formatter_26_1_2(&id),
                Some(formatter),
                "formatter mismatch for {id}"
            );
        }
    }

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
    fn stats_counter_base_semantics_match_java() {
        for sentinel in [
            "protected final Object2IntMap<Stat<?>> stats = Object2IntMaps.synchronize(new Object2IntOpenHashMap());",
            "this.stats.defaultReturnValue(0);",
            "int result = (int)Math.min((long)this.getValue(stat) + count, 2147483647L);",
            "this.setValue(player, stat, result);",
            "this.stats.put(stat, count);",
            "return type.contains(key) ? this.getValue(type.get(key)) : 0;",
            "return this.stats.getInt(stat);",
        ] {
            assert!(
                STATS_COUNTER_JAVA.contains(sentinel),
                "StatsCounter.java is missing sentinel: {sentinel}"
            );
        }

        let mut stat_type = StatTypeModel::new(
            "minecraft:custom",
            "minecraft:custom_stat",
            "stat_type.minecraft.custom",
        )
        .unwrap();
        let jump = stat_type
            .get_default("minecraft:jump")
            .unwrap()
            .clone();
        let jump_key = Identifier::parse("minecraft:jump").unwrap();
        let missing_key = Identifier::parse("minecraft:play_time").unwrap();
        let mut counter = StatsCounterModel::default();

        assert_eq!(counter.get_value(&jump), 0);
        assert_eq!(counter.get_typed_value(&stat_type, &jump_key), 0);
        assert_eq!(counter.get_typed_value(&stat_type, &missing_key), 0);

        counter.set_value(&jump, 12);
        assert_eq!(counter.get_value(&jump), 12);
        assert_eq!(counter.get_typed_value(&stat_type, &jump_key), 12);

        counter.increment(&jump, 5);
        assert_eq!(counter.get_value(&jump), 17);
        counter.increment(&jump, i32::MAX);
        assert_eq!(counter.get_value(&jump), i32::MAX);
        counter.increment(&jump, -10);
        assert_eq!(counter.get_value(&jump), i32::MAX - 10);
    }

    fn java_custom_stat_formatters() -> Vec<(String, StatFormatterKind)> {
        STATS_JAVA
            .lines()
            .filter_map(|line| {
                let line = line.trim_start();
                if !line.starts_with("public static final Identifier ") {
                    return None;
                }
                let id_start = line.find("makeCustomStat(\"")? + "makeCustomStat(\"".len();
                let id_remainder = &line[id_start..];
                let id_end = id_remainder.find('"')?;
                let id = format!("minecraft:{}", &id_remainder[..id_end]);
                let formatter_start = line.find("StatFormatter.")? + "StatFormatter.".len();
                let formatter_remainder = &line[formatter_start..];
                let formatter_end = formatter_remainder.find(')')?;
                let formatter = match &formatter_remainder[..formatter_end] {
                    "DEFAULT" => StatFormatterKind::Default,
                    "DIVIDE_BY_TEN" => StatFormatterKind::DivideByTen,
                    "DISTANCE" => StatFormatterKind::Distance,
                    "TIME" => StatFormatterKind::Time,
                    other => panic!("unknown Java stat formatter: {other}"),
                };
                Some((id, formatter))
            })
            .collect()
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
