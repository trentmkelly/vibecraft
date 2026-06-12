#![allow(dead_code)]

pub const ENEMY_XP_REWARD_NONE: i32 = 0;
pub const ENEMY_XP_REWARD_SMALL: i32 = 3;
pub const ENEMY_XP_REWARD_MEDIUM: i32 = 5;
pub const ENEMY_XP_REWARD_LARGE: i32 = 10;
pub const ENEMY_XP_REWARD_HUGE: i32 = 20;
pub const ENEMY_XP_REWARD_BOSS: i32 = 50;
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
pub const HORSE_VARIANT_COUNT: i32 = 7;
pub const HORSE_MARKINGS_COUNT: i32 = 5;
pub const HORSE_BABY_SCALE: f32 = 0.7;
pub const CHESTED_HORSE_BABY_SCALE: f32 = 0.5;
pub const CHESTED_HORSE_INVENTORY_COLUMNS: i32 = 5;
pub const CHESTED_HORSE_MOVEMENT_SPEED: f32 = 0.175;
pub const CHESTED_HORSE_JUMP_STRENGTH: f64 = 0.5;
pub const LLAMA_MAX_STRENGTH: i32 = 5;
pub const LLAMA_COMMON_MAX_STRENGTH: i32 = 3;
pub const LLAMA_RARE_MAX_STRENGTH_CHANCE: f32 = 0.04;
pub const LLAMA_BREED_STRENGTH_BONUS_CHANCE: f32 = 0.03;
pub const LLAMA_MAX_TEMPER: i32 = 30;
pub const LLAMA_RANGED_ATTACK_INTERVAL_TICKS: i32 = 40;
pub const LLAMA_RANGED_ATTACK_RADIUS: f32 = 20.0;
pub const LLAMA_SPIT_SPEED: f32 = 1.5;
pub const LLAMA_SPIT_INACCURACY: f32 = 10.0;
pub const VILLAGER_INVENTORY_SIZE: usize = 8;
pub const VILLAGER_INVENTORY_SLOT_OFFSET: i32 = 300;
pub const NO_ANGER_END_TIME: i64 = -1;
pub const SNOW_GOLEM_PUMPKIN_FLAG: u8 = 16;
pub const SNOW_GOLEM_MAX_HEALTH: f32 = 4.0;
pub const SNOW_GOLEM_MOVEMENT_SPEED: f32 = 0.2;
pub const SNOW_GOLEM_RANGED_ATTACK_INTERVAL_TICKS: i32 = 20;
pub const SNOW_GOLEM_RANGED_ATTACK_RADIUS: f32 = 10.0;
pub const SNOW_GOLEM_SNOWBALL_SPEED: f32 = 1.6;
pub const SNOW_GOLEM_SNOWBALL_INACCURACY: f32 = 12.0;
pub const IRON_GOLEM_PLAYER_CREATED_FLAG: u8 = 1;
pub const IRON_GOLEM_MAX_HEALTH: f32 = 100.0;
pub const IRON_GOLEM_MOVEMENT_SPEED: f32 = 0.25;
pub const IRON_GOLEM_KNOCKBACK_RESISTANCE: f32 = 1.0;
pub const IRON_GOLEM_ATTACK_DAMAGE: f32 = 15.0;
pub const IRON_GOLEM_STEP_HEIGHT: f32 = 1.0;
pub const IRON_GOLEM_REPAIR_HEAL_AMOUNT: f32 = 25.0;
pub const IRON_GOLEM_ATTACK_ANIMATION_TICKS: i32 = 10;
pub const IRON_GOLEM_OFFER_FLOWER_TICKS: i32 = 400;
pub const FOX_FLAG_SITTING: u8 = 1;
pub const FOX_FLAG_CROUCHING: u8 = 4;
pub const FOX_FLAG_INTERESTED: u8 = 8;
pub const FOX_FLAG_POUNCING: u8 = 16;
pub const FOX_FLAG_SLEEPING: u8 = 32;
pub const FOX_FLAG_FACEPLANTED: u8 = 64;
pub const FOX_FLAG_DEFENDING: u8 = 128;
pub const FOX_MIN_TICKS_BEFORE_EAT: i32 = 600;
pub const FOX_BERRY_WAIT_TICKS: i32 = 40;
pub const FOX_STALK_DISTANCE_SQR: f32 = 36.0;
pub const FOX_POUNCE_JUMP_SCALE: f32 = 0.4;
pub const PANDA_FLAG_SNEEZE: u8 = 2;
pub const PANDA_FLAG_ROLL: u8 = 4;
pub const PANDA_FLAG_SIT: u8 = 8;
pub const PANDA_FLAG_ON_BACK: u8 = 16;
pub const PANDA_EAT_TICK_INTERVAL: i32 = 5;
pub const PANDA_TOTAL_ROLL_STEPS: i32 = 32;
pub const PANDA_TOTAL_UNHAPPY_TIME: i32 = 32;
pub const PANDA_BASE_MOVEMENT_SPEED: f32 = 0.15;
pub const PANDA_LAZY_MOVEMENT_SPEED: f32 = 0.07;
pub const PANDA_ATTACK_DAMAGE: f32 = 6.0;
pub const PANDA_WEAK_MAX_HEALTH: f32 = 10.0;
pub const PARROT_MAX_HEALTH: f32 = 6.0;
pub const PARROT_FLYING_SPEED: f32 = 0.4;
pub const PARROT_MOVEMENT_SPEED: f32 = 0.2;
pub const PARROT_ATTACK_DAMAGE: f32 = 3.0;
pub const PARROT_TAME_ROLL_BOUND: i32 = 10;
pub const PARROT_POISON_TICKS: i32 = 900;
pub const PARROT_JUKEBOX_PARTY_DISTANCE: f32 = 3.46;
pub const PARROT_MIMIC_SCAN_RADIUS: f32 = 20.0;
pub const PARROT_MIMIC_TICK_ROLL_BOUND: i32 = 400;
pub const PARROT_MIMIC_SOUND_ROLL_BOUND: i32 = 2;
pub const SHOULDER_RIDING_COOLDOWN_TICKS: i32 = 100;
pub const HAPPY_GHAST_BABY_SCALE: f32 = 0.2375;
pub const HAPPY_GHAST_WANDER_GROUND_DISTANCE: i32 = 16;
pub const HAPPY_GHAST_SMALL_RESTRICTION_RADIUS: i32 = 32;
pub const HAPPY_GHAST_LARGE_RESTRICTION_RADIUS: i32 = 64;
pub const HAPPY_GHAST_RESTRICTION_RADIUS_BUFFER: i32 = 16;
pub const HAPPY_GHAST_FAST_HEALING_TICKS: i32 = 20;
pub const HAPPY_GHAST_SLOW_HEALING_TICKS: i32 = 600;
pub const HAPPY_GHAST_MAX_PASSENGERS: usize = 4;
pub const HAPPY_GHAST_STILL_TIMEOUT_ON_LOAD_GRACE_PERIOD: i32 = 60;
pub const HAPPY_GHAST_MAX_STILL_TIMEOUT: i32 = 10;
pub const HAPPY_GHAST_SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.0;
pub const HAPPY_GHAST_MAX_HEALTH: f32 = 20.0;
pub const HAPPY_GHAST_TEMPT_RANGE: f32 = 16.0;
pub const HAPPY_GHAST_FLYING_SPEED: f32 = 0.05;
pub const HAPPY_GHAST_MOVEMENT_SPEED: f32 = 0.05;
pub const HAPPY_GHAST_FOLLOW_RANGE: f32 = 16.0;
pub const HAPPY_GHAST_CAMERA_DISTANCE: f32 = 8.0;
pub const HAPPY_GHAST_LEASH_ELASTIC_DISTANCE: f32 = 10.0;
pub const HAPPY_GHAST_LEASH_SNAP_DISTANCE: f32 = 16.0;
pub const DRIED_GHAST_READY_HYDRATION_LEVEL: i32 = 3;
pub const SNIFFER_DIGGING_PARTICLES_DELAY_TICKS: i32 = 1700;
pub const SNIFFER_DIGGING_PARTICLES_DURATION_TICKS: i32 = 6000;
pub const SNIFFER_DIGGING_PARTICLES_AMOUNT: i32 = 30;
pub const SNIFFER_DIGGING_DROP_SEED_OFFSET_TICKS: i32 = 120;
pub const SNIFFER_BABY_START_AGE: i32 = -48000;
pub const SNIFFER_DIGGING_BB_HEIGHT_OFFSET: f32 = 0.4;
pub const SNIFFER_MOVEMENT_SPEED: f32 = 0.1;
pub const SNIFFER_MAX_HEALTH: f32 = 14.0;
pub const SNIFFER_EXPLORED_POSITION_LIMIT: usize = 20;
pub const SNIFFER_SNIFF_COOLDOWN_TICKS: i32 = 9600;
pub const SNIFFER_DIGGING_MIN_TICKS: i32 = 160;
pub const SNIFFER_DIGGING_MAX_TICKS: i32 = 180;
pub const SNIFFER_FINISHED_DIGGING_TICKS: i32 = 40;
pub const SNIFFER_SEARCHING_TICKS: i32 = 600;
pub const SNIFFER_EGG_MAX_HATCH_LEVEL: i32 = 2;
pub const SNIFFER_EGG_REGULAR_HATCH_TIME_TICKS: i32 = 24000;
pub const SNIFFER_EGG_BOOSTED_HATCH_TIME_TICKS: i32 = 12000;
pub const SNIFFER_EGG_RANDOM_HATCH_OFFSET_TICKS: i32 = 300;
pub const TURTLE_BABY_SCALE: f32 = 0.3;
pub const TURTLE_MAX_HEALTH: f32 = 30.0;
pub const TURTLE_MOVEMENT_SPEED: f32 = 0.25;
pub const TURTLE_STEP_HEIGHT: f32 = 1.0;
pub const TURTLE_AMBIENT_SOUND_INTERVAL: i32 = 200;
pub const TURTLE_BREED_PARENT_AGE: i32 = 6000;
pub const TURTLE_LAY_EGG_DELAY_TICKS: i32 = 200;
pub const TURTLE_LAY_EGG_PARTICLE_INTERVAL_TICKS: i32 = 5;
pub const TURTLE_GO_HOME_RANDOM_INTERVAL: i32 = 700;
pub const TURTLE_GO_HOME_GIVE_UP_TICKS: i32 = 600;
pub const TURTLE_GO_HOME_DISTANCE: f32 = 64.0;
pub const TURTLE_GO_HOME_CLOSE_DISTANCE: f32 = 7.0;
pub const TURTLE_LAY_EGG_HOME_DISTANCE: f32 = 9.0;
pub const TURTLE_GO_TO_WATER_GIVE_UP_TICKS: i32 = 1200;
pub const TURTLE_GO_TO_WATER_RECALC_INTERVAL_TICKS: i32 = 160;
pub const TURTLE_TRAVEL_XZ_RANGE: i32 = 512;
pub const TURTLE_TRAVEL_Y_RANGE: i32 = 4;
pub const TURTLE_EGG_MAX_HATCH_LEVEL: i32 = 2;
pub const TURTLE_EGG_MIN_EGGS: i32 = 1;
pub const TURTLE_EGG_MAX_EGGS: i32 = 4;
pub const TURTLE_EGG_STEP_RANDOM_BOUND: i32 = 100;
pub const TURTLE_EGG_FALL_RANDOM_BOUND: i32 = 3;
pub const TURTLE_HATCHLING_AGE: i32 = -24000;
pub const ARMADILLO_BABY_SCALE: f32 = 0.6;
pub const ARMADILLO_MAX_HEAD_ROTATION_EXTENT: f32 = 32.5;
pub const ARMADILLO_SCARE_CHECK_INTERVAL: i32 = 80;
pub const ARMADILLO_SCARE_DISTANCE_HORIZONTAL: f32 = 7.0;
pub const ARMADILLO_SCARE_DISTANCE_VERTICAL: f32 = 2.0;
pub const ARMADILLO_MAX_HEALTH: f32 = 12.0;
pub const ARMADILLO_MOVEMENT_SPEED: f32 = 0.14;
pub const ARMADILLO_SCUTE_DROP_MIN_TICKS: i32 = 20 * 60 * 5;
pub const ARMADILLO_SCUTE_DROP_RANDOM_BOUND: i32 = 20 * 60 * 5;
pub const ARMADILLO_BRUSH_DAMAGE: i32 = 16;
pub const ARMADILLO_BALL_UP_STAY_IN_STATE_TICKS: i32 = 5 * 60 * 20;
pub const ARMADILLO_DANGER_DELAY_TICKS: i32 = 5;
pub const ARMADILLO_DANGER_THRESHOLD_TICKS: i32 = 75;
pub const ARMADILLO_PEEK_EVENT: u8 = 64;
pub const ALLAY_ITEM_PICKUP_REACH: (i32, i32, i32) = (1, 1, 1);
pub const ALLAY_LIFTING_ITEM_ANIMATION_DURATION: i32 = 5;
pub const ALLAY_DANCING_LOOP_DURATION: f32 = 55.0;
pub const ALLAY_SPINNING_ANIMATION_DURATION: f32 = 15.0;
pub const ALLAY_DEFAULT_DUPLICATION_COOLDOWN: i32 = 0;
pub const ALLAY_DUPLICATION_COOLDOWN_TICKS: i32 = 6000;
pub const ALLAY_NUM_DUPLICATION_HEARTS: i32 = 3;
pub const ALLAY_MAX_NOTEBLOCK_DISTANCE: i32 = 1024;
pub const ALLAY_VIBRATION_LISTENER_RANGE: i32 = 16;
pub const ALLAY_TIME_TO_FORGET_NOTEBLOCK: i32 = 600;
pub const ALLAY_DISTANCE_TO_WANTED_ITEM: i32 = 32;
pub const ALLAY_GIVE_ITEM_TIMEOUT_DURATION: i32 = 20;
pub const ALLAY_LIKED_PLAYER_DISTANCE: f32 = 64.0;
pub const ALLAY_MAX_HEALTH: f32 = 20.0;
pub const ALLAY_FLYING_SPEED: f32 = 0.1;
pub const ALLAY_MOVEMENT_SPEED: f32 = 0.1;
pub const ALLAY_ATTACK_DAMAGE: f32 = 2.0;
pub const ALLAY_DUPLICATION_EVENT: u8 = 18;
pub const FELINE_CROUCH_SPEED_MOD: f32 = 0.6;
pub const FELINE_WALK_SPEED_MOD: f32 = 0.8;
pub const FELINE_SPRINT_SPEED_MOD: f32 = 1.33;
pub const FELINE_MAX_HEALTH: f32 = 10.0;
pub const FELINE_MOVEMENT_SPEED: f32 = 0.3;
pub const FELINE_ATTACK_DAMAGE: f32 = 3.0;
pub const FELINE_PLAYER_AVOID_DISTANCE: f32 = 16.0;
pub const FELINE_REMOVE_WHEN_FAR_TICKS: i32 = 2400;
pub const CAT_AMBIENT_SOUND_INTERVAL: i32 = 120;
pub const OCELOT_AMBIENT_SOUND_INTERVAL: i32 = 900;
pub const CAT_OWNER_RELAX_DISTANCE_SQR: f32 = 100.0;
pub const CAT_LIE_ON_OWNER_DISTANCE_SQR: f32 = 2.5;
pub const CAT_ON_BED_RELAX_TICKS: i32 = 16;
pub const CAT_BEG_SOUND_INTERVAL_TICKS: i32 = 100;
pub const CAT_TEMPT_SELECT_INTERVAL_TICKS: i32 = 600;
pub const CAT_TEMPT_FORGET_INTERVAL_TICKS: i32 = 500;
pub const CAT_TAME_ROLL_BOUND: i32 = 3;
pub const OCELOT_TRUST_ROLL_BOUND: i32 = 3;
pub const CAT_STRAY_SPAWNER_TICK_DELAY: i32 = 1200;
pub const CAT_VILLAGE_HOME_POI_RADIUS: i32 = 48;
pub const CAT_VILLAGE_MIN_OCCUPIED_HOMES: i32 = 5;
pub const CAT_VILLAGE_MAX_CATS: usize = 5;
pub const CAT_HUT_CAT_RADIUS: i32 = 16;
pub const OCELOT_SPAWN_ROLL_BOUND: i32 = 3;

pub const TAMABLE_FLAG_SITTING: u8 = 1;
pub const TAMABLE_FLAG_TAME: u8 = 4;
pub const HORSE_FLAG_TAME: u8 = 2;
pub const HORSE_FLAG_BRED: u8 = 8;
pub const HORSE_FLAG_EATING: u8 = 16;
pub const HORSE_FLAG_STANDING: u8 = 32;
pub const HORSE_FLAG_OPEN_MOUTH: u8 = 64;

fn set_flag(flags: &mut u8, flag: u8, value: bool) {
    if value {
        *flags |= flag;
    } else {
        *flags &= !flag;
    }
}

fn lerp(part: f32, start: f32, end: f32) -> f32 {
    start + part * (end - start)
}

mod monsters;
#[cfg(test)]
use monsters::*;

mod golems;
#[cfg(test)]
use golems::*;

mod frogs_foxes;
#[cfg(test)]
use frogs_foxes::*;

mod pandas;
#[cfg(test)]
use pandas::*;

mod parrots_happy_ghasts;
#[cfg(test)]
use parrots_happy_ghasts::*;

mod dried_ghasts_sniffers;
#[cfg(test)]
use dried_ghasts_sniffers::*;

mod turtles;
#[cfg(test)]
use turtles::*;

mod armadillos;
#[cfg(test)]
use armadillos::*;

mod allays;
#[cfg(test)]
use allays::*;

mod felines;
#[cfg(test)]
use felines::*;

mod creepers;
#[cfg(test)]
use creepers::*;

mod slimes;
#[cfg(test)]
use slimes::*;

mod phantoms;
#[cfg(test)]
use phantoms::*;

mod vexes;
#[cfg(test)]
use vexes::*;

mod silverfish;
#[cfg(test)]
use silverfish::*;

mod piglin_hoglin_types;
pub use piglin_hoglin_types::*;

mod hoglins_piglins;
#[cfg(test)]
use hoglins_piglins::*;

mod ghasts;
#[cfg(test)]
use ghasts::*;

mod striders;
#[cfg(test)]
use striders::*;

mod witches;
#[cfg(test)]
use witches::*;

mod guardians;
pub use guardians::*;

mod ravagers;
#[cfg(test)]
use ravagers::*;

mod shulkers;
#[cfg(test)]
use shulkers::*;

mod giants;
#[cfg(test)]
use giants::*;

mod zombies;
pub use zombies::*;

mod zombie_villagers;
#[cfg(test)]
pub use zombie_villagers::*;

mod blazes;
#[cfg(test)]
use blazes::*;

mod zombified_piglins;
pub use zombified_piglins::*;

mod drowned;
#[cfg(test)]
use drowned::*;

mod husks;
#[cfg(test)]
use husks::*;

mod endermites;
#[cfg(test)]
use endermites::*;

mod endermen;
#[cfg(test)]
use endermen::*;

mod skeletons;
#[cfg(test)]
use skeletons::*;

mod spiders;
#[cfg(test)]
use spiders::*;

mod common;
pub use common::*;

mod axolotls;
#[cfg(test)]
use axolotls::*;

mod chickens_cows;
#[cfg(test)]
use chickens_cows::*;

mod dolphins;
#[cfg(test)]
use dolphins::*;

mod bees;
#[cfg(test)]
use bees::*;

mod camels;
#[cfg(test)]
use camels::*;

mod goats;
#[cfg(test)]
use goats::*;

mod pigs;
#[cfg(test)]
use pigs::*;

mod polar_bears;
#[cfg(test)]
use polar_bears::*;

mod rabbits;
#[cfg(test)]
use rabbits::*;

mod sheep;
pub use sheep::*;

mod squids;
#[cfg(test)]
use squids::*;

mod fish_variants;
#[cfg(test)]
use fish_variants::*;

mod fish_common;
#[cfg(test)]
use fish_common::*;

mod bucketables_mooshrooms;
pub use bucketables_mooshrooms::*;

mod horses_llamas;
#[cfg(test)]
use horses_llamas::*;

mod villagers_anger_conversion;
pub use villagers_anger_conversion::*;

mod coverage;
#[cfg(test)]
use coverage::*;

#[cfg(test)]
mod tests;
