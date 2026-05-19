#![allow(dead_code)]

use std::collections::BTreeMap;

pub const VANILLA_GAME_RULE_COUNT_26_1_2: usize = 59;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRuleCategory {
    Updates,
    Misc,
    Drops,
    Chat,
    Player,
    Mobs,
    Spawning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRuleType {
    Bool,
    Int,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRuleValue {
    Bool(bool),
    Int(i32),
}

impl GameRuleValue {
    pub fn command_result(self) -> i32 {
        match self {
            Self::Bool(value) => i32::from(value),
            Self::Int(value) => value,
        }
    }

    pub fn sync_value(self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRuleDefinition {
    pub name: &'static str,
    pub category: GameRuleCategory,
    pub rule_type: GameRuleType,
    pub default: GameRuleValue,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub requires_minecart_improvements: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleSync {
    pub rule: String,
    pub value: String,
    pub command_result: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRules {
    values: BTreeMap<&'static str, GameRuleValue>,
}

impl GameRules {
    pub fn new(minecart_improvements: bool) -> Self {
        Self {
            values: vanilla_game_rules()
                .iter()
                .filter(|rule| !rule.requires_minecart_improvements || minecart_improvements)
                .map(|rule| (rule.name, rule.default))
                .collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<GameRuleValue> {
        let normalized = normalize_game_rule_name(name);
        self.values.get(normalized.as_str()).copied()
    }

    pub fn set(&mut self, name: &str, raw_value: &str) -> Result<GameRuleSync, GameRuleError> {
        let normalized = normalize_game_rule_name(name);
        let definition = game_rule_definition(&normalized).ok_or(GameRuleError::UnknownRule)?;
        if !self.values.contains_key(definition.name) {
            return Err(GameRuleError::DisabledByFeature);
        }
        let value = parse_game_rule_value(raw_value, definition)?;
        self.values.insert(definition.name, value);
        Ok(GameRuleSync {
            rule: format!("minecraft:{}", definition.name),
            value: value.sync_value(),
            command_result: value.command_result(),
        })
    }

    pub fn runtime_effects(&self) -> GameRuleRuntimeEffects {
        GameRuleRuntimeEffects {
            advance_time: self.bool("advance_time"),
            advance_weather: self.bool("advance_weather"),
            allow_entering_nether_using_portals: self.bool("allow_entering_nether_using_portals"),
            block_drops: self.bool("block_drops"),
            block_explosion_drop_decay: self.bool("block_explosion_drop_decay"),
            command_blocks_work: self.bool("command_blocks_work"),
            command_block_output: self.bool("command_block_output"),
            drowning_damage: self.bool("drowning_damage"),
            elytra_movement_check: self.bool("elytra_movement_check"),
            ender_pearls_vanish_on_death: self.bool("ender_pearls_vanish_on_death"),
            entity_drops: self.bool("entity_drops"),
            fall_damage: self.bool("fall_damage"),
            fire_damage: self.bool("fire_damage"),
            fire_spread_radius_around_player: self.int("fire_spread_radius_around_player"),
            forgive_dead_players: self.bool("forgive_dead_players"),
            freeze_damage: self.bool("freeze_damage"),
            global_sound_events: self.bool("global_sound_events"),
            immediate_respawn: self.bool("immediate_respawn"),
            keep_inventory: self.bool("keep_inventory"),
            lava_source_conversion: self.bool("lava_source_conversion"),
            limited_crafting: self.bool("limited_crafting"),
            locator_bar: self.bool("locator_bar"),
            log_admin_commands: self.bool("log_admin_commands"),
            max_block_modifications: self.int("max_block_modifications"),
            max_command_forks: self.int("max_command_forks"),
            max_command_sequence_length: self.int("max_command_sequence_length"),
            max_entity_cramming: self.int("max_entity_cramming"),
            max_minecart_speed: self.int("max_minecart_speed"),
            max_snow_accumulation_height: self.int("max_snow_accumulation_height"),
            mob_drops: self.bool("mob_drops"),
            mob_explosion_drop_decay: self.bool("mob_explosion_drop_decay"),
            mob_griefing: self.bool("mob_griefing"),
            natural_health_regeneration: self.bool("natural_health_regeneration"),
            player_movement_check: self.bool("player_movement_check"),
            nether_portal_creative_delay: self.int("players_nether_portal_creative_delay"),
            nether_portal_default_delay: self.int("players_nether_portal_default_delay"),
            players_sleeping_percentage: self.int("players_sleeping_percentage"),
            projectiles_can_break_blocks: self.bool("projectiles_can_break_blocks"),
            pvp: self.bool("pvp"),
            raids: self.bool("raids"),
            random_tick_speed: self.int("random_tick_speed"),
            reduced_debug_info: self.bool("reduced_debug_info"),
            respawn_radius: self.int("respawn_radius"),
            send_command_feedback: self.bool("send_command_feedback"),
            show_advancement_messages: self.bool("show_advancement_messages"),
            show_death_messages: self.bool("show_death_messages"),
            spawner_blocks_work: self.bool("spawner_blocks_work"),
            spawn_mobs: self.bool("spawn_mobs"),
            spawn_monsters: self.bool("spawn_monsters"),
            spawn_patrols: self.bool("spawn_patrols"),
            spawn_phantoms: self.bool("spawn_phantoms"),
            spawn_wandering_traders: self.bool("spawn_wandering_traders"),
            spawn_wardens: self.bool("spawn_wardens"),
            spectators_generate_chunks: self.bool("spectators_generate_chunks"),
            spread_vines: self.bool("spread_vines"),
            tnt_explodes: self.bool("tnt_explodes"),
            tnt_explosion_drop_decay: self.bool("tnt_explosion_drop_decay"),
            universal_anger: self.bool("universal_anger"),
            water_source_conversion: self.bool("water_source_conversion"),
        }
    }

    fn bool(&self, name: &'static str) -> bool {
        matches!(self.values.get(name), Some(GameRuleValue::Bool(true)))
    }

    fn int(&self, name: &'static str) -> i32 {
        match self.values.get(name) {
            Some(GameRuleValue::Int(value)) => *value,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRuleRuntimeEffects {
    pub advance_time: bool,
    pub advance_weather: bool,
    pub allow_entering_nether_using_portals: bool,
    pub block_drops: bool,
    pub block_explosion_drop_decay: bool,
    pub command_blocks_work: bool,
    pub command_block_output: bool,
    pub drowning_damage: bool,
    pub elytra_movement_check: bool,
    pub ender_pearls_vanish_on_death: bool,
    pub entity_drops: bool,
    pub fall_damage: bool,
    pub fire_damage: bool,
    pub fire_spread_radius_around_player: i32,
    pub forgive_dead_players: bool,
    pub freeze_damage: bool,
    pub global_sound_events: bool,
    pub immediate_respawn: bool,
    pub keep_inventory: bool,
    pub lava_source_conversion: bool,
    pub limited_crafting: bool,
    pub locator_bar: bool,
    pub log_admin_commands: bool,
    pub max_block_modifications: i32,
    pub max_command_forks: i32,
    pub max_command_sequence_length: i32,
    pub max_entity_cramming: i32,
    pub max_minecart_speed: i32,
    pub max_snow_accumulation_height: i32,
    pub mob_drops: bool,
    pub mob_explosion_drop_decay: bool,
    pub mob_griefing: bool,
    pub natural_health_regeneration: bool,
    pub player_movement_check: bool,
    pub nether_portal_creative_delay: i32,
    pub nether_portal_default_delay: i32,
    pub players_sleeping_percentage: i32,
    pub projectiles_can_break_blocks: bool,
    pub pvp: bool,
    pub raids: bool,
    pub random_tick_speed: i32,
    pub reduced_debug_info: bool,
    pub respawn_radius: i32,
    pub send_command_feedback: bool,
    pub show_advancement_messages: bool,
    pub show_death_messages: bool,
    pub spawner_blocks_work: bool,
    pub spawn_mobs: bool,
    pub spawn_monsters: bool,
    pub spawn_patrols: bool,
    pub spawn_phantoms: bool,
    pub spawn_wandering_traders: bool,
    pub spawn_wardens: bool,
    pub spectators_generate_chunks: bool,
    pub spread_vines: bool,
    pub tnt_explodes: bool,
    pub tnt_explosion_drop_decay: bool,
    pub universal_anger: bool,
    pub water_source_conversion: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRuleError {
    UnknownRule,
    DisabledByFeature,
    WrongType,
    OutOfRange,
}

pub fn vanilla_game_rules() -> &'static [GameRuleDefinition] {
    VANILLA_GAME_RULES
}

pub fn game_rule_definition(name: &str) -> Option<&'static GameRuleDefinition> {
    let normalized = normalize_game_rule_name(name);
    VANILLA_GAME_RULES
        .iter()
        .find(|definition| definition.name == normalized)
}

pub fn normalize_game_rule_name(rule: &str) -> String {
    let rule = rule.strip_prefix("minecraft:").unwrap_or(rule);
    match rule {
        "commandBlockOutput" => "command_block_output",
        "doDaylightCycle" => "advance_time",
        "doEntityDrops" => "entity_drops",
        "doImmediateRespawn" => "immediate_respawn",
        "doInsomnia" => "spawn_phantoms",
        "doLimitedCrafting" => "limited_crafting",
        "doMobLoot" => "mob_drops",
        "doMobSpawning" => "spawn_mobs",
        "doPatrolSpawning" => "spawn_patrols",
        "doTileDrops" => "block_drops",
        "doTraderSpawning" => "spawn_wandering_traders",
        "doVinesSpread" => "spread_vines",
        "doWardenSpawning" => "spawn_wardens",
        "doWeatherCycle" => "advance_weather",
        "drowningDamage" => "drowning_damage",
        "fallDamage" => "fall_damage",
        "fireDamage" => "fire_damage",
        "forgiveDeadPlayers" => "forgive_dead_players",
        "freezeDamage" => "freeze_damage",
        "globalSoundEvents" => "global_sound_events",
        "keepInventory" => "keep_inventory",
        "logAdminCommands" => "log_admin_commands",
        "maxCommandChainLength" => "max_command_sequence_length",
        "maxCommandForkCount" => "max_command_forks",
        "maxEntityCramming" => "max_entity_cramming",
        "mobGriefing" => "mob_griefing",
        "naturalRegeneration" => "natural_health_regeneration",
        "playersSleepingPercentage" => "players_sleeping_percentage",
        "randomTickSpeed" => "random_tick_speed",
        "reducedDebugInfo" => "reduced_debug_info",
        "sendCommandFeedback" => "send_command_feedback",
        "showDeathMessages" => "show_death_messages",
        "spawnRadius" => "respawn_radius",
        "spectatorsGenerateChunks" => "spectators_generate_chunks",
        "tntExplodes" => "tnt_explodes",
        "universalAnger" => "universal_anger",
        _ => rule,
    }
    .to_string()
}

pub fn parse_game_rule_value(
    raw: &str,
    definition: &GameRuleDefinition,
) -> Result<GameRuleValue, GameRuleError> {
    match definition.rule_type {
        GameRuleType::Bool => match raw {
            "true" => Ok(GameRuleValue::Bool(true)),
            "false" => Ok(GameRuleValue::Bool(false)),
            _ => Err(GameRuleError::WrongType),
        },
        GameRuleType::Int => {
            let value = raw.parse::<i32>().map_err(|_| GameRuleError::WrongType)?;
            if definition.min.is_some_and(|min| value < min)
                || definition.max.is_some_and(|max| value > max)
            {
                Err(GameRuleError::OutOfRange)
            } else {
                Ok(GameRuleValue::Int(value))
            }
        }
    }
}

const VANILLA_GAME_RULES: &[GameRuleDefinition] = &[
    bool_rule("advance_time", GameRuleCategory::Updates, true),
    bool_rule("advance_weather", GameRuleCategory::Updates, true),
    bool_rule(
        "allow_entering_nether_using_portals",
        GameRuleCategory::Misc,
        true,
    ),
    bool_rule("block_drops", GameRuleCategory::Drops, true),
    bool_rule("block_explosion_drop_decay", GameRuleCategory::Drops, true),
    bool_rule("command_blocks_work", GameRuleCategory::Misc, true),
    bool_rule("command_block_output", GameRuleCategory::Chat, true),
    bool_rule("drowning_damage", GameRuleCategory::Player, true),
    bool_rule("elytra_movement_check", GameRuleCategory::Player, true),
    bool_rule(
        "ender_pearls_vanish_on_death",
        GameRuleCategory::Player,
        true,
    ),
    bool_rule("entity_drops", GameRuleCategory::Drops, true),
    bool_rule("fall_damage", GameRuleCategory::Player, true),
    bool_rule("fire_damage", GameRuleCategory::Player, true),
    int_rule_min(
        "fire_spread_radius_around_player",
        GameRuleCategory::Updates,
        128,
        -1,
    ),
    bool_rule("forgive_dead_players", GameRuleCategory::Mobs, true),
    bool_rule("freeze_damage", GameRuleCategory::Player, true),
    bool_rule("global_sound_events", GameRuleCategory::Misc, true),
    bool_rule("immediate_respawn", GameRuleCategory::Player, false),
    bool_rule("keep_inventory", GameRuleCategory::Player, false),
    bool_rule("lava_source_conversion", GameRuleCategory::Updates, false),
    bool_rule("limited_crafting", GameRuleCategory::Player, false),
    bool_rule("locator_bar", GameRuleCategory::Player, true),
    bool_rule("log_admin_commands", GameRuleCategory::Chat, true),
    int_rule_min("max_block_modifications", GameRuleCategory::Misc, 32768, 1),
    int_rule_min("max_command_forks", GameRuleCategory::Misc, 65536, 0),
    int_rule_min(
        "max_command_sequence_length",
        GameRuleCategory::Misc,
        65536,
        0,
    ),
    int_rule_min("max_entity_cramming", GameRuleCategory::Mobs, 24, 0),
    int_rule_range_feature(
        "max_minecart_speed",
        GameRuleCategory::Misc,
        8,
        1,
        1000,
        true,
    ),
    int_rule_range(
        "max_snow_accumulation_height",
        GameRuleCategory::Updates,
        1,
        0,
        8,
    ),
    bool_rule("mob_drops", GameRuleCategory::Drops, true),
    bool_rule("mob_explosion_drop_decay", GameRuleCategory::Drops, true),
    bool_rule("mob_griefing", GameRuleCategory::Mobs, true),
    bool_rule(
        "natural_health_regeneration",
        GameRuleCategory::Player,
        true,
    ),
    bool_rule("player_movement_check", GameRuleCategory::Player, true),
    int_rule_min(
        "players_nether_portal_creative_delay",
        GameRuleCategory::Player,
        0,
        0,
    ),
    int_rule_min(
        "players_nether_portal_default_delay",
        GameRuleCategory::Player,
        80,
        0,
    ),
    int_rule_min(
        "players_sleeping_percentage",
        GameRuleCategory::Player,
        100,
        0,
    ),
    bool_rule(
        "projectiles_can_break_blocks",
        GameRuleCategory::Drops,
        true,
    ),
    bool_rule("pvp", GameRuleCategory::Player, true),
    bool_rule("raids", GameRuleCategory::Mobs, true),
    int_rule_min("random_tick_speed", GameRuleCategory::Updates, 3, 0),
    bool_rule("reduced_debug_info", GameRuleCategory::Misc, false),
    int_rule_min("respawn_radius", GameRuleCategory::Player, 10, 0),
    bool_rule("send_command_feedback", GameRuleCategory::Chat, true),
    bool_rule("show_advancement_messages", GameRuleCategory::Chat, true),
    bool_rule("show_death_messages", GameRuleCategory::Chat, true),
    bool_rule("spawner_blocks_work", GameRuleCategory::Misc, true),
    bool_rule("spawn_mobs", GameRuleCategory::Spawning, true),
    bool_rule("spawn_monsters", GameRuleCategory::Spawning, true),
    bool_rule("spawn_patrols", GameRuleCategory::Spawning, true),
    bool_rule("spawn_phantoms", GameRuleCategory::Spawning, true),
    bool_rule("spawn_wandering_traders", GameRuleCategory::Spawning, true),
    bool_rule("spawn_wardens", GameRuleCategory::Spawning, true),
    bool_rule("spectators_generate_chunks", GameRuleCategory::Player, true),
    bool_rule("spread_vines", GameRuleCategory::Updates, true),
    bool_rule("tnt_explodes", GameRuleCategory::Misc, true),
    bool_rule("tnt_explosion_drop_decay", GameRuleCategory::Drops, false),
    bool_rule("universal_anger", GameRuleCategory::Mobs, false),
    bool_rule("water_source_conversion", GameRuleCategory::Updates, true),
];

const fn bool_rule(
    name: &'static str,
    category: GameRuleCategory,
    default: bool,
) -> GameRuleDefinition {
    GameRuleDefinition {
        name,
        category,
        rule_type: GameRuleType::Bool,
        default: GameRuleValue::Bool(default),
        min: None,
        max: None,
        requires_minecart_improvements: false,
    }
}

const fn int_rule_min(
    name: &'static str,
    category: GameRuleCategory,
    default: i32,
    min: i32,
) -> GameRuleDefinition {
    GameRuleDefinition {
        name,
        category,
        rule_type: GameRuleType::Int,
        default: GameRuleValue::Int(default),
        min: Some(min),
        max: None,
        requires_minecart_improvements: false,
    }
}

const fn int_rule_range(
    name: &'static str,
    category: GameRuleCategory,
    default: i32,
    min: i32,
    max: i32,
) -> GameRuleDefinition {
    int_rule_range_feature(name, category, default, min, max, false)
}

const fn int_rule_range_feature(
    name: &'static str,
    category: GameRuleCategory,
    default: i32,
    min: i32,
    max: i32,
    requires_minecart_improvements: bool,
) -> GameRuleDefinition {
    GameRuleDefinition {
        name,
        category,
        rule_type: GameRuleType::Int,
        default: GameRuleValue::Int(default),
        min: Some(min),
        max: Some(max),
        requires_minecart_improvements,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        game_rule_definition, normalize_game_rule_name, parse_game_rule_value, vanilla_game_rules,
        GameRuleCategory, GameRuleError, GameRuleType, GameRuleValue, GameRules,
        VANILLA_GAME_RULE_COUNT_26_1_2,
    };

    #[test]
    fn registry_matches_vanilla_26_1_2_names_defaults_categories_and_feature_gate() {
        let rules = vanilla_game_rules();
        assert_eq!(rules.len(), VANILLA_GAME_RULE_COUNT_26_1_2);
        assert_eq!(rules.first().unwrap().name, "advance_time");
        assert_eq!(rules.last().unwrap().name, "water_source_conversion");

        let max_minecart_speed = game_rule_definition("max_minecart_speed").unwrap();
        assert_eq!(max_minecart_speed.rule_type, GameRuleType::Int);
        assert_eq!(max_minecart_speed.default, GameRuleValue::Int(8));
        assert_eq!(max_minecart_speed.min, Some(1));
        assert_eq!(max_minecart_speed.max, Some(1000));
        assert!(max_minecart_speed.requires_minecart_improvements);

        assert_eq!(
            game_rule_definition("mob_griefing").unwrap().category,
            GameRuleCategory::Mobs
        );
        assert_eq!(
            game_rule_definition("immediate_respawn").unwrap().default,
            GameRuleValue::Bool(false)
        );
        assert_eq!(
            game_rule_definition("random_tick_speed").unwrap().default,
            GameRuleValue::Int(3)
        );
    }

    #[test]
    fn legacy_command_aliases_normalize_to_current_rule_ids() {
        assert_eq!(normalize_game_rule_name("doDaylightCycle"), "advance_time");
        assert_eq!(normalize_game_rule_name("keepInventory"), "keep_inventory");
        assert_eq!(normalize_game_rule_name("mobGriefing"), "mob_griefing");
        assert_eq!(
            normalize_game_rule_name("minecraft:randomTickSpeed"),
            "random_tick_speed"
        );
    }

    #[test]
    fn parsing_enforces_bool_and_integer_bounds() {
        let keep_inventory = game_rule_definition("keep_inventory").unwrap();
        assert_eq!(
            parse_game_rule_value("true", keep_inventory),
            Ok(GameRuleValue::Bool(true))
        );
        assert_eq!(
            parse_game_rule_value("1", keep_inventory),
            Err(GameRuleError::WrongType)
        );

        let snow = game_rule_definition("max_snow_accumulation_height").unwrap();
        assert_eq!(parse_game_rule_value("8", snow), Ok(GameRuleValue::Int(8)));
        assert_eq!(
            parse_game_rule_value("9", snow),
            Err(GameRuleError::OutOfRange)
        );

        let fire = game_rule_definition("fire_spread_radius_around_player").unwrap();
        assert_eq!(
            parse_game_rule_value("-1", fire),
            Ok(GameRuleValue::Int(-1))
        );
    }

    #[test]
    fn game_rules_copy_feature_filter_and_emit_command_sync_shape() {
        let without_feature = GameRules::new(false);
        assert!(without_feature.get("max_minecart_speed").is_none());

        let mut rules = GameRules::new(true);
        assert_eq!(rules.get("max_minecart_speed"), Some(GameRuleValue::Int(8)));
        let sync = rules.set("keepInventory", "true").unwrap();
        assert_eq!(sync.rule, "minecraft:keep_inventory");
        assert_eq!(sync.value, "true");
        assert_eq!(sync.command_result, 1);
        assert_eq!(rules.get("keep_inventory"), Some(GameRuleValue::Bool(true)));
        assert_eq!(
            rules.set("max_snow_accumulation_height", "9"),
            Err(GameRuleError::OutOfRange)
        );
    }

    #[test]
    fn runtime_effects_expose_all_client_and_server_rule_hooks() {
        let mut rules = GameRules::new(true);
        rules.set("doImmediateRespawn", "true").unwrap();
        rules.set("keepInventory", "true").unwrap();
        rules.set("randomTickSpeed", "12").unwrap();
        rules.set("mobGriefing", "false").unwrap();
        rules.set("sendCommandFeedback", "false").unwrap();

        let effects = rules.runtime_effects();
        assert!(effects.immediate_respawn);
        assert!(effects.keep_inventory);
        assert_eq!(effects.random_tick_speed, 12);
        assert!(!effects.mob_griefing);
        assert!(!effects.send_command_feedback);
        assert!(effects.allow_entering_nether_using_portals);
        assert!(effects.command_block_output);
        assert!(effects.ender_pearls_vanish_on_death);
        assert!(effects.global_sound_events);
        assert!(effects.log_admin_commands);
        assert!(effects.spawn_patrols);
        assert!(effects.spawn_wardens);
        assert_eq!(effects.max_block_modifications, 32768);
        assert_eq!(effects.nether_portal_default_delay, 80);
        assert_eq!(effects.max_minecart_speed, 8);
    }
}
