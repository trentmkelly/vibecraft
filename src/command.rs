#![allow(dead_code)]

use std::net::IpAddr;

use crate::enchantment_system::{are_compatible, enchantment};
use crate::entity_category::mob_category;
use crate::player_access::{BanEntry, NameAndId};
use crate::runtime::{TickRateController, MAX_TICK_RATE, MIN_TICK_RATE};
use crate::storage::nbt::{parse_snbt, Tag};
use crate::world_border::{WorldBorder, WORLD_BORDER_MAX_CENTER_COORDINATE, WORLD_BORDER_MAX_SIZE};
use crate::worldgen::{
    builtin_noise_router, configured_feature, get_biome, noise_router_id_for_settings,
    resolve_world_preset, ClimateSampler, ResolvedChunkGenerator, JIGSAW_STRUCTURE_START_POOLS,
};

mod models;
pub use models::*;

mod functions;
pub use functions::*;

mod world_models;
pub use world_models::*;

mod errors_versions;
pub use errors_versions::*;

mod impls;

mod dispatch;
pub use dispatch::*;

mod admin_player;
use admin_player::*;

mod inventory_items;
use inventory_items::*;
pub(crate) use inventory_items::item_max_stack_size;

mod locate_loot_place_raid;
use locate_loot_place_raid::*;

mod teleport_time_ui;
use teleport_time_ui::*;

mod world_editing;
pub use world_editing::*;

mod server_data_debug;
use server_data_debug::*;

mod execute_experience_profile;
use execute_experience_profile::*;

mod execute_args;
mod execute_command;
use execute_command::execute_command;
mod execute_condition_support;
mod execute_conditions;
mod execute_math;
mod execute_runner;
mod execute_store;

mod enchant_gamemode_rules;
use enchant_gamemode_rules::*;

mod sound_advancement_attribute;
use sound_advancement_attribute::*;

mod stopwatch_schedule_function;
use stopwatch_schedule_function::*;

mod scoreboard;
use scoreboard::*;

mod spawn_spread_spectate;
use spawn_spread_spectate::*;

mod entity_team_misc;
use entity_team_misc::*;

mod parsers_usage_random;
pub use parsers_usage_random::*;

mod permissions;
pub use permissions::*;

#[cfg(test)]
mod tests;
