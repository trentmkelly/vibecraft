#[cfg(vibecraft_has_decompiled_sources)]
#[allow(unused_macros)]
macro_rules! vibecraft_java_source {
    ($path:literal) => {
        include_str!(concat!(env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT"), $path))
    };
}

#[cfg(not(vibecraft_has_decompiled_sources))]
#[allow(unused_macros)]
macro_rules! vibecraft_java_source {
    ($path:literal) => {
        ""
    };
}

include!("module_declarations.rs");

use std::env;
use std::path::{Path, PathBuf};
use std::process;

use cli::CliOptions;
use eula::Eula;
use log::{LogLevel, Logger};
use management_security::TLS_PASSWORD_ENV;
use management_server::{startup_plan, ManagementServerConfig, ManagementStartupPlan};
use network::query::{spawn_query_server, QueryServerInfo};
use network::rcon::spawn_rcon_server;
use network::status::{read_code_of_conducts, run_status_server, ActiveLoginRegistry};
use resources::{
    configure_pack_repository, DataPackRepository, PackConfigureOptions, WorldDataConfiguration,
};
use server_properties::ServerProperties;
use storage::datafix::{run_world_upgrade, WorldUpgradeOptions};
use storage::nbt::Tag;
use storage::world::{LevelVersion, PrimaryLevelData, WorldLayout};
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
    logger.info("Starting VibeCraft target server for Minecraft Java Edition 26.1.2")?;
    write_pid_file(&options)?;

    if generate_reports_if_requested(&options, &logger)? {
        return Ok(());
    }

    let startup = load_startup_files()?;
    if exit_after_startup_file_gate(&options, &logger, &startup)? {
        return Ok(());
    }

    validate_code_of_conduct_configuration(&startup.properties)?;
    validate_management_server_configuration(&logger, &startup.properties)?;

    let watchdog = runtime::Watchdog::from_max_tick_time_millis(startup.properties.max_tick_time);
    let runtime = runtime_selection(&options, &startup.properties);
    log_runtime_selection(&logger, &options, &runtime, &watchdog)?;
    run_configured_world_upgrade(&logger, &options, &runtime)?;
    check_world_version_compatibility(&logger, &runtime)?;
    migrate_legacy_announce_player_achievements(&startup.properties, &runtime)?;

    // Acquire exclusive session lock to prevent concurrent world access.
    // Matches Java LevelStorageSource.LevelStorageAccess constructor.
    let world_dir = runtime.universe.join(&runtime.world_name);
    let _session_lock = storage::world::SessionLock::acquire(&world_dir).map_err(|err| {
        format!(
            "Failed to acquire session lock on '{}': {err}",
            world_dir.display()
        )
    })?;

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
            std::env::var("VIBECRAFT_LOG")
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

    if !startup.eula.has_agreed_to_eula() {
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
            let _ = logger
                .info("Failed to load world data. World files may be corrupted. Shutting down.");
            return Err(format!("Failed to read level.dat: {err}"));
        }
    };

    let Some(version) = LevelVersion::parse_level_dat(&tag) else {
        // level.dat exists but has no parseable version info — treat as corrupted
        let _ =
            logger.info("Failed to load world data. World files may be corrupted. Shutting down.");
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

fn migrate_legacy_announce_player_achievements(
    properties: &ServerProperties,
    runtime: &RuntimeSelection,
) -> Result<(), String> {
    let Some(enabled) = properties.announce_player_achievements else {
        return Ok(());
    };

    let layout = WorldLayout::new(runtime.universe.join(&runtime.world_name));
    if !layout.level_dat().is_file() && !layout.level_dat_old().is_file() {
        return Ok(());
    }

    let tag = layout
        .load_level_dat_with_backup()
        .map_err(|err| format!("Failed to read level.dat for legacy gamerule migration: {err}"))?;
    let mut level = PrimaryLevelData::from_level_dat(&tag)
        .ok_or_else(|| "level.dat exists but has no parseable level data".to_string())?;
    upsert_game_rule_string(
        &mut level.game_rules,
        "show_advancement_messages",
        enabled.to_string(),
    );
    layout
        .save_level_dat(&level.to_level_dat()?)
        .map_err(|err| format!("Failed to write level.dat for legacy gamerule migration: {err}"))
}

fn upsert_game_rule_string(game_rules: &mut Tag, name: &str, value: String) {
    let Tag::Compound(entries) = game_rules else {
        *game_rules = Tag::Compound(vec![(name.to_string(), Tag::String(value))]);
        return;
    };
    if let Some((_, existing)) = entries
        .iter_mut()
        .find(|(entry_name, _)| entry_name == name)
    {
        *existing = Tag::String(value);
    } else {
        entries.push((name.to_string(), Tag::String(value)));
    }
}

/// Java Main chooses stored configuration for existing worlds and properties
/// only for new worlds. PackRepository also needs the corresponding init flag.
fn load_initial_data_configuration(
    layout: &WorldLayout,
    properties: &ServerProperties,
) -> Result<(WorldDataConfiguration, bool), String> {
    if layout.level_dat().is_file() || layout.level_dat_old().is_file() {
        let tag = layout
            .load_level_dat_with_backup()
            .map_err(|error| error.to_string())?;
        let registry = registry::FeatureFlagRegistry::main_26_1_2()?;
        let data = match &tag {
            Tag::Compound(root) => root
                .iter()
                .find(|(key, _)| key == "Data")
                .map(|(_, value)| value),
            _ => None,
        };
        let config = data
            .and_then(|tag| WorldDataConfiguration::from_nbt(tag, &registry).ok())
            .unwrap_or_else(WorldDataConfiguration::default_26_1_2);
        return Ok((config, false));
    }
    Ok((
        WorldDataConfiguration {
            data_packs: properties.initial_data_pack_configuration(),
            enabled_features: registry::feature_flags::default_flags_26_1_2(),
        },
        true,
    ))
}

fn configure_initial_data_packs(
    logger: &Logger,
    options: &CliOptions,
    properties: &ServerProperties,
    runtime: &RuntimeSelection,
) -> Result<WorldOptions, String> {
    let datapack_dir = runtime.universe.join(&runtime.world_name).join("datapacks");
    let mut pack_repository = DataPackRepository::server_repository(&datapack_dir)?;
    let layout = WorldLayout::new(runtime.universe.join(&runtime.world_name));
    let (initial_data_config, init_mode) = load_initial_data_configuration(&layout, properties)?;
    let configured_data = configure_pack_repository(
        &mut pack_repository,
        &initial_data_config,
        PackConfigureOptions {
            init_mode,
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

fn validate_management_server_configuration(
    logger: &Logger,
    properties: &ServerProperties,
) -> Result<ManagementStartupPlan, String> {
    let config = ManagementServerConfig::from_properties(properties);
    let env_password = env::var(TLS_PASSWORD_ENV).ok();
    let plan = startup_plan(&config, env_password.as_deref(), None);
    match &plan {
        ManagementStartupPlan::Disabled => {
            logger.info("Management JSON-RPC server disabled")?;
        }
        ManagementStartupPlan::Listen {
            host,
            port,
            tls,
            allowed_origins,
        } => {
            logger.info(&format!(
                "Starting json RPC server on {host}:{port} ({}, allowed_origins={allowed_origins:?})",
                if tls.is_some() { "tls" } else { "plain" }
            ))?;
        }
        ManagementStartupPlan::Refused(decision) => {
            return Err(format!(
                "Failed to configure management server: {decision:?}"
            ));
        }
    }
    Ok(plan)
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
        listener_bind_ip, migrate_legacy_announce_player_achievements, run, runtime_selection,
        validate_code_of_conduct_configuration, validate_management_server_configuration,
        CliOptions, LogLevel, Logger, ManagementStartupPlan, RuntimeSelection,
    };
    use crate::management_server::AllowedOrigins;
    use crate::server_properties::ServerProperties;
    use crate::storage::nbt::Tag;
    use crate::storage::world::{
        DataPackSelection, LevelDifficulty, LevelGameType, LevelSpawnData, LevelVersionInfo,
        PrimaryLevelData, WorldLayout,
    };
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
        dir.push(format!("vibecraft-main-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp workdir");
        dir
    }

    fn minimal_level_data(game_rules: Tag) -> PrimaryLevelData {
        PrimaryLevelData {
            data_version: crate::storage::datafix::TARGET_DATA_VERSION,
            level_data_version: 19133,
            version: LevelVersionInfo {
                id: crate::storage::datafix::TARGET_DATA_VERSION,
                name: "26.1.2".to_string(),
                series: "main".to_string(),
                snapshot: false,
            },
            level_name: "world".to_string(),
            spawn: LevelSpawnData {
                x: 0,
                y: 64,
                z: 0,
                yaw: 0.0,
                ..Default::default()
            },
            game_type: LevelGameType::Survival,
            difficulty_settings: crate::storage::world::DifficultySettings {
                difficulty: LevelDifficulty::Easy,
                hardcore: false,
                locked: true,
            },
            day_time: 0,
            time: 0,
            generator_name: "default".to_string(),
            generator_settings: Tag::Compound(vec![]),
            allow_commands: false,
            initialized: true,
            was_modded: false,
            data_configuration: crate::resources::WorldDataConfiguration {
                data_packs: DataPackSelection::new(
                    vec!["vanilla".to_string()],
                    Vec::<String>::new(),
                ),
                enabled_features: crate::registry::feature_flags::default_flags_26_1_2(),
            },
            scheduled_events: Tag::List(vec![]),
            server_brands: Vec::new(),
            custom_boss_events: Tag::Compound(vec![]),
            dragon_fight: Tag::Compound(vec![]),
            scoreboard: Tag::Compound(vec![]),
            game_rules,
        }
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
    fn management_server_startup_settings_match_java_fail_fast_gates() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("management-startup");
        let _guard = CurrentDirGuard::enter(&dir);
        fs::create_dir_all("logs").expect("logs dir");
        let logger = Logger::open_with_level(Path::new("logs"), LogLevel::Info).expect("logger");

        fs::write("server.properties", "management-server-enabled=false\n")
            .expect("write disabled properties");
        let disabled = ServerProperties::load_or_default(Path::new("server.properties")).unwrap();
        assert_eq!(
            validate_management_server_configuration(&logger, &disabled).unwrap(),
            ManagementStartupPlan::Disabled
        );

        fs::write(
            "server.properties",
            "management-server-enabled=true\n\
management-server-host=127.0.0.1\n\
management-server-port=24454\n\
management-server-secret=0123456789abcdefghijklmnopqrstuvwxyzABCD\n\
management-server-tls-enabled=false\n\
management-server-allowed-origins=https://admin.example\n",
        )
        .expect("write plain properties");
        let plain = ServerProperties::load_or_default(Path::new("server.properties")).unwrap();
        assert_eq!(
            validate_management_server_configuration(&logger, &plain).unwrap(),
            ManagementStartupPlan::Listen {
                host: "127.0.0.1".to_string(),
                port: 24454,
                tls: None,
                allowed_origins: AllowedOrigins::parse("https://admin.example"),
            }
        );

        fs::write(
            "server.properties",
            "management-server-enabled=true\nmanagement-server-secret=bad\n",
        )
        .expect("write invalid properties");
        let invalid = ServerProperties::load_or_default(Path::new("server.properties")).unwrap();
        let err = validate_management_server_configuration(&logger, &invalid).unwrap_err();
        assert!(err.contains("DisabledInvalidSecret"));
    }

    #[test]
    fn legacy_announce_player_achievements_updates_persisted_gamerule_like_java() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("legacy-announce");
        let world = dir.join("world");
        let layout = WorldLayout::new(&world);
        let data = minimal_level_data(Tag::Compound(vec![(
            "show_advancement_messages".to_string(),
            Tag::String("true".to_string()),
        )]));
        layout
            .save_level_dat(&data.to_level_dat().unwrap())
            .expect("save level.dat");
        let properties_path = dir.join("server.properties");
        fs::write(&properties_path, "announce-player-achievements=false\n")
            .expect("write properties");
        let properties = ServerProperties::load_or_default(&properties_path).unwrap();
        let runtime = RuntimeSelection {
            world_name: "world".to_string(),
            universe: dir.clone(),
            port: 25565,
            server_id: None,
        };

        migrate_legacy_announce_player_achievements(&properties, &runtime).unwrap();

        let migrated = PrimaryLevelData::from_level_dat(&layout.load_level_dat().unwrap()).unwrap();
        let Tag::Compound(rules) = migrated.game_rules else {
            panic!("expected GameRules compound");
        };
        assert_eq!(
            rules
                .iter()
                .find(|(name, _)| name == "show_advancement_messages")
                .map(|(_, value)| value),
            Some(&Tag::String("false".to_string()))
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn absent_legacy_announce_player_achievements_leaves_level_dat_unchanged() {
        let _lock = CWD_LOCK.lock().unwrap();
        let dir = temp_workdir("legacy-announce-absent");
        let world = dir.join("world");
        let layout = WorldLayout::new(&world);
        let data = minimal_level_data(Tag::Compound(vec![(
            "show_advancement_messages".to_string(),
            Tag::String("true".to_string()),
        )]));
        layout
            .save_level_dat(&data.to_level_dat().unwrap())
            .expect("save level.dat");
        let before = layout.load_level_dat().unwrap();
        let properties_path = dir.join("server.properties");
        fs::write(&properties_path, "").expect("write properties");
        let properties = ServerProperties::load_or_default(&properties_path).unwrap();
        let runtime = RuntimeSelection {
            world_name: "world".to_string(),
            universe: dir.clone(),
            port: 25565,
            server_id: None,
        };

        migrate_legacy_announce_player_achievements(&properties, &runtime).unwrap();

        assert_eq!(layout.load_level_dat().unwrap(), before);
        let _ = fs::remove_dir_all(&dir);
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
    #[test]
    fn startup_uses_stored_pack_configuration_and_new_world_initialization() {
        let dir = temp_workdir("stored-data-config");
        let properties_path = dir.join("server.properties");
        fs::write(
            &properties_path,
            "initial-enabled-packs=vanilla,file/initial\n",
        )
        .unwrap();
        let properties = ServerProperties::load_or_default(&properties_path).unwrap();
        let layout = WorldLayout::new(dir.join("world"));
        let (fresh, init) = super::load_initial_data_configuration(&layout, &properties).unwrap();
        assert!(init);
        assert_eq!(fresh.data_packs.enabled, ["vanilla", "file/initial"]);
        let mut level = minimal_level_data(Tag::Compound(vec![]));
        level.data_configuration = crate::resources::WorldDataConfiguration {
            data_packs: crate::resources::DataPackConfig::new(
                ["vanilla", "file/saved"],
                ["file/initial"],
            ),
            enabled_features: crate::registry::FeatureFlagSet::of(&[
                crate::registry::feature_flags::TRADE_REBALANCE,
            ]),
        };
        layout
            .save_level_dat(&level.to_level_dat().unwrap())
            .unwrap();
        let (stored, init) = super::load_initial_data_configuration(&layout, &properties).unwrap();
        assert!(!init);
        assert_eq!(stored, level.data_configuration);
        // Existing worlds with absent fields use codec defaults, not initial properties.
        layout
            .save_level_dat(&Tag::Compound(vec![(
                "Data".to_owned(),
                Tag::Compound(vec![]),
            )]))
            .unwrap();
        let (stored, init) = super::load_initial_data_configuration(&layout, &properties).unwrap();
        assert!(!init);
        assert_eq!(stored, crate::resources::WorldDataConfiguration::default_26_1_2());
        fs::remove_dir_all(dir).unwrap();
    }
}
