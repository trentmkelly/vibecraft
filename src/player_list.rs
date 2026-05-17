#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::network::codec::Uuid;
use crate::network::play::GameMode;
use crate::registry::Identifier;

pub const DUPLICATE_LOGIN_DISCONNECT: &str = "multiplayer.disconnect.duplicate_login";
pub const SERVER_FULL_DISCONNECT: &str = "multiplayer.disconnect.server_full";
pub const NOT_WHITELISTED_DISCONNECT: &str = "multiplayer.disconnect.not_whitelisted";
pub const BANNED_DISCONNECT: &str = "multiplayer.disconnect.banned.reason";
pub const BANNED_IP_DISCONNECT: &str = "multiplayer.disconnect.banned_ip.reason";
pub const SERVER_SHUTDOWN_DISCONNECT: &str = "multiplayer.disconnect.server_shutdown";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerProfile {
    pub uuid: Uuid,
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListEntry {
    pub profile: PlayerProfile,
    pub listed: bool,
    pub latency: i32,
    pub game_mode: GameMode,
    pub display_name: Option<String>,
    pub show_hat: bool,
    pub list_order: i32,
    pub chat_session: Option<String>,
    pub transferred: bool,
}

impl PlayerListEntry {
    pub fn new(profile: PlayerProfile, game_mode: GameMode) -> Self {
        Self {
            profile,
            listed: true,
            latency: 0,
            game_mode,
            display_name: None,
            show_hat: true,
            list_order: 0,
            chat_session: None,
            transferred: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerInfoAction {
    AddPlayer,
    InitializeChat,
    UpdateGameMode,
    UpdateListed,
    UpdateLatency,
    UpdateDisplayName,
    UpdateListOrder,
    UpdateHat,
}

impl PlayerInfoAction {
    pub fn initializing_actions() -> Vec<Self> {
        vec![
            Self::AddPlayer,
            Self::InitializeChat,
            Self::UpdateGameMode,
            Self::UpdateListed,
            Self::UpdateLatency,
            Self::UpdateDisplayName,
            Self::UpdateHat,
            Self::UpdateListOrder,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoUpdatePacket {
    pub actions: Vec<PlayerInfoAction>,
    pub entries: Vec<PlayerListEntry>,
}

impl PlayerInfoUpdatePacket {
    pub fn initializing(entries: Vec<PlayerListEntry>) -> Self {
        Self {
            actions: PlayerInfoAction::initializing_actions(),
            entries,
        }
    }

    pub fn single_action(action: PlayerInfoAction, entries: Vec<PlayerListEntry>) -> Self {
        Self {
            actions: vec![action],
            entries,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoRemovePacket {
    pub profile_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginDecision {
    Accepted {
        self_initializing: PlayerInfoUpdatePacket,
        broadcast_new_player: PlayerInfoUpdatePacket,
        duplicate_disconnects: Vec<DisconnectPlan>,
        send_status: bool,
    },
    Rejected {
        translation_key: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespawnPlan {
    pub profile_id: Uuid,
    pub keep_all_player_data: bool,
    pub removal_reason: RemovalReason,
    pub missing_respawn_block: bool,
    pub data_to_keep: u8,
    pub copy_respawn_position: bool,
    pub packets: Vec<RespawnPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    Killed,
    ChangedDimension,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespawnPacket {
    NoRespawnBlockAvailable,
    Respawn {
        dimension: Identifier,
        game_mode: GameMode,
    },
    TeleportToRespawnPosition,
    SetDefaultSpawnPosition,
    ChangeDifficulty,
    SetExperience,
    SendActiveEffects,
    SendLevelInfo,
    SendPermissionLevel,
    InitInventoryMenu,
    RespawnAnchorDepleteSound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferDecision {
    Transfer { host: String, port: u16 },
    Rejected { translation_key: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectPlan {
    pub profile_id: Uuid,
    pub translation_key: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabListUpdate {
    pub profile_id: Uuid,
    pub actions: Vec<PlayerInfoAction>,
    pub packet: PlayerInfoUpdatePacket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListModel {
    players: Vec<PlayerListEntry>,
    players_by_uuid: BTreeMap<UuidKey, PlayerListEntry>,
    login_queue: VecDeque<PlayerProfile>,
    whitelist_enabled: bool,
    whitelist: BTreeSet<UuidKey>,
    banned_profiles: BTreeSet<UuidKey>,
    banned_ips: BTreeSet<String>,
    max_players: usize,
    accepts_transfers: bool,
}

impl PlayerListModel {
    pub fn new(max_players: usize) -> Self {
        Self {
            players: Vec::new(),
            players_by_uuid: BTreeMap::new(),
            login_queue: VecDeque::new(),
            whitelist_enabled: false,
            whitelist: BTreeSet::new(),
            banned_profiles: BTreeSet::new(),
            banned_ips: BTreeSet::new(),
            max_players,
            accepts_transfers: false,
        }
    }

    pub fn set_whitelist_enabled(&mut self, enabled: bool) {
        self.whitelist_enabled = enabled;
    }

    pub fn set_accepts_transfers(&mut self, accepts_transfers: bool) {
        self.accepts_transfers = accepts_transfers;
    }

    pub fn allow_uuid(&mut self, uuid: Uuid) {
        self.whitelist.insert(UuidKey::from(uuid));
    }

    pub fn ban_uuid(&mut self, uuid: Uuid) {
        self.banned_profiles.insert(UuidKey::from(uuid));
    }

    pub fn ban_ip(&mut self, ip: impl Into<String>) {
        self.banned_ips.insert(ip.into());
    }

    pub fn enqueue_login(&mut self, profile: PlayerProfile) {
        self.login_queue.push_back(profile);
    }

    pub fn queued_profiles(&self) -> impl Iterator<Item = &PlayerProfile> {
        self.login_queue.iter()
    }

    pub fn process_next_login(
        &mut self,
        game_mode: GameMode,
        remote_ip: &str,
        transferred_cookie: bool,
    ) -> Option<LoginDecision> {
        let profile = self.login_queue.pop_front()?;
        Some(self.login(profile, game_mode, remote_ip, transferred_cookie))
    }

    pub fn login(
        &mut self,
        profile: PlayerProfile,
        game_mode: GameMode,
        remote_ip: &str,
        transferred_cookie: bool,
    ) -> LoginDecision {
        let key = UuidKey::from(profile.uuid);
        if self.banned_profiles.contains(&key) {
            return LoginDecision::Rejected {
                translation_key: BANNED_DISCONNECT,
            };
        }
        if self.banned_ips.contains(remote_ip) {
            return LoginDecision::Rejected {
                translation_key: BANNED_IP_DISCONNECT,
            };
        }
        if self.whitelist_enabled && !self.whitelist.contains(&key) {
            return LoginDecision::Rejected {
                translation_key: NOT_WHITELISTED_DISCONNECT,
            };
        }

        let duplicate_disconnects = self.disconnect_all_players_with_profile(profile.uuid);
        if self.players.len() >= self.max_players {
            return LoginDecision::Rejected {
                translation_key: SERVER_FULL_DISCONNECT,
            };
        }

        let mut entry = PlayerListEntry::new(profile, game_mode);
        entry.transferred = transferred_cookie;
        let self_initializing = PlayerInfoUpdatePacket::initializing(self.players.clone());
        self.players.push(entry.clone());
        self.players_by_uuid.insert(key, entry.clone());
        let broadcast_new_player = PlayerInfoUpdatePacket::initializing(vec![entry]);

        LoginDecision::Accepted {
            self_initializing,
            broadcast_new_player,
            duplicate_disconnects,
            send_status: !transferred_cookie,
        }
    }

    pub fn disconnect_all_players_with_profile(&mut self, uuid: Uuid) -> Vec<DisconnectPlan> {
        let key = UuidKey::from(uuid);
        let mut disconnected = Vec::new();
        self.players.retain(|player| {
            if player.profile.uuid == uuid {
                disconnected.push(DisconnectPlan {
                    profile_id: player.profile.uuid,
                    translation_key: DUPLICATE_LOGIN_DISCONNECT,
                });
                false
            } else {
                true
            }
        });
        if self.players_by_uuid.remove(&key).is_some()
            && !disconnected.iter().any(|plan| plan.profile_id == uuid)
        {
            disconnected.push(DisconnectPlan {
                profile_id: uuid,
                translation_key: DUPLICATE_LOGIN_DISCONNECT,
            });
        }
        disconnected
    }

    pub fn remove(&mut self, uuid: Uuid) -> Option<PlayerInfoRemovePacket> {
        let before = self.players.len();
        self.players.retain(|player| player.profile.uuid != uuid);
        self.players_by_uuid.remove(&UuidKey::from(uuid));
        (self.players.len() != before).then_some(PlayerInfoRemovePacket {
            profile_ids: vec![uuid],
        })
    }

    pub fn shutdown_disconnects(&self) -> Vec<DisconnectPlan> {
        self.players
            .iter()
            .map(|player| DisconnectPlan {
                profile_id: player.profile.uuid,
                translation_key: SERVER_SHUTDOWN_DISCONNECT,
            })
            .collect()
    }

    pub fn broadcast_latency(&self) -> PlayerInfoUpdatePacket {
        PlayerInfoUpdatePacket::single_action(PlayerInfoAction::UpdateLatency, self.players.clone())
    }

    pub fn update_tab_entry<F>(&mut self, uuid: Uuid, mut update: F) -> Option<TabListUpdate>
    where
        F: FnMut(&mut PlayerListEntry) -> Vec<PlayerInfoAction>,
    {
        let player = self
            .players
            .iter_mut()
            .find(|player| player.profile.uuid == uuid)?;
        let actions = update(player);
        if actions.is_empty() {
            return None;
        }
        let packet = PlayerInfoUpdatePacket {
            actions: actions.clone(),
            entries: vec![player.clone()],
        };
        self.players_by_uuid
            .insert(UuidKey::from(uuid), player.clone());
        Some(TabListUpdate {
            profile_id: uuid,
            actions,
            packet,
        })
    }

    pub fn plan_respawn(
        &mut self,
        uuid: Uuid,
        keep_all_player_data: bool,
        removal_reason: RemovalReason,
        missing_respawn_block: bool,
        anchor_depleted: bool,
        dimension: Identifier,
        game_mode: GameMode,
    ) -> Option<RespawnPlan> {
        self.players
            .iter()
            .find(|player| player.profile.uuid == uuid)?;
        let mut packets = Vec::new();
        if missing_respawn_block {
            packets.push(RespawnPacket::NoRespawnBlockAvailable);
        }
        packets.extend([
            RespawnPacket::Respawn {
                dimension,
                game_mode,
            },
            RespawnPacket::TeleportToRespawnPosition,
            RespawnPacket::SetDefaultSpawnPosition,
            RespawnPacket::ChangeDifficulty,
            RespawnPacket::SetExperience,
            RespawnPacket::SendActiveEffects,
            RespawnPacket::SendLevelInfo,
            RespawnPacket::SendPermissionLevel,
            RespawnPacket::InitInventoryMenu,
        ]);
        if anchor_depleted && !keep_all_player_data {
            packets.push(RespawnPacket::RespawnAnchorDepleteSound);
        }

        Some(RespawnPlan {
            profile_id: uuid,
            keep_all_player_data,
            removal_reason,
            missing_respawn_block,
            data_to_keep: u8::from(keep_all_player_data),
            copy_respawn_position: !missing_respawn_block,
            packets,
        })
    }

    pub fn transfer(&self, host: impl Into<String>, port: u16) -> TransferDecision {
        if self.accepts_transfers {
            TransferDecision::Transfer {
                host: host.into(),
                port,
            }
        } else {
            TransferDecision::Rejected {
                translation_key: "commands.transfer.error",
            }
        }
    }

    pub fn players(&self) -> &[PlayerListEntry] {
        &self.players
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UuidKey([u8; 16]);

impl From<Uuid> for UuidKey {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LoginDecision, PlayerInfoAction, PlayerListModel, PlayerProfile, ProfileProperty,
        RemovalReason, RespawnPacket, TransferDecision, BANNED_DISCONNECT,
        DUPLICATE_LOGIN_DISCONNECT, NOT_WHITELISTED_DISCONNECT, SERVER_FULL_DISCONNECT,
        SERVER_SHUTDOWN_DISCONNECT,
    };
    use crate::network::codec::Uuid;
    use crate::network::play::GameMode;
    use crate::registry::Identifier;

    fn uuid(byte: u8) -> Uuid {
        Uuid([byte; 16])
    }

    fn profile(byte: u8, name: &str) -> PlayerProfile {
        PlayerProfile {
            uuid: uuid(byte),
            name: name.to_string(),
            properties: Vec::new(),
        }
    }

    #[test]
    fn initializing_actions_match_vanilla_order() {
        assert_eq!(
            PlayerInfoAction::initializing_actions(),
            vec![
                PlayerInfoAction::AddPlayer,
                PlayerInfoAction::InitializeChat,
                PlayerInfoAction::UpdateGameMode,
                PlayerInfoAction::UpdateListed,
                PlayerInfoAction::UpdateLatency,
                PlayerInfoAction::UpdateDisplayName,
                PlayerInfoAction::UpdateHat,
                PlayerInfoAction::UpdateListOrder,
            ]
        );
    }

    #[test]
    fn accepted_login_sends_existing_players_to_joiner_then_broadcasts_new_player() {
        let mut list = PlayerListModel::new(10);
        let first = list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false);
        assert!(matches!(
            first,
            LoginDecision::Accepted {
                send_status: true,
                ..
            }
        ));

        let second = list.login(profile(2, "Beta"), GameMode::Creative, "127.0.0.1", false);
        let LoginDecision::Accepted {
            self_initializing,
            broadcast_new_player,
            duplicate_disconnects,
            send_status,
        } = second
        else {
            panic!("second login should be accepted");
        };

        assert!(send_status);
        assert!(duplicate_disconnects.is_empty());
        assert_eq!(self_initializing.entries.len(), 1);
        assert_eq!(self_initializing.entries[0].profile.name, "Alpha");
        assert_eq!(broadcast_new_player.entries.len(), 1);
        assert_eq!(broadcast_new_player.entries[0].profile.name, "Beta");
        assert_eq!(list.players().len(), 2);
    }

    #[test]
    fn duplicate_login_disconnects_existing_profile_before_accepting_replacement() {
        let mut list = PlayerListModel::new(10);
        list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false);

        let replacement = list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false);
        let LoginDecision::Accepted {
            duplicate_disconnects,
            self_initializing,
            ..
        } = replacement
        else {
            panic!("replacement login should be accepted");
        };

        assert_eq!(duplicate_disconnects.len(), 1);
        assert_eq!(duplicate_disconnects[0].profile_id, uuid(1));
        assert_eq!(
            duplicate_disconnects[0].translation_key,
            DUPLICATE_LOGIN_DISCONNECT
        );
        assert!(self_initializing.entries.is_empty());
        assert_eq!(list.players().len(), 1);
    }

    #[test]
    fn whitelist_ban_and_full_server_rejections_use_vanilla_keys() {
        let mut list = PlayerListModel::new(1);
        list.set_whitelist_enabled(true);
        assert_eq!(
            list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false),
            LoginDecision::Rejected {
                translation_key: NOT_WHITELISTED_DISCONNECT
            }
        );

        list.allow_uuid(uuid(1));
        list.ban_uuid(uuid(1));
        assert_eq!(
            list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false),
            LoginDecision::Rejected {
                translation_key: BANNED_DISCONNECT
            }
        );

        let mut full = PlayerListModel::new(1);
        full.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false);
        assert_eq!(
            full.login(profile(2, "Beta"), GameMode::Survival, "127.0.0.1", false),
            LoginDecision::Rejected {
                translation_key: SERVER_FULL_DISCONNECT
            }
        );
    }

    #[test]
    fn login_queue_is_fifo_and_transfer_cookie_suppresses_status_send() {
        let mut list = PlayerListModel::new(10);
        list.enqueue_login(profile(1, "Alpha"));
        list.enqueue_login(profile(2, "Beta"));

        let first = list.process_next_login(GameMode::Survival, "127.0.0.1", true);
        assert!(matches!(
            first,
            Some(LoginDecision::Accepted {
                send_status: false,
                ..
            })
        ));
        let second = list.process_next_login(GameMode::Survival, "127.0.0.1", false);
        assert!(matches!(
            second,
            Some(LoginDecision::Accepted {
                send_status: true,
                ..
            })
        ));
        assert!(list
            .process_next_login(GameMode::Survival, "127.0.0.1", false)
            .is_none());
        assert_eq!(list.players()[0].profile.name, "Alpha");
        assert_eq!(list.players()[1].profile.name, "Beta");
    }

    #[test]
    fn remove_latency_and_shutdown_packets_target_current_players() {
        let mut list = PlayerListModel::new(10);
        list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false);
        list.login(profile(2, "Beta"), GameMode::Survival, "127.0.0.1", false);

        let latency = list.broadcast_latency();
        assert_eq!(latency.actions, vec![PlayerInfoAction::UpdateLatency]);
        assert_eq!(latency.entries.len(), 2);

        let remove = list.remove(uuid(1)).expect("remove packet");
        assert_eq!(remove.profile_ids, vec![uuid(1)]);
        assert_eq!(list.players().len(), 1);

        assert_eq!(
            list.shutdown_disconnects()[0].translation_key,
            SERVER_SHUTDOWN_DISCONNECT
        );
    }

    #[test]
    fn tab_list_updates_report_precise_changed_actions() {
        let mut list = PlayerListModel::new(10);
        list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false);

        let update = list
            .update_tab_entry(uuid(1), |entry| {
                entry.latency = 42;
                entry.game_mode = GameMode::Adventure;
                entry.display_name = Some("{\"text\":\"A\"}".to_string());
                entry.listed = false;
                entry.show_hat = false;
                entry.list_order = 7;
                entry.profile.properties.push(ProfileProperty {
                    name: "textures".to_string(),
                    value: "value".to_string(),
                    signature: Some("sig".to_string()),
                });
                vec![
                    PlayerInfoAction::UpdateLatency,
                    PlayerInfoAction::UpdateGameMode,
                    PlayerInfoAction::UpdateDisplayName,
                    PlayerInfoAction::UpdateListed,
                    PlayerInfoAction::UpdateHat,
                    PlayerInfoAction::UpdateListOrder,
                    PlayerInfoAction::AddPlayer,
                ]
            })
            .expect("tab update");

        assert_eq!(update.packet.entries[0].latency, 42);
        assert_eq!(update.packet.entries[0].game_mode, GameMode::Adventure);
        assert_eq!(update.packet.entries[0].profile.properties.len(), 1);
        assert_eq!(update.actions[0], PlayerInfoAction::UpdateLatency);
    }

    #[test]
    fn respawn_plan_preserves_vanilla_packet_order_and_flags() {
        let mut list = PlayerListModel::new(10);
        list.login(profile(1, "Alpha"), GameMode::Survival, "127.0.0.1", false);
        let dimension = Identifier::parse("minecraft:overworld").unwrap();

        let plan = list
            .plan_respawn(
                uuid(1),
                false,
                RemovalReason::Killed,
                true,
                true,
                dimension.clone(),
                GameMode::Survival,
            )
            .expect("respawn plan");

        assert_eq!(plan.data_to_keep, 0);
        assert!(!plan.copy_respawn_position);
        assert_eq!(plan.packets[0], RespawnPacket::NoRespawnBlockAvailable);
        assert_eq!(
            plan.packets[1],
            RespawnPacket::Respawn {
                dimension,
                game_mode: GameMode::Survival
            }
        );
        assert!(plan
            .packets
            .contains(&RespawnPacket::RespawnAnchorDepleteSound));
    }

    #[test]
    fn transfer_policy_matches_accepts_transfers_gate() {
        let mut list = PlayerListModel::new(10);
        assert_eq!(
            list.transfer("example.org", 25565),
            TransferDecision::Rejected {
                translation_key: "commands.transfer.error"
            }
        );

        list.set_accepts_transfers(true);
        assert_eq!(
            list.transfer("example.org", 25565),
            TransferDecision::Transfer {
                host: "example.org".to_string(),
                port: 25565
            }
        );
    }
}
