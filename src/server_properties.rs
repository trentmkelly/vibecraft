use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerProperties {
    pub raw: BTreeMap<String, String>,
    pub accepts_transfers: bool,
    pub allow_flight: bool,
    pub announce_player_achievements: Option<bool>,
    pub broadcast_console_to_ops: bool,
    pub broadcast_rcon_to_ops: bool,
    pub bug_report_link: String,
    pub code_of_conduct: bool,
    pub difficulty: String,
    pub enable_jmx_monitoring: bool,
    pub enable_query: bool,
    pub enable_rcon: bool,
    pub enable_status: bool,
    pub enforce_secure_profile: bool,
    pub enforce_whitelist: bool,
    pub entity_broadcast_range_percentage: u32,
    pub force_game_mode: bool,
    pub function_permission_level: u32,
    pub game_mode: String,
    pub generate_structures: bool,
    pub generator_settings: String,
    pub hardcore: bool,
    pub initial_disabled_packs: String,
    pub initial_enabled_packs: String,
    pub level_name: String,
    pub level_seed: String,
    pub level_type: String,
    pub log_ips: bool,
    pub management_server_allowed_origins: String,
    pub management_server_enabled: bool,
    pub management_server_host: String,
    pub management_server_port: u16,
    pub management_server_secret: String,
    pub management_server_tls_enabled: bool,
    pub management_server_tls_keystore: String,
    pub management_server_tls_keystore_password: String,
    pub max_chained_neighbor_updates: u32,
    pub max_tick_time: u64,
    pub max_world_size: u32,
    pub network_compression_threshold: i32,
    pub online_mode: bool,
    pub op_permission_level: u32,
    pub pause_when_empty_seconds: u32,
    pub player_idle_timeout: u32,
    pub prevent_proxy_connections: bool,
    pub query_port: u16,
    pub rate_limit_packets_per_second: u32,
    pub rcon_password: String,
    pub rcon_port: u16,
    pub region_file_compression: String,
    pub require_resource_pack: bool,
    pub resource_pack: String,
    pub resource_pack_id: String,
    pub resource_pack_prompt: String,
    pub resource_pack_sha1: String,
    pub server_ip: String,
    pub server_port: u16,
    pub max_players: u32,
    pub motd: String,
    pub hide_online_players: bool,
    pub simulation_distance: u32,
    pub spawn_protection: u32,
    pub status_heartbeat_interval: u32,
    pub sync_chunk_writes: bool,
    pub text_filtering_config: String,
    pub text_filtering_version: u32,
    pub use_native_transport: bool,
    pub view_distance: u32,
    pub white_list: bool,
}

impl ServerProperties {
    pub fn load_or_default(path: &Path) -> Result<Self, String> {
        let mut raw = vanilla_defaults();
        if path.exists() {
            for (key, value) in parse_properties(
                &fs::read_to_string(path)
                    .map_err(|err| format!("Failed to read '{}': {err}", path.display()))?,
            ) {
                raw.insert(key, value);
            }
        }

        Ok(Self::from_raw(raw))
    }

    fn from_raw(raw: BTreeMap<String, String>) -> Self {
        Self {
            accepts_transfers: bool_key(&raw, "accepts-transfers", false),
            allow_flight: bool_key(&raw, "allow-flight", false),
            announce_player_achievements: optional_bool_key(&raw, "announce-player-achievements"),
            broadcast_console_to_ops: bool_key(&raw, "broadcast-console-to-ops", true),
            broadcast_rcon_to_ops: bool_key(&raw, "broadcast-rcon-to-ops", true),
            bug_report_link: string_key(&raw, "bug-report-link", ""),
            code_of_conduct: bool_key(&raw, "enable-code-of-conduct", false),
            difficulty: string_key(&raw, "difficulty", "easy"),
            enable_jmx_monitoring: bool_key(&raw, "enable-jmx-monitoring", false),
            enable_query: bool_key(&raw, "enable-query", false),
            enable_rcon: bool_key(&raw, "enable-rcon", false),
            enable_status: bool_key(&raw, "enable-status", true),
            enforce_secure_profile: bool_key(&raw, "enforce-secure-profile", true),
            enforce_whitelist: bool_key(&raw, "enforce-whitelist", false),
            entity_broadcast_range_percentage: u32_key(
                &raw,
                "entity-broadcast-range-percentage",
                100,
            )
            .clamp(10, 1000),
            force_game_mode: bool_key(&raw, "force-gamemode", false),
            function_permission_level: u32_key(&raw, "function-permission-level", 2),
            game_mode: string_key(&raw, "gamemode", "survival"),
            generate_structures: bool_key(&raw, "generate-structures", true),
            generator_settings: string_key(&raw, "generator-settings", "{}"),
            hardcore: bool_key(&raw, "hardcore", false),
            initial_disabled_packs: string_key(&raw, "initial-disabled-packs", ""),
            initial_enabled_packs: string_key(&raw, "initial-enabled-packs", "vanilla"),
            level_name: string_key(&raw, "level-name", "world"),
            level_seed: string_key(&raw, "level-seed", ""),
            level_type: string_key(&raw, "level-type", "minecraft:normal"),
            log_ips: bool_key(&raw, "log-ips", true),
            management_server_allowed_origins: string_key(
                &raw,
                "management-server-allowed-origins",
                "",
            ),
            management_server_enabled: bool_key(&raw, "management-server-enabled", false),
            management_server_host: string_key(&raw, "management-server-host", "localhost"),
            management_server_port: u16_key(&raw, "management-server-port", 0),
            management_server_secret: string_key(&raw, "management-server-secret", ""),
            management_server_tls_enabled: bool_key(&raw, "management-server-tls-enabled", true),
            management_server_tls_keystore: string_key(&raw, "management-server-tls-keystore", ""),
            management_server_tls_keystore_password: string_key(
                &raw,
                "management-server-tls-keystore-password",
                "",
            ),
            max_chained_neighbor_updates: u32_key(&raw, "max-chained-neighbor-updates", 1_000_000),
            max_players: u32_key(&raw, "max-players", 20),
            max_tick_time: u64_key(&raw, "max-tick-time", 60_000),
            max_world_size: u32_key(&raw, "max-world-size", 29_999_984).clamp(1, 29_999_984),
            motd: string_key(&raw, "motd", "A Minecraft Server"),
            network_compression_threshold: i32_key(&raw, "network-compression-threshold", 256),
            online_mode: bool_key(&raw, "online-mode", true),
            op_permission_level: u32_key(&raw, "op-permission-level", 4),
            pause_when_empty_seconds: u32_key(&raw, "pause-when-empty-seconds", 60),
            player_idle_timeout: u32_key(&raw, "player-idle-timeout", 0),
            prevent_proxy_connections: bool_key(&raw, "prevent-proxy-connections", false),
            query_port: u16_key(&raw, "query.port", 25565),
            rate_limit_packets_per_second: u32_key(&raw, "rate-limit", 0),
            rcon_password: string_key(&raw, "rcon.password", ""),
            rcon_port: u16_key(&raw, "rcon.port", 25575),
            region_file_compression: string_key(&raw, "region-file-compression", "deflate"),
            require_resource_pack: bool_key(&raw, "require-resource-pack", false),
            resource_pack: string_key(&raw, "resource-pack", ""),
            resource_pack_id: string_key(&raw, "resource-pack-id", ""),
            resource_pack_prompt: string_key(&raw, "resource-pack-prompt", ""),
            resource_pack_sha1: string_key(&raw, "resource-pack-sha1", ""),
            server_ip: string_key(&raw, "server-ip", ""),
            server_port: u16_key(&raw, "server-port", 25565),
            hide_online_players: bool_key(&raw, "hide-online-players", false),
            simulation_distance: u32_key(&raw, "simulation-distance", 10),
            spawn_protection: u32_key(&raw, "spawn-protection", 16),
            status_heartbeat_interval: u32_key(&raw, "status-heartbeat-interval", 0),
            sync_chunk_writes: bool_key(&raw, "sync-chunk-writes", true),
            text_filtering_config: string_key(&raw, "text-filtering-config", ""),
            text_filtering_version: u32_key(&raw, "text-filtering-version", 0),
            use_native_transport: bool_key(&raw, "use-native-transport", true),
            view_distance: u32_key(&raw, "view-distance", 10),
            white_list: bool_key(&raw, "white-list", false),
            raw,
        }
    }

    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        let mut raw = self.raw.clone();
        raw.insert(key.to_string(), value.into());
        *self = Self::from_raw(raw);
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        for (key, value) in vanilla_defaults() {
            self.raw.entry(key).or_insert(value);
        }

        let mut output = String::from("# Minecraft server properties\n");
        for (key, value) in &self.raw {
            output.push_str(key);
            output.push('=');
            output.push_str(value);
            output.push('\n');
        }
        fs::write(path, output)
            .map_err(|err| format!("Failed to write '{}': {err}", path.display()))
    }
}

fn parse_properties(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        map.insert(key.trim().to_string(), value.trim().to_string());
    }
    map
}

fn string_key(raw: &BTreeMap<String, String>, key: &str, default: &str) -> String {
    raw.get(key).cloned().unwrap_or_else(|| default.to_string())
}

fn bool_key(raw: &BTreeMap<String, String>, key: &str, default: bool) -> bool {
    raw.get(key)
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

fn optional_bool_key(raw: &BTreeMap<String, String>, key: &str) -> Option<bool> {
    raw.get(key).map(|value| value.eq_ignore_ascii_case("true"))
}

fn u16_key(raw: &BTreeMap<String, String>, key: &str, default: u16) -> u16 {
    raw.get(key)
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(default)
}

fn u32_key(raw: &BTreeMap<String, String>, key: &str, default: u32) -> u32 {
    raw.get(key)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

fn u64_key(raw: &BTreeMap<String, String>, key: &str, default: u64) -> u64 {
    raw.get(key)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn i32_key(raw: &BTreeMap<String, String>, key: &str, default: i32) -> i32 {
    raw.get(key)
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default)
}

fn vanilla_defaults() -> BTreeMap<String, String> {
    let mut defaults = BTreeMap::new();
    for (key, value) in [
        ("accepts-transfers", "false"),
        ("allow-flight", "false"),
        ("broadcast-console-to-ops", "true"),
        ("broadcast-rcon-to-ops", "true"),
        ("bug-report-link", ""),
        ("difficulty", "easy"),
        ("enable-code-of-conduct", "false"),
        ("enable-command-block", "false"),
        ("enable-jmx-monitoring", "false"),
        ("enable-query", "false"),
        ("enable-rcon", "false"),
        ("enable-status", "true"),
        ("enforce-secure-profile", "true"),
        ("enforce-whitelist", "false"),
        ("entity-broadcast-range-percentage", "100"),
        ("force-gamemode", "false"),
        ("function-permission-level", "2"),
        ("gamemode", "survival"),
        ("generate-structures", "true"),
        ("generator-settings", "{}"),
        ("hardcore", "false"),
        ("hide-online-players", "false"),
        ("initial-disabled-packs", ""),
        ("initial-enabled-packs", "vanilla"),
        ("level-name", "world"),
        ("level-seed", ""),
        ("level-type", "minecraft:normal"),
        ("log-ips", "true"),
        ("management-server-allowed-origins", ""),
        ("management-server-enabled", "false"),
        ("management-server-host", "localhost"),
        ("management-server-port", "0"),
        ("management-server-secret", ""),
        ("management-server-tls-enabled", "true"),
        ("management-server-tls-keystore", ""),
        ("management-server-tls-keystore-password", ""),
        ("max-chained-neighbor-updates", "1000000"),
        ("max-players", "20"),
        ("max-tick-time", "60000"),
        ("max-world-size", "29999984"),
        ("motd", "A Minecraft Server"),
        ("network-compression-threshold", "256"),
        ("online-mode", "true"),
        ("op-permission-level", "4"),
        ("pause-when-empty-seconds", "60"),
        ("player-idle-timeout", "0"),
        ("prevent-proxy-connections", "false"),
        ("query.port", "25565"),
        ("rate-limit", "0"),
        ("rcon.password", ""),
        ("rcon.port", "25575"),
        ("region-file-compression", "deflate"),
        ("require-resource-pack", "false"),
        ("resource-pack", ""),
        ("resource-pack-id", ""),
        ("resource-pack-prompt", ""),
        ("resource-pack-sha1", ""),
        ("server-ip", ""),
        ("server-port", "25565"),
        ("simulation-distance", "10"),
        ("spawn-protection", "16"),
        ("status-heartbeat-interval", "0"),
        ("sync-chunk-writes", "true"),
        ("text-filtering-config", ""),
        ("text-filtering-version", "0"),
        ("use-native-transport", "true"),
        ("view-distance", "10"),
        ("white-list", "false"),
    ] {
        defaults.insert(key.to_string(), value.to_string());
    }
    defaults
}

#[cfg(test)]
mod tests {
    use super::{parse_properties, ServerProperties};
    use std::fs;

    #[test]
    fn parses_basic_properties_and_skips_comments() {
        let parsed = parse_properties("# comment\nserver-port=25566\nmotd = Test\n");
        assert_eq!(parsed.get("server-port").map(String::as_str), Some("25566"));
        assert_eq!(parsed.get("motd").map(String::as_str), Some("Test"));
    }

    #[test]
    fn preserves_unknown_keys_when_saving() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "rustcraft-server-properties-{}.properties",
            std::process::id()
        ));
        fs::write(&path, "custom-key=custom-value\nserver-port=25566\n").unwrap();

        let mut properties = ServerProperties::load_or_default(&path).unwrap();
        properties.save(&path).unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert!(saved.contains("custom-key=custom-value\n"));
        assert!(saved.contains("server-port=25566\n"));
    }

    #[test]
    fn exposes_typed_vanilla_properties() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "rustcraft-typed-server-properties-{}.properties",
            std::process::id()
        ));
        fs::write(
            &path,
            "\
allow-flight=true
server-ip=127.0.0.1
server-port=25566
max-players=42
motd=Typed
hide-online-players=true
max-world-size=999999999
entity-broadcast-range-percentage=5
management-server-port=24454
rcon.port=24455
query.port=24456
announce-player-achievements=true
broadcast-console-to-ops=false
bug-report-link=https://bugs.example.invalid/rustcraft
enable-code-of-conduct=true
enable-jmx-monitoring=true
max-chained-neighbor-updates=2048
sync-chunk-writes=false
text-filtering-config=text-filter.json
text-filtering-version=2
function-permission-level=3
resource-pack-id=00000000-0000-0000-0000-000000000001
resource-pack=https://example.invalid/pack.zip
resource-pack-sha1=0123456789abcdef0123456789abcdef01234567
require-resource-pack=true
resource-pack-prompt={\"text\":\"Use pack?\"}
",
        )
        .unwrap();

        let properties = ServerProperties::load_or_default(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert!(properties.allow_flight);
        assert_eq!(properties.server_ip, "127.0.0.1");
        assert_eq!(properties.server_port, 25566);
        assert_eq!(properties.max_players, 42);
        assert_eq!(properties.motd, "Typed");
        assert!(properties.hide_online_players);
        assert_eq!(properties.max_world_size, 29_999_984);
        assert_eq!(properties.entity_broadcast_range_percentage, 10);
        assert_eq!(properties.management_server_port, 24454);
        assert_eq!(properties.rcon_port, 24455);
        assert_eq!(properties.query_port, 24456);
        assert_eq!(properties.announce_player_achievements, Some(true));
        assert!(!properties.broadcast_console_to_ops);
        assert_eq!(
            properties.bug_report_link,
            "https://bugs.example.invalid/rustcraft"
        );
        assert!(properties.code_of_conduct);
        assert!(properties.enable_jmx_monitoring);
        assert_eq!(properties.max_chained_neighbor_updates, 2048);
        assert!(!properties.sync_chunk_writes);
        assert_eq!(properties.text_filtering_config, "text-filter.json");
        assert_eq!(properties.text_filtering_version, 2);
        assert_eq!(properties.function_permission_level, 3);
        assert_eq!(
            properties.resource_pack_id,
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(properties.resource_pack, "https://example.invalid/pack.zip");
        assert_eq!(
            properties.resource_pack_sha1,
            "0123456789abcdef0123456789abcdef01234567"
        );
        assert!(properties.require_resource_pack);
        assert_eq!(properties.resource_pack_prompt, "{\"text\":\"Use pack?\"}");
    }

    #[test]
    fn mutable_properties_rehydrate_typed_fields_and_save() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "rustcraft-mutable-server-properties-{}.properties",
            std::process::id()
        ));
        let mut properties = ServerProperties::load_or_default(&path).unwrap();

        properties.set("allow-flight", "true");
        properties.set("motd", "Changed");
        properties.set("force-gamemode", "true");
        properties.set("enforce-whitelist", "true");
        properties.set("difficulty", "hard");
        properties.set("gamemode", "creative");
        properties.set("spawn-protection", "0");
        properties.set("op-permission-level", "3");
        properties.set("view-distance", "12");
        properties.set("simulation-distance", "8");
        properties.set("max-players", "100");
        properties.set("enable-status", "false");
        properties.set("hide-online-players", "true");
        properties.set("entity-broadcast-range-percentage", "250");
        properties.set("player-idle-timeout", "5");
        properties.set("status-heartbeat-interval", "20");
        properties.set("white-list", "true");
        properties.set("pause-when-empty-seconds", "0");
        properties.set("accepts-transfers", "true");
        properties.save(&path).unwrap();

        let reloaded = ServerProperties::load_or_default(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert!(reloaded.allow_flight);
        assert_eq!(reloaded.motd, "Changed");
        assert!(reloaded.force_game_mode);
        assert!(reloaded.enforce_whitelist);
        assert_eq!(reloaded.difficulty, "hard");
        assert_eq!(reloaded.game_mode, "creative");
        assert_eq!(reloaded.spawn_protection, 0);
        assert_eq!(reloaded.op_permission_level, 3);
        assert_eq!(reloaded.view_distance, 12);
        assert_eq!(reloaded.simulation_distance, 8);
        assert_eq!(reloaded.max_players, 100);
        assert!(!reloaded.enable_status);
        assert!(reloaded.hide_online_players);
        assert_eq!(reloaded.entity_broadcast_range_percentage, 250);
        assert_eq!(reloaded.player_idle_timeout, 5);
        assert_eq!(reloaded.status_heartbeat_interval, 20);
        assert!(reloaded.white_list);
        assert_eq!(reloaded.pause_when_empty_seconds, 0);
        assert!(reloaded.accepts_transfers);
    }
}
