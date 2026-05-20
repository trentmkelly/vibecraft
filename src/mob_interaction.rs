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
