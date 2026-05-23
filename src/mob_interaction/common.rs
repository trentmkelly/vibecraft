use super::*;

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

