mod ai_system;
mod attribute_system;
mod base_entity;
mod biome;
mod block_behavior;
mod block_catalog;
mod block_entity;
mod block_metadata;
mod block_regression;
mod block_update;
mod boss_fight;
mod chat_trust;
mod chunk_manager;
mod chunk_task;
mod chunk_ticket;
mod chunk_watchdog;
mod cli;
mod collision_shape;
mod combat_damage;
mod command;
mod command_tree;
mod console;
mod container_block;
mod crash;
mod creative_inventory;
mod damage_type;
mod dispenser_cauldron;
mod enchantment_system;
mod entity_category;
mod entity_physics;
mod entity_validation;
mod entity_variants;
mod equipment_trim;
mod eula;
mod fire;
mod fluid;
mod gravity;
mod inhabited_time;
mod inventory;
mod item_catalog;
mod item_family_behavior;
mod item_properties;
mod item_stack;
mod light;
mod living_entity;
mod log;
mod management_security;
mod mob_family;
mod mob_interaction;
mod movement_physics;
mod movement_validation;
mod network;
mod non_living_entity;
mod plant;
mod player;
mod player_access;
mod player_entity;
mod player_inventory;
mod player_online_auth;
mod player_profile_key;
mod portal;
mod post_processing;
mod potion_fluid_container;
mod projectile_entity;
mod random_source;
mod random_tick;
mod recipe_system;
mod redstone;
mod registry;
mod resources;
mod respawn;
mod runtime;
mod scheduled_tick;
mod seed_validation;
mod server_properties;
mod spawning;
mod special_block;
mod status_effect;
mod storage;
mod vehicle_entity;
mod world;
mod world_border;
mod worldgen;

use std::env;
use std::path::PathBuf;
use std::process;

use cli::CliOptions;
use eula::Eula;
use log::Logger;
use network::query::{spawn_query_server, QueryServerInfo};
use network::rcon::spawn_rcon_server;
use network::status::run_status_server;
use resources::{
    configure_pack_repository, DataPackConfig, DataPackRepository, PackConfigureOptions,
    WorldDataConfiguration,
};
use server_properties::ServerProperties;
use world::WorldOptions;

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
    let logger = Logger::open("logs")?;
    logger.info("Starting RustCraft target server for Minecraft Java Edition 26.1.2")?;

    if let Some(pid_file) = &options.pid_file {
        let pid = process::id().to_string();
        std::fs::write(pid_file, pid)
            .map_err(|err| format!("Failed to write pid file '{}': {err}", pid_file.display()))?;
    }

    let settings_path = PathBuf::from("server.properties");
    let eula_path = PathBuf::from("eula.txt");

    let mut properties = ServerProperties::load_or_default(&settings_path)?;
    properties.save(&settings_path)?;

    let eula = Eula::load_or_create(&eula_path)?;

    if options.init_settings {
        logger.info(&format!(
            "Initialized '{}' and '{}'",
            settings_path.display(),
            eula_path.display()
        ))?;
        return Ok(());
    }

    if !eula.agreed {
        logger.info("You need to agree to the EULA in order to run the server. Go to eula.txt for more info.")?;
        return Ok(());
    }

    let world_name = options
        .world
        .clone()
        .unwrap_or_else(|| properties.level_name.clone());
    let universe = options.universe.clone();
    let port = if options.port >= 0 {
        options.port as u16
    } else {
        properties.server_port
    };

    logger.info(&format!("world={world_name}"))?;
    logger.info(&format!("universe={}", universe.display()))?;
    logger.info(&format!("port={port}"))?;
    logger.info(&format!("nogui={}", options.nogui))?;
    logger.info(&format!("safeMode={}", options.safe_mode))?;
    logger.info(&format!("demo={}", options.demo))?;
    logger.info(&format!("bonusChest={}", options.bonus_chest))?;
    logger.info(&format!(
        "serverId={}",
        options.server_id.as_deref().unwrap_or("")
    ))?;
    let (_console_input, _console_handle) = console::spawn_console_input_thread();
    logger.info("Started server console input thread")?;

    let datapack_dir = universe.join(&world_name).join("datapacks");
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

    logger.info("Starting status/ping listener. Login and gameplay are not implemented yet.")?;

    let bind_ip = if properties.server_ip.is_empty() {
        "0.0.0.0"
    } else {
        properties.server_ip.as_str()
    };
    if properties.enable_query {
        let query_info = QueryServerInfo {
            server_name: properties.motd.clone(),
            world_name: world_name.clone(),
            server_version: "26.1.2".to_string(),
            plugin_names: String::new(),
            host_ip: bind_ip.to_string(),
            server_port: port,
            player_count: 0,
            max_players: properties.max_players as usize,
            player_names: Vec::new(),
        };
        spawn_query_server(bind_ip, properties.query_port, query_info)?;
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
    logger.info(&format!("Status listener binding to {bind_ip}:{port}"))?;
    run_status_server(bind_ip, port, &properties)?;

    Ok(())
}
