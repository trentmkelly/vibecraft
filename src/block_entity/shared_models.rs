use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SculkCatalystEventResult {
    Ignored,
    Bloom {
        pulse_ticks: i32,
        consumed_experience: bool,
        added_cursors: usize,
        award_it_spreads: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SculkShriekerBlockEntity {
    pub warning_level: i32,
    pub vibration_data: VibrationData,
    pub shrieking_ticks: i32,
    pub can_summon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SculkShriekResult {
    Ignored,
    Shriek {
        warning_level: i32,
    },
    ReplySound {
        warning_level: i32,
        darkness_radius: i32,
    },
    SummonWarden {
        warning_level: i32,
        attempts: i32,
        range_xz: i32,
        range_y: i32,
        darkness_radius: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SculkSensorTickResult {
    None,
    Particle { travel_time_in_ticks: i32 },
    Activate { frequency: u8, redstone: u8 },
    Cooldown,
    Deactivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellTickEffects {
    pub play_resonate_sound: bool,
    pub glowing_raiders: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BellBlockEntity {
    pub last_ring_timestamp: u64,
    pub ticks: i32,
    pub shaking: bool,
    pub click_direction: Option<Direction>,
    pub heard_bell_entities: usize,
    pub nearby_raiders_within_hear_radius: usize,
    pub nearby_raiders_within_highlight_radius: usize,
    pub resonating: bool,
    pub resonation_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushResult {
    CoolingDown,
    InProgress { dusted: i32 },
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushableBlockEntity {
    pub brush_count: i32,
    pub brush_count_resets_at_tick: u64,
    pub cooldown_ends_at_tick: u64,
    pub item: Option<PotItemStack>,
    pub hit_direction: Option<Direction>,
    pub loot_table: Option<String>,
    pub loot_table_seed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockEntityError {
    UnknownType(String),
    MissingId,
    InvalidBlockState {
        ty: BlockEntityTypeId,
        block_state: String,
    },
    UnsupportedDataVersion(String),
}
