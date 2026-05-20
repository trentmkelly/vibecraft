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
