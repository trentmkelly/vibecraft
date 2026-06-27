#![allow(dead_code)]

use crate::chat_component::Component;
use crate::jsonrpc_api::JsonRpcPlayerDto;
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
}
