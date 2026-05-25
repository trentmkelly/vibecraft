#[cfg(test)]
mod advancement_system;
mod ai_system;
mod attribute_system;
mod base_entity;
mod biome;
mod block_behavior;
#[cfg(test)]
mod block_behavior_tests;
mod block_catalog;
mod block_entity;
mod block_metadata;
#[cfg(test)]
mod block_regression;
mod block_update;
mod boss_fight;
mod chat_component;
mod chat_trust;
mod chunk_manager;
mod chunk_task;
mod chunk_ticket;
mod chunk_watchdog;
mod cli;
mod collision_shape;
mod combat_damage;
mod command;
mod command_execution;
mod command_feedback;
mod command_parity;
#[cfg(test)]
mod command_selector;
mod command_tree;
mod console;
mod container_block;
mod container_menus;
mod crash;
mod crash_recovery_tests;
#[cfg(test)]
mod creative_inventory;
mod damage_type;
#[cfg(test)]
mod datapack_reload_tests;
#[cfg(test)]
mod dialog_system;
mod dispenser_cauldron;
mod enchantment_system;
#[cfg(test)]
mod entity_behavior_tests;
mod entity_category;
#[cfg(test)]
mod entity_metadata;
mod entity_physics;
mod entity_validation;
#[cfg(test)]
mod entity_variants;
mod environment_attributes;
#[cfg(test)]
mod equipment_trim;
mod eula;
mod experience_system;
mod fire;
mod fluid;
#[cfg(test)]
mod fuzz_tests;
mod game_event;
mod game_rules;
#[cfg(test)]
mod gametest_resources;
mod generated_reports;
mod gravity;
mod inhabited_time;
mod inventory;
mod inventory_transactions;
mod item_catalog;
mod item_entity;
#[cfg(test)]
mod item_family_behavior;
mod item_properties;
mod item_stack;
mod light;
mod living_entity;
mod localization_keys;
mod log;
mod loot_system;
mod management_security;
mod management_server;
mod map_state;
#[cfg(test)]
mod mob_family;
mod mob_interaction;
mod movement_physics;
mod movement_validation;
mod network;
mod non_living_entity;
#[cfg(test)]
mod operational_coverage;
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
mod projectile_entity;
mod raid;
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
#[cfg(test)]
mod statistics;
mod status_effect;
mod storage;
mod structure_resources;
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
use network::status::{read_code_of_conducts, run_status_server};
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
    // Determine the log level using the documented precedence:
    //   1. --log-level CLI flag
    //   2. RUSTCRAFT_LOG environment variable
    //   3. Default: Info
    let level = options
        .log_level
        .or_else(|| {
            std::env::var("RUSTCRAFT_LOG")
                .ok()
                .and_then(|val| LogLevel::from_str(&val).ok())
        })
        .unwrap_or(LogLevel::Info);

    let logger = crate::log::init(Logger::open_with_level("logs", level)?);
    logger.info("Starting RustCraft target server for Minecraft Java Edition 26.1.2")?;

    if let Some(pid_file) = &options.pid_file {
        let pid = process::id().to_string();
        std::fs::write(pid_file, pid)
            .map_err(|err| format!("Failed to write pid file '{}': {err}", pid_file.display()))?;
    }

    let settings_path = PathBuf::from("server.properties");
    let eula_path = PathBuf::from("eula.txt");

    if options.report {
        generated_reports::generate_reports("generated")?;
        logger.info("Generated reports under generated/reports")?;
        return Ok(());
    }

    let mut properties = ServerProperties::load_or_default(&settings_path)?;
    properties.save(&settings_path)?;
    let watchdog = runtime::Watchdog::from_max_tick_time_millis(properties.max_tick_time);

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

    validate_code_of_conduct_configuration(&properties)?;

    let runtime = runtime_selection(&options, &properties);
    let world_name = runtime.world_name.clone();
    let universe = runtime.universe.clone();
    let port = runtime.port;

    logger.info(&format!("world={world_name}"))?;
    logger.info(&format!("universe={}", universe.display()))?;
    logger.info(&format!("port={port}"))?;
    logger.info(&format!("nogui={}", options.nogui))?;
    logger.info(&format!("safeMode={}", options.safe_mode))?;
    logger.info(&format!("demo={}", options.demo))?;
    logger.info(&format!("bonusChest={}", options.bonus_chest))?;
    logger.info(&format!(
        "serverId={}",
        runtime.server_id.as_deref().unwrap_or("")
    ))?;
    logger.info(&format!("maxTickTime={}", watchdog.max_tick_time_millis()))?;

    if options.force_upgrade || options.erase_cache || options.recreate_region_files {
        let layout = WorldLayout::new(universe.join(&world_name));
        if layout.level_dat().is_file() || layout.level_dat_old().is_file() {
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
            ))?;
        } else {
            logger.info("Skipping world upgrade: no level.dat exists yet")?;
        }
    }

    let (console_input, _console_handle) = console::spawn_console_input_thread();
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

    logger.info("Starting status/login listener with minimal play join support.")?;

    let bind_ip = listener_bind_ip(&properties);
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
    run_status_server(
        bind_ip,
        port,
        &properties,
        &universe.join(&world_name),
        world_options.seed,
        &console_input,
    )?;

    Ok(())
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
        listener_bind_ip, run, runtime_selection, validate_code_of_conduct_configuration, CliOptions,
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
        let properties =
            ServerProperties::load_or_default(Path::new("server.properties")).unwrap();

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
}
