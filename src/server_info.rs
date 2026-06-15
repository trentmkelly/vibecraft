#![allow(dead_code)]

/// Minimal server status surface exposed by Java's `ServerInfo` interface.
pub trait ServerInfo {
    fn get_motd(&self) -> &str;

    fn get_server_version(&self) -> &str;

    fn get_player_count(&self) -> i32;

    fn get_max_players(&self) -> i32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticServerInfo {
    motd: String,
    server_version: String,
    player_count: i32,
    max_players: i32,
}

impl StaticServerInfo {
    pub fn new(
        motd: impl Into<String>,
        server_version: impl Into<String>,
        player_count: i32,
        max_players: i32,
    ) -> Self {
        Self {
            motd: motd.into(),
            server_version: server_version.into(),
            player_count,
            max_players,
        }
    }
}

impl ServerInfo for StaticServerInfo {
    fn get_motd(&self) -> &str {
        &self.motd
    }

    fn get_server_version(&self) -> &str {
        &self.server_version
    }

    fn get_player_count(&self) -> i32 {
        self.player_count
    }

    fn get_max_players(&self) -> i32 {
        self.max_players
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/ServerInfo.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_server_info_matches_java_interface_shape() {
        assert!(JAVA_SOURCE.contains("public interface ServerInfo"));
        assert!(JAVA_SOURCE.contains("String getMotd();"));
        assert!(JAVA_SOURCE.contains("String getServerVersion();"));
        assert!(JAVA_SOURCE.contains("int getPlayerCount();"));
        assert!(JAVA_SOURCE.contains("int getMaxPlayers();"));
    }

    #[test]
    fn server_utility_static_server_info_exposes_java_getter_surface() {
        let info = StaticServerInfo::new("A Minecraft Server", "1.26.1", 7, 20);

        assert_eq!(info.get_motd(), "A Minecraft Server");
        assert_eq!(info.get_server_version(), "1.26.1");
        assert_eq!(info.get_player_count(), 7);
        assert_eq!(info.get_max_players(), 20);
    }
}
