use super::*;

pub const DECORATED_POT_PATTERNS: &[DecoratedPotPatternEntry] = &[
    pot_pattern("blank", "decorated_pot_side", "minecraft:brick"),
    pot_pattern(
        "angler",
        "angler_pottery_pattern",
        "minecraft:angler_pottery_sherd",
    ),
    pot_pattern(
        "archer",
        "archer_pottery_pattern",
        "minecraft:archer_pottery_sherd",
    ),
    pot_pattern(
        "arms_up",
        "arms_up_pottery_pattern",
        "minecraft:arms_up_pottery_sherd",
    ),
    pot_pattern(
        "blade",
        "blade_pottery_pattern",
        "minecraft:blade_pottery_sherd",
    ),
    pot_pattern(
        "brewer",
        "brewer_pottery_pattern",
        "minecraft:brewer_pottery_sherd",
    ),
    pot_pattern(
        "burn",
        "burn_pottery_pattern",
        "minecraft:burn_pottery_sherd",
    ),
    pot_pattern(
        "danger",
        "danger_pottery_pattern",
        "minecraft:danger_pottery_sherd",
    ),
    pot_pattern(
        "explorer",
        "explorer_pottery_pattern",
        "minecraft:explorer_pottery_sherd",
    ),
    pot_pattern(
        "flow",
        "flow_pottery_pattern",
        "minecraft:flow_pottery_sherd",
    ),
    pot_pattern(
        "friend",
        "friend_pottery_pattern",
        "minecraft:friend_pottery_sherd",
    ),
    pot_pattern(
        "guster",
        "guster_pottery_pattern",
        "minecraft:guster_pottery_sherd",
    ),
    pot_pattern(
        "heart",
        "heart_pottery_pattern",
        "minecraft:heart_pottery_sherd",
    ),
    pot_pattern(
        "heartbreak",
        "heartbreak_pottery_pattern",
        "minecraft:heartbreak_pottery_sherd",
    ),
    pot_pattern(
        "howl",
        "howl_pottery_pattern",
        "minecraft:howl_pottery_sherd",
    ),
    pot_pattern(
        "miner",
        "miner_pottery_pattern",
        "minecraft:miner_pottery_sherd",
    ),
    pot_pattern(
        "mourner",
        "mourner_pottery_pattern",
        "minecraft:mourner_pottery_sherd",
    ),
    pot_pattern(
        "plenty",
        "plenty_pottery_pattern",
        "minecraft:plenty_pottery_sherd",
    ),
    pot_pattern(
        "prize",
        "prize_pottery_pattern",
        "minecraft:prize_pottery_sherd",
    ),
    pot_pattern(
        "scrape",
        "scrape_pottery_pattern",
        "minecraft:scrape_pottery_sherd",
    ),
    pot_pattern(
        "sheaf",
        "sheaf_pottery_pattern",
        "minecraft:sheaf_pottery_sherd",
    ),
    pot_pattern(
        "shelter",
        "shelter_pottery_pattern",
        "minecraft:shelter_pottery_sherd",
    ),
    pot_pattern(
        "skull",
        "skull_pottery_pattern",
        "minecraft:skull_pottery_sherd",
    ),
    pot_pattern(
        "snort",
        "snort_pottery_pattern",
        "minecraft:snort_pottery_sherd",
    ),
];

pub fn decorated_pot_pattern_from_item(item_id: &str) -> Option<&'static str> {
    DECORATED_POT_PATTERNS
        .iter()
        .find(|entry| entry.item == item_id)
        .map(|entry| entry.key)
}

pub fn decorated_pot_pattern_asset_id(key: &str) -> Option<&'static str> {
    DECORATED_POT_PATTERNS
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| entry.pattern.asset_id)
}

const fn pot_pattern(
    key: &'static str,
    asset_id: &'static str,
    item: &'static str,
) -> DecoratedPotPatternEntry {
    DecoratedPotPatternEntry {
        key,
        pattern: DecoratedPotPattern { asset_id },
        item,
    }
}
