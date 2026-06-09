#[cfg(test)]
mod advancement_criteria;
#[cfg(test)]
mod advancement_display;
#[cfg(test)]
mod advancement_model;
#[cfg(test)]
mod advancement_progress;
#[cfg(test)]
mod advancement_rewards;
#[cfg(test)]
mod advancement_system;
#[cfg(test)]
mod advancement_tree;
#[cfg(test)]
mod advancement_tree_position;
#[cfg(test)]
mod advancement_trigger_registry;
mod ai_system;
mod attribute_system;
mod base_entity;
mod biome;
mod block_behavior;
mod block_behaviour_defaults;
#[cfg(test)]
mod block_behaviour_defaults_tests;
mod block_behaviour_properties;
#[cfg(test)]
mod block_behavior_tests;
mod block_catalog;
mod block_entity;
mod block_metadata;
#[cfg(test)]
mod block_regression;
mod block_sounds;
mod block_update;
mod boss_fight;
mod chat_component;
mod chat_formatting;
mod chat_trust;
mod char_predicate;
mod chunk_manager;
mod chunk_task;
mod chunk_ticket;
mod chunk_watchdog;
mod cli;
mod collision_shape;
mod combat_damage;
mod combat_tracker;
mod command;
#[cfg(test)]
mod command_angle_argument;
#[cfg(test)]
mod command_argument_signatures;
#[cfg(test)]
mod command_argument_visitor;
#[cfg(test)]
mod command_brigadier_exceptions;
#[cfg(test)]
mod command_block_arguments;
#[cfg(test)]
mod command_cacheable_function;
#[cfg(test)]
mod command_block_position_arguments;
#[cfg(test)]
mod command_component_argument;
#[cfg(test)]
mod command_nbt_arguments;
#[cfg(test)]
mod command_nbt_path_argument;
#[cfg(test)]
mod command_build_context;
#[cfg(test)]
mod command_color_argument;
#[cfg(test)]
mod command_commands;
#[cfg(test)]
mod command_coordinate_arguments;
#[cfg(test)]
mod command_dimension_argument;
#[cfg(test)]
mod command_execution_source;
#[cfg(test)]
mod command_entity_anchor_argument;
#[cfg(test)]
mod command_entity_argument;
#[cfg(test)]
mod command_function_instantiation_exception;
#[cfg(test)]
mod command_game_mode_argument;
#[cfg(test)]
mod command_game_profile_argument;
#[cfg(test)]
mod command_hex_color_argument;
#[cfg(test)]
mod command_heightmap_type_argument;
#[cfg(test)]
mod command_identifier_argument;
#[cfg(test)]
mod command_item_arguments;
#[cfg(test)]
mod command_message_argument;
#[cfg(test)]
mod command_misc_argument_audits;
#[cfg(test)]
mod command_operation_argument;
#[cfg(test)]
mod command_objective_argument;
#[cfg(test)]
mod command_objective_criteria_argument;
#[cfg(test)]
mod command_parser_utils;
#[cfg(test)]
mod command_particle_argument;
#[cfg(test)]
mod command_range_argument;
#[cfg(test)]
mod command_result_callback;
#[cfg(test)]
mod command_resource_argument;
#[cfg(test)]
mod command_resource_key_argument;
#[cfg(test)]
mod command_resource_or_id_argument;
#[cfg(test)]
mod command_resource_or_tag_argument;
#[cfg(test)]
mod command_resource_or_tag_key_argument;
#[cfg(test)]
mod command_resource_selector_argument;
#[cfg(test)]
mod command_score_holder_argument;
#[cfg(test)]
mod command_signing_context;
#[cfg(test)]
mod command_slot_arguments;
#[cfg(test)]
mod command_style_argument;
#[cfg(test)]
mod command_swizzle_argument;
#[cfg(test)]
mod command_source;
#[cfg(test)]
mod command_source_stack;
#[cfg(test)]
mod command_scoreboard_slot_argument;
#[cfg(test)]
mod command_shared_suggestion_provider;
#[cfg(test)]
mod command_string_representable_argument;
mod command_execution;
mod command_feedback;
mod command_parity;
#[cfg(test)]
mod command_selector;
mod command_tree;
#[cfg(test)]
mod command_template_transform_arguments;
#[cfg(test)]
mod command_synchronization;
#[cfg(test)]
mod command_team_argument;
#[cfg(test)]
mod command_time_argument;
#[cfg(test)]
mod command_uuid_argument;
mod console;
mod container_block;
mod container_menus;
#[cfg(test)]
mod core_block_math;
#[cfg(test)]
mod core_block_box;
#[cfg(test)]
mod core_block_pos;
#[cfg(test)]
mod core_components;
#[cfg(test)]
mod core_component_predicates;
#[cfg(test)]
mod core_defaulted_registry;
#[cfg(test)]
mod core_dispenser;
#[cfg(test)]
mod core_direction;
#[cfg(test)]
mod core_holders;
#[cfg(test)]
mod core_layered_registry;
#[cfg(test)]
mod core_mapped_registry;
#[cfg(test)]
mod core_misc;
#[cfg(test)]
mod core_particles;
#[cfg(test)]
mod core_registry_codecs;
#[cfg(test)]
mod core_registries;
#[cfg(test)]
mod core_registry_helpers;
#[cfg(test)]
mod core_registry_interface;
#[cfg(test)]
mod core_registry_set_builder;
#[cfg(test)]
mod core_registry_synchronization;
#[cfg(test)]
mod core_section_pos;
#[cfg(test)]
mod core_orientation;
#[cfg(test)]
mod criterion_bred_animals;
#[cfg(test)]
mod criterion_brewed_potion;
#[cfg(test)]
mod criterion_change_dimension;
#[cfg(test)]
mod criterion_channeled_lightning;
#[cfg(test)]
mod criterion_collection_predicates;
#[cfg(test)]
mod criterion_construct_beacon;
#[cfg(test)]
mod criterion_consume_item;
#[cfg(test)]
mod criterion_context_aware_predicate;
#[cfg(test)]
mod criterion_cured_zombie_villager;
#[cfg(test)]
mod criterion_damage_predicate;
#[cfg(test)]
mod criterion_damage_source_predicate;
#[cfg(test)]
mod criterion_data_component_matchers;
#[cfg(test)]
mod criterion_distance_predicate;
#[cfg(test)]
mod criterion_distance_trigger;
#[cfg(test)]
mod criterion_effects_changed;
#[cfg(test)]
mod criterion_enchanted_item;
#[cfg(test)]
mod criterion_enchantment_predicate;
#[cfg(test)]
mod criterion_enter_block;
#[cfg(test)]
mod criterion_entity_equipment_predicate;
#[cfg(test)]
mod criterion_entity_flags_predicate;
#[cfg(test)]
mod criterion_entity_hurt_player;
#[cfg(test)]
mod criterion_entity_predicate;
#[cfg(test)]
mod criterion_entity_sub_predicates;
#[cfg(test)]
mod criterion_entity_type_predicate;
#[cfg(test)]
mod criterion_fall_after_explosion;
#[cfg(test)]
mod criterion_filled_bucket;
#[cfg(test)]
mod criterion_fishing_hook_predicate;
#[cfg(test)]
mod criterion_fishing_rod_hooked;
#[cfg(test)]
mod criterion_fluid_predicate;
#[cfg(test)]
mod criterion_food_predicate;
#[cfg(test)]
mod criterion_game_type_predicate;
#[cfg(test)]
mod criterion_impossible_trigger;
#[cfg(test)]
mod criterion_input_predicate;
#[cfg(test)]
mod criterion_inventory_change;
#[cfg(test)]
mod criterion_item_durability;
#[cfg(test)]
mod criterion_item_predicate;
#[cfg(test)]
mod criterion_killed_by_arrow;
#[cfg(test)]
mod criterion_killed_trigger;
#[cfg(test)]
mod criterion_levitation;
#[cfg(test)]
mod criterion_light_predicate;
#[cfg(test)]
mod criterion_lightning_bolt_predicate;
#[cfg(test)]
mod criterion_lightning_strike;
#[cfg(test)]
mod criterion_location_predicate;
#[cfg(test)]
mod criterion_loot_table_trigger;
#[cfg(test)]
mod criterion_min_max_bounds;
#[cfg(test)]
mod criterion_movement_predicate;
#[cfg(test)]
mod criterion_nbt_predicate;
#[cfg(test)]
mod criterion_picked_up_item_trigger;
#[cfg(test)]
mod criterion_player_hurt_entity;
#[cfg(test)]
mod criterion_player_interact;
#[cfg(test)]
mod criterion_player_predicate;
#[cfg(test)]
mod criterion_player_trigger;
#[cfg(test)]
mod criterion_raider_predicate;
#[cfg(test)]
mod criterion_recipe_crafted;
#[cfg(test)]
mod criterion_recipe_unlocked;
#[cfg(test)]
mod criterion_sheep_predicate;
#[cfg(test)]
mod criterion_shot_crossbow;
#[cfg(test)]
mod criterion_simple_trigger;
#[cfg(test)] mod criterion_single_component_item_predicate;
#[cfg(test)] mod criterion_slide_down_block;
#[cfg(test)] mod criterion_slime_predicate;
#[cfg(test)] mod criterion_block_predicate;
#[cfg(test)] mod criterion_bee_nest;
#[cfg(test)] mod criterion_block_interaction;
#[cfg(test)] mod criterion_slots_predicate;
#[cfg(test)] mod criterion_spear_mobs;
#[cfg(test)] mod criterion_start_riding;
#[cfg(test)] mod criterion_state_properties_predicate;
#[cfg(test)] mod criterion_summoned_entity;
#[cfg(test)] mod criterion_tag_predicate;
#[cfg(test)] mod criterion_tame_animal;
#[cfg(test)] mod criterion_target_block;
#[cfg(test)] mod criterion_trade_trigger;
#[cfg(test)] mod criterion_used_ender_eye;
#[cfg(test)] mod criterion_used_totem;
#[cfg(test)] mod criterion_using_item;
mod crash;
mod crash_recovery_tests;
#[cfg(test)] mod creative_inventory;
mod damage_type;
#[cfg(test)] mod data_advancement_packs;
#[cfg(test)] mod data_advancements;
#[cfg(test)] mod data_info;
#[cfg(test)] mod data_loot;
#[cfg(test)] mod data_loot_block;
#[cfg(test)] mod data_loot_chest;
#[cfg(test)] mod data_loot_entity;
#[cfg(test)] mod data_loot_gift;
#[cfg(test)] mod data_loot_packs;
#[cfg(test)] mod data_loot_piglin_barter;
#[cfg(test)] mod data_loot_provider;
#[cfg(test)] mod data_loot_shearing;
#[cfg(test)] mod data_loot_vanilla_entity;
#[cfg(test)] mod data_metadata;
#[cfg(test)] mod data_package;
#[cfg(test)] mod data_registries;
#[cfg(test)] mod data_recipe_unlock_advancement;
#[cfg(test)] mod data_recipes;
#[cfg(test)] mod data_recipes_package;
#[cfg(test)] mod data_shaped_recipe_builder;
#[cfg(test)] mod data_shapeless_recipe_builder;
#[cfg(test)] mod data_simple_cooking_recipe_builder;
#[cfg(test)] mod data_single_item_recipe_builder;
#[cfg(test)] mod data_smithing_transform_recipe_builder;
#[cfg(test)] mod data_smithing_trim_recipe_builder;
#[cfg(test)] mod data_special_recipe_builder;
#[cfg(test)] mod data_structures;
#[cfg(test)] mod data_tags_core;
#[cfg(test)] mod data_tags_entity_types;
#[cfg(test)] mod data_tags_medium_providers;
#[cfg(test)] mod data_tags_registry_providers;
#[cfg(test)] mod data_tags_trade_rebalance_trades;
#[cfg(test)] mod data_tags_vanilla_enchantments;
#[cfg(test)] mod data_transmute_recipe_builder;
#[cfg(test)] mod data_vanilla_recipe_provider;
#[cfg(test)] mod datapack_reload_tests;
#[cfg(test)] mod dialog_system;
mod dispenser_cauldron;
mod enchantment_system;
#[cfg(test)] mod entity_behavior_tests;
mod entity_category;
#[cfg(test)] mod entity_metadata;
mod entity_physics;
mod entity_validation;
#[cfg(test)] mod entity_variants;
mod environment_attributes;
#[cfg(test)] mod equipment_trim;
mod eula;
mod experience_system;
mod fire;
mod fluid;
#[cfg(test)] mod fuzz_tests;
mod game_event;
mod game_rules;
#[cfg(test)] mod gametest_resources;
mod generated_reports;
mod gravity;
mod inhabited_time;
mod identifier_exception;
mod inventory;
mod inventory_transactions;
mod item_catalog;
mod item_entity;
#[cfg(test)] mod item_family_behavior;
mod item_properties;
mod item_stack;
mod item_tags;
mod lighting;
mod living_entity;
mod localization_keys;
mod log;
mod loot_system;
mod management_security;
mod management_server;
mod map_state;
#[cfg(test)] mod mob_family;
mod mob_interaction;
#[cfg(test)]
mod movement_physics;
#[cfg(test)]
mod movement_validation;
mod network;
#[cfg(test)] mod non_living_entity;
#[cfg(test)]
mod operational_coverage;
mod optionull;
#[cfg(test)]
mod parity_harness;
mod performance_benchmarks;
#[cfg(test)]
mod persistence_roundtrip_tests;
mod plant;
mod player;
mod player_access;
mod player_entity;
mod player_game_mode;
mod player_inventory;
mod player_list;
mod player_online_auth;
#[cfg(test)]
mod player_presentation;
#[cfg(test)]
mod player_profile_key;
mod portal;
mod post_processing;
#[cfg(test)]
mod potion_fluid_container;
mod presentation_data;
mod project_foundation_tests;
#[cfg(test)]
mod projectile_entity;
mod raid;
mod random_source;
mod random_tick;
mod recipe_system;
mod redstone;
mod registry;
mod report_type;
mod resources;
mod respawn;
mod runtime;
mod scheduled_tick;
mod seed_validation;
mod server_properties;
mod spawning;
mod system_report;
mod special_block;
#[cfg(test)]
mod statistics;
mod status_effect;
mod storage;
mod structure_resources;
mod shared_constants;
mod suppress_forbidden;
mod tracing_executor;
mod trial_system;
#[cfg(test)]
mod vehicle_entity;
mod vibration;
mod villager_system;
mod villager_trade_resources;
#[cfg(test)]
mod waypoint;
mod weather;
mod world;
mod world_border;
mod world_time;
mod world_version;
mod worldgen;
mod worldgen_comparison;
mod worldgen_resources;

use std::env;
use std::path::{Path, PathBuf};
use std::process;

use cli::CliOptions;
use eula::Eula;
use log::{LogLevel, Logger};
use network::query::{spawn_query_server, QueryServerInfo};
use network::rcon::spawn_rcon_server;
use network::status::{read_code_of_conducts, run_status_server, ActiveLoginRegistry};
use resources::{
    configure_pack_repository, DataPackConfig, DataPackRepository, PackConfigureOptions,
    WorldDataConfiguration,
};
use server_properties::ServerProperties;
use storage::datafix::{run_world_upgrade, WorldUpgradeOptions};
use storage::world::{LevelVersion, WorldLayout};
use world::WorldOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeSelection {
    world_name: String,
    universe: PathBuf,
    port: u16,
    server_id: Option<String>,
}

struct StartupFiles {
    settings_path: PathBuf,
    eula_path: PathBuf,
    properties: ServerProperties,
    eula: Eula,
}

fn main() {
    crash::install_panic_hook();

    let options = match CliOptions::parse(env::args().skip(1)) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            eprintln!();
            eprintln!("{}", CliOptions::help());
            process::exit(2);
        }
    };

    if options.help {
        println!("{}", CliOptions::help());
        return;
    }

    if let Err(err) = run(options) {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run(options: CliOptions) -> Result<(), String> {
    let logger = initialize_logger(&options)?;
    logger.info("Starting RustCraft target server for Minecraft Java Edition 26.1.2")?;
    write_pid_file(&options)?;

    if generate_reports_if_requested(&options, &logger)? {
        return Ok(());
    }

    let startup = load_startup_files()?;
    if exit_after_startup_file_gate(&options, &logger, &startup)? {
        return Ok(());
    }

    validate_code_of_conduct_configuration(&startup.properties)?;

    let watchdog = runtime::Watchdog::from_max_tick_time_millis(startup.properties.max_tick_time);
    let runtime = runtime_selection(&options, &startup.properties);
    log_runtime_selection(&logger, &options, &runtime, &watchdog)?;
    run_configured_world_upgrade(&logger, &options, &runtime)?;
    check_world_version_compatibility(&logger, &runtime)?;

    // Acquire exclusive session lock to prevent concurrent world access.
    // Matches Java LevelStorageSource.LevelStorageAccess constructor.
    let world_dir = runtime.universe.join(&runtime.world_name);
    let _session_lock = storage::world::SessionLock::acquire(&world_dir)
        .map_err(|err| format!("Failed to acquire session lock on '{}': {err}", world_dir.display()))?;

    let (console_input, _console_handle) = console::spawn_console_input_thread()
        .map_err(|err| format!("Failed to start server console input thread: {err}"))?;
    logger.info("Started server console input thread")?;

    let world_options =
        configure_initial_data_packs(&logger, &options, &startup.properties, &runtime)?;
    start_network_listeners(
        &logger,
        &startup.properties,
        &runtime,
        world_options.seed,
        &console_input,
    )
}

fn initialize_logger(options: &CliOptions) -> Result<std::sync::Arc<Logger>, String> {
    let level = selected_log_level(options);
    Logger::open_with_level("logs", level).map(crate::log::init)
}

fn selected_log_level(options: &CliOptions) -> LogLevel {
    // Keep Java-main-style precedence: CLI flag, environment, then default.
    options
        .log_level
        .or_else(|| {
            std::env::var("RUSTCRAFT_LOG")
                .ok()
                .and_then(|val| LogLevel::from_str(&val).ok())
        })
        .unwrap_or(LogLevel::Info)
}

fn write_pid_file(options: &CliOptions) -> Result<(), String> {
    if let Some(pid_file) = &options.pid_file {
        let pid = process::id().to_string();
        std::fs::write(pid_file, pid)
            .map_err(|err| format!("Failed to write pid file '{}': {err}", pid_file.display()))?;
    }
    Ok(())
}

fn generate_reports_if_requested(options: &CliOptions, logger: &Logger) -> Result<bool, String> {
    if !options.report {
        return Ok(false);
    }

    generated_reports::generate_reports("generated")?;
    logger.info("Generated reports under generated/reports")?;
    Ok(true)
}

fn load_startup_files() -> Result<StartupFiles, String> {
    let settings_path = PathBuf::from("server.properties");
    let eula_path = PathBuf::from("eula.txt");
    let mut properties = ServerProperties::load_or_default(&settings_path)?;
    properties.save(&settings_path)?;
    let eula = Eula::load_or_create(&eula_path)?;

    Ok(StartupFiles {
        settings_path,
        eula_path,
        properties,
        eula,
    })
}

fn exit_after_startup_file_gate(
    options: &CliOptions,
    logger: &Logger,
    startup: &StartupFiles,
) -> Result<bool, String> {
    if options.init_settings {
        logger.info(&format!(
            "Initialized '{}' and '{}'",
            startup.settings_path.display(),
            startup.eula_path.display()
        ))?;
        return Ok(true);
    }

    if !startup.eula.agreed {
        logger.info("You need to agree to the EULA in order to run the server. Go to eula.txt for more info.")?;
        return Ok(true);
    }

    Ok(false)
}

fn log_runtime_selection(
    logger: &Logger,
    options: &CliOptions,
    runtime: &RuntimeSelection,
    watchdog: &runtime::Watchdog,
) -> Result<(), String> {
    logger.info(&format!("world={}", runtime.world_name))?;
    logger.info(&format!("universe={}", runtime.universe.display()))?;
    logger.info(&format!("port={}", runtime.port))?;
    logger.info(&format!("nogui={}", options.nogui))?;
    logger.info(&format!("safeMode={}", options.safe_mode))?;
    logger.info(&format!("demo={}", options.demo))?;
    logger.info(&format!("bonusChest={}", options.bonus_chest))?;
    logger.info(&format!(
        "serverId={}",
        runtime.server_id.as_deref().unwrap_or("")
    ))?;
    logger.info(&format!("maxTickTime={}", watchdog.max_tick_time_millis()))
}

fn run_configured_world_upgrade(
    logger: &Logger,
    options: &CliOptions,
    runtime: &RuntimeSelection,
) -> Result<(), String> {
    // Java Main: only --forceUpgrade or --recreateRegionFiles trigger the upgrade.
    // --eraseCache is a modifier passed into the upgrade, not a standalone trigger.
    if !(options.force_upgrade || options.recreate_region_files) {
        return Ok(());
    }

    let layout = WorldLayout::new(runtime.universe.join(&runtime.world_name));
    if !(layout.level_dat().is_file() || layout.level_dat_old().is_file()) {
        logger.info("Skipping world upgrade: no level.dat exists yet")?;
        return Ok(());
    }

    let tag = layout
        .load_level_dat_with_backup()
        .map_err(|err| format!("Failed to read level.dat for world upgrade: {err}"))?;
    let version = LevelVersion::parse_level_dat(&tag)
        .and_then(|version| version.data_version)
        .ok_or_else(|| "level.dat missing DataVersion for world upgrade".to_string())?;
    let report = run_world_upgrade(
        &layout,
        version,
        WorldUpgradeOptions {
            force_upgrade: options.force_upgrade,
            erase_cache: options.erase_cache,
            recreate_region_files: options.recreate_region_files,
        },
    )?;
    logger.info(&format!(
        "worldUpgradeSteps={:?}, chunks={}, entityChunks={}",
        report.plan.steps, report.chunk_count, report.entity_chunk_count
    ))
}

/// Refuse to start if the world's DataVersion is incompatible.
/// Matches Java `Main.java` line 144: `if (!summary.isCompatible())`.
///
/// Java checks `DataVersion.series` equality; for non-experimental worlds
/// this is always "main". We additionally check numeric version: a world
/// from a newer server (higher DataVersion) must not be loaded by an
/// older server.
fn check_world_version_compatibility(
    logger: &Logger,
    runtime: &RuntimeSelection,
) -> Result<(), String> {
    let layout = WorldLayout::new(runtime.universe.join(&runtime.world_name));
    if !layout.level_dat().is_file() && !layout.level_dat_old().is_file() {
        return Ok(());
    }

    let tag = match layout.load_level_dat_with_backup() {
        Ok(tag) => tag,
        Err(err) => {
            // Java Main.java line 133-135: corrupted level.dat → refuse startup.
            let _ = logger.info("Failed to load world data. World files may be corrupted. Shutting down.");
            return Err(format!(
                "Failed to read level.dat: {err}"
            ));
        }
    };

    let Some(version) = LevelVersion::parse_level_dat(&tag) else {
        // level.dat exists but has no parseable version info — treat as corrupted
        let _ = logger.info("Failed to load world data. World files may be corrupted. Shutting down.");
        return Err("level.dat exists but has no parseable version information".to_string());
    };

    if version.minecraft_version.series != "main" {
        let _ = logger.info("This world was created by an incompatible version.");
        return Err(format!(
            "World series '{}' is incompatible (expected 'main')",
            version.minecraft_version.series
        ));
    }

    if let Some(data_version) = version.data_version {
        storage::datafix::require_current_world_data_version(data_version)?;
    }

    Ok(())
}

fn configure_initial_data_packs(
    logger: &Logger,
    options: &CliOptions,
    properties: &ServerProperties,
    runtime: &RuntimeSelection,
) -> Result<WorldOptions, String> {
    let datapack_dir = runtime.universe.join(&runtime.world_name).join("datapacks");
    let mut pack_repository = DataPackRepository::server_repository(&datapack_dir)?;
    let initial_data_config = WorldDataConfiguration {
        data_packs: DataPackConfig::from_properties(
            &properties.initial_enabled_packs,
            &properties.initial_disabled_packs,
        ),
        enabled_features: registry::feature_flags::default_flags_26_1_2(),
    };
    let configured_data = configure_pack_repository(
        &mut pack_repository,
        &initial_data_config,
        PackConfigureOptions {
            init_mode: false,
            safe_mode: options.safe_mode,
        },
    );
    logger.info(&format!(
        "enabledDataPacks={}",
        configured_data.data_packs.enabled.join(",")
    ))?;
    logger.info(&format!(
        "disabledDataPacks={}",
        configured_data.data_packs.disabled.join(",")
    ))?;
    let world_options = WorldOptions::from_server_inputs(
        &properties.level_seed,
        properties.generate_structures,
        options.bonus_chest,
        options.demo,
    );
    logger.info(&format!("worldSeed={}", world_options.seed))?;
    logger.info(&format!(
        "generateStructures={}",
        world_options.generate_structures
    ))?;
    logger.info(&format!(
        "generateBonusChest={}",
        world_options.generate_bonus_chest
    ))?;
    Ok(world_options)
}

fn start_network_listeners(
    logger: &Logger,
    properties: &ServerProperties,
    runtime: &RuntimeSelection,
    world_seed: i64,
    console_input: &std::sync::mpsc::Receiver<console::ConsoleInput>,
) -> Result<(), String> {
    logger.info("Starting status/login listener with minimal play join support.")?;

    let bind_ip = listener_bind_ip(properties);
    // Shared registry of online players, populated by the status/login listener as
    // players reach the PLAY state. Both the status and GS4 query listeners read it
    // so they report the same live player count and names (Java `MinecraftServer`
    // owns the single `PlayerList` both protocols consult).
    let active_logins = ActiveLoginRegistry::default();
    if properties.enable_query {
        let query_info = QueryServerInfo {
            server_name: properties.motd.clone(),
            world_name: runtime.world_name.clone(),
            server_version: "26.1.2".to_string(),
            plugin_names: String::new(),
            host_ip: bind_ip.to_string(),
            server_port: runtime.port,
            player_count: 0,
            max_players: properties.max_players as usize,
            player_names: Vec::new(),
        };
        spawn_query_server(
            bind_ip,
            properties.query_port,
            query_info,
            Some(active_logins.clone()),
        )?;
        logger.info(&format!(
            "Query listener binding to {bind_ip}:{}",
            properties.query_port
        ))?;
    }
    if properties.enable_rcon {
        spawn_rcon_server(
            bind_ip,
            properties.rcon_port,
            properties.rcon_password.clone(),
            properties.broadcast_rcon_to_ops,
            |command| {
                format!("Unknown or incomplete command, see below for error\n{command}<--[HERE]")
            },
        )?;
        logger.info(&format!(
            "RCON listener binding to {bind_ip}:{}",
            properties.rcon_port
        ))?;
    }
    logger.info(&format!(
        "Status listener binding to {bind_ip}:{}",
        runtime.port
    ))?;
    run_status_server(
        bind_ip,
        runtime.port,
        properties,
        &runtime.universe.join(&runtime.world_name),
        world_seed,
        console_input,
        active_logins,
    )
}

fn runtime_selection(options: &CliOptions, properties: &ServerProperties) -> RuntimeSelection {
    RuntimeSelection {
        world_name: options
            .world
            .clone()
            .unwrap_or_else(|| properties.level_name.clone()),
        universe: options.universe.clone(),
        port: if options.port >= 0 {
            options.port as u16
        } else {
            properties.server_port
        },
        server_id: options.server_id.clone(),
    }
}

fn listener_bind_ip(properties: &ServerProperties) -> &str {
    if properties.server_ip.is_empty() {
        "0.0.0.0"
    } else {
        properties.server_ip.as_str()
    }
}

fn validate_code_of_conduct_configuration(properties: &ServerProperties) -> Result<(), String> {
    if !properties.code_of_conduct {
        return Ok(());
    }
    read_code_of_conducts(Path::new("codeofconduct"))
        .map(|_| ())
        .map_err(|err| format!("Failed to read Code of Conduct files: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{
        listener_bind_ip, run, runtime_selection, validate_code_of_conduct_configuration,
        CliOptions,
    };
    use crate::server_properties::ServerProperties;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    static CWD_LOCK: Mutex<()> = Mutex::new(());

    struct CurrentDirGuard {
        old: PathBuf,
    }

    impl CurrentDirGuard {
        fn enter(path: &Path) -> Self {
            let old = std::env::current_dir().expect("current dir");
            std::env::set_current_dir(path).expect("set temp current dir");
            Self { old }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.old);
        }
    }

    fn temp_workdir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-main-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp workdir");
        dir
    }

    #[test]
    fn init_settings_creates_properties_and_eula_before_startup() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("init-settings");
        let _guard = CurrentDirGuard::enter(&dir);

        let options = CliOptions {
            init_settings: true,
            ..CliOptions::default()
        };

        run(options).expect("init settings run");

        let properties = fs::read_to_string("server.properties").expect("server.properties");
        let eula = fs::read_to_string("eula.txt").expect("eula.txt");
        let log = fs::read_to_string("logs/latest.log").expect("latest.log");

        assert!(properties.contains("server-port="));
        assert!(eula.contains("eula=false"));
        assert!(log.contains("Initialized 'server.properties' and 'eula.txt'"));
    }

    #[test]
    fn missing_eula_refuses_startup_after_generating_files() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("eula-refusal");
        let _guard = CurrentDirGuard::enter(&dir);

        run(CliOptions::default()).expect("eula refusal run");

        let log = fs::read_to_string("logs/latest.log").expect("latest.log");
        assert!(Path::new("server.properties").is_file());
        assert!(Path::new("eula.txt").is_file());
        assert!(log.contains("You need to agree to the EULA in order to run the server"));
        assert!(!Path::new("world").exists());
    }

    #[test]
    fn pid_file_is_written_before_eula_refusal() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("pid-file");
        let _guard = CurrentDirGuard::enter(&dir);

        let options = CliOptions {
            pid_file: Some(PathBuf::from("server.pid")),
            ..CliOptions::default()
        };

        run(options).expect("pid-file run");

        let pid = fs::read_to_string("server.pid").expect("server.pid");
        assert_eq!(pid, std::process::id().to_string());
        assert!(Path::new("server.properties").is_file());
        assert!(Path::new("eula.txt").is_file());
        assert!(!Path::new("world").exists());
    }

    #[test]
    fn runtime_selection_uses_java_main_world_universe_port_and_server_id_options() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("runtime-selection");
        let _guard = CurrentDirGuard::enter(&dir);

        fs::write(
            "server.properties",
            "level-name=from_properties\nserver-port=25570\n",
        )
        .expect("write server.properties");
        let properties = ServerProperties::load_or_default(Path::new("server.properties")).unwrap();

        let defaulted = runtime_selection(&CliOptions::default(), &properties);
        assert_eq!(defaulted.world_name, "from_properties");
        assert_eq!(defaulted.universe, PathBuf::from("."));
        assert_eq!(defaulted.port, 25570);
        assert_eq!(defaulted.server_id, None);

        let options = CliOptions {
            world: Some("from_cli".to_string()),
            universe: PathBuf::from("worlds"),
            port: 25566,
            server_id: Some("server-123".to_string()),
            ..CliOptions::default()
        };

        let selected = runtime_selection(&options, &properties);
        assert_eq!(selected.world_name, "from_cli");
        assert_eq!(selected.universe, PathBuf::from("worlds"));
        assert_eq!(selected.port, 25566);
        assert_eq!(selected.server_id.as_deref(), Some("server-123"));
    }

    #[test]
    fn listener_bind_ip_uses_vanilla_server_ip_property_with_wildcard_default() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("server-ip");
        let _guard = CurrentDirGuard::enter(&dir);

        fs::write("server.properties", "").expect("write default server.properties");
        let defaulted = ServerProperties::load_or_default(Path::new("server.properties")).unwrap();
        assert_eq!(listener_bind_ip(&defaulted), "0.0.0.0");

        fs::write("server.properties", "server-ip=127.0.0.1\n")
            .expect("write explicit server.properties");
        let explicit = ServerProperties::load_or_default(Path::new("server.properties")).unwrap();
        assert_eq!(listener_bind_ip(&explicit), "127.0.0.1");
    }

    #[test]
    fn enable_code_of_conduct_requires_java_codeofconduct_directory() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("code-of-conduct");
        let _guard = CurrentDirGuard::enter(&dir);

        fs::write("server.properties", "enable-code-of-conduct=true\n")
            .expect("write server.properties");
        let properties = ServerProperties::load_or_default(Path::new("server.properties")).unwrap();
        let missing = validate_code_of_conduct_configuration(&properties).unwrap_err();
        assert!(missing.contains("Failed to read Code of Conduct files"));

        fs::create_dir("codeofconduct").expect("create codeofconduct");
        fs::write("codeofconduct/en_us.txt", "Rules").expect("write code of conduct");
        validate_code_of_conduct_configuration(&properties).expect("valid code of conduct");
    }

    #[test]
    fn report_flag_generates_reports_and_exits_before_eula_gate() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("report");
        let _guard = CurrentDirGuard::enter(&dir);

        let options = CliOptions {
            report: true,
            ..CliOptions::default()
        };

        run(options).expect("report run");

        assert!(Path::new("generated/reports/registries.json").is_file());
        assert!(Path::new("generated/reports/commands.json").is_file());
        assert!(Path::new("generated/reports/biomes.json").is_file());
        assert!(Path::new("generated/reports/blocks.json").is_file());
        assert!(Path::new("generated/reports/items.json").is_file());
        assert!(Path::new("generated/reports/worldgen_chunks.json").is_file());
        assert!(Path::new("generated/data/minecraft/tags/block/mineable.json").is_file());
        assert!(!Path::new("eula.txt").exists());
    }

    #[test]
    fn incompatible_world_series_refuses_startup() {
        use crate::log::{LogLevel, Logger};
        use crate::storage::nbt::Tag;
        use crate::storage::world::WorldLayout;

        let dir = temp_workdir("incompatible_series");
        let world = dir.join("world");
        let layout = WorldLayout::new(&world);
        fs::create_dir_all(layout.root()).expect("mkdir");
        let log_dir = dir.join("logs");
        fs::create_dir_all(&log_dir).expect("logs dir");
        let logger = Logger::open_with_level(&log_dir, LogLevel::Info).expect("logger");

        let data_version = crate::storage::datafix::TARGET_DATA_VERSION;
        let mut level_dat = Vec::new();
        crate::storage::nbt::write_named_tag(
            &mut level_dat,
            "",
            &Tag::Compound(vec![(
                "Data".to_string(),
                Tag::Compound(vec![
                    ("DataVersion".to_string(), Tag::Int(data_version)),
                    (
                        "Version".to_string(),
                        Tag::Compound(vec![
                            ("Id".to_string(), Tag::Int(data_version)),
                            ("Name".to_string(), Tag::String("26.1.2".to_string())),
                            (
                                "Series".to_string(),
                                Tag::String("experimental_snapshot".to_string()),
                            ),
                            ("Snapshot".to_string(), Tag::Byte(0)),
                        ]),
                    ),
                ]),
            )]),
        )
        .expect("write nbt");
        fs::write(layout.level_dat(), &level_dat).expect("write level.dat");

        let runtime = super::RuntimeSelection {
            world_name: "world".to_string(),
            universe: dir.clone(),
            port: 25565,
            server_id: None,
        };
        let result = super::check_world_version_compatibility(&logger, &runtime);
        assert!(result.is_err(), "should refuse incompatible series");
        assert!(
            result.unwrap_err().contains("experimental_snapshot"),
            "error should name the incompatible series"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupted_world_metadata_refuses_startup() {
        use crate::log::{LogLevel, Logger};
        use crate::storage::world::WorldLayout;

        let dir = temp_workdir("corrupted_metadata");
        let world = dir.join("world");
        let layout = WorldLayout::new(&world);
        fs::create_dir_all(layout.root()).expect("mkdir");
        let log_dir = dir.join("logs");
        fs::create_dir_all(&log_dir).expect("logs dir");
        let logger = Logger::open_with_level(&log_dir, LogLevel::Info).expect("logger");

        // Write garbage bytes as level.dat
        fs::write(layout.level_dat(), b"not valid nbt data").expect("write corrupt level.dat");

        let runtime = super::RuntimeSelection {
            world_name: "world".to_string(),
            universe: dir.clone(),
            port: 25565,
            server_id: None,
        };
        let result = super::check_world_version_compatibility(&logger, &runtime);
        assert!(result.is_err(), "should refuse corrupted level.dat");
        let _ = fs::remove_dir_all(&dir);
    }
}
