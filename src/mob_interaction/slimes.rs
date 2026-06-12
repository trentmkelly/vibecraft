#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlimeFamilyKind {
    Slime,
    MagmaCube,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeFamilyState {
    pub kind: SlimeFamilyKind,
    pub size: i32,
    pub was_on_ground: bool,
    pub target_squish: f32,
    pub squish: f32,
    pub old_squish: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeFamilyAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub armor: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeSplitChild {
    pub size: i32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub z_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeSpawnRuleInput {
    pub peaceful: bool,
    pub spawner_reason: bool,
    pub mob_spawn_rules_pass: bool,
    pub allows_surface_slime_spawns_biome: bool,
    pub y: i32,
    pub surface_slime_spawn_chance: f32,
    pub surface_random_float: f32,
    pub max_local_raw_brightness: i32,
    pub brightness_random_bound_8: i32,
    pub worldgen_level: bool,
    pub slime_chunk: bool,
    pub underground_random_bound_10: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeFloatGoalStep {
    pub can_use: bool,
    pub jump: bool,
    pub wanted_movement: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeMoveControlStep {
    pub speed: f32,
    pub jump: bool,
    pub play_jump_sound: bool,
    pub next_jump_delay: i32,
    pub zero_strafe: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlimeGoalRegistration {
    pub selector: SlimeGoalSelector,
    pub priority: i32,
    pub goal: SlimeGoalKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlimeGoalSelector {
    Goal,
    Target,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlimeGoalKind {
    Float,
    Attack,
    RandomDirection,
    KeepOnJumping,
    NearestPlayer,
    NearestIronGolem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlimeSoundSet {
    pub hurt: &'static str,
    pub death: &'static str,
    pub squish: &'static str,
    pub jump: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeSyncedSizeUpdate {
    pub refresh_dimensions: bool,
    pub y_rot_from_head: bool,
    pub body_rot_from_head: bool,
    pub water_splash: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeAttackGoalStep {
    pub can_use: bool,
    pub can_continue: bool,
    pub next_grow_tired_timer: i32,
    pub look_at_target: bool,
    pub set_direction: bool,
    pub aggressive: bool,
}

pub const SLIME_MIN_SIZE: i32 = 1;
pub const SLIME_MAX_SIZE: i32 = 127;
pub const SLIME_MAX_NATURAL_SIZE: i32 = 4;
pub const SLIME_DEFAULT_SERIALIZED_SIZE: i32 = 0;
pub const SLIME_DEFAULT_WAS_ON_GROUND: bool = false;
pub const SLIME_DEFAULT_SIZE: i32 = 1;
pub const SLIME_BASE_MOVEMENT_SPEED: f32 = 0.2;
pub const SLIME_MOVEMENT_SPEED_PER_SIZE: f32 = 0.1;
pub const SLIME_FLOAT_WANTED_MOVEMENT: f32 = 1.2;
pub const SLIME_FLOAT_JUMP_CHANCE: f32 = 0.8;
pub const SLIME_KEEP_JUMPING_WANTED_MOVEMENT: f32 = 1.0;
pub const SLIME_ATTACK_TARGET_VERTICAL_RANGE: f64 = 4.0;
pub const SLIME_ATTACK_GROW_TIRED_TICKS: i32 = 300;
pub const SLIME_RANDOM_DIRECTION_MIN_TICKS: i32 = 40;
pub const SLIME_RANDOM_DIRECTION_RANDOM_BOUND: i32 = 60;
pub const SLIME_JUMP_DELAY_MIN: i32 = 10;
pub const SLIME_JUMP_DELAY_RANDOM_BOUND: i32 = 20;
pub const SLIME_AGGRESSIVE_JUMP_DELAY_DIVISOR: i32 = 3;
pub const SLIME_SPLIT_MIN_COUNT: i32 = 2;
pub const SLIME_SPLIT_RANDOM_BOUND: i32 = 3;
pub const SLIME_SOUND_VOLUME_PER_SIZE: f32 = 0.4;
pub const SLIME_PASSENGER_ATTACHMENT_SIZE_OFFSET: f32 = 0.015625;
pub const SLIME_LANDING_PARTICLES_PER_WIDTH_UNIT: f32 = 16.0;
pub const SLIME_LANDING_TARGET_SQUISH: f32 = -0.5;
pub const SLIME_AIRBORNE_TARGET_SQUISH: f32 = 1.0;
pub const SLIME_SQUISH_INTERPOLATION: f32 = 0.5;
pub const SLIME_DECREASE_SQUISH_FACTOR: f32 = 0.6;
pub const SLIME_SOUND_SOURCE: &str = "hostile";
pub const SLIME_PARTICLE_TYPE: &str = "minecraft:item_slime";
pub const SLIME_SPLIT_CONVERSION_TYPE: &str = "split_on_death";
pub const SLIME_SPLIT_SPAWN_REASON: &str = "triggered";
pub const SLIME_SYNCED_SIZE_WATER_SPLASH_RANDOM_BOUND: i32 = 20;
pub const SLIME_MAX_HEAD_X_ROT: i32 = 0;
pub const SLIME_ATTACK_SOUND: &str = "minecraft:entity.slime.attack";
pub const SLIME_ATTACK_SOUND_VOLUME: f32 = 1.0;
pub const SLIME_MOVE_CONTROL_ROT_LERP_DEGREES: f32 = 90.0;
pub const MAGMA_CUBE_CREATE_ATTRIBUTES_MOVEMENT_SPEED: f32 = 0.2;
pub const MAGMA_CUBE_IS_ON_FIRE: bool = false;
pub const MAGMA_CUBE_ARMOR_PER_SIZE: f32 = 3.0;
pub const MAGMA_CUBE_JUMP_DELAY_MULTIPLIER: i32 = 4;
pub const MAGMA_CUBE_DECREASE_SQUISH_FACTOR: f32 = 0.9;
pub const MAGMA_CUBE_GROUND_JUMP_PER_SIZE: f32 = 0.1;
pub const MAGMA_CUBE_LAVA_JUMP_BASE: f32 = 0.22;
pub const MAGMA_CUBE_LAVA_JUMP_PER_SIZE: f32 = 0.05;
pub const MAGMA_CUBE_ATTACK_DAMAGE_BONUS: f32 = 2.0;

impl SlimeFamilyState {
    pub fn new(kind: SlimeFamilyKind, size: i32) -> Self {
        Self {
            kind,
            size: clamp_slime_size(size),
            was_on_ground: SLIME_DEFAULT_WAS_ON_GROUND,
            target_squish: 0.0,
            squish: 0.0,
            old_squish: 0.0,
        }
    }

    pub fn read_save_data(
        kind: SlimeFamilyKind,
        serialized_size: Option<i32>,
        was_on_ground: Option<bool>,
    ) -> Self {
        let mut state = Self::new(
            kind,
            serialized_size.unwrap_or(SLIME_DEFAULT_SERIALIZED_SIZE) + 1,
        );
        state.was_on_ground = was_on_ground.unwrap_or(SLIME_DEFAULT_WAS_ON_GROUND);
        state
    }

    pub fn serialized_size(self) -> i32 {
        self.size - 1
    }

    pub fn is_tiny(self) -> bool {
        self.size <= 1
    }

    pub fn attributes(self) -> SlimeFamilyAttributes {
        let base_attack = self.size as f32;
        SlimeFamilyAttributes {
            max_health: (self.size * self.size) as f32,
            movement_speed: SLIME_BASE_MOVEMENT_SPEED
                + SLIME_MOVEMENT_SPEED_PER_SIZE * self.size as f32,
            attack_damage: match self.kind {
                SlimeFamilyKind::Slime => base_attack,
                SlimeFamilyKind::MagmaCube => base_attack + MAGMA_CUBE_ATTACK_DAMAGE_BONUS,
            },
            armor: match self.kind {
                SlimeFamilyKind::Slime => 0.0,
                SlimeFamilyKind::MagmaCube => self.size as f32 * MAGMA_CUBE_ARMOR_PER_SIZE,
            },
            xp_reward: self.size,
        }
    }

    pub fn deals_damage(self, effective_ai: bool) -> bool {
        match self.kind {
            SlimeFamilyKind::Slime => !self.is_tiny() && effective_ai,
            SlimeFamilyKind::MagmaCube => effective_ai,
        }
    }

    pub fn jump_delay(self, random_0_to_19: i32, aggressive: bool) -> i32 {
        let mut delay =
            SLIME_JUMP_DELAY_MIN + random_0_to_19.rem_euclid(SLIME_JUMP_DELAY_RANDOM_BOUND);
        if self.kind == SlimeFamilyKind::MagmaCube {
            delay *= MAGMA_CUBE_JUMP_DELAY_MULTIPLIER;
        }
        if aggressive {
            delay /= SLIME_AGGRESSIVE_JUMP_DELAY_DIVISOR;
        }
        delay
    }

    pub fn tick_squish(&mut self, on_ground: bool) -> i32 {
        self.old_squish = self.squish;
        self.squish += (self.target_squish - self.squish) * SLIME_SQUISH_INTERPOLATION;
        let mut landing_particle_count = 0;
        if on_ground && !self.was_on_ground {
            let width = self.size as f32 * 2.0;
            landing_particle_count = (width * SLIME_LANDING_PARTICLES_PER_WIDTH_UNIT) as i32;
            self.target_squish = SLIME_LANDING_TARGET_SQUISH;
        } else if !on_ground && self.was_on_ground {
            self.target_squish = SLIME_AIRBORNE_TARGET_SQUISH;
        }
        self.was_on_ground = on_ground;
        self.target_squish *= match self.kind {
            SlimeFamilyKind::Slime => SLIME_DECREASE_SQUISH_FACTOR,
            SlimeFamilyKind::MagmaCube => MAGMA_CUBE_DECREASE_SQUISH_FACTOR,
        };
        landing_particle_count
    }

    pub fn ground_jump_y_velocity(self, base_jump_power: f32) -> f32 {
        match self.kind {
            SlimeFamilyKind::Slime => base_jump_power,
            SlimeFamilyKind::MagmaCube => {
                base_jump_power + self.size as f32 * MAGMA_CUBE_GROUND_JUMP_PER_SIZE
            }
        }
    }

    pub fn lava_jump_y_velocity(self) -> Option<f32> {
        (self.kind == SlimeFamilyKind::MagmaCube)
            .then_some(MAGMA_CUBE_LAVA_JUMP_BASE + self.size as f32 * MAGMA_CUBE_LAVA_JUMP_PER_SIZE)
    }

    pub fn float_goal_step(
        self,
        in_water: bool,
        in_lava: bool,
        has_slime_move_control: bool,
        random_float: f32,
    ) -> SlimeFloatGoalStep {
        let can_use = (in_water || in_lava) && has_slime_move_control;
        SlimeFloatGoalStep {
            can_use,
            jump: can_use && random_float < SLIME_FLOAT_JUMP_CHANCE,
            wanted_movement: if can_use {
                SLIME_FLOAT_WANTED_MOVEMENT
            } else {
                0.0
            },
        }
    }

    pub fn keep_on_jumping_can_use(self, passenger: bool) -> bool {
        !passenger
    }

    pub fn random_direction_can_use(
        self,
        target_present: bool,
        on_ground: bool,
        in_water: bool,
        in_lava: bool,
        has_levitation: bool,
        has_slime_move_control: bool,
    ) -> bool {
        !target_present
            && (on_ground || in_water || in_lava || has_levitation)
            && has_slime_move_control
    }

    pub fn random_direction_next_time(self, random_0_to_59: i32) -> i32 {
        SLIME_RANDOM_DIRECTION_MIN_TICKS
            + random_0_to_59.rem_euclid(SLIME_RANDOM_DIRECTION_RANDOM_BOUND)
    }

    pub fn attack_goal_start_timer(self) -> i32 {
        SLIME_ATTACK_GROW_TIRED_TICKS
    }

    pub fn attack_goal_step(
        self,
        target_present: bool,
        can_attack_target: bool,
        has_slime_move_control: bool,
        grow_tired_timer: i32,
        effective_ai: bool,
    ) -> SlimeAttackGoalStep {
        let can_use = target_present && can_attack_target && has_slime_move_control;
        let next_grow_tired_timer = grow_tired_timer - 1;
        let can_continue = target_present && can_attack_target && next_grow_tired_timer > 0;
        SlimeAttackGoalStep {
            can_use,
            can_continue,
            next_grow_tired_timer,
            look_at_target: target_present,
            set_direction: target_present && has_slime_move_control,
            aggressive: self.deals_damage(effective_ai),
        }
    }

    pub fn move_control_step(self, input: SlimeMoveControlInput) -> SlimeMoveControlStep {
        if !input.operation_move_to {
            return SlimeMoveControlStep {
                speed: 0.0,
                jump: false,
                play_jump_sound: false,
                next_jump_delay: input.jump_delay,
                zero_strafe: false,
            };
        }

        let speed = input.speed_modifier * input.movement_speed_attribute;
        if !input.on_ground {
            return SlimeMoveControlStep {
                speed,
                jump: false,
                play_jump_sound: false,
                next_jump_delay: input.jump_delay,
                zero_strafe: false,
            };
        }

        if input.jump_delay <= 0 {
            let next_jump_delay = self.jump_delay(input.random_0_to_19, input.aggressive);
            SlimeMoveControlStep {
                speed,
                jump: true,
                play_jump_sound: self.size > 0,
                next_jump_delay,
                zero_strafe: false,
            }
        } else {
            SlimeMoveControlStep {
                speed: 0.0,
                jump: false,
                play_jump_sound: false,
                next_jump_delay: input.jump_delay - 1,
                zero_strafe: true,
            }
        }
    }

    pub fn sound_volume(self) -> f32 {
        SLIME_SOUND_VOLUME_PER_SIZE * self.size as f32
    }

    pub fn passenger_attachment_y(self, dimensions_height: f32, scale: f32) -> f32 {
        dimensions_height - SLIME_PASSENGER_ATTACHMENT_SIZE_OFFSET * self.size as f32 * scale
    }

    pub fn default_dimension_scale(self, base_width: f32, base_height: f32) -> (f32, f32) {
        (base_width * self.size as f32, base_height * self.size as f32)
    }

    pub fn sound_set(self) -> SlimeSoundSet {
        let tiny = self.is_tiny();
        SlimeSoundSet {
            hurt: if tiny {
                "minecraft:entity.slime.hurt_small"
            } else {
                "minecraft:entity.slime.hurt"
            },
            death: if tiny {
                "minecraft:entity.slime.death_small"
            } else {
                "minecraft:entity.slime.death"
            },
            squish: if tiny {
                "minecraft:entity.slime.squish_small"
            } else {
                "minecraft:entity.slime.squish"
            },
            jump: if tiny {
                "minecraft:entity.slime.jump_small"
            } else {
                "minecraft:entity.slime.jump"
            },
        }
    }

    pub fn sound_pitch_multiplier(self) -> f32 {
        if self.is_tiny() {
            1.4
        } else {
            0.8
        }
    }

    pub fn split_children(self, split_random_0_to_2: i32) -> Vec<SlimeSplitChild> {
        if self.size <= 1 {
            return Vec::new();
        }
        let half_size = self.size / 2;
        let count =
            SLIME_SPLIT_MIN_COUNT + split_random_0_to_2.rem_euclid(SLIME_SPLIT_RANDOM_BOUND);
        let spawn_offset = self.size as f32 / 2.0;
        (0..count)
            .map(|i| SlimeSplitChild {
                size: half_size,
                x_offset: ((i % 2) as f32 - 0.5) * spawn_offset,
                y_offset: 0.5,
                z_offset: ((i / 2) as f32 - 0.5) * spawn_offset,
            })
            .collect()
    }
}

pub const SLIME_GOALS: [SlimeGoalRegistration; 6] = [
    SlimeGoalRegistration {
        selector: SlimeGoalSelector::Goal,
        priority: 1,
        goal: SlimeGoalKind::Float,
    },
    SlimeGoalRegistration {
        selector: SlimeGoalSelector::Goal,
        priority: 2,
        goal: SlimeGoalKind::Attack,
    },
    SlimeGoalRegistration {
        selector: SlimeGoalSelector::Goal,
        priority: 3,
        goal: SlimeGoalKind::RandomDirection,
    },
    SlimeGoalRegistration {
        selector: SlimeGoalSelector::Goal,
        priority: 5,
        goal: SlimeGoalKind::KeepOnJumping,
    },
    SlimeGoalRegistration {
        selector: SlimeGoalSelector::Target,
        priority: 1,
        goal: SlimeGoalKind::NearestPlayer,
    },
    SlimeGoalRegistration {
        selector: SlimeGoalSelector::Target,
        priority: 3,
        goal: SlimeGoalKind::NearestIronGolem,
    },
];

pub fn clamp_slime_size(size: i32) -> i32 {
    size.clamp(SLIME_MIN_SIZE, SLIME_MAX_SIZE)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlimeMoveControlInput {
    pub operation_move_to: bool,
    pub on_ground: bool,
    pub speed_modifier: f32,
    pub movement_speed_attribute: f32,
    pub jump_delay: i32,
    pub random_0_to_19: i32,
    pub aggressive: bool,
}

pub fn slime_finalize_spawn_size(
    size_scale_random_0_to_2: i32,
    special_multiplier: f32,
    boost_random_float: f32,
) -> i32 {
    let mut size_scale = size_scale_random_0_to_2.rem_euclid(3);
    if size_scale < 2 && boost_random_float < 0.5 * special_multiplier {
        size_scale += 1;
    }
    1 << size_scale
}

pub fn slime_spawn_allowed(input: SlimeSpawnRuleInput) -> bool {
    if input.peaceful {
        return false;
    }
    if input.spawner_reason {
        return input.mob_spawn_rules_pass;
    }
    if input.allows_surface_slime_spawns_biome
        && input.y > 50
        && input.y < 70
        && input.surface_random_float < input.surface_slime_spawn_chance
        && input.max_local_raw_brightness <= input.brightness_random_bound_8
    {
        return input.mob_spawn_rules_pass;
    }
    input.worldgen_level
        && input.underground_random_bound_10 == 0
        && input.slime_chunk
        && input.y < 40
        && input.mob_spawn_rules_pass
}

pub fn slime_target_player_allowed(slime_y: f64, target_y: f64) -> bool {
    (target_y - slime_y).abs() <= SLIME_ATTACK_TARGET_VERTICAL_RANGE
}

pub fn slime_synced_size_update(in_water: bool, random_0_to_19: i32) -> SlimeSyncedSizeUpdate {
    SlimeSyncedSizeUpdate {
        refresh_dimensions: true,
        y_rot_from_head: true,
        body_rot_from_head: true,
        water_splash: in_water
            && random_0_to_19.rem_euclid(SLIME_SYNCED_SIZE_WATER_SPLASH_RANDOM_BOUND) == 0,
    }
}

pub fn magma_cube_spawn_allowed(peaceful: bool) -> bool {
    !peaceful
}

pub fn magma_cube_is_on_fire() -> bool {
    MAGMA_CUBE_IS_ON_FIRE
}
