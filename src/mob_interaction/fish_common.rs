use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbstractFishAttributes {
    pub max_health: f32,
    pub panic_speed_modifier: f32,
    pub avoid_player_distance: f32,
    pub avoid_player_near_speed: f32,
    pub avoid_player_far_speed: f32,
    pub random_swim_speed_modifier: f32,
    pub random_swim_interval_ticks: i32,
    pub water_drag: f32,
    pub no_target_gravity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishFlopStep {
    pub jump: bool,
    pub sync_needed: bool,
    pub play_flop_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchoolingFishState {
    pub school_size: i32,
    pub max_school_size: i32,
    pub leader_alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishBucketModel {
    pub bucket_item: &'static str,
    pub pickup_sound: &'static str,
    pub from_bucket_save_field: &'static str,
    pub requires_persistence_from_bucket: bool,
    pub discard_on_pickup: bool,
}

pub const ABSTRACT_FISH_MAX_HEALTH: f32 = 3.0;
pub const ABSTRACT_FISH_MAX_SPAWN_CLUSTER_SIZE: i32 = 8;
pub const ABSTRACT_FISH_PANIC_SPEED_MODIFIER: f32 = 1.25;
pub const ABSTRACT_FISH_AVOID_PLAYER_DISTANCE: f32 = 8.0;
pub const ABSTRACT_FISH_AVOID_PLAYER_NEAR_SPEED: f32 = 1.6;
pub const ABSTRACT_FISH_AVOID_PLAYER_FAR_SPEED: f32 = 1.4;
pub const ABSTRACT_FISH_RANDOM_SWIM_SPEED_MODIFIER: f32 = 1.0;
pub const ABSTRACT_FISH_RANDOM_SWIM_INTERVAL_TICKS: i32 = 40;
pub const ABSTRACT_FISH_WATER_DRAG: f32 = 0.9;
pub const ABSTRACT_FISH_NO_TARGET_GRAVITY: f32 = -0.005;
pub const ABSTRACT_FISH_EYE_WATER_BOOST_Y: f32 = 0.005;
pub const ABSTRACT_FISH_FLOP_JUMP_Y: f32 = 0.4;
pub const ABSTRACT_FISH_FLOP_RANDOM_XZ_SCALE: f32 = 0.05;
pub const SCHOOLING_FISH_NEIGHBOR_SCAN_CHANCE_BOUND: i32 = 200;
pub const SCHOOLING_FISH_NEIGHBOR_SCAN_HIT: i32 = 1;
pub const SCHOOLING_FISH_NEIGHBOR_SCAN_RANGE: f32 = 8.0;
pub const SCHOOLING_FISH_LEADER_RANGE_SQR: f32 = 121.0;
pub const SALMON_MAX_SCHOOL_SIZE: i32 = 5;
pub const COD_BUCKET_ITEM: &str = "minecraft:cod_bucket";
pub const SALMON_BUCKET_ITEM: &str = "minecraft:salmon_bucket";
pub const TROPICAL_FISH_BUCKET_ITEM: &str = "minecraft:tropical_fish_bucket";
pub const PUFFERFISH_BUCKET_ITEM: &str = "minecraft:pufferfish_bucket";
pub const FISH_PICKUP_SOUND: &str = "minecraft:item.bucket.fill_fish";
pub const FISH_FROM_BUCKET_SAVE_FIELD: &str = "FromBucket";

pub fn abstract_fish_attributes() -> AbstractFishAttributes {
    AbstractFishAttributes {
        max_health: ABSTRACT_FISH_MAX_HEALTH,
        panic_speed_modifier: ABSTRACT_FISH_PANIC_SPEED_MODIFIER,
        avoid_player_distance: ABSTRACT_FISH_AVOID_PLAYER_DISTANCE,
        avoid_player_near_speed: ABSTRACT_FISH_AVOID_PLAYER_NEAR_SPEED,
        avoid_player_far_speed: ABSTRACT_FISH_AVOID_PLAYER_FAR_SPEED,
        random_swim_speed_modifier: ABSTRACT_FISH_RANDOM_SWIM_SPEED_MODIFIER,
        random_swim_interval_ticks: ABSTRACT_FISH_RANDOM_SWIM_INTERVAL_TICKS,
        water_drag: ABSTRACT_FISH_WATER_DRAG,
        no_target_gravity: ABSTRACT_FISH_NO_TARGET_GRAVITY,
    }
}

pub fn fish_requires_custom_persistence(super_requires: bool, from_bucket: bool) -> bool {
    super_requires || from_bucket
}

pub fn fish_remove_when_far_away(from_bucket: bool, has_custom_name: bool) -> bool {
    !from_bucket && !has_custom_name
}

pub fn fish_can_random_swim(is_schooling_follower: bool) -> bool {
    !is_schooling_follower
}

pub fn fish_flop_step(in_water: bool, on_ground: bool, vertical_collision: bool) -> FishFlopStep {
    let jump = !in_water && on_ground && vertical_collision;
    FishFlopStep {
        jump,
        sync_needed: jump,
        play_flop_sound: jump,
    }
}

pub fn fish_travel_y_delta(target_present: bool, old_delta_y: f32) -> f32 {
    if target_present {
        old_delta_y
    } else {
        old_delta_y + ABSTRACT_FISH_NO_TARGET_GRAVITY
    }
}

pub fn schooling_fish_can_be_followed(state: SchoolingFishState) -> bool {
    state.school_size > 1 && state.school_size < state.max_school_size
}

pub fn schooling_fish_is_follower(leader_present: bool, leader_alive: bool) -> bool {
    leader_present && leader_alive
}

pub fn schooling_fish_should_reset_size(random_0_to_199: i32, neighbor_count: usize) -> bool {
    random_0_to_199.rem_euclid(SCHOOLING_FISH_NEIGHBOR_SCAN_CHANCE_BOUND)
        == SCHOOLING_FISH_NEIGHBOR_SCAN_HIT
        && neighbor_count <= 1
}

pub fn schooling_fish_followers_added(
    current_school_size: i32,
    max_school_size: i32,
    candidates: usize,
) -> i32 {
    (max_school_size - current_school_size)
        .max(0)
        .min(candidates as i32)
}

pub fn fish_bucket_model(entity_type: &str, from_bucket: bool) -> Option<FishBucketModel> {
    let bucket_item = match entity_type {
        "minecraft:cod" => COD_BUCKET_ITEM,
        "minecraft:salmon" => SALMON_BUCKET_ITEM,
        "minecraft:tropical_fish" => TROPICAL_FISH_BUCKET_ITEM,
        "minecraft:pufferfish" => PUFFERFISH_BUCKET_ITEM,
        _ => return None,
    };
    Some(FishBucketModel {
        bucket_item,
        pickup_sound: FISH_PICKUP_SOUND,
        from_bucket_save_field: FISH_FROM_BUCKET_SAVE_FIELD,
        requires_persistence_from_bucket: from_bucket,
        discard_on_pickup: true,
    })
}

