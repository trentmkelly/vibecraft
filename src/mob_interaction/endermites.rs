#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndermiteState {
    pub life: i32,
    pub persistent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermiteAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermiteClassSurface {
    pub movement_emission: &'static str,
    pub ambient_sound: &'static str,
    pub hurt_sound: &'static str,
    pub death_sound: &'static str,
    pub step_sound: &'static str,
    pub step_sound_volume: f32,
    pub step_sound_pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermiteGoalSurface {
    pub float_goal_priority: i32,
    pub powder_snow_goal_priority: i32,
    pub melee_attack_priority: i32,
    pub melee_attack_speed: f32,
    pub melee_attack_follow_even_if_not_seen: bool,
    pub random_stroll_priority: i32,
    pub random_stroll_speed: f32,
    pub look_at_player_priority: i32,
    pub look_at_player_range: f32,
    pub random_look_priority: i32,
    pub hurt_by_target_priority: i32,
    pub hurt_by_alerts_others: bool,
    pub nearest_player_target_priority: i32,
    pub nearest_player_must_see: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndermiteTickOutcome {
    pub life: i32,
    pub discard: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermiteRotationTick {
    pub y_rot: f32,
    pub y_body_rot: f32,
}

pub const ENDERMITE_MAX_LIFE_TICKS: i32 = 2_400;
pub const ENDERMITE_DEFAULT_LIFE: i32 = 0;
pub const ENDERMITE_XP_REWARD: i32 = 3;
pub const ENDERMITE_MAX_HEALTH: f32 = 8.0;
pub const ENDERMITE_MOVEMENT_SPEED: f32 = 0.25;
pub const ENDERMITE_ATTACK_DAMAGE: f32 = 2.0;
pub const ENDERMITE_ENDER_PEARL_SPAWN_CHANCE: f32 = 0.05;
pub const ENDERMITE_NEAREST_PLAYER_SPAWN_REJECTION_RANGE: f64 = 5.0;
pub const ENDERMITE_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const ENDERMITE_CLIENT_PORTAL_PARTICLES_PER_TICK: i32 = 2;
pub const ENDERMITE_MOVEMENT_EMISSION: &str = "events";
pub const ENDERMITE_AMBIENT_SOUND: &str = "minecraft:entity.endermite.ambient";
pub const ENDERMITE_HURT_SOUND: &str = "minecraft:entity.endermite.hurt";
pub const ENDERMITE_DEATH_SOUND: &str = "minecraft:entity.endermite.death";
pub const ENDERMITE_STEP_SOUND: &str = "minecraft:entity.endermite.step";
pub const ENDERMITE_STEP_SOUND_VOLUME: f32 = 0.15;
pub const ENDERMITE_STEP_SOUND_PITCH: f32 = 1.0;
pub const ENDERMITE_FLOAT_GOAL_PRIORITY: i32 = 1;
pub const ENDERMITE_POWDER_SNOW_GOAL_PRIORITY: i32 = 1;
pub const ENDERMITE_MELEE_ATTACK_PRIORITY: i32 = 2;
pub const ENDERMITE_MELEE_ATTACK_SPEED: f32 = 1.0;
pub const ENDERMITE_RANDOM_STROLL_PRIORITY: i32 = 3;
pub const ENDERMITE_RANDOM_STROLL_SPEED: f32 = 1.0;
pub const ENDERMITE_LOOK_AT_PLAYER_PRIORITY: i32 = 7;
pub const ENDERMITE_RANDOM_LOOK_PRIORITY: i32 = 8;
pub const ENDERMITE_HURT_BY_TARGET_PRIORITY: i32 = 1;
pub const ENDERMITE_NEAREST_PLAYER_TARGET_PRIORITY: i32 = 2;
pub const ENDERMAN_TARGETS_ENDERMITES: bool = true;

impl EndermiteState {
    pub fn new() -> Self {
        Self {
            life: ENDERMITE_DEFAULT_LIFE,
            persistent: false,
        }
    }

    pub fn read_save_data(life: Option<i32>, persistent: bool) -> Self {
        Self {
            life: life.unwrap_or(ENDERMITE_DEFAULT_LIFE),
            persistent,
        }
    }

    pub fn ai_step(&mut self) -> EndermiteTickOutcome {
        if !self.persistent {
            self.life += 1;
        }
        EndermiteTickOutcome {
            life: self.life,
            discard: self.life >= ENDERMITE_MAX_LIFE_TICKS,
        }
    }
}

impl Default for EndermiteState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn endermite_attributes() -> EndermiteAttributes {
    EndermiteAttributes {
        max_health: ENDERMITE_MAX_HEALTH,
        movement_speed: ENDERMITE_MOVEMENT_SPEED,
        attack_damage: ENDERMITE_ATTACK_DAMAGE,
        xp_reward: ENDERMITE_XP_REWARD,
    }
}

pub fn endermite_class_surface() -> EndermiteClassSurface {
    EndermiteClassSurface {
        movement_emission: ENDERMITE_MOVEMENT_EMISSION,
        ambient_sound: ENDERMITE_AMBIENT_SOUND,
        hurt_sound: ENDERMITE_HURT_SOUND,
        death_sound: ENDERMITE_DEATH_SOUND,
        step_sound: ENDERMITE_STEP_SOUND,
        step_sound_volume: ENDERMITE_STEP_SOUND_VOLUME,
        step_sound_pitch: ENDERMITE_STEP_SOUND_PITCH,
    }
}

pub fn endermite_goal_surface() -> EndermiteGoalSurface {
    EndermiteGoalSurface {
        float_goal_priority: ENDERMITE_FLOAT_GOAL_PRIORITY,
        powder_snow_goal_priority: ENDERMITE_POWDER_SNOW_GOAL_PRIORITY,
        melee_attack_priority: ENDERMITE_MELEE_ATTACK_PRIORITY,
        melee_attack_speed: ENDERMITE_MELEE_ATTACK_SPEED,
        melee_attack_follow_even_if_not_seen: false,
        random_stroll_priority: ENDERMITE_RANDOM_STROLL_PRIORITY,
        random_stroll_speed: ENDERMITE_RANDOM_STROLL_SPEED,
        look_at_player_priority: ENDERMITE_LOOK_AT_PLAYER_PRIORITY,
        look_at_player_range: ENDERMITE_LOOK_AT_PLAYER_RANGE,
        random_look_priority: ENDERMITE_RANDOM_LOOK_PRIORITY,
        hurt_by_target_priority: ENDERMITE_HURT_BY_TARGET_PRIORITY,
        hurt_by_alerts_others: true,
        nearest_player_target_priority: ENDERMITE_NEAREST_PLAYER_TARGET_PRIORITY,
        nearest_player_must_see: true,
    }
}

pub fn endermite_tick_rotation(y_rot: f32) -> EndermiteRotationTick {
    EndermiteRotationTick {
        y_rot,
        y_body_rot: y_rot,
    }
}

pub fn endermite_set_y_body_rot(_current_y_rot: f32, requested_y_body_rot: f32) -> EndermiteRotationTick {
    EndermiteRotationTick {
        y_rot: requested_y_body_rot,
        y_body_rot: requested_y_body_rot,
    }
}

pub fn endermite_spawn_allowed(
    any_light_monster_spawn_rules_pass: bool,
    spawner_reason: bool,
    nearest_player_within_5_blocks: bool,
) -> bool {
    any_light_monster_spawn_rules_pass && (spawner_reason || !nearest_player_within_5_blocks)
}

pub fn endermite_from_ender_pearl(random_float: f32, level_spawning_monsters: bool) -> bool {
    level_spawning_monsters && random_float < ENDERMITE_ENDER_PEARL_SPAWN_CHANCE
}

pub fn enderman_targets_endermite() -> bool {
    ENDERMAN_TARGETS_ENDERMITES
}
