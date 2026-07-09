#![allow(dead_code)]

use crate::chat_component::Component;
use crate::jsonrpc_api::JsonRpcPlayerDto;
use crate::jsonrpc_ip_banlist_service::{JsonRpcIpBan, JsonRpcIpBanlist};
use crate::jsonrpc_methods::ClientInfo;

pub const MANAGEMENT_BAN_SOURCE: &str = "Management server";
pub const USER_BANNED_MESSAGE_KEY: &str = "multiplayer.disconnect.banned";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcUserBan {
    pub player: JsonRpcPlayerDto,
    pub reason: Option<String>,
    pub source: String,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcUserBanDto {
    pub player: JsonRpcPlayerDto,
    pub reason: Option<String>,
    pub source: Option<String>,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcBanlist {
    pub entries: Vec<JsonRpcUserBan>,
    pub events: Vec<JsonRpcBanlistEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcBanUserDirectory {
    pub users: Vec<JsonRpcPlayerDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcOnlineBanPlayers {
    pub players: Vec<JsonRpcPlayerDto>,
    pub disconnects: Vec<JsonRpcBanDisconnect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcBanDisconnect {
    pub player: JsonRpcPlayerDto,
    pub message: Component,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcBanlistEvent {
    AddUserBan {
        ban: JsonRpcUserBan,
        client_info: ClientInfo,
    },
    ClearUserBans {
        client_info: ClientInfo,
    },
    RemoveUserBan {
        user: JsonRpcPlayerDto,
        client_info: ClientInfo,
    },
}

pub struct BanlistService;

pub trait MinecraftBanListService {
    fn add_user_ban(&mut self, ban: JsonRpcUserBan, client_info: ClientInfo);
    fn remove_user_ban(&mut self, name_and_id: &JsonRpcPlayerDto, client_info: ClientInfo);
    fn get_user_ban_entries(&self) -> Vec<JsonRpcUserBan>;
    fn get_ip_ban_entries(&self) -> Vec<JsonRpcIpBan>;
    fn add_ip_ban(&mut self, ip_ban_entry: JsonRpcIpBan, client_info: ClientInfo);
    fn clear_ip_bans(&mut self, client_info: ClientInfo);
    fn remove_ip_ban(&mut self, ip: &str, client_info: ClientInfo);
    fn clear_user_bans(&mut self, client_info: ClientInfo);
}

/// The two vanilla `StoredUserList` instances exposed through Java's single
/// internal ban-list service interface.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MinecraftBanLists {
    pub user_bans: JsonRpcBanlist,
    pub ip_bans: JsonRpcIpBanlist,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftBanListServiceImplModel {
    pub server_ban_lists: MinecraftBanLists,
    pub log_messages: Vec<(ClientInfo, String)>,
}

impl MinecraftBanListServiceImplModel {
    pub const fn new(server_ban_lists: MinecraftBanLists) -> Self {
        Self {
            server_ban_lists,
            log_messages: Vec::new(),
        }
    }

    fn log(&mut self, client_info: ClientInfo, message: String) {
        self.log_messages.push((client_info, message));
    }
}

impl MinecraftBanListService for MinecraftBanListServiceImplModel {
    fn add_user_ban(&mut self, ban: JsonRpcUserBan, client_info: ClientInfo) {
        let display_name = ban.player.name.as_deref().unwrap_or("(Unknown)");
        let reason = ban
            .reason
            .as_deref()
            .unwrap_or("Banned by an operator.");
        self.log(
            client_info,
            format!("Add player '{display_name}' to banlist. Reason: '{reason}'"),
        );
        self.server_ban_lists.add_user_ban(ban, client_info);
    }

    fn remove_user_ban(&mut self, name_and_id: &JsonRpcPlayerDto, client_info: ClientInfo) {
        self.log(
            client_info,
            format!(
                "Remove player '{}' from banlist",
                name_and_id_display(name_and_id)
            ),
        );
        self.server_ban_lists
            .remove_user_ban(name_and_id, client_info);
    }

    fn get_user_ban_entries(&self) -> Vec<JsonRpcUserBan> {
        self.server_ban_lists.get_user_ban_entries()
    }

    fn get_ip_ban_entries(&self) -> Vec<JsonRpcIpBan> {
        self.server_ban_lists.get_ip_ban_entries()
    }

    fn add_ip_ban(&mut self, ip_ban_entry: JsonRpcIpBan, client_info: ClientInfo) {
        self.log(
            client_info,
            format!("Add ip '{}' to ban list", ip_ban_entry.ip),
        );
        self.server_ban_lists
            .add_ip_ban(ip_ban_entry, client_info);
    }

    fn clear_ip_bans(&mut self, client_info: ClientInfo) {
        self.log(client_info, "Clear ip ban list".to_string());
        self.server_ban_lists.clear_ip_bans(client_info);
    }

    fn remove_ip_ban(&mut self, ip: &str, client_info: ClientInfo) {
        self.log(client_info, format!("Remove ip '{ip}' from ban list"));
        self.server_ban_lists.remove_ip_ban(ip, client_info);
    }

    fn clear_user_bans(&mut self, client_info: ClientInfo) {
        // Java intentionally performs this one mutation without a JSON-RPC log.
        self.server_ban_lists.clear_user_bans(client_info);
    }
}

fn name_and_id_display(player: &JsonRpcPlayerDto) -> String {
    let id = player.id.as_deref().unwrap_or("null");
    let name = player.name.as_deref().unwrap_or("null");
    format!("NameAndId[id={id}, name={name}]")
}

impl MinecraftBanLists {
    pub const fn new(user_bans: JsonRpcBanlist, ip_bans: JsonRpcIpBanlist) -> Self {
        Self {
            user_bans,
            ip_bans,
        }
    }
}

impl MinecraftBanListService for MinecraftBanLists {
    fn add_user_ban(&mut self, ban: JsonRpcUserBan, client_info: ClientInfo) {
        self.user_bans.add_user_ban(ban, client_info);
    }

    fn remove_user_ban(&mut self, name_and_id: &JsonRpcPlayerDto, client_info: ClientInfo) {
        self.user_bans.remove_user_ban(name_and_id, client_info);
    }

    fn get_user_ban_entries(&self) -> Vec<JsonRpcUserBan> {
        self.user_bans.entries.clone()
    }

    fn get_ip_ban_entries(&self) -> Vec<JsonRpcIpBan> {
        self.ip_bans.entries.clone()
    }

    fn add_ip_ban(&mut self, ip_ban_entry: JsonRpcIpBan, client_info: ClientInfo) {
        self.ip_bans.add_ip_ban(ip_ban_entry, client_info);
    }

    fn clear_ip_bans(&mut self, client_info: ClientInfo) {
        self.ip_bans.clear_ip_bans(client_info);
    }

    fn remove_ip_ban(&mut self, ip: &str, client_info: ClientInfo) {
        self.ip_bans.remove_ip_ban(ip, client_info);
    }

    fn clear_user_bans(&mut self, client_info: ClientInfo) {
        self.user_bans.clear_user_bans(client_info);
    }
}

impl JsonRpcUserBan {
    fn to_dto(&self) -> JsonRpcUserBanDto {
        JsonRpcUserBanDto {
            player: self.player.clone(),
            reason: self.reason.clone(),
            source: Some(self.source.clone()),
            expires: self.expires.clone(),
        }
    }
}

impl JsonRpcUserBanDto {
    pub fn new(
        player: JsonRpcPlayerDto,
        reason: Option<String>,
        source: Option<String>,
        expires: Option<String>,
    ) -> Self {
        Self {
            player,
            reason,
            source,
            expires,
        }
    }

    fn to_user_ban(&self, player: JsonRpcPlayerDto) -> JsonRpcUserBan {
        JsonRpcUserBan {
            player,
            reason: self.reason.clone(),
            source: match &self.source {
                Some(source) => source.clone(),
                None => MANAGEMENT_BAN_SOURCE.to_string(),
            },
            expires: self.expires.clone(),
        }
    }
}

impl JsonRpcBanlist {
    pub fn new(entries: Vec<JsonRpcUserBan>) -> Self {
        Self {
            entries,
            events: Vec::new(),
        }
    }

    fn add_user_ban(&mut self, ban: JsonRpcUserBan, client_info: ClientInfo) {
        let previous = self
            .entries
            .iter()
            .position(|entry| same_ban_player(&entry.player, &ban.player));
        if let Some(index) = previous {
            if self.entries[index] != ban {
                self.entries[index] = ban.clone();
            }
        } else {
            self.entries.push(ban.clone());
        }
        self.events
            .push(JsonRpcBanlistEvent::AddUserBan { ban, client_info });
    }

    fn clear_user_bans(&mut self, client_info: ClientInfo) {
        self.entries.clear();
        self.events
            .push(JsonRpcBanlistEvent::ClearUserBans { client_info });
    }

    fn remove_user_ban(&mut self, user: &JsonRpcPlayerDto, client_info: ClientInfo) {
        self.entries.retain(|ban| ban.player != *user);
        self.events.push(JsonRpcBanlistEvent::RemoveUserBan {
            user: user.clone(),
            client_info,
        });
    }
}

impl JsonRpcBanUserDirectory {
    pub fn new(users: Vec<JsonRpcPlayerDto>) -> Self {
        Self { users }
    }

    fn get_user(&self, player: &JsonRpcPlayerDto) -> Option<JsonRpcPlayerDto> {
        if let Some(id) = &player.id {
            self.users
                .iter()
                .find(|user| user.id.as_ref() == Some(id))
                .cloned()
        } else {
            player.name.as_ref().and_then(|name| {
                self.users
                    .iter()
                    .find(|user| user.name.as_ref() == Some(name))
                    .cloned()
            })
        }
    }
}

impl JsonRpcOnlineBanPlayers {
    pub fn new(players: Vec<JsonRpcPlayerDto>) -> Self {
        Self {
            players,
            disconnects: Vec::new(),
        }
    }

    fn disconnect_banned_player(&mut self, player: &JsonRpcPlayerDto) {
        let Some(id) = &player.id else {
            return;
        };
        if let Some(online_player) = self
            .players
            .iter()
            .find(|online_player| online_player.id.as_ref() == Some(id))
        {
            self.disconnects.push(JsonRpcBanDisconnect {
                player: online_player.clone(),
                message: Component::translatable(USER_BANNED_MESSAGE_KEY, Vec::new()),
            });
        }
    }
}

impl BanlistService {
    pub fn get(banlist: &JsonRpcBanlist) -> Vec<JsonRpcUserBanDto> {
        banlist.entries.iter().map(JsonRpcUserBan::to_dto).collect()
    }

    pub fn add(
        users: &JsonRpcBanUserDirectory,
        online_players: &mut JsonRpcOnlineBanPlayers,
        banlist: &mut JsonRpcBanlist,
        bans: &[JsonRpcUserBanDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcUserBanDto> {
        for ban in bans {
            if let Some(user) = users.get_user(&ban.player) {
                let user_ban = ban.to_user_ban(user);
                banlist.add_user_ban(user_ban.clone(), client_info);
                online_players.disconnect_banned_player(&user_ban.player);
            }
        }

        Self::get(banlist)
    }

    pub fn clear(
        banlist: &mut JsonRpcBanlist,
        client_info: ClientInfo,
    ) -> Vec<JsonRpcUserBanDto> {
        banlist.clear_user_bans(client_info);
        Self::get(banlist)
    }

    pub fn remove(
        users: &JsonRpcBanUserDirectory,
        banlist: &mut JsonRpcBanlist,
        remove: &[JsonRpcPlayerDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcUserBanDto> {
        for player_dto in remove {
            if let Some(user) = users.get_user(player_dto) {
                banlist.remove_user_ban(&user, client_info);
            }
        }

        Self::get(banlist)
    }

    pub fn set(
        users: &JsonRpcBanUserDirectory,
        online_players: &mut JsonRpcOnlineBanPlayers,
        banlist: &mut JsonRpcBanlist,
        bans: &[JsonRpcUserBanDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcUserBanDto> {
        let final_bans = resolved_unique_bans(users, bans);
        let current_bans = unique_bans(&banlist.entries);

        for current in &current_bans {
            if !final_bans.contains(current) {
                banlist.remove_user_ban(&current.player, client_info);
            }
        }
        for final_ban in final_bans {
            if !current_bans.contains(&final_ban) {
                banlist.add_user_ban(final_ban.clone(), client_info);
                online_players.disconnect_banned_player(&final_ban.player);
            }
        }

        Self::get(banlist)
    }
}

fn resolved_unique_bans(
    users: &JsonRpcBanUserDirectory,
    bans: &[JsonRpcUserBanDto],
) -> Vec<JsonRpcUserBan> {
    let mut resolved = Vec::new();
    for ban in bans {
        if let Some(user) = users.get_user(&ban.player) {
            let user_ban = ban.to_user_ban(user);
            if !resolved.contains(&user_ban) {
                resolved.push(user_ban);
            }
        }
    }
    resolved
}

fn unique_bans(bans: &[JsonRpcUserBan]) -> Vec<JsonRpcUserBan> {
    let mut unique = Vec::new();
    for ban in bans {
        if !unique.contains(ban) {
            unique.push(ban.clone());
        }
    }
    unique
}

fn same_ban_player(left: &JsonRpcPlayerDto, right: &JsonRpcPlayerDto) -> bool {
    match (&left.id, &right.id) {
        (Some(left_id), Some(right_id)) => left_id == right_id,
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_add_and_clear_match_java_user_ban_dto_mapping() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let users = JsonRpcBanUserDirectory::new(vec![steve.clone()]);
        let mut online = JsonRpcOnlineBanPlayers::new(vec![steve.clone()]);
        let mut banlist = JsonRpcBanlist::default();

        let result = BanlistService::add(
            &users,
            &mut online,
            &mut banlist,
            &[JsonRpcUserBanDto::new(
                JsonRpcPlayerDto::new(None, Some("Steve".to_string())),
                Some("bad".to_string()),
                None,
                Some("2026-06-27T00:00:00Z".to_string()),
            )],
            ClientInfo::of(20),
        );

        assert_eq!(
            result,
            vec![JsonRpcUserBanDto::new(
                steve.clone(),
                Some("bad".to_string()),
                Some(MANAGEMENT_BAN_SOURCE.to_string()),
                Some("2026-06-27T00:00:00Z".to_string()),
            )]
        );
        assert_eq!(
            online.disconnects,
            vec![JsonRpcBanDisconnect {
                player: steve.clone(),
                message: Component::translatable(USER_BANNED_MESSAGE_KEY, Vec::new()),
            }]
        );
        assert_eq!(
            BanlistService::clear(&mut banlist, ClientInfo::of(21)),
            Vec::<JsonRpcUserBanDto>::new()
        );
    }

    #[test]
    fn remove_resolves_requested_players_before_mutation() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex");
        let users = JsonRpcBanUserDirectory::new(vec![steve.clone(), alex.clone()]);
        let mut banlist = JsonRpcBanlist::new(vec![
            ban(steve.clone(), None, "Console", None),
            ban(alex.clone(), Some("griefing"), "Console", None),
        ]);

        let result = BanlistService::remove(
            &users,
            &mut banlist,
            &[JsonRpcPlayerDto::new(
                Some("11111111-1111-1111-1111-111111111111".to_string()),
                Some("WrongName".to_string()),
            )],
            ClientInfo::of(22),
        );

        assert_eq!(
            result,
            vec![JsonRpcUserBanDto::new(
                alex,
                Some("griefing".to_string()),
                Some("Console".to_string()),
                None,
            )]
        );
        assert_eq!(
            banlist.events,
            vec![JsonRpcBanlistEvent::RemoveUserBan {
                user: steve,
                client_info: ClientInfo::of(22),
            }]
        );
    }

    #[test]
    fn add_replaces_existing_entry_for_same_player_key() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let users = JsonRpcBanUserDirectory::new(vec![steve.clone()]);
        let mut online = JsonRpcOnlineBanPlayers::new(vec![steve.clone()]);
        let mut banlist = JsonRpcBanlist::new(vec![ban(steve.clone(), Some("old"), "Console", None)]);

        let result = BanlistService::add(
            &users,
            &mut online,
            &mut banlist,
            &[JsonRpcUserBanDto::new(
                JsonRpcPlayerDto::new(
                    Some("11111111-1111-1111-1111-111111111111".to_string()),
                    Some("Renamed".to_string()),
                ),
                Some("new".to_string()),
                Some("Console".to_string()),
                None,
            )],
            ClientInfo::of(24),
        );

        assert_eq!(
            result,
            vec![JsonRpcUserBanDto::new(
                steve,
                Some("new".to_string()),
                Some("Console".to_string()),
                None,
            )]
        );
        assert_eq!(banlist.entries.len(), 1);
    }

    #[test]
    fn set_matches_java_user_ban_record_set_difference_and_disconnects_new_bans() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex");
        let users = JsonRpcBanUserDirectory::new(vec![steve.clone(), alex.clone()]);
        let mut online = JsonRpcOnlineBanPlayers::new(vec![steve.clone(), alex.clone()]);
        let mut banlist = JsonRpcBanlist::new(vec![ban(
            alex.clone(),
            Some("old"),
            "Console",
            None,
        )]);

        let result = BanlistService::set(
            &users,
            &mut online,
            &mut banlist,
            &[
                JsonRpcUserBanDto::new(
                    JsonRpcPlayerDto::new(
                        Some("22222222-2222-2222-2222-222222222222".to_string()),
                        Some("Alex".to_string()),
                    ),
                    Some("new".to_string()),
                    Some("Console".to_string()),
                    None,
                ),
                JsonRpcUserBanDto::new(
                    JsonRpcPlayerDto::new(None, Some("Steve".to_string())),
                    None,
                    None,
                    None,
                ),
            ],
            ClientInfo::of(23),
        );

        assert_eq!(
            result,
            vec![
                JsonRpcUserBanDto::new(
                    alex.clone(),
                    Some("new".to_string()),
                    Some("Console".to_string()),
                    None,
                ),
                JsonRpcUserBanDto::new(
                    steve.clone(),
                    None,
                    Some(MANAGEMENT_BAN_SOURCE.to_string()),
                    None,
                ),
            ]
        );
        assert_eq!(
            banlist.events,
            vec![
                JsonRpcBanlistEvent::RemoveUserBan {
                    user: alex.clone(),
                    client_info: ClientInfo::of(23),
                },
                JsonRpcBanlistEvent::AddUserBan {
                    ban: ban(alex.clone(), Some("new"), "Console", None),
                    client_info: ClientInfo::of(23),
                },
                JsonRpcBanlistEvent::AddUserBan {
                    ban: ban(steve.clone(), None, MANAGEMENT_BAN_SOURCE, None),
                    client_info: ClientInfo::of(23),
                },
            ]
        );
        assert_eq!(
            online.disconnects,
            vec![
                JsonRpcBanDisconnect {
                    player: alex,
                    message: Component::translatable(USER_BANNED_MESSAGE_KEY, Vec::new()),
                },
                JsonRpcBanDisconnect {
                    player: steve,
                    message: Component::translatable(USER_BANNED_MESSAGE_KEY, Vec::new()),
                },
            ]
        );
    }

    #[test]
    fn minecraft_ban_list_service_unifies_user_and_ip_stores() {
        use crate::jsonrpc_ip_banlist_service::JsonRpcIpBanlistEvent;

        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex");
        let old_user_ban = ban(steve.clone(), Some("old"), "Console", None);
        let old_ip_ban = ip_ban("192.0.2.10", Some("old"));
        let mut service = MinecraftBanLists::new(
            JsonRpcBanlist::new(vec![old_user_ban.clone()]),
            JsonRpcIpBanlist::new(vec![old_ip_ban.clone()]),
        );
        let client_info = ClientInfo::of(31);

        assert_eq!(service.get_user_ban_entries(), vec![old_user_ban]);
        assert_eq!(service.get_ip_ban_entries(), vec![old_ip_ban]);

        let new_user_ban = ban(alex.clone(), Some("new"), "Management server", None);
        let new_ip_ban = ip_ban("192.0.2.20", Some("new"));
        service.add_user_ban(new_user_ban.clone(), client_info);
        service.add_ip_ban(new_ip_ban.clone(), client_info);
        service.remove_user_ban(&steve, client_info);
        service.remove_ip_ban("192.0.2.10", client_info);

        assert_eq!(service.get_user_ban_entries(), vec![new_user_ban.clone()]);
        assert_eq!(service.get_ip_ban_entries(), vec![new_ip_ban.clone()]);
        assert_eq!(
            service.user_bans.events,
            vec![
                JsonRpcBanlistEvent::AddUserBan {
                    ban: new_user_ban,
                    client_info,
                },
                JsonRpcBanlistEvent::RemoveUserBan {
                    user: steve,
                    client_info,
                },
            ]
        );
        assert_eq!(
            service.ip_bans.events,
            vec![
                JsonRpcIpBanlistEvent::AddIpBan {
                    ban: new_ip_ban,
                    client_info,
                },
                JsonRpcIpBanlistEvent::RemoveIpBan {
                    ip: "192.0.2.10".to_string(),
                    client_info,
                },
            ]
        );

        service.clear_user_bans(client_info);
        assert!(service.get_user_ban_entries().is_empty());
        assert!(!service.get_ip_ban_entries().is_empty());
        service.clear_ip_bans(client_info);
        assert!(service.get_ip_ban_entries().is_empty());
        assert_eq!(
            service.user_bans.events.last(),
            Some(&JsonRpcBanlistEvent::ClearUserBans { client_info })
        );
        assert_eq!(
            service.ip_bans.events.last(),
            Some(&JsonRpcIpBanlistEvent::ClearIpBans { client_info })
        );
    }

    #[test]
    fn minecraft_ban_list_service_impl_logs_and_delegates_like_java() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let user_ban = ban(steve.clone(), None, "Console", None);
        let first_ip_ban = ip_ban("192.0.2.10", Some("proxy abuse"));
        let second_ip_ban = ip_ban("192.0.2.20", None);
        let client_info = ClientInfo::of(32);
        let mut service = MinecraftBanListServiceImplModel::new(MinecraftBanLists::new(
            JsonRpcBanlist::default(),
            JsonRpcIpBanlist::new(vec![first_ip_ban.clone()]),
        ));

        service.add_user_ban(user_ban.clone(), client_info);
        service.add_ip_ban(second_ip_ban.clone(), client_info);
        assert_eq!(service.get_user_ban_entries(), vec![user_ban]);
        assert_eq!(
            service.get_ip_ban_entries(),
            vec![first_ip_ban, second_ip_ban]
        );
        service.remove_user_ban(&steve, client_info);
        service.remove_ip_ban("192.0.2.10", client_info);
        service.clear_user_bans(client_info);
        service.clear_ip_bans(client_info);

        assert!(service.get_user_ban_entries().is_empty());
        assert!(service.get_ip_ban_entries().is_empty());
        assert_eq!(
            service.log_messages,
            vec![
                (
                    client_info,
                    "Add player 'Steve' to banlist. Reason: 'Banned by an operator.'".to_string(),
                ),
                (
                    client_info,
                    "Add ip '192.0.2.20' to ban list".to_string(),
                ),
                (
                    client_info,
                    "Remove player 'NameAndId[id=11111111-1111-1111-1111-111111111111, name=Steve]' from banlist".to_string(),
                ),
                (
                    client_info,
                    "Remove ip '192.0.2.10' from ban list".to_string(),
                ),
                (client_info, "Clear ip ban list".to_string()),
            ]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn minecraft_ban_list_service_impl_matches_java_contract() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftBanListServiceImpl.java"
        );
        for sentinel in [
            "private final MinecraftServer server;",
            "private final JsonRpcLogger jsonrpcLogger;",
            "public MinecraftBanListServiceImpl(final MinecraftServer server, final JsonRpcLogger jsonrpcLogger)",
            "this.server = server;",
            "this.jsonrpcLogger = jsonrpcLogger;",
            "this.jsonrpcLogger.log(clientInfo, \"Add player '{}' to banlist. Reason: '{}'\", ban.getDisplayName(), ban.getReasonMessage().getString());",
            "this.server.getPlayerList().getBans().add(ban);",
            "this.jsonrpcLogger.log(clientInfo, \"Remove player '{}' from banlist\", nameAndId);",
            "this.server.getPlayerList().getBans().remove(nameAndId);",
            "this.server.getPlayerList().getBans().clear();",
            "return this.server.getPlayerList().getBans().getEntries();",
            "return this.server.getPlayerList().getIpBans().getEntries();",
            "this.jsonrpcLogger.log(clientInfo, \"Add ip '{}' to ban list\", ipBanEntry.getUser());",
            "this.server.getPlayerList().getIpBans().add(ipBanEntry);",
            "this.jsonrpcLogger.log(clientInfo, \"Clear ip ban list\");",
            "this.server.getPlayerList().getIpBans().clear();",
            "this.jsonrpcLogger.log(clientInfo, \"Remove ip '{}' from ban list\", ip);",
            "this.server.getPlayerList().getIpBans().remove(ip);",
        ] {
            assert!(
                SOURCE.contains(sentinel),
                "MinecraftBanListServiceImpl.java missing: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn minecraft_ban_list_service_interface_matches_java_contract() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftBanListService.java"
        );
        for sentinel in [
            "import java.util.Collection;",
            "import net.minecraft.server.jsonrpc.methods.ClientInfo;",
            "import net.minecraft.server.players.IpBanListEntry;",
            "import net.minecraft.server.players.NameAndId;",
            "import net.minecraft.server.players.UserBanListEntry;",
            "void addUserBan(UserBanListEntry ban, ClientInfo clientInfo);",
            "void removeUserBan(NameAndId nameAndId, ClientInfo clientInfo);",
            "Collection<UserBanListEntry> getUserBanEntries();",
            "Collection<IpBanListEntry> getIpBanEntries();",
            "void addIpBan(IpBanListEntry ipBanEntry, ClientInfo clientInfo);",
            "void clearIpBans(ClientInfo clientInfo);",
            "void removeIpBan(String ip, ClientInfo clientInfo);",
            "void clearUserBans(ClientInfo clientInfo);",
        ] {
            assert!(
                SOURCE.contains(sentinel),
                "MinecraftBanListService.java missing: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn banlist_service_source_matches_java_26_1_2() {
        const BANLIST_SERVICE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/BanlistService.java");

        for sentinel in [
            "private static final String BAN_SOURCE = \"Management server\";",
            "public static List<BanlistService.UserBanDto> get(final MinecraftApi minecraftApi)",
            "filter(p -> p.getUser() != null)",
            "map(BanlistService.UserBan::from)",
            "map(BanlistService.UserBanDto::from)",
            "getUser(banx.player().id(), banx.player().name()).thenApply(u -> u.map(banx::toUserBan))",
            "minecraftApi.banListService().addUserBan(userBan.toBanEntry(), clientInfo);",
            "ServerPlayer player = minecraftApi.playerListService().getPlayer(ban.get().player().id());",
            "player.connection.disconnect(Component.translatable(\"multiplayer.disconnect.banned\"));",
            "minecraftApi.banListService().clearUserBans(clientInfo);",
            "minecraftApi.banListService().removeUserBan(user.get(), clientInfo);",
            "Set<BanlistService.UserBan> finalAllowList = Util.sequence(fetch).join().stream().flatMap(Optional::stream).collect(Collectors.toSet());",
            "Set<BanlistService.UserBan> currentAllowList = minecraftApi.banListService()",
            "forEach(ban -> minecraftApi.banListService().removeUserBan(ban.player(), clientInfo));",
            "filter(ban -> !currentAllowList.contains(ban))",
            "private record UserBan(NameAndId player, @Nullable String reason, String source, Optional<Instant> expires)",
            "Optional.ofNullable(entry.getExpires()).map(Date::toInstant)",
            "new NameAndId(this.player().id(), this.player().name())",
            "public record UserBanDto(PlayerDto player, Optional<String> reason, Optional<String> source, Optional<Instant> expires)",
            "ExtraCodecs.INSTANT_ISO8601.optionalFieldOf(\"expires\").forGetter(BanlistService.UserBanDto::expires)",
            "Optional.ofNullable(ban.reason())",
            "Optional.of(ban.source())",
            "this.source().orElse(\"Management server\")",
        ] {
            assert!(
                BANLIST_SERVICE.contains(sentinel),
                "BanlistService.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn player(id: &str, name: &str) -> JsonRpcPlayerDto {
        JsonRpcPlayerDto::new(Some(id.to_string()), Some(name.to_string()))
    }

    fn ban(
        player: JsonRpcPlayerDto,
        reason: Option<&str>,
        source: &str,
        expires: Option<&str>,
    ) -> JsonRpcUserBan {
        JsonRpcUserBan {
            player,
            reason: reason.map(ToString::to_string),
            source: source.to_string(),
            expires: expires.map(ToString::to_string),
        }
    }

    fn ip_ban(ip: &str, reason: Option<&str>) -> JsonRpcIpBan {
        JsonRpcIpBan {
            ip: ip.to_string(),
            reason: reason.map(str::to_string),
            source: "Console".to_string(),
            expires: None,
        }
    }
}
