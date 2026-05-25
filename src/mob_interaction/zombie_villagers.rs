
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieVillagerBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZombieVillagerInteraction {
    PassToZombie,
    ConsumeGoldenApple,
    StartConversion {
        consumed_golden_apple: bool,
        remove_weakness: bool,
        strength_effect_ticks: i32,
        strength_amplifier: i32,
        broadcast_event: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZombieVillagerConversionTick {
    pub conversion_time: i32,
    pub finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZombieVillagerFinishConversion {
    pub target_entity: &'static str,
    pub copy_position_motion_vehicle_passengers: bool,
    pub preserve_non_binding_equipment: bool,
    pub preserve_villager_data: bool,
    pub preserve_gossips: bool,
    pub copy_trade_offers: bool,
    pub preserve_xp: bool,
    pub finalize_spawn_reason: &'static str,
    pub refresh_brain: bool,
    pub trigger_cured_advancement: bool,
    pub emit_reputation_event: bool,
    pub nausea_ticks: i32,
    pub level_event: Option<i32>,
}

pub const ZOMBIE_VILLAGER_CONVERSION_WAIT_MIN: i32 = 3_600;
pub const ZOMBIE_VILLAGER_CONVERSION_WAIT_MAX: i32 = 6_000;
pub const ZOMBIE_VILLAGER_CONVERSION_WAIT_RANDOM_BOUND: i32 = 2_401;
pub const ZOMBIE_VILLAGER_SPECIAL_BLOCK_RADIUS: i32 = 4;
pub const ZOMBIE_VILLAGER_MAX_SPECIAL_BLOCKS_COUNT: i32 = 14;
pub const ZOMBIE_VILLAGER_SPECIAL_BLOCK_SCAN_CHANCE: f32 = 0.01;
pub const ZOMBIE_VILLAGER_SPECIAL_BLOCK_PROGRESS_CHANCE: f32 = 0.3;
pub const ZOMBIE_VILLAGER_CURE_ENTITY_EVENT: u8 = 16;
pub const ZOMBIE_VILLAGER_FINISH_CONVERSION_LEVEL_EVENT: i32 = 1027;
pub const ZOMBIE_VILLAGER_NAUSEA_TICKS: i32 = 200;
pub const ZOMBIE_VILLAGER_DEFAULT_XP: i32 = 0;
pub const ZOMBIE_VILLAGER_NOT_CONVERTING: i32 = -1;
pub const ZOMBIE_VILLAGER_BABY_WIDTH: f32 = 0.49;
pub const ZOMBIE_VILLAGER_BABY_HEIGHT: f32 = 0.99;
pub const ZOMBIE_VILLAGER_BABY_EYE_HEIGHT: f32 = 0.67;
pub const ZOMBIE_VILLAGER_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.125;

pub fn zombie_villager_baby_dimensions() -> ZombieVillagerBabyDimensions {
    ZombieVillagerBabyDimensions {
        width: ZOMBIE_VILLAGER_BABY_WIDTH,
        height: ZOMBIE_VILLAGER_BABY_HEIGHT,
        eye_height: ZOMBIE_VILLAGER_BABY_EYE_HEIGHT,
        vehicle_attachment_y: ZOMBIE_VILLAGER_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn zombie_villager_finalize_spawn_sets_biome_type(villager_data_finalized: bool) -> bool {
    !villager_data_finalized
}

pub fn zombie_villager_conversion_time_from_roll(random_0_to_2400: i32) -> i32 {
    ZOMBIE_VILLAGER_CONVERSION_WAIT_MIN
        + random_0_to_2400.rem_euclid(ZOMBIE_VILLAGER_CONVERSION_WAIT_RANDOM_BOUND)
}

pub fn zombie_villager_interact(
    item: &str,
    has_weakness: bool,
    server_side: bool,
    difficulty_id: i32,
    conversion_time: i32,
) -> ZombieVillagerInteraction {
    if item != "minecraft:golden_apple" {
        return ZombieVillagerInteraction::PassToZombie;
    }
    if !has_weakness {
        return ZombieVillagerInteraction::ConsumeGoldenApple;
    }
    if !server_side {
        return ZombieVillagerInteraction::StartConversion {
            consumed_golden_apple: true,
            remove_weakness: false,
            strength_effect_ticks: conversion_time,
            strength_amplifier: 0,
            broadcast_event: ZOMBIE_VILLAGER_CURE_ENTITY_EVENT,
        };
    }
    ZombieVillagerInteraction::StartConversion {
        consumed_golden_apple: true,
        remove_weakness: true,
        strength_effect_ticks: conversion_time,
        strength_amplifier: (difficulty_id - 1).min(0),
        broadcast_event: ZOMBIE_VILLAGER_CURE_ENTITY_EVENT,
    }
}

pub fn zombie_villager_remove_when_far_away(converting: bool, villager_xp: i32) -> bool {
    !converting && villager_xp == ZOMBIE_VILLAGER_DEFAULT_XP
}

pub fn zombie_villager_conversion_progress(
    scan_random_float: f32,
    special_blocks_found: i32,
    successful_progress_rolls: i32,
) -> i32 {
    if scan_random_float >= ZOMBIE_VILLAGER_SPECIAL_BLOCK_SCAN_CHANCE {
        return 1;
    }
    1 + successful_progress_rolls.min(
        special_blocks_found
            .max(0)
            .min(ZOMBIE_VILLAGER_MAX_SPECIAL_BLOCKS_COUNT),
    )
}

pub fn zombie_villager_conversion_tick(
    converting: bool,
    alive: bool,
    client_side: bool,
    conversion_time: i32,
    progress: i32,
) -> ZombieVillagerConversionTick {
    if client_side || !alive || !converting {
        return ZombieVillagerConversionTick {
            conversion_time,
            finished: false,
        };
    }
    let next = conversion_time - progress;
    ZombieVillagerConversionTick {
        conversion_time: next,
        finished: next <= 0,
    }
}

pub fn zombie_villager_finish_conversion(
    has_conversion_starter: bool,
    starter_is_server_player: bool,
    silent: bool,
) -> ZombieVillagerFinishConversion {
    let player_credit = has_conversion_starter && starter_is_server_player;
    ZombieVillagerFinishConversion {
        target_entity: "minecraft:villager",
        copy_position_motion_vehicle_passengers: false,
        preserve_non_binding_equipment: true,
        preserve_villager_data: true,
        preserve_gossips: true,
        copy_trade_offers: true,
        preserve_xp: true,
        finalize_spawn_reason: "conversion",
        refresh_brain: true,
        trigger_cured_advancement: player_credit,
        emit_reputation_event: player_credit,
        nausea_ticks: ZOMBIE_VILLAGER_NAUSEA_TICKS,
        level_event: (!silent).then_some(ZOMBIE_VILLAGER_FINISH_CONVERSION_LEVEL_EVENT),
    }
}

pub fn zombie_villager_set_villager_data_clears_offers(
    profession_changed: bool,
    had_trade_offers: bool,
) -> bool {
    profession_changed && had_trade_offers
}

