use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GiantAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub camera_distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GiantEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
    pub natural_spawn: bool,
    pub custom_ai_goals: i32,
}

pub const GIANT_MAX_HEALTH: f32 = 100.0;
pub const GIANT_MOVEMENT_SPEED: f32 = 0.5;
pub const GIANT_ATTACK_DAMAGE: f32 = 50.0;
pub const GIANT_CAMERA_DISTANCE: f32 = 16.0;
pub const GIANT_WIDTH: f32 = 3.6;
pub const GIANT_HEIGHT: f32 = 12.0;
pub const GIANT_EYE_HEIGHT: f32 = 10.44;
pub const GIANT_RIDING_OFFSET: f32 = -3.75;
pub const GIANT_CLIENT_TRACKING_RANGE: i32 = 10;
pub const GIANT_NOT_IN_PEACEFUL: bool = true;
pub const GIANT_NATURAL_SPAWN: bool = false;
pub const GIANT_CUSTOM_AI_GOALS: i32 = 0;

pub fn giant_attributes() -> GiantAttributes {
    GiantAttributes {
        max_health: GIANT_MAX_HEALTH,
        movement_speed: GIANT_MOVEMENT_SPEED,
        attack_damage: GIANT_ATTACK_DAMAGE,
        camera_distance: GIANT_CAMERA_DISTANCE,
    }
}

pub fn giant_entity_type_surface() -> GiantEntityTypeSurface {
    GiantEntityTypeSurface {
        width: GIANT_WIDTH,
        height: GIANT_HEIGHT,
        eye_height: GIANT_EYE_HEIGHT,
        riding_offset: GIANT_RIDING_OFFSET,
        client_tracking_range: GIANT_CLIENT_TRACKING_RANGE,
        not_in_peaceful: GIANT_NOT_IN_PEACEFUL,
        natural_spawn: GIANT_NATURAL_SPAWN,
        custom_ai_goals: GIANT_CUSTOM_AI_GOALS,
    }
}

pub fn giant_walk_target_value(pathfinding_cost_from_light_levels: f32) -> f32 {
    pathfinding_cost_from_light_levels
}

