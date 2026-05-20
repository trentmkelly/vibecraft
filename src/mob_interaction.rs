#![allow(dead_code)]

pub const BABY_START_AGE: i32 = -24_000;
pub const AGE_LOCK_COOLDOWN_TICKS: i32 = 40;
pub const FORCED_AGE_PARTICLE_TICKS: i32 = 40;
pub const PARENT_AGE_AFTER_BREEDING: i32 = 6_000;
pub const IN_LOVE_TICKS: i32 = 600;
pub const ANIMAL_TEMPT_RANGE: f32 = 10.0;
pub const ANIMAL_AMBIENT_SOUND_INTERVAL: i32 = 120;
pub const TAMABLE_TELEPORT_DISTANCE_SQUARED: i32 = 144;
pub const TAMABLE_TELEPORT_ATTEMPTS: i32 = 10;
pub const TAMABLE_TELEPORT_MIN_HORIZONTAL: i32 = 2;
pub const TAMABLE_TELEPORT_MAX_HORIZONTAL: i32 = 3;
pub const TAMABLE_TELEPORT_MAX_VERTICAL: i32 = 1;
pub const HORSE_CHEST_SLOT_OFFSET: i32 = 499;
pub const HORSE_INVENTORY_SLOT_OFFSET: i32 = 500;
pub const HORSE_BREEDING_CROSS_FACTOR: f64 = 0.15;
pub const HORSE_INVENTORY_ROWS: i32 = 3;
pub const VILLAGER_INVENTORY_SIZE: usize = 8;
pub const VILLAGER_INVENTORY_SLOT_OFFSET: i32 = 300;
pub const NO_ANGER_END_TIME: i64 = -1;

pub const TAMABLE_FLAG_SITTING: u8 = 1;
pub const TAMABLE_FLAG_TAME: u8 = 4;
pub const HORSE_FLAG_TAME: u8 = 2;
pub const HORSE_FLAG_BRED: u8 = 8;
pub const HORSE_FLAG_EATING: u8 = 16;
pub const HORSE_FLAG_STANDING: u8 = 32;
pub const HORSE_FLAG_OPEN_MOUTH: u8 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreeperState {
    pub old_swell: i32,
    pub swell: i32,
    pub max_swell: i32,
    pub explosion_radius: i32,
    pub swell_dir: i32,
    pub powered: bool,
    pub ignited: bool,
    pub dropped_skulls: bool,
    pub alive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreeperTickResult {
    pub primed_sound: bool,
    pub prime_fuse_game_event: bool,
    pub explosion_radius: Option<f32>,
    pub lingering_cloud: Option<CreeperLingeringCloud>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreeperLingeringCloud {
    pub radius: f32,
    pub radius_on_use: f32,
    pub wait_time: i32,
    pub duration: i32,
    pub potion_duration_scale: f32,
    pub radius_per_tick: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreeperIgniterUse {
    NotIgniter,
    ConsumeOne,
    DamageOne,
}

pub const CREEPER_DEFAULT_SWELL_DIR: i32 = -1;
pub const CREEPER_DEFAULT_POWERED: bool = false;
pub const CREEPER_DEFAULT_IGNITED: bool = false;
pub const CREEPER_DEFAULT_MAX_SWELL: i32 = 30;
pub const CREEPER_DEFAULT_EXPLOSION_RADIUS: i32 = 3;
pub const CREEPER_MOVEMENT_SPEED: f32 = 0.25;
pub const CREEPER_SWELL_FALL_DAMAGE_FACTOR: f64 = 1.5;
pub const CREEPER_SWELL_FALL_DAMAGE_CLAMP_OFFSET: i32 = 5;
pub const CREEPER_POWERED_EXPLOSION_MULTIPLIER: f32 = 2.0;
pub const CREEPER_NORMAL_EXPLOSION_MULTIPLIER: f32 = 1.0;
pub const CREEPER_SWELL_RENDER_DENOMINATOR_OFFSET: i32 = 2;
pub const CREEPER_CAT_AVOID_DISTANCE: f32 = 6.0;
pub const CREEPER_CAT_AVOID_WALK_SPEED: f32 = 1.0;
pub const CREEPER_CAT_AVOID_SPRINT_SPEED: f32 = 1.2;
pub const CREEPER_STROLL_SPEED: f32 = 0.8;
pub const CREEPER_LOOK_AT_PLAYER_DISTANCE: f32 = 8.0;
pub const CREEPER_LINGERING_CLOUD_RADIUS: f32 = 2.5;
pub const CREEPER_LINGERING_CLOUD_RADIUS_ON_USE: f32 = -0.5;
pub const CREEPER_LINGERING_CLOUD_WAIT_TIME: i32 = 10;
pub const CREEPER_LINGERING_CLOUD_DURATION: i32 = 300;
pub const CREEPER_LINGERING_CLOUD_POTION_DURATION_SCALE: f32 = 0.25;

impl CreeperState {
    pub fn new() -> Self {
        Self {
            old_swell: 0,
            swell: 0,
            max_swell: CREEPER_DEFAULT_MAX_SWELL,
            explosion_radius: CREEPER_DEFAULT_EXPLOSION_RADIUS,
            swell_dir: CREEPER_DEFAULT_SWELL_DIR,
            powered: CREEPER_DEFAULT_POWERED,
            ignited: CREEPER_DEFAULT_IGNITED,
            dropped_skulls: false,
            alive: true,
        }
    }

    pub fn read_save_data(
        powered: bool,
        fuse: Option<i16>,
        explosion_radius: Option<i8>,
        ignited: bool,
    ) -> Self {
        let mut state = Self::new();
        state.powered = powered;
        state.max_swell = fuse.unwrap_or(CREEPER_DEFAULT_MAX_SWELL as i16) as i32;
        state.explosion_radius =
            explosion_radius.unwrap_or(CREEPER_DEFAULT_EXPLOSION_RADIUS as i8) as i32;
        if ignited {
            state.ignite();
        }
        state
    }

    pub fn ignite(&mut self) {
        self.ignited = true;
    }

    pub fn thunder_hit(&mut self) {
        self.powered = true;
    }

    pub fn set_swell_dir(&mut self, dir: i32) {
        self.swell_dir = dir;
    }

    pub fn cause_fall_damage(&mut self, fall_distance: f64) {
        self.swell += (fall_distance * CREEPER_SWELL_FALL_DAMAGE_FACTOR) as i32;
        let max_preloaded_swell = self.max_swell - CREEPER_SWELL_FALL_DAMAGE_CLAMP_OFFSET;
        if self.swell > max_preloaded_swell {
            self.swell = max_preloaded_swell;
        }
    }

    pub fn tick(&mut self, active_effect_count: usize) -> CreeperTickResult {
        let mut result = CreeperTickResult {
            primed_sound: false,
            prime_fuse_game_event: false,
            explosion_radius: None,
            lingering_cloud: None,
        };
        if !self.alive {
            return result;
        }

        self.old_swell = self.swell;
        if self.ignited {
            self.set_swell_dir(1);
        }

        if self.swell_dir > 0 && self.swell == 0 {
            result.primed_sound = true;
            result.prime_fuse_game_event = true;
        }

        self.swell += self.swell_dir;
        if self.swell < 0 {
            self.swell = 0;
        }
        if self.swell >= self.max_swell {
            self.swell = self.max_swell;
            result.explosion_radius = Some(self.effective_explosion_radius());
            if active_effect_count > 0 {
                result.lingering_cloud = Some(creeper_lingering_cloud());
            }
            self.alive = false;
        }

        result
    }

    pub fn effective_explosion_radius(self) -> f32 {
        let multiplier = if self.powered {
            CREEPER_POWERED_EXPLOSION_MULTIPLIER
        } else {
            CREEPER_NORMAL_EXPLOSION_MULTIPLIER
        };
        self.explosion_radius as f32 * multiplier
    }

    pub fn swelling(self, partial_tick: f32) -> f32 {
        let interpolated =
            self.old_swell as f32 + (self.swell - self.old_swell) as f32 * partial_tick;
        interpolated / (self.max_swell - CREEPER_SWELL_RENDER_DENOMINATOR_OFFSET) as f32
    }

    pub fn killed_entity_drops_charged_creeper_loot(&mut self, should_drop_loot: bool) -> bool {
        if should_drop_loot && self.powered && !self.dropped_skulls {
            self.dropped_skulls = true;
            true
        } else {
            false
        }
    }

    pub fn can_target(self, target_is_goat: bool) -> bool {
        !target_is_goat
    }
}

pub fn creeper_igniter_use(is_igniter: bool, is_damageable_item: bool) -> CreeperIgniterUse {
    if !is_igniter {
        CreeperIgniterUse::NotIgniter
    } else if is_damageable_item {
        CreeperIgniterUse::DamageOne
    } else {
        CreeperIgniterUse::ConsumeOne
    }
}

pub fn creeper_lingering_cloud() -> CreeperLingeringCloud {
    CreeperLingeringCloud {
        radius: CREEPER_LINGERING_CLOUD_RADIUS,
        radius_on_use: CREEPER_LINGERING_CLOUD_RADIUS_ON_USE,
        wait_time: CREEPER_LINGERING_CLOUD_WAIT_TIME,
        duration: CREEPER_LINGERING_CLOUD_DURATION,
        potion_duration_scale: CREEPER_LINGERING_CLOUD_POTION_DURATION_SCALE,
        radius_per_tick: -CREEPER_LINGERING_CLOUD_RADIUS / CREEPER_LINGERING_CLOUD_DURATION as f32,
    }
}

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

pub const SLIME_MIN_SIZE: i32 = 1;
pub const SLIME_MAX_SIZE: i32 = 127;
pub const SLIME_MAX_NATURAL_SIZE: i32 = 4;
pub const SLIME_DEFAULT_SERIALIZED_SIZE: i32 = 0;
pub const SLIME_DEFAULT_WAS_ON_GROUND: bool = false;
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

    pub fn move_control_step(
        self,
        operation_move_to: bool,
        on_ground: bool,
        speed_modifier: f32,
        movement_speed_attribute: f32,
        jump_delay: i32,
        random_0_to_19: i32,
        aggressive: bool,
    ) -> SlimeMoveControlStep {
        if !operation_move_to {
            return SlimeMoveControlStep {
                speed: 0.0,
                jump: false,
                play_jump_sound: false,
                next_jump_delay: jump_delay,
                zero_strafe: false,
            };
        }

        let speed = speed_modifier * movement_speed_attribute;
        if !on_ground {
            return SlimeMoveControlStep {
                speed,
                jump: false,
                play_jump_sound: false,
                next_jump_delay: jump_delay,
                zero_strafe: false,
            };
        }

        if jump_delay <= 0 {
            let next_jump_delay = self.jump_delay(random_0_to_19, aggressive);
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
                next_jump_delay: jump_delay - 1,
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

pub fn clamp_slime_size(size: i32) -> i32 {
    size.clamp(SLIME_MIN_SIZE, SLIME_MAX_SIZE)
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

pub fn magma_cube_spawn_allowed(peaceful: bool) -> bool {
    !peaceful
}

pub fn magma_cube_is_on_fire() -> bool {
    MAGMA_CUBE_IS_ON_FIRE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhantomAttackPhase {
    Circle,
    Swoop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomState {
    pub size: i32,
    pub anchor_point: Option<PhantomBlockPos>,
    pub attack_phase: PhantomAttackPhase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhantomAttributes {
    pub attack_damage: f32,
    pub xp_reward: i32,
    pub dimensions_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhantomAttackStrategy {
    pub attack_phase: PhantomAttackPhase,
    pub next_sweep_tick: i32,
    pub anchor_point: Option<PhantomBlockPos>,
    pub played_swoop_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhantomSwoopContinuation {
    Continue,
    StopNoTarget,
    StopDeadTarget,
    StopCreativeOrSpectatorPlayer,
    StopNoLongerSwooping,
    StopScaredOfCat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhantomSwoopTick {
    Flying,
    HitTarget { level_event: Option<i32> },
    CancelledToCircle,
}

pub const PHANTOM_MIN_SIZE: i32 = 0;
pub const PHANTOM_MAX_SIZE: i32 = 64;
pub const PHANTOM_DEFAULT_SIZE: i32 = 0;
pub const PHANTOM_XP_REWARD: i32 = 5;
pub const PHANTOM_BASE_ATTACK_DAMAGE: f32 = 6.0;
pub const PHANTOM_DIMENSIONS_SCALE_PER_SIZE: f32 = 0.15;
pub const PHANTOM_FLAP_DEGREES_PER_TICK: f32 = 7.448451;
pub const PHANTOM_TICKS_PER_FLAP: i32 = 25;
pub const PHANTOM_UNIQUE_FLAP_TICK_OFFSET_MULTIPLIER: i32 = 3;
pub const PHANTOM_FINALIZE_ANCHOR_ABOVE: i32 = 5;
pub const PHANTOM_TARGET_SCAN_INITIAL_TICKS: i32 = 20;
pub const PHANTOM_TARGET_SCAN_RESET_TICKS: i32 = 60;
pub const PHANTOM_TARGET_RANGE: f32 = 64.0;
pub const PHANTOM_TARGET_BOX_INFLATE_XZ: f32 = 16.0;
pub const PHANTOM_TARGET_BOX_INFLATE_Y: f32 = 64.0;
pub const PHANTOM_ATTACK_STRATEGY_START_SWEEP_TICKS: i32 = 10;
pub const PHANTOM_SWEEP_DELAY_BASE_SECONDS: i32 = 8;
pub const PHANTOM_SWEEP_DELAY_RANDOM_SECONDS_BOUND: i32 = 4;
pub const PHANTOM_ANCHOR_ABOVE_TARGET_MIN: i32 = 20;
pub const PHANTOM_ANCHOR_ABOVE_TARGET_RANDOM_BOUND: i32 = 20;
pub const PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_MIN: i32 = 10;
pub const PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_RANDOM_BOUND: i32 = 20;
pub const PHANTOM_CIRCLE_DISTANCE_MIN: f32 = 5.0;
pub const PHANTOM_CIRCLE_DISTANCE_RANDOM_SPAN: f32 = 10.0;
pub const PHANTOM_CIRCLE_DISTANCE_MAX: f32 = 15.0;
pub const PHANTOM_CIRCLE_HEIGHT_BASE: f32 = -4.0;
pub const PHANTOM_CIRCLE_HEIGHT_RANDOM_SPAN: f32 = 9.0;
pub const PHANTOM_CIRCLE_ANGLE_STEP_DEGREES: f32 = 15.0;
pub const PHANTOM_TOUCHING_TARGET_DISTANCE_SQUARED: f64 = 4.0;
pub const PHANTOM_SWEEP_CAT_SEARCH_TICK_DELAY: i32 = 20;
pub const PHANTOM_CAT_AVOID_INFLATE: f32 = 16.0;
pub const PHANTOM_SWEEP_HIT_INFLATE: f32 = 0.2;
pub const PHANTOM_SWEEP_HIT_LEVEL_EVENT: i32 = 1039;
pub const PHANTOM_LOOT_ITEM: &str = "minecraft:phantom_membrane";
pub const PHANTOM_USES_NEAREST_PLAYERS_MEMORY: bool = false;
pub const PHANTOM_BURNS_IN_DAYLIGHT: bool = false;

impl PhantomState {
    pub fn new() -> Self {
        Self {
            size: PHANTOM_DEFAULT_SIZE,
            anchor_point: None,
            attack_phase: PhantomAttackPhase::Circle,
        }
    }

    pub fn set_size(&mut self, size: i32) {
        self.size = clamp_phantom_size(size);
    }

    pub fn read_save_data(size: Option<i32>, anchor_point: Option<PhantomBlockPos>) -> Self {
        let mut state = Self::new();
        state.set_size(size.unwrap_or(PHANTOM_DEFAULT_SIZE));
        state.anchor_point = anchor_point;
        state
    }

    pub fn finalize_spawn(block_position: PhantomBlockPos) -> Self {
        Self {
            size: PHANTOM_DEFAULT_SIZE,
            anchor_point: Some(block_position.above(PHANTOM_FINALIZE_ANCHOR_ABOVE)),
            attack_phase: PhantomAttackPhase::Circle,
        }
    }

    pub fn attributes(self) -> PhantomAttributes {
        PhantomAttributes {
            attack_damage: PHANTOM_BASE_ATTACK_DAMAGE + self.size as f32,
            xp_reward: PHANTOM_XP_REWARD,
            dimensions_scale: 1.0 + PHANTOM_DIMENSIONS_SCALE_PER_SIZE * self.size as f32,
        }
    }

    pub fn is_flapping(self, entity_id: i32, tick_count: i32) -> bool {
        (phantom_unique_flap_tick_offset(entity_id) + tick_count).rem_euclid(PHANTOM_TICKS_PER_FLAP)
            == 0
    }
}

impl Default for PhantomState {
    fn default() -> Self {
        Self::new()
    }
}

impl PhantomBlockPos {
    pub fn above(self, amount: i32) -> Self {
        Self {
            y: self.y + amount,
            ..self
        }
    }
}

pub fn clamp_phantom_size(size: i32) -> i32 {
    size.clamp(PHANTOM_MIN_SIZE, PHANTOM_MAX_SIZE)
}

pub fn phantom_unique_flap_tick_offset(entity_id: i32) -> i32 {
    entity_id * PHANTOM_UNIQUE_FLAP_TICK_OFFSET_MULTIPLIER
}

pub fn phantom_target_scan_tick(next_scan_tick: i32) -> Option<i32> {
    (next_scan_tick > 0).then_some(next_scan_tick - 1)
}

pub fn phantom_target_scan_reset_ticks() -> i32 {
    PHANTOM_TARGET_SCAN_RESET_TICKS
}

pub fn phantom_can_continue_swoop(
    has_target: bool,
    target_alive: bool,
    target_player_creative: bool,
    target_player_spectator: bool,
    attack_phase: PhantomAttackPhase,
    cats_nearby: bool,
) -> PhantomSwoopContinuation {
    if !has_target {
        PhantomSwoopContinuation::StopNoTarget
    } else if !target_alive {
        PhantomSwoopContinuation::StopDeadTarget
    } else if target_player_creative || target_player_spectator {
        PhantomSwoopContinuation::StopCreativeOrSpectatorPlayer
    } else if attack_phase != PhantomAttackPhase::Swoop {
        PhantomSwoopContinuation::StopNoLongerSwooping
    } else if cats_nearby {
        PhantomSwoopContinuation::StopScaredOfCat
    } else {
        PhantomSwoopContinuation::Continue
    }
}

pub fn phantom_attack_strategy_start(
    target_pos: PhantomBlockPos,
    target_anchor_random_0_to_19: i32,
    sea_level: i32,
) -> PhantomAttackStrategy {
    PhantomAttackStrategy {
        attack_phase: PhantomAttackPhase::Circle,
        next_sweep_tick: PHANTOM_ATTACK_STRATEGY_START_SWEEP_TICKS,
        anchor_point: Some(phantom_anchor_above_target(
            target_pos,
            target_anchor_random_0_to_19,
            sea_level,
        )),
        played_swoop_sound: false,
    }
}

pub fn phantom_attack_strategy_tick(
    attack_phase: PhantomAttackPhase,
    next_sweep_tick: i32,
    target_pos: PhantomBlockPos,
    target_anchor_random_0_to_19: i32,
    next_sweep_random_0_to_3: i32,
    sea_level: i32,
) -> PhantomAttackStrategy {
    if attack_phase != PhantomAttackPhase::Circle {
        return PhantomAttackStrategy {
            attack_phase,
            next_sweep_tick,
            anchor_point: None,
            played_swoop_sound: false,
        };
    }
    let decremented = next_sweep_tick - 1;
    if decremented > 0 {
        return PhantomAttackStrategy {
            attack_phase,
            next_sweep_tick: decremented,
            anchor_point: None,
            played_swoop_sound: false,
        };
    }
    PhantomAttackStrategy {
        attack_phase: PhantomAttackPhase::Swoop,
        next_sweep_tick: (PHANTOM_SWEEP_DELAY_BASE_SECONDS
            + next_sweep_random_0_to_3.rem_euclid(PHANTOM_SWEEP_DELAY_RANDOM_SECONDS_BOUND))
            * 20,
        anchor_point: Some(phantom_anchor_above_target(
            target_pos,
            target_anchor_random_0_to_19,
            sea_level,
        )),
        played_swoop_sound: true,
    }
}

pub fn phantom_anchor_above_target(
    target_pos: PhantomBlockPos,
    random_0_to_19: i32,
    sea_level: i32,
) -> PhantomBlockPos {
    let mut anchor = target_pos.above(
        PHANTOM_ANCHOR_ABOVE_TARGET_MIN
            + random_0_to_19.rem_euclid(PHANTOM_ANCHOR_ABOVE_TARGET_RANDOM_BOUND),
    );
    if anchor.y < sea_level {
        anchor.y = sea_level + 1;
    }
    anchor
}

pub fn phantom_stop_anchor_after_heightmap(
    anchor_x: i32,
    heightmap_y: i32,
    anchor_z: i32,
    random_0_to_19: i32,
) -> PhantomBlockPos {
    PhantomBlockPos {
        x: anchor_x,
        y: heightmap_y
            + PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_MIN
            + random_0_to_19.rem_euclid(PHANTOM_STOP_ANCHOR_ABOVE_HEIGHTMAP_RANDOM_BOUND),
        z: anchor_z,
    }
}

pub fn phantom_swoop_tick(
    target_intersects_inflated_box: bool,
    horizontal_collision: bool,
    hurt_time_positive: bool,
    silent: bool,
) -> PhantomSwoopTick {
    if target_intersects_inflated_box {
        PhantomSwoopTick::HitTarget {
            level_event: (!silent).then_some(PHANTOM_SWEEP_HIT_LEVEL_EVENT),
        }
    } else if horizontal_collision || hurt_time_positive {
        PhantomSwoopTick::CancelledToCircle
    } else {
        PhantomSwoopTick::Flying
    }
}

pub fn phantom_burns_in_daylight() -> bool {
    PHANTOM_BURNS_IN_DAYLIGHT
}

pub fn phantom_uses_nearest_players_memory() -> bool {
    PHANTOM_USES_NEAREST_PLAYERS_MEMORY
}

pub fn phantom_membrane_loot_roll(
    killed_by_player: bool,
    base_roll_0_or_1: i32,
    looting_roll_0_to_level: i32,
) -> i32 {
    if killed_by_player {
        base_roll_0_or_1.clamp(0, 1) + looting_roll_0_to_level.max(0)
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VexAttributes {
    pub max_health: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexTickOutcome {
    pub no_physics_during_tick: bool,
    pub no_gravity_after_tick: bool,
    pub limited_life_ticks: i32,
    pub starve_damage: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VexSummonPlan {
    pub can_summon: bool,
    pub count: i32,
    pub limited_life_ticks: i32,
    pub y_offset: i32,
    pub horizontal_random_bound: i32,
    pub copy_evoker_team: bool,
    pub game_event: &'static str,
}

pub const VEX_MAX_HEALTH: f32 = 14.0;
pub const VEX_ATTACK_DAMAGE: f32 = 4.0;
pub const VEX_XP_REWARD: i32 = 3;
pub const VEX_FLAP_DEGREES_PER_TICK: f32 = 45.836624;
pub const VEX_TICKS_PER_FLAP: i32 = 4;
pub const VEX_CHARGING_FLAG: u8 = 1;
pub const VEX_DEFAULT_MAINHAND_ITEM: &str = "minecraft:iron_sword";
pub const VEX_MAINHAND_DROP_CHANCE: f32 = 0.0;
pub const VEX_LIGHT_LEVEL_MAGIC_VALUE: f32 = 1.0;
pub const VEX_CHARGE_RANDOM_BOUND: i32 = 7;
pub const VEX_CHARGE_MIN_DISTANCE_SQR: f32 = 4.0;
pub const VEX_RETARGET_DISTANCE_SQR: f32 = 9.0;
pub const VEX_RANDOM_MOVE_ATTEMPTS: i32 = 3;
pub const VEX_RANDOM_MOVE_XZ_RANDOM_BOUND: i32 = 15;
pub const VEX_RANDOM_MOVE_Y_RANDOM_BOUND: i32 = 11;
pub const VEX_RANDOM_MOVE_XZ_OFFSET: i32 = 7;
pub const VEX_RANDOM_MOVE_Y_OFFSET: i32 = 5;
pub const VEX_RANDOM_MOVE_SPEED: f32 = 0.25;
pub const VEX_MOVE_ACCELERATION: f32 = 0.05;
pub const VEX_MOVE_CLOSE_DAMPING: f32 = 0.5;
pub const VEX_OWNER_TARGET_RANGE: f32 = 16.0;
pub const EVOKER_VEX_SUMMON_COUNT: i32 = 3;
pub const EVOKER_VEX_SUMMON_CASTING_TIME: i32 = 100;
pub const EVOKER_VEX_SUMMON_INTERVAL: i32 = 340;
pub const EVOKER_VEX_LIMITED_LIFE_MIN_TICKS: i32 = 20 * 30;
pub const EVOKER_VEX_LIMITED_LIFE_RANDOM_BOUND_SECONDS: i32 = 90;

pub fn vex_attributes() -> VexAttributes {
    VexAttributes {
        max_health: VEX_MAX_HEALTH,
        attack_damage: VEX_ATTACK_DAMAGE,
        xp_reward: VEX_XP_REWARD,
    }
}

pub fn vex_is_flapping(tick_count: i32) -> bool {
    tick_count.rem_euclid(VEX_TICKS_PER_FLAP) == 0
}

pub fn vex_set_charging(flags: u8, charging: bool) -> u8 {
    if charging {
        flags | VEX_CHARGING_FLAG
    } else {
        flags & !VEX_CHARGING_FLAG
    }
}

pub fn vex_is_charging(flags: u8) -> bool {
    flags & VEX_CHARGING_FLAG != 0
}

pub fn vex_tick(has_limited_life: bool, limited_life_ticks: i32) -> VexTickOutcome {
    if has_limited_life {
        let next_life = limited_life_ticks - 1;
        if next_life <= 0 {
            return VexTickOutcome {
                no_physics_during_tick: true,
                no_gravity_after_tick: true,
                limited_life_ticks: 20,
                starve_damage: true,
            };
        }
        VexTickOutcome {
            no_physics_during_tick: true,
            no_gravity_after_tick: true,
            limited_life_ticks: next_life,
            starve_damage: false,
        }
    } else {
        VexTickOutcome {
            no_physics_during_tick: true,
            no_gravity_after_tick: true,
            limited_life_ticks,
            starve_damage: false,
        }
    }
}

pub fn vex_charge_attack_can_use(
    target_present: bool,
    target_alive: bool,
    move_control_has_wanted: bool,
    random_0_to_6: i32,
    distance_sqr: f32,
) -> bool {
    target_present
        && target_alive
        && !move_control_has_wanted
        && random_0_to_6.rem_euclid(VEX_CHARGE_RANDOM_BOUND) == 0
        && distance_sqr > VEX_CHARGE_MIN_DISTANCE_SQR
}

pub fn vex_charge_attack_can_continue(
    move_control_has_wanted: bool,
    charging: bool,
    target_present: bool,
    target_alive: bool,
) -> bool {
    move_control_has_wanted && charging && target_present && target_alive
}

pub fn vex_charge_attack_tick(intersects_target: bool, distance_sqr: f32) -> (bool, bool) {
    if intersects_target {
        (true, false)
    } else if distance_sqr < VEX_RETARGET_DISTANCE_SQR {
        (false, true)
    } else {
        (false, false)
    }
}

pub fn vex_copy_owner_target_can_use(
    owner_present: bool,
    owner_target_present: bool,
    can_attack_owner_target: bool,
) -> bool {
    owner_present && owner_target_present && can_attack_owner_target
}

pub fn vex_random_move_can_use(move_control_has_wanted: bool, random_0_to_6: i32) -> bool {
    !move_control_has_wanted && random_0_to_6.rem_euclid(VEX_CHARGE_RANDOM_BOUND) == 0
}

pub fn evoker_vex_summon_can_use(
    super_can_use: bool,
    nearby_vex_count: i32,
    random_1_to_8: i32,
) -> bool {
    super_can_use && random_1_to_8 > nearby_vex_count
}

pub fn evoker_vex_limited_life_ticks(random_0_to_89: i32) -> i32 {
    20 * (30 + random_0_to_89.rem_euclid(EVOKER_VEX_LIMITED_LIFE_RANDOM_BOUND_SECONDS))
}

pub fn evoker_vex_summon_plan(
    super_can_use: bool,
    nearby_vex_count: i32,
    random_1_to_8: i32,
    random_life_0_to_89: i32,
    evoker_has_team: bool,
) -> VexSummonPlan {
    VexSummonPlan {
        can_summon: evoker_vex_summon_can_use(super_can_use, nearby_vex_count, random_1_to_8),
        count: EVOKER_VEX_SUMMON_COUNT,
        limited_life_ticks: evoker_vex_limited_life_ticks(random_life_0_to_89),
        y_offset: 1,
        horizontal_random_bound: 5,
        copy_evoker_team: evoker_has_team,
        game_event: "minecraft:entity_place",
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoglinAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub knockback_resistance: f32,
    pub attack_knockback: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinBaseThrowVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub hurt_marked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinConversionTick {
    pub time_in_overworld: i32,
    pub convert_to_zoglin: bool,
    pub nausea_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub knockback_resistance: f32,
    pub attack_knockback: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoglinEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub passenger_attachment_y: f32,
    pub client_tracking_range: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoglinAiAction {
    SetAttackTarget,
    SetAvoidTarget,
    BroadcastAttackTarget,
    BroadcastRetreat,
    None,
}

pub const ZOGLIN_MAX_HEALTH: f32 = 40.0;
pub const ZOGLIN_MOVEMENT_SPEED: f32 = 0.3;
pub const ZOGLIN_KNOCKBACK_RESISTANCE: f32 = 0.6;
pub const ZOGLIN_ATTACK_KNOCKBACK: f32 = 1.0;
pub const ZOGLIN_ATTACK_DAMAGE: f32 = 6.0;
pub const ZOGLIN_BABY_ATTACK_DAMAGE: f32 = 0.5;
pub const ZOGLIN_XP_REWARD: i32 = 5;
pub const ZOGLIN_BABY_RANDOM_CHANCE: f32 = 0.2;
pub const ZOGLIN_ATTACK_INTERVAL_TICKS: i32 = 40;
pub const ZOGLIN_BABY_ATTACK_INTERVAL_TICKS: i32 = 15;
pub const ZOGLIN_ATTACK_TARGET_MEMORY_TICKS: i64 = 200;
pub const ZOGLIN_ATTACK_ANIMATION_DURATION_TICKS: i32 = 10;
pub const ZOGLIN_IDLE_SPEED_MULTIPLIER: f32 = 0.4;
pub const ZOGLIN_FIGHTING_MOVEMENT_SPEED: f32 = 0.3;
pub const ZOGLIN_LOOK_TARGET_RANGE: f32 = 8.0;
pub const ZOGLIN_LOOK_INTERVAL_MIN_TICKS: i32 = 30;
pub const ZOGLIN_LOOK_INTERVAL_MAX_TICKS: i32 = 60;
pub const ZOGLIN_DO_NOTHING_MIN_TICKS: i32 = 30;
pub const ZOGLIN_DO_NOTHING_MAX_TICKS: i32 = 60;
pub const ZOGLIN_HURT_RETARGET_DISTANCE_MARGIN: f32 = 4.0;
pub const ZOGLIN_ATTACK_EVENT_ID: u8 = 4;
pub const ZOGLIN_STEP_SOUND_VOLUME: f32 = 0.15;
pub const ZOGLIN_STEP_SOUND_PITCH: f32 = 1.0;
pub const HOGLIN_CONVERSION_TIME_TICKS: i32 = 300;
pub const HOGLIN_CONVERSION_NAUSEA_TICKS: i32 = 200;
pub const HOGLIN_MAX_HEALTH: f32 = 40.0;
pub const HOGLIN_MOVEMENT_SPEED: f32 = 0.3;
pub const HOGLIN_KNOCKBACK_RESISTANCE: f32 = 0.6;
pub const HOGLIN_ATTACK_KNOCKBACK: f32 = 1.0;
pub const HOGLIN_ATTACK_DAMAGE: f32 = 6.0;
pub const HOGLIN_BABY_ATTACK_DAMAGE: f32 = 0.5;
pub const HOGLIN_XP_REWARD: i32 = 5;
pub const HOGLIN_BABY_XP_REWARD: i32 = 3;
pub const HOGLIN_WIDTH: f32 = 1.3964844;
pub const HOGLIN_HEIGHT: f32 = 1.4;
pub const HOGLIN_PASSENGER_ATTACHMENT_Y: f32 = 1.49375;
pub const HOGLIN_CLIENT_TRACKING_RANGE: i32 = 8;
pub const HOGLIN_BABY_RANDOM_CHANCE: f32 = 0.2;
pub const HOGLIN_ATTACK_ANIMATION_DURATION_TICKS: i32 = 10;
pub const HOGLIN_ATTACK_EVENT_ID: u8 = 4;
pub const HOGLIN_ATTACK_TARGET_MEMORY_TICKS: i64 = 200;
pub const HOGLIN_ATTACK_INTERVAL_TICKS: i32 = 40;
pub const HOGLIN_BABY_ATTACK_INTERVAL_TICKS: i32 = 15;
pub const HOGLIN_REPELLENT_DETECTION_HORIZONTAL: i32 = 8;
pub const HOGLIN_REPELLENT_DETECTION_VERTICAL: i32 = 4;
pub const HOGLIN_REPELLENT_PACIFY_TIME: i32 = 200;
pub const HOGLIN_RETREAT_MIN_SECONDS: i32 = 5;
pub const HOGLIN_RETREAT_MAX_SECONDS: i32 = 20;
pub const HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_IDLING: i32 = 8;
pub const HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_RETREATING: i32 = 15;
pub const HOGLIN_AVOID_REPELLENT_SPEED: f32 = 1.0;
pub const HOGLIN_RETREAT_SPEED: f32 = 1.3;
pub const HOGLIN_BREEDING_SPEED: f32 = 0.6;
pub const HOGLIN_IDLE_SPEED: f32 = 0.4;
pub const HOGLIN_BABY_FOLLOW_ADULT_SPEED: f32 = 0.6;
pub const HOGLIN_ADULT_FOLLOW_RANGE_MIN: i32 = 5;
pub const HOGLIN_ADULT_FOLLOW_RANGE_MAX: i32 = 16;
pub const HOGLIN_LOOK_TARGET_RANGE: f32 = 8.0;
pub const HOGLIN_LOOK_INTERVAL_MIN_TICKS: i32 = 30;
pub const HOGLIN_LOOK_INTERVAL_MAX_TICKS: i32 = 60;
pub const HOGLIN_DO_NOTHING_MIN_TICKS: i32 = 30;
pub const HOGLIN_DO_NOTHING_MAX_TICKS: i32 = 60;
pub const HOGLIN_STEP_SOUND_VOLUME: f32 = 0.15;
pub const HOGLIN_STEP_SOUND_PITCH: f32 = 1.0;

pub fn zoglin_attributes() -> ZoglinAttributes {
    ZoglinAttributes {
        max_health: ZOGLIN_MAX_HEALTH,
        movement_speed: ZOGLIN_MOVEMENT_SPEED,
        knockback_resistance: ZOGLIN_KNOCKBACK_RESISTANCE,
        attack_knockback: ZOGLIN_ATTACK_KNOCKBACK,
        attack_damage: ZOGLIN_ATTACK_DAMAGE,
        xp_reward: ZOGLIN_XP_REWARD,
    }
}

pub fn zoglin_attack_damage(baby: bool) -> f32 {
    if baby {
        ZOGLIN_BABY_ATTACK_DAMAGE
    } else {
        ZOGLIN_ATTACK_DAMAGE
    }
}

pub fn zoglin_attack_interval_ticks(baby: bool) -> i32 {
    if baby {
        ZOGLIN_BABY_ATTACK_INTERVAL_TICKS
    } else {
        ZOGLIN_ATTACK_INTERVAL_TICKS
    }
}

pub fn zoglin_finalize_spawn_is_baby(random_float_0_to_1: f32) -> bool {
    random_float_0_to_1 < ZOGLIN_BABY_RANDOM_CHANCE
}

pub fn zoglin_valid_attack_target(
    target_entity_type: &'static str,
    sensor_attackable: bool,
) -> bool {
    sensor_attackable
        && target_entity_type != "minecraft:zoglin"
        && target_entity_type != "minecraft:creeper"
}

pub fn zoglin_ambient_sound(has_attack_target: bool, client_side: bool) -> Option<&'static str> {
    if client_side {
        None
    } else if has_attack_target {
        Some("minecraft:entity.zoglin.angry")
    } else {
        Some("minecraft:entity.zoglin.ambient")
    }
}

pub fn zoglin_on_hurt_should_retarget(
    was_hurt: bool,
    attacker_is_living: bool,
    can_attack_attacker: bool,
    other_target_much_further_than_current: bool,
) -> bool {
    was_hurt && attacker_is_living && can_attack_attacker && !other_target_much_further_than_current
}

pub fn zoglin_event_attack_animation_ticks(event_id: u8) -> Option<i32> {
    (event_id == ZOGLIN_ATTACK_EVENT_ID).then_some(ZOGLIN_ATTACK_ANIMATION_DURATION_TICKS)
}

pub fn zoglin_next_attack_animation_ticks(current_ticks: i32) -> i32 {
    (current_ticks - 1).max(0)
}

pub fn zoglin_blocked_by_item_throws_target(baby: bool) -> bool {
    !baby
}

pub fn zoglin_save_is_baby_key() -> &'static str {
    "IsBaby"
}

pub fn zoglin_is_immune_to_regular_zombification() -> bool {
    true
}

pub fn hoglin_is_converting(
    immune_to_zombification: bool,
    no_ai: bool,
    piglins_zombify_environment: bool,
) -> bool {
    !immune_to_zombification && !no_ai && piglins_zombify_environment
}

pub fn hoglin_conversion_tick(
    time_in_overworld: i32,
    immune_to_zombification: bool,
    no_ai: bool,
    piglins_zombify_environment: bool,
) -> HoglinConversionTick {
    if hoglin_is_converting(immune_to_zombification, no_ai, piglins_zombify_environment) {
        let next = time_in_overworld + 1;
        HoglinConversionTick {
            time_in_overworld: next,
            convert_to_zoglin: next > HOGLIN_CONVERSION_TIME_TICKS,
            nausea_ticks: if next > HOGLIN_CONVERSION_TIME_TICKS {
                HOGLIN_CONVERSION_NAUSEA_TICKS
            } else {
                0
            },
        }
    } else {
        HoglinConversionTick {
            time_in_overworld: 0,
            convert_to_zoglin: false,
            nausea_ticks: 0,
        }
    }
}

pub fn hoglin_base_attack_damage(
    body_is_baby: bool,
    attack_damage: f32,
    random_0_to_attack_damage_minus_1: i32,
) -> f32 {
    let attack_damage_int = attack_damage as i32;
    if !body_is_baby && attack_damage_int > 0 {
        attack_damage / 2.0 + random_0_to_attack_damage_minus_1.rem_euclid(attack_damage_int) as f32
    } else {
        attack_damage
    }
}

pub fn hoglin_base_throw_target(
    body_x: f64,
    body_z: f64,
    target_x: f64,
    target_z: f64,
    attack_knockback: f64,
    target_knockback_resistance: f64,
    random_y_rot_minus_10_to_10: f64,
    random_float_0_to_1_for_horizontal: f64,
    random_float_0_to_1_for_vertical: f64,
) -> Option<HoglinBaseThrowVector> {
    let effective_knockback_power = attack_knockback - target_knockback_resistance;
    if effective_knockback_power <= 0.0 {
        return None;
    }

    let dx = target_x - body_x;
    let dz = target_z - body_z;
    let length = (dx * dx + dz * dz).sqrt();
    if length == 0.0 {
        return Some(HoglinBaseThrowVector {
            x: 0.0,
            y: effective_knockback_power * random_float_0_to_1_for_vertical * 0.5,
            z: 0.0,
            hurt_marked: true,
        });
    }

    let horizontal_scale =
        effective_knockback_power * (random_float_0_to_1_for_horizontal * 0.5 + 0.2);
    let x = dx / length * horizontal_scale;
    let z = dz / length * horizontal_scale;
    let cos = random_y_rot_minus_10_to_10.cos();
    let sin = random_y_rot_minus_10_to_10.sin();
    Some(HoglinBaseThrowVector {
        x: x * cos + z * sin,
        y: effective_knockback_power * random_float_0_to_1_for_vertical * 0.5,
        z: z * cos - x * sin,
        hurt_marked: true,
    })
}

pub fn hoglin_attributes() -> HoglinAttributes {
    HoglinAttributes {
        max_health: HOGLIN_MAX_HEALTH,
        movement_speed: HOGLIN_MOVEMENT_SPEED,
        knockback_resistance: HOGLIN_KNOCKBACK_RESISTANCE,
        attack_knockback: HOGLIN_ATTACK_KNOCKBACK,
        attack_damage: HOGLIN_ATTACK_DAMAGE,
        xp_reward: HOGLIN_XP_REWARD,
    }
}

pub fn hoglin_entity_type_surface() -> HoglinEntityTypeSurface {
    HoglinEntityTypeSurface {
        width: HOGLIN_WIDTH,
        height: HOGLIN_HEIGHT,
        passenger_attachment_y: HOGLIN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: HOGLIN_CLIENT_TRACKING_RANGE,
    }
}

pub fn hoglin_attack_damage(baby: bool) -> f32 {
    if baby {
        HOGLIN_BABY_ATTACK_DAMAGE
    } else {
        HOGLIN_ATTACK_DAMAGE
    }
}

pub fn hoglin_xp_reward(baby: bool) -> i32 {
    if baby {
        HOGLIN_BABY_XP_REWARD
    } else {
        HOGLIN_XP_REWARD
    }
}

pub fn hoglin_attack_interval_ticks(baby: bool) -> i32 {
    if baby {
        HOGLIN_BABY_ATTACK_INTERVAL_TICKS
    } else {
        HOGLIN_ATTACK_INTERVAL_TICKS
    }
}

pub fn hoglin_finalize_spawn_is_baby(random_float_0_to_1: f32) -> bool {
    random_float_0_to_1 < HOGLIN_BABY_RANDOM_CHANCE
}

pub fn hoglin_spawn_allowed(block_below: &'static str) -> bool {
    block_below != "minecraft:nether_wart_block"
}

pub fn hoglin_walk_target_value(near_repellent: bool, block_below: &'static str) -> f32 {
    if near_repellent {
        -1.0
    } else if block_below == "minecraft:crimson_nylium" {
        10.0
    } else {
        0.0
    }
}

pub fn hoglin_can_be_hunted(adult: bool, cannot_be_hunted: bool) -> bool {
    adult && !cannot_be_hunted
}

pub fn hoglin_can_fall_in_love(pacified: bool, super_can_fall_in_love: bool) -> bool {
    !pacified && super_can_fall_in_love
}

pub fn hoglin_piglins_outnumber_hoglins(
    baby: bool,
    visible_adult_piglins: i32,
    visible_adult_hoglins: i32,
) -> bool {
    !baby && visible_adult_piglins > visible_adult_hoglins + 1
}

pub fn hoglin_on_hit_target_action(
    baby: bool,
    target_entity_type: &'static str,
    piglins_outnumber_hoglins: bool,
) -> HoglinAiAction {
    if baby {
        HoglinAiAction::None
    } else if target_entity_type == "minecraft:piglin" && piglins_outnumber_hoglins {
        HoglinAiAction::BroadcastRetreat
    } else {
        HoglinAiAction::BroadcastAttackTarget
    }
}

pub fn hoglin_was_hurt_action(
    baby: bool,
    attacker_entity_type: &'static str,
    active_activity_avoid: bool,
    other_target_much_further: bool,
    sensor_attackable: bool,
) -> HoglinAiAction {
    if baby {
        HoglinAiAction::SetAvoidTarget
    } else if active_activity_avoid && attacker_entity_type == "minecraft:piglin" {
        HoglinAiAction::None
    } else if attacker_entity_type == "minecraft:hoglin"
        || other_target_much_further
        || !sensor_attackable
    {
        HoglinAiAction::None
    } else {
        HoglinAiAction::SetAttackTarget
    }
}

pub fn hoglin_find_nearest_valid_attack_target(
    pacified: bool,
    breeding: bool,
    nearest_visible_attackable_player: bool,
) -> bool {
    !pacified && !breeding && nearest_visible_attackable_player
}

pub fn hoglin_activity_sound(
    activity: &'static str,
    converting: bool,
    near_repellent: bool,
    client_side: bool,
) -> Option<&'static str> {
    if client_side {
        None
    } else if activity == "avoid" || converting {
        Some("minecraft:entity.hoglin.retreat")
    } else if activity == "fight" {
        Some("minecraft:entity.hoglin.angry")
    } else if near_repellent {
        Some("minecraft:entity.hoglin.retreat")
    } else {
        Some("minecraft:entity.hoglin.ambient")
    }
}

pub fn hoglin_event_attack_animation_ticks(event_id: u8) -> Option<i32> {
    (event_id == HOGLIN_ATTACK_EVENT_ID).then_some(HOGLIN_ATTACK_ANIMATION_DURATION_TICKS)
}

pub fn hoglin_next_attack_animation_ticks(current_ticks: i32) -> i32 {
    (current_ticks - 1).max(0)
}

pub fn hoglin_blocked_by_item_throws_target(baby: bool) -> bool {
    !baby
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastAttributes {
    pub max_health: f32,
    pub follow_range: f32,
    pub camera_distance: f32,
    pub flying_speed: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub fire_immune: bool,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastShootTick {
    pub charge_time: i32,
    pub charging: bool,
    pub warn_level_event: Option<i32>,
    pub shoot_level_event: Option<i32>,
    pub fireball: Option<GhastFireballPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GhastFireballPlan {
    pub spawn_offset: f64,
    pub y_offset_from_ghast_mid: f64,
    pub explosion_power: i32,
}

pub const GHAST_MAX_HEALTH: f32 = 10.0;
pub const GHAST_FOLLOW_RANGE: f32 = 100.0;
pub const GHAST_CAMERA_DISTANCE: f32 = 8.0;
pub const GHAST_FLYING_SPEED: f32 = 0.06;
pub const GHAST_XP_REWARD: i32 = 5;
pub const GHAST_WIDTH: f32 = 4.0;
pub const GHAST_HEIGHT: f32 = 4.0;
pub const GHAST_EYE_HEIGHT: f32 = 2.6;
pub const GHAST_PASSENGER_ATTACHMENT_Y: f32 = 4.0625;
pub const GHAST_RIDING_OFFSET: f32 = 0.5;
pub const GHAST_CLIENT_TRACKING_RANGE: i32 = 10;
pub const GHAST_DEFAULT_EXPLOSION_POWER: i32 = 1;
pub const GHAST_SOUND_VOLUME: f32 = 5.0;
pub const GHAST_TARGET_VERTICAL_RANGE: f64 = 4.0;
pub const GHAST_TARGET_CHANCE_INTERVAL: i32 = 10;
pub const GHAST_SHOOT_MAX_DISTANCE_SQR: f64 = 4096.0;
pub const GHAST_CHARGE_WARN_TICKS: i32 = 10;
pub const GHAST_CHARGE_SHOOT_TICKS: i32 = 20;
pub const GHAST_CHARGE_COOLDOWN_AFTER_SHOT: i32 = -40;
pub const GHAST_WARN_LEVEL_EVENT: i32 = 1015;
pub const GHAST_SHOOT_LEVEL_EVENT: i32 = 1016;
pub const GHAST_FIREBALL_SPAWN_OFFSET: f64 = 4.0;
pub const GHAST_FIREBALL_Y_OFFSET_FROM_MID: f64 = 0.5;
pub const GHAST_FIREBALL_ENTITY_DAMAGE: f32 = 6.0;
pub const GHAST_REFLECTED_FIREBALL_DAMAGE: f32 = 1000.0;
pub const GHAST_RANDOM_FLOAT_MAX_ATTEMPTS: i32 = 64;
pub const GHAST_RANDOM_FLOAT_RANGE: f64 = 16.0;
pub const GHAST_RANDOM_FLOAT_REACHED_DISTANCE_SQR: f64 = 1.0;
pub const GHAST_RANDOM_FLOAT_TOO_FAR_DISTANCE_SQR: f64 = 3600.0;
pub const GHAST_MOVE_FLOAT_DURATION_RANDOM_BOUND: i32 = 5;
pub const GHAST_MOVE_FLOAT_DURATION_MIN_ADD: i32 = 2;
pub const GHAST_MOVE_ACCELERATION_SCALE: f64 = 5.0 / 3.0;
pub const GHAST_LEASH_ELASTIC_DISTANCE: f64 = 10.0;
pub const GHAST_LEASH_SNAP_DISTANCE: f64 = 16.0;

pub fn ghast_attributes() -> GhastAttributes {
    GhastAttributes {
        max_health: GHAST_MAX_HEALTH,
        follow_range: GHAST_FOLLOW_RANGE,
        camera_distance: GHAST_CAMERA_DISTANCE,
        flying_speed: GHAST_FLYING_SPEED,
        xp_reward: GHAST_XP_REWARD,
    }
}

pub fn ghast_entity_type_surface() -> GhastEntityTypeSurface {
    GhastEntityTypeSurface {
        width: GHAST_WIDTH,
        height: GHAST_HEIGHT,
        eye_height: GHAST_EYE_HEIGHT,
        passenger_attachment_y: GHAST_PASSENGER_ATTACHMENT_Y,
        riding_offset: GHAST_RIDING_OFFSET,
        client_tracking_range: GHAST_CLIENT_TRACKING_RANGE,
        fire_immune: true,
        not_in_peaceful: true,
    }
}

pub fn ghast_spawn_allowed(
    peaceful_difficulty: bool,
    random_0_to_19: i32,
    mob_spawn_rules_pass: bool,
) -> bool {
    !peaceful_difficulty && random_0_to_19.rem_euclid(20) == 0 && mob_spawn_rules_pass
}

pub fn ghast_target_predicate_matches(abs_target_y_delta: f64) -> bool {
    abs_target_y_delta <= GHAST_TARGET_VERTICAL_RANGE
}

pub fn ghast_is_reflected_fireball(
    direct_entity: &'static str,
    source_entity: &'static str,
) -> bool {
    direct_entity == "minecraft:fireball" && source_entity == "minecraft:player"
}

pub fn ghast_hurt_damage(
    reflected_fireball: bool,
    invulnerable_to_source: bool,
    incoming_damage: f32,
) -> Option<f32> {
    if reflected_fireball {
        Some(GHAST_REFLECTED_FIREBALL_DAMAGE)
    } else if invulnerable_to_source {
        None
    } else {
        Some(incoming_damage)
    }
}

pub fn ghast_shoot_fireball_tick(
    charge_time: i32,
    target_present: bool,
    target_distance_sqr: f64,
    has_line_of_sight: bool,
    silent: bool,
    explosion_power: i32,
) -> GhastShootTick {
    if !target_present {
        return GhastShootTick {
            charge_time,
            charging: false,
            warn_level_event: None,
            shoot_level_event: None,
            fireball: None,
        };
    }

    if target_distance_sqr < GHAST_SHOOT_MAX_DISTANCE_SQR && has_line_of_sight {
        let next_charge_time = charge_time + 1;
        if next_charge_time == GHAST_CHARGE_SHOOT_TICKS {
            return GhastShootTick {
                charge_time: GHAST_CHARGE_COOLDOWN_AFTER_SHOT,
                charging: false,
                warn_level_event: None,
                shoot_level_event: (!silent).then_some(GHAST_SHOOT_LEVEL_EVENT),
                fireball: Some(GhastFireballPlan {
                    spawn_offset: GHAST_FIREBALL_SPAWN_OFFSET,
                    y_offset_from_ghast_mid: GHAST_FIREBALL_Y_OFFSET_FROM_MID,
                    explosion_power,
                }),
            };
        }

        GhastShootTick {
            charge_time: next_charge_time,
            charging: next_charge_time > GHAST_CHARGE_WARN_TICKS,
            warn_level_event: (next_charge_time == GHAST_CHARGE_WARN_TICKS && !silent)
                .then_some(GHAST_WARN_LEVEL_EVENT),
            shoot_level_event: None,
            fireball: None,
        }
    } else {
        let next_charge_time = if charge_time > 0 {
            charge_time - 1
        } else {
            charge_time
        };
        GhastShootTick {
            charge_time: next_charge_time,
            charging: next_charge_time > GHAST_CHARGE_WARN_TICKS,
            warn_level_event: None,
            shoot_level_event: None,
            fireball: None,
        }
    }
}

pub fn ghast_random_float_can_use(move_control_has_wanted: bool, wanted_distance_sqr: f64) -> bool {
    !move_control_has_wanted
        || wanted_distance_sqr < GHAST_RANDOM_FLOAT_REACHED_DISTANCE_SQR
        || wanted_distance_sqr > GHAST_RANDOM_FLOAT_TOO_FAR_DISTANCE_SQR
}

pub fn ghast_random_float_target(
    center: (f64, f64, f64),
    random_x: f64,
    random_y: f64,
    random_z: f64,
) -> (f64, f64, f64) {
    (
        center.0 + (random_x * 2.0 - 1.0) * GHAST_RANDOM_FLOAT_RANGE,
        center.1 + (random_y * 2.0 - 1.0) * GHAST_RANDOM_FLOAT_RANGE,
        center.2 + (random_z * 2.0 - 1.0) * GHAST_RANDOM_FLOAT_RANGE,
    )
}

pub fn ghast_move_float_duration_tick(current_duration: i32, random_0_to_4: i32) -> i32 {
    current_duration - 1
        + random_0_to_4.rem_euclid(GHAST_MOVE_FLOAT_DURATION_RANDOM_BOUND)
        + GHAST_MOVE_FLOAT_DURATION_MIN_ADD
}

pub fn large_fireball_hit_outcome(mob_griefing: bool, explosion_power: i32) -> (f32, bool, bool) {
    (explosion_power as f32, mob_griefing, true)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StriderAttributes {
    pub movement_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StriderEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub client_tracking_range: i32,
    pub fire_immune: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StriderSuffocationState {
    pub suffocating: bool,
    pub movement_speed_modifier: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StriderFinalizeSpawn {
    None,
    ZombifiedPiglinJockey {
        jockey_holds: &'static str,
        strider_saddle: &'static str,
        guaranteed_saddle_drop: bool,
    },
    BabyStriderJockey {
        baby_age: i32,
    },
    AgeableGroup {
        baby_chance: i32,
    },
}

pub const STRIDER_MOVEMENT_SPEED: f32 = 0.175;
pub const STRIDER_WIDTH: f32 = 0.9;
pub const STRIDER_HEIGHT: f32 = 1.7;
pub const STRIDER_CLIENT_TRACKING_RANGE: i32 = 10;
pub const STRIDER_WATER_PATHFINDING_MALUS: f32 = -1.0;
pub const STRIDER_LAVA_PATHFINDING_MALUS: f32 = 0.0;
pub const STRIDER_FIRE_PATHFINDING_MALUS: f32 = 0.0;
pub const STRIDER_SUFFOCATING_MODIFIER: f32 = -0.34;
pub const STRIDER_SUFFOCATE_STEERING_MODIFIER: f32 = 0.35;
pub const STRIDER_STEERING_MODIFIER: f32 = 0.55;
pub const STRIDER_PANIC_SPEED: f32 = 1.65;
pub const STRIDER_BREED_SPEED: f32 = 1.0;
pub const STRIDER_TEMPT_SPEED: f32 = 1.4;
pub const STRIDER_GO_TO_LAVA_SPEED: f32 = 1.0;
pub const STRIDER_GO_TO_LAVA_SEARCH_RANGE: i32 = 8;
pub const STRIDER_GO_TO_LAVA_VERTICAL_SEARCH_RANGE: i32 = 2;
pub const STRIDER_RANDOM_STROLL_SPEED: f32 = 1.0;
pub const STRIDER_RANDOM_STROLL_INTERVAL: i32 = 60;
pub const STRIDER_LOOK_RANGE: f32 = 8.0;
pub const STRIDER_HAPPY_SOUND_RANDOM_BOUND: i32 = 140;
pub const STRIDER_RETREAT_SOUND_RANDOM_BOUND: i32 = 60;
pub const STRIDER_FLOAT_LAVA_DAMPING: f64 = 0.5;
pub const STRIDER_FLOAT_LAVA_UPWARD_PUSH: f64 = 0.05;
pub const STRIDER_LIQUID_COLLISION_HEIGHT: f64 = 8.0;
pub const STRIDER_STEP_DISTANCE_INCREMENT: f32 = 0.6;
pub const STRIDER_STEP_SOUND_VOLUME: f32 = 1.0;
pub const STRIDER_STEP_SOUND_PITCH: f32 = 1.0;
pub const STRIDER_ZOMBIFIED_PIGLIN_JOCKEY_RANDOM_BOUND: i32 = 30;
pub const STRIDER_BABY_JOCKEY_RANDOM_BOUND: i32 = 10;
pub const STRIDER_BABY_JOCKEY_AGE: i32 = -24000;
pub const STRIDER_GROUP_BABY_CHANCE_PERCENT: i32 = 50;

pub fn strider_attributes() -> StriderAttributes {
    StriderAttributes {
        movement_speed: STRIDER_MOVEMENT_SPEED,
    }
}

pub fn strider_entity_type_surface() -> StriderEntityTypeSurface {
    StriderEntityTypeSurface {
        width: STRIDER_WIDTH,
        height: STRIDER_HEIGHT,
        client_tracking_range: STRIDER_CLIENT_TRACKING_RANGE,
        fire_immune: true,
    }
}

pub fn strider_can_stand_on_fluid(fluid: &'static str) -> bool {
    fluid == "minecraft:lava"
}

pub fn strider_spawn_allowed(first_non_lava_block_above_is_air: bool) -> bool {
    first_non_lava_block_above_is_air
}

pub fn strider_can_use_saddle_slot(alive: bool, baby: bool) -> bool {
    alive && !baby
}

pub fn strider_controlling_passenger(
    saddled: bool,
    first_passenger_is_player: bool,
    player_holds_warped_fungus_on_a_stick: bool,
) -> bool {
    saddled && first_passenger_is_player && player_holds_warped_fungus_on_a_stick
}

pub fn strider_ridden_speed(movement_speed: f32, suffocating: bool, boost_factor: f32) -> f32 {
    movement_speed
        * if suffocating {
            STRIDER_SUFFOCATE_STEERING_MODIFIER
        } else {
            STRIDER_STEERING_MODIFIER
        }
        * boost_factor
}

pub fn strider_suffocation_state(
    no_ai: bool,
    inside_warm_block: bool,
    on_warm_block: bool,
    lava_fluid_height_positive: bool,
    vehicle_is_warm_strider: bool,
) -> Option<StriderSuffocationState> {
    if no_ai {
        return None;
    }
    let warm =
        inside_warm_block || on_warm_block || lava_fluid_height_positive || vehicle_is_warm_strider;
    Some(StriderSuffocationState {
        suffocating: !warm,
        movement_speed_modifier: (!warm).then_some(STRIDER_SUFFOCATING_MODIFIER),
    })
}

pub fn strider_walk_target_value(target_has_lava_fluid: bool, currently_in_lava: bool) -> f32 {
    if target_has_lava_fluid {
        10.0
    } else if currently_in_lava {
        f32::NEG_INFINITY
    } else {
        0.0
    }
}

pub fn strider_can_add_passenger(already_vehicle: bool, eye_in_lava: bool) -> bool {
    !already_vehicle && !eye_in_lava
}

pub fn strider_interaction_starts_riding(
    has_food_in_hand: bool,
    saddled: bool,
    already_vehicle: bool,
    player_secondary_use_active: bool,
) -> bool {
    !has_food_in_hand && saddled && !already_vehicle && !player_secondary_use_active
}

pub fn strider_float_in_lava(
    in_lava: bool,
    above_liquid_collision_shape: bool,
    block_above_is_lava: bool,
    current_delta: (f64, f64, f64),
) -> (bool, (f64, f64, f64)) {
    if !in_lava {
        return (false, current_delta);
    }
    if above_liquid_collision_shape && !block_above_is_lava {
        (true, current_delta)
    } else {
        (
            false,
            (
                current_delta.0 * STRIDER_FLOAT_LAVA_DAMPING,
                current_delta.1 * STRIDER_FLOAT_LAVA_DAMPING + STRIDER_FLOAT_LAVA_UPWARD_PUSH,
                current_delta.2 * STRIDER_FLOAT_LAVA_DAMPING,
            ),
        )
    }
}

pub fn strider_go_to_lava_can_use(in_lava: bool, super_can_use: bool) -> bool {
    !in_lava && super_can_use
}

pub fn strider_go_to_lava_can_continue(in_lava: bool, target_valid: bool) -> bool {
    !in_lava && target_valid
}

pub fn strider_go_to_lava_valid_target(
    block: &'static str,
    block_above_pathfindable_land: bool,
) -> bool {
    block == "minecraft:lava" && block_above_pathfindable_land
}

pub fn strider_navigation_valid_path_type(path_type: &'static str, super_valid: bool) -> bool {
    matches!(path_type, "lava" | "fire" | "fire_in_neighbor") || super_valid
}

pub fn strider_finalize_spawn(
    baby: bool,
    random_0_to_29: i32,
    random_0_to_9_after_failed_zombie_jockey_roll: i32,
) -> StriderFinalizeSpawn {
    if baby {
        return StriderFinalizeSpawn::None;
    }
    if random_0_to_29.rem_euclid(STRIDER_ZOMBIFIED_PIGLIN_JOCKEY_RANDOM_BOUND) == 0 {
        StriderFinalizeSpawn::ZombifiedPiglinJockey {
            jockey_holds: "minecraft:warped_fungus_on_a_stick",
            strider_saddle: "minecraft:saddle",
            guaranteed_saddle_drop: true,
        }
    } else if random_0_to_9_after_failed_zombie_jockey_roll
        .rem_euclid(STRIDER_BABY_JOCKEY_RANDOM_BOUND)
        == 0
    {
        StriderFinalizeSpawn::BabyStriderJockey {
            baby_age: STRIDER_BABY_JOCKEY_AGE,
        }
    } else {
        StriderFinalizeSpawn::AgeableGroup {
            baby_chance: STRIDER_GROUP_BABY_CHANCE_PERCENT,
        }
    }
}

pub fn strider_is_sensitive_to_water() -> bool {
    true
}

pub fn strider_is_on_fire() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchRangedAttack {
    pub potion: &'static str,
    pub clear_target: bool,
    pub velocity: f32,
    pub inaccuracy: f32,
    pub throw_sound: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchDrinkStart {
    pub potion: &'static str,
    pub using_item: bool,
    pub speed_modifier: f32,
    pub drink_sound: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WitchDrinkFinish {
    pub using_item: bool,
    pub clear_main_hand: bool,
    pub apply_potion_effects: bool,
    pub game_event: &'static str,
    pub remove_speed_modifier: bool,
}

pub const WITCH_MAX_HEALTH: f32 = 26.0;
pub const WITCH_MOVEMENT_SPEED: f32 = 0.25;
pub const WITCH_DRINKING_SPEED_MODIFIER: f32 = -0.25;
pub const WITCH_RANGED_ATTACK_SPEED: f32 = 1.0;
pub const WITCH_RANGED_ATTACK_INTERVAL_TICKS: i32 = 60;
pub const WITCH_RANGED_ATTACK_RADIUS: f32 = 10.0;
pub const WITCH_RANDOM_STROLL_SPEED: f32 = 1.0;
pub const WITCH_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const WITCH_ATTACK_PLAYER_RANDOM_INTERVAL: i32 = 10;
pub const WITCH_WATER_BREATHING_CHANCE: f32 = 0.15;
pub const WITCH_FIRE_RESISTANCE_CHANCE: f32 = 0.15;
pub const WITCH_HEALING_CHANCE: f32 = 0.05;
pub const WITCH_SWIFTNESS_CHANCE: f32 = 0.5;
pub const WITCH_SWIFTNESS_DISTANCE_SQR: f32 = 121.0;
pub const WITCH_THROW_SLOWNESS_DISTANCE: f64 = 8.0;
pub const WITCH_THROW_POISON_MIN_HEALTH: f32 = 8.0;
pub const WITCH_THROW_WEAKNESS_DISTANCE: f64 = 3.0;
pub const WITCH_THROW_WEAKNESS_CHANCE: f32 = 0.25;
pub const WITCH_RAIDER_HEALING_HEALTH: f32 = 4.0;
pub const WITCH_CLOSE_THROW_VELOCITY: f32 = 0.45;
pub const WITCH_FAR_THROW_VELOCITY: f32 = 0.75;
pub const WITCH_THROW_CLOSE_DISTANCE: f64 = 2.0;
pub const WITCH_THROW_INACCURACY: f32 = 8.0;
pub const WITCH_PARTICLE_EVENT_ID: u8 = 15;
pub const WITCH_PARTICLE_CHANCE: f32 = 7.5E-4;
pub const WITCH_PARTICLE_MIN_COUNT: i32 = 10;
pub const WITCH_PARTICLE_RANDOM_BOUND: i32 = 35;
pub const WITCH_RESISTANT_DAMAGE_SCALE: f32 = 0.15;
pub const WITCH_CAN_BE_RAID_LEADER: bool = false;
pub const WITCH_RAID_BUFFS_APPLIED: bool = false;

pub fn witch_attributes() -> WitchAttributes {
    WitchAttributes {
        max_health: WITCH_MAX_HEALTH,
        movement_speed: WITCH_MOVEMENT_SPEED,
    }
}

pub fn witch_heal_raiders_goal_enabled(
    has_active_raid: bool,
    target_entity_type: &'static str,
) -> bool {
    has_active_raid && target_entity_type != "minecraft:witch"
}

pub fn witch_attack_players_enabled(heal_raiders_cooldown: i32) -> bool {
    heal_raiders_cooldown <= 0
}

pub fn witch_select_drink_potion(
    water_roll: f32,
    fire_roll: f32,
    heal_roll: f32,
    speed_roll: f32,
    eye_in_water: bool,
    has_water_breathing: bool,
    on_fire_or_fire_damage: bool,
    has_fire_resistance: bool,
    health: f32,
    max_health: f32,
    target_present: bool,
    has_speed: bool,
    target_distance_sqr: f32,
) -> Option<&'static str> {
    if water_roll < WITCH_WATER_BREATHING_CHANCE && eye_in_water && !has_water_breathing {
        Some("minecraft:water_breathing")
    } else if fire_roll < WITCH_FIRE_RESISTANCE_CHANCE
        && on_fire_or_fire_damage
        && !has_fire_resistance
    {
        Some("minecraft:fire_resistance")
    } else if heal_roll < WITCH_HEALING_CHANCE && health < max_health {
        Some("minecraft:healing")
    } else if speed_roll < WITCH_SWIFTNESS_CHANCE
        && target_present
        && !has_speed
        && target_distance_sqr > WITCH_SWIFTNESS_DISTANCE_SQR
    {
        Some("minecraft:swiftness")
    } else {
        None
    }
}

pub fn witch_start_drinking(potion: Option<&'static str>, silent: bool) -> Option<WitchDrinkStart> {
    potion.map(|potion| WitchDrinkStart {
        potion,
        using_item: true,
        speed_modifier: WITCH_DRINKING_SPEED_MODIFIER,
        drink_sound: (!silent).then_some("minecraft:entity.witch.drink"),
    })
}

pub fn witch_finish_drinking(
    item_is_potion: bool,
    potion_contents_present: bool,
) -> WitchDrinkFinish {
    WitchDrinkFinish {
        using_item: false,
        clear_main_hand: true,
        apply_potion_effects: item_is_potion && potion_contents_present,
        game_event: "minecraft:drink",
        remove_speed_modifier: true,
    }
}

pub fn witch_ranged_attack(
    drinking_potion: bool,
    target_is_raider: bool,
    target_health: f32,
    target_has_slowness: bool,
    target_has_poison: bool,
    target_has_weakness: bool,
    horizontal_distance: f64,
    weakness_roll: f32,
    silent: bool,
) -> Option<WitchRangedAttack> {
    if drinking_potion {
        return None;
    }

    let (potion, clear_target) = if target_is_raider {
        (
            if target_health <= WITCH_RAIDER_HEALING_HEALTH {
                "minecraft:healing"
            } else {
                "minecraft:regeneration"
            },
            true,
        )
    } else if horizontal_distance >= WITCH_THROW_SLOWNESS_DISTANCE && !target_has_slowness {
        ("minecraft:slowness", false)
    } else if target_health >= WITCH_THROW_POISON_MIN_HEALTH && !target_has_poison {
        ("minecraft:poison", false)
    } else if horizontal_distance <= WITCH_THROW_WEAKNESS_DISTANCE
        && !target_has_weakness
        && weakness_roll < WITCH_THROW_WEAKNESS_CHANCE
    {
        ("minecraft:weakness", false)
    } else {
        ("minecraft:harming", false)
    };

    Some(WitchRangedAttack {
        potion,
        clear_target,
        velocity: if horizontal_distance <= WITCH_THROW_CLOSE_DISTANCE {
            WITCH_CLOSE_THROW_VELOCITY
        } else {
            WITCH_FAR_THROW_VELOCITY
        },
        inaccuracy: WITCH_THROW_INACCURACY,
        throw_sound: (!silent).then_some("minecraft:entity.witch.throw"),
    })
}

pub fn witch_projectile_y_adjustment(horizontal_distance: f64) -> f64 {
    horizontal_distance * 0.2
}

pub fn witch_damage_after_magic_absorb(
    source_is_self: bool,
    witch_resistant_damage_type: bool,
    damage_after_super: f32,
) -> f32 {
    if source_is_self {
        0.0
    } else if witch_resistant_damage_type {
        damage_after_super * WITCH_RESISTANT_DAMAGE_SCALE
    } else {
        damage_after_super
    }
}

pub fn witch_particle_count(random_0_to_34: i32) -> i32 {
    random_0_to_34.rem_euclid(WITCH_PARTICLE_RANDOM_BOUND) + WITCH_PARTICLE_MIN_COUNT
}

pub fn witch_finalize_can_join_raid(spawn_reason_natural: bool) -> bool {
    !spawn_reason_natural
}

pub fn witch_ravager_rider_in_java_26_1_2() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuardianAttackTick {
    pub attack_time: i32,
    pub active_attack_target: Option<i32>,
    pub broadcast_event: Option<u8>,
    pub magic_damage: Option<f32>,
    pub melee_hit: bool,
    pub clear_target: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElderGuardianEffectPulse {
    pub mining_fatigue_ticks: i32,
    pub amplifier: i32,
    pub radius: f64,
    pub display_limit_ticks: i32,
    pub game_event_strength: f32,
}

pub const GUARDIAN_MAX_HEALTH: f32 = 30.0;
pub const GUARDIAN_MOVEMENT_SPEED: f32 = 0.5;
pub const GUARDIAN_ATTACK_DAMAGE: f32 = 6.0;
pub const GUARDIAN_XP_REWARD: i32 = 10;
pub const GUARDIAN_WIDTH: f32 = 0.85;
pub const GUARDIAN_HEIGHT: f32 = 0.85;
pub const GUARDIAN_EYE_HEIGHT: f32 = 0.425;
pub const GUARDIAN_PASSENGER_ATTACHMENT_Y: f32 = 0.975;
pub const GUARDIAN_CLIENT_TRACKING_RANGE: i32 = 8;
pub const GUARDIAN_ATTACK_DURATION_TICKS: i32 = 80;
pub const GUARDIAN_ATTACK_START_TICKS: i32 = -10;
pub const GUARDIAN_ATTACK_EVENT_ID: u8 = 21;
pub const GUARDIAN_ATTACK_SELECTOR_MIN_DISTANCE_SQR: f32 = 9.0;
pub const GUARDIAN_TARGET_SCAN_INTERVAL: i32 = 10;
pub const GUARDIAN_WATER_PATHFINDING_MALUS: f32 = 0.0;
pub const GUARDIAN_AMBIENT_SOUND_INTERVAL: i32 = 160;
pub const GUARDIAN_WATER_WALK_TARGET_BASE: f32 = 10.0;
pub const GUARDIAN_AIR_SUPPLY_IN_WATER: i32 = 300;
pub const GUARDIAN_LAND_FLOP_Y_PUSH: f64 = 0.5;
pub const GUARDIAN_LAND_FLOP_XZ_SCALE: f64 = 0.4;
pub const GUARDIAN_THORNS_DAMAGE: f32 = 2.0;
pub const GUARDIAN_TRAVEL_WATER_RELATIVE: f32 = 0.1;
pub const GUARDIAN_TRAVEL_WATER_DAMPING: f64 = 0.9;
pub const GUARDIAN_IDLE_SINKING_Y: f64 = -0.005;
pub const GUARDIAN_MAX_HEAD_X_ROT: i32 = 180;

pub const ELDER_GUARDIAN_MAX_HEALTH: f32 = 80.0;
pub const ELDER_GUARDIAN_MOVEMENT_SPEED: f32 = 0.3;
pub const ELDER_GUARDIAN_ATTACK_DAMAGE: f32 = 8.0;
pub const ELDER_GUARDIAN_WIDTH: f32 = 1.9975;
pub const ELDER_GUARDIAN_HEIGHT: f32 = 1.9975;
pub const ELDER_GUARDIAN_EYE_HEIGHT: f32 = 0.99875;
pub const ELDER_GUARDIAN_PASSENGER_ATTACHMENT_Y: f32 = 2.350625;
pub const ELDER_GUARDIAN_CLIENT_TRACKING_RANGE: i32 = 10;
pub const ELDER_GUARDIAN_ATTACK_DURATION_TICKS: i32 = 60;
pub const ELDER_GUARDIAN_RANDOM_STROLL_INTERVAL: i32 = 400;
pub const ELDER_GUARDIAN_EFFECT_INTERVAL: i32 = 1200;
pub const ELDER_GUARDIAN_EFFECT_RADIUS: f64 = 50.0;
pub const ELDER_GUARDIAN_EFFECT_DURATION: i32 = 6000;
pub const ELDER_GUARDIAN_EFFECT_AMPLIFIER: i32 = 2;
pub const ELDER_GUARDIAN_EFFECT_DISPLAY_LIMIT: i32 = 1200;
pub const ELDER_GUARDIAN_HOME_RADIUS: i32 = 16;

pub fn guardian_attributes() -> GuardianAttributes {
    GuardianAttributes {
        max_health: GUARDIAN_MAX_HEALTH,
        movement_speed: GUARDIAN_MOVEMENT_SPEED,
        attack_damage: GUARDIAN_ATTACK_DAMAGE,
        xp_reward: GUARDIAN_XP_REWARD,
    }
}

pub fn elder_guardian_attributes() -> GuardianAttributes {
    GuardianAttributes {
        max_health: ELDER_GUARDIAN_MAX_HEALTH,
        movement_speed: ELDER_GUARDIAN_MOVEMENT_SPEED,
        attack_damage: ELDER_GUARDIAN_ATTACK_DAMAGE,
        xp_reward: GUARDIAN_XP_REWARD,
    }
}

pub fn guardian_entity_type_surface() -> GuardianEntityTypeSurface {
    GuardianEntityTypeSurface {
        width: GUARDIAN_WIDTH,
        height: GUARDIAN_HEIGHT,
        eye_height: GUARDIAN_EYE_HEIGHT,
        passenger_attachment_y: GUARDIAN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: GUARDIAN_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn elder_guardian_entity_type_surface() -> GuardianEntityTypeSurface {
    GuardianEntityTypeSurface {
        width: ELDER_GUARDIAN_WIDTH,
        height: ELDER_GUARDIAN_HEIGHT,
        eye_height: ELDER_GUARDIAN_EYE_HEIGHT,
        passenger_attachment_y: ELDER_GUARDIAN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: ELDER_GUARDIAN_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn guardian_spawn_allowed(
    random_0_to_19: i32,
    can_see_sky_from_below_water: bool,
    peaceful_difficulty: bool,
    spawn_reason_is_spawner: bool,
    current_fluid_is_water: bool,
    below_fluid_is_water: bool,
) -> bool {
    (random_0_to_19.rem_euclid(20) == 0 || !can_see_sky_from_below_water)
        && !peaceful_difficulty
        && (spawn_reason_is_spawner || current_fluid_is_water)
        && below_fluid_is_water
}

pub fn guardian_attack_selector_matches(
    target_entity_type: &'static str,
    distance_sqr: f32,
) -> bool {
    matches!(
        target_entity_type,
        "minecraft:player" | "minecraft:squid" | "minecraft:axolotl"
    ) && distance_sqr > GUARDIAN_ATTACK_SELECTOR_MIN_DISTANCE_SQR
}

pub fn guardian_attack_can_continue(
    elder: bool,
    target_present: bool,
    target_distance_sqr: f32,
    super_can_continue: bool,
) -> bool {
    super_can_continue
        && (elder
            || (target_present && target_distance_sqr > GUARDIAN_ATTACK_SELECTOR_MIN_DISTANCE_SQR))
}

pub fn guardian_attack_tick(
    attack_time: i32,
    target_id: i32,
    has_line_of_sight: bool,
    silent: bool,
    hard_difficulty: bool,
    elder: bool,
) -> GuardianAttackTick {
    if !has_line_of_sight {
        return GuardianAttackTick {
            attack_time,
            active_attack_target: None,
            broadcast_event: None,
            magic_damage: None,
            melee_hit: false,
            clear_target: true,
        };
    }

    let next = attack_time + 1;
    if next == 0 {
        return GuardianAttackTick {
            attack_time: next,
            active_attack_target: Some(target_id),
            broadcast_event: (!silent).then_some(GUARDIAN_ATTACK_EVENT_ID),
            magic_damage: None,
            melee_hit: false,
            clear_target: false,
        };
    }

    let duration = if elder {
        ELDER_GUARDIAN_ATTACK_DURATION_TICKS
    } else {
        GUARDIAN_ATTACK_DURATION_TICKS
    };
    if next >= duration {
        let mut damage = 1.0;
        if hard_difficulty {
            damage += 2.0;
        }
        if elder {
            damage += 2.0;
        }
        GuardianAttackTick {
            attack_time: next,
            active_attack_target: None,
            broadcast_event: None,
            magic_damage: Some(damage),
            melee_hit: true,
            clear_target: true,
        }
    } else {
        GuardianAttackTick {
            attack_time: next,
            active_attack_target: None,
            broadcast_event: None,
            magic_damage: None,
            melee_hit: false,
            clear_target: false,
        }
    }
}

pub fn guardian_thorns_damage(
    moving: bool,
    source_avoids_guardian_thorns: bool,
    source_is_thorns: bool,
    direct_entity_is_living: bool,
) -> Option<f32> {
    (!moving && !source_avoids_guardian_thorns && !source_is_thorns && direct_entity_is_living)
        .then_some(GUARDIAN_THORNS_DAMAGE)
}

pub fn guardian_walk_target_value(in_water: bool, light_level_cost: f32, fallback: f32) -> f32 {
    if in_water {
        GUARDIAN_WATER_WALK_TARGET_BASE + light_level_cost
    } else {
        fallback
    }
}

pub fn elder_guardian_effect_pulse(
    tick_count: i32,
    entity_id: i32,
    silent: bool,
) -> Option<ElderGuardianEffectPulse> {
    ((tick_count + entity_id).rem_euclid(ELDER_GUARDIAN_EFFECT_INTERVAL) == 0).then_some(
        ElderGuardianEffectPulse {
            mining_fatigue_ticks: ELDER_GUARDIAN_EFFECT_DURATION,
            amplifier: ELDER_GUARDIAN_EFFECT_AMPLIFIER,
            radius: ELDER_GUARDIAN_EFFECT_RADIUS,
            display_limit_ticks: ELDER_GUARDIAN_EFFECT_DISPLAY_LIMIT,
            game_event_strength: if silent { 0.0 } else { 1.0 },
        },
    )
}

pub fn elder_guardian_sets_home_when_missing(has_home: bool) -> Option<i32> {
    (!has_home).then_some(ELDER_GUARDIAN_HOME_RADIUS)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub knockback_resistance: f32,
    pub attack_damage: f32,
    pub attack_knockback: f32,
    pub follow_range: f32,
    pub step_height: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub passenger_attachment_y: f32,
    pub passenger_attachment_z: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerAiStep {
    pub movement_speed: f32,
    pub attack_tick: i32,
    pub stunned_tick: i32,
    pub roar_tick: i32,
    pub roar_now: bool,
    pub start_roar_sound: bool,
    pub should_jump_after_leaf_collision: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerBlockedByItem {
    pub stunned_tick: i32,
    pub roar_tick: i32,
    pub stun_event: Option<u8>,
    pub strong_knockback: bool,
    pub defender_hurt_marked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RavagerRoarEffect {
    pub damage: Option<f32>,
    pub strong_knockback: bool,
    pub include_armor_stand: bool,
    pub event: Option<u8>,
}

pub const RAVAGER_MAX_HEALTH: f32 = 100.0;
pub const RAVAGER_BASE_MOVEMENT_SPEED: f32 = 0.3;
pub const RAVAGER_ATTACK_MOVEMENT_SPEED: f32 = 0.35;
pub const RAVAGER_KNOCKBACK_RESISTANCE: f32 = 0.75;
pub const RAVAGER_ATTACK_DAMAGE: f32 = 12.0;
pub const RAVAGER_ATTACK_KNOCKBACK: f32 = 1.5;
pub const RAVAGER_FOLLOW_RANGE: f32 = 32.0;
pub const RAVAGER_STEP_HEIGHT: f32 = 1.0;
pub const RAVAGER_XP_REWARD: i32 = 20;
pub const RAVAGER_WIDTH: f32 = 1.95;
pub const RAVAGER_HEIGHT: f32 = 2.2;
pub const RAVAGER_PASSENGER_ATTACHMENT_Y: f32 = 2.2625;
pub const RAVAGER_PASSENGER_ATTACHMENT_Z: f32 = -0.0625;
pub const RAVAGER_CLIENT_TRACKING_RANGE: i32 = 10;
pub const RAVAGER_ATTACK_DURATION: i32 = 10;
pub const RAVAGER_STUN_DURATION: i32 = 40;
pub const RAVAGER_ROAR_WINDUP_TICKS: i32 = 20;
pub const RAVAGER_ROAR_DAMAGE_TICK: i32 = 10;
pub const RAVAGER_ROAR_RADIUS: f32 = 4.0;
pub const RAVAGER_ROAR_DAMAGE: f32 = 6.0;
pub const RAVAGER_ATTACK_EVENT_ID: u8 = 4;
pub const RAVAGER_STUN_EVENT_ID: u8 = 39;
pub const RAVAGER_ROAR_EVENT_ID: u8 = 69;
pub const RAVAGER_LEAVES_PATHFINDING_MALUS: f32 = 0.0;
pub const RAVAGER_MAX_HEAD_Y_ROT: i32 = 45;
pub const RAVAGER_ATTACK_BB_DEFLATE_XZ: f64 = 0.05;
pub const RAVAGER_RANDOM_STROLL_SPEED: f32 = 0.4;
pub const RAVAGER_LOOK_AT_PLAYER_RANGE: f32 = 6.0;
pub const RAVAGER_LOOK_AT_MOB_RANGE: f32 = 8.0;

pub fn ravager_attributes() -> RavagerAttributes {
    RavagerAttributes {
        max_health: RAVAGER_MAX_HEALTH,
        movement_speed: RAVAGER_BASE_MOVEMENT_SPEED,
        knockback_resistance: RAVAGER_KNOCKBACK_RESISTANCE,
        attack_damage: RAVAGER_ATTACK_DAMAGE,
        attack_knockback: RAVAGER_ATTACK_KNOCKBACK,
        follow_range: RAVAGER_FOLLOW_RANGE,
        step_height: RAVAGER_STEP_HEIGHT,
        xp_reward: RAVAGER_XP_REWARD,
    }
}

pub fn ravager_entity_type_surface() -> RavagerEntityTypeSurface {
    RavagerEntityTypeSurface {
        width: RAVAGER_WIDTH,
        height: RAVAGER_HEIGHT,
        passenger_attachment_y: RAVAGER_PASSENGER_ATTACHMENT_Y,
        passenger_attachment_z: RAVAGER_PASSENGER_ATTACHMENT_Z,
        client_tracking_range: RAVAGER_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn ravager_target_selector_matches(entity_type: &'static str, is_baby: bool) -> bool {
    matches!(entity_type, "minecraft:player" | "minecraft:iron_golem")
        || (entity_type == "minecraft:villager" && !is_baby)
}

pub fn ravager_control_flags_enabled(
    controlling_passenger_is_mob: bool,
    controlling_passenger_is_raider: bool,
    vehicle_is_boat: bool,
) -> (bool, bool, bool, bool) {
    let no_controller = !controlling_passenger_is_mob || controlling_passenger_is_raider;
    (
        no_controller,
        no_controller && !vehicle_is_boat,
        no_controller,
        no_controller,
    )
}

pub fn ravager_ai_step(
    base_movement_speed: f32,
    has_target: bool,
    immobile: bool,
    attack_tick: i32,
    stunned_tick: i32,
    roar_tick: i32,
    horizontal_collision: bool,
    mob_griefing: bool,
    destroyed_leaves: bool,
    on_ground: bool,
) -> RavagerAiStep {
    let movement_speed = if immobile {
        0.0
    } else {
        let target_speed = if has_target {
            RAVAGER_ATTACK_MOVEMENT_SPEED
        } else {
            RAVAGER_BASE_MOVEMENT_SPEED
        };
        base_movement_speed + (target_speed - base_movement_speed) * 0.1
    };

    let mut next_roar = roar_tick;
    let mut roar_now = false;
    if next_roar > 0 {
        next_roar -= 1;
        roar_now = next_roar == RAVAGER_ROAR_DAMAGE_TICK;
    }

    let next_attack = (attack_tick - 1).max(0);
    let mut next_stun = stunned_tick;
    let mut start_roar_sound = false;
    if next_stun > 0 {
        next_stun -= 1;
        if next_stun == 0 {
            start_roar_sound = true;
            next_roar = RAVAGER_ROAR_WINDUP_TICKS;
        }
    }

    RavagerAiStep {
        movement_speed,
        attack_tick: next_attack,
        stunned_tick: next_stun,
        roar_tick: next_roar,
        roar_now,
        start_roar_sound,
        should_jump_after_leaf_collision: horizontal_collision
            && mob_griefing
            && !destroyed_leaves
            && on_ground,
    }
}

pub fn ravager_is_immobile(
    super_immobile: bool,
    attack_tick: i32,
    stunned_tick: i32,
    roar_tick: i32,
) -> bool {
    super_immobile || attack_tick > 0 || stunned_tick > 0 || roar_tick > 0
}

pub fn ravager_has_line_of_sight_allowed(
    stunned_tick: i32,
    roar_tick: i32,
    super_has_line_of_sight: bool,
) -> bool {
    stunned_tick <= 0 && roar_tick <= 0 && super_has_line_of_sight
}

pub fn ravager_blocked_by_item(roar_tick: i32, stun_roll_under_half: bool) -> RavagerBlockedByItem {
    if roar_tick != 0 {
        return RavagerBlockedByItem {
            stunned_tick: 0,
            roar_tick,
            stun_event: None,
            strong_knockback: false,
            defender_hurt_marked: false,
        };
    }
    RavagerBlockedByItem {
        stunned_tick: if stun_roll_under_half {
            RAVAGER_STUN_DURATION
        } else {
            0
        },
        roar_tick: 0,
        stun_event: stun_roll_under_half.then_some(RAVAGER_STUN_EVENT_ID),
        strong_knockback: !stun_roll_under_half,
        defender_hurt_marked: true,
    }
}

pub fn ravager_roar_effect(
    target_entity_type: &'static str,
    target_alive: bool,
    mob_griefing: bool,
) -> Option<RavagerRoarEffect> {
    if !target_alive || target_entity_type == "minecraft:ravager" {
        return None;
    }
    if !mob_griefing && target_entity_type == "minecraft:armor_stand" {
        return None;
    }
    let target_is_illager = matches!(
        target_entity_type,
        "minecraft:evoker" | "minecraft:illusioner" | "minecraft:pillager" | "minecraft:vindicator"
    );
    Some(RavagerRoarEffect {
        damage: (!target_is_illager).then_some(RAVAGER_ROAR_DAMAGE),
        strong_knockback: target_entity_type != "minecraft:player",
        include_armor_stand: target_entity_type == "minecraft:armor_stand" && mob_griefing,
        event: Some(RAVAGER_ROAR_EVENT_ID),
    })
}

pub fn ravager_do_hurt_target_event() -> (i32, u8) {
    (RAVAGER_ATTACK_DURATION, RAVAGER_ATTACK_EVENT_ID)
}

pub fn ravager_can_spawn_without_obstruction(contains_liquid_in_bounding_box: bool) -> bool {
    !contains_liquid_in_bounding_box
}

pub fn ravager_can_be_raid_leader() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShulkerDirection {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl ShulkerDirection {
    pub fn axis(self) -> ShulkerAxis {
        match self {
            ShulkerDirection::Down | ShulkerDirection::Up => ShulkerAxis::Y,
            ShulkerDirection::North | ShulkerDirection::South => ShulkerAxis::Z,
            ShulkerDirection::West | ShulkerDirection::East => ShulkerAxis::X,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShulkerAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerAttributes {
    pub max_health: f32,
    pub covered_armor_bonus: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub client_tracking_range: i32,
    pub fire_immune: bool,
    pub can_spawn_far_from_player: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerBulletSurface {
    pub width: f32,
    pub height: f32,
    pub client_tracking_range: i32,
    pub no_loot_table: bool,
    pub no_physics: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerAttackTick {
    pub attack_time: i32,
    pub raw_peek: i32,
    pub shoot_bullet: bool,
    pub clear_target: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerHitByBullet {
    pub should_spawn_baby: bool,
    pub failure_chance: f32,
}

pub const SHULKER_MAX_HEALTH: f32 = 30.0;
pub const SHULKER_XP_REWARD: i32 = 5;
pub const SHULKER_WIDTH: f32 = 1.0;
pub const SHULKER_HEIGHT: f32 = 1.0;
pub const SHULKER_EYE_HEIGHT: f32 = 0.5;
pub const SHULKER_CLIENT_TRACKING_RANGE: i32 = 10;
pub const SHULKER_DEFAULT_ATTACH_FACE: ShulkerDirection = ShulkerDirection::Down;
pub const SHULKER_DEFAULT_PEEK: i32 = 0;
pub const SHULKER_ATTACK_PEEK: i32 = 100;
pub const SHULKER_IDLE_PEEK: i32 = 30;
pub const SHULKER_NO_COLOR: i32 = 16;
pub const SHULKER_DEFAULT_COLOR: i32 = 16;
pub const SHULKER_COVERED_ARMOR_BONUS: f32 = 20.0;
pub const SHULKER_TELEPORT_STEPS: i32 = 6;
pub const SHULKER_MAX_TELEPORT_DISTANCE: i32 = 8;
pub const SHULKER_TELEPORT_ATTEMPTS: i32 = 5;
pub const SHULKER_OTHER_SCAN_RADIUS: f32 = 8.0;
pub const SHULKER_OTHER_LIMIT: f32 = 5.0;
pub const SHULKER_PEEK_PER_TICK: f32 = 0.05;
pub const SHULKER_MAX_SCALE: f32 = 3.0;
pub const SHULKER_MAX_LID_OPEN: f32 = 1.0;
pub const SHULKER_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const SHULKER_LOOK_AT_PLAYER_PROBABILITY: f32 = 0.02;
pub const SHULKER_TARGET_RANGE_SQR: f32 = 400.0;
pub const SHULKER_ATTACK_START_TICKS: i32 = 20;
pub const SHULKER_ATTACK_RANDOM_STEP: i32 = 10;
pub const SHULKER_ATTACK_RANDOM_MULTIPLIER: i32 = 10;
pub const SHULKER_PEEK_GOAL_ROLL: i32 = 40;
pub const SHULKER_PEEK_BASE_SECONDS: i32 = 20;
pub const SHULKER_BULLET_SPEED: f32 = 0.15;
pub const SHULKER_BULLET_GRAVITY: f32 = 0.04;
pub const SHULKER_BULLET_DAMAGE: f32 = 4.0;
pub const SHULKER_BULLET_LEVITATION_TICKS: i32 = 200;
pub const SHULKER_BULLET_WIDTH: f32 = 0.3125;
pub const SHULKER_BULLET_HEIGHT: f32 = 0.3125;
pub const SHULKER_BULLET_CLIENT_TRACKING_RANGE: i32 = 8;
pub const SHULKER_MAX_HEAD_X_ROT: i32 = 180;
pub const SHULKER_MAX_HEAD_Y_ROT: i32 = 180;
pub const SHULKER_RENDER_DISTANCE_SQR: f32 = 16384.0;

pub fn shulker_attributes() -> ShulkerAttributes {
    ShulkerAttributes {
        max_health: SHULKER_MAX_HEALTH,
        covered_armor_bonus: SHULKER_COVERED_ARMOR_BONUS,
        xp_reward: SHULKER_XP_REWARD,
    }
}

pub fn shulker_entity_type_surface() -> ShulkerEntityTypeSurface {
    ShulkerEntityTypeSurface {
        width: SHULKER_WIDTH,
        height: SHULKER_HEIGHT,
        eye_height: SHULKER_EYE_HEIGHT,
        client_tracking_range: SHULKER_CLIENT_TRACKING_RANGE,
        fire_immune: true,
        can_spawn_far_from_player: true,
    }
}

pub fn shulker_bullet_surface() -> ShulkerBulletSurface {
    ShulkerBulletSurface {
        width: SHULKER_BULLET_WIDTH,
        height: SHULKER_BULLET_HEIGHT,
        client_tracking_range: SHULKER_BULLET_CLIENT_TRACKING_RANGE,
        no_loot_table: true,
        no_physics: true,
    }
}

pub fn shulker_update_peek_amount(current: f32, raw_peek: i32) -> f32 {
    let target = raw_peek as f32 * 0.01;
    if current > target {
        (current - SHULKER_PEEK_PER_TICK).clamp(target, SHULKER_MAX_LID_OPEN)
    } else if current < target {
        (current + SHULKER_PEEK_PER_TICK).clamp(0.0, target)
    } else {
        current
    }
}

pub fn shulker_raw_peek_armor_bonus(raw_peek: i32) -> Option<f32> {
    (raw_peek == 0).then_some(SHULKER_COVERED_ARMOR_BONUS)
}

pub fn shulker_color_from_data(color: i32) -> Option<i32> {
    (0..=15).contains(&color).then_some(color)
}

pub fn shulker_sanitized_scale(scale: f32) -> f32 {
    scale.min(SHULKER_MAX_SCALE)
}

pub fn shulker_hurt_allowed(raw_peek: i32, direct_entity_type: &'static str) -> bool {
    !(raw_peek == 0 && direct_entity_type == "minecraft:arrow")
}

pub fn shulker_should_teleport_after_hurt(
    health: f32,
    max_health: f32,
    random_0_to_3: i32,
) -> bool {
    health < max_health * 0.5 && random_0_to_3.rem_euclid(4) == 0
}

pub fn shulker_hit_by_bullet(
    raw_peek: i32,
    teleport_succeeded: bool,
    nearby_shulker_count: i32,
    random_float_0_to_1: f32,
) -> ShulkerHitByBullet {
    let failure_chance = (nearby_shulker_count - 1) as f32 / SHULKER_OTHER_LIMIT;
    ShulkerHitByBullet {
        should_spawn_baby: raw_peek != 0
            && teleport_succeeded
            && random_float_0_to_1 >= failure_chance,
        failure_chance,
    }
}

pub fn shulker_attack_can_use(target_alive: bool, peaceful_difficulty: bool) -> bool {
    target_alive && !peaceful_difficulty
}

pub fn shulker_attack_tick(
    attack_time: i32,
    peaceful_difficulty: bool,
    target_alive: bool,
    distance_sqr: f32,
    random_0_to_9: i32,
) -> ShulkerAttackTick {
    if peaceful_difficulty || !target_alive {
        return ShulkerAttackTick {
            attack_time,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: false,
            clear_target: false,
        };
    }

    let next = attack_time - 1;
    if distance_sqr >= SHULKER_TARGET_RANGE_SQR {
        return ShulkerAttackTick {
            attack_time: next,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: false,
            clear_target: true,
        };
    }

    if next <= 0 {
        ShulkerAttackTick {
            attack_time: 20 + random_0_to_9.rem_euclid(10) * SHULKER_ATTACK_RANDOM_MULTIPLIER / 2,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: true,
            clear_target: false,
        }
    } else {
        ShulkerAttackTick {
            attack_time: next,
            raw_peek: SHULKER_ATTACK_PEEK,
            shoot_bullet: false,
            clear_target: false,
        }
    }
}

pub fn shulker_defense_search_inflate(
    attach_face: ShulkerDirection,
    follow_distance: f32,
) -> (f32, f32, f32) {
    match attach_face.axis() {
        ShulkerAxis::X => (4.0, follow_distance, follow_distance),
        ShulkerAxis::Z => (follow_distance, follow_distance, 4.0),
        ShulkerAxis::Y => (follow_distance, 4.0, follow_distance),
    }
}

pub fn shulker_bullet_on_hit_entity(
    target_hurt: bool,
    target_is_living: bool,
) -> Option<(f32, i32)> {
    (target_hurt && target_is_living)
        .then_some((SHULKER_BULLET_DAMAGE, SHULKER_BULLET_LEVITATION_TICKS))
}

pub fn shulker_bullet_discards_in_peaceful(peaceful_difficulty: bool) -> bool {
    peaceful_difficulty
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieAttributes {
    pub follow_range: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub armor: f32,
    pub baby_speed_modifier: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieFinalizeSpawnOutcome {
    pub can_pick_up_loot: Option<bool>,
    pub is_baby: bool,
    pub tried_existing_chicken_jockey: bool,
    pub spawned_new_chicken_jockey: bool,
    pub can_break_doors: bool,
    pub halloween_head: Option<&'static str>,
    pub halloween_head_drop_chance: Option<f32>,
    pub reinforcement_base_chance: f64,
    pub knockback_resistance_bonus: f64,
    pub follow_range_bonus: Option<f64>,
    pub leader_reinforcement_bonus: Option<f64>,
    pub leader_max_health_bonus: Option<f64>,
    pub reset_health_to_max: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombieReinforcementOutcome {
    pub attempts: i32,
    pub spawned: bool,
    pub caller_reinforcement_delta: f64,
    pub callee_reinforcement_delta: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZombieVillagerConversionOutcome {
    NoConversion,
    PerishedNormally,
    Converted {
        preserve_villager_data: bool,
        preserve_gossips: bool,
        preserve_trade_offers: bool,
        preserve_xp: bool,
        level_event: Option<i32>,
    },
}

pub const ZOMBIE_FOLLOW_RANGE: f32 = 35.0;
pub const ZOMBIE_MOVEMENT_SPEED: f32 = 0.23;
pub const ZOMBIE_ATTACK_DAMAGE: f32 = 3.0;
pub const ZOMBIE_ARMOR: f32 = 2.0;
pub const ZOMBIE_BABY_SPEED_MODIFIER: f32 = 0.5;
pub const ZOMBIE_BABY_WIDTH: f32 = 0.49;
pub const ZOMBIE_BABY_HEIGHT: f32 = 0.99;
pub const ZOMBIE_BABY_EYE_HEIGHT: f32 = 0.775;
pub const ZOMBIE_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const ZOMBIE_BABY_SPAWN_CHANCE: f32 = 0.05;
pub const ZOMBIE_BABY_XP_MULTIPLIER: f32 = 2.5;
pub const ZOMBIE_WATER_CONVERSION_START_TICKS: i32 = 600;
pub const ZOMBIE_WATER_CONVERSION_DURATION_TICKS: i32 = 300;
pub const ZOMBIE_DROWNED_CONVERSION_TARGET: &str = "minecraft:drowned";
pub const ZOMBIE_DROWNED_CONVERSION_LEVEL_EVENT: i32 = 1040;
pub const ZOMBIE_VILLAGER_CONVERSION_LEVEL_EVENT: i32 = 1026;
pub const ZOMBIE_REINFORCEMENT_ATTEMPTS: i32 = 50;
pub const ZOMBIE_REINFORCEMENT_RANGE_MIN: i32 = 7;
pub const ZOMBIE_REINFORCEMENT_RANGE_MAX: i32 = 40;
pub const ZOMBIE_REINFORCEMENT_CALLER_DELTA: f64 = -0.05;
pub const ZOMBIE_REINFORCEMENT_CALLEE_DELTA: f64 = -0.05;
pub const ZOMBIE_LOOT_PICKUP_CHANCE_SCALE: f32 = 0.55;
pub const ZOMBIE_BREAK_DOOR_CHANCE_SCALE: f32 = 0.1;
pub const ZOMBIE_CHICKEN_JOCKEY_EXISTING_CHANCE: f32 = 0.05;
pub const ZOMBIE_CHICKEN_JOCKEY_NEW_CHANCE: f32 = 0.05;
pub const ZOMBIE_EQUIPMENT_CHANCE_NORMAL: f32 = 0.01;
pub const ZOMBIE_EQUIPMENT_CHANCE_HARD: f32 = 0.05;
pub const ZOMBIE_EQUIPMENT_RANDOM_BOUND: i32 = 6;
pub const ZOMBIE_FIRE_ON_HIT_CHANCE_SCALE: f32 = 0.3;
pub const ZOMBIE_HALLOWEEN_HEAD_CHANCE: f32 = 0.25;
pub const ZOMBIE_HALLOWEEN_JACK_O_LANTERN_CHANCE: f32 = 0.1;
pub const ZOMBIE_RANDOM_REINFORCEMENT_BASE_MAX: f64 = 0.1;
pub const ZOMBIE_RANDOM_KNOCKBACK_RESISTANCE_MAX: f64 = 0.05;
pub const ZOMBIE_FOLLOW_RANGE_BONUS_SCALE: f64 = 1.5;
pub const ZOMBIE_FOLLOW_RANGE_BONUS_THRESHOLD: f64 = 1.0;
pub const ZOMBIE_LEADER_CHANCE_SCALE: f32 = 0.05;
pub const ZOMBIE_LEADER_REINFORCEMENT_BONUS_MIN: f64 = 0.5;
pub const ZOMBIE_LEADER_REINFORCEMENT_BONUS_RANGE: f64 = 0.25;
pub const ZOMBIE_LEADER_MAX_HEALTH_BONUS_MIN: f64 = 1.0;
pub const ZOMBIE_LEADER_MAX_HEALTH_BONUS_RANGE: f64 = 3.0;

pub fn zombie_attributes() -> ZombieAttributes {
    ZombieAttributes {
        follow_range: ZOMBIE_FOLLOW_RANGE,
        movement_speed: ZOMBIE_MOVEMENT_SPEED,
        attack_damage: ZOMBIE_ATTACK_DAMAGE,
        armor: ZOMBIE_ARMOR,
        baby_speed_modifier: ZOMBIE_BABY_SPEED_MODIFIER,
    }
}

pub fn zombie_baby_dimensions() -> ZombieBabyDimensions {
    ZombieBabyDimensions {
        width: ZOMBIE_BABY_WIDTH,
        height: ZOMBIE_BABY_HEIGHT,
        eye_height: ZOMBIE_BABY_EYE_HEIGHT,
        vehicle_attachment_y: ZOMBIE_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn zombie_is_sun_sensitive() -> bool {
    true
}

pub fn zombie_baby_xp_reward(base_xp_reward: i32, baby: bool) -> i32 {
    if baby {
        (base_xp_reward as f32 * ZOMBIE_BABY_XP_MULTIPLIER) as i32
    } else {
        base_xp_reward
    }
}

pub fn zombie_spawn_as_baby(random_float: f32) -> bool {
    random_float < ZOMBIE_BABY_SPAWN_CHANCE
}

pub fn zombie_water_conversion_tick(
    baby: bool,
    converts_in_water: bool,
    under_water_converting: bool,
    conversion_time: i32,
    in_water_time: i32,
    eye_in_water: bool,
) -> (i32, i32, bool, Option<&'static str>) {
    if baby || !converts_in_water {
        return (in_water_time, conversion_time, under_water_converting, None);
    }
    if under_water_converting {
        let next_conversion_time = conversion_time - 1;
        return (
            in_water_time,
            next_conversion_time,
            true,
            (next_conversion_time < 0).then_some(ZOMBIE_DROWNED_CONVERSION_TARGET),
        );
    }
    if eye_in_water {
        let next_in_water_time = in_water_time + 1;
        if next_in_water_time >= ZOMBIE_WATER_CONVERSION_START_TICKS {
            (
                next_in_water_time,
                ZOMBIE_WATER_CONVERSION_DURATION_TICKS,
                true,
                None,
            )
        } else {
            (next_in_water_time, conversion_time, false, None)
        }
    } else {
        (-1, conversion_time, false, None)
    }
}

pub fn zombie_drowned_conversion_event(silent: bool) -> Option<i32> {
    (!silent).then_some(ZOMBIE_DROWNED_CONVERSION_LEVEL_EVENT)
}

pub fn zombie_fire_on_hit_seconds(
    super_hurt_succeeded: bool,
    main_hand_empty: bool,
    on_fire: bool,
    random_float: f32,
    effective_difficulty: f32,
) -> Option<i32> {
    (super_hurt_succeeded
        && main_hand_empty
        && on_fire
        && random_float < effective_difficulty * ZOMBIE_FIRE_ON_HIT_CHANCE_SCALE)
        .then_some(2 * effective_difficulty as i32)
}

pub fn zombie_reinforcement_attempt(
    super_hurt_succeeded: bool,
    target_present: bool,
    hard_difficulty: bool,
    random_float: f32,
    spawn_reinforcements_chance: f32,
    level_spawning_monsters: bool,
    valid_candidate_found: bool,
) -> ZombieReinforcementOutcome {
    let can_try = super_hurt_succeeded
        && target_present
        && hard_difficulty
        && random_float < spawn_reinforcements_chance
        && level_spawning_monsters;
    ZombieReinforcementOutcome {
        attempts: if can_try {
            ZOMBIE_REINFORCEMENT_ATTEMPTS
        } else {
            0
        },
        spawned: can_try && valid_candidate_found,
        caller_reinforcement_delta: if can_try && valid_candidate_found {
            ZOMBIE_REINFORCEMENT_CALLER_DELTA
        } else {
            0.0
        },
        callee_reinforcement_delta: if can_try && valid_candidate_found {
            ZOMBIE_REINFORCEMENT_CALLEE_DELTA
        } else {
            0.0
        },
    }
}

pub fn zombie_default_main_hand_item(
    hard_difficulty: bool,
    random_float: f32,
    random_0_to_5: i32,
) -> Option<&'static str> {
    let threshold = if hard_difficulty {
        ZOMBIE_EQUIPMENT_CHANCE_HARD
    } else {
        ZOMBIE_EQUIPMENT_CHANCE_NORMAL
    };
    if random_float >= threshold {
        return None;
    }
    match random_0_to_5.rem_euclid(ZOMBIE_EQUIPMENT_RANDOM_BOUND) {
        0 => Some("minecraft:iron_sword"),
        1 => Some("minecraft:iron_spear"),
        _ => Some("minecraft:iron_shovel"),
    }
}

pub fn zombie_can_hold_item(item: &str, baby: bool, passenger: bool) -> bool {
    !(item == "minecraft:egg" && baby && passenger)
}

pub fn zombie_wants_to_pick_up(item: &str) -> bool {
    item != "minecraft:glow_ink_sac"
}

pub fn zombie_killed_villager_outcome(
    difficulty: &str,
    killed_entity_is_villager: bool,
    normal_difficulty_skip_roll: bool,
    conversion_succeeds: bool,
    silent: bool,
) -> ZombieVillagerConversionOutcome {
    if !killed_entity_is_villager || (difficulty != "normal" && difficulty != "hard") {
        return ZombieVillagerConversionOutcome::NoConversion;
    }
    if difficulty == "normal" && normal_difficulty_skip_roll {
        return ZombieVillagerConversionOutcome::PerishedNormally;
    }
    if conversion_succeeds {
        ZombieVillagerConversionOutcome::Converted {
            preserve_villager_data: true,
            preserve_gossips: true,
            preserve_trade_offers: true,
            preserve_xp: true,
            level_event: (!silent).then_some(ZOMBIE_VILLAGER_CONVERSION_LEVEL_EVENT),
        }
    } else {
        ZombieVillagerConversionOutcome::PerishedNormally
    }
}

pub fn zombie_finalize_spawn_outcome(
    spawn_reason_conversion: bool,
    spawn_reason_load_or_dimension_travel: bool,
    group_data_present: bool,
    group_baby: bool,
    group_can_spawn_jockey: bool,
    spawn_baby_random_float: f32,
    loot_random_float: f32,
    difficulty_special_multiplier: f32,
    existing_chicken_random_float: f32,
    existing_chicken_available: bool,
    new_chicken_random_float: f32,
    door_random_float: f32,
    halloween: bool,
    head_empty: bool,
    halloween_head_random_float: f32,
    jack_o_lantern_random_float: f32,
    reinforcement_base_random_double: f64,
    knockback_random_double: f64,
    follow_range_random_double: f64,
    leader_random_float: f32,
    leader_reinforcement_random_double: f64,
    leader_health_random_double: f64,
) -> ZombieFinalizeSpawnOutcome {
    let is_baby = if group_data_present {
        group_baby
    } else {
        zombie_spawn_as_baby(spawn_baby_random_float)
    };
    let can_pick_up_loot = (!spawn_reason_conversion).then_some(
        loot_random_float < ZOMBIE_LOOT_PICKUP_CHANCE_SCALE * difficulty_special_multiplier,
    );

    let mut tried_existing_chicken_jockey = false;
    let mut spawned_new_chicken_jockey = false;
    if is_baby && group_can_spawn_jockey {
        if existing_chicken_random_float < ZOMBIE_CHICKEN_JOCKEY_EXISTING_CHANCE {
            tried_existing_chicken_jockey = true;
        } else if new_chicken_random_float < ZOMBIE_CHICKEN_JOCKEY_NEW_CHANCE {
            spawned_new_chicken_jockey = true;
        }
    }

    let mut can_break_doors =
        door_random_float < difficulty_special_multiplier * ZOMBIE_BREAK_DOOR_CHANCE_SCALE;
    let leader = leader_random_float < difficulty_special_multiplier * ZOMBIE_LEADER_CHANCE_SCALE;
    if leader {
        can_break_doors = true;
    }

    let halloween_head =
        if head_empty && halloween && halloween_head_random_float < ZOMBIE_HALLOWEEN_HEAD_CHANCE {
            Some(
                if jack_o_lantern_random_float < ZOMBIE_HALLOWEEN_JACK_O_LANTERN_CHANCE {
                    "minecraft:jack_o_lantern"
                } else {
                    "minecraft:carved_pumpkin"
                },
            )
        } else {
            None
        };

    let follow_range_bonus = follow_range_random_double
        * ZOMBIE_FOLLOW_RANGE_BONUS_SCALE
        * difficulty_special_multiplier as f64;
    let leader_reinforcement_bonus = leader.then_some(
        leader_reinforcement_random_double * ZOMBIE_LEADER_REINFORCEMENT_BONUS_RANGE
            + ZOMBIE_LEADER_REINFORCEMENT_BONUS_MIN,
    );
    let leader_max_health_bonus = leader.then_some(
        leader_health_random_double * ZOMBIE_LEADER_MAX_HEALTH_BONUS_RANGE
            + ZOMBIE_LEADER_MAX_HEALTH_BONUS_MIN,
    );

    ZombieFinalizeSpawnOutcome {
        can_pick_up_loot,
        is_baby,
        tried_existing_chicken_jockey: tried_existing_chicken_jockey && existing_chicken_available,
        spawned_new_chicken_jockey,
        can_break_doors,
        halloween_head,
        halloween_head_drop_chance: halloween_head.map(|_| 0.0),
        reinforcement_base_chance: reinforcement_base_random_double
            * ZOMBIE_RANDOM_REINFORCEMENT_BASE_MAX,
        knockback_resistance_bonus: knockback_random_double
            * ZOMBIE_RANDOM_KNOCKBACK_RESISTANCE_MAX,
        follow_range_bonus: (follow_range_bonus > ZOMBIE_FOLLOW_RANGE_BONUS_THRESHOLD)
            .then_some(follow_range_bonus),
        leader_reinforcement_bonus,
        leader_max_health_bonus,
        reset_health_to_max: leader
            && !spawn_reason_conversion
            && !spawn_reason_load_or_dimension_travel,
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlazeAttributes {
    pub attack_damage: f32,
    pub movement_speed: f32,
    pub follow_range: f32,
    pub water_pathfinding_malus: f32,
    pub lava_pathfinding_malus: f32,
    pub fire_neighbor_pathfinding_malus: f32,
    pub fire_pathfinding_malus: f32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlazeAiStep {
    pub delta_y: f32,
    pub needs_sync: bool,
    pub allowed_height_offset: f32,
    pub next_height_offset_change_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlazeAttackState {
    pub attack_step: i32,
    pub attack_time: i32,
    pub last_seen: i32,
    pub charged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlazeAttackAction {
    None,
    Melee {
        cooldown_ticks: i32,
    },
    Charge {
        charge_ticks: i32,
    },
    ShootSmallFireball {
        cooldown_ticks: i32,
        level_event: i32,
    },
    Cooldown {
        cooldown_ticks: i32,
    },
    MoveTowardTarget,
}

pub const BLAZE_ATTACK_DAMAGE: f32 = 6.0;
pub const BLAZE_MOVEMENT_SPEED: f32 = 0.23;
pub const BLAZE_FOLLOW_RANGE: f32 = 48.0;
pub const BLAZE_WATER_PATHFINDING_MALUS: f32 = -1.0;
pub const BLAZE_LAVA_PATHFINDING_MALUS: f32 = 8.0;
pub const BLAZE_FIRE_PATHFINDING_MALUS: f32 = 0.0;
pub const BLAZE_XP_REWARD: i32 = 10;
pub const BLAZE_DEFAULT_ALLOWED_HEIGHT_OFFSET: f32 = 0.5;
pub const BLAZE_HEIGHT_OFFSET_REFRESH_TICKS: i32 = 100;
pub const BLAZE_FALL_DAMPING: f32 = 0.6;
pub const BLAZE_ASCEND_TARGET_Y_SPEED: f32 = 0.3;
pub const BLAZE_ASCEND_ACCELERATION: f32 = 0.3;
pub const BLAZE_MELEE_RANGE_SQR: f32 = 4.0;
pub const BLAZE_MELEE_COOLDOWN_TICKS: i32 = 20;
pub const BLAZE_CHARGE_TICKS: i32 = 60;
pub const BLAZE_FIREBALL_BURST_COOLDOWN_TICKS: i32 = 6;
pub const BLAZE_ATTACK_CYCLE_COOLDOWN_TICKS: i32 = 100;
pub const BLAZE_LOST_SIGHT_CHASE_TICKS: i32 = 5;
pub const BLAZE_SHOOT_LEVEL_EVENT: i32 = 1018;
pub const BLAZE_FIREBALL_INACCURACY: f32 = 2.297;
pub const BLAZE_FIREBALL_SPREAD_SCALE: f32 = 0.5;
pub const BLAZE_LOOT_ITEM: &str = "minecraft:blaze_rod";

pub fn blaze_attributes() -> BlazeAttributes {
    BlazeAttributes {
        attack_damage: BLAZE_ATTACK_DAMAGE,
        movement_speed: BLAZE_MOVEMENT_SPEED,
        follow_range: BLAZE_FOLLOW_RANGE,
        water_pathfinding_malus: BLAZE_WATER_PATHFINDING_MALUS,
        lava_pathfinding_malus: BLAZE_LAVA_PATHFINDING_MALUS,
        fire_neighbor_pathfinding_malus: BLAZE_FIRE_PATHFINDING_MALUS,
        fire_pathfinding_malus: BLAZE_FIRE_PATHFINDING_MALUS,
        xp_reward: BLAZE_XP_REWARD,
    }
}

pub fn blaze_ai_step(
    on_ground: bool,
    delta_y: f32,
    next_height_offset_change_tick: i32,
    sampled_allowed_height_offset: f32,
    target_present: bool,
    target_eye_y_above_self_eye_y: f32,
    can_attack_target: bool,
) -> BlazeAiStep {
    let mut next_delta_y = if !on_ground && delta_y < 0.0 {
        delta_y * BLAZE_FALL_DAMPING
    } else {
        delta_y
    };
    let mut next_tick = next_height_offset_change_tick - 1;
    let mut allowed_height_offset = BLAZE_DEFAULT_ALLOWED_HEIGHT_OFFSET;
    if next_tick <= 0 {
        next_tick = BLAZE_HEIGHT_OFFSET_REFRESH_TICKS;
        allowed_height_offset = sampled_allowed_height_offset;
    }

    let should_ascend = target_present
        && target_eye_y_above_self_eye_y > allowed_height_offset
        && can_attack_target;
    if should_ascend {
        next_delta_y += (BLAZE_ASCEND_TARGET_Y_SPEED - next_delta_y) * BLAZE_ASCEND_ACCELERATION;
    }

    BlazeAiStep {
        delta_y: next_delta_y,
        needs_sync: should_ascend,
        allowed_height_offset,
        next_height_offset_change_tick: next_tick,
    }
}

pub fn blaze_attack_goal_can_use(
    target_present: bool,
    target_alive: bool,
    can_attack: bool,
) -> bool {
    target_present && target_alive && can_attack
}

pub fn blaze_attack_goal_start() -> BlazeAttackState {
    BlazeAttackState {
        attack_step: 0,
        attack_time: 0,
        last_seen: 0,
        charged: false,
    }
}

pub fn blaze_attack_goal_stop(mut state: BlazeAttackState) -> BlazeAttackState {
    state.charged = false;
    state.last_seen = 0;
    state
}

pub fn blaze_attack_goal_tick(
    mut state: BlazeAttackState,
    target_present: bool,
    has_line_of_sight: bool,
    distance_sqr: f32,
    follow_range: f32,
    silent: bool,
) -> (BlazeAttackState, BlazeAttackAction) {
    state.attack_time -= 1;
    if !target_present {
        return (state, BlazeAttackAction::None);
    }

    if has_line_of_sight {
        state.last_seen = 0;
    } else {
        state.last_seen += 1;
    }

    if distance_sqr < BLAZE_MELEE_RANGE_SQR {
        if !has_line_of_sight {
            return (state, BlazeAttackAction::None);
        }
        if state.attack_time <= 0 {
            state.attack_time = BLAZE_MELEE_COOLDOWN_TICKS;
            return (
                state,
                BlazeAttackAction::Melee {
                    cooldown_ticks: BLAZE_MELEE_COOLDOWN_TICKS,
                },
            );
        }
        return (state, BlazeAttackAction::MoveTowardTarget);
    }

    if distance_sqr < follow_range * follow_range && has_line_of_sight {
        if state.attack_time <= 0 {
            state.attack_step += 1;
            if state.attack_step == 1 {
                state.attack_time = BLAZE_CHARGE_TICKS;
                state.charged = true;
                return (
                    state,
                    BlazeAttackAction::Charge {
                        charge_ticks: BLAZE_CHARGE_TICKS,
                    },
                );
            }
            if state.attack_step <= 4 {
                state.attack_time = BLAZE_FIREBALL_BURST_COOLDOWN_TICKS;
                return (
                    state,
                    BlazeAttackAction::ShootSmallFireball {
                        cooldown_ticks: BLAZE_FIREBALL_BURST_COOLDOWN_TICKS,
                        level_event: if silent { 0 } else { BLAZE_SHOOT_LEVEL_EVENT },
                    },
                );
            }

            state.attack_time = BLAZE_ATTACK_CYCLE_COOLDOWN_TICKS;
            state.attack_step = 0;
            state.charged = false;
            return (
                state,
                BlazeAttackAction::Cooldown {
                    cooldown_ticks: BLAZE_ATTACK_CYCLE_COOLDOWN_TICKS,
                },
            );
        }
        return (state, BlazeAttackAction::None);
    }

    if state.last_seen < BLAZE_LOST_SIGHT_CHASE_TICKS {
        return (state, BlazeAttackAction::MoveTowardTarget);
    }

    (state, BlazeAttackAction::None)
}

pub fn blaze_fireball_spread(distance_sqr: f32) -> f32 {
    distance_sqr.sqrt().sqrt() * BLAZE_FIREBALL_SPREAD_SCALE
}

pub fn blaze_is_on_fire(charged: bool) -> bool {
    charged
}

pub fn blaze_is_sensitive_to_water() -> bool {
    true
}

pub fn blaze_light_level_dependent_magic_value() -> f32 {
    1.0
}

pub fn blaze_loot_roll(
    killed_by_player: bool,
    base_roll_0_or_1: i32,
    looting_roll_0_to_level: i32,
) -> i32 {
    if killed_by_player {
        base_roll_0_or_1.clamp(0, 1) + looting_roll_0_to_level.max(0)
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombifiedPiglinAttributes {
    pub follow_range: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub armor: f32,
    pub spawn_reinforcements_chance: f32,
    pub attacking_speed_modifier: f32,
    pub lava_pathfinding_malus: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZombifiedPiglinBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZombifiedPiglinAiStep {
    pub has_attacking_speed_modifier: bool,
    pub play_first_anger_sound_in: i32,
    pub played_first_anger_sound: bool,
    pub ticks_until_next_alert: i32,
    pub alert_others: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZombifiedPiglinPortalSpawn {
    pub spawn: bool,
    pub spawn_pos_above_portal_floor: bool,
    pub set_entity_portal_cooldown: bool,
    pub set_vehicle_portal_cooldown: bool,
}

pub const ZOMBIFIED_PIGLIN_ATTACK_DAMAGE: f32 = 5.0;
pub const ZOMBIFIED_PIGLIN_SPAWN_REINFORCEMENTS_CHANCE: f32 = 0.0;
pub const ZOMBIFIED_PIGLIN_ATTACKING_SPEED_MODIFIER: f32 = 0.05;
pub const ZOMBIFIED_PIGLIN_LAVA_PATHFINDING_MALUS: f32 = 8.0;
pub const ZOMBIFIED_PIGLIN_BABY_WIDTH: f32 = 0.49;
pub const ZOMBIFIED_PIGLIN_BABY_HEIGHT: f32 = 0.99;
pub const ZOMBIFIED_PIGLIN_BABY_EYE_HEIGHT: f32 = 0.78;
pub const ZOMBIFIED_PIGLIN_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MIN_TICKS: i32 = 0;
pub const ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MAX_TICKS: i32 = 20;
pub const ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MIN_TICKS: i32 = 20 * 20;
pub const ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MAX_TICKS: i32 = 39 * 20;
pub const ZOMBIFIED_PIGLIN_ALERT_RANGE_Y: f32 = 10.0;
pub const ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MIN_TICKS: i32 = 4 * 20;
pub const ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MAX_TICKS: i32 = 6 * 20;
pub const ZOMBIFIED_PIGLIN_PORTAL_SPAWN_RANDOM_BOUND: i32 = 2_000;

pub fn zombified_piglin_attributes() -> ZombifiedPiglinAttributes {
    ZombifiedPiglinAttributes {
        follow_range: ZOMBIE_FOLLOW_RANGE,
        movement_speed: ZOMBIE_MOVEMENT_SPEED,
        attack_damage: ZOMBIFIED_PIGLIN_ATTACK_DAMAGE,
        armor: ZOMBIE_ARMOR,
        spawn_reinforcements_chance: ZOMBIFIED_PIGLIN_SPAWN_REINFORCEMENTS_CHANCE,
        attacking_speed_modifier: ZOMBIFIED_PIGLIN_ATTACKING_SPEED_MODIFIER,
        lava_pathfinding_malus: ZOMBIFIED_PIGLIN_LAVA_PATHFINDING_MALUS,
    }
}

pub fn zombified_piglin_baby_dimensions() -> ZombifiedPiglinBabyDimensions {
    ZombifiedPiglinBabyDimensions {
        width: ZOMBIFIED_PIGLIN_BABY_WIDTH,
        height: ZOMBIFIED_PIGLIN_BABY_HEIGHT,
        eye_height: ZOMBIFIED_PIGLIN_BABY_EYE_HEIGHT,
        vehicle_attachment_y: ZOMBIFIED_PIGLIN_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn zombified_piglin_start_persistent_anger_time(random_ticks_20_to_39_seconds: i32) -> i32 {
    random_ticks_20_to_39_seconds.clamp(
        ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MIN_TICKS,
        ZOMBIFIED_PIGLIN_PERSISTENT_ANGER_MAX_TICKS,
    )
}

pub fn zombified_piglin_set_target_delays(
    had_target: bool,
    new_target_present: bool,
    first_anger_sound_delay: i32,
    alert_interval: i32,
) -> Option<(i32, i32)> {
    (!had_target && new_target_present).then_some((
        first_anger_sound_delay.clamp(
            ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MIN_TICKS,
            ZOMBIFIED_PIGLIN_FIRST_ANGER_SOUND_DELAY_MAX_TICKS,
        ),
        alert_interval.clamp(
            ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MIN_TICKS,
            ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MAX_TICKS,
        ),
    ))
}

pub fn zombified_piglin_ai_step(
    angry: bool,
    baby: bool,
    has_attacking_speed_modifier: bool,
    play_first_anger_sound_in: i32,
    target_present: bool,
    ticks_until_next_alert: i32,
    has_line_of_sight_to_target: bool,
    sampled_next_alert_interval: i32,
) -> ZombifiedPiglinAiStep {
    let next_modifier = angry && (!baby || has_attacking_speed_modifier);
    let mut next_sound = play_first_anger_sound_in;
    let mut played_first_anger_sound = false;
    if angry && next_sound > 0 {
        next_sound -= 1;
        played_first_anger_sound = next_sound == 0;
    }

    let mut next_alert = ticks_until_next_alert;
    let mut alert_others = false;
    if target_present {
        if next_alert > 0 {
            next_alert -= 1;
        } else {
            alert_others = has_line_of_sight_to_target;
            next_alert = sampled_next_alert_interval.clamp(
                ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MIN_TICKS,
                ZOMBIFIED_PIGLIN_ALERT_INTERVAL_MAX_TICKS,
            );
        }
    }

    ZombifiedPiglinAiStep {
        has_attacking_speed_modifier: next_modifier,
        play_first_anger_sound_in: next_sound,
        played_first_anger_sound,
        ticks_until_next_alert: next_alert,
        alert_others,
    }
}

pub fn zombified_piglin_alerts_other(
    same_entity: bool,
    other_has_target: bool,
    allied_to_target: bool,
    no_spectators_filter: bool,
) -> bool {
    !same_entity && !other_has_target && !allied_to_target && no_spectators_filter
}

pub fn zombified_piglin_spawn_allowed(
    difficulty_peaceful: bool,
    below_is_nether_wart_block: bool,
) -> bool {
    !difficulty_peaceful && !below_is_nether_wart_block
}

pub fn zombified_piglin_spawn_obstruction(unobstructed: bool, contains_liquid: bool) -> bool {
    unobstructed && !contains_liquid
}

pub fn zombified_piglin_default_main_hand_item(random_0_to_19: i32) -> &'static str {
    if random_0_to_19.rem_euclid(20) == 0 {
        "minecraft:golden_spear"
    } else {
        "minecraft:golden_sword"
    }
}

pub fn zombified_piglin_portal_spawn(
    spawning_monsters: bool,
    portal_spawns_piglins: bool,
    random_0_to_1999: i32,
    difficulty_id: i32,
    player_close_enough_for_spawning: bool,
    portal_floor_valid_spawn: bool,
    spawned_entity_has_vehicle: bool,
) -> ZombifiedPiglinPortalSpawn {
    let spawn = spawning_monsters
        && portal_spawns_piglins
        && random_0_to_1999.rem_euclid(ZOMBIFIED_PIGLIN_PORTAL_SPAWN_RANDOM_BOUND) < difficulty_id
        && player_close_enough_for_spawning
        && portal_floor_valid_spawn;
    ZombifiedPiglinPortalSpawn {
        spawn,
        spawn_pos_above_portal_floor: spawn,
        set_entity_portal_cooldown: spawn,
        set_vehicle_portal_cooldown: spawn && spawned_entity_has_vehicle,
    }
}

pub fn zombified_piglin_prevents_player_rest(angry_at_player: bool) -> bool {
    angry_at_player
}

pub fn zombified_piglin_wants_to_pick_up(can_hold_item: bool) -> bool {
    can_hold_item
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedAttributes {
    pub follow_range: f32,
    pub movement_speed: f32,
    pub attack_damage: f32,
    pub armor: f32,
    pub step_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
    pub amphibious_navigation: bool,
    pub water_pathfinding_malus: f32,
    pub can_spawn_in_liquids: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrownedFinalizeSpawnOutcome {
    pub offhand_nautilus_shell: bool,
    pub guaranteed_offhand_drop: bool,
    pub spawned_zombie_nautilus_jockey: bool,
    pub zombie_nautilus_persistent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrownedTridentShot {
    pub item: &'static str,
    pub power: f32,
    pub inaccuracy: i32,
    pub y_lead_scale: f32,
    pub sound: &'static str,
}

pub const DROWNED_FOLLOW_RANGE: f32 = 35.0;
pub const DROWNED_MOVEMENT_SPEED: f32 = 0.23;
pub const DROWNED_ATTACK_DAMAGE: f32 = 3.0;
pub const DROWNED_ARMOR: f32 = 2.0;
pub const DROWNED_STEP_HEIGHT: f32 = 1.0;
pub const DROWNED_WIDTH: f32 = 0.6;
pub const DROWNED_HEIGHT: f32 = 1.95;
pub const DROWNED_EYE_HEIGHT: f32 = 1.74;
pub const DROWNED_PASSENGER_ATTACHMENT_Y: f32 = 2.0125;
pub const DROWNED_RIDING_OFFSET: f32 = -0.7;
pub const DROWNED_CLIENT_TRACKING_RANGE: i32 = 8;
pub const DROWNED_NOT_IN_PEACEFUL: bool = true;
pub const DROWNED_BABY_WIDTH: f32 = 0.49;
pub const DROWNED_BABY_HEIGHT: f32 = 0.99;
pub const DROWNED_BABY_EYE_HEIGHT: f32 = 0.775;
pub const DROWNED_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const DROWNED_NAUTILUS_SHELL_CHANCE: f32 = 0.03;
pub const DROWNED_ZOMBIE_NAUTILUS_JOCKEY_CHANCE: f32 = 0.5;
pub const DROWNED_DEFAULT_WATER_PATHFINDING_MALUS: f32 = 0.0;
pub const DROWNED_TRIDENT_ATTACK_INTERVAL_TICKS: i32 = 40;
pub const DROWNED_TRIDENT_ATTACK_RADIUS: f32 = 10.0;
pub const DROWNED_TRIDENT_POWER: f32 = 1.6;
pub const DROWNED_TRIDENT_Y_LEAD_SCALE: f32 = 0.2;
pub const DROWNED_EQUIPMENT_ROLL_THRESHOLD: f32 = 0.9;
pub const DROWNED_EQUIPMENT_RANDOM_BOUND: i32 = 16;
pub const DROWNED_TRIDENT_RANDOM_CUTOFF: i32 = 10;
pub const DROWNED_MORE_FREQUENT_SPAWN_RANDOM_BOUND: i32 = 15;
pub const DROWNED_DEFAULT_SPAWN_RANDOM_BOUND: i32 = 40;
pub const DROWNED_DEEP_SPAWN_SEA_LEVEL_OFFSET: i32 = 5;
pub const DROWNED_SWIM_UP_SEA_LEVEL_OFFSET: i32 = 2;
pub const DROWNED_GO_TO_BEACH_SEA_LEVEL_OFFSET: i32 = 3;
pub const DROWNED_WATER_SEARCH_ATTEMPTS: i32 = 10;
pub const DROWNED_WATER_SEARCH_XZ_RANGE: i32 = 10;
pub const DROWNED_WATER_SEARCH_Y_UP: i32 = 2;
pub const DROWNED_WATER_SEARCH_Y_DOWN: i32 = 5;

pub fn drowned_attributes() -> DrownedAttributes {
    DrownedAttributes {
        follow_range: DROWNED_FOLLOW_RANGE,
        movement_speed: DROWNED_MOVEMENT_SPEED,
        attack_damage: DROWNED_ATTACK_DAMAGE,
        armor: DROWNED_ARMOR,
        step_height: DROWNED_STEP_HEIGHT,
    }
}

pub fn drowned_entity_type_surface() -> DrownedEntityTypeSurface {
    DrownedEntityTypeSurface {
        width: DROWNED_WIDTH,
        height: DROWNED_HEIGHT,
        eye_height: DROWNED_EYE_HEIGHT,
        passenger_attachment_y: DROWNED_PASSENGER_ATTACHMENT_Y,
        riding_offset: DROWNED_RIDING_OFFSET,
        client_tracking_range: DROWNED_CLIENT_TRACKING_RANGE,
        not_in_peaceful: DROWNED_NOT_IN_PEACEFUL,
        amphibious_navigation: true,
        water_pathfinding_malus: DROWNED_DEFAULT_WATER_PATHFINDING_MALUS,
        can_spawn_in_liquids: true,
    }
}

pub fn drowned_baby_dimensions() -> DrownedBabyDimensions {
    DrownedBabyDimensions {
        width: DROWNED_BABY_WIDTH,
        height: DROWNED_BABY_HEIGHT,
        eye_height: DROWNED_BABY_EYE_HEIGHT,
        vehicle_attachment_y: DROWNED_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn drowned_spawn_allowed(
    below_is_water: bool,
    pos_is_water: bool,
    spawner_reason: bool,
    reinforcement_reason: bool,
    ignores_light_requirements: bool,
    difficulty_peaceful: bool,
    dark_enough_to_spawn: bool,
    more_frequent_drowned_biome: bool,
    random_roll: i32,
    y: i32,
    sea_level: i32,
) -> bool {
    if !below_is_water && !spawner_reason {
        return false;
    }

    let can_monster_spawn = !difficulty_peaceful
        && (ignores_light_requirements || dark_enough_to_spawn)
        && (spawner_reason || pos_is_water);
    if !can_monster_spawn {
        return false;
    }

    if spawner_reason || reinforcement_reason {
        return true;
    }

    if more_frequent_drowned_biome {
        random_roll.rem_euclid(DROWNED_MORE_FREQUENT_SPAWN_RANDOM_BOUND) == 0
    } else {
        random_roll.rem_euclid(DROWNED_DEFAULT_SPAWN_RANDOM_BOUND) == 0
            && y < sea_level - DROWNED_DEEP_SPAWN_SEA_LEVEL_OFFSET
    }
}

pub fn drowned_default_main_hand_item(
    random_float: f32,
    random_0_to_15: i32,
) -> Option<&'static str> {
    if random_float <= DROWNED_EQUIPMENT_ROLL_THRESHOLD {
        return None;
    }
    if random_0_to_15.rem_euclid(DROWNED_EQUIPMENT_RANDOM_BOUND) < DROWNED_TRIDENT_RANDOM_CUTOFF {
        Some("minecraft:trident")
    } else {
        Some("minecraft:fishing_rod")
    }
}

pub fn drowned_finalize_spawn_outcome(
    offhand_empty: bool,
    nautilus_random_float: f32,
    natural_or_structure_spawn: bool,
    structure_spawn: bool,
    main_hand_trident: bool,
    zombie_nautilus_random_float: f32,
    baby: bool,
    more_frequent_drowned_biome: bool,
) -> DrownedFinalizeSpawnOutcome {
    let offhand_nautilus_shell =
        offhand_empty && nautilus_random_float < DROWNED_NAUTILUS_SHELL_CHANCE;
    let spawned_zombie_nautilus_jockey = natural_or_structure_spawn
        && main_hand_trident
        && zombie_nautilus_random_float < DROWNED_ZOMBIE_NAUTILUS_JOCKEY_CHANCE
        && !baby
        && !more_frequent_drowned_biome;
    DrownedFinalizeSpawnOutcome {
        offhand_nautilus_shell,
        guaranteed_offhand_drop: offhand_nautilus_shell,
        spawned_zombie_nautilus_jockey,
        zombie_nautilus_persistent: spawned_zombie_nautilus_jockey && structure_spawn,
    }
}

pub fn drowned_can_replace_current_item(current_item: &str) -> bool {
    current_item != "minecraft:nautilus_shell"
}

pub fn drowned_wants_to_pick_up(item: &str) -> bool {
    item != "minecraft:trident" && item != "minecraft:iron_spear"
}

pub fn drowned_ok_target(
    target_present: bool,
    bright_outside: bool,
    target_in_water: bool,
) -> bool {
    target_present && (!bright_outside || target_in_water)
}

pub fn drowned_wants_to_swim(
    searching_for_land: bool,
    target_present: bool,
    target_in_water: bool,
) -> bool {
    searching_for_land || (target_present && target_in_water)
}

pub fn drowned_should_update_swimming(
    effective_ai: bool,
    underwater: bool,
    searching_for_land: bool,
    target_present: bool,
    target_in_water: bool,
) -> bool {
    effective_ai
        && underwater
        && drowned_wants_to_swim(searching_for_land, target_present, target_in_water)
}

pub fn drowned_trident_attack_can_use(ranged_goal_can_use: bool, main_hand_trident: bool) -> bool {
    ranged_goal_can_use && main_hand_trident
}

pub fn drowned_trident_shot(difficulty_id: i32, main_hand_trident: bool) -> DrownedTridentShot {
    let _uses_existing_trident_stack = main_hand_trident;
    DrownedTridentShot {
        item: "minecraft:trident",
        power: DROWNED_TRIDENT_POWER,
        inaccuracy: 14 - difficulty_id * 4,
        y_lead_scale: DROWNED_TRIDENT_Y_LEAD_SCALE,
        sound: "minecraft:entity.drowned.shoot",
    }
}

pub fn drowned_go_to_water_goal_can_use(
    bright_outside: bool,
    in_water: bool,
    found_water_pos: bool,
) -> bool {
    bright_outside && !in_water && found_water_pos
}

pub fn drowned_go_to_beach_goal_can_use(
    move_to_block_can_use: bool,
    bright_outside: bool,
    in_water: bool,
    y: i32,
    sea_level: i32,
) -> bool {
    move_to_block_can_use
        && !bright_outside
        && in_water
        && y >= sea_level - DROWNED_GO_TO_BEACH_SEA_LEVEL_OFFSET
}

pub fn drowned_swim_up_goal_can_use(
    bright_outside: bool,
    in_water: bool,
    y: i32,
    sea_level: i32,
) -> bool {
    !bright_outside && in_water && y < sea_level - DROWNED_SWIM_UP_SEA_LEVEL_OFFSET
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HuskEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub riding_offset: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HuskBabyDimensions {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub vehicle_attachment_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HuskFinalizeSpawnOutcome {
    pub can_pick_up_loot: Option<bool>,
    pub tried_to_spawn_camel_husk: bool,
    pub spawned_camel_husk: bool,
    pub spawned_parched_passenger: bool,
    pub equipped_iron_spear: bool,
}

pub const HUSK_WIDTH: f32 = 0.6;
pub const HUSK_HEIGHT: f32 = 1.95;
pub const HUSK_EYE_HEIGHT: f32 = 1.74;
pub const HUSK_PASSENGER_ATTACHMENT_Y: f32 = 2.075;
pub const HUSK_RIDING_OFFSET: f32 = -0.7;
pub const HUSK_CLIENT_TRACKING_RANGE: i32 = 8;
pub const HUSK_NOT_IN_PEACEFUL: bool = true;
pub const HUSK_BABY_WIDTH: f32 = 0.49;
pub const HUSK_BABY_HEIGHT: f32 = 0.99;
pub const HUSK_BABY_EYE_HEIGHT: f32 = 0.825;
pub const HUSK_BABY_VEHICLE_ATTACHMENT_Y: f32 = 0.1875;
pub const HUSK_HUNGER_EFFECT_ID: &str = "minecraft:hunger";
pub const HUSK_HUNGER_DURATION_SCALE_TICKS: i32 = 140;
pub const HUSK_HUNGER_AMPLIFIER: u8 = 0;
pub const HUSK_CONVERTS_IN_WATER: bool = true;
pub const HUSK_UNDERWATER_CONVERSION_TARGET: &str = "minecraft:zombie";
pub const HUSK_UNDERWATER_CONVERSION_LEVEL_EVENT: i32 = 1041;
pub const HUSK_LOOT_PICKUP_CHANCE_SCALE: f32 = 0.55;
pub const HUSK_CAMEL_HUSK_SPAWN_CHANCE: f32 = 0.1;
pub const HUSK_CAMEL_HUSK_RIDER_ITEM: &str = "minecraft:iron_spear";

pub fn husk_entity_type_surface() -> HuskEntityTypeSurface {
    HuskEntityTypeSurface {
        width: HUSK_WIDTH,
        height: HUSK_HEIGHT,
        eye_height: HUSK_EYE_HEIGHT,
        passenger_attachment_y: HUSK_PASSENGER_ATTACHMENT_Y,
        riding_offset: HUSK_RIDING_OFFSET,
        client_tracking_range: HUSK_CLIENT_TRACKING_RANGE,
        not_in_peaceful: HUSK_NOT_IN_PEACEFUL,
    }
}

pub fn husk_baby_dimensions() -> HuskBabyDimensions {
    HuskBabyDimensions {
        width: HUSK_BABY_WIDTH,
        height: HUSK_BABY_HEIGHT,
        eye_height: HUSK_BABY_EYE_HEIGHT,
        vehicle_attachment_y: HUSK_BABY_VEHICLE_ATTACHMENT_Y,
    }
}

pub fn husk_is_sun_sensitive() -> bool {
    false
}

pub fn husk_hunger_duration_ticks(
    effective_difficulty: f32,
    super_hurt_succeeded: bool,
    main_hand_empty: bool,
    target_is_living: bool,
) -> Option<i32> {
    (super_hurt_succeeded && main_hand_empty && target_is_living)
        .then_some(HUSK_HUNGER_DURATION_SCALE_TICKS * effective_difficulty as i32)
}

pub fn husk_should_pick_up_loot(
    spawn_reason_conversion: bool,
    random_float: f32,
    difficulty_special_multiplier: f32,
) -> Option<bool> {
    (!spawn_reason_conversion)
        .then_some(random_float < HUSK_LOOT_PICKUP_CHANCE_SCALE * difficulty_special_multiplier)
}

pub fn husk_underwater_conversion_event(silent: bool) -> Option<i32> {
    (!silent).then_some(HUSK_UNDERWATER_CONVERSION_LEVEL_EVENT)
}

pub fn husk_finalize_spawn_outcome(
    spawn_reason_natural: bool,
    spawn_reason_conversion: bool,
    random_loot_float: f32,
    difficulty_special_multiplier: f32,
    camel_husk_collision_free: bool,
    camel_husk_random_float: f32,
) -> HuskFinalizeSpawnOutcome {
    let mut tried_to_spawn_camel_husk = !spawn_reason_natural;
    let mut spawned_camel_husk = false;

    if !tried_to_spawn_camel_husk && camel_husk_collision_free {
        tried_to_spawn_camel_husk = true;
        spawned_camel_husk = camel_husk_random_float < HUSK_CAMEL_HUSK_SPAWN_CHANCE;
    }

    HuskFinalizeSpawnOutcome {
        can_pick_up_loot: husk_should_pick_up_loot(
            spawn_reason_conversion,
            random_loot_float,
            difficulty_special_multiplier,
        ),
        tried_to_spawn_camel_husk,
        spawned_camel_husk,
        spawned_parched_passenger: spawned_camel_husk,
        equipped_iron_spear: spawned_camel_husk,
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermanAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
    pub attacking_speed_bonus: f32,
    pub attack_damage: f32,
    pub follow_range: f32,
    pub step_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermanEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub passenger_attachment_y: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndermanTargetState {
    pub target_change_time: i32,
    pub creepy: bool,
    pub stared_at: bool,
    pub speed_modifier_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndermanHurtResponse {
    NormalHurt,
    NormalHurtAndMaybeTeleport,
    WaterPotionHurtAndTryTeleport64,
    ProjectileTryTeleport64,
}

pub const ENDERMAN_MAX_HEALTH: f32 = 40.0;
pub const ENDERMAN_MOVEMENT_SPEED: f32 = 0.3;
pub const ENDERMAN_ATTACKING_SPEED_BONUS: f32 = 0.15;
pub const ENDERMAN_ATTACK_DAMAGE: f32 = 7.0;
pub const ENDERMAN_FOLLOW_RANGE: f32 = 64.0;
pub const ENDERMAN_STEP_HEIGHT: f32 = 1.0;
pub const ENDERMAN_WIDTH: f32 = 0.6;
pub const ENDERMAN_HEIGHT: f32 = 2.9;
pub const ENDERMAN_EYE_HEIGHT: f32 = 2.55;
pub const ENDERMAN_PASSENGER_ATTACHMENT_Y: f32 = 2.80625;
pub const ENDERMAN_CLIENT_TRACKING_RANGE: i32 = 8;
pub const ENDERMAN_WATER_PATHFINDING_MALUS: f32 = -1.0;
pub const ENDERMAN_STARE_SOUND_COOLDOWN: i32 = 400;
pub const ENDERMAN_MIN_DEAGGRESSION_TIME: i32 = 600;
pub const ENDERMAN_PERSISTENT_ANGER_MIN_SECONDS: i32 = 20;
pub const ENDERMAN_PERSISTENT_ANGER_MAX_SECONDS: i32 = 39;
pub const ENDERMAN_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const ENDERMAN_FREEZE_STARE_DISTANCE_SQR: f32 = 256.0;
pub const ENDERMAN_CLOSE_STARE_TELEPORT_DISTANCE_SQR: f32 = 16.0;
pub const ENDERMAN_FAR_TARGET_TELEPORT_DISTANCE_SQR: f32 = 256.0;
pub const ENDERMAN_FAR_TARGET_TELEPORT_DELAY: i32 = 30;
pub const ENDERMAN_AGGRO_TIME: i32 = 5;
pub const ENDERMAN_STARE_DOT_THRESHOLD: f64 = 0.025;
pub const ENDERMAN_RANDOM_TELEPORT_HORIZONTAL_RANGE: f64 = 64.0;
pub const ENDERMAN_RANDOM_TELEPORT_VERTICAL_RANGE: i32 = 64;
pub const ENDERMAN_TELEPORT_TOWARDS_DISTANCE: f64 = 16.0;
pub const ENDERMAN_TELEPORT_TOWARDS_RANDOM_HORIZONTAL: f64 = 8.0;
pub const ENDERMAN_TELEPORT_TOWARDS_RANDOM_VERTICAL: i32 = 16;
pub const ENDERMAN_PROJECTILE_TELEPORT_ATTEMPTS: i32 = 64;
pub const ENDERMAN_NON_LIVING_DAMAGE_TELEPORT_ROLL: i32 = 10;
pub const ENDERMAN_TAKE_BLOCK_ROLL: i32 = 20;
pub const ENDERMAN_LEAVE_BLOCK_ROLL: i32 = 2000;

pub fn enderman_attributes() -> EndermanAttributes {
    EndermanAttributes {
        max_health: ENDERMAN_MAX_HEALTH,
        movement_speed: ENDERMAN_MOVEMENT_SPEED,
        attacking_speed_bonus: ENDERMAN_ATTACKING_SPEED_BONUS,
        attack_damage: ENDERMAN_ATTACK_DAMAGE,
        follow_range: ENDERMAN_FOLLOW_RANGE,
        step_height: ENDERMAN_STEP_HEIGHT,
    }
}

pub fn enderman_entity_type_surface() -> EndermanEntityTypeSurface {
    EndermanEntityTypeSurface {
        width: ENDERMAN_WIDTH,
        height: ENDERMAN_HEIGHT,
        eye_height: ENDERMAN_EYE_HEIGHT,
        passenger_attachment_y: ENDERMAN_PASSENGER_ATTACHMENT_Y,
        client_tracking_range: ENDERMAN_CLIENT_TRACKING_RANGE,
        not_in_peaceful: true,
    }
}

pub fn enderman_set_target_state(target_present: bool, tick_count: i32) -> EndermanTargetState {
    if target_present {
        EndermanTargetState {
            target_change_time: tick_count,
            creepy: true,
            stared_at: false,
            speed_modifier_present: true,
        }
    } else {
        EndermanTargetState {
            target_change_time: 0,
            creepy: false,
            stared_at: false,
            speed_modifier_present: false,
        }
    }
}

pub fn enderman_stare_sound_allowed(tick_count: i32, last_stare_sound: i32) -> bool {
    tick_count >= last_stare_sound + ENDERMAN_STARE_SOUND_COOLDOWN
}

pub fn enderman_should_daylight_deaggro_and_teleport(
    bright_outside: bool,
    tick_count: i32,
    target_change_time: i32,
    light_magic: f32,
    can_see_sky: bool,
    random_float_0_to_1: f32,
) -> bool {
    bright_outside
        && tick_count >= target_change_time + ENDERMAN_MIN_DEAGGRESSION_TIME
        && light_magic > 0.5
        && can_see_sky
        && random_float_0_to_1 * 30.0 < (light_magic - 0.4) * 2.0
}

pub fn enderman_hurt_response(
    projectile_damage: bool,
    thrown_potion: bool,
    potion_is_water: bool,
    source_entity_is_living: bool,
    random_0_to_9: i32,
) -> EndermanHurtResponse {
    if projectile_damage || thrown_potion {
        if thrown_potion && potion_is_water {
            EndermanHurtResponse::WaterPotionHurtAndTryTeleport64
        } else {
            EndermanHurtResponse::ProjectileTryTeleport64
        }
    } else if !source_entity_is_living && random_0_to_9.rem_euclid(10) != 0 {
        EndermanHurtResponse::NormalHurtAndMaybeTeleport
    } else {
        EndermanHurtResponse::NormalHurt
    }
}

pub fn enderman_freeze_when_looked_at(
    player_target: bool,
    distance_sqr: f32,
    stared_by_player: bool,
) -> bool {
    player_target && distance_sqr <= ENDERMAN_FREEZE_STARE_DISTANCE_SQR && stared_by_player
}

pub fn enderman_look_goal_starts_aggro(pending_target_present: bool) -> Option<i32> {
    pending_target_present.then_some(ENDERMAN_AGGRO_TIME)
}

pub fn enderman_look_goal_tick(
    pending_aggro_time: Option<i32>,
    target_present: bool,
    being_stared_by_target: bool,
    target_distance_sqr: f32,
    teleport_time: i32,
    is_passenger: bool,
) -> (Option<i32>, bool, i32, bool) {
    if let Some(aggro_time) = pending_aggro_time {
        let next = aggro_time - 1;
        return (Some(next), next <= 0, teleport_time, false);
    }
    if !target_present || is_passenger {
        return (None, false, teleport_time, false);
    }
    if being_stared_by_target {
        return (
            None,
            false,
            0,
            target_distance_sqr < ENDERMAN_CLOSE_STARE_TELEPORT_DISTANCE_SQR,
        );
    }
    let next_teleport_time = if target_distance_sqr > ENDERMAN_FAR_TARGET_TELEPORT_DISTANCE_SQR {
        teleport_time + 1
    } else {
        teleport_time
    };
    (
        None,
        false,
        next_teleport_time,
        target_distance_sqr > ENDERMAN_FAR_TARGET_TELEPORT_DISTANCE_SQR
            && teleport_time >= ENDERMAN_FAR_TARGET_TELEPORT_DELAY,
    )
}

pub fn enderman_take_block_can_use(
    carried_block_present: bool,
    mob_griefing: bool,
    random_0_to_19: i32,
) -> bool {
    !carried_block_present
        && mob_griefing
        && random_0_to_19.rem_euclid(ENDERMAN_TAKE_BLOCK_ROLL) == 0
}

pub fn enderman_leave_block_can_use(
    carried_block_present: bool,
    mob_griefing: bool,
    random_0_to_1999: i32,
) -> bool {
    carried_block_present
        && mob_griefing
        && random_0_to_1999.rem_euclid(ENDERMAN_LEAVE_BLOCK_ROLL) == 0
}

pub fn enderman_can_place_carried_block(
    target_is_air: bool,
    below_is_air: bool,
    below_is_bedrock: bool,
    below_full_collision: bool,
    carried_can_survive: bool,
    entity_collision_empty: bool,
) -> bool {
    target_is_air
        && !below_is_air
        && !below_is_bedrock
        && below_full_collision
        && carried_can_survive
        && entity_collision_empty
}

pub fn enderman_requires_custom_persistence(
    super_requires: bool,
    carried_block_present: bool,
) -> bool {
    super_requires || carried_block_present
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveSpiderAttributes {
    pub max_health: f32,
    pub movement_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveSpiderEntityTypeSurface {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
    pub client_tracking_range: i32,
    pub not_in_peaceful: bool,
}

pub const SPIDER_MAX_HEALTH: f32 = 16.0;
pub const SPIDER_MOVEMENT_SPEED: f32 = 0.3;
pub const SPIDER_WIDTH: f32 = 1.4;
pub const SPIDER_HEIGHT: f32 = 0.9;
pub const SPIDER_EYE_HEIGHT: f32 = 0.65;
pub const SPIDER_CLIENT_TRACKING_RANGE: i32 = 8;
pub const SPIDER_CLIMBING_FLAG: u8 = 1;
pub const SPIDER_SPECIAL_EFFECT_CHANCE: f32 = 0.1;
pub const SPIDER_JOCKEY_RANDOM_BOUND: i32 = 100;
pub const SPIDER_JOCKEY_RANDOM_HIT: i32 = 0;
pub const SPIDER_ATTACK_LIGHT_BREAK_THRESHOLD: f32 = 0.5;
pub const SPIDER_ATTACK_LIGHT_BREAK_RANDOM_BOUND: i32 = 100;
pub const SPIDER_VEHICLE_ATTACHMENT_Y: f32 = 0.3125;
pub const SPIDER_POISON_IMMUNE: bool = true;
pub const SPIDER_AVOID_ARMADILLO_DISTANCE: f32 = 6.0;
pub const SPIDER_AVOID_ARMADILLO_WALK_SPEED: f32 = 1.0;
pub const SPIDER_AVOID_ARMADILLO_SPRINT_SPEED: f32 = 1.2;
pub const SPIDER_LEAP_AT_TARGET_POWER: f32 = 0.4;
pub const SPIDER_RANDOM_STROLL_SPEED: f32 = 0.8;
pub const SPIDER_LOOK_AT_PLAYER_RANGE: f32 = 8.0;
pub const CAVE_SPIDER_MAX_HEALTH: f32 = 12.0;
pub const CAVE_SPIDER_WIDTH: f32 = 0.7;
pub const CAVE_SPIDER_HEIGHT: f32 = 0.5;
pub const CAVE_SPIDER_EYE_HEIGHT: f32 = 0.45;
pub const CAVE_SPIDER_CLIENT_TRACKING_RANGE: i32 = 8;
pub const CAVE_SPIDER_NOT_IN_PEACEFUL: bool = true;
pub const CAVE_SPIDER_POISON_SECONDS_NORMAL: i32 = 7;
pub const CAVE_SPIDER_POISON_SECONDS_HARD: i32 = 15;
pub const CAVE_SPIDER_POISON_AMPLIFIER: u8 = 0;
pub const CAVE_SPIDER_VEHICLE_ATTACHMENT_Y: f32 = 0.21875;

pub fn cave_spider_attributes() -> CaveSpiderAttributes {
    CaveSpiderAttributes {
        max_health: CAVE_SPIDER_MAX_HEALTH,
        movement_speed: SPIDER_MOVEMENT_SPEED,
    }
}

pub fn cave_spider_entity_type_surface() -> CaveSpiderEntityTypeSurface {
    CaveSpiderEntityTypeSurface {
        width: CAVE_SPIDER_WIDTH,
        height: CAVE_SPIDER_HEIGHT,
        eye_height: CAVE_SPIDER_EYE_HEIGHT,
        client_tracking_range: CAVE_SPIDER_CLIENT_TRACKING_RANGE,
        not_in_peaceful: CAVE_SPIDER_NOT_IN_PEACEFUL,
    }
}

pub fn cave_spider_poison_duration_ticks(
    difficulty: &str,
    super_hurt_succeeded: bool,
) -> Option<i32> {
    if !super_hurt_succeeded {
        return None;
    }
    match difficulty {
        "normal" => Some(CAVE_SPIDER_POISON_SECONDS_NORMAL * 20),
        "hard" => Some(CAVE_SPIDER_POISON_SECONDS_HARD * 20),
        _ => None,
    }
}

pub fn cave_spider_finalize_spawn_preserves_group_data<T>(group_data: Option<T>) -> Option<T> {
    group_data
}

pub fn cave_spider_vehicle_attachment_y(
    vehicle_width: f32,
    cave_spider_width: f32,
    scale: f32,
) -> Option<f32> {
    (vehicle_width <= cave_spider_width).then_some(CAVE_SPIDER_VEHICLE_ATTACHMENT_Y * scale)
}

pub fn spider_set_climbing_flags(flags: u8, climbing: bool) -> u8 {
    if climbing {
        flags | SPIDER_CLIMBING_FLAG
    } else {
        flags & !SPIDER_CLIMBING_FLAG
    }
}

pub fn spider_is_climbing(flags: u8) -> bool {
    flags & SPIDER_CLIMBING_FLAG != 0
}

pub fn spider_tick_climbing_flags(flags: u8, horizontal_collision: bool) -> u8 {
    spider_set_climbing_flags(flags, horizontal_collision)
}

pub fn spider_attack_goal_can_use(super_can_use: bool, is_vehicle: bool) -> bool {
    super_can_use && !is_vehicle
}

pub fn spider_target_goal_can_use(light_value: f32, super_can_use: bool) -> bool {
    light_value < SPIDER_ATTACK_LIGHT_BREAK_THRESHOLD && super_can_use
}

pub fn spider_avoids_armadillo(armadillo_is_scared: bool) -> bool {
    !armadillo_is_scared
}

pub fn spider_can_be_affected(effect_id: &str) -> bool {
    effect_id != "minecraft:poison"
}

pub fn spider_jockey_from_finalize_spawn(random_0_to_99: i32) -> bool {
    random_0_to_99.rem_euclid(SPIDER_JOCKEY_RANDOM_BOUND) == SPIDER_JOCKEY_RANDOM_HIT
}

pub fn spider_should_drop_target_in_light(light_value: f32, random_0_to_99: i32) -> bool {
    light_value >= SPIDER_ATTACK_LIGHT_BREAK_THRESHOLD
        && random_0_to_99.rem_euclid(SPIDER_ATTACK_LIGHT_BREAK_RANDOM_BOUND) == 0
}

pub fn spider_effect_from_group_data_selection(random_0_to_4: i32) -> &'static str {
    match random_0_to_4.rem_euclid(5) {
        0 | 1 => "minecraft:speed",
        2 => "minecraft:strength",
        3 => "minecraft:regeneration",
        _ => "minecraft:invisibility",
    }
}

pub fn spider_should_roll_special_effect(
    difficulty: &str,
    random_float: f32,
    special_multiplier: f32,
) -> bool {
    difficulty == "hard" && random_float < SPIDER_SPECIAL_EFFECT_CHANCE * special_multiplier
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgeState {
    pub age: i32,
    pub forced_age: i32,
    pub forced_age_timer: i32,
    pub age_locked: bool,
    pub age_lock_particle_timer: i32,
}

impl AgeState {
    pub fn baby() -> Self {
        Self {
            age: BABY_START_AGE,
            forced_age: 0,
            forced_age_timer: 0,
            age_locked: false,
            age_lock_particle_timer: 0,
        }
    }

    pub fn is_baby(self) -> bool {
        self.age < 0
    }

    pub fn can_age_up(self) -> bool {
        self.is_baby() && !self.age_locked
    }

    pub fn tick(mut self) -> Self {
        if self.can_age_up() {
            self.age += 1;
        } else if self.age > 0 {
            self.age -= 1;
        }
        if self.age_lock_particle_timer > 0 {
            self.age_lock_particle_timer -= 1;
        }
        self
    }

    pub fn age_up(mut self, seconds: i32, forced: bool) -> Self {
        let old_age = self.age;
        self.age = (self.age + seconds * 20).min(0);
        let delta = self.age - old_age;
        if forced {
            self.forced_age += delta;
            if self.forced_age_timer == 0 {
                self.forced_age_timer = FORCED_AGE_PARTICLE_TICKS;
            }
        }
        if self.age == 0 {
            self.age = self.forced_age;
        }
        self
    }

    pub fn toggle_age_lock(mut self) -> Self {
        if self.is_baby() {
            self.age_locked = !self.age_locked;
            self.age = BABY_START_AGE;
            self.age_lock_particle_timer = AGE_LOCK_COOLDOWN_TICKS;
        }
        self
    }
}

pub fn speed_up_seconds_when_feeding(ticks_until_adult: i32) -> i32 {
    (ticks_until_adult / 20) / 10
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimalFeedResult {
    SetInLove,
    AgeUp { seconds: i32 },
    ConsumeClientOnly,
    NotFood,
}

pub fn animal_feed_result(
    is_food: bool,
    age: i32,
    in_love: i32,
    can_age_up: bool,
    client: bool,
) -> AnimalFeedResult {
    if !is_food {
        return AnimalFeedResult::NotFood;
    }
    if age == 0 && in_love <= 0 {
        AnimalFeedResult::SetInLove
    } else if can_age_up {
        AnimalFeedResult::AgeUp {
            seconds: speed_up_seconds_when_feeding(-age),
        }
    } else if client {
        AnimalFeedResult::ConsumeClientOnly
    } else {
        AnimalFeedResult::NotFood
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoveState {
    pub in_love: i32,
    pub love_cause_present: bool,
}

impl LoveState {
    pub fn set_in_love(player_present: bool) -> Self {
        Self {
            in_love: IN_LOVE_TICKS,
            love_cause_present: player_present,
        }
    }

    pub fn tick(mut self, age: i32) -> Self {
        if age != 0 {
            self.in_love = 0;
        } else if self.in_love > 0 {
            self.in_love -= 1;
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreedingResult {
    pub parent_age: i32,
    pub partner_age: i32,
    pub child_age: i32,
    pub love_reset: bool,
    pub xp_min: i32,
    pub xp_max_inclusive: i32,
}

pub fn breeding_result(mob_drops: bool) -> BreedingResult {
    BreedingResult {
        parent_age: PARENT_AGE_AFTER_BREEDING,
        partner_age: PARENT_AGE_AFTER_BREEDING,
        child_age: BABY_START_AGE,
        love_reset: true,
        xp_min: if mob_drops { 1 } else { 0 },
        xp_max_inclusive: if mob_drops { 7 } else { 0 },
    }
}

pub fn can_mate(
    same_class: bool,
    same_entity: bool,
    first_in_love: bool,
    second_in_love: bool,
) -> bool {
    !same_entity && same_class && first_in_love && second_in_love
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TamableState {
    pub flags: u8,
    pub owner_present: bool,
    pub ordered_to_sit: bool,
}

impl TamableState {
    pub fn set_tame(mut self, tame: bool) -> Self {
        set_flag(&mut self.flags, TAMABLE_FLAG_TAME, tame);
        self
    }

    pub fn set_sitting_pose(mut self, sitting: bool) -> Self {
        set_flag(&mut self.flags, TAMABLE_FLAG_SITTING, sitting);
        self.ordered_to_sit = sitting;
        self
    }

    pub fn is_tame(self) -> bool {
        self.flags & TAMABLE_FLAG_TAME != 0
    }

    pub fn is_sitting(self) -> bool {
        self.flags & TAMABLE_FLAG_SITTING != 0
    }
}

pub fn should_tamable_teleport_to_owner(owner_present: bool, distance_squared: i32) -> bool {
    owner_present && distance_squared >= TAMABLE_TELEPORT_DISTANCE_SQUARED
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketPickupResult {
    NotApplicable,
    FilledBucketAndDiscardEntity,
}

pub fn bucket_pickup_result(held_item: &str, entity_alive: bool) -> BucketPickupResult {
    if held_item == "minecraft:water_bucket" && entity_alive {
        BucketPickupResult::FilledBucketAndDiscardEntity
    } else {
        BucketPickupResult::NotApplicable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxolotlVariantModel {
    pub name: &'static str,
    pub id: i32,
    pub common_spawn: bool,
}

pub const AXOLOTL_VARIANTS: &[AxolotlVariantModel] = &[
    AxolotlVariantModel {
        name: "lucy",
        id: 0,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "wild",
        id: 1,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "gold",
        id: 2,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "cyan",
        id: 3,
        common_spawn: true,
    },
    AxolotlVariantModel {
        name: "blue",
        id: 4,
        common_spawn: false,
    },
];

pub const DEFAULT_AXOLOTL_VARIANT_ID: i32 = 0;
pub const AXOLOTL_RARE_VARIANT_CHANCE: i32 = 1200;

pub fn axolotl_variant_by_id(id: i32) -> AxolotlVariantModel {
    AXOLOTL_VARIANTS
        .get(id as usize)
        .copied()
        .unwrap_or(AXOLOTL_VARIANTS[DEFAULT_AXOLOTL_VARIANT_ID as usize])
}

pub fn axolotl_variant_by_name(name: &str) -> Option<AxolotlVariantModel> {
    AXOLOTL_VARIANTS
        .iter()
        .copied()
        .find(|variant| variant.name == name)
}

pub fn axolotl_spawn_variants(common_spawn: bool) -> Vec<AxolotlVariantModel> {
    AXOLOTL_VARIANTS
        .iter()
        .copied()
        .filter(|variant| variant.common_spawn == common_spawn)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxolotlState {
    pub playing_dead: bool,
    pub from_bucket: bool,
    pub air_supply: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxolotlHurtContext {
    pub no_ai: bool,
    pub random_one_in_three: bool,
    pub random_damage_gate: i32,
    pub damage: f32,
    pub current_health: f32,
    pub max_health: f32,
    pub in_water: bool,
    pub source_entity_present: bool,
    pub direct_entity_present: bool,
}

pub const AXOLOTL_PLAY_DEAD_TICKS: i32 = 200;
pub const AXOLOTL_MAX_AIR_SUPPLY: i32 = 6000;
pub const AXOLOTL_REHYDRATE_AIR_TICKS: i32 = 1800;
pub const AXOLOTL_DRY_OUT_DAMAGE: f32 = 2.0;

impl AxolotlState {
    pub fn new() -> Self {
        Self {
            playing_dead: false,
            from_bucket: false,
            air_supply: AXOLOTL_MAX_AIR_SUPPLY,
        }
    }

    pub fn should_play_ambient_sound(self) -> bool {
        !self.playing_dead
    }

    pub fn can_be_seen_as_enemy(self, super_can_be_seen_as_enemy: bool) -> bool {
        !self.playing_dead && super_can_be_seen_as_enemy
    }

    pub fn update_playing_dead_from_memory(&mut self, play_dead_ticks: Option<i32>, no_ai: bool) {
        if !no_ai {
            self.playing_dead = play_dead_ticks.is_some_and(|ticks| ticks > 0);
        }
    }

    pub fn rehydrate(&mut self) {
        self.air_supply =
            (self.air_supply + AXOLOTL_REHYDRATE_AIR_TICKS).min(AXOLOTL_MAX_AIR_SUPPLY);
    }
}

pub fn axolotl_play_dead_memory_on_hurt(context: AxolotlHurtContext) -> Option<i32> {
    let low_health = context.current_health / context.max_health < 0.5;
    let damaging_entity_present = context.source_entity_present || context.direct_entity_present;
    if !context.no_ai
        && context.random_one_in_three
        && ((context.random_damage_gate as f32) < context.damage || low_health)
        && context.damage < context.current_health
        && context.in_water
        && damaging_entity_present
    {
        Some(AXOLOTL_PLAY_DEAD_TICKS)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChickenState {
    pub egg_time: i32,
    pub is_chicken_jockey: bool,
    pub flap: f32,
    pub flap_speed: f32,
    pub flapping: f32,
    pub delta_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChickenTickEvent {
    None,
    LayEgg,
}

pub const CHICKEN_EGG_TIME_MIN: i32 = 6000;
pub const CHICKEN_EGG_TIME_RANDOM_BOUND: i32 = 6000;
pub const CHICKEN_ADULT_WIDTH: f32 = 0.4;
pub const CHICKEN_ADULT_HEIGHT: f32 = 0.7;
pub const CHICKEN_BABY_WIDTH: f32 = 0.3;
pub const CHICKEN_BABY_HEIGHT: f32 = 0.4;
pub const CHICKEN_BABY_EYE_HEIGHT: f32 = 0.28;
pub const CHICKEN_JOCKEY_BASE_EXPERIENCE: i32 = 10;

impl ChickenState {
    pub fn new(random_egg_offset: i32) -> Self {
        Self {
            egg_time: chicken_next_egg_time(random_egg_offset),
            is_chicken_jockey: false,
            flap: 0.0,
            flap_speed: 0.0,
            flapping: 1.0,
            delta_y: 0.0,
        }
    }

    pub fn tick(
        &mut self,
        on_ground: bool,
        alive: bool,
        baby: bool,
        server_level: bool,
        next_random_egg_offset: i32,
    ) -> ChickenTickEvent {
        self.flap_speed += if on_ground { -0.3 } else { 1.2 };
        self.flap_speed = self.flap_speed.clamp(0.0, 1.0);
        if !on_ground && self.flapping < 1.0 {
            self.flapping = 1.0;
        }
        self.flapping *= 0.9;
        if !on_ground && self.delta_y < 0.0 {
            self.delta_y *= 0.6;
        }
        self.flap += self.flapping * 2.0;

        if server_level && alive && !baby && !self.is_chicken_jockey {
            self.egg_time -= 1;
            if self.egg_time <= 0 {
                self.egg_time = chicken_next_egg_time(next_random_egg_offset);
                return ChickenTickEvent::LayEgg;
            }
        }

        ChickenTickEvent::None
    }

    pub fn remove_when_far_away(self) -> bool {
        self.is_chicken_jockey
    }

    pub fn base_experience_reward(self, super_reward: i32) -> i32 {
        if self.is_chicken_jockey {
            CHICKEN_JOCKEY_BASE_EXPERIENCE
        } else {
            super_reward
        }
    }
}

pub fn chicken_next_egg_time(random_offset: i32) -> i32 {
    CHICKEN_EGG_TIME_MIN + random_offset.clamp(0, CHICKEN_EGG_TIME_RANDOM_BOUND - 1)
}

pub fn chicken_dimensions(baby: bool) -> (f32, f32, Option<f32>) {
    if baby {
        (
            CHICKEN_BABY_WIDTH,
            CHICKEN_BABY_HEIGHT,
            Some(CHICKEN_BABY_EYE_HEIGHT),
        )
    } else {
        (CHICKEN_ADULT_WIDTH, CHICKEN_ADULT_HEIGHT, None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CowInteraction {
    FillMilkBucket,
    Delegate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MooshroomVariant {
    Red,
    Brown,
}

pub const COW_FOOD_ITEM: &str = "minecraft:wheat";
pub const COW_ADULT_WIDTH: f32 = 0.9;
pub const COW_ADULT_HEIGHT: f32 = 1.4;
pub const COW_BABY_WIDTH: f32 = 0.45;
pub const COW_BABY_HEIGHT: f32 = 0.7;
pub const COW_BABY_EYE_HEIGHT: f32 = 0.665;
pub const MOOSHROOM_MUTATE_CHANCE: i32 = 1024;

pub fn cow_interaction(item: &str, baby: bool) -> CowInteraction {
    if item == "minecraft:bucket" && !baby {
        CowInteraction::FillMilkBucket
    } else {
        CowInteraction::Delegate
    }
}

pub fn cow_is_food(item: &str) -> bool {
    item == COW_FOOD_ITEM
}

pub fn cow_dimensions(baby: bool) -> (f32, f32, Option<f32>) {
    if baby {
        (COW_BABY_WIDTH, COW_BABY_HEIGHT, Some(COW_BABY_EYE_HEIGHT))
    } else {
        (COW_ADULT_WIDTH, COW_ADULT_HEIGHT, None)
    }
}

pub fn cow_breed_variant(
    parent_variant: &'static str,
    partner_variant: &'static str,
    choose_parent: bool,
) -> &'static str {
    if choose_parent {
        parent_variant
    } else {
        partner_variant
    }
}

pub fn mooshroom_thunder_variant(
    current: MooshroomVariant,
    last_lightning_uuid: Option<&str>,
    lightning_uuid: &str,
) -> MooshroomVariant {
    if last_lightning_uuid == Some(lightning_uuid) {
        current
    } else {
        match current {
            MooshroomVariant::Red => MooshroomVariant::Brown,
            MooshroomVariant::Brown => MooshroomVariant::Red,
        }
    }
}

pub fn mooshroom_offspring_variant(
    parent: MooshroomVariant,
    partner: MooshroomVariant,
    mutate_same_variant: bool,
    choose_parent: bool,
) -> MooshroomVariant {
    if parent == partner && mutate_same_variant {
        match parent {
            MooshroomVariant::Red => MooshroomVariant::Brown,
            MooshroomVariant::Brown => MooshroomVariant::Red,
        }
    } else if choose_parent {
        parent
    } else {
        partner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DolphinState {
    pub got_fish: bool,
    pub moistness: i32,
    pub air_supply: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DolphinFeedResult {
    AgeUp { seconds: i32 },
    SetGotFish,
    NotFish,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DolphinLandTickOutcome {
    pub dry_out_damage: Option<f32>,
    pub jump_delta_y: Option<f64>,
    pub needs_sync: bool,
}

pub const DOLPHIN_TOTAL_AIR_SUPPLY: i32 = 4800;
pub const DOLPHIN_TOTAL_MOISTNESS_LEVEL: i32 = 2400;
pub const DOLPHIN_TREASURE_MIN_AIR_SUPPLY: i32 = 100;
pub const DOLPHIN_TREASURE_SEARCH_RADIUS: i32 = 50;
pub const DOLPHIN_TREASURE_STOP_DISTANCE: f64 = 4.0;
pub const DOLPHIN_SWIM_WITH_PLAYER_RANGE: f64 = 10.0;
pub const DOLPHIN_SWIM_WITH_PLAYER_CONTINUE_DISTANCE_SQUARED: f64 = 256.0;
pub const DOLPHIN_GRACE_DURATION_TICKS: i32 = 100;
pub const DOLPHIN_GRACE_REFRESH_RANDOM_BOUND: i32 = 6;
pub const DOLPHIN_BABY_SCALE: f32 = 0.65;
pub const DOLPHIN_DRY_OUT_DAMAGE: f32 = 1.0;

impl DolphinState {
    pub fn new() -> Self {
        Self {
            got_fish: false,
            moistness: DOLPHIN_TOTAL_MOISTNESS_LEVEL,
            air_supply: DOLPHIN_TOTAL_AIR_SUPPLY,
        }
    }

    pub fn tick_moistness(
        &mut self,
        no_ai: bool,
        in_water_or_rain: bool,
        on_ground: bool,
    ) -> DolphinLandTickOutcome {
        if no_ai {
            self.air_supply = DOLPHIN_TOTAL_AIR_SUPPLY;
            return DolphinLandTickOutcome {
                dry_out_damage: None,
                jump_delta_y: None,
                needs_sync: false,
            };
        }

        if in_water_or_rain {
            self.moistness = DOLPHIN_TOTAL_MOISTNESS_LEVEL;
            return DolphinLandTickOutcome {
                dry_out_damage: None,
                jump_delta_y: None,
                needs_sync: false,
            };
        }

        self.moistness -= 1;
        DolphinLandTickOutcome {
            dry_out_damage: (self.moistness <= 0).then_some(DOLPHIN_DRY_OUT_DAMAGE),
            jump_delta_y: on_ground.then_some(0.5),
            needs_sync: on_ground,
        }
    }

    pub fn can_start_treasure_goal(self) -> bool {
        self.got_fish && self.air_supply >= DOLPHIN_TREASURE_MIN_AIR_SUPPLY
    }

    pub fn should_clear_got_fish_on_treasure_stop(
        self,
        treasure_missing: bool,
        within_stop_distance: bool,
        stuck: bool,
    ) -> bool {
        treasure_missing || within_stop_distance || stuck
    }
}

pub fn dolphin_feed_result(
    item_is_fish: bool,
    can_age_up: bool,
    ticks_until_adult: i32,
) -> DolphinFeedResult {
    if !item_is_fish {
        DolphinFeedResult::NotFish
    } else if can_age_up {
        DolphinFeedResult::AgeUp {
            seconds: speed_up_seconds_when_feeding(ticks_until_adult),
        }
    } else {
        DolphinFeedResult::SetGotFish
    }
}

pub fn dolphin_grace_refresh_ticks(player_swimming: bool, random_roll: i32) -> Option<i32> {
    (player_swimming && random_roll.rem_euclid(DOLPHIN_GRACE_REFRESH_RANDOM_BOUND) == 0)
        .then_some(DOLPHIN_GRACE_DURATION_TICKS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeeState {
    pub flags: u8,
    pub time_since_sting: i32,
    pub ticks_without_nectar_since_exiting_hive: i32,
    pub stay_out_of_hive_countdown: i32,
    pub crops_grown_since_pollination: i32,
    pub under_water_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeeServerStepOutcome {
    pub drown_damage: Option<f32>,
    pub sting_death_damage: bool,
}

pub const BEE_FLAG_ROLL: u8 = 2;
pub const BEE_FLAG_HAS_STUNG: u8 = 4;
pub const BEE_FLAG_HAS_NECTAR: u8 = 8;
pub const BEE_STING_DEATH_COUNTDOWN: i32 = 1200;
pub const BEE_TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME: i32 = 3600;
pub const BEE_MAX_CROPS_GROWABLE: i32 = 10;
pub const BEE_POISON_SECONDS_NORMAL: i32 = 10;
pub const BEE_POISON_SECONDS_HARD: i32 = 18;
pub const BEE_TOO_FAR_DISTANCE: i32 = 48;
pub const BEE_HIVE_CLOSE_ENOUGH_DISTANCE: i32 = 2;
pub const BEE_HIVE_SEARCH_DISTANCE: i32 = 20;
pub const BEE_COOLDOWN_BEFORE_LOCATING_NEW_HIVE: i32 = 200;
pub const BEE_MIN_FIND_FLOWER_RETRY_COOLDOWN: i32 = 20;
pub const BEE_MAX_FIND_FLOWER_RETRY_COOLDOWN: i32 = 60;
pub const BEE_PERSISTENT_ANGER_MIN_TICKS: i32 = 20 * 20;
pub const BEE_PERSISTENT_ANGER_MAX_TICKS: i32 = 39 * 20;
pub const BEE_DROWN_DAMAGE: f32 = 1.0;

impl BeeState {
    pub fn new() -> Self {
        Self {
            flags: 0,
            time_since_sting: 0,
            ticks_without_nectar_since_exiting_hive: 0,
            stay_out_of_hive_countdown: 0,
            crops_grown_since_pollination: 0,
            under_water_ticks: 0,
        }
    }

    pub fn has_nectar(self) -> bool {
        self.get_flag(BEE_FLAG_HAS_NECTAR)
    }

    pub fn has_stung(self) -> bool {
        self.get_flag(BEE_FLAG_HAS_STUNG)
    }

    pub fn is_rolling(self) -> bool {
        self.get_flag(BEE_FLAG_ROLL)
    }

    pub fn set_has_nectar(&mut self, has_nectar: bool) {
        if has_nectar {
            self.ticks_without_nectar_since_exiting_hive = 0;
        }
        self.set_flag(BEE_FLAG_HAS_NECTAR, has_nectar);
    }

    pub fn set_has_stung(&mut self, has_stung: bool) {
        self.set_flag(BEE_FLAG_HAS_STUNG, has_stung);
    }

    pub fn set_rolling(&mut self, rolling: bool) {
        self.set_flag(BEE_FLAG_ROLL, rolling);
    }

    pub fn wants_to_enter_hive(
        self,
        pollinating: bool,
        has_target: bool,
        bees_stay_in_hive_environment: bool,
        hive_near_fire: bool,
    ) -> bool {
        if self.stay_out_of_hive_countdown > 0 || pollinating || self.has_stung() || has_target {
            return false;
        }
        (self.has_nectar()
            || self.ticks_without_nectar_since_exiting_hive
                > BEE_TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME
            || bees_stay_in_hive_environment)
            && !hive_near_fire
    }

    pub fn tick_server_ai(
        &mut self,
        in_water: bool,
        sting_death_roll_hits: bool,
    ) -> BeeServerStepOutcome {
        if in_water {
            self.under_water_ticks += 1;
        } else {
            self.under_water_ticks = 0;
        }

        let drown_damage = (self.under_water_ticks > 20).then_some(BEE_DROWN_DAMAGE);
        let mut sting_death_damage = false;
        if self.has_stung() {
            self.time_since_sting += 1;
            sting_death_damage = self.time_since_sting % 5 == 0 && sting_death_roll_hits;
        }

        if !self.has_nectar() {
            self.ticks_without_nectar_since_exiting_hive += 1;
        }

        BeeServerStepOutcome {
            drown_damage,
            sting_death_damage,
        }
    }

    pub fn tick_ai_step(&mut self, angry: bool, has_target: bool, target_distance_squared: f64) {
        if self.stay_out_of_hive_countdown > 0 {
            self.stay_out_of_hive_countdown -= 1;
        }
        self.set_rolling(angry && !self.has_stung() && has_target && target_distance_squared < 4.0);
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }

    fn get_flag(self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

pub fn bee_poison_duration_ticks(difficulty: &str) -> Option<i32> {
    match difficulty {
        "normal" => Some(BEE_POISON_SECONDS_NORMAL * 20),
        "hard" => Some(BEE_POISON_SECONDS_HARD * 20),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamelState {
    pub dashing: bool,
    pub dash_cooldown: i32,
    pub last_pose_change_tick: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamelTickOutcome {
    pub dash_ready_sound: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamelPassengerAttachment {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub const CAMEL_BABY_SCALE: f32 = 0.6;
pub const CAMEL_DASH_COOLDOWN_TICKS: i32 = 55;
pub const CAMEL_MAX_HEAD_Y_ROT: i32 = 30;
pub const CAMEL_RUNNING_SPEED_BONUS: f32 = 0.1;
pub const CAMEL_DASH_VERTICAL_MOMENTUM: f32 = 1.4285;
pub const CAMEL_DASH_HORIZONTAL_MOMENTUM: f32 = 22.2222;
pub const CAMEL_DASH_MINIMUM_DURATION_TICKS: i32 = 5;
pub const CAMEL_SITDOWN_DURATION_TICKS: i64 = 40;
pub const CAMEL_STANDUP_DURATION_TICKS: i64 = 52;
pub const CAMEL_IDLE_MINIMAL_DURATION_TICKS: i32 = 80;
pub const CAMEL_SITTING_HEIGHT_DIFFERENCE: f32 = 1.43;
pub const CAMEL_SITTING_EYE_HEIGHT: f32 = 0.845;
pub const CAMEL_DEFAULT_LAST_POSE_CHANGE_TICK: i64 = 0;

impl CamelState {
    pub fn new_standing(current_game_time: i64) -> Self {
        let mut state = Self {
            dashing: false,
            dash_cooldown: 0,
            last_pose_change_tick: CAMEL_DEFAULT_LAST_POSE_CHANGE_TICK,
        };
        state.reset_last_pose_change_tick_to_full_stand(current_game_time);
        state
    }

    pub fn from_saved_pose_tick(saved_pose_tick: i64) -> Self {
        Self {
            dashing: false,
            dash_cooldown: 0,
            last_pose_change_tick: saved_pose_tick,
        }
    }

    pub fn is_sitting(self) -> bool {
        self.last_pose_change_tick < 0
    }

    pub fn pose_time(self, current_game_time: i64) -> i64 {
        current_game_time - self.last_pose_change_tick.abs()
    }

    pub fn is_in_pose_transition(self, current_game_time: i64) -> bool {
        let pose_time = self.pose_time(current_game_time);
        pose_time
            < if self.is_sitting() {
                CAMEL_SITDOWN_DURATION_TICKS
            } else {
                CAMEL_STANDUP_DURATION_TICKS
            }
    }

    pub fn refuse_to_move(self, current_game_time: i64) -> bool {
        self.is_sitting() || self.is_in_pose_transition(current_game_time)
    }

    pub fn sit_down(&mut self, current_game_time: i64) -> bool {
        if self.is_sitting() {
            return false;
        }
        self.last_pose_change_tick = -current_game_time;
        true
    }

    pub fn stand_up(&mut self, current_game_time: i64) -> bool {
        if !self.is_sitting() {
            return false;
        }
        self.last_pose_change_tick = current_game_time;
        true
    }

    pub fn stand_up_instantly(&mut self, current_game_time: i64) {
        self.reset_last_pose_change_tick_to_full_stand(current_game_time);
    }

    pub fn reset_last_pose_change_tick_to_full_stand(&mut self, current_game_time: i64) {
        self.last_pose_change_tick = (current_game_time - CAMEL_STANDUP_DURATION_TICKS - 1).max(0);
    }

    pub fn can_start_dash(self, saddled: bool, on_ground: bool) -> bool {
        saddled && self.dash_cooldown <= 0 && on_ground
    }

    pub fn start_dash(&mut self) {
        self.dashing = true;
        self.dash_cooldown = CAMEL_DASH_COOLDOWN_TICKS;
    }

    pub fn tick(
        &mut self,
        on_ground: bool,
        in_liquid: bool,
        is_passenger: bool,
    ) -> CamelTickOutcome {
        if self.dashing
            && self.dash_cooldown < CAMEL_DASH_COOLDOWN_TICKS - CAMEL_DASH_MINIMUM_DURATION_TICKS
            && (on_ground || in_liquid || is_passenger)
        {
            self.dashing = false;
        }

        let mut dash_ready_sound = false;
        if self.dash_cooldown > 0 {
            self.dash_cooldown -= 1;
            dash_ready_sound = self.dash_cooldown == 0;
        }

        CamelTickOutcome { dash_ready_sound }
    }

    pub fn ridden_speed(self, base_movement_speed: f32, controller_sprinting: bool) -> f32 {
        base_movement_speed
            + if controller_sprinting && self.dash_cooldown == 0 {
                CAMEL_RUNNING_SPEED_BONUS
            } else {
                0.0
            }
    }

    pub fn can_add_passenger(passenger_count: usize) -> bool {
        passenger_count <= 2
    }
}

pub fn camel_dash_impulse(
    amount: f32,
    movement_speed: f64,
    block_speed_factor: f64,
    jump_power: f64,
) -> (f64, f64) {
    (
        CAMEL_DASH_HORIZONTAL_MOMENTUM as f64 * amount as f64 * movement_speed * block_speed_factor,
        CAMEL_DASH_VERTICAL_MOMENTUM as f64 * amount as f64 * jump_power,
    )
}

pub fn camel_passenger_attachment_point(
    passenger_index: usize,
    passenger_count: usize,
    passenger_is_animal: bool,
    camel_sitting: bool,
    removed: bool,
    dimensions_width: f32,
    dimensions_height: f32,
    scale: f32,
) -> CamelPassengerAttachment {
    let driver = passenger_index == 0;
    let mut offset = 0.5;
    let height = if removed {
        0.01
    } else {
        camel_body_anchor_y(camel_sitting, false, driver, 0.0, dimensions_height, scale)
    };
    if passenger_count > 1 {
        if !driver {
            offset = -0.7;
        }
        if passenger_is_animal {
            offset += 0.2;
        }
    }
    let _ = dimensions_width;
    CamelPassengerAttachment {
        x: 0.0,
        y: height,
        z: offset as f64 * scale as f64,
    }
}

fn camel_body_anchor_y(
    sitting: bool,
    in_pose_transition: bool,
    front: bool,
    pose_time: f32,
    dimensions_height: f32,
    scale: f32,
) -> f64 {
    let mut base_sit_offset = dimensions_height - 0.375 * scale;
    let sitting_height_difference = scale * CAMEL_SITTING_HEIGHT_DIFFERENCE;
    let vertical_drop = sitting_height_difference - scale * 0.2;
    let bottom_point = sitting_height_difference - vertical_drop;
    if in_pose_transition {
        let animation_duration = if sitting {
            CAMEL_SITDOWN_DURATION_TICKS as f32
        } else {
            CAMEL_STANDUP_DURATION_TICKS as f32
        };
        let (half_point, flex_point_offset) = if sitting {
            (28.0, if front { 0.5 } else { 0.1 })
        } else {
            (
                if front { 24.0 } else { 32.0 },
                if front { 0.6 } else { 0.35 },
            )
        };
        let pose_time = pose_time.clamp(0.0, animation_duration);
        let first_part = pose_time < half_point;
        let part = if first_part {
            pose_time / half_point
        } else {
            (pose_time - half_point) / (animation_duration - half_point)
        };
        let flex_point = sitting_height_difference - flex_point_offset * vertical_drop;
        base_sit_offset += if sitting {
            lerp(
                part,
                if first_part {
                    sitting_height_difference
                } else {
                    flex_point
                },
                if first_part { flex_point } else { bottom_point },
            )
        } else {
            lerp(
                part,
                if first_part {
                    bottom_point - sitting_height_difference
                } else {
                    bottom_point - flex_point
                },
                if first_part {
                    bottom_point - flex_point
                } else {
                    0.0
                },
            )
        };
    }
    if sitting && !in_pose_transition {
        base_sit_offset += bottom_point;
    }
    base_sit_offset as f64
}

fn lerp(part: f32, start: f32, end: f32) -> f32 {
    start + part * (end - start)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoatState {
    pub is_screaming: bool,
    pub has_left_horn: bool,
    pub has_right_horn: bool,
    pub lower_head_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoatHornDrop {
    None,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoatInteraction {
    Milk,
    Delegate,
}

pub const GOAT_SCREAMING_CHANCE_DENOMINATOR: i32 = 50;
pub const GOAT_INITIAL_MISSING_HORN_CHANCE_DENOMINATOR: i32 = 10;
pub const GOAT_RAM_PREPARE_TIME: i32 = 20;
pub const GOAT_RAM_MIN_DISTANCE: i32 = 4;
pub const GOAT_RAM_MAX_DISTANCE: i32 = 7;
pub const GOAT_TIME_BETWEEN_RAMS_MIN: i32 = 600;
pub const GOAT_TIME_BETWEEN_RAMS_MAX: i32 = 6000;
pub const GOAT_TIME_BETWEEN_RAMS_SCREAMER_MIN: i32 = 100;
pub const GOAT_TIME_BETWEEN_RAMS_SCREAMER_MAX: i32 = 300;
pub const GOAT_TIME_BETWEEN_LONG_JUMPS_MIN: i32 = 600;
pub const GOAT_TIME_BETWEEN_LONG_JUMPS_MAX: i32 = 1200;
pub const GOAT_MAX_LONG_JUMP_HEIGHT: i32 = 5;
pub const GOAT_MAX_LONG_JUMP_WIDTH: i32 = 5;
pub const GOAT_MAX_JUMP_VELOCITY_MULTIPLIER: f32 = 3.5714288;
pub const GOAT_PREPARE_RAM_SPEED_MULTIPLIER: f32 = 1.25;
pub const GOAT_RAMMING_SPEED_MULTIPLIER: f32 = 3.0;
pub const GOAT_ADULT_RAM_KNOCKBACK_FORCE: f32 = 2.5;
pub const GOAT_BABY_RAM_KNOCKBACK_FORCE: f32 = 1.0;
pub const GOAT_MAX_ADULT_RAMMING_X_HEAD_ROT_DEGREES: f32 = 30.0;
pub const GOAT_MAX_BABY_RAMMING_X_HEAD_ROT_DEGREES: f32 = 52.5;
pub const GOAT_MAX_LOWER_HEAD_TICK: i32 = 20;
pub const GOAT_LONG_JUMPING_WIDTH: f32 = 0.9 * 0.7;
pub const GOAT_LONG_JUMPING_HEIGHT: f32 = 1.3 * 0.7;

impl GoatState {
    pub fn new() -> Self {
        Self {
            is_screaming: false,
            has_left_horn: true,
            has_right_horn: true,
            lower_head_tick: 0,
        }
    }

    pub fn finalize_spawn(
        screaming_roll_zero_of_50: bool,
        adult: bool,
        missing_horn_roll_zero_of_10: bool,
        remove_left_horn: bool,
    ) -> Self {
        let mut state = Self::new();
        state.is_screaming = screaming_roll_zero_of_50;
        if adult && missing_horn_roll_zero_of_10 {
            if remove_left_horn {
                state.has_left_horn = false;
            } else {
                state.has_right_horn = false;
            }
        }
        state
    }

    pub fn drop_horn(&mut self, baby: bool, choose_left_when_both_present: bool) -> GoatHornDrop {
        if baby {
            return GoatHornDrop::None;
        }

        let horn_to_drop = match (self.has_left_horn, self.has_right_horn) {
            (false, false) => GoatHornDrop::None,
            (true, false) => GoatHornDrop::Left,
            (false, true) => GoatHornDrop::Right,
            (true, true) => {
                if choose_left_when_both_present {
                    GoatHornDrop::Left
                } else {
                    GoatHornDrop::Right
                }
            }
        };

        match horn_to_drop {
            GoatHornDrop::Left => self.has_left_horn = false,
            GoatHornDrop::Right => self.has_right_horn = false,
            GoatHornDrop::None => {}
        }
        horn_to_drop
    }

    pub fn tick_lower_head(&mut self, lowering_head: bool) {
        if lowering_head {
            self.lower_head_tick += 1;
        } else {
            self.lower_head_tick -= 2;
        }
        self.lower_head_tick = self.lower_head_tick.clamp(0, GOAT_MAX_LOWER_HEAD_TICK);
    }

    pub fn ramming_x_head_rot_radians(self, baby: bool) -> f32 {
        let max_rotation = if baby {
            GOAT_MAX_BABY_RAMMING_X_HEAD_ROT_DEGREES
        } else {
            GOAT_MAX_ADULT_RAMMING_X_HEAD_ROT_DEGREES
        };
        self.lower_head_tick as f32 / GOAT_MAX_LOWER_HEAD_TICK as f32
            * max_rotation
            * std::f32::consts::PI
            / 180.0
    }

    pub fn ram_cooldown_range(self) -> (i32, i32) {
        if self.is_screaming {
            (
                GOAT_TIME_BETWEEN_RAMS_SCREAMER_MIN,
                GOAT_TIME_BETWEEN_RAMS_SCREAMER_MAX,
            )
        } else {
            (GOAT_TIME_BETWEEN_RAMS_MIN, GOAT_TIME_BETWEEN_RAMS_MAX)
        }
    }
}

pub fn goat_interaction(item: &str, baby: bool) -> GoatInteraction {
    if item == "minecraft:bucket" && !baby {
        GoatInteraction::Milk
    } else {
        GoatInteraction::Delegate
    }
}

pub fn goat_ram_knockback_force(baby: bool) -> f32 {
    if baby {
        GOAT_BABY_RAM_KNOCKBACK_FORCE
    } else {
        GOAT_ADULT_RAM_KNOCKBACK_FORCE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PigState {
    pub saddled: bool,
    pub boost_time_total: i32,
    pub boosting: bool,
    pub boost_time: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PigInteraction {
    StartRide,
    DelegateToAnimal,
    EquipSaddle,
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PigBoostUseResult {
    BoostStarted {
        damage: i32,
        converts_to_fishing_rod: bool,
    },
    Pass,
}

pub const PIG_MAX_HEALTH: f32 = 10.0;
pub const PIG_MOVEMENT_SPEED: f64 = 0.25;
pub const PIG_RIDDEN_SPEED_FACTOR: f64 = 0.225;
pub const PIG_BOOST_MIN_TIME: i32 = 140;
pub const PIG_BOOST_MAX_TIME: i32 = 980;
pub const PIG_BOOST_RANDOM_BOUND: i32 = 841;
pub const PIG_BOOST_SPEED_AMPLIFIER: f32 = 1.15;
pub const PIG_CARROT_ON_A_STICK_DURABILITY: i32 = 25;
pub const PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST: i32 = 7;
pub const PIG_TEMPT_SPEED: f32 = 1.2;
pub const PIG_PANIC_SPEED: f32 = 1.25;
pub const PIG_FOLLOW_PARENT_SPEED: f32 = 1.1;
pub const PIG_LEASH_EYE_HEIGHT_FACTOR: f32 = 0.6;
pub const PIG_LEASH_WIDTH_FACTOR: f32 = 0.4;

impl PigState {
    pub fn new() -> Self {
        Self {
            saddled: false,
            boost_time_total: 0,
            boosting: false,
            boost_time: 0,
        }
    }

    pub fn controlling_passenger(
        self,
        first_passenger_is_player: bool,
        player_holds_carrot_on_a_stick: bool,
    ) -> bool {
        self.saddled && first_passenger_is_player && player_holds_carrot_on_a_stick
    }

    pub fn can_use_saddle_slot(alive: bool, baby: bool) -> bool {
        alive && !baby
    }

    pub fn boost(&mut self, random_offset_0_to_840: i32) -> bool {
        if self.boosting {
            return false;
        }
        self.boosting = true;
        self.boost_time = 0;
        self.boost_time_total =
            PIG_BOOST_MIN_TIME + random_offset_0_to_840.clamp(0, PIG_BOOST_RANDOM_BOUND - 1);
        true
    }

    pub fn tick_boost(&mut self) {
        if self.boosting {
            let previous_boost_time = self.boost_time;
            self.boost_time += 1;
            if previous_boost_time > self.boost_time_total {
                self.boosting = false;
            }
        }
    }

    pub fn boost_factor(self) -> f32 {
        if self.boosting {
            1.0 + PIG_BOOST_SPEED_AMPLIFIER
                * ((self.boost_time as f32 / self.boost_time_total as f32) * std::f32::consts::PI)
                    .sin()
        } else {
            1.0
        }
    }

    pub fn ridden_speed(self, movement_speed: f64) -> f32 {
        (movement_speed * PIG_RIDDEN_SPEED_FACTOR * self.boost_factor() as f64) as f32
    }
}

pub fn pig_interaction(
    has_food: bool,
    saddled: bool,
    is_vehicle: bool,
    player_secondary_use_active: bool,
    super_interaction_consumes: bool,
    item_equippable_saddle: bool,
) -> PigInteraction {
    if !has_food && saddled && !is_vehicle && !player_secondary_use_active {
        PigInteraction::StartRide
    } else if super_interaction_consumes {
        PigInteraction::DelegateToAnimal
    } else if item_equippable_saddle {
        PigInteraction::EquipSaddle
    } else {
        PigInteraction::Pass
    }
}

pub fn pig_thunder_converts_to_zombified_piglin(difficulty: &str) -> bool {
    difficulty != "peaceful"
}

pub fn pig_food_on_a_stick_use(
    server_side: bool,
    player_is_passenger: bool,
    controlled_vehicle_is_pig: bool,
    boost_started: bool,
    current_damage: i32,
) -> PigBoostUseResult {
    if server_side && player_is_passenger && controlled_vehicle_is_pig && boost_started {
        PigBoostUseResult::BoostStarted {
            damage: PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST,
            converts_to_fishing_rod: current_damage + PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST
                >= PIG_CARROT_ON_A_STICK_DURABILITY,
        }
    } else {
        PigBoostUseResult::Pass
    }
}

pub fn pig_offspring_variant<'a>(
    first_parent_variant: &'a str,
    second_parent_variant: &'a str,
    choose_first_parent: bool,
) -> &'a str {
    if choose_first_parent {
        first_parent_variant
    } else {
        second_parent_variant
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolarBearState {
    pub standing: bool,
    pub warning_sound_ticks: i32,
    pub client_stand_animation: f32,
    pub client_stand_animation_old: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarBearMeleeSignal {
    AttackAndStopStanding,
    StandAndWarn,
    StopStanding,
}

pub const POLAR_BEAR_MAX_HEALTH: f32 = 30.0;
pub const POLAR_BEAR_FOLLOW_RANGE: f64 = 20.0;
pub const POLAR_BEAR_PLAYER_ATTACK_FOLLOW_DISTANCE_FACTOR: f64 = 0.5;
pub const POLAR_BEAR_MOVEMENT_SPEED: f64 = 0.25;
pub const POLAR_BEAR_ATTACK_DAMAGE: f32 = 6.0;
pub const POLAR_BEAR_MELEE_SPEED: f32 = 1.25;
pub const POLAR_BEAR_PANIC_SPEED: f32 = 2.0;
pub const POLAR_BEAR_FOLLOW_PARENT_SPEED: f32 = 1.25;
pub const POLAR_BEAR_RANDOM_STROLL_SPEED: f32 = 1.0;
pub const POLAR_BEAR_LOOK_AT_PLAYER_DISTANCE: f32 = 6.0;
pub const POLAR_BEAR_STAND_ANIMATION_TICKS: f32 = 6.0;
pub const POLAR_BEAR_WARNING_SOUND_COOLDOWN_TICKS: i32 = 40;
pub const POLAR_BEAR_WARNING_ATTACK_TICKS: i32 = 10;
pub const POLAR_BEAR_CUB_ALERT_XZ_RANGE: f64 = 8.0;
pub const POLAR_BEAR_CUB_ALERT_Y_RANGE: f64 = 4.0;
pub const POLAR_BEAR_PERSISTENT_ANGER_MIN_TICKS: i32 = 20 * 20;
pub const POLAR_BEAR_PERSISTENT_ANGER_MAX_TICKS: i32 = 39 * 20;
pub const POLAR_BEAR_WATER_SLOWDOWN: f32 = 0.98;

impl PolarBearState {
    pub fn new() -> Self {
        Self {
            standing: false,
            warning_sound_ticks: 0,
            client_stand_animation: 0.0,
            client_stand_animation_old: 0.0,
        }
    }

    pub fn play_warning_sound(&mut self) -> bool {
        if self.warning_sound_ticks <= 0 {
            self.warning_sound_ticks = POLAR_BEAR_WARNING_SOUND_COOLDOWN_TICKS;
            true
        } else {
            false
        }
    }

    pub fn tick(&mut self, client_side: bool) {
        if client_side {
            self.client_stand_animation_old = self.client_stand_animation;
            if self.standing {
                self.client_stand_animation = (self.client_stand_animation + 1.0)
                    .clamp(0.0, POLAR_BEAR_STAND_ANIMATION_TICKS);
            } else {
                self.client_stand_animation = (self.client_stand_animation - 1.0)
                    .clamp(0.0, POLAR_BEAR_STAND_ANIMATION_TICKS);
            }
        }
        if self.warning_sound_ticks > 0 {
            self.warning_sound_ticks -= 1;
        }
    }

    pub fn standing_animation_scale(self, partial_tick: f32) -> f32 {
        lerp(
            partial_tick,
            self.client_stand_animation_old,
            self.client_stand_animation,
        ) / POLAR_BEAR_STAND_ANIMATION_TICKS
    }

    pub fn dimensions_height_scale(self) -> f32 {
        if self.client_stand_animation > 0.0 {
            1.0 + self.client_stand_animation / POLAR_BEAR_STAND_ANIMATION_TICKS
        } else {
            1.0
        }
    }
}

pub fn polar_bear_should_attack_player(
    bear_is_baby: bool,
    nearest_target_goal_can_use: bool,
    baby_bear_nearby: bool,
) -> bool {
    !bear_is_baby && nearest_target_goal_can_use && baby_bear_nearby
}

pub fn polar_bear_should_attack_fox(bear_is_baby: bool) -> bool {
    !bear_is_baby
}

pub fn polar_bear_should_alert_other_on_hurt(
    other_is_polar_bear: bool,
    other_is_baby: bool,
) -> bool {
    other_is_polar_bear && !other_is_baby
}

pub fn polar_bear_melee_signal(
    can_perform_attack: bool,
    distance_to_target_squared: f64,
    target_width: f32,
    ticks_until_next_attack: i32,
    time_to_attack: bool,
) -> PolarBearMeleeSignal {
    if can_perform_attack {
        PolarBearMeleeSignal::AttackAndStopStanding
    } else {
        let warning_distance = (target_width as f64 + 3.0) * (target_width as f64 + 3.0);
        if distance_to_target_squared < warning_distance {
            if ticks_until_next_attack <= POLAR_BEAR_WARNING_ATTACK_TICKS {
                PolarBearMeleeSignal::StandAndWarn
            } else if time_to_attack {
                PolarBearMeleeSignal::StopStanding
            } else {
                PolarBearMeleeSignal::StopStanding
            }
        } else {
            PolarBearMeleeSignal::StopStanding
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RabbitVariant {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RabbitState {
    pub variant: RabbitVariant,
    pub more_carrot_ticks: i32,
    pub jump_ticks: i32,
    pub jump_duration: i32,
    pub jump_delay_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RabbitRaidGardenResult {
    DestroyCrop,
    ReduceCarrotAge(i32),
    Noop,
}

pub const RABBIT_MAX_HEALTH: f32 = 3.0;
pub const RABBIT_MOVEMENT_SPEED: f32 = 0.3;
pub const RABBIT_ATTACK_DAMAGE: f32 = 3.0;
pub const RABBIT_EVIL_ATTACK_POWER_INCREMENT: f32 = 5.0;
pub const RABBIT_EVIL_ARMOR_VALUE: f32 = 8.0;
pub const RABBIT_STROLL_SPEED_MOD: f64 = 0.6;
pub const RABBIT_BREED_SPEED_MOD: f64 = 0.8;
pub const RABBIT_FOLLOW_SPEED_MOD: f64 = 1.0;
pub const RABBIT_FLEE_SPEED_MOD: f64 = 2.2;
pub const RABBIT_ATTACK_SPEED_MOD: f64 = 1.4;
pub const RABBIT_BABY_JUMP_HEIGHT: f64 = 0.5;
pub const RABBIT_ADULT_JUMP_HEIGHT: f64 = 1.5;
pub const RABBIT_JUMP_DELAY_TICKS: i32 = 10;
pub const RABBIT_PANIC_JUMP_DELAY_TICKS: i32 = 3;
pub const RABBIT_JUMP_DURATION_TICKS: i32 = 15;
pub const RABBIT_MORE_CARROTS_DELAY: i32 = 40;
pub const RABBIT_IDLE_MINIMAL_DURATION_TICKS: i32 = 180;
pub const RABBIT_BABY_WIDTH: f32 = 0.24;
pub const RABBIT_BABY_HEIGHT: f32 = 0.4;
pub const RABBIT_BABY_EYE_HEIGHT: f32 = 0.39;

impl RabbitVariant {
    pub fn id(self) -> i32 {
        match self {
            Self::Brown => 0,
            Self::White => 1,
            Self::Black => 2,
            Self::WhiteSplotched => 3,
            Self::Gold => 4,
            Self::Salt => 5,
            Self::Evil => 99,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Brown => "brown",
            Self::White => "white",
            Self::Black => "black",
            Self::WhiteSplotched => "white_splotched",
            Self::Gold => "gold",
            Self::Salt => "salt",
            Self::Evil => "evil",
        }
    }

    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        }
    }

    pub fn is_evil(self) -> bool {
        matches!(self, Self::Evil)
    }
}

impl RabbitState {
    pub fn new() -> Self {
        Self {
            variant: RabbitVariant::Brown,
            more_carrot_ticks: 0,
            jump_ticks: 0,
            jump_duration: 0,
            jump_delay_ticks: 0,
        }
    }

    pub fn wants_more_food(self) -> bool {
        self.more_carrot_ticks <= 0
    }

    pub fn tick_more_carrots(&mut self, random_subtract_0_to_2: i32) {
        if self.more_carrot_ticks > 0 {
            self.more_carrot_ticks -= random_subtract_0_to_2.clamp(0, 2);
            if self.more_carrot_ticks < 0 {
                self.more_carrot_ticks = 0;
            }
        }
    }

    pub fn start_jumping(&mut self) {
        self.jump_duration = RABBIT_JUMP_DURATION_TICKS;
        self.jump_ticks = 0;
    }

    pub fn ai_step_jump(&mut self) {
        if self.jump_ticks != self.jump_duration {
            self.jump_ticks += 1;
        } else if self.jump_duration != 0 {
            self.jump_ticks = 0;
            self.jump_duration = 0;
        }
    }

    pub fn jump_completion(self, partial_tick: f32) -> f32 {
        if self.jump_duration == 0 {
            0.0
        } else {
            (self.jump_ticks as f32 + partial_tick) / self.jump_duration as f32
        }
    }

    pub fn set_landing_delay(&mut self, speed_modifier: f64) {
        self.jump_delay_ticks = if speed_modifier < RABBIT_FLEE_SPEED_MOD {
            RABBIT_JUMP_DELAY_TICKS
        } else {
            RABBIT_PANIC_JUMP_DELAY_TICKS
        };
    }
}

pub fn rabbit_random_variant(
    spawns_white_rabbits: bool,
    spawns_gold_rabbits: bool,
    random_0_to_99: i32,
) -> RabbitVariant {
    let random = random_0_to_99.clamp(0, 99);
    if spawns_white_rabbits {
        if random < 80 {
            RabbitVariant::White
        } else {
            RabbitVariant::WhiteSplotched
        }
    } else if spawns_gold_rabbits {
        RabbitVariant::Gold
    } else if random < 50 {
        RabbitVariant::Brown
    } else if random < 90 {
        RabbitVariant::Salt
    } else {
        RabbitVariant::Black
    }
}

pub fn rabbit_offspring_variant(
    biome_variant: RabbitVariant,
    first_parent_variant: RabbitVariant,
    second_parent_variant: RabbitVariant,
    random_0_to_19: i32,
    choose_partner: bool,
) -> RabbitVariant {
    if random_0_to_19 == 0 {
        biome_variant
    } else if choose_partner {
        second_parent_variant
    } else {
        first_parent_variant
    }
}

pub fn rabbit_should_avoid_entity(variant: RabbitVariant, base_goal_can_use: bool) -> bool {
    !variant.is_evil() && base_goal_can_use
}

pub fn rabbit_raid_garden(
    mob_griefing: bool,
    wants_more_food: bool,
    reached_target: bool,
    carrot_age: Option<i32>,
) -> RabbitRaidGardenResult {
    if !mob_griefing || !wants_more_food || !reached_target {
        return RabbitRaidGardenResult::Noop;
    }
    match carrot_age {
        Some(0) => RabbitRaidGardenResult::DestroyCrop,
        Some(age) if age > 0 => RabbitRaidGardenResult::ReduceCarrotAge(age - 1),
        _ => RabbitRaidGardenResult::Noop,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyeColorModel {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SheepState {
    pub wool_data: u8,
    pub eat_animation_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheepInteraction {
    ShearServer,
    ConsumeClientOrNotReady,
    Delegate,
}

pub const SHEEP_EAT_ANIMATION_TICKS: i32 = 40;
pub const SHEEP_SHEARED_FLAG: u8 = 16;
pub const SHEEP_COLOR_MASK: u8 = 15;
pub const SHEEP_MAX_HEALTH: f32 = 8.0;
pub const SHEEP_MOVEMENT_SPEED: f32 = 0.23;
pub const SHEEP_PANIC_SPEED: f32 = 1.25;
pub const SHEEP_BREED_SPEED: f32 = 1.0;
pub const SHEEP_TEMPT_SPEED: f32 = 1.1;
pub const SHEEP_FOLLOW_PARENT_SPEED: f32 = 1.1;
pub const SHEEP_STROLL_SPEED: f32 = 1.0;
pub const SHEEP_LOOK_AT_PLAYER_DISTANCE: f32 = 6.0;
pub const SHEEP_ATE_AGE_UP_SECONDS: i32 = 60;

impl DyeColorModel {
    pub fn id(self) -> u8 {
        match self {
            Self::White => 0,
            Self::Orange => 1,
            Self::Magenta => 2,
            Self::LightBlue => 3,
            Self::Yellow => 4,
            Self::Lime => 5,
            Self::Pink => 6,
            Self::Gray => 7,
            Self::LightGray => 8,
            Self::Cyan => 9,
            Self::Purple => 10,
            Self::Blue => 11,
            Self::Brown => 12,
            Self::Green => 13,
            Self::Red => 14,
            Self::Black => 15,
        }
    }

    pub fn by_id(id: u8) -> Self {
        match id {
            1 => Self::Orange,
            2 => Self::Magenta,
            3 => Self::LightBlue,
            4 => Self::Yellow,
            5 => Self::Lime,
            6 => Self::Pink,
            7 => Self::Gray,
            8 => Self::LightGray,
            9 => Self::Cyan,
            10 => Self::Purple,
            11 => Self::Blue,
            12 => Self::Brown,
            13 => Self::Green,
            14 => Self::Red,
            15 => Self::Black,
            _ => Self::White,
        }
    }
}

impl SheepState {
    pub fn new() -> Self {
        Self {
            wool_data: 0,
            eat_animation_tick: 0,
        }
    }

    pub fn color(self) -> DyeColorModel {
        DyeColorModel::by_id(self.wool_data & SHEEP_COLOR_MASK)
    }

    pub fn set_color(&mut self, color: DyeColorModel) {
        self.wool_data = (self.wool_data & 0xF0) | (color.id() & SHEEP_COLOR_MASK);
    }

    pub fn is_sheared(self) -> bool {
        self.wool_data & SHEEP_SHEARED_FLAG != 0
    }

    pub fn set_sheared(&mut self, value: bool) {
        if value {
            self.wool_data |= SHEEP_SHEARED_FLAG;
        } else {
            self.wool_data &= !SHEEP_SHEARED_FLAG;
        }
    }

    pub fn ready_for_shearing(self, alive: bool, baby: bool) -> bool {
        alive && !self.is_sheared() && !baby
    }

    pub fn ate(&mut self, can_age_up: bool) -> Option<i32> {
        self.set_sheared(false);
        can_age_up.then_some(SHEEP_ATE_AGE_UP_SECONDS)
    }

    pub fn handle_entity_event(&mut self, event_id: u8) -> bool {
        if event_id == 10 {
            self.eat_animation_tick = SHEEP_EAT_ANIMATION_TICKS;
            true
        } else {
            false
        }
    }

    pub fn client_ai_step(&mut self) {
        self.eat_animation_tick = (self.eat_animation_tick - 1).max(0);
    }

    pub fn head_eat_position_scale(self, partial_tick: f32) -> f32 {
        if self.eat_animation_tick <= 0 {
            0.0
        } else if (4..=36).contains(&self.eat_animation_tick) {
            1.0
        } else if self.eat_animation_tick < 4 {
            (self.eat_animation_tick as f32 - partial_tick) / 4.0
        } else {
            -(self.eat_animation_tick as f32 - SHEEP_EAT_ANIMATION_TICKS as f32 - partial_tick)
                / 4.0
        }
    }

    pub fn head_eat_angle_scale(self, partial_tick: f32, x_rot_degrees: f32) -> f32 {
        if self.eat_animation_tick > 4 && self.eat_animation_tick <= 36 {
            let scale = (self.eat_animation_tick as f32 - 4.0 - partial_tick) / 32.0;
            std::f32::consts::PI / 5.0 + 0.21991149 * (scale * 28.7).sin()
        } else if self.eat_animation_tick > 0 {
            std::f32::consts::PI / 5.0
        } else {
            x_rot_degrees * std::f32::consts::PI / 180.0
        }
    }
}

pub fn sheep_interaction(
    item: &str,
    server_level: bool,
    ready_for_shearing: bool,
) -> SheepInteraction {
    if item == "minecraft:shears" {
        if server_level && ready_for_shearing {
            SheepInteraction::ShearServer
        } else {
            SheepInteraction::ConsumeClientOrNotReady
        }
    } else {
        SheepInteraction::Delegate
    }
}

pub fn sheep_spawn_color(
    warm_variant_biome: bool,
    cold_variant_biome: bool,
    primary_roll_0_to_99: i32,
    common_roll_0_to_499: i32,
) -> DyeColorModel {
    let primary = primary_roll_0_to_99.clamp(0, 99);
    let common = common_roll_0_to_499.clamp(0, 499);
    let common_color = if warm_variant_biome {
        DyeColorModel::Brown
    } else if cold_variant_biome {
        DyeColorModel::Black
    } else {
        DyeColorModel::White
    };
    if warm_variant_biome {
        match primary {
            0..=4 => DyeColorModel::Gray,
            5..=9 => DyeColorModel::LightGray,
            10..=14 => DyeColorModel::White,
            15..=17 => DyeColorModel::Black,
            _ => {
                if common == 499 {
                    DyeColorModel::Pink
                } else {
                    common_color
                }
            }
        }
    } else if cold_variant_biome {
        match primary {
            0..=4 => DyeColorModel::LightGray,
            5..=9 => DyeColorModel::Gray,
            10..=14 => DyeColorModel::White,
            15..=17 => DyeColorModel::Brown,
            _ => {
                if common == 499 {
                    DyeColorModel::Pink
                } else {
                    common_color
                }
            }
        }
    } else {
        match primary {
            0..=4 => DyeColorModel::Black,
            5..=9 => DyeColorModel::Gray,
            10..=14 => DyeColorModel::LightGray,
            15..=17 => DyeColorModel::Brown,
            _ => {
                if common == 499 {
                    DyeColorModel::Pink
                } else {
                    common_color
                }
            }
        }
    }
}

pub fn sheep_offspring_color(
    recipe_mixed_color: Option<DyeColorModel>,
    first_parent_color: DyeColorModel,
    second_parent_color: DyeColorModel,
    choose_first_parent: bool,
) -> DyeColorModel {
    recipe_mixed_color.unwrap_or(if choose_first_parent {
        first_parent_color
    } else {
        second_parent_color
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquidState {
    pub movement_vector: (f64, f64, f64),
    pub tentacle_movement: f32,
    pub tentacle_speed: f32,
    pub tentacle_angle: f32,
    pub rotate_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlowSquidState {
    pub dark_ticks_remaining: i32,
}

pub const SQUID_MAX_HEALTH: f32 = 10.0;
pub const SQUID_DEFAULT_GRAVITY: f64 = 0.08;
pub const SQUID_SOUND_VOLUME: f32 = 0.4;
pub const SQUID_BABY_WIDTH: f32 = 0.5;
pub const SQUID_BABY_HEIGHT: f32 = 0.63;
pub const SQUID_BABY_EYE_HEIGHT: f32 = 0.37;
pub const SQUID_INK_PARTICLE_COUNT: i32 = 30;
pub const SQUID_INK_BABY_OFFSET_SCALE: f32 = 0.1;
pub const SQUID_INK_ADULT_OFFSET_SCALE: f32 = 0.3;
pub const SQUID_FLEE_SPEED: f64 = 3.0;
pub const SQUID_FLEE_MIN_DISTANCE: f64 = 5.0;
pub const SQUID_FLEE_MAX_DISTANCE: f64 = 10.0;
pub const SQUID_FLEE_DISTANCE_SQUARED: f64 = 100.0;
pub const SQUID_FLEE_VECTOR_SCALE: f64 = 20.0;
pub const SQUID_BUBBLE_INTERVAL_TICKS: i32 = 10;
pub const SQUID_BUBBLE_PHASE_TICK: i32 = 5;
pub const GLOW_SQUID_DEFAULT_DARK_TICKS_REMAINING: i32 = 0;
pub const GLOW_SQUID_DARK_TICKS_ON_HURT: i32 = 100;
pub const GLOW_SQUID_SPAWN_SEA_LEVEL_OFFSET: i32 = 33;

impl SquidState {
    pub fn new(tentacle_speed_random_float: f32) -> Self {
        Self {
            movement_vector: (0.0, 0.0, 0.0),
            tentacle_movement: 0.0,
            tentacle_speed: 1.0 / (tentacle_speed_random_float.clamp(0.0, 1.0) + 1.0) * 0.2,
            tentacle_angle: 0.0,
            rotate_speed: 0.0,
        }
    }

    pub fn has_movement_vector(self) -> bool {
        let (x, y, z) = self.movement_vector;
        x * x + y * y + z * z > 1.0E-5
    }

    pub fn handle_entity_event(&mut self, event_id: u8) -> bool {
        if event_id == 19 {
            self.tentacle_movement = 0.0;
            true
        } else {
            false
        }
    }
}

impl GlowSquidState {
    pub fn new() -> Self {
        Self {
            dark_ticks_remaining: GLOW_SQUID_DEFAULT_DARK_TICKS_REMAINING,
        }
    }

    pub fn ai_step(&mut self) {
        if self.dark_ticks_remaining > 0 {
            self.dark_ticks_remaining -= 1;
        }
    }

    pub fn on_hurt(&mut self, hurt: bool) {
        if hurt {
            self.dark_ticks_remaining = GLOW_SQUID_DARK_TICKS_ON_HURT;
        }
    }
}

pub fn squid_hurt_spawns_ink(super_hurt: bool, last_hurt_by_mob_present: bool) -> bool {
    super_hurt && last_hurt_by_mob_present
}

pub fn squid_flee_can_use(
    in_water: bool,
    last_hurt_by_mob_present: bool,
    distance_squared: f64,
) -> bool {
    in_water && last_hurt_by_mob_present && distance_squared < SQUID_FLEE_DISTANCE_SQUARED
}

pub fn squid_flee_vector(
    squid_pos: (f64, f64, f64),
    attacker_pos: (f64, f64, f64),
    target_block_is_water: bool,
    target_block_is_air: bool,
) -> Option<(f64, f64, f64)> {
    if !target_block_is_water && !target_block_is_air {
        return None;
    }
    let mut x = squid_pos.0 - attacker_pos.0;
    let mut y = squid_pos.1 - attacker_pos.1;
    let mut z = squid_pos.2 - attacker_pos.2;
    let length = (x * x + y * y + z * z).sqrt();
    if length > 0.0 {
        x /= length;
        y /= length;
        z /= length;
        let mut avoid_speed = SQUID_FLEE_SPEED;
        if length > SQUID_FLEE_MIN_DISTANCE {
            avoid_speed -= (length - SQUID_FLEE_MIN_DISTANCE) / SQUID_FLEE_MIN_DISTANCE;
        }
        if avoid_speed > 0.0 {
            x *= avoid_speed;
            y *= avoid_speed;
            z *= avoid_speed;
        }
    }
    if target_block_is_air {
        y = 0.0;
    }
    Some((
        x / SQUID_FLEE_VECTOR_SCALE,
        y / SQUID_FLEE_VECTOR_SCALE,
        z / SQUID_FLEE_VECTOR_SCALE,
    ))
}

pub fn squid_flee_emits_bubble(flee_ticks: i32) -> bool {
    flee_ticks.rem_euclid(SQUID_BUBBLE_INTERVAL_TICKS) == SQUID_BUBBLE_PHASE_TICK
}

pub fn glow_squid_spawn_allowed(
    y: i32,
    sea_level: i32,
    raw_brightness: i32,
    block_is_water: bool,
) -> bool {
    y <= sea_level - GLOW_SQUID_SPAWN_SEA_LEVEL_OFFSET && raw_brightness == 0 && block_is_water
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PufferfishState {
    pub puff_state: u8,
    pub inflate_counter: i32,
    pub deflate_timer: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PufferfishContactEffect {
    pub damage: i32,
    pub poison_effect: &'static str,
    pub poison_duration_ticks: i32,
    pub poison_amplifier: u8,
}

impl PufferfishState {
    pub const SMALL: u8 = 0;
    pub const MID: u8 = 1;
    pub const FULL: u8 = 2;

    pub fn new() -> Self {
        Self {
            puff_state: Self::SMALL,
            inflate_counter: 0,
            deflate_timer: 0,
        }
    }

    pub fn start_inflating(&mut self) {
        self.inflate_counter = 1;
        self.deflate_timer = 0;
    }

    pub fn stop_inflating(&mut self) {
        self.inflate_counter = 0;
    }

    pub fn tick(&mut self, alive: bool, effective_ai: bool) {
        if !alive || !effective_ai {
            return;
        }

        if self.inflate_counter > 0 {
            if self.puff_state == Self::SMALL {
                self.puff_state = Self::MID;
            } else if self.inflate_counter > 40 && self.puff_state == Self::MID {
                self.puff_state = Self::FULL;
            }
            self.inflate_counter += 1;
        } else if self.puff_state != Self::SMALL {
            if self.deflate_timer > 60 && self.puff_state == Self::FULL {
                self.puff_state = Self::MID;
            } else if self.deflate_timer > 100 && self.puff_state == Self::MID {
                self.puff_state = Self::SMALL;
            }
            self.deflate_timer += 1;
        }
    }

    pub fn contact_effect(
        self,
        target_alive: bool,
        target_scary: bool,
    ) -> Option<PufferfishContactEffect> {
        if !target_alive || !target_scary || self.puff_state == Self::SMALL {
            return None;
        }

        Some(PufferfishContactEffect {
            damage: 1 + i32::from(self.puff_state),
            poison_effect: "minecraft:poison",
            poison_duration_ticks: 60 * i32::from(self.puff_state),
            poison_amplifier: 0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SalmonVariantModel {
    pub name: &'static str,
    pub id: i32,
    pub bounding_box_scale: f32,
    pub spawn_weight: i32,
}

pub const SALMON_VARIANTS: &[SalmonVariantModel] = &[
    SalmonVariantModel {
        name: "small",
        id: 0,
        bounding_box_scale: 0.5,
        spawn_weight: 30,
    },
    SalmonVariantModel {
        name: "medium",
        id: 1,
        bounding_box_scale: 1.0,
        spawn_weight: 50,
    },
    SalmonVariantModel {
        name: "large",
        id: 2,
        bounding_box_scale: 1.5,
        spawn_weight: 15,
    },
];

pub const DEFAULT_SALMON_VARIANT_ID: i32 = 1;

pub fn salmon_variant_by_id(id: i32) -> SalmonVariantModel {
    let clamped = id.clamp(0, (SALMON_VARIANTS.len() - 1) as i32);
    SALMON_VARIANTS[clamped as usize]
}

pub fn salmon_variant_by_name(name: &str) -> Option<SalmonVariantModel> {
    SALMON_VARIANTS
        .iter()
        .copied()
        .find(|variant| variant.name == name)
}

pub fn salmon_spawn_weight_total() -> i32 {
    SALMON_VARIANTS
        .iter()
        .map(|variant| variant.spawn_weight)
        .sum()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TropicalFishBase {
    Small = 0,
    Large = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TropicalFishPatternModel {
    pub name: &'static str,
    pub base: TropicalFishBase,
    pub index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TropicalFishVariantModel {
    pub pattern: TropicalFishPatternModel,
    pub base_color_id: i32,
    pub pattern_color_id: i32,
}

pub const TROPICAL_FISH_PATTERNS: &[TropicalFishPatternModel] = &[
    TropicalFishPatternModel {
        name: "kob",
        base: TropicalFishBase::Small,
        index: 0,
    },
    TropicalFishPatternModel {
        name: "sunstreak",
        base: TropicalFishBase::Small,
        index: 1,
    },
    TropicalFishPatternModel {
        name: "snooper",
        base: TropicalFishBase::Small,
        index: 2,
    },
    TropicalFishPatternModel {
        name: "dasher",
        base: TropicalFishBase::Small,
        index: 3,
    },
    TropicalFishPatternModel {
        name: "brinely",
        base: TropicalFishBase::Small,
        index: 4,
    },
    TropicalFishPatternModel {
        name: "spotty",
        base: TropicalFishBase::Small,
        index: 5,
    },
    TropicalFishPatternModel {
        name: "flopper",
        base: TropicalFishBase::Large,
        index: 0,
    },
    TropicalFishPatternModel {
        name: "stripey",
        base: TropicalFishBase::Large,
        index: 1,
    },
    TropicalFishPatternModel {
        name: "glitter",
        base: TropicalFishBase::Large,
        index: 2,
    },
    TropicalFishPatternModel {
        name: "blockfish",
        base: TropicalFishBase::Large,
        index: 3,
    },
    TropicalFishPatternModel {
        name: "betty",
        base: TropicalFishBase::Large,
        index: 4,
    },
    TropicalFishPatternModel {
        name: "clayfish",
        base: TropicalFishBase::Large,
        index: 5,
    },
];

pub const DEFAULT_TROPICAL_FISH_VARIANT_PACKED_ID: i32 = 0;

pub fn tropical_fish_pattern_packed_id(pattern: TropicalFishPatternModel) -> i32 {
    pattern.base as i32 | pattern.index << 8
}

pub fn tropical_fish_pack_variant(
    pattern: TropicalFishPatternModel,
    base_color_id: i32,
    pattern_color_id: i32,
) -> i32 {
    tropical_fish_pattern_packed_id(pattern) & 65_535
        | (base_color_id & 0xff) << 16
        | (pattern_color_id & 0xff) << 24
}

pub fn tropical_fish_base_color_id(packed_variant: i32) -> i32 {
    packed_variant >> 16 & 0xff
}

pub fn tropical_fish_pattern_color_id(packed_variant: i32) -> i32 {
    packed_variant >> 24 & 0xff
}

pub fn tropical_fish_pattern_by_packed_id(packed_id: i32) -> TropicalFishPatternModel {
    TROPICAL_FISH_PATTERNS
        .iter()
        .copied()
        .find(|pattern| tropical_fish_pattern_packed_id(*pattern) == packed_id)
        .unwrap_or(TROPICAL_FISH_PATTERNS[0])
}

pub fn tropical_fish_pattern_from_variant(packed_variant: i32) -> TropicalFishPatternModel {
    tropical_fish_pattern_by_packed_id(packed_variant & 65_535)
}

pub fn tropical_fish_common_variants() -> Vec<TropicalFishVariantModel> {
    const ORANGE: i32 = 1;
    const LIGHT_BLUE: i32 = 3;
    const YELLOW: i32 = 4;
    const LIME: i32 = 5;
    const PINK: i32 = 6;
    const GRAY: i32 = 7;
    const CYAN: i32 = 9;
    const PURPLE: i32 = 10;
    const BLUE: i32 = 11;
    const RED: i32 = 14;
    const WHITE: i32 = 0;

    [
        ("stripey", ORANGE, GRAY),
        ("flopper", GRAY, GRAY),
        ("flopper", GRAY, BLUE),
        ("clayfish", WHITE, GRAY),
        ("sunstreak", BLUE, GRAY),
        ("kob", ORANGE, WHITE),
        ("spotty", PINK, LIGHT_BLUE),
        ("blockfish", PURPLE, YELLOW),
        ("clayfish", WHITE, RED),
        ("spotty", WHITE, YELLOW),
        ("glitter", WHITE, GRAY),
        ("clayfish", WHITE, ORANGE),
        ("dasher", CYAN, PINK),
        ("brinely", LIME, LIGHT_BLUE),
        ("betty", RED, WHITE),
        ("snooper", GRAY, RED),
        ("blockfish", RED, WHITE),
        ("flopper", WHITE, YELLOW),
        ("kob", RED, WHITE),
        ("sunstreak", GRAY, WHITE),
        ("dasher", CYAN, YELLOW),
        ("flopper", YELLOW, YELLOW),
    ]
    .into_iter()
    .map(
        |(name, base_color_id, pattern_color_id)| TropicalFishVariantModel {
            pattern: *TROPICAL_FISH_PATTERNS
                .iter()
                .find(|pattern| pattern.name == name)
                .expect("common tropical fish pattern must exist"),
            base_color_id,
            pattern_color_id,
        },
    )
    .collect()
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketEntityData {
    pub no_ai: bool,
    pub silent: bool,
    pub no_gravity: bool,
    pub glowing: bool,
    pub invulnerable: bool,
    pub health_saved: bool,
}

pub fn save_default_bucket_data(
    no_ai: bool,
    silent: bool,
    no_gravity: bool,
    glowing: bool,
    invulnerable: bool,
) -> BucketEntityData {
    BucketEntityData {
        no_ai,
        silent,
        no_gravity,
        glowing,
        invulnerable,
        health_saved: true,
    }
}

pub fn ready_for_shearing(alive: bool, baby: bool, already_sheared: bool) -> bool {
    alive && !baby && !already_sheared
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MooshroomInteraction {
    FillStew,
    FillSuspiciousStew,
    ShearIntoCow,
    StoreSuspiciousEffects,
    Delegate,
}

pub fn mooshroom_interaction(
    item: &str,
    baby: bool,
    brown: bool,
    has_stew_effects: bool,
    item_has_suspicious_effect: bool,
) -> MooshroomInteraction {
    if item == "minecraft:bowl" && !baby {
        if has_stew_effects {
            MooshroomInteraction::FillSuspiciousStew
        } else {
            MooshroomInteraction::FillStew
        }
    } else if item == "minecraft:shears" && ready_for_shearing(true, baby, false) {
        MooshroomInteraction::ShearIntoCow
    } else if brown && !baby && item_has_suspicious_effect {
        MooshroomInteraction::StoreSuspiciousEffects
    } else {
        MooshroomInteraction::Delegate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorseState {
    pub flags: u8,
    pub temper: i32,
    pub max_temper: i32,
    pub baby: bool,
    pub alive: bool,
}

impl HorseState {
    pub fn is_tamed(self) -> bool {
        self.flags & HORSE_FLAG_TAME != 0
    }

    pub fn can_use_saddle_slot(self) -> bool {
        self.alive && !self.baby && self.is_tamed()
    }

    pub fn modify_temper(mut self, amount: i32) -> Self {
        self.temper = (self.temper + amount).clamp(0, self.max_temper);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VillagerTradeResult {
    LazyLoadOffers,
    IncreaseUsesRewardXpTriggerAdvancement,
    StopTrading,
}

pub fn villager_slot_index(raw_slot: i32) -> Option<usize> {
    let index = raw_slot - VILLAGER_INVENTORY_SLOT_OFFSET;
    (index >= 0 && (index as usize) < VILLAGER_INVENTORY_SIZE).then_some(index as usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AngerState {
    pub anger_end_time: i64,
    pub target_present: bool,
}

impl AngerState {
    pub fn is_angry(self, game_time: i64) -> bool {
        self.anger_end_time > 0 && self.anger_end_time - game_time > 0
    }

    pub fn set_time_to_remain_angry(mut self, game_time: i64, remaining_time: i64) -> Self {
        self.anger_end_time = game_time + remaining_time;
        self
    }

    pub fn stop_being_angry(mut self) -> Self {
        self.anger_end_time = NO_ANGER_END_TIME;
        self.target_present = false;
        self
    }
}

pub fn should_stop_anger_for_player(creative: bool, spectator: bool, peaceful: bool) -> bool {
    creative || spectator || peaceful
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionTypeModel {
    Single,
    SplitOnDeath,
}

impl ConversionTypeModel {
    pub fn discard_after_conversion(self) -> bool {
        matches!(self, Self::Single)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversionParamsModel {
    pub conversion_type: ConversionTypeModel,
    pub keep_equipment: bool,
    pub preserve_can_pick_up_loot: bool,
    pub team_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversionCopyPlan {
    pub copy_position_motion_passenger_vehicle: bool,
    pub keep_equipment: bool,
    pub copy_effects_absorption_age_anger_and_flags: bool,
    pub preserve_can_pick_up_loot: bool,
    pub move_scoreboard_team: bool,
    pub discard_original: bool,
}

pub fn conversion_copy_plan(params: ConversionParamsModel) -> ConversionCopyPlan {
    ConversionCopyPlan {
        copy_position_motion_passenger_vehicle: params.conversion_type
            == ConversionTypeModel::Single,
        keep_equipment: params.keep_equipment,
        copy_effects_absorption_age_anger_and_flags: true,
        preserve_can_pick_up_loot: params.preserve_can_pick_up_loot,
        move_scoreboard_team: params.team_present,
        discard_original: params.conversion_type.discard_after_conversion(),
    }
}

fn set_flag(flags: &mut u8, flag: u8, value: bool) {
    if value {
        *flags |= flag;
    } else {
        *flags &= !flag;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobInteractionCoverage {
    pub source: &'static str,
    pub covered_rules: &'static [&'static str],
}

pub const MOB_INTERACTION_COVERAGE: &[MobInteractionCoverage] = &[
    MobInteractionCoverage {
        source: "AgeableMob",
        covered_rules: &["ageable", "age lock", "baby variants"],
    },
    MobInteractionCoverage {
        source: "Animal",
        covered_rules: &["breedable", "love mode", "feeding"],
    },
    MobInteractionCoverage {
        source: "TamableAnimal",
        covered_rules: &["tameable", "owner", "sitting", "teleport"],
    },
    MobInteractionCoverage {
        source: "AbstractHorse",
        covered_rules: &["rideable", "saddle", "temper"],
    },
    MobInteractionCoverage {
        source: "Bucketable",
        covered_rules: &["bucketable", "bucket save/load"],
    },
    MobInteractionCoverage {
        source: "Shearable/MushroomCow",
        covered_rules: &["shearable", "transformation"],
    },
    MobInteractionCoverage {
        source: "Variant bootstraps",
        covered_rules: &["variant"],
    },
    MobInteractionCoverage {
        source: "AbstractVillager",
        covered_rules: &["trading"],
    },
    MobInteractionCoverage {
        source: "NeutralMob",
        covered_rules: &["anger"],
    },
    MobInteractionCoverage {
        source: "ConversionType",
        covered_rules: &["conversion", "transformation"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ageable_mobs_tick_feed_and_age_lock_like_vanilla() {
        let baby = AgeState::baby();
        assert!(baby.is_baby());
        assert_eq!(baby.tick().age, -23_999);
        assert_eq!(speed_up_seconds_when_feeding(24_000), 120);
        assert_eq!(
            baby.age_up(120, true),
            AgeState {
                age: -21_600,
                forced_age: 2_400,
                forced_age_timer: 40,
                age_locked: false,
                age_lock_particle_timer: 0,
            }
        );
        let locked = baby.toggle_age_lock();
        assert!(locked.age_locked);
        assert_eq!(locked.age_lock_particle_timer, 40);
        assert_eq!(locked.tick().age, BABY_START_AGE);
    }

    #[test]
    fn animals_enter_love_age_up_breed_and_reset_parent_state() {
        assert_eq!(
            animal_feed_result(true, 0, 0, false, false),
            AnimalFeedResult::SetInLove
        );
        assert_eq!(
            animal_feed_result(true, -10_000, 0, true, false),
            AnimalFeedResult::AgeUp { seconds: 50 }
        );
        assert_eq!(LoveState::set_in_love(true).tick(0).in_love, 599);
        assert_eq!(LoveState::set_in_love(true).tick(-1).in_love, 0);
        assert!(can_mate(true, false, true, true));
        assert!(!can_mate(false, false, true, true));

        assert_eq!(
            breeding_result(true),
            BreedingResult {
                parent_age: 6_000,
                partner_age: 6_000,
                child_age: -24_000,
                love_reset: true,
                xp_min: 1,
                xp_max_inclusive: 7,
            }
        );
    }

    #[test]
    fn tamable_flags_owner_sitting_and_teleport_distance_match_vanilla() {
        let state = TamableState {
            flags: 0,
            owner_present: true,
            ordered_to_sit: false,
        }
        .set_tame(true)
        .set_sitting_pose(true);
        assert!(state.is_tame());
        assert!(state.is_sitting());
        assert!(state.ordered_to_sit);
        assert!(should_tamable_teleport_to_owner(true, 144));
        assert!(!should_tamable_teleport_to_owner(true, 143));
        assert_eq!(TAMABLE_TELEPORT_ATTEMPTS, 10);
        assert_eq!(TAMABLE_TELEPORT_MIN_HORIZONTAL, 2);
        assert_eq!(TAMABLE_TELEPORT_MAX_HORIZONTAL, 3);
        assert_eq!(TAMABLE_TELEPORT_MAX_VERTICAL, 1);
    }

    #[test]
    fn bucketable_shearable_and_mooshroom_transformations_follow_interaction_gates() {
        assert_eq!(
            bucket_pickup_result("minecraft:water_bucket", true),
            BucketPickupResult::FilledBucketAndDiscardEntity
        );
        assert_eq!(
            save_default_bucket_data(true, true, false, true, false),
            BucketEntityData {
                no_ai: true,
                silent: true,
                no_gravity: false,
                glowing: true,
                invulnerable: false,
                health_saved: true,
            }
        );
        assert!(ready_for_shearing(true, false, false));
        assert!(!ready_for_shearing(true, true, false));
        assert_eq!(
            mooshroom_interaction("minecraft:bowl", false, false, false, false),
            MooshroomInteraction::FillStew
        );
        assert_eq!(
            mooshroom_interaction("minecraft:bowl", false, true, true, false),
            MooshroomInteraction::FillSuspiciousStew
        );
        assert_eq!(
            mooshroom_interaction("minecraft:shears", false, false, false, false),
            MooshroomInteraction::ShearIntoCow
        );
        assert_eq!(
            mooshroom_interaction("minecraft:poppy", false, true, false, true),
            MooshroomInteraction::StoreSuspiciousEffects
        );
    }

    #[test]
    fn horse_riding_saddle_temper_and_villager_trading_slots_match_vanilla() {
        let horse = HorseState {
            flags: HORSE_FLAG_TAME,
            temper: 5,
            max_temper: 100,
            baby: false,
            alive: true,
        };
        assert!(horse.can_use_saddle_slot());
        assert_eq!(horse.modify_temper(200).temper, 100);
        assert_eq!(HORSE_CHEST_SLOT_OFFSET, 499);
        assert_eq!(HORSE_INVENTORY_SLOT_OFFSET, 500);
        assert_eq!(HORSE_BREEDING_CROSS_FACTOR, 0.15);
        assert_eq!(villager_slot_index(300), Some(0));
        assert_eq!(villager_slot_index(307), Some(7));
        assert_eq!(villager_slot_index(308), None);
        assert_eq!(VILLAGER_INVENTORY_SIZE, 8);
    }

    #[test]
    fn anger_and_conversion_preserve_server_visible_state() {
        let angry = AngerState {
            anger_end_time: NO_ANGER_END_TIME,
            target_present: true,
        }
        .set_time_to_remain_angry(100, 40);
        assert!(angry.is_angry(120));
        assert!(!angry.is_angry(140));
        assert!(should_stop_anger_for_player(false, false, true));
        assert_eq!(angry.stop_being_angry().anger_end_time, NO_ANGER_END_TIME);

        assert_eq!(
            conversion_copy_plan(ConversionParamsModel {
                conversion_type: ConversionTypeModel::Single,
                keep_equipment: true,
                preserve_can_pick_up_loot: true,
                team_present: true,
            }),
            ConversionCopyPlan {
                copy_position_motion_passenger_vehicle: true,
                keep_equipment: true,
                copy_effects_absorption_age_anger_and_flags: true,
                preserve_can_pick_up_loot: true,
                move_scoreboard_team: true,
                discard_original: true,
            }
        );
        assert!(!ConversionTypeModel::SplitOnDeath.discard_after_conversion());
    }

    #[test]
    fn axolotl_variants_match_java_ids_default_and_rare_flag() {
        assert_eq!(DEFAULT_AXOLOTL_VARIANT_ID, 0);
        assert_eq!(AXOLOTL_RARE_VARIANT_CHANCE, 1200);
        assert_eq!(
            AXOLOTL_VARIANTS,
            &[
                AxolotlVariantModel {
                    name: "lucy",
                    id: 0,
                    common_spawn: true,
                },
                AxolotlVariantModel {
                    name: "wild",
                    id: 1,
                    common_spawn: true,
                },
                AxolotlVariantModel {
                    name: "gold",
                    id: 2,
                    common_spawn: true,
                },
                AxolotlVariantModel {
                    name: "cyan",
                    id: 3,
                    common_spawn: true,
                },
                AxolotlVariantModel {
                    name: "blue",
                    id: 4,
                    common_spawn: false,
                },
            ]
        );
        assert_eq!(axolotl_variant_by_id(-1).name, "lucy");
        assert_eq!(axolotl_variant_by_id(99).name, "lucy");
        assert_eq!(axolotl_variant_by_name("blue").unwrap().id, 4);

        let common = axolotl_spawn_variants(true);
        assert_eq!(common.len(), 4);
        assert!(common.iter().all(|variant| variant.common_spawn));

        let rare = axolotl_spawn_variants(false);
        assert_eq!(rare, vec![AXOLOTL_VARIANTS[4]]);
    }

    #[test]
    fn axolotl_play_dead_and_air_rules_match_java_gates() {
        let mut axolotl = AxolotlState::new();
        assert_eq!(axolotl.air_supply, AXOLOTL_MAX_AIR_SUPPLY);
        assert!(axolotl.should_play_ambient_sound());
        assert!(axolotl.can_be_seen_as_enemy(true));

        axolotl.update_playing_dead_from_memory(Some(AXOLOTL_PLAY_DEAD_TICKS), false);
        assert!(axolotl.playing_dead);
        assert!(!axolotl.should_play_ambient_sound());
        assert!(!axolotl.can_be_seen_as_enemy(true));

        axolotl.update_playing_dead_from_memory(Some(AXOLOTL_PLAY_DEAD_TICKS), true);
        assert!(axolotl.playing_dead);
        axolotl.update_playing_dead_from_memory(None, false);
        assert!(!axolotl.playing_dead);

        axolotl.air_supply = 5000;
        axolotl.rehydrate();
        assert_eq!(axolotl.air_supply, AXOLOTL_MAX_AIR_SUPPLY);
        axolotl.air_supply = 1000;
        axolotl.rehydrate();
        assert_eq!(axolotl.air_supply, 2800);

        let triggering = AxolotlHurtContext {
            no_ai: false,
            random_one_in_three: true,
            random_damage_gate: 1,
            damage: 2.0,
            current_health: 10.0,
            max_health: 14.0,
            in_water: true,
            source_entity_present: true,
            direct_entity_present: false,
        };
        assert_eq!(
            axolotl_play_dead_memory_on_hurt(triggering),
            Some(AXOLOTL_PLAY_DEAD_TICKS)
        );

        assert_eq!(
            axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
                in_water: false,
                ..triggering
            }),
            None
        );
        assert_eq!(
            axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
                current_health: 2.0,
                damage: 2.0,
                ..triggering
            }),
            None
        );
        assert_eq!(
            axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
                source_entity_present: false,
                direct_entity_present: false,
                ..triggering
            }),
            None
        );
        assert_eq!(
            axolotl_play_dead_memory_on_hurt(AxolotlHurtContext {
                random_one_in_three: false,
                ..triggering
            }),
            None
        );
        assert_eq!(AXOLOTL_DRY_OUT_DAMAGE, 2.0);
    }

    #[test]
    fn chicken_egg_flap_jockey_and_dimensions_match_java_rules() {
        assert_eq!(chicken_next_egg_time(-1), CHICKEN_EGG_TIME_MIN);
        assert_eq!(chicken_next_egg_time(0), 6000);
        assert_eq!(chicken_next_egg_time(5999), 11999);
        assert_eq!(chicken_next_egg_time(6000), 11999);
        assert_eq!(chicken_dimensions(false), (0.4, 0.7, None));
        assert_eq!(chicken_dimensions(true), (0.3, 0.4, Some(0.28)));

        let mut chicken = ChickenState::new(0);
        chicken.egg_time = 1;
        chicken.delta_y = -1.0;
        assert_eq!(
            chicken.tick(false, true, false, true, 42),
            ChickenTickEvent::LayEgg
        );
        assert_eq!(chicken.egg_time, 6042);
        assert_eq!(chicken.flap_speed, 1.0);
        assert_eq!(chicken.flapping, 0.9);
        assert_eq!(chicken.delta_y, -0.6);
        assert_eq!(chicken.flap, 1.8);

        let mut baby = ChickenState::new(0);
        baby.egg_time = 1;
        assert_eq!(baby.tick(true, true, true, true, 0), ChickenTickEvent::None);
        assert_eq!(baby.egg_time, 1);

        let mut jockey = ChickenState::new(0);
        jockey.is_chicken_jockey = true;
        jockey.egg_time = 1;
        assert_eq!(
            jockey.tick(true, true, false, true, 0),
            ChickenTickEvent::None
        );
        assert_eq!(jockey.egg_time, 1);
        assert!(jockey.remove_when_far_away());
        assert_eq!(
            jockey.base_experience_reward(1),
            CHICKEN_JOCKEY_BASE_EXPERIENCE
        );
    }

    #[test]
    fn cow_milking_dimensions_breeding_and_mooshroom_mutation_match_java_rules() {
        assert!(cow_is_food("minecraft:wheat"));
        assert!(!cow_is_food("minecraft:hay_block"));
        assert_eq!(
            cow_interaction("minecraft:bucket", false),
            CowInteraction::FillMilkBucket
        );
        assert_eq!(
            cow_interaction("minecraft:bucket", true),
            CowInteraction::Delegate
        );
        assert_eq!(
            cow_interaction("minecraft:bowl", false),
            CowInteraction::Delegate
        );
        assert_eq!(cow_dimensions(false), (0.9, 1.4, None));
        assert_eq!(cow_dimensions(true), (0.45, 0.7, Some(0.665)));

        assert_eq!(
            cow_breed_variant("minecraft:warm", "minecraft:cold", true),
            "minecraft:warm"
        );
        assert_eq!(
            cow_breed_variant("minecraft:warm", "minecraft:cold", false),
            "minecraft:cold"
        );

        assert_eq!(
            mooshroom_thunder_variant(MooshroomVariant::Red, None, "bolt-a"),
            MooshroomVariant::Brown
        );
        assert_eq!(
            mooshroom_thunder_variant(MooshroomVariant::Brown, Some("bolt-a"), "bolt-a"),
            MooshroomVariant::Brown
        );
        assert_eq!(MOOSHROOM_MUTATE_CHANCE, 1024);
        assert_eq!(
            mooshroom_offspring_variant(MooshroomVariant::Red, MooshroomVariant::Red, true, true),
            MooshroomVariant::Brown
        );
        assert_eq!(
            mooshroom_offspring_variant(
                MooshroomVariant::Brown,
                MooshroomVariant::Brown,
                true,
                true
            ),
            MooshroomVariant::Red
        );
        assert_eq!(
            mooshroom_offspring_variant(
                MooshroomVariant::Red,
                MooshroomVariant::Brown,
                false,
                false
            ),
            MooshroomVariant::Brown
        );
    }

    #[test]
    fn dolphin_moistness_feeding_treasure_and_grace_match_java_rules() {
        let mut dolphin = DolphinState::new();
        assert_eq!(dolphin.air_supply, DOLPHIN_TOTAL_AIR_SUPPLY);
        assert_eq!(dolphin.moistness, DOLPHIN_TOTAL_MOISTNESS_LEVEL);
        assert!(!dolphin.got_fish);
        assert_eq!(DOLPHIN_BABY_SCALE, 0.65);
        assert_eq!(DOLPHIN_TREASURE_SEARCH_RADIUS, 50);
        assert_eq!(DOLPHIN_TREASURE_STOP_DISTANCE, 4.0);

        dolphin.moistness = 10;
        assert_eq!(
            dolphin.tick_moistness(false, true, false),
            DolphinLandTickOutcome {
                dry_out_damage: None,
                jump_delta_y: None,
                needs_sync: false,
            }
        );
        assert_eq!(dolphin.moistness, DOLPHIN_TOTAL_MOISTNESS_LEVEL);

        dolphin.moistness = 1;
        assert_eq!(
            dolphin.tick_moistness(false, false, true),
            DolphinLandTickOutcome {
                dry_out_damage: Some(DOLPHIN_DRY_OUT_DAMAGE),
                jump_delta_y: Some(0.5),
                needs_sync: true,
            }
        );
        assert_eq!(dolphin.moistness, 0);

        dolphin.air_supply = 1;
        let no_ai_outcome = dolphin.tick_moistness(true, false, true);
        assert_eq!(dolphin.air_supply, DOLPHIN_TOTAL_AIR_SUPPLY);
        assert_eq!(no_ai_outcome.dry_out_damage, None);

        assert_eq!(
            dolphin_feed_result(false, false, 24_000),
            DolphinFeedResult::NotFish
        );
        assert_eq!(
            dolphin_feed_result(true, true, 24_000),
            DolphinFeedResult::AgeUp { seconds: 120 }
        );
        assert_eq!(
            dolphin_feed_result(true, false, 24_000),
            DolphinFeedResult::SetGotFish
        );

        dolphin.got_fish = true;
        dolphin.air_supply = 99;
        assert!(!dolphin.can_start_treasure_goal());
        dolphin.air_supply = 100;
        assert!(dolphin.can_start_treasure_goal());
        assert!(dolphin.should_clear_got_fish_on_treasure_stop(false, true, false));
        assert!(dolphin.should_clear_got_fish_on_treasure_stop(false, false, true));
        assert!(!dolphin.should_clear_got_fish_on_treasure_stop(false, false, false));

        assert_eq!(DOLPHIN_SWIM_WITH_PLAYER_RANGE, 10.0);
        assert_eq!(DOLPHIN_SWIM_WITH_PLAYER_CONTINUE_DISTANCE_SQUARED, 256.0);
        assert_eq!(dolphin_grace_refresh_ticks(true, 0), Some(100));
        assert_eq!(dolphin_grace_refresh_ticks(true, 5), None);
        assert_eq!(dolphin_grace_refresh_ticks(false, 0), None);
    }

    #[test]
    fn bee_flags_sting_hive_and_pollination_counters_match_java_rules() {
        let mut bee = BeeState::new();
        assert_eq!(bee.flags, 0);
        assert!(!bee.has_nectar());
        assert!(!bee.has_stung());
        assert!(!bee.is_rolling());
        assert_eq!(BEE_FLAG_ROLL, 2);
        assert_eq!(BEE_FLAG_HAS_STUNG, 4);
        assert_eq!(BEE_FLAG_HAS_NECTAR, 8);
        assert_eq!(BEE_STING_DEATH_COUNTDOWN, 1200);
        assert_eq!(BEE_TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME, 3600);
        assert_eq!(BEE_MAX_CROPS_GROWABLE, 10);
        assert_eq!(BEE_TOO_FAR_DISTANCE, 48);
        assert_eq!(BEE_HIVE_CLOSE_ENOUGH_DISTANCE, 2);
        assert_eq!(BEE_HIVE_SEARCH_DISTANCE, 20);
        assert_eq!(BEE_COOLDOWN_BEFORE_LOCATING_NEW_HIVE, 200);
        assert_eq!(BEE_MIN_FIND_FLOWER_RETRY_COOLDOWN, 20);
        assert_eq!(BEE_MAX_FIND_FLOWER_RETRY_COOLDOWN, 60);
        assert_eq!(BEE_PERSISTENT_ANGER_MIN_TICKS, 400);
        assert_eq!(BEE_PERSISTENT_ANGER_MAX_TICKS, 780);

        bee.ticks_without_nectar_since_exiting_hive = 12;
        bee.set_has_nectar(true);
        assert!(bee.has_nectar());
        assert_eq!(bee.ticks_without_nectar_since_exiting_hive, 0);
        bee.set_has_nectar(false);
        assert!(!bee.has_nectar());

        bee.stay_out_of_hive_countdown = 1;
        assert!(!bee.wants_to_enter_hive(false, false, true, false));
        bee.stay_out_of_hive_countdown = 0;
        assert!(bee.wants_to_enter_hive(false, false, true, false));
        assert!(!bee.wants_to_enter_hive(false, false, true, true));
        bee.set_has_stung(true);
        assert!(!bee.wants_to_enter_hive(false, false, true, false));

        bee.set_has_stung(false);
        bee.tick_ai_step(true, true, 3.99);
        assert!(bee.is_rolling());
        bee.tick_ai_step(true, true, 4.0);
        assert!(!bee.is_rolling());

        for _ in 0..20 {
            assert_eq!(
                bee.tick_server_ai(true, false).drown_damage,
                None,
                "bee only takes drown damage after more than 20 underwater ticks"
            );
        }
        assert_eq!(
            bee.tick_server_ai(true, false).drown_damage,
            Some(BEE_DROWN_DAMAGE)
        );
        assert_eq!(bee.tick_server_ai(false, false).drown_damage, None);

        bee.set_has_stung(true);
        bee.time_since_sting = 4;
        let sting_outcome = bee.tick_server_ai(false, true);
        assert!(sting_outcome.sting_death_damage);
        assert_eq!(bee.time_since_sting, 5);
        assert_eq!(bee_poison_duration_ticks("peaceful"), None);
        assert_eq!(bee_poison_duration_ticks("normal"), Some(200));
        assert_eq!(bee_poison_duration_ticks("hard"), Some(360));
    }

    #[test]
    fn camel_dash_pose_and_passenger_offsets_match_java_rules() {
        assert_eq!(CAMEL_BABY_SCALE, 0.6);
        assert_eq!(CAMEL_DASH_COOLDOWN_TICKS, 55);
        assert_eq!(CAMEL_MAX_HEAD_Y_ROT, 30);
        assert_eq!(CAMEL_RUNNING_SPEED_BONUS, 0.1);
        assert_eq!(CAMEL_DASH_VERTICAL_MOMENTUM, 1.4285);
        assert_eq!(CAMEL_DASH_HORIZONTAL_MOMENTUM, 22.2222);
        assert_eq!(CAMEL_DASH_MINIMUM_DURATION_TICKS, 5);
        assert_eq!(CAMEL_SITDOWN_DURATION_TICKS, 40);
        assert_eq!(CAMEL_STANDUP_DURATION_TICKS, 52);
        assert_eq!(CAMEL_IDLE_MINIMAL_DURATION_TICKS, 80);
        assert_eq!(CAMEL_SITTING_HEIGHT_DIFFERENCE, 1.43);
        assert_eq!(CAMEL_SITTING_EYE_HEIGHT, 0.845);

        let mut camel = CamelState::new_standing(100);
        assert_eq!(camel.last_pose_change_tick, 47);
        assert!(!camel.is_in_pose_transition(100));
        assert!(!camel.refuse_to_move(100));
        assert!(camel.can_start_dash(true, true));
        assert!(!camel.can_start_dash(false, true));
        assert!(!camel.can_start_dash(true, false));

        camel.start_dash();
        assert!(camel.dashing);
        assert_eq!(camel.dash_cooldown, CAMEL_DASH_COOLDOWN_TICKS);
        assert!(!camel.tick(true, false, false).dash_ready_sound);
        assert!(
            camel.dashing,
            "dash remains visible through the first five ticks"
        );
        for _ in 0..4 {
            camel.tick(true, false, false);
        }
        assert_eq!(camel.dash_cooldown, 50);
        camel.tick(true, false, false);
        assert!(camel.dashing);
        assert_eq!(camel.dash_cooldown, 49);
        camel.tick(true, false, false);
        assert!(!camel.dashing);
        for _ in 0..47 {
            assert!(!camel.tick(false, false, false).dash_ready_sound);
        }
        assert!(camel.tick(false, false, false).dash_ready_sound);
        assert_eq!(camel.dash_cooldown, 0);
        assert_eq!(camel.ridden_speed(0.09, true), 0.19);

        assert!(camel.sit_down(200));
        assert!(camel.is_sitting());
        assert_eq!(camel.last_pose_change_tick, -200);
        assert!(camel.refuse_to_move(239));
        assert!(
            camel.refuse_to_move(240),
            "sitting camels refuse movement after transition too"
        );
        assert!(camel.stand_up(300));
        assert!(!camel.is_sitting());
        assert!(camel.is_in_pose_transition(351));
        assert!(!camel.is_in_pose_transition(352));
        camel.stand_up_instantly(400);
        assert_eq!(camel.last_pose_change_tick, 347);

        let (horizontal, vertical) = camel_dash_impulse(1.0, 0.09, 1.0, 0.42);
        assert!((horizontal - 1.999998).abs() < 0.00001);
        assert!((vertical - 0.59997).abs() < 0.00001);

        let driver = camel_passenger_attachment_point(0, 2, false, false, false, 1.7, 2.375, 1.0);
        assert_eq!(driver.x, 0.0);
        assert!((driver.y - 2.0).abs() < 0.00001);
        assert!((driver.z - 0.5).abs() < 0.00001);
        let rear_animal =
            camel_passenger_attachment_point(1, 2, true, false, false, 1.7, 2.375, 1.0);
        assert!((rear_animal.z - -0.5).abs() < 0.00001);
        assert!(CamelState::can_add_passenger(2));
        assert!(!CamelState::can_add_passenger(3));
    }

    #[test]
    fn goat_milking_horns_ram_and_head_lowering_match_java_rules() {
        assert_eq!(GOAT_SCREAMING_CHANCE_DENOMINATOR, 50);
        assert_eq!(GOAT_INITIAL_MISSING_HORN_CHANCE_DENOMINATOR, 10);
        assert_eq!(GOAT_RAM_PREPARE_TIME, 20);
        assert_eq!(GOAT_RAM_MIN_DISTANCE, 4);
        assert_eq!(GOAT_RAM_MAX_DISTANCE, 7);
        assert_eq!(GOAT_TIME_BETWEEN_RAMS_MIN, 600);
        assert_eq!(GOAT_TIME_BETWEEN_RAMS_MAX, 6000);
        assert_eq!(GOAT_TIME_BETWEEN_RAMS_SCREAMER_MIN, 100);
        assert_eq!(GOAT_TIME_BETWEEN_RAMS_SCREAMER_MAX, 300);
        assert_eq!(GOAT_TIME_BETWEEN_LONG_JUMPS_MIN, 600);
        assert_eq!(GOAT_TIME_BETWEEN_LONG_JUMPS_MAX, 1200);
        assert_eq!(GOAT_MAX_LONG_JUMP_HEIGHT, 5);
        assert_eq!(GOAT_MAX_LONG_JUMP_WIDTH, 5);
        assert_eq!(GOAT_MAX_JUMP_VELOCITY_MULTIPLIER, 3.5714288);
        assert_eq!(GOAT_PREPARE_RAM_SPEED_MULTIPLIER, 1.25);
        assert_eq!(GOAT_RAMMING_SPEED_MULTIPLIER, 3.0);
        assert_eq!(GOAT_ADULT_RAM_KNOCKBACK_FORCE, 2.5);
        assert_eq!(GOAT_BABY_RAM_KNOCKBACK_FORCE, 1.0);
        assert!((GOAT_LONG_JUMPING_WIDTH - 0.63).abs() < 0.00001);
        assert!((GOAT_LONG_JUMPING_HEIGHT - 0.91).abs() < 0.00001);

        assert_eq!(
            goat_interaction("minecraft:bucket", false),
            GoatInteraction::Milk
        );
        assert_eq!(
            goat_interaction("minecraft:bucket", true),
            GoatInteraction::Delegate
        );
        assert_eq!(
            goat_interaction("minecraft:wheat", false),
            GoatInteraction::Delegate
        );
        assert_eq!(goat_ram_knockback_force(false), 2.5);
        assert_eq!(goat_ram_knockback_force(true), 1.0);

        let normal = GoatState::finalize_spawn(false, true, false, true);
        assert!(!normal.is_screaming);
        assert!(normal.has_left_horn);
        assert!(normal.has_right_horn);
        assert_eq!(normal.ram_cooldown_range(), (600, 6000));

        let mut screaming_missing_left = GoatState::finalize_spawn(true, true, true, true);
        assert!(screaming_missing_left.is_screaming);
        assert!(!screaming_missing_left.has_left_horn);
        assert!(screaming_missing_left.has_right_horn);
        assert_eq!(screaming_missing_left.ram_cooldown_range(), (100, 300));
        assert_eq!(
            screaming_missing_left.drop_horn(false, true),
            GoatHornDrop::Right
        );
        assert!(!screaming_missing_left.has_right_horn);
        assert_eq!(
            screaming_missing_left.drop_horn(false, true),
            GoatHornDrop::None
        );

        let baby_missing_roll = GoatState::finalize_spawn(true, false, true, false);
        assert!(baby_missing_roll.has_left_horn);
        assert!(baby_missing_roll.has_right_horn);
        let mut both_horns = GoatState::new();
        assert_eq!(both_horns.drop_horn(true, true), GoatHornDrop::None);
        assert_eq!(both_horns.drop_horn(false, false), GoatHornDrop::Right);
        assert!(both_horns.has_left_horn);
        assert!(!both_horns.has_right_horn);

        let mut lowering = GoatState::new();
        for _ in 0..25 {
            lowering.tick_lower_head(true);
        }
        assert_eq!(lowering.lower_head_tick, GOAT_MAX_LOWER_HEAD_TICK);
        let adult_rot = lowering.ramming_x_head_rot_radians(false);
        let baby_rot = lowering.ramming_x_head_rot_radians(true);
        assert!((adult_rot - std::f32::consts::PI / 6.0).abs() < 0.00001);
        assert!((baby_rot - (52.5_f32 * std::f32::consts::PI / 180.0)).abs() < 0.00001);
        lowering.tick_lower_head(false);
        assert_eq!(lowering.lower_head_tick, 18);
        for _ in 0..20 {
            lowering.tick_lower_head(false);
        }
        assert_eq!(lowering.lower_head_tick, 0);
    }

    #[test]
    fn pig_saddle_boost_food_on_a_stick_and_lightning_match_java_rules() {
        assert_eq!(PIG_MAX_HEALTH, 10.0);
        assert_eq!(PIG_MOVEMENT_SPEED, 0.25);
        assert_eq!(PIG_RIDDEN_SPEED_FACTOR, 0.225);
        assert_eq!(PIG_BOOST_MIN_TIME, 140);
        assert_eq!(PIG_BOOST_MAX_TIME, 980);
        assert_eq!(PIG_BOOST_RANDOM_BOUND, 841);
        assert_eq!(PIG_BOOST_SPEED_AMPLIFIER, 1.15);
        assert_eq!(PIG_CARROT_ON_A_STICK_DURABILITY, 25);
        assert_eq!(PIG_CARROT_ON_A_STICK_DAMAGE_PER_BOOST, 7);
        assert_eq!(PIG_TEMPT_SPEED, 1.2);
        assert_eq!(PIG_PANIC_SPEED, 1.25);
        assert_eq!(PIG_FOLLOW_PARENT_SPEED, 1.1);
        assert_eq!(PIG_LEASH_EYE_HEIGHT_FACTOR, 0.6);
        assert_eq!(PIG_LEASH_WIDTH_FACTOR, 0.4);

        let mut pig = PigState::new();
        assert!(!pig.controlling_passenger(true, true));
        pig.saddled = true;
        assert!(pig.controlling_passenger(true, true));
        assert!(!pig.controlling_passenger(false, true));
        assert!(!pig.controlling_passenger(true, false));
        assert!(PigState::can_use_saddle_slot(true, false));
        assert!(!PigState::can_use_saddle_slot(false, false));
        assert!(!PigState::can_use_saddle_slot(true, true));

        assert_eq!(
            pig_interaction(false, true, false, false, false, false),
            PigInteraction::StartRide
        );
        assert_eq!(
            pig_interaction(true, true, false, false, true, false),
            PigInteraction::DelegateToAnimal
        );
        assert_eq!(
            pig_interaction(false, true, true, false, false, true),
            PigInteraction::EquipSaddle
        );
        assert_eq!(
            pig_interaction(false, false, false, false, false, false),
            PigInteraction::Pass
        );

        assert!(pig.boost(0));
        assert_eq!(pig.boost_time_total, PIG_BOOST_MIN_TIME);
        assert_eq!(pig.boost_time, 0);
        assert!(!pig.boost(840), "ItemBasedSteering rejects nested boosts");
        pig.boost_time = pig.boost_time_total / 2;
        assert!((pig.boost_factor() - 2.15).abs() < 0.00001);
        assert!((pig.ridden_speed(PIG_MOVEMENT_SPEED) - 0.1209375).abs() < 0.00001);
        pig.boost_time = pig.boost_time_total;
        pig.tick_boost();
        assert!(
            pig.boosting,
            "Java stops only after old boostTime is greater than total"
        );
        pig.tick_boost();
        assert!(!pig.boosting);
        assert_eq!(pig.boost_factor(), 1.0);

        let mut max_boost = PigState::new();
        assert!(max_boost.boost(840));
        assert_eq!(max_boost.boost_time_total, PIG_BOOST_MAX_TIME);
        assert!(max_boost.boost(999) == false);

        assert_eq!(
            pig_food_on_a_stick_use(true, true, true, true, 0),
            PigBoostUseResult::BoostStarted {
                damage: 7,
                converts_to_fishing_rod: false,
            }
        );
        assert_eq!(
            pig_food_on_a_stick_use(true, true, true, true, 18),
            PigBoostUseResult::BoostStarted {
                damage: 7,
                converts_to_fishing_rod: true,
            }
        );
        assert_eq!(
            pig_food_on_a_stick_use(false, true, true, true, 0),
            PigBoostUseResult::Pass
        );

        assert!(!pig_thunder_converts_to_zombified_piglin("peaceful"));
        assert!(pig_thunder_converts_to_zombified_piglin("easy"));
        assert_eq!(
            pig_offspring_variant("minecraft:temperate", "minecraft:cold", true),
            "minecraft:temperate"
        );
        assert_eq!(
            pig_offspring_variant("minecraft:temperate", "minecraft:cold", false),
            "minecraft:cold"
        );
    }

    #[test]
    fn polar_bear_cub_targeting_standing_warning_and_swim_match_java_rules() {
        assert_eq!(POLAR_BEAR_MAX_HEALTH, 30.0);
        assert_eq!(POLAR_BEAR_FOLLOW_RANGE, 20.0);
        assert_eq!(POLAR_BEAR_PLAYER_ATTACK_FOLLOW_DISTANCE_FACTOR, 0.5);
        assert_eq!(
            POLAR_BEAR_FOLLOW_RANGE * POLAR_BEAR_PLAYER_ATTACK_FOLLOW_DISTANCE_FACTOR,
            10.0
        );
        assert_eq!(POLAR_BEAR_MOVEMENT_SPEED, 0.25);
        assert_eq!(POLAR_BEAR_ATTACK_DAMAGE, 6.0);
        assert_eq!(POLAR_BEAR_MELEE_SPEED, 1.25);
        assert_eq!(POLAR_BEAR_PANIC_SPEED, 2.0);
        assert_eq!(POLAR_BEAR_FOLLOW_PARENT_SPEED, 1.25);
        assert_eq!(POLAR_BEAR_RANDOM_STROLL_SPEED, 1.0);
        assert_eq!(POLAR_BEAR_LOOK_AT_PLAYER_DISTANCE, 6.0);
        assert_eq!(POLAR_BEAR_STAND_ANIMATION_TICKS, 6.0);
        assert_eq!(POLAR_BEAR_WARNING_SOUND_COOLDOWN_TICKS, 40);
        assert_eq!(POLAR_BEAR_WARNING_ATTACK_TICKS, 10);
        assert_eq!(POLAR_BEAR_CUB_ALERT_XZ_RANGE, 8.0);
        assert_eq!(POLAR_BEAR_CUB_ALERT_Y_RANGE, 4.0);
        assert_eq!(POLAR_BEAR_PERSISTENT_ANGER_MIN_TICKS, 400);
        assert_eq!(POLAR_BEAR_PERSISTENT_ANGER_MAX_TICKS, 780);
        assert_eq!(POLAR_BEAR_WATER_SLOWDOWN, 0.98);

        assert!(polar_bear_should_attack_player(false, true, true));
        assert!(!polar_bear_should_attack_player(true, true, true));
        assert!(!polar_bear_should_attack_player(false, false, true));
        assert!(!polar_bear_should_attack_player(false, true, false));
        assert!(polar_bear_should_attack_fox(false));
        assert!(!polar_bear_should_attack_fox(true));
        assert!(polar_bear_should_alert_other_on_hurt(true, false));
        assert!(!polar_bear_should_alert_other_on_hurt(true, true));
        assert!(!polar_bear_should_alert_other_on_hurt(false, false));

        let mut bear = PolarBearState::new();
        assert!(bear.play_warning_sound());
        assert_eq!(bear.warning_sound_ticks, 40);
        assert!(!bear.play_warning_sound());
        bear.tick(false);
        assert_eq!(bear.warning_sound_ticks, 39);
        bear.warning_sound_ticks = 0;
        assert!(bear.play_warning_sound());

        bear.standing = true;
        for _ in 0..10 {
            bear.tick(true);
        }
        assert_eq!(
            bear.client_stand_animation,
            POLAR_BEAR_STAND_ANIMATION_TICKS
        );
        assert_eq!(bear.dimensions_height_scale(), 2.0);
        bear.standing = false;
        bear.tick(true);
        assert_eq!(bear.client_stand_animation, 5.0);
        assert!((bear.standing_animation_scale(0.5) - (5.5 / 6.0)).abs() < 0.00001);

        assert_eq!(
            polar_bear_melee_signal(true, 100.0, 0.6, 20, false),
            PolarBearMeleeSignal::AttackAndStopStanding
        );
        assert_eq!(
            polar_bear_melee_signal(false, 12.0, 0.6, 10, false),
            PolarBearMeleeSignal::StandAndWarn
        );
        assert_eq!(
            polar_bear_melee_signal(false, 12.0, 0.6, 11, true),
            PolarBearMeleeSignal::StopStanding
        );
        assert_eq!(
            polar_bear_melee_signal(false, 13.0, 0.6, 10, false),
            PolarBearMeleeSignal::StopStanding
        );
    }

    #[test]
    fn rabbit_variants_garden_raid_and_jump_timing_match_java_rules() {
        assert_eq!(RABBIT_MAX_HEALTH, 3.0);
        assert_eq!(RABBIT_MOVEMENT_SPEED, 0.3);
        assert_eq!(RABBIT_ATTACK_DAMAGE, 3.0);
        assert_eq!(RABBIT_EVIL_ATTACK_POWER_INCREMENT, 5.0);
        assert_eq!(RABBIT_EVIL_ARMOR_VALUE, 8.0);
        assert_eq!(RABBIT_STROLL_SPEED_MOD, 0.6);
        assert_eq!(RABBIT_BREED_SPEED_MOD, 0.8);
        assert_eq!(RABBIT_FOLLOW_SPEED_MOD, 1.0);
        assert_eq!(RABBIT_FLEE_SPEED_MOD, 2.2);
        assert_eq!(RABBIT_ATTACK_SPEED_MOD, 1.4);
        assert_eq!(RABBIT_BABY_JUMP_HEIGHT, 0.5);
        assert_eq!(RABBIT_ADULT_JUMP_HEIGHT, 1.5);
        assert_eq!(RABBIT_JUMP_DELAY_TICKS, 10);
        assert_eq!(RABBIT_PANIC_JUMP_DELAY_TICKS, 3);
        assert_eq!(RABBIT_JUMP_DURATION_TICKS, 15);
        assert_eq!(RABBIT_MORE_CARROTS_DELAY, 40);
        assert_eq!(RABBIT_IDLE_MINIMAL_DURATION_TICKS, 180);
        assert_eq!(RABBIT_BABY_WIDTH, 0.24);
        assert_eq!(RABBIT_BABY_HEIGHT, 0.4);
        assert_eq!(RABBIT_BABY_EYE_HEIGHT, 0.39);

        let variants = [
            (RabbitVariant::Brown, 0, "brown"),
            (RabbitVariant::White, 1, "white"),
            (RabbitVariant::Black, 2, "black"),
            (RabbitVariant::WhiteSplotched, 3, "white_splotched"),
            (RabbitVariant::Gold, 4, "gold"),
            (RabbitVariant::Salt, 5, "salt"),
            (RabbitVariant::Evil, 99, "evil"),
        ];
        for (variant, id, name) in variants {
            assert_eq!(variant.id(), id);
            assert_eq!(variant.name(), name);
            assert_eq!(RabbitVariant::by_id(id), variant);
        }
        assert_eq!(RabbitVariant::by_id(12345), RabbitVariant::Brown);
        assert!(RabbitVariant::Evil.is_evil());
        assert!(!RabbitVariant::Brown.is_evil());

        assert_eq!(rabbit_random_variant(true, false, 79), RabbitVariant::White);
        assert_eq!(
            rabbit_random_variant(true, false, 80),
            RabbitVariant::WhiteSplotched
        );
        assert_eq!(rabbit_random_variant(false, true, 99), RabbitVariant::Gold);
        assert_eq!(
            rabbit_random_variant(false, false, 49),
            RabbitVariant::Brown
        );
        assert_eq!(rabbit_random_variant(false, false, 89), RabbitVariant::Salt);
        assert_eq!(
            rabbit_random_variant(false, false, 90),
            RabbitVariant::Black
        );

        assert_eq!(
            rabbit_offspring_variant(
                RabbitVariant::Gold,
                RabbitVariant::Brown,
                RabbitVariant::Salt,
                0,
                true,
            ),
            RabbitVariant::Gold
        );
        assert_eq!(
            rabbit_offspring_variant(
                RabbitVariant::Gold,
                RabbitVariant::Brown,
                RabbitVariant::Salt,
                1,
                true,
            ),
            RabbitVariant::Salt
        );
        assert_eq!(
            rabbit_offspring_variant(
                RabbitVariant::Gold,
                RabbitVariant::Brown,
                RabbitVariant::Salt,
                1,
                false,
            ),
            RabbitVariant::Brown
        );

        let mut rabbit = RabbitState::new();
        assert!(rabbit.wants_more_food());
        rabbit.more_carrot_ticks = 40;
        assert!(!rabbit.wants_more_food());
        rabbit.tick_more_carrots(2);
        assert_eq!(rabbit.more_carrot_ticks, 38);
        rabbit.more_carrot_ticks = 1;
        rabbit.tick_more_carrots(2);
        assert_eq!(rabbit.more_carrot_ticks, 0);

        assert_eq!(
            rabbit_raid_garden(true, true, true, Some(0)),
            RabbitRaidGardenResult::DestroyCrop
        );
        assert_eq!(
            rabbit_raid_garden(true, true, true, Some(7)),
            RabbitRaidGardenResult::ReduceCarrotAge(6)
        );
        assert_eq!(
            rabbit_raid_garden(false, true, true, Some(7)),
            RabbitRaidGardenResult::Noop
        );
        assert!(rabbit_should_avoid_entity(RabbitVariant::Brown, true));
        assert!(!rabbit_should_avoid_entity(RabbitVariant::Evil, true));

        rabbit.start_jumping();
        assert_eq!(rabbit.jump_duration, 15);
        assert_eq!(rabbit.jump_ticks, 0);
        rabbit.jump_ticks = 7;
        assert!((rabbit.jump_completion(0.5) - 0.5).abs() < 0.00001);
        rabbit.jump_ticks = rabbit.jump_duration;
        rabbit.ai_step_jump();
        assert_eq!(rabbit.jump_ticks, 0);
        assert_eq!(rabbit.jump_duration, 0);
        rabbit.set_landing_delay(0.6);
        assert_eq!(rabbit.jump_delay_ticks, 10);
        rabbit.set_landing_delay(2.2);
        assert_eq!(rabbit.jump_delay_ticks, 3);
    }

    #[test]
    fn sheep_wool_shearing_eating_and_color_rules_match_java() {
        assert_eq!(SHEEP_EAT_ANIMATION_TICKS, 40);
        assert_eq!(SHEEP_SHEARED_FLAG, 16);
        assert_eq!(SHEEP_COLOR_MASK, 15);
        assert_eq!(SHEEP_MAX_HEALTH, 8.0);
        assert_eq!(SHEEP_MOVEMENT_SPEED, 0.23);
        assert_eq!(SHEEP_PANIC_SPEED, 1.25);
        assert_eq!(SHEEP_BREED_SPEED, 1.0);
        assert_eq!(SHEEP_TEMPT_SPEED, 1.1);
        assert_eq!(SHEEP_FOLLOW_PARENT_SPEED, 1.1);
        assert_eq!(SHEEP_STROLL_SPEED, 1.0);
        assert_eq!(SHEEP_LOOK_AT_PLAYER_DISTANCE, 6.0);
        assert_eq!(SHEEP_ATE_AGE_UP_SECONDS, 60);

        let colors = [
            DyeColorModel::White,
            DyeColorModel::Orange,
            DyeColorModel::Magenta,
            DyeColorModel::LightBlue,
            DyeColorModel::Yellow,
            DyeColorModel::Lime,
            DyeColorModel::Pink,
            DyeColorModel::Gray,
            DyeColorModel::LightGray,
            DyeColorModel::Cyan,
            DyeColorModel::Purple,
            DyeColorModel::Blue,
            DyeColorModel::Brown,
            DyeColorModel::Green,
            DyeColorModel::Red,
            DyeColorModel::Black,
        ];
        for (id, color) in colors.into_iter().enumerate() {
            assert_eq!(color.id(), id as u8);
            assert_eq!(DyeColorModel::by_id(id as u8), color);
        }
        assert_eq!(DyeColorModel::by_id(99), DyeColorModel::White);

        let mut sheep = SheepState::new();
        assert_eq!(sheep.color(), DyeColorModel::White);
        assert!(!sheep.is_sheared());
        sheep.set_color(DyeColorModel::Blue);
        assert_eq!(sheep.wool_data, 11);
        sheep.set_sheared(true);
        assert_eq!(sheep.wool_data, 27);
        assert_eq!(sheep.color(), DyeColorModel::Blue);
        assert!(sheep.is_sheared());
        assert!(!sheep.ready_for_shearing(true, false));
        sheep.set_sheared(false);
        assert_eq!(sheep.wool_data, 11);
        assert!(sheep.ready_for_shearing(true, false));
        assert!(!sheep.ready_for_shearing(false, false));
        assert!(!sheep.ready_for_shearing(true, true));

        assert_eq!(
            sheep_interaction("minecraft:shears", true, true),
            SheepInteraction::ShearServer
        );
        assert_eq!(
            sheep_interaction("minecraft:shears", false, true),
            SheepInteraction::ConsumeClientOrNotReady
        );
        assert_eq!(
            sheep_interaction("minecraft:wheat", true, true),
            SheepInteraction::Delegate
        );

        sheep.set_sheared(true);
        assert_eq!(sheep.ate(true), Some(60));
        assert!(!sheep.is_sheared());
        sheep.set_sheared(true);
        assert_eq!(sheep.ate(false), None);
        assert!(!sheep.is_sheared());
        assert!(sheep.handle_entity_event(10));
        assert_eq!(sheep.eat_animation_tick, 40);
        assert!(!sheep.handle_entity_event(9));
        sheep.client_ai_step();
        assert_eq!(sheep.eat_animation_tick, 39);
        sheep.eat_animation_tick = 0;
        assert_eq!(sheep.head_eat_position_scale(0.0), 0.0);
        sheep.eat_animation_tick = 3;
        assert!((sheep.head_eat_position_scale(1.0) - 0.5).abs() < 0.00001);
        sheep.eat_animation_tick = 20;
        assert_eq!(sheep.head_eat_position_scale(0.0), 1.0);
        sheep.eat_animation_tick = 39;
        assert!((sheep.head_eat_position_scale(1.0) - 0.5).abs() < 0.00001);
        assert!(sheep.head_eat_angle_scale(0.0, 30.0) > 0.0);
        sheep.eat_animation_tick = 0;
        assert!(
            (sheep.head_eat_angle_scale(0.0, 30.0) - std::f32::consts::PI / 6.0).abs() < 0.00001
        );

        assert_eq!(sheep_spawn_color(false, false, 0, 0), DyeColorModel::Black);
        assert_eq!(sheep_spawn_color(false, false, 5, 0), DyeColorModel::Gray);
        assert_eq!(
            sheep_spawn_color(false, false, 10, 0),
            DyeColorModel::LightGray
        );
        assert_eq!(sheep_spawn_color(false, false, 15, 0), DyeColorModel::Brown);
        assert_eq!(sheep_spawn_color(false, false, 18, 0), DyeColorModel::White);
        assert_eq!(
            sheep_spawn_color(false, false, 18, 499),
            DyeColorModel::Pink
        );
        assert_eq!(sheep_spawn_color(true, false, 18, 0), DyeColorModel::Brown);
        assert_eq!(sheep_spawn_color(false, true, 18, 0), DyeColorModel::Black);
        assert_eq!(
            sheep_offspring_color(
                Some(DyeColorModel::Purple),
                DyeColorModel::Red,
                DyeColorModel::Blue,
                true,
            ),
            DyeColorModel::Purple
        );
        assert_eq!(
            sheep_offspring_color(None, DyeColorModel::Red, DyeColorModel::Blue, false),
            DyeColorModel::Blue
        );
    }

    #[test]
    fn squid_and_glow_squid_ink_flee_and_dark_ticks_match_java_rules() {
        assert_eq!(SQUID_MAX_HEALTH, 10.0);
        assert_eq!(SQUID_DEFAULT_GRAVITY, 0.08);
        assert_eq!(SQUID_SOUND_VOLUME, 0.4);
        assert_eq!(SQUID_BABY_WIDTH, 0.5);
        assert_eq!(SQUID_BABY_HEIGHT, 0.63);
        assert_eq!(SQUID_BABY_EYE_HEIGHT, 0.37);
        assert_eq!(SQUID_INK_PARTICLE_COUNT, 30);
        assert_eq!(SQUID_INK_BABY_OFFSET_SCALE, 0.1);
        assert_eq!(SQUID_INK_ADULT_OFFSET_SCALE, 0.3);
        assert_eq!(SQUID_FLEE_SPEED, 3.0);
        assert_eq!(SQUID_FLEE_MIN_DISTANCE, 5.0);
        assert_eq!(SQUID_FLEE_MAX_DISTANCE, 10.0);
        assert_eq!(SQUID_FLEE_DISTANCE_SQUARED, 100.0);
        assert_eq!(SQUID_FLEE_VECTOR_SCALE, 20.0);
        assert_eq!(SQUID_BUBBLE_INTERVAL_TICKS, 10);
        assert_eq!(SQUID_BUBBLE_PHASE_TICK, 5);
        assert_eq!(GLOW_SQUID_DEFAULT_DARK_TICKS_REMAINING, 0);
        assert_eq!(GLOW_SQUID_DARK_TICKS_ON_HURT, 100);
        assert_eq!(GLOW_SQUID_SPAWN_SEA_LEVEL_OFFSET, 33);

        let mut squid = SquidState::new(0.0);
        assert_eq!(squid.tentacle_speed, 0.2);
        assert!(!squid.has_movement_vector());
        squid.movement_vector = (0.01, 0.0, 0.0);
        assert!(squid.has_movement_vector());
        squid.tentacle_movement = 2.0;
        assert!(squid.handle_entity_event(19));
        assert_eq!(squid.tentacle_movement, 0.0);
        assert!(!squid.handle_entity_event(18));

        assert!(squid_hurt_spawns_ink(true, true));
        assert!(!squid_hurt_spawns_ink(true, false));
        assert!(!squid_hurt_spawns_ink(false, true));
        assert!(squid_flee_can_use(true, true, 99.99));
        assert!(!squid_flee_can_use(true, true, 100.0));
        assert!(!squid_flee_can_use(false, true, 1.0));

        let flee = squid_flee_vector((0.0, 0.0, 0.0), (-5.0, 0.0, 0.0), true, false)
            .expect("water target produces flee vector");
        assert!((flee.0 - 0.15).abs() < 0.00001);
        assert_eq!(flee.1, 0.0);
        assert_eq!(flee.2, 0.0);
        let far_flee = squid_flee_vector((0.0, 0.0, 0.0), (-10.0, 0.0, 0.0), true, false)
            .expect("10 block target still has a Java flee vector");
        assert!((far_flee.0 - 0.1).abs() < 0.00001);
        assert_eq!(far_flee.1, 0.0);
        assert_eq!(far_flee.2, 0.0);
        let air_flee = squid_flee_vector((0.0, 0.0, 0.0), (0.0, -5.0, 0.0), false, true)
            .expect("air target is allowed but clamps vertical vector");
        assert_eq!(air_flee.1, 0.0);
        assert_eq!(
            squid_flee_vector((0.0, 0.0, 0.0), (-5.0, 0.0, 0.0), false, false),
            None
        );
        assert!(squid_flee_emits_bubble(5));
        assert!(squid_flee_emits_bubble(15));
        assert!(!squid_flee_emits_bubble(6));

        let mut glow = GlowSquidState::new();
        assert_eq!(glow.dark_ticks_remaining, 0);
        glow.on_hurt(true);
        assert_eq!(glow.dark_ticks_remaining, 100);
        glow.ai_step();
        assert_eq!(glow.dark_ticks_remaining, 99);
        glow.on_hurt(false);
        assert_eq!(glow.dark_ticks_remaining, 99);
        assert!(glow_squid_spawn_allowed(30, 63, 0, true));
        assert!(!glow_squid_spawn_allowed(31, 63, 0, true));
        assert!(!glow_squid_spawn_allowed(30, 63, 1, true));
        assert!(!glow_squid_spawn_allowed(30, 63, 0, false));
    }

    #[test]
    fn creeper_fuse_ignition_power_and_cloud_match_java_rules() {
        let mut creeper = CreeperState::new();
        assert_eq!(creeper.swell_dir, CREEPER_DEFAULT_SWELL_DIR);
        assert_eq!(creeper.max_swell, 30);
        assert_eq!(creeper.explosion_radius, 3);
        assert_eq!(CREEPER_MOVEMENT_SPEED, 0.25);
        assert_eq!(CREEPER_CAT_AVOID_DISTANCE, 6.0);
        assert_eq!(CREEPER_LOOK_AT_PLAYER_DISTANCE, 8.0);

        creeper.cause_fall_damage(40.0);
        assert_eq!(creeper.swell, 25);
        assert_eq!(creeper.swelling(0.5), 12.5 / 28.0);

        let loaded = CreeperState::read_save_data(true, Some(40), Some(5), true);
        assert!(loaded.powered);
        assert!(loaded.ignited);
        assert_eq!(loaded.max_swell, 40);
        assert_eq!(loaded.explosion_radius, 5);
        assert_eq!(loaded.effective_explosion_radius(), 10.0);

        assert_eq!(
            creeper_igniter_use(false, false),
            CreeperIgniterUse::NotIgniter
        );
        assert_eq!(
            creeper_igniter_use(true, false),
            CreeperIgniterUse::ConsumeOne
        );
        assert_eq!(
            creeper_igniter_use(true, true),
            CreeperIgniterUse::DamageOne
        );

        let mut ticking = CreeperState::new();
        ticking.ignite();
        let first = ticking.tick(0);
        assert!(first.primed_sound);
        assert!(first.prime_fuse_game_event);
        assert_eq!(ticking.swell_dir, 1);
        assert_eq!(ticking.swell, 1);
        let second = ticking.tick(0);
        assert!(!second.primed_sound);
        assert_eq!(ticking.swell, 2);

        ticking.swell = ticking.max_swell - 1;
        let exploded = ticking.tick(2);
        assert_eq!(ticking.swell, ticking.max_swell);
        assert!(!ticking.alive);
        assert_eq!(exploded.explosion_radius, Some(3.0));
        assert_eq!(exploded.lingering_cloud, Some(creeper_lingering_cloud()));
        assert_eq!(
            exploded.lingering_cloud.unwrap(),
            CreeperLingeringCloud {
                radius: 2.5,
                radius_on_use: -0.5,
                wait_time: 10,
                duration: 300,
                potion_duration_scale: 0.25,
                radius_per_tick: -2.5 / 300.0,
            }
        );

        let mut powered = CreeperState::new();
        powered.thunder_hit();
        assert_eq!(powered.effective_explosion_radius(), 6.0);
        assert!(powered.killed_entity_drops_charged_creeper_loot(true));
        assert!(!powered.killed_entity_drops_charged_creeper_loot(true));
        assert!(!CreeperState::new().killed_entity_drops_charged_creeper_loot(true));
        assert!(powered.can_target(false));
        assert!(!powered.can_target(true));
    }

    #[test]
    fn slime_and_magma_cube_size_split_spawn_and_jump_match_java_rules() {
        assert_eq!(clamp_slime_size(0), 1);
        assert_eq!(clamp_slime_size(200), 127);
        assert_eq!(SLIME_MAX_NATURAL_SIZE, 4);
        assert_eq!(SLIME_ATTACK_TARGET_VERTICAL_RANGE, 4.0);
        assert_eq!(SLIME_ATTACK_GROW_TIRED_TICKS, 300);
        assert_eq!(SLIME_RANDOM_DIRECTION_MIN_TICKS, 40);
        assert_eq!(SLIME_RANDOM_DIRECTION_RANDOM_BOUND, 60);
        assert_eq!(SLIME_FLOAT_WANTED_MOVEMENT, 1.2);
        assert_eq!(SLIME_KEEP_JUMPING_WANTED_MOVEMENT, 1.0);

        assert_eq!(slime_finalize_spawn_size(0, 0.0, 0.0), 1);
        assert_eq!(slime_finalize_spawn_size(0, 1.0, 0.49), 2);
        assert_eq!(slime_finalize_spawn_size(1, 1.0, 0.49), 4);
        assert_eq!(slime_finalize_spawn_size(2, 1.0, 0.0), 4);

        let slime = SlimeFamilyState::new(SlimeFamilyKind::Slime, 4);
        assert_eq!(slime.serialized_size(), 3);
        assert_eq!(
            slime.attributes(),
            SlimeFamilyAttributes {
                max_health: 16.0,
                movement_speed: 0.6,
                attack_damage: 4.0,
                armor: 0.0,
                xp_reward: 4,
            }
        );
        assert!(slime.deals_damage(true));
        assert!(!SlimeFamilyState::new(SlimeFamilyKind::Slime, 1).deals_damage(true));
        assert_eq!(slime.jump_delay(19, false), 29);
        assert_eq!(slime.jump_delay(19, true), 9);
        assert_eq!(
            slime.float_goal_step(true, false, true, 0.79),
            SlimeFloatGoalStep {
                can_use: true,
                jump: true,
                wanted_movement: 1.2,
            }
        );
        assert_eq!(
            slime.float_goal_step(false, false, true, 0.0),
            SlimeFloatGoalStep {
                can_use: false,
                jump: false,
                wanted_movement: 0.0,
            }
        );
        assert!(slime.keep_on_jumping_can_use(false));
        assert!(!slime.keep_on_jumping_can_use(true));
        assert!(slime.random_direction_can_use(false, false, true, false, false, true));
        assert!(!slime.random_direction_can_use(true, true, false, false, false, true));
        assert_eq!(
            slime.move_control_step(true, true, 1.2, 0.6, 0, 5, true),
            SlimeMoveControlStep {
                speed: 0.72,
                jump: true,
                play_jump_sound: true,
                next_jump_delay: 5,
                zero_strafe: false,
            }
        );
        assert_eq!(
            slime.move_control_step(true, true, 1.2, 0.6, 3, 5, false),
            SlimeMoveControlStep {
                speed: 0.0,
                jump: false,
                play_jump_sound: false,
                next_jump_delay: 2,
                zero_strafe: true,
            }
        );
        assert_eq!(
            slime
                .move_control_step(true, false, 1.2, 0.6, 3, 5, false)
                .speed,
            0.72
        );
        assert_eq!(slime.sound_volume(), 1.6);
        assert_eq!(slime.passenger_attachment_y(2.04, 1.0), 1.9775);

        let children = slime.split_children(2);
        assert_eq!(children.len(), 4);
        assert_eq!(
            children[0],
            SlimeSplitChild {
                size: 2,
                x_offset: -1.0,
                y_offset: 0.5,
                z_offset: -1.0,
            }
        );
        assert_eq!(children[3].size, 2);
        assert!(SlimeFamilyState::new(SlimeFamilyKind::Slime, 1)
            .split_children(2)
            .is_empty());

        let mut squish =
            SlimeFamilyState::read_save_data(SlimeFamilyKind::Slime, Some(3), Some(false));
        assert_eq!(squish.size, 4);
        assert_eq!(squish.tick_squish(true), 128);
        assert_eq!(squish.target_squish, -0.3);
        assert_eq!(squish.tick_squish(false), 0);
        assert_eq!(squish.target_squish, 0.6);

        let surface = SlimeSpawnRuleInput {
            peaceful: false,
            spawner_reason: false,
            mob_spawn_rules_pass: true,
            allows_surface_slime_spawns_biome: true,
            y: 60,
            surface_slime_spawn_chance: 0.25,
            surface_random_float: 0.24,
            max_local_raw_brightness: 3,
            brightness_random_bound_8: 3,
            worldgen_level: false,
            slime_chunk: false,
            underground_random_bound_10: 9,
        };
        assert!(slime_spawn_allowed(surface));
        assert!(!slime_spawn_allowed(SlimeSpawnRuleInput {
            peaceful: true,
            ..surface
        }));
        assert!(slime_spawn_allowed(SlimeSpawnRuleInput {
            spawner_reason: true,
            allows_surface_slime_spawns_biome: false,
            mob_spawn_rules_pass: true,
            ..surface
        }));
        assert!(slime_spawn_allowed(SlimeSpawnRuleInput {
            allows_surface_slime_spawns_biome: false,
            y: 20,
            surface_random_float: 1.0,
            worldgen_level: true,
            slime_chunk: true,
            underground_random_bound_10: 0,
            ..surface
        }));

        let magma = SlimeFamilyState::new(SlimeFamilyKind::MagmaCube, 4);
        assert_eq!(
            magma.attributes(),
            SlimeFamilyAttributes {
                max_health: 16.0,
                movement_speed: 0.6,
                attack_damage: 6.0,
                armor: 12.0,
                xp_reward: 4,
            }
        );
        assert_eq!(MAGMA_CUBE_CREATE_ATTRIBUTES_MOVEMENT_SPEED, 0.2);
        assert!(SlimeFamilyState::new(SlimeFamilyKind::MagmaCube, 1).deals_damage(true));
        assert_eq!(magma.jump_delay(0, false), 40);
        assert_eq!(magma.ground_jump_y_velocity(0.42), 0.82);
        assert_eq!(magma.lava_jump_y_velocity(), Some(0.42000002));
        assert_eq!(magma_cube_spawn_allowed(false), true);
        assert_eq!(magma_cube_spawn_allowed(true), false);
        assert!(!magma_cube_is_on_fire());
    }

    #[test]
    fn phantom_size_anchor_swoop_and_cat_gates_match_java_rules() {
        let mut phantom = PhantomState::new();
        phantom.set_size(200);
        assert_eq!(phantom.size, 64);
        phantom.set_size(-5);
        assert_eq!(phantom.size, 0);

        let finalized = PhantomState::finalize_spawn(PhantomBlockPos { x: 3, y: 70, z: -2 });
        assert_eq!(finalized.size, 0);
        assert_eq!(
            finalized.anchor_point,
            Some(PhantomBlockPos { x: 3, y: 75, z: -2 })
        );
        assert_eq!(
            PhantomState::read_save_data(Some(4), Some(PhantomBlockPos { x: 1, y: 2, z: 3 }))
                .attributes(),
            PhantomAttributes {
                attack_damage: 10.0,
                xp_reward: 5,
                dimensions_scale: 1.6,
            }
        );

        assert_eq!(PHANTOM_FLAP_DEGREES_PER_TICK, 7.448451);
        assert_eq!(PHANTOM_TICKS_PER_FLAP, 25);
        assert_eq!(phantom_unique_flap_tick_offset(7), 21);
        assert!(finalized.is_flapping(7, 4));
        assert_eq!(phantom_target_scan_tick(2), Some(1));
        assert_eq!(phantom_target_scan_tick(0), None);
        assert_eq!(phantom_target_scan_reset_ticks(), 60);
        assert_eq!(PHANTOM_TARGET_RANGE, 64.0);
        assert_eq!(PHANTOM_TARGET_BOX_INFLATE_XZ, 16.0);
        assert_eq!(PHANTOM_TARGET_BOX_INFLATE_Y, 64.0);

        let target = PhantomBlockPos {
            x: 10,
            y: 50,
            z: -10,
        };
        assert_eq!(
            phantom_anchor_above_target(target, 19, 80),
            PhantomBlockPos {
                x: 10,
                y: 89,
                z: -10
            }
        );
        assert_eq!(
            phantom_stop_anchor_after_heightmap(4, 63, 5, 19),
            PhantomBlockPos { x: 4, y: 92, z: 5 }
        );

        let started = phantom_attack_strategy_start(target, 0, 63);
        assert_eq!(started.attack_phase, PhantomAttackPhase::Circle);
        assert_eq!(started.next_sweep_tick, 10);
        assert_eq!(
            started.anchor_point,
            Some(PhantomBlockPos {
                x: 10,
                y: 70,
                z: -10
            })
        );

        let waiting = phantom_attack_strategy_tick(PhantomAttackPhase::Circle, 2, target, 0, 3, 63);
        assert_eq!(waiting.next_sweep_tick, 1);
        assert_eq!(waiting.attack_phase, PhantomAttackPhase::Circle);
        assert!(!waiting.played_swoop_sound);

        let swoop = phantom_attack_strategy_tick(PhantomAttackPhase::Circle, 1, target, 3, 3, 63);
        assert_eq!(swoop.attack_phase, PhantomAttackPhase::Swoop);
        assert_eq!(swoop.next_sweep_tick, 220);
        assert!(swoop.played_swoop_sound);

        assert_eq!(
            phantom_can_continue_swoop(true, true, false, false, PhantomAttackPhase::Swoop, true),
            PhantomSwoopContinuation::StopScaredOfCat
        );
        assert_eq!(
            phantom_can_continue_swoop(true, true, true, false, PhantomAttackPhase::Swoop, false),
            PhantomSwoopContinuation::StopCreativeOrSpectatorPlayer
        );
        assert_eq!(
            phantom_can_continue_swoop(true, true, false, false, PhantomAttackPhase::Swoop, false),
            PhantomSwoopContinuation::Continue
        );
        assert_eq!(
            phantom_swoop_tick(true, false, false, false),
            PhantomSwoopTick::HitTarget {
                level_event: Some(1039)
            }
        );
        assert_eq!(
            phantom_swoop_tick(false, true, false, false),
            PhantomSwoopTick::CancelledToCircle
        );
        assert_eq!(PHANTOM_SWEEP_CAT_SEARCH_TICK_DELAY, 20);
        assert_eq!(PHANTOM_CAT_AVOID_INFLATE, 16.0);
        assert_eq!(PHANTOM_LOOT_ITEM, "minecraft:phantom_membrane");
        assert!(!phantom_burns_in_daylight());
        assert!(!phantom_uses_nearest_players_memory());
        assert_eq!(phantom_membrane_loot_roll(false, 1, 3), 0);
        assert_eq!(phantom_membrane_loot_roll(true, 1, 2), 3);
    }

    #[test]
    fn vex_lifetime_charge_and_evoker_summon_gates_match_java_rules() {
        assert_eq!(
            vex_attributes(),
            VexAttributes {
                max_health: 14.0,
                attack_damage: 4.0,
                xp_reward: 3,
            }
        );
        assert_eq!(VEX_FLAP_DEGREES_PER_TICK, 45.836624);
        assert_eq!(VEX_TICKS_PER_FLAP, 4);
        assert!(vex_is_flapping(8));
        assert!(!vex_is_flapping(9));
        assert_eq!(VEX_DEFAULT_MAINHAND_ITEM, "minecraft:iron_sword");
        assert_eq!(VEX_MAINHAND_DROP_CHANCE, 0.0);
        assert_eq!(VEX_LIGHT_LEVEL_MAGIC_VALUE, 1.0);

        let flags = vex_set_charging(0, true);
        assert!(vex_is_charging(flags));
        assert!(!vex_is_charging(vex_set_charging(flags, false)));
        assert_eq!(
            vex_tick(true, 2),
            VexTickOutcome {
                no_physics_during_tick: true,
                no_gravity_after_tick: true,
                limited_life_ticks: 1,
                starve_damage: false,
            }
        );
        assert_eq!(
            vex_tick(true, 1),
            VexTickOutcome {
                no_physics_during_tick: true,
                no_gravity_after_tick: true,
                limited_life_ticks: 20,
                starve_damage: true,
            }
        );
        assert!(!vex_tick(false, 0).starve_damage);

        assert!(vex_charge_attack_can_use(true, true, false, 0, 4.1));
        assert!(!vex_charge_attack_can_use(true, true, false, 1, 4.1));
        assert!(!vex_charge_attack_can_use(true, true, true, 0, 4.1));
        assert!(!vex_charge_attack_can_use(true, true, false, 0, 4.0));
        assert!(vex_charge_attack_can_continue(true, true, true, true));
        assert!(!vex_charge_attack_can_continue(true, false, true, true));
        assert_eq!(vex_charge_attack_tick(true, 16.0), (true, false));
        assert_eq!(vex_charge_attack_tick(false, 8.9), (false, true));
        assert_eq!(vex_charge_attack_tick(false, 9.0), (false, false));

        assert!(vex_copy_owner_target_can_use(true, true, true));
        assert!(!vex_copy_owner_target_can_use(true, false, true));
        assert!(vex_random_move_can_use(false, 0));
        assert!(!vex_random_move_can_use(false, 1));
        assert_eq!(VEX_RANDOM_MOVE_ATTEMPTS, 3);
        assert_eq!(VEX_RANDOM_MOVE_XZ_RANDOM_BOUND, 15);
        assert_eq!(VEX_RANDOM_MOVE_Y_RANDOM_BOUND, 11);
        assert_eq!(VEX_RANDOM_MOVE_XZ_OFFSET, 7);
        assert_eq!(VEX_RANDOM_MOVE_Y_OFFSET, 5);
        assert_eq!(VEX_RANDOM_MOVE_SPEED, 0.25);
        assert_eq!(VEX_MOVE_ACCELERATION, 0.05);
        assert_eq!(VEX_MOVE_CLOSE_DAMPING, 0.5);
        assert_eq!(VEX_OWNER_TARGET_RANGE, 16.0);

        assert!(evoker_vex_summon_can_use(true, 2, 3));
        assert!(!evoker_vex_summon_can_use(true, 3, 3));
        assert!(!evoker_vex_summon_can_use(false, 0, 8));
        assert_eq!(evoker_vex_limited_life_ticks(0), 600);
        assert_eq!(evoker_vex_limited_life_ticks(89), 2380);
        assert_eq!(
            evoker_vex_summon_plan(true, 2, 3, 10, true),
            VexSummonPlan {
                can_summon: true,
                count: 3,
                limited_life_ticks: 800,
                y_offset: 1,
                horizontal_random_bound: 5,
                copy_evoker_team: true,
                game_event: "minecraft:entity_place",
            }
        );
        assert_eq!(EVOKER_VEX_SUMMON_CASTING_TIME, 100);
        assert_eq!(EVOKER_VEX_SUMMON_INTERVAL, 340);
    }

    #[test]
    fn silverfish_infested_merge_and_wake_rules_match_java_rules() {
        assert_eq!(
            silverfish_attributes(),
            SilverfishAttributes {
                max_health: 8.0,
                movement_speed: 0.25,
                attack_damage: 1.0,
            }
        );
        assert!(silverfish_spawn_allowed(true, true, true));
        assert!(silverfish_spawn_allowed(true, false, false));
        assert!(!silverfish_spawn_allowed(true, false, true));
        assert!(!silverfish_spawn_allowed(false, true, false));
        assert_eq!(SILVERFISH_NEAR_PLAYER_SPAWN_BLOCK_RANGE, 5.0);
        assert_eq!(SILVERFISH_STEP_SOUND_VOLUME, 0.15);
        assert_eq!(SILVERFISH_STEP_SOUND_PITCH, 1.0);

        assert_eq!(
            silverfish_infested_block_for_host("minecraft:stone"),
            Some("minecraft:infested_stone")
        );
        assert_eq!(
            silverfish_infested_block_for_host("minecraft:cobblestone"),
            Some("minecraft:infested_cobblestone")
        );
        assert_eq!(
            silverfish_infested_block_for_host("minecraft:stone_bricks"),
            Some("minecraft:infested_stone_bricks")
        );
        assert_eq!(
            silverfish_infested_block_for_host("minecraft:mossy_stone_bricks"),
            Some("minecraft:infested_mossy_stone_bricks")
        );
        assert_eq!(
            silverfish_infested_block_for_host("minecraft:cracked_stone_bricks"),
            Some("minecraft:infested_cracked_stone_bricks")
        );
        assert_eq!(
            silverfish_infested_block_for_host("minecraft:chiseled_stone_bricks"),
            Some("minecraft:infested_chiseled_stone_bricks")
        );
        assert_eq!(
            silverfish_infested_block_for_host("minecraft:deepslate"),
            Some("minecraft:infested_deepslate")
        );
        assert_eq!(silverfish_infested_block_for_host("minecraft:dirt"), None);
        assert_eq!(
            silverfish_host_block_for_infested("minecraft:infested_deepslate"),
            Some("minecraft:deepslate")
        );

        assert_eq!(
            silverfish_walk_target_value("minecraft:stone", 0.25),
            SILVERFISH_WALK_TARGET_HOST_VALUE
        );
        assert_eq!(silverfish_walk_target_value("minecraft:dirt", 0.25), 0.25);
        assert!(silverfish_merge_can_use(
            false,
            true,
            true,
            0,
            "minecraft:stone"
        ));
        assert!(!silverfish_merge_can_use(
            true,
            true,
            true,
            0,
            "minecraft:stone"
        ));
        assert!(!silverfish_merge_can_use(
            false,
            false,
            true,
            0,
            "minecraft:stone"
        ));
        assert!(!silverfish_merge_can_use(
            false,
            true,
            false,
            0,
            "minecraft:stone"
        ));
        assert!(!silverfish_merge_can_use(
            false,
            true,
            true,
            1,
            "minecraft:stone"
        ));
        assert!(!silverfish_merge_can_use(
            false,
            true,
            true,
            0,
            "minecraft:dirt"
        ));
        assert_eq!(SILVERFISH_MERGE_SPEED, 1.0);
        assert_eq!(SILVERFISH_MERGE_INTERVAL_TICKS, 10);

        assert_eq!(
            silverfish_notify_hurt_delay(0, true, false),
            SILVERFISH_WAKE_DELAY_TICKS
        );
        assert_eq!(
            silverfish_notify_hurt_delay(0, false, true),
            SILVERFISH_WAKE_DELAY_TICKS
        );
        assert_eq!(silverfish_notify_hurt_delay(7, true, false), 7);
        assert_eq!(silverfish_notify_hurt_delay(0, false, false), 0);

        assert!(silverfish_infested_break_spawns_silverfish(true, false));
        assert!(!silverfish_infested_break_spawns_silverfish(false, false));
        assert!(!silverfish_infested_break_spawns_silverfish(true, true));

        let offsets = silverfish_wake_scan_offsets();
        assert_eq!(offsets.len(), 11 * 21 * 21);
        assert_eq!(offsets[0], (0, 0, 0));
        assert_eq!(offsets[1], (0, 0, 1));
        assert_eq!(offsets[2], (0, 0, -1));
        assert_eq!(offsets[21], (1, 0, 0));
        assert_eq!(offsets[21 * 21], (0, 1, 0));
        assert_eq!(*offsets.last().unwrap(), (-10, -5, -10));

        assert_eq!(
            silverfish_wake_step((1, 0, -1), "minecraft:infested_stone", true),
            Some(SilverfishWakeStep {
                offset: (1, 0, -1),
                action: SilverfishWakeAction::DestroyInfestedBlock,
            })
        );
        assert_eq!(
            silverfish_wake_step((1, 0, -1), "minecraft:infested_stone", false),
            Some(SilverfishWakeStep {
                offset: (1, 0, -1),
                action: SilverfishWakeAction::RestoreHostBlock,
            })
        );
        assert_eq!(
            silverfish_wake_step((0, 0, 0), "minecraft:stone", true),
            None
        );

        let mut scan = vec!["minecraft:air"; offsets.len()];
        scan[2] = "minecraft:infested_stone";
        scan[21] = "minecraft:infested_deepslate";
        assert_eq!(
            silverfish_wake_steps_until_random_stop(&scan, true, &[false, true]),
            vec![
                SilverfishWakeStep {
                    offset: (0, 0, -1),
                    action: SilverfishWakeAction::DestroyInfestedBlock,
                },
                SilverfishWakeStep {
                    offset: (1, 0, 0),
                    action: SilverfishWakeAction::DestroyInfestedBlock,
                },
            ]
        );
    }

    #[test]
    fn zoglin_attack_target_and_hoglin_conversion_rules_match_java_rules() {
        assert_eq!(
            zoglin_attributes(),
            ZoglinAttributes {
                max_health: 40.0,
                movement_speed: 0.3,
                knockback_resistance: 0.6,
                attack_knockback: 1.0,
                attack_damage: 6.0,
                xp_reward: 5,
            }
        );
        assert_eq!(zoglin_attack_damage(false), 6.0);
        assert_eq!(zoglin_attack_damage(true), 0.5);
        assert_eq!(zoglin_attack_interval_ticks(false), 40);
        assert_eq!(zoglin_attack_interval_ticks(true), 15);
        assert!(zoglin_finalize_spawn_is_baby(0.199));
        assert!(!zoglin_finalize_spawn_is_baby(0.2));
        assert_eq!(ZOGLIN_IDLE_SPEED_MULTIPLIER, 0.4);
        assert_eq!(ZOGLIN_FIGHTING_MOVEMENT_SPEED, 0.3);
        assert_eq!(ZOGLIN_LOOK_TARGET_RANGE, 8.0);
        assert_eq!(ZOGLIN_LOOK_INTERVAL_MIN_TICKS, 30);
        assert_eq!(ZOGLIN_LOOK_INTERVAL_MAX_TICKS, 60);
        assert_eq!(ZOGLIN_DO_NOTHING_MIN_TICKS, 30);
        assert_eq!(ZOGLIN_DO_NOTHING_MAX_TICKS, 60);

        assert!(zoglin_valid_attack_target("minecraft:player", true));
        assert!(!zoglin_valid_attack_target("minecraft:zoglin", true));
        assert!(!zoglin_valid_attack_target("minecraft:creeper", true));
        assert!(!zoglin_valid_attack_target("minecraft:player", false));
        assert_eq!(
            zoglin_ambient_sound(false, false),
            Some("minecraft:entity.zoglin.ambient")
        );
        assert_eq!(
            zoglin_ambient_sound(true, false),
            Some("minecraft:entity.zoglin.angry")
        );
        assert_eq!(zoglin_ambient_sound(true, true), None);
        assert!(zoglin_on_hurt_should_retarget(true, true, true, false));
        assert!(!zoglin_on_hurt_should_retarget(true, true, true, true));
        assert_eq!(
            zoglin_event_attack_animation_ticks(4),
            Some(ZOGLIN_ATTACK_ANIMATION_DURATION_TICKS)
        );
        assert_eq!(zoglin_event_attack_animation_ticks(3), None);
        assert_eq!(zoglin_next_attack_animation_ticks(10), 9);
        assert_eq!(zoglin_next_attack_animation_ticks(0), 0);
        assert!(zoglin_blocked_by_item_throws_target(false));
        assert!(!zoglin_blocked_by_item_throws_target(true));
        assert_eq!(zoglin_save_is_baby_key(), "IsBaby");
        assert!(zoglin_is_immune_to_regular_zombification());
        assert_eq!(ZOGLIN_HURT_RETARGET_DISTANCE_MARGIN, 4.0);
        assert_eq!(ZOGLIN_STEP_SOUND_VOLUME, 0.15);
        assert_eq!(ZOGLIN_STEP_SOUND_PITCH, 1.0);

        assert_eq!(
            hoglin_attributes(),
            HoglinAttributes {
                max_health: 40.0,
                movement_speed: 0.3,
                knockback_resistance: 0.6,
                attack_knockback: 1.0,
                attack_damage: 6.0,
                xp_reward: 5,
            }
        );
        assert_eq!(
            hoglin_entity_type_surface(),
            HoglinEntityTypeSurface {
                width: 1.3964844,
                height: 1.4,
                passenger_attachment_y: 1.49375,
                client_tracking_range: 8,
            }
        );
        assert_eq!(hoglin_attack_damage(false), 6.0);
        assert_eq!(hoglin_attack_damage(true), 0.5);
        assert_eq!(hoglin_xp_reward(false), 5);
        assert_eq!(hoglin_xp_reward(true), 3);
        assert_eq!(hoglin_attack_interval_ticks(false), 40);
        assert_eq!(hoglin_attack_interval_ticks(true), 15);
        assert!(hoglin_finalize_spawn_is_baby(0.199));
        assert!(!hoglin_finalize_spawn_is_baby(0.2));
        assert!(!hoglin_spawn_allowed("minecraft:nether_wart_block"));
        assert!(hoglin_spawn_allowed("minecraft:crimson_nylium"));
        assert_eq!(
            hoglin_walk_target_value(true, "minecraft:crimson_nylium"),
            -1.0
        );
        assert_eq!(
            hoglin_walk_target_value(false, "minecraft:crimson_nylium"),
            10.0
        );
        assert_eq!(hoglin_walk_target_value(false, "minecraft:netherrack"), 0.0);
        assert!(hoglin_can_be_hunted(true, false));
        assert!(!hoglin_can_be_hunted(false, false));
        assert!(!hoglin_can_be_hunted(true, true));
        assert!(hoglin_can_fall_in_love(false, true));
        assert!(!hoglin_can_fall_in_love(true, true));
        assert!(hoglin_piglins_outnumber_hoglins(false, 3, 1));
        assert!(!hoglin_piglins_outnumber_hoglins(false, 2, 1));
        assert!(!hoglin_piglins_outnumber_hoglins(true, 3, 1));
        assert_eq!(
            hoglin_on_hit_target_action(false, "minecraft:piglin", true),
            HoglinAiAction::BroadcastRetreat
        );
        assert_eq!(
            hoglin_on_hit_target_action(false, "minecraft:player", false),
            HoglinAiAction::BroadcastAttackTarget
        );
        assert_eq!(
            hoglin_on_hit_target_action(true, "minecraft:player", false),
            HoglinAiAction::None
        );
        assert_eq!(
            hoglin_was_hurt_action(true, "minecraft:player", false, false, true),
            HoglinAiAction::SetAvoidTarget
        );
        assert_eq!(
            hoglin_was_hurt_action(false, "minecraft:piglin", true, false, true),
            HoglinAiAction::None
        );
        assert_eq!(
            hoglin_was_hurt_action(false, "minecraft:player", false, false, true),
            HoglinAiAction::SetAttackTarget
        );
        assert_eq!(
            hoglin_was_hurt_action(false, "minecraft:hoglin", false, false, true),
            HoglinAiAction::None
        );
        assert!(hoglin_find_nearest_valid_attack_target(false, false, true));
        assert!(!hoglin_find_nearest_valid_attack_target(true, false, true));
        assert!(!hoglin_find_nearest_valid_attack_target(false, true, true));
        assert_eq!(
            hoglin_activity_sound("avoid", false, false, false),
            Some("minecraft:entity.hoglin.retreat")
        );
        assert_eq!(
            hoglin_activity_sound("fight", false, false, false),
            Some("minecraft:entity.hoglin.angry")
        );
        assert_eq!(
            hoglin_activity_sound("idle", false, true, false),
            Some("minecraft:entity.hoglin.retreat")
        );
        assert_eq!(
            hoglin_activity_sound("idle", false, false, false),
            Some("minecraft:entity.hoglin.ambient")
        );
        assert_eq!(hoglin_activity_sound("fight", false, false, true), None);
        assert_eq!(hoglin_event_attack_animation_ticks(4), Some(10));
        assert_eq!(hoglin_event_attack_animation_ticks(3), None);
        assert_eq!(hoglin_next_attack_animation_ticks(10), 9);
        assert_eq!(hoglin_next_attack_animation_ticks(0), 0);
        assert!(hoglin_blocked_by_item_throws_target(false));
        assert!(!hoglin_blocked_by_item_throws_target(true));
        assert_eq!(HOGLIN_REPELLENT_DETECTION_HORIZONTAL, 8);
        assert_eq!(HOGLIN_REPELLENT_DETECTION_VERTICAL, 4);
        assert_eq!(HOGLIN_REPELLENT_PACIFY_TIME, 200);
        assert_eq!(HOGLIN_RETREAT_MIN_SECONDS, 5);
        assert_eq!(HOGLIN_RETREAT_MAX_SECONDS, 20);
        assert_eq!(HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_IDLING, 8);
        assert_eq!(HOGLIN_DESIRED_DISTANCE_FROM_PIGLIN_RETREATING, 15);
        assert_eq!(HOGLIN_AVOID_REPELLENT_SPEED, 1.0);
        assert_eq!(HOGLIN_RETREAT_SPEED, 1.3);
        assert_eq!(HOGLIN_BREEDING_SPEED, 0.6);
        assert_eq!(HOGLIN_IDLE_SPEED, 0.4);
        assert_eq!(HOGLIN_BABY_FOLLOW_ADULT_SPEED, 0.6);
        assert_eq!(HOGLIN_ADULT_FOLLOW_RANGE_MIN, 5);
        assert_eq!(HOGLIN_ADULT_FOLLOW_RANGE_MAX, 16);
        assert_eq!(HOGLIN_LOOK_TARGET_RANGE, 8.0);
        assert_eq!(HOGLIN_LOOK_INTERVAL_MIN_TICKS, 30);
        assert_eq!(HOGLIN_LOOK_INTERVAL_MAX_TICKS, 60);
        assert_eq!(HOGLIN_DO_NOTHING_MIN_TICKS, 30);
        assert_eq!(HOGLIN_DO_NOTHING_MAX_TICKS, 60);
        assert_eq!(HOGLIN_STEP_SOUND_VOLUME, 0.15);
        assert_eq!(HOGLIN_STEP_SOUND_PITCH, 1.0);

        assert_eq!(
            hoglin_conversion_tick(299, false, false, true),
            HoglinConversionTick {
                time_in_overworld: 300,
                convert_to_zoglin: false,
                nausea_ticks: 0,
            }
        );
        assert_eq!(
            hoglin_conversion_tick(300, false, false, true),
            HoglinConversionTick {
                time_in_overworld: 301,
                convert_to_zoglin: true,
                nausea_ticks: 200,
            }
        );
        assert_eq!(
            hoglin_conversion_tick(42, true, false, true),
            HoglinConversionTick {
                time_in_overworld: 0,
                convert_to_zoglin: false,
                nausea_ticks: 0,
            }
        );
        assert_eq!(
            hoglin_conversion_tick(42, false, true, true).time_in_overworld,
            0
        );
        assert_eq!(
            hoglin_conversion_tick(42, false, false, false).time_in_overworld,
            0
        );

        assert_eq!(hoglin_base_attack_damage(false, 6.0, 5), 8.0);
        assert_eq!(hoglin_base_attack_damage(false, 6.0, 6), 3.0);
        assert_eq!(hoglin_base_attack_damage(true, 0.5, 0), 0.5);
        assert_eq!(hoglin_base_attack_damage(false, 0.0, 0), 0.0);

        assert_eq!(
            hoglin_base_throw_target(0.0, 0.0, 4.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0),
            None
        );
        assert_eq!(
            hoglin_base_throw_target(0.0, 0.0, 4.0, 0.0, 1.0, 0.25, 0.0, 0.0, 1.0),
            Some(HoglinBaseThrowVector {
                x: 0.15000000000000002,
                y: 0.375,
                z: 0.0,
                hurt_marked: true,
            })
        );
    }

    #[test]
    fn ghast_fireball_spawn_and_movement_gates_match_java_rules() {
        assert_eq!(
            ghast_attributes(),
            GhastAttributes {
                max_health: 10.0,
                follow_range: 100.0,
                camera_distance: 8.0,
                flying_speed: 0.06,
                xp_reward: 5,
            }
        );
        assert_eq!(
            ghast_entity_type_surface(),
            GhastEntityTypeSurface {
                width: 4.0,
                height: 4.0,
                eye_height: 2.6,
                passenger_attachment_y: 4.0625,
                riding_offset: 0.5,
                client_tracking_range: 10,
                fire_immune: true,
                not_in_peaceful: true,
            }
        );
        assert!(ghast_spawn_allowed(false, 0, true));
        assert!(!ghast_spawn_allowed(true, 0, true));
        assert!(!ghast_spawn_allowed(false, 1, true));
        assert!(!ghast_spawn_allowed(false, 0, false));
        assert!(ghast_target_predicate_matches(4.0));
        assert!(!ghast_target_predicate_matches(4.01));
        assert_eq!(GHAST_SOUND_VOLUME, 5.0);
        assert_eq!(GHAST_DEFAULT_EXPLOSION_POWER, 1);
        assert_eq!(GHAST_LEASH_ELASTIC_DISTANCE, 10.0);
        assert_eq!(GHAST_LEASH_SNAP_DISTANCE, 16.0);

        assert!(ghast_is_reflected_fireball(
            "minecraft:fireball",
            "minecraft:player"
        ));
        assert!(!ghast_is_reflected_fireball(
            "minecraft:small_fireball",
            "minecraft:player"
        ));
        assert_eq!(ghast_hurt_damage(true, true, 1.0), Some(1000.0));
        assert_eq!(ghast_hurt_damage(false, true, 6.0), None);
        assert_eq!(ghast_hurt_damage(false, false, 6.0), Some(6.0));
        assert_eq!(GHAST_FIREBALL_ENTITY_DAMAGE, 6.0);

        assert_eq!(
            ghast_shoot_fireball_tick(9, true, 4095.9, true, false, 1),
            GhastShootTick {
                charge_time: 10,
                charging: false,
                warn_level_event: Some(1015),
                shoot_level_event: None,
                fireball: None,
            }
        );
        assert_eq!(
            ghast_shoot_fireball_tick(10, true, 4095.9, true, false, 1).charging,
            true
        );
        assert_eq!(
            ghast_shoot_fireball_tick(19, true, 4095.9, true, false, 3),
            GhastShootTick {
                charge_time: -40,
                charging: false,
                warn_level_event: None,
                shoot_level_event: Some(1016),
                fireball: Some(GhastFireballPlan {
                    spawn_offset: 4.0,
                    y_offset_from_ghast_mid: 0.5,
                    explosion_power: 3,
                }),
            }
        );
        assert_eq!(
            ghast_shoot_fireball_tick(19, true, 4095.9, true, true, 1).shoot_level_event,
            None
        );
        assert_eq!(
            ghast_shoot_fireball_tick(3, true, 4096.0, true, false, 1).charge_time,
            2
        );
        assert_eq!(
            ghast_shoot_fireball_tick(0, true, 4096.0, true, false, 1).charge_time,
            0
        );
        assert_eq!(
            ghast_shoot_fireball_tick(12, false, 0.0, true, false, 1),
            GhastShootTick {
                charge_time: 12,
                charging: false,
                warn_level_event: None,
                shoot_level_event: None,
                fireball: None,
            }
        );

        assert!(ghast_random_float_can_use(false, 100.0));
        assert!(ghast_random_float_can_use(true, 0.99));
        assert!(ghast_random_float_can_use(true, 3600.01));
        assert!(!ghast_random_float_can_use(true, 1.0));
        assert!(!ghast_random_float_can_use(true, 3600.0));
        assert_eq!(GHAST_RANDOM_FLOAT_MAX_ATTEMPTS, 64);
        assert_eq!(
            ghast_random_float_target((10.0, 20.0, 30.0), 0.0, 0.5, 1.0),
            (-6.0, 20.0, 46.0)
        );
        assert_eq!(ghast_move_float_duration_tick(0, 4), 5);
        assert_eq!(GHAST_MOVE_ACCELERATION_SCALE, 5.0 / 3.0);
        assert_eq!(large_fireball_hit_outcome(true, 2), (2.0, true, true));
        assert_eq!(large_fireball_hit_outcome(false, 1), (1.0, false, true));
    }

    #[test]
    fn strider_lava_saddle_suffocation_and_jockey_rules_match_java() {
        assert_eq!(
            strider_attributes(),
            StriderAttributes {
                movement_speed: 0.175
            }
        );
        assert_eq!(
            strider_entity_type_surface(),
            StriderEntityTypeSurface {
                width: 0.9,
                height: 1.7,
                client_tracking_range: 10,
                fire_immune: true,
            }
        );
        assert_eq!(STRIDER_WATER_PATHFINDING_MALUS, -1.0);
        assert_eq!(STRIDER_LAVA_PATHFINDING_MALUS, 0.0);
        assert_eq!(STRIDER_FIRE_PATHFINDING_MALUS, 0.0);
        assert!(strider_can_stand_on_fluid("minecraft:lava"));
        assert!(!strider_can_stand_on_fluid("minecraft:water"));
        assert!(strider_spawn_allowed(true));
        assert!(!strider_spawn_allowed(false));
        assert!(strider_can_use_saddle_slot(true, false));
        assert!(!strider_can_use_saddle_slot(true, true));
        assert!(!strider_can_use_saddle_slot(false, false));

        assert!(strider_controlling_passenger(true, true, true));
        assert!(!strider_controlling_passenger(true, true, false));
        assert_eq!(strider_ridden_speed(0.175, false, 1.0), 0.09625);
        assert!((strider_ridden_speed(0.175, true, 1.0) - 0.06125).abs() < 0.000001);
        assert_eq!(STRIDER_SUFFOCATE_STEERING_MODIFIER, 0.35);
        assert_eq!(STRIDER_STEERING_MODIFIER, 0.55);
        assert_eq!(STRIDER_SUFFOCATING_MODIFIER, -0.34);

        assert_eq!(
            strider_suffocation_state(false, false, false, false, false),
            Some(StriderSuffocationState {
                suffocating: true,
                movement_speed_modifier: Some(-0.34),
            })
        );
        assert_eq!(
            strider_suffocation_state(false, false, false, true, false),
            Some(StriderSuffocationState {
                suffocating: false,
                movement_speed_modifier: None,
            })
        );
        assert_eq!(
            strider_suffocation_state(false, false, false, false, true)
                .unwrap()
                .suffocating,
            false
        );
        assert_eq!(
            strider_suffocation_state(true, false, false, false, false),
            None
        );
        assert_eq!(strider_walk_target_value(true, false), 10.0);
        assert_eq!(strider_walk_target_value(false, false), 0.0);
        assert_eq!(strider_walk_target_value(false, true), f32::NEG_INFINITY);

        assert!(strider_can_add_passenger(false, false));
        assert!(!strider_can_add_passenger(true, false));
        assert!(!strider_can_add_passenger(false, true));
        assert!(strider_interaction_starts_riding(false, true, false, false));
        assert!(!strider_interaction_starts_riding(true, true, false, false));
        assert_eq!(STRIDER_HAPPY_SOUND_RANDOM_BOUND, 140);
        assert_eq!(STRIDER_RETREAT_SOUND_RANDOM_BOUND, 60);
        assert_eq!(STRIDER_STEP_DISTANCE_INCREMENT, 0.6);
        assert_eq!(STRIDER_STEP_SOUND_VOLUME, 1.0);
        assert_eq!(STRIDER_STEP_SOUND_PITCH, 1.0);
        assert_eq!(STRIDER_LIQUID_COLLISION_HEIGHT, 8.0);
        assert_eq!(
            strider_float_in_lava(true, true, false, (0.2, -0.1, 0.4)),
            (true, (0.2, -0.1, 0.4))
        );
        assert_eq!(
            strider_float_in_lava(true, false, false, (0.2, -0.1, 0.4)),
            (false, (0.1, 0.0, 0.2))
        );
        assert_eq!(
            strider_float_in_lava(false, false, false, (0.2, -0.1, 0.4)),
            (false, (0.2, -0.1, 0.4))
        );

        assert!(strider_go_to_lava_can_use(false, true));
        assert!(!strider_go_to_lava_can_use(true, true));
        assert!(strider_go_to_lava_can_continue(false, true));
        assert!(!strider_go_to_lava_can_continue(true, true));
        assert!(strider_go_to_lava_valid_target("minecraft:lava", true));
        assert!(!strider_go_to_lava_valid_target("minecraft:lava", false));
        assert_eq!(STRIDER_GO_TO_LAVA_SEARCH_RANGE, 8);
        assert_eq!(STRIDER_GO_TO_LAVA_VERTICAL_SEARCH_RANGE, 2);
        assert_eq!(STRIDER_RANDOM_STROLL_INTERVAL, 60);
        assert!(strider_navigation_valid_path_type("lava", false));
        assert!(strider_navigation_valid_path_type("fire", false));
        assert!(strider_navigation_valid_path_type(
            "fire_in_neighbor",
            false
        ));
        assert!(!strider_navigation_valid_path_type("water", false));
        assert!(strider_navigation_valid_path_type("water", true));

        assert_eq!(
            strider_finalize_spawn(false, 0, 9),
            StriderFinalizeSpawn::ZombifiedPiglinJockey {
                jockey_holds: "minecraft:warped_fungus_on_a_stick",
                strider_saddle: "minecraft:saddle",
                guaranteed_saddle_drop: true,
            }
        );
        assert_eq!(
            strider_finalize_spawn(false, 1, 0),
            StriderFinalizeSpawn::BabyStriderJockey { baby_age: -24000 }
        );
        assert_eq!(
            strider_finalize_spawn(false, 1, 1),
            StriderFinalizeSpawn::AgeableGroup { baby_chance: 50 }
        );
        assert_eq!(
            strider_finalize_spawn(true, 0, 0),
            StriderFinalizeSpawn::None
        );
        assert!(strider_is_sensitive_to_water());
        assert!(!strider_is_on_fire());
    }

    #[test]
    fn witch_drinking_throwing_and_raid_gates_match_java_rules() {
        assert_eq!(
            witch_attributes(),
            WitchAttributes {
                max_health: 26.0,
                movement_speed: 0.25,
            }
        );
        assert_eq!(WITCH_RANGED_ATTACK_SPEED, 1.0);
        assert_eq!(WITCH_RANGED_ATTACK_INTERVAL_TICKS, 60);
        assert_eq!(WITCH_RANGED_ATTACK_RADIUS, 10.0);
        assert_eq!(WITCH_RANDOM_STROLL_SPEED, 1.0);
        assert_eq!(WITCH_LOOK_AT_PLAYER_RANGE, 8.0);
        assert!(witch_heal_raiders_goal_enabled(true, "minecraft:pillager"));
        assert!(!witch_heal_raiders_goal_enabled(true, "minecraft:witch"));
        assert!(!witch_heal_raiders_goal_enabled(
            false,
            "minecraft:pillager"
        ));
        assert!(witch_attack_players_enabled(0));
        assert!(!witch_attack_players_enabled(1));

        assert_eq!(
            witch_select_drink_potion(
                0.149, 1.0, 1.0, 1.0, true, false, false, false, 26.0, 26.0, false, false, 0.0,
            ),
            Some("minecraft:water_breathing")
        );
        assert_eq!(
            witch_select_drink_potion(
                1.0, 0.149, 1.0, 1.0, false, false, true, false, 26.0, 26.0, false, false, 0.0,
            ),
            Some("minecraft:fire_resistance")
        );
        assert_eq!(
            witch_select_drink_potion(
                1.0, 1.0, 0.049, 1.0, false, false, false, false, 25.0, 26.0, false, false, 0.0,
            ),
            Some("minecraft:healing")
        );
        assert_eq!(
            witch_select_drink_potion(
                1.0, 1.0, 1.0, 0.499, false, false, false, false, 26.0, 26.0, true, false, 121.1,
            ),
            Some("minecraft:swiftness")
        );
        assert_eq!(
            witch_select_drink_potion(
                0.15, 0.15, 0.05, 0.5, true, false, true, false, 25.0, 26.0, true, false, 122.0,
            ),
            None
        );
        assert_eq!(
            witch_start_drinking(Some("minecraft:healing"), false),
            Some(WitchDrinkStart {
                potion: "minecraft:healing",
                using_item: true,
                speed_modifier: -0.25,
                drink_sound: Some("minecraft:entity.witch.drink"),
            })
        );
        assert_eq!(witch_start_drinking(None, false), None);
        assert_eq!(
            witch_finish_drinking(true, true),
            WitchDrinkFinish {
                using_item: false,
                clear_main_hand: true,
                apply_potion_effects: true,
                game_event: "minecraft:drink",
                remove_speed_modifier: true,
            }
        );
        assert!(!witch_finish_drinking(true, false).apply_potion_effects);

        assert_eq!(
            witch_ranged_attack(true, false, 20.0, false, false, false, 10.0, 0.0, false),
            None
        );
        assert_eq!(
            witch_ranged_attack(false, true, 4.0, false, false, false, 1.0, 1.0, false),
            Some(WitchRangedAttack {
                potion: "minecraft:healing",
                clear_target: true,
                velocity: 0.45,
                inaccuracy: 8.0,
                throw_sound: Some("minecraft:entity.witch.throw"),
            })
        );
        assert_eq!(
            witch_ranged_attack(false, true, 4.1, false, false, false, 4.0, 1.0, true)
                .unwrap()
                .potion,
            "minecraft:regeneration"
        );
        assert_eq!(
            witch_ranged_attack(false, false, 20.0, false, false, false, 8.0, 1.0, false)
                .unwrap()
                .potion,
            "minecraft:slowness"
        );
        assert_eq!(
            witch_ranged_attack(false, false, 8.0, true, false, false, 4.0, 1.0, false)
                .unwrap()
                .potion,
            "minecraft:poison"
        );
        assert_eq!(
            witch_ranged_attack(false, false, 7.9, true, true, false, 3.0, 0.249, false)
                .unwrap()
                .potion,
            "minecraft:weakness"
        );
        assert_eq!(
            witch_ranged_attack(false, false, 7.9, true, true, false, 3.0, 0.25, false)
                .unwrap()
                .potion,
            "minecraft:harming"
        );
        assert_eq!(
            witch_ranged_attack(false, false, 20.0, true, false, false, 2.0, 1.0, false)
                .unwrap()
                .velocity,
            0.45
        );
        assert_eq!(witch_projectile_y_adjustment(8.0), 1.6);

        assert_eq!(witch_damage_after_magic_absorb(true, true, 10.0), 0.0);
        assert_eq!(witch_damage_after_magic_absorb(false, true, 10.0), 1.5);
        assert_eq!(witch_damage_after_magic_absorb(false, false, 10.0), 10.0);
        assert_eq!(WITCH_PARTICLE_EVENT_ID, 15);
        assert_eq!(WITCH_PARTICLE_CHANCE, 7.5E-4);
        assert_eq!(witch_particle_count(0), 10);
        assert_eq!(witch_particle_count(34), 44);
        assert!(!WITCH_CAN_BE_RAID_LEADER);
        assert!(!WITCH_RAID_BUFFS_APPLIED);
        assert!(!witch_finalize_can_join_raid(true));
        assert!(witch_finalize_can_join_raid(false));
        assert!(!witch_ravager_rider_in_java_26_1_2());
    }

    #[test]
    fn guardian_beam_thorns_and_elder_curse_rules_match_java() {
        assert_eq!(
            guardian_attributes(),
            GuardianAttributes {
                max_health: 30.0,
                movement_speed: 0.5,
                attack_damage: 6.0,
                xp_reward: 10,
            }
        );
        assert_eq!(
            elder_guardian_attributes(),
            GuardianAttributes {
                max_health: 80.0,
                movement_speed: 0.3,
                attack_damage: 8.0,
                xp_reward: 10,
            }
        );
        assert_eq!(
            guardian_entity_type_surface(),
            GuardianEntityTypeSurface {
                width: 0.85,
                height: 0.85,
                eye_height: 0.425,
                passenger_attachment_y: 0.975,
                client_tracking_range: 8,
                not_in_peaceful: true,
            }
        );
        assert_eq!(
            elder_guardian_entity_type_surface(),
            GuardianEntityTypeSurface {
                width: 1.9975,
                height: 1.9975,
                eye_height: 0.99875,
                passenger_attachment_y: 2.350625,
                client_tracking_range: 10,
                not_in_peaceful: true,
            }
        );
        assert_eq!(GUARDIAN_WATER_PATHFINDING_MALUS, 0.0);
        assert_eq!(GUARDIAN_AMBIENT_SOUND_INTERVAL, 160);
        assert_eq!(GUARDIAN_MAX_HEAD_X_ROT, 180);
        assert_eq!(GUARDIAN_ATTACK_START_TICKS, -10);
        assert_eq!(GUARDIAN_ATTACK_DURATION_TICKS, 80);
        assert_eq!(ELDER_GUARDIAN_ATTACK_DURATION_TICKS, 60);
        assert!(guardian_spawn_allowed(0, true, false, false, true, true));
        assert!(guardian_spawn_allowed(1, false, false, false, true, true));
        assert!(guardian_spawn_allowed(0, true, false, true, false, true));
        assert!(!guardian_spawn_allowed(1, true, false, true, false, true));
        assert!(!guardian_spawn_allowed(1, true, false, false, true, true));
        assert!(!guardian_spawn_allowed(0, true, true, false, true, true));
        assert!(!guardian_spawn_allowed(0, true, false, false, false, true));
        assert!(!guardian_spawn_allowed(0, true, false, false, true, false));

        assert!(guardian_attack_selector_matches("minecraft:player", 9.1));
        assert!(guardian_attack_selector_matches("minecraft:squid", 10.0));
        assert!(guardian_attack_selector_matches("minecraft:axolotl", 10.0));
        assert!(!guardian_attack_selector_matches("minecraft:zombie", 10.0));
        assert!(!guardian_attack_selector_matches("minecraft:player", 9.0));
        assert!(guardian_attack_can_continue(false, true, 9.1, true));
        assert!(!guardian_attack_can_continue(false, true, 9.0, true));
        assert!(guardian_attack_can_continue(true, false, 0.0, true));

        assert_eq!(
            guardian_attack_tick(-1, 42, true, false, false, false),
            GuardianAttackTick {
                attack_time: 0,
                active_attack_target: Some(42),
                broadcast_event: Some(21),
                magic_damage: None,
                melee_hit: false,
                clear_target: false,
            }
        );
        assert_eq!(
            guardian_attack_tick(79, 42, true, false, true, false),
            GuardianAttackTick {
                attack_time: 80,
                active_attack_target: None,
                broadcast_event: None,
                magic_damage: Some(3.0),
                melee_hit: true,
                clear_target: true,
            }
        );
        assert_eq!(
            guardian_attack_tick(59, 42, true, false, true, true).magic_damage,
            Some(5.0)
        );
        assert!(guardian_attack_tick(5, 42, false, false, false, false).clear_target);

        assert_eq!(guardian_thorns_damage(false, false, false, true), Some(2.0));
        assert_eq!(guardian_thorns_damage(true, false, false, true), None);
        assert_eq!(guardian_thorns_damage(false, true, false, true), None);
        assert_eq!(guardian_thorns_damage(false, false, true, true), None);
        assert_eq!(guardian_thorns_damage(false, false, false, false), None);
        assert_eq!(guardian_walk_target_value(true, 0.25, 0.0), 10.25);
        assert_eq!(guardian_walk_target_value(false, 0.25, 0.5), 0.5);
        assert_eq!(GUARDIAN_AIR_SUPPLY_IN_WATER, 300);
        assert_eq!(GUARDIAN_LAND_FLOP_Y_PUSH, 0.5);
        assert_eq!(GUARDIAN_LAND_FLOP_XZ_SCALE, 0.4);
        assert_eq!(GUARDIAN_TRAVEL_WATER_RELATIVE, 0.1);
        assert_eq!(GUARDIAN_TRAVEL_WATER_DAMPING, 0.9);
        assert_eq!(GUARDIAN_IDLE_SINKING_Y, -0.005);

        assert_eq!(
            elder_guardian_effect_pulse(1199, 1, false),
            Some(ElderGuardianEffectPulse {
                mining_fatigue_ticks: 6000,
                amplifier: 2,
                radius: 50.0,
                display_limit_ticks: 1200,
                game_event_strength: 1.0,
            })
        );
        assert_eq!(
            elder_guardian_effect_pulse(1199, 1, true)
                .unwrap()
                .game_event_strength,
            0.0
        );
        assert_eq!(elder_guardian_effect_pulse(1198, 1, false), None);
        assert_eq!(elder_guardian_sets_home_when_missing(false), Some(16));
        assert_eq!(elder_guardian_sets_home_when_missing(true), None);
        assert_eq!(ELDER_GUARDIAN_RANDOM_STROLL_INTERVAL, 400);
    }

    #[test]
    fn ravager_attack_stun_roar_and_leaf_griefing_match_java() {
        assert_eq!(
            ravager_attributes(),
            RavagerAttributes {
                max_health: 100.0,
                movement_speed: 0.3,
                knockback_resistance: 0.75,
                attack_damage: 12.0,
                attack_knockback: 1.5,
                follow_range: 32.0,
                step_height: 1.0,
                xp_reward: 20,
            }
        );
        assert_eq!(
            ravager_entity_type_surface(),
            RavagerEntityTypeSurface {
                width: 1.95,
                height: 2.2,
                passenger_attachment_y: 2.2625,
                passenger_attachment_z: -0.0625,
                client_tracking_range: 10,
                not_in_peaceful: true,
            }
        );
        assert_eq!(RAVAGER_LEAVES_PATHFINDING_MALUS, 0.0);
        assert_eq!(RAVAGER_MAX_HEAD_Y_ROT, 45);
        assert_eq!(RAVAGER_ATTACK_BB_DEFLATE_XZ, 0.05);
        assert_eq!(RAVAGER_RANDOM_STROLL_SPEED, 0.4);
        assert_eq!(RAVAGER_LOOK_AT_PLAYER_RANGE, 6.0);
        assert_eq!(RAVAGER_LOOK_AT_MOB_RANGE, 8.0);

        assert!(ravager_target_selector_matches("minecraft:player", false));
        assert!(ravager_target_selector_matches("minecraft:villager", false));
        assert!(!ravager_target_selector_matches("minecraft:villager", true));
        assert!(ravager_target_selector_matches(
            "minecraft:iron_golem",
            false
        ));
        assert!(!ravager_target_selector_matches("minecraft:zombie", false));

        assert_eq!(
            ravager_control_flags_enabled(false, false, false),
            (true, true, true, true)
        );
        assert_eq!(
            ravager_control_flags_enabled(true, true, false),
            (true, true, true, true)
        );
        assert_eq!(
            ravager_control_flags_enabled(true, false, false),
            (false, false, false, false)
        );
        assert_eq!(
            ravager_control_flags_enabled(true, false, true),
            (false, false, false, false)
        );

        assert_eq!(
            ravager_ai_step(0.3, true, false, 10, 0, 0, false, false, false, false).movement_speed,
            0.305
        );
        assert_eq!(
            ravager_ai_step(0.35, false, false, 0, 0, 0, false, false, false, false).movement_speed,
            0.345
        );
        assert_eq!(
            ravager_ai_step(0.3, true, true, 0, 0, 0, false, false, false, false).movement_speed,
            0.0
        );
        assert!(ravager_ai_step(0.3, false, false, 0, 0, 11, false, false, false, false).roar_now);
        assert_eq!(
            ravager_ai_step(0.3, false, false, 3, 1, 0, false, false, false, false),
            RavagerAiStep {
                movement_speed: 0.3,
                attack_tick: 2,
                stunned_tick: 0,
                roar_tick: 20,
                roar_now: false,
                start_roar_sound: true,
                should_jump_after_leaf_collision: false,
            }
        );
        assert!(
            ravager_ai_step(0.3, false, false, 0, 0, 0, true, true, false, true)
                .should_jump_after_leaf_collision
        );
        assert!(
            !ravager_ai_step(0.3, false, false, 0, 0, 0, true, true, true, true)
                .should_jump_after_leaf_collision
        );
        assert!(ravager_is_immobile(false, 1, 0, 0));
        assert!(ravager_is_immobile(false, 0, 1, 0));
        assert!(ravager_is_immobile(false, 0, 0, 1));
        assert!(!ravager_is_immobile(false, 0, 0, 0));
        assert!(!ravager_has_line_of_sight_allowed(1, 0, true));
        assert!(!ravager_has_line_of_sight_allowed(0, 1, true));
        assert!(ravager_has_line_of_sight_allowed(0, 0, true));

        assert_eq!(
            ravager_blocked_by_item(0, true),
            RavagerBlockedByItem {
                stunned_tick: 40,
                roar_tick: 0,
                stun_event: Some(39),
                strong_knockback: false,
                defender_hurt_marked: true,
            }
        );
        assert_eq!(
            ravager_blocked_by_item(0, false),
            RavagerBlockedByItem {
                stunned_tick: 0,
                roar_tick: 0,
                stun_event: None,
                strong_knockback: true,
                defender_hurt_marked: true,
            }
        );
        assert_eq!(ravager_blocked_by_item(5, true).defender_hurt_marked, false);

        assert_eq!(
            ravager_roar_effect("minecraft:player", true, true),
            Some(RavagerRoarEffect {
                damage: Some(6.0),
                strong_knockback: false,
                include_armor_stand: false,
                event: Some(69),
            })
        );
        assert_eq!(
            ravager_roar_effect("minecraft:vindicator", true, true)
                .unwrap()
                .damage,
            None
        );
        assert_eq!(ravager_roar_effect("minecraft:ravager", true, true), None);
        assert_eq!(
            ravager_roar_effect("minecraft:armor_stand", true, false),
            None
        );
        assert!(
            ravager_roar_effect("minecraft:armor_stand", true, true)
                .unwrap()
                .include_armor_stand
        );
        assert_eq!(
            ravager_do_hurt_target_event(),
            (RAVAGER_ATTACK_DURATION, RAVAGER_ATTACK_EVENT_ID)
        );
        assert!(ravager_can_spawn_without_obstruction(false));
        assert!(!ravager_can_spawn_without_obstruction(true));
        assert!(!ravager_can_be_raid_leader());
    }

    #[test]
    fn shulker_attach_peek_teleport_and_bullet_rules_match_java() {
        assert_eq!(
            shulker_attributes(),
            ShulkerAttributes {
                max_health: 30.0,
                covered_armor_bonus: 20.0,
                xp_reward: 5,
            }
        );
        assert_eq!(
            shulker_entity_type_surface(),
            ShulkerEntityTypeSurface {
                width: 1.0,
                height: 1.0,
                eye_height: 0.5,
                client_tracking_range: 10,
                fire_immune: true,
                can_spawn_far_from_player: true,
            }
        );
        assert_eq!(
            shulker_bullet_surface(),
            ShulkerBulletSurface {
                width: 0.3125,
                height: 0.3125,
                client_tracking_range: 8,
                no_loot_table: true,
                no_physics: true,
            }
        );
        assert_eq!(SHULKER_DEFAULT_ATTACH_FACE, ShulkerDirection::Down);
        assert_eq!(SHULKER_DEFAULT_PEEK, 0);
        assert_eq!(SHULKER_DEFAULT_COLOR, 16);
        assert_eq!(SHULKER_NO_COLOR, 16);
        assert_eq!(SHULKER_TELEPORT_STEPS, 6);
        assert_eq!(SHULKER_MAX_TELEPORT_DISTANCE, 8);
        assert_eq!(SHULKER_TELEPORT_ATTEMPTS, 5);
        assert_eq!(SHULKER_OTHER_SCAN_RADIUS, 8.0);
        assert_eq!(SHULKER_LOOK_AT_PLAYER_RANGE, 8.0);
        assert_eq!(SHULKER_LOOK_AT_PLAYER_PROBABILITY, 0.02);
        assert_eq!(SHULKER_MAX_HEAD_X_ROT, 180);
        assert_eq!(SHULKER_MAX_HEAD_Y_ROT, 180);
        assert_eq!(SHULKER_RENDER_DISTANCE_SQR, 16384.0);

        assert_eq!(shulker_update_peek_amount(0.0, 30), 0.05);
        assert!((shulker_update_peek_amount(0.35, 30) - 0.3).abs() < f32::EPSILON);
        assert_eq!(shulker_update_peek_amount(1.0, 100), 1.0);
        assert_eq!(shulker_raw_peek_armor_bonus(0), Some(20.0));
        assert_eq!(shulker_raw_peek_armor_bonus(1), None);
        assert_eq!(shulker_color_from_data(0), Some(0));
        assert_eq!(shulker_color_from_data(15), Some(15));
        assert_eq!(shulker_color_from_data(16), None);
        assert_eq!(shulker_color_from_data(99), None);
        assert_eq!(shulker_sanitized_scale(2.5), 2.5);
        assert_eq!(shulker_sanitized_scale(4.0), 3.0);

        assert!(!shulker_hurt_allowed(0, "minecraft:arrow"));
        assert!(shulker_hurt_allowed(1, "minecraft:arrow"));
        assert!(shulker_hurt_allowed(0, "minecraft:trident"));
        assert!(shulker_should_teleport_after_hurt(14.9, 30.0, 0));
        assert!(!shulker_should_teleport_after_hurt(15.0, 30.0, 0));
        assert!(!shulker_should_teleport_after_hurt(14.9, 30.0, 1));

        assert_eq!(
            shulker_hit_by_bullet(1, true, 1, 0.0),
            ShulkerHitByBullet {
                should_spawn_baby: true,
                failure_chance: 0.0,
            }
        );
        assert_eq!(
            shulker_hit_by_bullet(1, true, 6, 0.99),
            ShulkerHitByBullet {
                should_spawn_baby: false,
                failure_chance: 1.0,
            }
        );
        assert!(!shulker_hit_by_bullet(0, true, 1, 0.0).should_spawn_baby);
        assert!(!shulker_hit_by_bullet(1, false, 1, 0.0).should_spawn_baby);

        assert!(shulker_attack_can_use(true, false));
        assert!(!shulker_attack_can_use(true, true));
        assert!(!shulker_attack_can_use(false, false));
        assert_eq!(
            shulker_attack_tick(1, false, true, 399.9, 3),
            ShulkerAttackTick {
                attack_time: 35,
                raw_peek: 100,
                shoot_bullet: true,
                clear_target: false,
            }
        );
        assert_eq!(
            shulker_attack_tick(20, false, true, 400.0, 0),
            ShulkerAttackTick {
                attack_time: 19,
                raw_peek: 100,
                shoot_bullet: false,
                clear_target: true,
            }
        );
        assert_eq!(
            shulker_defense_search_inflate(ShulkerDirection::East, 16.0),
            (4.0, 16.0, 16.0)
        );
        assert_eq!(
            shulker_defense_search_inflate(ShulkerDirection::North, 16.0),
            (16.0, 16.0, 4.0)
        );
        assert_eq!(
            shulker_defense_search_inflate(ShulkerDirection::Up, 16.0),
            (16.0, 4.0, 16.0)
        );

        assert_eq!(shulker_bullet_on_hit_entity(true, true), Some((4.0, 200)));
        assert_eq!(shulker_bullet_on_hit_entity(false, true), None);
        assert_eq!(SHULKER_BULLET_SPEED, 0.15);
        assert_eq!(SHULKER_BULLET_GRAVITY, 0.04);
        assert!(shulker_bullet_discards_in_peaceful(true));
        assert!(!shulker_bullet_discards_in_peaceful(false));
    }

    #[test]
    fn giant_attributes_dimensions_and_spawn_surface_match_java_rules() {
        assert_eq!(
            giant_attributes(),
            GiantAttributes {
                max_health: 100.0,
                movement_speed: 0.5,
                attack_damage: 50.0,
                camera_distance: 16.0,
            }
        );
        assert_eq!(
            giant_entity_type_surface(),
            GiantEntityTypeSurface {
                width: 3.6,
                height: 12.0,
                eye_height: 10.44,
                riding_offset: -3.75,
                client_tracking_range: 10,
                not_in_peaceful: true,
                natural_spawn: false,
                custom_ai_goals: 0,
            }
        );
        assert_eq!(giant_walk_target_value(0.42), 0.42);
    }

    #[test]
    fn zombie_conversion_baby_reinforcement_and_spawn_gates_match_java_rules() {
        assert_eq!(
            zombie_attributes(),
            ZombieAttributes {
                follow_range: 35.0,
                movement_speed: 0.23,
                attack_damage: 3.0,
                armor: 2.0,
                baby_speed_modifier: 0.5,
            }
        );
        assert_eq!(
            zombie_baby_dimensions(),
            ZombieBabyDimensions {
                width: 0.49,
                height: 0.99,
                eye_height: 0.775,
                vehicle_attachment_y: 0.1875,
            }
        );
        assert!(zombie_is_sun_sensitive());
        assert_eq!(zombie_baby_xp_reward(5, true), 12);
        assert_eq!(zombie_baby_xp_reward(5, false), 5);
        assert!(zombie_spawn_as_baby(0.049));
        assert!(!zombie_spawn_as_baby(0.05));

        assert_eq!(
            zombie_water_conversion_tick(false, true, false, -1, 599, true),
            (600, 300, true, None)
        );
        assert_eq!(
            zombie_water_conversion_tick(false, true, true, 0, 600, true),
            (600, -1, true, Some("minecraft:drowned"))
        );
        assert_eq!(
            zombie_water_conversion_tick(false, true, false, -1, 4, false),
            (-1, -1, false, None)
        );
        assert_eq!(
            zombie_water_conversion_tick(true, true, false, -1, 599, true),
            (599, -1, false, None)
        );
        assert_eq!(zombie_drowned_conversion_event(false), Some(1040));
        assert_eq!(zombie_drowned_conversion_event(true), None);

        assert_eq!(
            zombie_fire_on_hit_seconds(true, true, true, 0.59, 2.0),
            Some(4)
        );
        assert_eq!(zombie_fire_on_hit_seconds(true, true, true, 0.6, 2.0), None);
        assert_eq!(
            zombie_fire_on_hit_seconds(true, false, true, 0.0, 3.0),
            None
        );

        assert_eq!(
            zombie_reinforcement_attempt(true, true, true, 0.04, 0.05, true, true),
            ZombieReinforcementOutcome {
                attempts: 50,
                spawned: true,
                caller_reinforcement_delta: -0.05,
                callee_reinforcement_delta: -0.05,
            }
        );
        assert_eq!(
            zombie_reinforcement_attempt(true, true, false, 0.0, 1.0, true, true).attempts,
            0
        );
        assert_eq!(ZOMBIE_REINFORCEMENT_RANGE_MIN, 7);
        assert_eq!(ZOMBIE_REINFORCEMENT_RANGE_MAX, 40);

        assert_eq!(
            zombie_default_main_hand_item(false, 0.009, 0),
            Some("minecraft:iron_sword")
        );
        assert_eq!(
            zombie_default_main_hand_item(true, 0.049, 1),
            Some("minecraft:iron_spear")
        );
        assert_eq!(
            zombie_default_main_hand_item(true, 0.049, 2),
            Some("minecraft:iron_shovel")
        );
        assert_eq!(zombie_default_main_hand_item(false, 0.01, 0), None);
        assert!(!zombie_can_hold_item("minecraft:egg", true, true));
        assert!(zombie_can_hold_item("minecraft:egg", true, false));
        assert!(!zombie_wants_to_pick_up("minecraft:glow_ink_sac"));
        assert!(zombie_wants_to_pick_up("minecraft:rotten_flesh"));

        assert_eq!(
            zombie_killed_villager_outcome("easy", true, false, true, false),
            ZombieVillagerConversionOutcome::NoConversion
        );
        assert_eq!(
            zombie_killed_villager_outcome("normal", true, true, true, false),
            ZombieVillagerConversionOutcome::PerishedNormally
        );
        assert_eq!(
            zombie_killed_villager_outcome("hard", true, false, true, false),
            ZombieVillagerConversionOutcome::Converted {
                preserve_villager_data: true,
                preserve_gossips: true,
                preserve_trade_offers: true,
                preserve_xp: true,
                level_event: Some(1026),
            }
        );

        let spawned = zombie_finalize_spawn_outcome(
            false, false, false, false, true, 0.049, 0.54, 1.0, 0.06, true, 0.04, 0.09, true, true,
            0.24, 0.09, 0.5, 0.5, 1.0, 0.04, 1.0, 1.0,
        );
        assert!(spawned.can_pick_up_loot.unwrap());
        assert!(spawned.is_baby);
        assert!(!spawned.tried_existing_chicken_jockey);
        assert!(spawned.spawned_new_chicken_jockey);
        assert!(spawned.can_break_doors);
        assert_eq!(spawned.halloween_head, Some("minecraft:jack_o_lantern"));
        assert_eq!(spawned.halloween_head_drop_chance, Some(0.0));
        assert_eq!(spawned.reinforcement_base_chance, 0.05);
        assert_eq!(spawned.knockback_resistance_bonus, 0.025);
        assert_eq!(spawned.follow_range_bonus, Some(1.5));
        assert_eq!(spawned.leader_reinforcement_bonus, Some(0.75));
        assert_eq!(spawned.leader_max_health_bonus, Some(4.0));
        assert!(spawned.reset_health_to_max);

        let conversion_spawn = zombie_finalize_spawn_outcome(
            true, false, true, true, false, 0.0, 0.0, 1.0, 0.0, true, 0.0, 1.0, false, true, 0.0,
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        );
        assert_eq!(conversion_spawn.can_pick_up_loot, None);
        assert!(conversion_spawn.is_baby);
        assert!(!conversion_spawn.can_break_doors);
        assert!(!conversion_spawn.reset_health_to_max);
    }

    #[test]
    fn zombie_villager_cure_timer_preservation_and_reputation_match_java_rules() {
        assert_eq!(
            zombie_villager_baby_dimensions(),
            ZombieVillagerBabyDimensions {
                width: 0.49,
                height: 0.99,
                eye_height: 0.67,
                vehicle_attachment_y: 0.125,
            }
        );
        assert!(zombie_villager_finalize_spawn_sets_biome_type(false));
        assert!(!zombie_villager_finalize_spawn_sets_biome_type(true));
        assert_eq!(zombie_villager_conversion_time_from_roll(0), 3600);
        assert_eq!(zombie_villager_conversion_time_from_roll(2400), 6000);
        assert_eq!(ZOMBIE_VILLAGER_NOT_CONVERTING, -1);

        assert_eq!(
            zombie_villager_interact("minecraft:stick", true, true, 2, 3600),
            ZombieVillagerInteraction::PassToZombie
        );
        assert_eq!(
            zombie_villager_interact("minecraft:golden_apple", false, true, 2, 3600),
            ZombieVillagerInteraction::ConsumeGoldenApple
        );
        assert_eq!(
            zombie_villager_interact("minecraft:golden_apple", true, true, 3, 4200),
            ZombieVillagerInteraction::StartConversion {
                consumed_golden_apple: true,
                remove_weakness: true,
                strength_effect_ticks: 4200,
                strength_amplifier: 0,
                broadcast_event: 16,
            }
        );

        assert!(zombie_villager_remove_when_far_away(false, 0));
        assert!(!zombie_villager_remove_when_far_away(true, 0));
        assert!(!zombie_villager_remove_when_far_away(false, 1));
        assert_eq!(zombie_villager_conversion_progress(0.01, 14, 14), 1);
        assert_eq!(zombie_villager_conversion_progress(0.009, 20, 20), 15);
        assert_eq!(
            zombie_villager_conversion_tick(true, true, false, 3, 2),
            ZombieVillagerConversionTick {
                conversion_time: 1,
                finished: false,
            }
        );
        assert_eq!(
            zombie_villager_conversion_tick(true, true, false, 1, 2),
            ZombieVillagerConversionTick {
                conversion_time: -1,
                finished: true,
            }
        );
        assert_eq!(
            zombie_villager_conversion_tick(true, true, true, 1, 2),
            ZombieVillagerConversionTick {
                conversion_time: 1,
                finished: false,
            }
        );

        assert_eq!(
            zombie_villager_finish_conversion(true, true, false),
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
                trigger_cured_advancement: true,
                emit_reputation_event: true,
                nausea_ticks: 200,
                level_event: Some(1027),
            }
        );
        assert_eq!(
            zombie_villager_finish_conversion(true, false, true).level_event,
            None
        );
        assert!(zombie_villager_set_villager_data_clears_offers(true, true));
        assert!(!zombie_villager_set_villager_data_clears_offers(
            false, true
        ));
    }

    #[test]
    fn zombified_piglin_anger_alert_spawn_and_portal_gates_match_java_rules() {
        assert_eq!(
            zombified_piglin_attributes(),
            ZombifiedPiglinAttributes {
                follow_range: 35.0,
                movement_speed: 0.23,
                attack_damage: 5.0,
                armor: 2.0,
                spawn_reinforcements_chance: 0.0,
                attacking_speed_modifier: 0.05,
                lava_pathfinding_malus: 8.0,
            }
        );
        assert_eq!(
            zombified_piglin_baby_dimensions(),
            ZombifiedPiglinBabyDimensions {
                width: 0.49,
                height: 0.99,
                eye_height: 0.78,
                vehicle_attachment_y: 0.1875,
            }
        );
        assert_eq!(zombified_piglin_start_persistent_anger_time(399), 400);
        assert_eq!(zombified_piglin_start_persistent_anger_time(781), 780);
        assert_eq!(
            zombified_piglin_set_target_delays(false, true, 7, 99),
            Some((7, 99))
        );
        assert_eq!(zombified_piglin_set_target_delays(true, true, 7, 99), None);

        assert_eq!(
            zombified_piglin_ai_step(true, false, false, 1, true, 0, true, 80),
            ZombifiedPiglinAiStep {
                has_attacking_speed_modifier: true,
                play_first_anger_sound_in: 0,
                played_first_anger_sound: true,
                ticks_until_next_alert: 80,
                alert_others: true,
            }
        );
        assert_eq!(
            zombified_piglin_ai_step(false, false, true, 0, false, 0, false, 80)
                .has_attacking_speed_modifier,
            false
        );
        assert_eq!(
            zombified_piglin_ai_step(true, true, false, 0, false, 0, false, 80)
                .has_attacking_speed_modifier,
            false
        );
        assert!(zombified_piglin_alerts_other(false, false, false, true));
        assert!(!zombified_piglin_alerts_other(true, false, false, true));
        assert!(!zombified_piglin_alerts_other(false, true, false, true));
        assert_eq!(ZOMBIFIED_PIGLIN_ALERT_RANGE_Y, 10.0);

        assert!(zombified_piglin_spawn_allowed(false, false));
        assert!(!zombified_piglin_spawn_allowed(true, false));
        assert!(!zombified_piglin_spawn_allowed(false, true));
        assert!(zombified_piglin_spawn_obstruction(true, false));
        assert!(!zombified_piglin_spawn_obstruction(true, true));
        assert_eq!(
            zombified_piglin_default_main_hand_item(0),
            "minecraft:golden_spear"
        );
        assert_eq!(
            zombified_piglin_default_main_hand_item(1),
            "minecraft:golden_sword"
        );

        assert_eq!(
            zombified_piglin_portal_spawn(true, true, 1, 2, true, true, true),
            ZombifiedPiglinPortalSpawn {
                spawn: true,
                spawn_pos_above_portal_floor: true,
                set_entity_portal_cooldown: true,
                set_vehicle_portal_cooldown: true,
            }
        );
        assert!(!zombified_piglin_portal_spawn(true, true, 2, 2, true, true, false).spawn);
        assert!(!zombified_piglin_portal_spawn(false, true, 0, 3, true, true, false).spawn);
        assert!(zombified_piglin_prevents_player_rest(true));
        assert!(!zombified_piglin_prevents_player_rest(false));
        assert!(zombified_piglin_wants_to_pick_up(true));
        assert!(!zombified_piglin_wants_to_pick_up(false));
    }

    #[test]
    fn blaze_attack_hover_fire_and_loot_gates_match_java_rules() {
        assert_eq!(
            blaze_attributes(),
            BlazeAttributes {
                attack_damage: 6.0,
                movement_speed: 0.23,
                follow_range: 48.0,
                water_pathfinding_malus: -1.0,
                lava_pathfinding_malus: 8.0,
                fire_neighbor_pathfinding_malus: 0.0,
                fire_pathfinding_malus: 0.0,
                xp_reward: 10,
            }
        );
        assert!(blaze_attack_goal_can_use(true, true, true));
        assert!(!blaze_attack_goal_can_use(true, false, true));
        assert!(blaze_is_sensitive_to_water());
        assert_eq!(blaze_light_level_dependent_magic_value(), 1.0);
        assert!(blaze_is_on_fire(true));
        assert!(!blaze_is_on_fire(false));

        assert_eq!(
            blaze_ai_step(false, -0.2, 10, 3.0, false, 0.0, false),
            BlazeAiStep {
                delta_y: -0.120000005,
                needs_sync: false,
                allowed_height_offset: 0.5,
                next_height_offset_change_tick: 9,
            }
        );
        assert_eq!(
            blaze_ai_step(true, 0.0, 1, 4.25, true, 5.0, true),
            BlazeAiStep {
                delta_y: 0.09,
                needs_sync: true,
                allowed_height_offset: 4.25,
                next_height_offset_change_tick: 100,
            }
        );

        let state = blaze_attack_goal_start();
        assert_eq!(state.attack_step, 0);
        let (state, action) =
            blaze_attack_goal_tick(state, true, true, 16.0, BLAZE_FOLLOW_RANGE, false);
        assert_eq!(state.attack_step, 1);
        assert!(state.charged);
        assert_eq!(action, BlazeAttackAction::Charge { charge_ticks: 60 });

        let mut state = BlazeAttackState {
            attack_step: 1,
            attack_time: 0,
            last_seen: 0,
            charged: true,
        };
        for expected_step in 2..=4 {
            let (next_state, action) =
                blaze_attack_goal_tick(state, true, true, 16.0, BLAZE_FOLLOW_RANGE, false);
            assert_eq!(next_state.attack_step, expected_step);
            assert!(next_state.charged);
            assert_eq!(
                action,
                BlazeAttackAction::ShootSmallFireball {
                    cooldown_ticks: 6,
                    level_event: 1018,
                }
            );
            state = BlazeAttackState {
                attack_time: 0,
                ..next_state
            };
        }

        let (state, action) =
            blaze_attack_goal_tick(state, true, true, 16.0, BLAZE_FOLLOW_RANGE, true);
        assert_eq!(state.attack_step, 0);
        assert!(!state.charged);
        assert_eq!(
            action,
            BlazeAttackAction::Cooldown {
                cooldown_ticks: 100,
            }
        );

        let close_state = BlazeAttackState {
            attack_step: 0,
            attack_time: 0,
            last_seen: 0,
            charged: false,
        };
        assert_eq!(
            blaze_attack_goal_tick(close_state, true, true, 3.0, BLAZE_FOLLOW_RANGE, false).1,
            BlazeAttackAction::Melee { cooldown_ticks: 20 }
        );
        assert_eq!(
            blaze_attack_goal_tick(close_state, true, false, 3.0, BLAZE_FOLLOW_RANGE, false).1,
            BlazeAttackAction::None
        );
        assert_eq!(
            blaze_attack_goal_tick(close_state, true, false, 400.0, BLAZE_FOLLOW_RANGE, false).1,
            BlazeAttackAction::MoveTowardTarget
        );

        let stopped = blaze_attack_goal_stop(BlazeAttackState {
            attack_step: 2,
            attack_time: 6,
            last_seen: 3,
            charged: true,
        });
        assert!(!stopped.charged);
        assert_eq!(stopped.last_seen, 0);
        assert_eq!(blaze_fireball_spread(16.0), 1.0);
        assert_eq!(BLAZE_FIREBALL_INACCURACY, 2.297);
        assert_eq!(BLAZE_LOOT_ITEM, "minecraft:blaze_rod");
        assert_eq!(blaze_loot_roll(false, 1, 3), 0);
        assert_eq!(blaze_loot_roll(true, 1, 2), 3);
    }

    #[test]
    fn drowned_spawn_equipment_swim_and_trident_gates_match_java_rules() {
        assert_eq!(
            drowned_attributes(),
            DrownedAttributes {
                follow_range: 35.0,
                movement_speed: 0.23,
                attack_damage: 3.0,
                armor: 2.0,
                step_height: 1.0,
            }
        );
        assert_eq!(
            drowned_entity_type_surface(),
            DrownedEntityTypeSurface {
                width: 0.6,
                height: 1.95,
                eye_height: 1.74,
                passenger_attachment_y: 2.0125,
                riding_offset: -0.7,
                client_tracking_range: 8,
                not_in_peaceful: true,
                amphibious_navigation: true,
                water_pathfinding_malus: 0.0,
                can_spawn_in_liquids: true,
            }
        );
        assert_eq!(
            drowned_baby_dimensions(),
            DrownedBabyDimensions {
                width: 0.49,
                height: 0.99,
                eye_height: 0.775,
                vehicle_attachment_y: 0.1875,
            }
        );

        assert!(!drowned_spawn_allowed(
            false, true, false, false, true, false, true, false, 0, 50, 63
        ));
        assert!(drowned_spawn_allowed(
            false, true, true, false, true, false, true, false, 39, 80, 63
        ));
        assert!(drowned_spawn_allowed(
            true, true, false, true, false, false, true, false, 39, 80, 63
        ));
        assert!(drowned_spawn_allowed(
            true, true, false, false, false, false, true, true, 0, 80, 63
        ));
        assert!(!drowned_spawn_allowed(
            true, true, false, false, false, false, true, true, 1, 80, 63
        ));
        assert!(drowned_spawn_allowed(
            true, true, false, false, false, false, true, false, 0, 57, 63
        ));
        assert!(!drowned_spawn_allowed(
            true, true, false, false, false, false, true, false, 0, 58, 63
        ));
        assert!(!drowned_spawn_allowed(
            true, true, false, false, false, true, true, false, 0, 57, 63
        ));

        assert_eq!(drowned_default_main_hand_item(0.9, 0), None);
        assert_eq!(
            drowned_default_main_hand_item(0.91, 9),
            Some("minecraft:trident")
        );
        assert_eq!(
            drowned_default_main_hand_item(0.91, 10),
            Some("minecraft:fishing_rod")
        );

        let finalize =
            drowned_finalize_spawn_outcome(true, 0.029, true, true, true, 0.49, false, false);
        assert_eq!(
            finalize,
            DrownedFinalizeSpawnOutcome {
                offhand_nautilus_shell: true,
                guaranteed_offhand_drop: true,
                spawned_zombie_nautilus_jockey: true,
                zombie_nautilus_persistent: true,
            }
        );
        assert!(
            !drowned_finalize_spawn_outcome(true, 0.03, true, false, true, 0.0, false, false)
                .offhand_nautilus_shell
        );
        assert!(
            !drowned_finalize_spawn_outcome(true, 0.0, true, false, true, 0.0, true, false)
                .spawned_zombie_nautilus_jockey
        );
        assert!(
            !drowned_finalize_spawn_outcome(true, 0.0, true, false, true, 0.0, false, true)
                .spawned_zombie_nautilus_jockey
        );

        assert!(!drowned_can_replace_current_item(
            "minecraft:nautilus_shell"
        ));
        assert!(drowned_can_replace_current_item("minecraft:stick"));
        assert!(!drowned_wants_to_pick_up("minecraft:trident"));
        assert!(!drowned_wants_to_pick_up("minecraft:iron_spear"));
        assert!(drowned_wants_to_pick_up("minecraft:rotten_flesh"));

        assert!(drowned_ok_target(true, false, false));
        assert!(drowned_ok_target(true, true, true));
        assert!(!drowned_ok_target(true, true, false));
        assert!(!drowned_ok_target(false, false, true));
        assert!(drowned_wants_to_swim(true, false, false));
        assert!(drowned_wants_to_swim(false, true, true));
        assert!(!drowned_wants_to_swim(false, true, false));
        assert!(drowned_should_update_swimming(
            true, true, true, false, false
        ));
        assert!(!drowned_should_update_swimming(
            false, true, true, false, false
        ));

        assert!(drowned_trident_attack_can_use(true, true));
        assert!(!drowned_trident_attack_can_use(true, false));
        assert_eq!(
            drowned_trident_shot(2, true),
            DrownedTridentShot {
                item: "minecraft:trident",
                power: 1.6,
                inaccuracy: 6,
                y_lead_scale: 0.2,
                sound: "minecraft:entity.drowned.shoot",
            }
        );

        assert!(drowned_go_to_water_goal_can_use(true, false, true));
        assert!(!drowned_go_to_water_goal_can_use(false, false, true));
        assert!(drowned_go_to_beach_goal_can_use(true, false, true, 60, 63));
        assert!(!drowned_go_to_beach_goal_can_use(true, true, true, 60, 63));
        assert!(drowned_swim_up_goal_can_use(false, true, 60, 63));
        assert!(!drowned_swim_up_goal_can_use(true, true, 60, 63));
        assert_eq!(DROWNED_WATER_SEARCH_ATTEMPTS, 10);
        assert_eq!(DROWNED_WATER_SEARCH_XZ_RANGE, 10);
        assert_eq!(DROWNED_WATER_SEARCH_Y_UP, 2);
        assert_eq!(DROWNED_WATER_SEARCH_Y_DOWN, 5);
    }

    #[test]
    fn husk_daylight_hunger_conversion_and_camel_spawn_match_java_rules() {
        assert_eq!(
            husk_entity_type_surface(),
            HuskEntityTypeSurface {
                width: 0.6,
                height: 1.95,
                eye_height: 1.74,
                passenger_attachment_y: 2.075,
                riding_offset: -0.7,
                client_tracking_range: 8,
                not_in_peaceful: true,
            }
        );
        assert_eq!(
            husk_baby_dimensions(),
            HuskBabyDimensions {
                width: 0.49,
                height: 0.99,
                eye_height: 0.825,
                vehicle_attachment_y: 0.1875,
            }
        );
        assert!(!husk_is_sun_sensitive());
        assert_eq!(husk_hunger_duration_ticks(2.0, true, true, true), Some(280));
        assert_eq!(husk_hunger_duration_ticks(3.0, true, true, true), Some(420));
        assert_eq!(husk_hunger_duration_ticks(3.0, false, true, true), None);
        assert_eq!(husk_hunger_duration_ticks(3.0, true, false, true), None);
        assert_eq!(husk_hunger_duration_ticks(3.0, true, true, false), None);
        assert_eq!(HUSK_HUNGER_EFFECT_ID, "minecraft:hunger");
        assert_eq!(HUSK_HUNGER_AMPLIFIER, 0);

        assert!(HUSK_CONVERTS_IN_WATER);
        assert_eq!(HUSK_UNDERWATER_CONVERSION_TARGET, "minecraft:zombie");
        assert_eq!(husk_underwater_conversion_event(false), Some(1041));
        assert_eq!(husk_underwater_conversion_event(true), None);

        assert_eq!(husk_should_pick_up_loot(true, 0.0, 1.0), None);
        assert_eq!(husk_should_pick_up_loot(false, 0.54, 1.0), Some(true));
        assert_eq!(husk_should_pick_up_loot(false, 0.55, 1.0), Some(false));

        let camel_spawn = husk_finalize_spawn_outcome(true, false, 0.0, 1.0, true, 0.09);
        assert_eq!(
            camel_spawn,
            HuskFinalizeSpawnOutcome {
                can_pick_up_loot: Some(true),
                tried_to_spawn_camel_husk: true,
                spawned_camel_husk: true,
                spawned_parched_passenger: true,
                equipped_iron_spear: true,
            }
        );
        assert_eq!(HUSK_CAMEL_HUSK_RIDER_ITEM, "minecraft:iron_spear");

        let skipped_by_spawn_reason = husk_finalize_spawn_outcome(false, true, 0.0, 1.0, true, 0.0);
        assert_eq!(skipped_by_spawn_reason.can_pick_up_loot, None);

        let non_natural = husk_finalize_spawn_outcome(false, false, 0.0, 1.0, true, 0.0);
        assert!(non_natural.tried_to_spawn_camel_husk);
        assert!(!non_natural.spawned_camel_husk);

        let natural_blocked = husk_finalize_spawn_outcome(true, false, 0.0, 1.0, false, 0.0);
        assert!(!natural_blocked.tried_to_spawn_camel_husk);
        assert!(!natural_blocked.spawned_camel_husk);

        let natural_miss = husk_finalize_spawn_outcome(true, false, 0.0, 1.0, true, 0.1);
        assert!(natural_miss.tried_to_spawn_camel_husk);
        assert!(!natural_miss.spawned_camel_husk);
    }

    #[test]
    fn endermite_lifetime_spawn_and_pearl_gates_match_java_rules() {
        assert_eq!(
            endermite_attributes(),
            EndermiteAttributes {
                max_health: 8.0,
                movement_speed: 0.25,
                attack_damage: 2.0,
                xp_reward: 3,
            }
        );
        assert_eq!(ENDERMITE_MAX_LIFE_TICKS, 2400);
        assert_eq!(ENDERMITE_CLIENT_PORTAL_PARTICLES_PER_TICK, 2);
        assert_eq!(ENDERMITE_LOOK_AT_PLAYER_RANGE, 8.0);
        assert_eq!(ENDERMITE_STEP_SOUND_VOLUME, 0.15);
        assert_eq!(ENDERMITE_STEP_SOUND_PITCH, 1.0);

        let mut mite = EndermiteState::read_save_data(Some(2399), false);
        assert_eq!(
            mite.ai_step(),
            EndermiteTickOutcome {
                life: 2400,
                discard: true,
            }
        );

        let mut persistent = EndermiteState::read_save_data(Some(2399), true);
        assert_eq!(
            persistent.ai_step(),
            EndermiteTickOutcome {
                life: 2399,
                discard: false,
            }
        );

        assert!(endermite_spawn_allowed(true, true, true));
        assert!(!endermite_spawn_allowed(true, false, true));
        assert!(endermite_spawn_allowed(true, false, false));
        assert!(!endermite_spawn_allowed(false, true, false));
        assert!(endermite_from_ender_pearl(0.049, true));
        assert!(!endermite_from_ender_pearl(0.05, true));
        assert!(!endermite_from_ender_pearl(0.0, false));
        assert!(enderman_targets_endermite());
    }

    #[test]
    fn enderman_carry_stare_anger_and_teleport_gates_match_java() {
        assert_eq!(
            enderman_attributes(),
            EndermanAttributes {
                max_health: 40.0,
                movement_speed: 0.3,
                attacking_speed_bonus: 0.15,
                attack_damage: 7.0,
                follow_range: 64.0,
                step_height: 1.0,
            }
        );
        assert_eq!(
            enderman_entity_type_surface(),
            EndermanEntityTypeSurface {
                width: 0.6,
                height: 2.9,
                eye_height: 2.55,
                passenger_attachment_y: 2.80625,
                client_tracking_range: 8,
                not_in_peaceful: true,
            }
        );
        assert_eq!(ENDERMAN_WATER_PATHFINDING_MALUS, -1.0);
        assert_eq!(ENDERMAN_STARE_SOUND_COOLDOWN, 400);
        assert_eq!(ENDERMAN_MIN_DEAGGRESSION_TIME, 600);
        assert_eq!(ENDERMAN_PERSISTENT_ANGER_MIN_SECONDS, 20);
        assert_eq!(ENDERMAN_PERSISTENT_ANGER_MAX_SECONDS, 39);
        assert_eq!(ENDERMAN_LOOK_AT_PLAYER_RANGE, 8.0);
        assert_eq!(ENDERMAN_STARE_DOT_THRESHOLD, 0.025);
        assert_eq!(ENDERMAN_RANDOM_TELEPORT_HORIZONTAL_RANGE, 64.0);
        assert_eq!(ENDERMAN_RANDOM_TELEPORT_VERTICAL_RANGE, 64);
        assert_eq!(ENDERMAN_TELEPORT_TOWARDS_DISTANCE, 16.0);
        assert_eq!(ENDERMAN_TELEPORT_TOWARDS_RANDOM_HORIZONTAL, 8.0);
        assert_eq!(ENDERMAN_TELEPORT_TOWARDS_RANDOM_VERTICAL, 16);
        assert_eq!(ENDERMAN_PROJECTILE_TELEPORT_ATTEMPTS, 64);

        assert_eq!(
            enderman_set_target_state(true, 123),
            EndermanTargetState {
                target_change_time: 123,
                creepy: true,
                stared_at: false,
                speed_modifier_present: true,
            }
        );
        assert_eq!(
            enderman_set_target_state(false, 123),
            EndermanTargetState {
                target_change_time: 0,
                creepy: false,
                stared_at: false,
                speed_modifier_present: false,
            }
        );
        assert!(enderman_stare_sound_allowed(400, 0));
        assert!(!enderman_stare_sound_allowed(399, 0));
        assert!(enderman_should_daylight_deaggro_and_teleport(
            true, 700, 100, 0.6, true, 0.0
        ));
        assert!(!enderman_should_daylight_deaggro_and_teleport(
            true, 699, 100, 0.6, true, 0.0
        ));
        assert!(!enderman_should_daylight_deaggro_and_teleport(
            true, 700, 100, 0.5, true, 0.0
        ));

        assert_eq!(
            enderman_hurt_response(false, false, false, true, 1),
            EndermanHurtResponse::NormalHurt
        );
        assert_eq!(
            enderman_hurt_response(false, false, false, false, 1),
            EndermanHurtResponse::NormalHurtAndMaybeTeleport
        );
        assert_eq!(
            enderman_hurt_response(false, false, false, false, 0),
            EndermanHurtResponse::NormalHurt
        );
        assert_eq!(
            enderman_hurt_response(true, false, false, false, 0),
            EndermanHurtResponse::ProjectileTryTeleport64
        );
        assert_eq!(
            enderman_hurt_response(false, true, true, false, 0),
            EndermanHurtResponse::WaterPotionHurtAndTryTeleport64
        );

        assert!(enderman_freeze_when_looked_at(true, 256.0, true));
        assert!(!enderman_freeze_when_looked_at(true, 256.1, true));
        assert!(!enderman_freeze_when_looked_at(false, 1.0, true));
        assert_eq!(enderman_look_goal_starts_aggro(true), Some(5));
        assert_eq!(enderman_look_goal_starts_aggro(false), None);
        assert_eq!(
            enderman_look_goal_tick(Some(1), false, false, 0.0, 0, false),
            (Some(0), true, 0, false)
        );
        assert_eq!(
            enderman_look_goal_tick(None, true, true, 15.9, 9, false),
            (None, false, 0, true)
        );
        assert_eq!(
            enderman_look_goal_tick(None, true, false, 257.0, 30, false),
            (None, false, 31, true)
        );
        assert_eq!(
            enderman_look_goal_tick(None, true, false, 257.0, 29, false),
            (None, false, 30, false)
        );

        assert!(enderman_take_block_can_use(false, true, 0));
        assert!(!enderman_take_block_can_use(true, true, 0));
        assert!(!enderman_take_block_can_use(false, false, 0));
        assert!(!enderman_take_block_can_use(false, true, 1));
        assert!(enderman_leave_block_can_use(true, true, 0));
        assert!(!enderman_leave_block_can_use(false, true, 0));
        assert!(!enderman_leave_block_can_use(true, false, 0));
        assert!(enderman_can_place_carried_block(
            true, false, false, true, true, true
        ));
        assert!(!enderman_can_place_carried_block(
            true, true, false, true, true, true
        ));
        assert!(!enderman_can_place_carried_block(
            true, false, true, true, true, true
        ));
        assert!(!enderman_can_place_carried_block(
            true, false, false, false, true, true
        ));
        assert!(enderman_requires_custom_persistence(false, true));
        assert!(!enderman_requires_custom_persistence(false, false));
    }

    #[test]
    fn cave_spider_dimensions_poison_and_inherited_spider_gates_match_java_rules() {
        assert_eq!(
            cave_spider_attributes(),
            CaveSpiderAttributes {
                max_health: 12.0,
                movement_speed: 0.3,
            }
        );
        assert_eq!(
            cave_spider_entity_type_surface(),
            CaveSpiderEntityTypeSurface {
                width: 0.7,
                height: 0.5,
                eye_height: 0.45,
                client_tracking_range: 8,
                not_in_peaceful: true,
            }
        );

        assert_eq!(cave_spider_poison_duration_ticks("peaceful", true), None);
        assert_eq!(cave_spider_poison_duration_ticks("easy", true), None);
        assert_eq!(cave_spider_poison_duration_ticks("normal", true), Some(140));
        assert_eq!(cave_spider_poison_duration_ticks("hard", true), Some(300));
        assert_eq!(cave_spider_poison_duration_ticks("hard", false), None);
        assert_eq!(CAVE_SPIDER_POISON_AMPLIFIER, 0);

        assert_eq!(
            cave_spider_finalize_spawn_preserves_group_data(Some(7)),
            Some(7)
        );
        assert_eq!(
            cave_spider_vehicle_attachment_y(0.7, 0.7, 1.0),
            Some(0.21875)
        );
        assert_eq!(cave_spider_vehicle_attachment_y(0.8, 0.7, 1.0), None);

        let flags = spider_set_climbing_flags(0, true);
        assert!(spider_is_climbing(flags));
        assert!(!spider_is_climbing(spider_set_climbing_flags(flags, false)));
        assert!(spider_is_climbing(spider_tick_climbing_flags(0, true)));
        assert!(!spider_is_climbing(spider_tick_climbing_flags(
            flags, false
        )));
        assert!(spider_attack_goal_can_use(true, false));
        assert!(!spider_attack_goal_can_use(true, true));
        assert!(spider_target_goal_can_use(0.49, true));
        assert!(!spider_target_goal_can_use(0.5, true));
        assert!(spider_avoids_armadillo(false));
        assert!(!spider_avoids_armadillo(true));
        assert_eq!(SPIDER_POISON_IMMUNE, true);
        assert!(!spider_can_be_affected("minecraft:poison"));
        assert!(spider_can_be_affected("minecraft:speed"));
        assert!(spider_jockey_from_finalize_spawn(0));
        assert!(!spider_jockey_from_finalize_spawn(1));
        assert!(spider_should_drop_target_in_light(0.5, 0));
        assert!(!spider_should_drop_target_in_light(0.49, 0));
        assert!(!spider_should_drop_target_in_light(0.5, 1));
        assert_eq!(SPIDER_SPECIAL_EFFECT_CHANCE, 0.1);
        assert_eq!(SPIDER_VEHICLE_ATTACHMENT_Y, 0.3125);
        assert_eq!(SPIDER_AVOID_ARMADILLO_DISTANCE, 6.0);
        assert_eq!(SPIDER_AVOID_ARMADILLO_WALK_SPEED, 1.0);
        assert_eq!(SPIDER_AVOID_ARMADILLO_SPRINT_SPEED, 1.2);
        assert_eq!(SPIDER_LEAP_AT_TARGET_POWER, 0.4);
        assert_eq!(SPIDER_RANDOM_STROLL_SPEED, 0.8);
        assert_eq!(SPIDER_LOOK_AT_PLAYER_RANGE, 8.0);
        assert_eq!(
            spider_effect_from_group_data_selection(0),
            "minecraft:speed"
        );
        assert_eq!(
            spider_effect_from_group_data_selection(1),
            "minecraft:speed"
        );
        assert_eq!(
            spider_effect_from_group_data_selection(2),
            "minecraft:strength"
        );
        assert_eq!(
            spider_effect_from_group_data_selection(3),
            "minecraft:regeneration"
        );
        assert_eq!(
            spider_effect_from_group_data_selection(4),
            "minecraft:invisibility"
        );
        assert!(spider_should_roll_special_effect("hard", 0.09, 1.0));
        assert!(!spider_should_roll_special_effect("normal", 0.0, 1.0));
        assert!(!spider_should_roll_special_effect("hard", 0.11, 1.0));
    }

    #[test]
    fn pufferfish_puff_timing_and_contact_effects_match_java_thresholds() {
        let mut fish = PufferfishState::new();

        fish.start_inflating();
        fish.tick(true, true);
        assert_eq!(fish.puff_state, PufferfishState::MID);
        assert_eq!(fish.inflate_counter, 2);

        for _ in 0..39 {
            fish.tick(true, true);
        }
        assert_eq!(fish.puff_state, PufferfishState::MID);
        assert_eq!(fish.inflate_counter, 41);

        fish.tick(true, true);
        assert_eq!(fish.puff_state, PufferfishState::FULL);
        assert_eq!(
            fish.contact_effect(true, true),
            Some(PufferfishContactEffect {
                damage: 3,
                poison_effect: "minecraft:poison",
                poison_duration_ticks: 120,
                poison_amplifier: 0,
            })
        );
        assert_eq!(fish.contact_effect(true, false), None);

        fish.stop_inflating();
        for _ in 0..61 {
            fish.tick(true, true);
        }
        assert_eq!(fish.puff_state, PufferfishState::FULL);
        fish.tick(true, true);
        assert_eq!(fish.puff_state, PufferfishState::MID);
        assert_eq!(
            fish.contact_effect(true, true),
            Some(PufferfishContactEffect {
                damage: 2,
                poison_effect: "minecraft:poison",
                poison_duration_ticks: 60,
                poison_amplifier: 0,
            })
        );

        while fish.deflate_timer <= 100 {
            fish.tick(true, true);
        }
        fish.tick(true, true);
        assert_eq!(fish.puff_state, PufferfishState::SMALL);
        assert_eq!(fish.contact_effect(true, true), None);
    }

    #[test]
    fn salmon_size_variants_match_java_ids_scales_defaults_and_weights() {
        assert_eq!(DEFAULT_SALMON_VARIANT_ID, 1);
        assert_eq!(
            SALMON_VARIANTS,
            &[
                SalmonVariantModel {
                    name: "small",
                    id: 0,
                    bounding_box_scale: 0.5,
                    spawn_weight: 30,
                },
                SalmonVariantModel {
                    name: "medium",
                    id: 1,
                    bounding_box_scale: 1.0,
                    spawn_weight: 50,
                },
                SalmonVariantModel {
                    name: "large",
                    id: 2,
                    bounding_box_scale: 1.5,
                    spawn_weight: 15,
                },
            ]
        );
        assert_eq!(salmon_variant_by_id(-1).name, "small");
        assert_eq!(salmon_variant_by_id(99).name, "large");
        assert_eq!(salmon_variant_by_name("medium").unwrap().id, 1);
        assert_eq!(salmon_spawn_weight_total(), 95);
    }

    #[test]
    fn tropical_fish_packed_variants_match_java_pattern_and_color_layout() {
        assert_eq!(DEFAULT_TROPICAL_FISH_VARIANT_PACKED_ID, 0);
        assert_eq!(TROPICAL_FISH_PATTERNS.len(), 12);

        let kob = TROPICAL_FISH_PATTERNS[0];
        let flopper = TROPICAL_FISH_PATTERNS[6];
        let stripey = TROPICAL_FISH_PATTERNS[7];
        let clayfish = TROPICAL_FISH_PATTERNS[11];
        assert_eq!(tropical_fish_pattern_packed_id(kob), 0);
        assert_eq!(tropical_fish_pattern_packed_id(flopper), 1);
        assert_eq!(tropical_fish_pattern_packed_id(stripey), 257);
        assert_eq!(tropical_fish_pattern_packed_id(clayfish), 1281);

        let packed = tropical_fish_pack_variant(stripey, 1, 7);
        assert_eq!(packed, 117_506_305);
        assert_eq!(tropical_fish_pattern_from_variant(packed), stripey);
        assert_eq!(tropical_fish_base_color_id(packed), 1);
        assert_eq!(tropical_fish_pattern_color_id(packed), 7);
        assert_eq!(tropical_fish_pattern_by_packed_id(99_999), kob);

        let common = tropical_fish_common_variants();
        assert_eq!(common.len(), 22);
        assert_eq!(common[0].pattern.name, "stripey");
        assert_eq!(
            tropical_fish_pack_variant(
                common[0].pattern,
                common[0].base_color_id,
                common[0].pattern_color_id
            ),
            packed
        );
        assert_eq!(common[21].pattern.name, "flopper");
        assert_eq!(common[21].base_color_id, 4);
        assert_eq!(common[21].pattern_color_id, 4);
    }

    #[test]
    fn abstract_fish_bucket_and_schooling_gates_match_java_rules() {
        assert_eq!(
            abstract_fish_attributes(),
            AbstractFishAttributes {
                max_health: 3.0,
                panic_speed_modifier: 1.25,
                avoid_player_distance: 8.0,
                avoid_player_near_speed: 1.6,
                avoid_player_far_speed: 1.4,
                random_swim_speed_modifier: 1.0,
                random_swim_interval_ticks: 40,
                water_drag: 0.9,
                no_target_gravity: -0.005,
            }
        );
        assert_eq!(ABSTRACT_FISH_MAX_SPAWN_CLUSTER_SIZE, 8);
        assert_eq!(SALMON_MAX_SCHOOL_SIZE, 5);
        assert_eq!(ABSTRACT_FISH_EYE_WATER_BOOST_Y, 0.005);
        assert_eq!(ABSTRACT_FISH_FLOP_JUMP_Y, 0.4);
        assert_eq!(ABSTRACT_FISH_FLOP_RANDOM_XZ_SCALE, 0.05);
        assert_eq!(SCHOOLING_FISH_NEIGHBOR_SCAN_RANGE, 8.0);
        assert_eq!(SCHOOLING_FISH_LEADER_RANGE_SQR, 121.0);

        assert!(fish_requires_custom_persistence(false, true));
        assert!(fish_requires_custom_persistence(true, false));
        assert!(!fish_requires_custom_persistence(false, false));
        assert!(fish_remove_when_far_away(false, false));
        assert!(!fish_remove_when_far_away(true, false));
        assert!(!fish_remove_when_far_away(false, true));
        assert!(fish_can_random_swim(false));
        assert!(!fish_can_random_swim(true));

        assert_eq!(
            fish_flop_step(false, true, true),
            FishFlopStep {
                jump: true,
                sync_needed: true,
                play_flop_sound: true,
            }
        );
        assert!(!fish_flop_step(true, true, true).jump);
        assert_eq!(fish_travel_y_delta(false, 0.0), -0.005);
        assert_eq!(fish_travel_y_delta(true, 0.1), 0.1);

        assert!(schooling_fish_is_follower(true, true));
        assert!(!schooling_fish_is_follower(true, false));
        assert!(schooling_fish_can_be_followed(SchoolingFishState {
            school_size: 2,
            max_school_size: 8,
            leader_alive: true,
        }));
        assert!(!schooling_fish_can_be_followed(SchoolingFishState {
            school_size: 1,
            max_school_size: 8,
            leader_alive: true,
        }));
        assert_eq!(schooling_fish_followers_added(3, 8, 10), 5);
        assert!(schooling_fish_should_reset_size(1, 1));
        assert!(!schooling_fish_should_reset_size(2, 1));
        assert!(!schooling_fish_should_reset_size(1, 2));

        assert_eq!(
            fish_bucket_model("minecraft:cod", true),
            Some(FishBucketModel {
                bucket_item: "minecraft:cod_bucket",
                pickup_sound: "minecraft:item.bucket.fill_fish",
                from_bucket_save_field: "FromBucket",
                requires_persistence_from_bucket: true,
                discard_on_pickup: true,
            })
        );
        assert_eq!(
            fish_bucket_model("minecraft:salmon", false)
                .expect("salmon bucket model")
                .bucket_item,
            "minecraft:salmon_bucket"
        );
        assert_eq!(
            fish_bucket_model("minecraft:tropical_fish", false)
                .expect("tropical fish bucket model")
                .bucket_item,
            "minecraft:tropical_fish_bucket"
        );
        assert_eq!(
            fish_bucket_model("minecraft:pufferfish", false)
                .expect("pufferfish bucket model")
                .bucket_item,
            "minecraft:pufferfish_bucket"
        );
        assert_eq!(fish_bucket_model("minecraft:squid", false), None);
    }

    #[test]
    fn mob_interaction_checklist_surface_is_covered() {
        let covered: Vec<&str> = MOB_INTERACTION_COVERAGE
            .iter()
            .flat_map(|entry| entry.covered_rules.iter().copied())
            .collect();
        for expected in [
            "tameable",
            "breedable",
            "rideable",
            "shearable",
            "bucketable",
            "variant",
            "ageable",
            "trading",
            "anger",
            "conversion",
            "transformation",
        ] {
            assert!(covered.contains(&expected), "missing {expected}");
        }
    }
}
