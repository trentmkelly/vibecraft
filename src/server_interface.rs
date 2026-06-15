#![allow(dead_code)]

use crate::server_info::ServerInfo;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedicatedServerPropertiesModel {
    pub server_ip: String,
    pub server_port: i32,
    pub level_name: String,
}

impl DedicatedServerPropertiesModel {
    pub fn new(server_ip: impl Into<String>, server_port: i32, level_name: impl Into<String>) -> Self {
        Self {
            server_ip: server_ip.into(),
            server_port,
            level_name: level_name.into(),
        }
    }
}

/// Java's `ServerInterface` extends the status-only `ServerInfo` surface with
/// dedicated-server administration getters and a command execution hook.
pub trait ServerInterface: ServerInfo {
    fn get_properties(&self) -> &DedicatedServerPropertiesModel;

    fn get_server_ip(&self) -> &str;

    fn get_server_port(&self) -> i32;

    fn get_server_name(&self) -> &str;

    fn get_player_names(&self) -> &[String];

    fn get_level_id_name(&self) -> &str;

    fn get_plugin_names(&self) -> &str;

    fn run_command(&self, command: &str) -> String;
}

#[derive(Debug)]
pub struct StaticServerInterface {
    properties: DedicatedServerPropertiesModel,
    server_name: String,
    player_names: Vec<String>,
    plugin_names: String,
    motd: String,
    server_version: String,
    max_players: i32,
    command_result: String,
    executed_commands: RefCell<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticServerInterfaceConfig {
    pub properties: DedicatedServerPropertiesModel,
    pub server_name: String,
    pub player_names: Vec<String>,
    pub plugin_names: String,
    pub motd: String,
    pub server_version: String,
    pub max_players: i32,
    pub command_result: String,
}

impl StaticServerInterface {
    pub fn new(config: StaticServerInterfaceConfig) -> Self {
        Self {
            properties: config.properties,
            server_name: config.server_name,
            player_names: config.player_names,
            plugin_names: config.plugin_names,
            motd: config.motd,
            server_version: config.server_version,
            max_players: config.max_players,
            command_result: config.command_result,
            executed_commands: RefCell::new(Vec::new()),
        }
    }

    pub fn executed_commands(&self) -> Vec<String> {
        self.executed_commands.borrow().clone()
    }
}

impl ServerInfo for StaticServerInterface {
    fn get_motd(&self) -> &str {
        &self.motd
    }

    fn get_server_version(&self) -> &str {
        &self.server_version
    }

    fn get_player_count(&self) -> i32 {
        i32::try_from(self.player_names.len()).unwrap_or(i32::MAX)
    }

    fn get_max_players(&self) -> i32 {
        self.max_players
    }
}

impl ServerInterface for StaticServerInterface {
    fn get_properties(&self) -> &DedicatedServerPropertiesModel {
        &self.properties
    }

    fn get_server_ip(&self) -> &str {
        &self.properties.server_ip
    }

    fn get_server_port(&self) -> i32 {
        self.properties.server_port
    }

    fn get_server_name(&self) -> &str {
        &self.server_name
    }

    fn get_player_names(&self) -> &[String] {
        &self.player_names
    }

    fn get_level_id_name(&self) -> &str {
        &self.properties.level_name
    }

    fn get_plugin_names(&self) -> &str {
        &self.plugin_names
    }

    fn run_command(&self, command: &str) -> String {
        self.executed_commands
            .borrow_mut()
            .push(command.to_string());
        self.command_result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/ServerInterface.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_server_interface_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public interface ServerInterface extends ServerInfo"));
        assert!(JAVA_SOURCE.contains("DedicatedServerProperties getProperties();"));
        assert!(JAVA_SOURCE.contains("String getServerIp();"));
        assert!(JAVA_SOURCE.contains("int getServerPort();"));
        assert!(JAVA_SOURCE.contains("String getServerName();"));
        assert!(JAVA_SOURCE.contains("String[] getPlayerNames();"));
        assert!(JAVA_SOURCE.contains("String getLevelIdName();"));
        assert!(JAVA_SOURCE.contains("String getPluginNames();"));
        assert!(JAVA_SOURCE.contains("String runCommand(String command);"));
    }

    #[test]
    fn server_utility_static_server_interface_exposes_java_getter_surface() {
        let properties = DedicatedServerPropertiesModel::new("127.0.0.1", 25565, "world");
        let server = StaticServerInterface::new(StaticServerInterfaceConfig {
            properties: properties.clone(),
            server_name: "Dedicated Server".to_string(),
            player_names: vec!["Alex".to_string(), "Steve".to_string()],
            plugin_names: String::new(),
            motd: "A Minecraft Server".to_string(),
            server_version: "1.26.1".to_string(),
            max_players: 20,
            command_result: "Executed command".to_string(),
        });

        assert_eq!(server.get_properties(), &properties);
        assert_eq!(server.get_server_ip(), "127.0.0.1");
        assert_eq!(server.get_server_port(), 25565);
        assert_eq!(server.get_server_name(), "Dedicated Server");
        assert_eq!(
            server.get_player_names(),
            &["Alex".to_string(), "Steve".to_string()]
        );
        assert_eq!(server.get_level_id_name(), "world");
        assert_eq!(server.get_plugin_names(), "");
        assert_eq!(server.get_motd(), "A Minecraft Server");
        assert_eq!(server.get_server_version(), "1.26.1");
        assert_eq!(server.get_player_count(), 2);
        assert_eq!(server.get_max_players(), 20);
    }

    #[test]
    fn server_utility_static_server_interface_records_command_invocations() {
        let server = StaticServerInterface::new(StaticServerInterfaceConfig {
            properties: DedicatedServerPropertiesModel::new("", 25565, "world"),
            server_name: "Dedicated Server".to_string(),
            player_names: Vec::new(),
            plugin_names: String::new(),
            motd: "motd".to_string(),
            server_version: "1.26.1".to_string(),
            max_players: 20,
            command_result: "ok".to_string(),
        });

        assert_eq!(server.run_command("say hello"), "ok");
        assert_eq!(server.run_command("list"), "ok");
        assert_eq!(
            server.executed_commands(),
            vec!["say hello".to_string(), "list".to_string()]
        );
    }
}
