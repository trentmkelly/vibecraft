use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SilverfishAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SilverfishWakeStep {
    pub offset: (i32, i32, i32),
    pub action: SilverfishWakeAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilverfishWakeAction {
    DestroyInfestedBlock,
    RestoreHostBlock,
}

pub const SILVERFISH_MAX_HEALTH: f32 = 8.0;
pub const SILVERFISH_MOVEMENT_SPEED: f32 = 0.25;
pub const SILVERFISH_ATTACK_DAMAGE: f32 = 1.0;
pub const SILVERFISH_WAKE_DELAY_TICKS: i32 = 20;
pub const SILVERFISH_WAKE_SCAN_XZ_RADIUS: i32 = 10;
pub const SILVERFISH_WAKE_SCAN_Y_RADIUS: i32 = 5;
pub const SILVERFISH_MERGE_RANDOM_BOUND: i32 = 10;
pub const SILVERFISH_MERGE_SPEED: f32 = 1.0;
pub const SILVERFISH_MERGE_INTERVAL_TICKS: i32 = 10;
pub const SILVERFISH_WALK_TARGET_HOST_VALUE: f32 = 10.0;
pub const SILVERFISH_NEAR_PLAYER_SPAWN_BLOCK_RANGE: f64 = 5.0;
pub const SILVERFISH_STEP_SOUND_VOLUME: f32 = 0.15;
pub const SILVERFISH_STEP_SOUND_PITCH: f32 = 1.0;

pub fn silverfish_attributes() -> SilverfishAttributes {
    SilverfishAttributes {
        max_health: SILVERFISH_MAX_HEALTH,
        movement_speed: SILVERFISH_MOVEMENT_SPEED,
        attack_damage: SILVERFISH_ATTACK_DAMAGE,
    }
}

pub fn silverfish_spawn_allowed(
    any_light_monster_rules_pass: bool,
    spawn_reason_is_spawner: bool,
    nearest_player_within_5_blocks: bool,
) -> bool {
    any_light_monster_rules_pass && (spawn_reason_is_spawner || !nearest_player_within_5_blocks)
}

pub fn silverfish_notify_hurt_delay(
    current_look_for_friends: i32,
    source_has_entity: bool,
    source_always_triggers_silverfish: bool,
) -> i32 {
    if current_look_for_friends == 0 && (source_has_entity || source_always_triggers_silverfish) {
        SILVERFISH_WAKE_DELAY_TICKS
    } else {
        current_look_for_friends
    }
}

pub fn silverfish_merge_can_use(
    has_target: bool,
    navigation_done: bool,
    mob_griefing: bool,
    random_0_to_9: i32,
    adjacent_block: &'static str,
) -> bool {
    !has_target
        && navigation_done
        && mob_griefing
        && random_0_to_9.rem_euclid(SILVERFISH_MERGE_RANDOM_BOUND) == 0
        && silverfish_infested_block_for_host(adjacent_block).is_some()
}

pub fn silverfish_walk_target_value(block_below: &'static str, fallback: f32) -> f32 {
    if silverfish_infested_block_for_host(block_below).is_some() {
        SILVERFISH_WALK_TARGET_HOST_VALUE
    } else {
        fallback
    }
}

pub fn silverfish_infested_block_for_host(host_block: &'static str) -> Option<&'static str> {
    match host_block {
        "minecraft:stone" => Some("minecraft:infested_stone"),
        "minecraft:cobblestone" => Some("minecraft:infested_cobblestone"),
        "minecraft:stone_bricks" => Some("minecraft:infested_stone_bricks"),
        "minecraft:mossy_stone_bricks" => Some("minecraft:infested_mossy_stone_bricks"),
        "minecraft:cracked_stone_bricks" => Some("minecraft:infested_cracked_stone_bricks"),
        "minecraft:chiseled_stone_bricks" => Some("minecraft:infested_chiseled_stone_bricks"),
        "minecraft:deepslate" => Some("minecraft:infested_deepslate"),
        _ => None,
    }
}

pub fn silverfish_host_block_for_infested(infested_block: &'static str) -> Option<&'static str> {
    match infested_block {
        "minecraft:infested_stone" => Some("minecraft:stone"),
        "minecraft:infested_cobblestone" => Some("minecraft:cobblestone"),
        "minecraft:infested_stone_bricks" => Some("minecraft:stone_bricks"),
        "minecraft:infested_mossy_stone_bricks" => Some("minecraft:mossy_stone_bricks"),
        "minecraft:infested_cracked_stone_bricks" => Some("minecraft:cracked_stone_bricks"),
        "minecraft:infested_chiseled_stone_bricks" => Some("minecraft:chiseled_stone_bricks"),
        "minecraft:infested_deepslate" => Some("minecraft:deepslate"),
        _ => None,
    }
}

pub fn silverfish_infested_break_spawns_silverfish(
    block_drops_enabled: bool,
    tool_prevents_infested_spawns: bool,
) -> bool {
    block_drops_enabled && !tool_prevents_infested_spawns
}

pub fn silverfish_wake_scan_offsets() -> Vec<(i32, i32, i32)> {
    let mut offsets = Vec::new();
    for y in silverfish_java_symmetric_offsets(SILVERFISH_WAKE_SCAN_Y_RADIUS) {
        for x in silverfish_java_symmetric_offsets(SILVERFISH_WAKE_SCAN_XZ_RADIUS) {
            for z in silverfish_java_symmetric_offsets(SILVERFISH_WAKE_SCAN_XZ_RADIUS) {
                offsets.push((x, y, z));
            }
        }
    }
    offsets
}

pub fn silverfish_wake_step(
    offset: (i32, i32, i32),
    block: &'static str,
    mob_griefing: bool,
) -> Option<SilverfishWakeStep> {
    silverfish_host_block_for_infested(block).map(|_| SilverfishWakeStep {
        offset,
        action: if mob_griefing {
            SilverfishWakeAction::DestroyInfestedBlock
        } else {
            SilverfishWakeAction::RestoreHostBlock
        },
    })
}

pub fn silverfish_wake_steps_until_random_stop(
    blocks_in_java_scan_order: &[&'static str],
    mob_griefing: bool,
    random_stop_after_each_hit: &[bool],
) -> Vec<SilverfishWakeStep> {
    let mut steps = Vec::new();
    let mut hit_index = 0;
    for (offset, block) in silverfish_wake_scan_offsets()
        .into_iter()
        .zip(blocks_in_java_scan_order.iter())
    {
        if let Some(step) = silverfish_wake_step(offset, block, mob_griefing) {
            steps.push(step);
            let should_stop = random_stop_after_each_hit
                .get(hit_index)
                .copied()
                .unwrap_or(false);
            hit_index += 1;
            if should_stop {
                break;
            }
        }
    }
    steps
}

fn silverfish_java_symmetric_offsets(radius: i32) -> Vec<i32> {
    let mut values = Vec::new();
    let mut value = 0;
    while value <= radius && value >= -radius {
        values.push(value);
        value = if value <= 0 { 1 } else { 0 } - value;
    }
    values
}

