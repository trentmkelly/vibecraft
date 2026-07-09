#![allow(dead_code)]

use std::net::IpAddr;

use crate::chat_component::Component;
use crate::jsonrpc_api::JsonRpcPlayerDto;
use crate::jsonrpc_methods::ClientInfo;

pub const MANAGEMENT_IP_BAN_SOURCE: &str = "Management server";
pub const IP_BANNED_MESSAGE_KEY: &str = "multiplayer.disconnect.ip_banned";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcIpBan {
    pub ip: String,
    pub reason: Option<String>,
    pub source: String,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcIncomingIpBanDto {
    pub player: Option<JsonRpcPlayerDto>,
    pub ip: Option<String>,
    pub reason: Option<String>,
    pub source: Option<String>,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcIpBanDto {
    pub ip: String,
    pub reason: Option<String>,
    pub source: Option<String>,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcIpBanlist {
    pub entries: Vec<JsonRpcIpBan>,
    pub events: Vec<JsonRpcIpBanlistEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcIpBanOnlinePlayer {
    pub player: JsonRpcPlayerDto,
    pub ip: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcIpBanOnlinePlayers {
    pub players: Vec<JsonRpcIpBanOnlinePlayer>,
    pub disconnects: Vec<JsonRpcIpBanDisconnect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcIpBanDisconnect {
    pub player: JsonRpcPlayerDto,
    pub message: Component,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcIpBanlistEvent {
    AddIpBan {
        ban: JsonRpcIpBan,
        client_info: ClientInfo,
    },
    ClearIpBans {
        client_info: ClientInfo,
    },
    RemoveIpBan {
        ip: String,
        client_info: ClientInfo,
    },
}

pub struct IpBanlistService;

impl JsonRpcIpBan {
    fn to_dto(&self) -> JsonRpcIpBanDto {
        JsonRpcIpBanDto {
            ip: self.ip.clone(),
            reason: self.reason.clone(),
            source: Some(self.source.clone()),
            expires: self.expires.clone(),
        }
    }
}

impl JsonRpcIncomingIpBanDto {
    pub fn new(
        player: Option<JsonRpcPlayerDto>,
        ip: Option<String>,
        reason: Option<String>,
        source: Option<String>,
        expires: Option<String>,
    ) -> Self {
        Self {
            player,
            ip,
            reason,
            source,
            expires,
        }
    }

    fn to_direct_ip_ban(&self) -> Option<JsonRpcIpBan> {
        let ip = self.ip.as_ref()?;
        if !is_inet_address(ip) {
            return None;
        }

        Some(self.to_ip_ban(ip.clone()))
    }

    fn to_player_ip_ban(&self, player: &JsonRpcIpBanOnlinePlayer) -> JsonRpcIpBan {
        self.to_ip_ban(player.ip.clone())
    }

    fn to_ip_ban(&self, ip: String) -> JsonRpcIpBan {
        JsonRpcIpBan {
            ip,
            reason: self.reason.clone(),
            source: match &self.source {
                Some(source) => source.clone(),
                None => MANAGEMENT_IP_BAN_SOURCE.to_string(),
            },
            expires: self.expires.clone(),
        }
    }
}

impl JsonRpcIpBanDto {
    pub fn new(
        ip: String,
        reason: Option<String>,
        source: Option<String>,
        expires: Option<String>,
    ) -> Self {
        Self {
            ip,
            reason,
            source,
            expires,
        }
    }

    fn to_ip_ban(&self) -> JsonRpcIpBan {
        JsonRpcIpBan {
            ip: self.ip.clone(),
            reason: self.reason.clone(),
            source: match &self.source {
                Some(source) => source.clone(),
                None => MANAGEMENT_IP_BAN_SOURCE.to_string(),
            },
            expires: self.expires.clone(),
        }
    }
}

impl JsonRpcIpBanlist {
    pub fn new(entries: Vec<JsonRpcIpBan>) -> Self {
        Self {
            entries,
            events: Vec::new(),
        }
    }

    pub(crate) fn add_ip_ban(&mut self, ban: JsonRpcIpBan, client_info: ClientInfo) {
        let previous = self.entries.iter().position(|entry| entry.ip == ban.ip);
        if let Some(index) = previous {
            if self.entries[index] != ban {
                self.entries[index] = ban.clone();
            }
        } else {
            self.entries.push(ban.clone());
        }
        self.events
            .push(JsonRpcIpBanlistEvent::AddIpBan { ban, client_info });
    }

    pub(crate) fn clear_ip_bans(&mut self, client_info: ClientInfo) {
        self.entries.clear();
        self.events
            .push(JsonRpcIpBanlistEvent::ClearIpBans { client_info });
    }

    pub(crate) fn remove_ip_ban(&mut self, ip: &str, client_info: ClientInfo) {
        self.entries.retain(|ban| ban.ip != ip);
        self.events.push(JsonRpcIpBanlistEvent::RemoveIpBan {
            ip: ip.to_string(),
            client_info,
        });
    }
}

impl JsonRpcIpBanOnlinePlayer {
    pub fn new(player: JsonRpcPlayerDto, ip: String) -> Self {
        Self { player, ip }
    }
}

impl JsonRpcIpBanOnlinePlayers {
    pub fn new(players: Vec<JsonRpcIpBanOnlinePlayer>) -> Self {
        Self {
            players,
            disconnects: Vec::new(),
        }
    }

    fn get_player(&self, player: &JsonRpcPlayerDto) -> Option<JsonRpcIpBanOnlinePlayer> {
        if let Some(id) = &player.id {
            self.players
                .iter()
                .find(|online| online.player.id.as_ref() == Some(id))
                .cloned()
        } else {
            player.name.as_ref().and_then(|name| {
                self.players
                    .iter()
                    .find(|online| online.player.name.as_ref() == Some(name))
                    .cloned()
            })
        }
    }

    fn disconnect_players_with_address(&mut self, ip: &str) {
        for player in self.players.iter().filter(|player| player.ip == ip) {
            self.disconnects.push(JsonRpcIpBanDisconnect {
                player: player.player.clone(),
                message: Component::translatable(IP_BANNED_MESSAGE_KEY, Vec::new()),
            });
        }
    }
}

impl IpBanlistService {
    pub fn get(banlist: &JsonRpcIpBanlist) -> Vec<JsonRpcIpBanDto> {
        banlist.entries.iter().map(JsonRpcIpBan::to_dto).collect()
    }

    pub fn add(
        online_players: &mut JsonRpcIpBanOnlinePlayers,
        banlist: &mut JsonRpcIpBanlist,
        bans: &[JsonRpcIncomingIpBanDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcIpBanDto> {
        for incoming in bans {
            if let Some(ban) = resolve_incoming_ban(online_players, incoming) {
                ban_ip(online_players, banlist, ban, client_info);
            }
        }

        Self::get(banlist)
    }

    pub fn clear(
        banlist: &mut JsonRpcIpBanlist,
        client_info: ClientInfo,
    ) -> Vec<JsonRpcIpBanDto> {
        banlist.clear_ip_bans(client_info);
        Self::get(banlist)
    }

    pub fn remove(
        banlist: &mut JsonRpcIpBanlist,
        bans: &[String],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcIpBanDto> {
        for ip in bans {
            banlist.remove_ip_ban(ip, client_info);
        }

        Self::get(banlist)
    }

    pub fn set(
        online_players: &mut JsonRpcIpBanOnlinePlayers,
        banlist: &mut JsonRpcIpBanlist,
        ips: &[JsonRpcIpBanDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcIpBanDto> {
        let final_banlist = valid_unique_bans(ips);
        let current_bans = unique_bans(&banlist.entries);

        for current in &current_bans {
            if !final_banlist.contains(current) {
                banlist.remove_ip_ban(&current.ip, client_info);
            }
        }
        for final_ban in final_banlist {
            if !current_bans.contains(&final_ban) {
                ban_ip(online_players, banlist, final_ban, client_info);
            }
        }

        Self::get(banlist)
    }
}

fn resolve_incoming_ban(
    online_players: &JsonRpcIpBanOnlinePlayers,
    incoming: &JsonRpcIncomingIpBanDto,
) -> Option<JsonRpcIpBan> {
    if let Some(ban) = incoming.to_direct_ip_ban() {
        return Some(ban);
    }

    let player = incoming.player.as_ref()?;
    let online_player = online_players.get_player(player)?;
    Some(incoming.to_player_ip_ban(&online_player))
}

fn ban_ip(
    online_players: &mut JsonRpcIpBanOnlinePlayers,
    banlist: &mut JsonRpcIpBanlist,
    ban: JsonRpcIpBan,
    client_info: ClientInfo,
) {
    banlist.add_ip_ban(ban.clone(), client_info);
    online_players.disconnect_players_with_address(&ban.ip);
}

fn valid_unique_bans(ips: &[JsonRpcIpBanDto]) -> Vec<JsonRpcIpBan> {
    let mut unique = Vec::new();
    for ip in ips {
        if is_inet_address(&ip.ip) {
            let ban = ip.to_ip_ban();
            if !unique.contains(&ban) {
                unique.push(ban);
            }
        }
    }
    unique
}

fn unique_bans(bans: &[JsonRpcIpBan]) -> Vec<JsonRpcIpBan> {
    let mut unique = Vec::new();
    for ban in bans {
        if !unique.contains(ban) {
            unique.push(ban.clone());
        }
    }
    unique
}

fn is_inet_address(ip: &str) -> bool {
    ip.parse::<IpAddr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_add_and_clear_match_java_ip_ban_facade() {
        let steve = online_player(
            "11111111-1111-1111-1111-111111111111",
            "Steve",
            "192.0.2.10",
        );
        let mut online = JsonRpcIpBanOnlinePlayers::new(vec![steve.clone()]);
        let mut banlist = JsonRpcIpBanlist::default();

        let result = IpBanlistService::add(
            &mut online,
            &mut banlist,
            &[
                JsonRpcIncomingIpBanDto::new(
                    None,
                    Some("192.0.2.10".to_string()),
                    Some("bad".to_string()),
                    None,
                    Some("2026-06-27T00:00:00Z".to_string()),
                ),
                JsonRpcIncomingIpBanDto::new(
                    None,
                    Some("not an ip".to_string()),
                    Some("ignored".to_string()),
                    None,
                    None,
                ),
            ],
            ClientInfo::of(30),
        );

        assert_eq!(
            result,
            vec![JsonRpcIpBanDto::new(
                "192.0.2.10".to_string(),
                Some("bad".to_string()),
                Some(MANAGEMENT_IP_BAN_SOURCE.to_string()),
                Some("2026-06-27T00:00:00Z".to_string()),
            )]
        );
        assert_eq!(
            online.disconnects,
            vec![JsonRpcIpBanDisconnect {
                player: steve.player,
                message: Component::translatable(IP_BANNED_MESSAGE_KEY, Vec::new()),
            }]
        );
        assert_eq!(
            IpBanlistService::clear(&mut banlist, ClientInfo::of(31)),
            Vec::<JsonRpcIpBanDto>::new()
        );
    }

    #[test]
    fn incoming_direct_ip_wins_over_player_lookup_and_replaces_by_ip_key() {
        let steve = online_player(
            "11111111-1111-1111-1111-111111111111",
            "Steve",
            "192.0.2.10",
        );
        let alex = online_player(
            "22222222-2222-2222-2222-222222222222",
            "Alex",
            "192.0.2.20",
        );
        let mut online = JsonRpcIpBanOnlinePlayers::new(vec![steve, alex.clone()]);
        let mut banlist = JsonRpcIpBanlist::new(vec![ip_ban("192.0.2.20", Some("old"), "Console", None)]);

        let result = IpBanlistService::add(
            &mut online,
            &mut banlist,
            &[JsonRpcIncomingIpBanDto::new(
                Some(JsonRpcPlayerDto::new(None, Some("Steve".to_string()))),
                Some("192.0.2.20".to_string()),
                Some("new".to_string()),
                Some("Console".to_string()),
                None,
            )],
            ClientInfo::of(32),
        );

        assert_eq!(
            result,
            vec![JsonRpcIpBanDto::new(
                "192.0.2.20".to_string(),
                Some("new".to_string()),
                Some("Console".to_string()),
                None,
            )]
        );
        assert_eq!(banlist.entries.len(), 1);
        assert_eq!(
            online.disconnects,
            vec![JsonRpcIpBanDisconnect {
                player: alex.player,
                message: Component::translatable(IP_BANNED_MESSAGE_KEY, Vec::new()),
            }]
        );
    }

    #[test]
    fn incoming_player_ban_uses_online_player_ip_when_direct_ip_is_missing_or_invalid() {
        let steve = online_player(
            "11111111-1111-1111-1111-111111111111",
            "Steve",
            "2001:db8::1",
        );
        let mut online = JsonRpcIpBanOnlinePlayers::new(vec![steve.clone()]);
        let mut banlist = JsonRpcIpBanlist::default();

        let result = IpBanlistService::add(
            &mut online,
            &mut banlist,
            &[JsonRpcIncomingIpBanDto::new(
                Some(JsonRpcPlayerDto::new(
                    Some("11111111-1111-1111-1111-111111111111".to_string()),
                    Some("WrongName".to_string()),
                )),
                Some("invalid".to_string()),
                None,
                None,
                None,
            )],
            ClientInfo::of(33),
        );

        assert_eq!(
            result,
            vec![JsonRpcIpBanDto::new(
                "2001:db8::1".to_string(),
                None,
                Some(MANAGEMENT_IP_BAN_SOURCE.to_string()),
                None,
            )]
        );
        assert_eq!(
            online.disconnects,
            vec![JsonRpcIpBanDisconnect {
                player: steve.player,
                message: Component::translatable(IP_BANNED_MESSAGE_KEY, Vec::new()),
            }]
        );
    }

    #[test]
    fn remove_and_set_match_java_record_set_difference() {
        let steve = online_player(
            "11111111-1111-1111-1111-111111111111",
            "Steve",
            "192.0.2.10",
        );
        let alex = online_player(
            "22222222-2222-2222-2222-222222222222",
            "Alex",
            "192.0.2.20",
        );
        let mut online = JsonRpcIpBanOnlinePlayers::new(vec![steve.clone(), alex.clone()]);
        let mut banlist = JsonRpcIpBanlist::new(vec![
            ip_ban("192.0.2.10", Some("old"), "Console", None),
            ip_ban("192.0.2.20", None, "Console", None),
        ]);

        let set_result = IpBanlistService::set(
            &mut online,
            &mut banlist,
            &[
                JsonRpcIpBanDto::new(
                    "192.0.2.10".to_string(),
                    Some("new".to_string()),
                    Some("Console".to_string()),
                    None,
                ),
                JsonRpcIpBanDto::new(
                    "not an ip".to_string(),
                    Some("ignored".to_string()),
                    Some("Console".to_string()),
                    None,
                ),
            ],
            ClientInfo::of(34),
        );

        assert_eq!(
            set_result,
            vec![JsonRpcIpBanDto::new(
                "192.0.2.10".to_string(),
                Some("new".to_string()),
                Some("Console".to_string()),
                None,
            )]
        );
        assert_eq!(
            banlist.events,
            vec![
                JsonRpcIpBanlistEvent::RemoveIpBan {
                    ip: "192.0.2.10".to_string(),
                    client_info: ClientInfo::of(34),
                },
                JsonRpcIpBanlistEvent::RemoveIpBan {
                    ip: "192.0.2.20".to_string(),
                    client_info: ClientInfo::of(34),
                },
                JsonRpcIpBanlistEvent::AddIpBan {
                    ban: ip_ban("192.0.2.10", Some("new"), "Console", None),
                    client_info: ClientInfo::of(34),
                },
            ]
        );
        assert_eq!(
            online.disconnects,
            vec![JsonRpcIpBanDisconnect {
                player: steve.player,
                message: Component::translatable(IP_BANNED_MESSAGE_KEY, Vec::new()),
            }]
        );

        assert_eq!(
            IpBanlistService::remove(&mut banlist, &["192.0.2.10".to_string()], ClientInfo::of(35)),
            Vec::<JsonRpcIpBanDto>::new()
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn ip_banlist_service_source_matches_java_26_1_2() {
        const IP_BANLIST_SERVICE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/IpBanlistService.java");

        for sentinel in [
            "private static final String BAN_SOURCE = \"Management server\";",
            "public static List<IpBanlistService.IpBanDto> get(final MinecraftApi minecraftApi)",
            "getIpBanEntries().stream().map(IpBanlistService.IpBan::from).map(IpBanlistService.IpBanDto::from).toList();",
            ".flatMap(Collection::stream)",
            "player.connection.disconnect(Component.translatable(\"multiplayer.disconnect.ip_banned\"))",
            "IpBanlistService.IpBan ipBan = ban.toIpBan();",
            "if (ban.player().isPresent())",
            "getPlayer(ban.player().get().id(), ban.player().get().name())",
            "return banIp(minecraftApi, ban.toIpBan(player.get()), clientInfo);",
            "minecraftApi.banListService().addIpBan(ban.toIpBanEntry(), clientInfo);",
            "return minecraftApi.playerListService().getPlayersWithAddress(ban.ip());",
            "minecraftApi.banListService().clearIpBans(clientInfo);",
            "ban.forEach(ip -> minecraftApi.banListService().removeIpBan(ip, clientInfo));",
            "filter(ban -> InetAddresses.isInetAddress(ban.ip()))",
            "map(IpBanlistService.IpBanDto::toIpBan)",
            "currentBans.stream().filter(ban -> !finalBanlist.contains(ban)).forEach(ban -> minecraftApi.banListService().removeIpBan(ban.ip(), clientInfo));",
            "finalBanlist.stream().filter(ban -> !currentBans.contains(ban)).forEach(ban -> minecraftApi.banListService().addIpBan(ban.toIpBanEntry(), clientInfo));",
            "flatMap(ban -> minecraftApi.playerListService().getPlayersWithAddress(ban.ip()).stream())",
            "public record IncomingIpBanDto(Optional<PlayerDto> player, Optional<String> ip, Optional<String> reason, Optional<String> source, Optional<Instant> expires)",
            "return new IpBanlistService.IpBan(player.getIpAddress(), this.reason().orElse(null), this.source().orElse(\"Management server\"), this.expires());",
            "? new IpBanlistService.IpBan(this.ip().get(), this.reason().orElse(null), this.source().orElse(\"Management server\"), this.expires())",
            "private record IpBan(String ip, @Nullable String reason, String source, Optional<Instant> expires)",
            "Optional.ofNullable(entry.getExpires()).map(Date::toInstant)",
            "private IpBanListEntry toIpBanEntry()",
            "public record IpBanDto(String ip, Optional<String> reason, Optional<String> source, Optional<Instant> expires)",
            "return new IpBanlistService.IpBanDto(ban.ip(), Optional.ofNullable(ban.reason()), Optional.of(ban.source()), ban.expires());",
            "return new IpBanlistService.IpBan(this.ip(), this.reason().orElse(null), this.source().orElse(\"Management server\"), this.expires());",
        ] {
            assert!(
                IP_BANLIST_SERVICE.contains(sentinel),
                "IpBanlistService.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn online_player(id: &str, name: &str, ip: &str) -> JsonRpcIpBanOnlinePlayer {
        JsonRpcIpBanOnlinePlayer::new(
            JsonRpcPlayerDto::new(Some(id.to_string()), Some(name.to_string())),
            ip.to_string(),
        )
    }

    fn ip_ban(
        ip: &str,
        reason: Option<&str>,
        source: &str,
        expires: Option<&str>,
    ) -> JsonRpcIpBan {
        JsonRpcIpBan {
            ip: ip.to_string(),
            reason: reason.map(ToString::to_string),
            source: source.to_string(),
            expires: expires.map(ToString::to_string),
        }
    }
}
