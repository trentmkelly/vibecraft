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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndermiteTickOutcome {
    pub life: i32,
    pub discard: bool,
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
pub const ENDERMITE_STEP_SOUND_VOLUME: f32 = 0.15;
pub const ENDERMITE_STEP_SOUND_PITCH: f32 = 1.0;
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
