#![allow(dead_code)]

use crate::command::{LevelBasedPermissionSet, PermissionLevel};
use crate::jsonrpc_methods::{
    ClientInfo, JsonRpcDifficulty, JsonRpcServerSettings, MinecraftServerSettingsService,
};
use crate::jsonrpc_minecraft_api::DedicatedServerIdentity;
use crate::network::play::GameMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinecraftServerSettingsEvent {
    KickUnlistedPlayers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftServerSettingsServiceImplModel {
    pub server: DedicatedServerIdentity,
    pub settings: JsonRpcServerSettings,
    pub log_messages: Vec<(ClientInfo, String)>,
    pub server_events: Vec<MinecraftServerSettingsEvent>,
}

impl MinecraftServerSettingsServiceImplModel {
    pub const fn new(server: DedicatedServerIdentity, settings: JsonRpcServerSettings) -> Self {
        Self {
            server,
            settings,
            log_messages: Vec::new(),
            server_events: Vec::new(),
        }
    }

    fn log(&mut self, client_info: ClientInfo, message: String) {
        self.log_messages.push((client_info, message));
    }

    fn kick_unlisted_players(&mut self) {
        self.server_events
            .push(MinecraftServerSettingsEvent::KickUnlistedPlayers);
    }
}

impl MinecraftServerSettingsService for MinecraftServerSettingsServiceImplModel {
    fn is_auto_save(&self) -> bool {
        self.settings.is_auto_save()
    }

    fn set_auto_save(&mut self, enabled: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!("Update autosave from {} to {}", self.is_auto_save(), enabled),
        );
        self.settings.set_auto_save(enabled, client_info)
    }

    fn get_difficulty(&self) -> JsonRpcDifficulty {
        self.settings.get_difficulty()
    }

    fn set_difficulty(
        &mut self,
        difficulty: JsonRpcDifficulty,
        client_info: ClientInfo,
    ) -> JsonRpcDifficulty {
        self.log(
            client_info,
            format!(
                "Update difficulty from '{}' to '{}'",
                difficulty_name(self.get_difficulty()),
                difficulty_name(difficulty)
            ),
        );
        self.settings.set_difficulty(difficulty, client_info)
    }

    fn is_enforce_whitelist(&self) -> bool {
        self.settings.is_enforce_whitelist()
    }

    fn set_enforce_whitelist(&mut self, enforce: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!(
                "Update enforce allowlist from {} to {}",
                self.is_enforce_whitelist(),
                enforce
            ),
        );
        let result = self.settings.set_enforce_whitelist(enforce, client_info);
        self.kick_unlisted_players();
        result
    }

    fn is_using_whitelist(&self) -> bool {
        self.settings.is_using_whitelist()
    }

    fn set_using_whitelist(&mut self, use_whitelist: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!(
                "Update using allowlist from {} to {}",
                self.is_using_whitelist(),
                use_whitelist
            ),
        );
        let result = self
            .settings
            .set_using_whitelist(use_whitelist, client_info);
        self.kick_unlisted_players();
        result
    }

    fn get_max_players(&self) -> i32 {
        self.settings.get_max_players()
    }

    fn set_max_players(&mut self, max_players: i32, client_info: ClientInfo) -> i32 {
        self.log(
            client_info,
            format!(
                "Update max players from {} to {}",
                self.get_max_players(),
                max_players
            ),
        );
        self.settings.set_max_players(max_players, client_info)
    }

    fn get_pause_when_empty_seconds(&self) -> i32 {
        self.settings.get_pause_when_empty_seconds()
    }

    fn set_pause_when_empty_seconds(&mut self, seconds: i32, client_info: ClientInfo) -> i32 {
        self.log(
            client_info,
            format!(
                "Update pause when empty from {} seconds to {} seconds",
                self.get_pause_when_empty_seconds(),
                seconds
            ),
        );
        self.settings
            .set_pause_when_empty_seconds(seconds, client_info)
    }

    fn get_player_idle_timeout(&self) -> i32 {
        self.settings.get_player_idle_timeout()
    }

    fn set_player_idle_timeout(&mut self, minutes: i32, client_info: ClientInfo) -> i32 {
        self.log(
            client_info,
            format!(
                "Update player idle timeout from {} minutes to {} minutes",
                self.get_player_idle_timeout(),
                minutes
            ),
        );
        self.settings
            .set_player_idle_timeout(minutes, client_info)
    }

    fn allow_flight(&self) -> bool {
        self.settings.allow_flight()
    }

    fn set_allow_flight(&mut self, allow: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!("Update allow flight from {} to {}", self.allow_flight(), allow),
        );
        self.settings.set_allow_flight(allow, client_info)
    }

    fn get_spawn_protection_radius(&self) -> i32 {
        self.settings.get_spawn_protection_radius()
    }

    fn set_spawn_protection_radius(&mut self, radius: i32, client_info: ClientInfo) -> i32 {
        self.log(
            client_info,
            format!(
                "Update spawn protection radius from {} to {}",
                self.get_spawn_protection_radius(),
                radius
            ),
        );
        self.settings
            .set_spawn_protection_radius(radius, client_info)
    }

    fn get_motd(&self) -> String {
        self.settings.get_motd()
    }

    fn set_motd(&mut self, motd: String, client_info: ClientInfo) -> String {
        self.log(
            client_info,
            format!("Update MOTD from '{}' to '{}'", self.get_motd(), motd),
        );
        self.settings.set_motd(motd, client_info)
    }

    fn force_game_mode(&self) -> bool {
        self.settings.force_game_mode()
    }

    fn set_force_game_mode(&mut self, force: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!(
                "Update force game mode from {} to {}",
                self.force_game_mode(),
                force
            ),
        );
        self.settings.set_force_game_mode(force, client_info)
    }

    fn get_game_mode(&self) -> GameMode {
        self.settings.get_game_mode()
    }

    fn set_game_mode(&mut self, game_mode: GameMode, client_info: ClientInfo) -> GameMode {
        self.log(
            client_info,
            format!(
                "Update game mode from '{}' to '{}'",
                game_mode_name(self.get_game_mode()),
                game_mode_name(game_mode)
            ),
        );
        self.settings.set_game_mode(game_mode, client_info)
    }

    fn get_view_distance(&self) -> i32 {
        self.settings.get_view_distance()
    }

    fn set_view_distance(&mut self, distance: i32, client_info: ClientInfo) -> i32 {
        self.log(
            client_info,
            format!(
                "Update view distance from {} to {}",
                self.get_view_distance(),
                distance
            ),
        );
        self.settings.set_view_distance(distance, client_info)
    }

    fn get_simulation_distance(&self) -> i32 {
        self.settings.get_simulation_distance()
    }

    fn set_simulation_distance(&mut self, distance: i32, client_info: ClientInfo) -> i32 {
        self.log(
            client_info,
            format!(
                "Update simulation distance from {} to {}",
                self.get_simulation_distance(),
                distance
            ),
        );
        self.settings.set_simulation_distance(distance, client_info)
    }

    fn accepts_transfers(&self) -> bool {
        self.settings.accepts_transfers()
    }

    fn set_accepts_transfers(&mut self, accept: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!(
                "Update accepts transfers from {} to {}",
                self.accepts_transfers(),
                accept
            ),
        );
        self.settings.set_accepts_transfers(accept, client_info)
    }

    fn get_status_heartbeat_interval(&self) -> i32 {
        self.settings.get_status_heartbeat_interval()
    }

    fn set_status_heartbeat_interval(&mut self, interval: i32, client_info: ClientInfo) -> i32 {
        self.log(
            client_info,
            format!(
                "Update status heartbeat interval from {} to {}",
                self.get_status_heartbeat_interval(),
                interval
            ),
        );
        self.settings
            .set_status_heartbeat_interval(interval, client_info)
    }

    fn get_operator_user_permissions(&self) -> LevelBasedPermissionSet {
        self.settings.get_operator_user_permissions()
    }

    fn set_operator_user_permissions(
        &mut self,
        permissions: LevelBasedPermissionSet,
        client_info: ClientInfo,
    ) -> LevelBasedPermissionSet {
        self.log(
            client_info,
            format!(
                "Update operator user permission level from {} to {}",
                permission_set_name(self.get_operator_user_permissions()),
                permission_level_name(permissions.level())
            ),
        );
        self.settings
            .set_operator_user_permissions(permissions, client_info)
    }

    fn hides_online_players(&self) -> bool {
        self.settings.hides_online_players()
    }

    fn set_hides_online_players(&mut self, hide: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!(
                "Update hides online players from {} to {}",
                self.hides_online_players(),
                hide
            ),
        );
        self.settings.set_hides_online_players(hide, client_info)
    }

    fn replies_to_status(&self) -> bool {
        self.settings.replies_to_status()
    }

    fn set_replies_to_status(&mut self, enable: bool, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!(
                "Update replies to status from {} to {}",
                self.replies_to_status(),
                enable
            ),
        );
        self.settings.set_replies_to_status(enable, client_info)
    }

    fn get_entity_broadcast_range_percentage(&self) -> i32 {
        self.settings.get_entity_broadcast_range_percentage()
    }

    fn set_entity_broadcast_range_percentage(
        &mut self,
        percentage: i32,
        client_info: ClientInfo,
    ) -> i32 {
        self.log(
            client_info,
            format!(
                "Update entity broadcast range percentage from {}% to {}%",
                self.get_entity_broadcast_range_percentage(),
                percentage
            ),
        );
        self.settings
            .set_entity_broadcast_range_percentage(percentage, client_info)
    }
}

fn difficulty_name(difficulty: JsonRpcDifficulty) -> &'static str {
    match difficulty {
        JsonRpcDifficulty::Peaceful => "PEACEFUL",
        JsonRpcDifficulty::Easy => "EASY",
        JsonRpcDifficulty::Normal => "NORMAL",
        JsonRpcDifficulty::Hard => "HARD",
    }
}

fn game_mode_name(game_mode: GameMode) -> &'static str {
    match game_mode {
        GameMode::Survival => "SURVIVAL",
        GameMode::Creative => "CREATIVE",
        GameMode::Adventure => "ADVENTURE",
        GameMode::Spectator => "SPECTATOR",
    }
}

fn permission_set_name(permissions: LevelBasedPermissionSet) -> String {
    format!("permission level: {}", permission_level_name(permissions.level()))
}

fn permission_level_name(level: PermissionLevel) -> &'static str {
    match level {
        PermissionLevel::All => "ALL",
        PermissionLevel::Moderators => "MODERATORS",
        PermissionLevel::Gamemasters => "GAMEMASTERS",
        PermissionLevel::Admins => "ADMINS",
        PermissionLevel::Owners => "OWNERS",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implementation_logs_then_applies_all_twenty_settings() {
        let client = ClientInfo::of(111);
        let mut service = MinecraftServerSettingsServiceImplModel::new(
            DedicatedServerIdentity(110),
            JsonRpcServerSettings::default(),
        );
        apply_all_settings(&mut service, client);

        assert_eq!(service.log_messages.len(), 20);
        assert_eq!(
            service.log_messages,
            expected_log_messages(client)
                .into_iter()
                .map(|message| (client, message.to_string()))
                .collect::<Vec<_>>()
        );
        assert_eq!(service.server, DedicatedServerIdentity(110));
        assert_eq!(service.settings.motd, "Parity");
        assert_eq!(service.settings.difficulty, JsonRpcDifficulty::Hard);
        assert_eq!(service.settings.game_mode, GameMode::Creative);
        assert_eq!(
            service.settings.operator_user_permissions,
            LevelBasedPermissionSet::ADMIN
        );
        assert_eq!(service.settings.entity_broadcast_range_percentage, 150);
        assert_eq!(
            service.server_events,
            vec![
                MinecraftServerSettingsEvent::KickUnlistedPlayers,
                MinecraftServerSettingsEvent::KickUnlistedPlayers,
            ]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn implementation_source_matches_java_26_1_2() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftServerSettingsServiceImpl.java"
        );
        for sentinel in [
            "private final DedicatedServer server;",
            "private final JsonRpcLogger jsonrpcLogger;",
            "public MinecraftServerSettingsServiceImpl(final DedicatedServer server, final JsonRpcLogger jsonrpcLogger)",
            "this.server = server;",
            "this.jsonrpcLogger = jsonrpcLogger;",
            "Update autosave from {} to {}",
            "Update difficulty from '{}' to '{}'",
            "Update enforce allowlist from {} to {}",
            "Update using allowlist from {} to {}",
            "Update max players from {} to {}",
            "Update pause when empty from {} seconds to {} seconds",
            "Update player idle timeout from {} minutes to {} minutes",
            "Update allow flight from {} to {}",
            "Update spawn protection radius from {} to {}",
            "Update MOTD from '{}' to '{}'",
            "Update force game mode from {} to {}",
            "Update game mode from '{}' to '{}'",
            "Update view distance from {} to {}",
            "Update simulation distance from {} to {}",
            "Update accepts transfers from {} to {}",
            "Update status heartbeat interval from {} to {}",
            "Update operator user permission level from {} to {}",
            "Update hides online players from {} to {}",
            "Update replies to status from {} to {}",
            "Update entity broadcast range percentage from {}% to {}%",
            "this.server.kickUnlistedPlayers();",
            "this.server.setOperatorUserPermissions(permissions);",
            "return this.getEntityBroadcastRangePercentage();",
        ] {
            assert!(
                SOURCE.contains(sentinel),
                "MinecraftServerSettingsServiceImpl.java missing: {sentinel}"
            );
        }
        assert_eq!(SOURCE.matches("this.server.kickUnlistedPlayers();").count(), 2);
        assert_eq!(SOURCE.matches("this.jsonrpcLogger.log(").count(), 20);
    }

    fn apply_all_settings(
        service: &mut MinecraftServerSettingsServiceImplModel,
        client: ClientInfo,
    ) {
        service.set_auto_save(false, client);
        service.set_difficulty(JsonRpcDifficulty::Hard, client);
        service.set_enforce_whitelist(true, client);
        service.set_using_whitelist(true, client);
        service.set_max_players(40, client);
        service.set_pause_when_empty_seconds(5, client);
        service.set_player_idle_timeout(15, client);
        service.set_allow_flight(true, client);
        service.set_spawn_protection_radius(3, client);
        service.set_motd("Parity".to_string(), client);
        service.set_force_game_mode(true, client);
        service.set_game_mode(GameMode::Creative, client);
        service.set_view_distance(16, client);
        service.set_simulation_distance(12, client);
        service.set_accepts_transfers(true, client);
        service.set_status_heartbeat_interval(30, client);
        service.set_operator_user_permissions(LevelBasedPermissionSet::ADMIN, client);
        service.set_hides_online_players(true, client);
        service.set_replies_to_status(false, client);
        service.set_entity_broadcast_range_percentage(150, client);
    }

    fn expected_log_messages(_client: ClientInfo) -> [&'static str; 20] {
        [
            "Update autosave from true to false",
            "Update difficulty from 'NORMAL' to 'HARD'",
            "Update enforce allowlist from false to true",
            "Update using allowlist from false to true",
            "Update max players from 20 to 40",
            "Update pause when empty from 60 seconds to 5 seconds",
            "Update player idle timeout from 0 minutes to 15 minutes",
            "Update allow flight from false to true",
            "Update spawn protection radius from 16 to 3",
            "Update MOTD from 'A Minecraft Server' to 'Parity'",
            "Update force game mode from false to true",
            "Update game mode from 'SURVIVAL' to 'CREATIVE'",
            "Update view distance from 10 to 16",
            "Update simulation distance from 10 to 12",
            "Update accepts transfers from false to true",
            "Update status heartbeat interval from 0 to 30",
            "Update operator user permission level from permission level: GAMEMASTERS to ADMINS",
            "Update hides online players from false to true",
            "Update replies to status from true to false",
            "Update entity broadcast range percentage from 100% to 150%",
        ]
    }
}
