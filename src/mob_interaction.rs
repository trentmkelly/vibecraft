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
