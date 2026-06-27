#![allow(dead_code)]

use crate::jsonrpc_api::JsonRpcPlayerDto;
use crate::jsonrpc_methods::ClientInfo;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcAllowlist {
    pub entries: Vec<JsonRpcPlayerDto>,
    pub events: Vec<JsonRpcAllowlistEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcAllowlistUserDirectory {
    pub users: Vec<JsonRpcPlayerDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcAllowlistEvent {
    Add {
        user: JsonRpcPlayerDto,
        client_info: ClientInfo,
    },
    Clear {
        client_info: ClientInfo,
    },
    Remove {
        user: JsonRpcPlayerDto,
        client_info: ClientInfo,
    },
    KickUnlistedPlayers {
        client_info: ClientInfo,
    },
}

pub struct AllowlistService;

impl JsonRpcAllowlist {
    pub fn new(entries: Vec<JsonRpcPlayerDto>) -> Self {
        Self {
            entries,
            events: Vec::new(),
        }
    }

    fn add(&mut self, user: JsonRpcPlayerDto, client_info: ClientInfo) {
        if !self.entries.contains(&user) {
            self.entries.push(user.clone());
        }
        self.events
            .push(JsonRpcAllowlistEvent::Add { user, client_info });
    }

    fn clear(&mut self, client_info: ClientInfo) {
        self.entries.clear();
        self.events
            .push(JsonRpcAllowlistEvent::Clear { client_info });
    }

    fn remove(&mut self, user: &JsonRpcPlayerDto, client_info: ClientInfo) {
        self.entries.retain(|entry| entry != user);
        self.events.push(JsonRpcAllowlistEvent::Remove {
            user: user.clone(),
            client_info,
        });
    }

    fn kick_unlisted_players(&mut self, client_info: ClientInfo) {
        self.events
            .push(JsonRpcAllowlistEvent::KickUnlistedPlayers { client_info });
    }
}

impl JsonRpcAllowlistUserDirectory {
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

impl AllowlistService {
    pub fn get(allowlist: &JsonRpcAllowlist) -> Vec<JsonRpcPlayerDto> {
        allowlist.entries.clone()
    }

    pub fn add(
        users: &JsonRpcAllowlistUserDirectory,
        allowlist: &mut JsonRpcAllowlist,
        player_dtos: &[JsonRpcPlayerDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcPlayerDto> {
        for player_dto in player_dtos {
            if let Some(user) = users.get_user(player_dto) {
                allowlist.add(user, client_info);
            }
        }

        Self::get(allowlist)
    }

    pub fn clear(
        allowlist: &mut JsonRpcAllowlist,
        client_info: ClientInfo,
    ) -> Vec<JsonRpcPlayerDto> {
        allowlist.clear(client_info);
        Self::get(allowlist)
    }

    pub fn remove(
        users: &JsonRpcAllowlistUserDirectory,
        allowlist: &mut JsonRpcAllowlist,
        player_dtos: &[JsonRpcPlayerDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcPlayerDto> {
        for player_dto in player_dtos {
            if let Some(user) = users.get_user(player_dto) {
                allowlist.remove(&user, client_info);
            }
        }

        allowlist.kick_unlisted_players(client_info);
        Self::get(allowlist)
    }

    pub fn set(
        users: &JsonRpcAllowlistUserDirectory,
        allowlist: &mut JsonRpcAllowlist,
        player_dtos: &[JsonRpcPlayerDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcPlayerDto> {
        let final_allowlist = resolved_unique_users(users, player_dtos);
        let current_allowlist = unique_users(&allowlist.entries);

        for current in &current_allowlist {
            if !final_allowlist.contains(current) {
                allowlist.remove(current, client_info);
            }
        }
        for final_user in final_allowlist {
            if !current_allowlist.contains(&final_user) {
                allowlist.add(final_user, client_info);
            }
        }
        allowlist.kick_unlisted_players(client_info);

        Self::get(allowlist)
    }
}

fn resolved_unique_users(
    users: &JsonRpcAllowlistUserDirectory,
    player_dtos: &[JsonRpcPlayerDto],
) -> Vec<JsonRpcPlayerDto> {
    let mut resolved = Vec::new();
    for player_dto in player_dtos {
        if let Some(user) = users.get_user(player_dto) {
            if !resolved.contains(&user) {
                resolved.push(user);
            }
        }
    }
    resolved
}

fn unique_users(users: &[JsonRpcPlayerDto]) -> Vec<JsonRpcPlayerDto> {
    let mut unique = Vec::new();
    for user in users {
        if !unique.contains(user) {
            unique.push(user.clone());
        }
    }
    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_add_and_clear_match_java_allowlist_facade() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let users = JsonRpcAllowlistUserDirectory::new(vec![steve.clone()]);
        let mut allowlist = JsonRpcAllowlist::default();

        assert_eq!(
            AllowlistService::add(
                &users,
                &mut allowlist,
                &[JsonRpcPlayerDto::new(None, Some("Steve".to_string()))],
                ClientInfo::of(2),
            ),
            vec![steve.clone()]
        );
        assert_eq!(AllowlistService::get(&allowlist), vec![steve.clone()]);
        assert_eq!(
            AllowlistService::clear(&mut allowlist, ClientInfo::of(3)),
            Vec::<JsonRpcPlayerDto>::new()
        );
        assert_eq!(
            allowlist.events,
            vec![
                JsonRpcAllowlistEvent::Add {
                    user: steve,
                    client_info: ClientInfo::of(2),
                },
                JsonRpcAllowlistEvent::Clear {
                    client_info: ClientInfo::of(3),
                },
            ]
        );
    }

    #[test]
    fn remove_resolves_requested_players_and_kicks_unlisted_players() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex");
        let users = JsonRpcAllowlistUserDirectory::new(vec![steve.clone(), alex.clone()]);
        let mut allowlist = JsonRpcAllowlist::new(vec![steve.clone(), alex.clone()]);

        let result = AllowlistService::remove(
            &users,
            &mut allowlist,
            &[
                JsonRpcPlayerDto::new(
                    Some("11111111-1111-1111-1111-111111111111".to_string()),
                    Some("WrongName".to_string()),
                ),
                JsonRpcPlayerDto::new(None, Some("Missing".to_string())),
            ],
            ClientInfo::of(4),
        );

        assert_eq!(result, vec![alex]);
        assert_eq!(
            allowlist.events,
            vec![
                JsonRpcAllowlistEvent::Remove {
                    user: steve,
                    client_info: ClientInfo::of(4),
                },
                JsonRpcAllowlistEvent::KickUnlistedPlayers {
                    client_info: ClientInfo::of(4),
                },
            ]
        );
    }

    #[test]
    fn set_matches_java_allowlist_set_difference_and_kicks_afterwards() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex");
        let users = JsonRpcAllowlistUserDirectory::new(vec![steve.clone(), alex.clone()]);
        let mut allowlist = JsonRpcAllowlist::new(vec![alex.clone()]);

        let result = AllowlistService::set(
            &users,
            &mut allowlist,
            &[JsonRpcPlayerDto::new(
                Some("11111111-1111-1111-1111-111111111111".to_string()),
                Some("Steve".to_string()),
            )],
            ClientInfo::of(5),
        );

        assert_eq!(result, vec![steve.clone()]);
        assert_eq!(
            allowlist.events,
            vec![
                JsonRpcAllowlistEvent::Remove {
                    user: alex,
                    client_info: ClientInfo::of(5),
                },
                JsonRpcAllowlistEvent::Add {
                    user: steve,
                    client_info: ClientInfo::of(5),
                },
                JsonRpcAllowlistEvent::KickUnlistedPlayers {
                    client_info: ClientInfo::of(5),
                },
            ]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn allowlist_service_source_matches_java_26_1_2() {
        const ALLOWLIST_SERVICE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/AllowlistService.java");

        for sentinel in [
            "public static List<PlayerDto> get(final MinecraftApi minecraftApi)",
            "return minecraftApi.allowListService().getEntries().stream().filter(p -> p.getUser() != null).map(u -> PlayerDto.from(u.getUser())).toList();",
            "public static List<PlayerDto> add(final MinecraftApi minecraftApi, final List<PlayerDto> playerDtos, final ClientInfo clientInfo)",
            "minecraftApi.playerListService().getUser(playerDto.id(), playerDto.name())",
            "user.ifPresent(nameAndId -> minecraftApi.allowListService().add(new UserWhiteListEntry(nameAndId), clientInfo));",
            "minecraftApi.allowListService().clear(clientInfo);",
            "user.ifPresent(nameAndId -> minecraftApi.allowListService().remove(nameAndId, clientInfo));",
            "minecraftApi.allowListService().kickUnlistedPlayers(clientInfo);",
            "Set<NameAndId> finalAllowList = Util.sequence(fetch).join().stream().flatMap(Optional::stream).collect(Collectors.toSet());",
            "Set<NameAndId> currentAllowList = minecraftApi.allowListService().getEntries().stream().map(StoredUserEntry::getUser).collect(Collectors.toSet());",
            "currentAllowList.stream().filter(user -> !finalAllowList.contains(user)).forEach(user -> minecraftApi.allowListService().remove(user, clientInfo));",
            "filter(user -> !currentAllowList.contains(user))",
            "forEach(user -> minecraftApi.allowListService().add(new UserWhiteListEntry(user), clientInfo));",
        ] {
            assert!(
                ALLOWLIST_SERVICE.contains(sentinel),
                "AllowlistService.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn player(id: &str, name: &str) -> JsonRpcPlayerDto {
        JsonRpcPlayerDto::new(Some(id.to_string()), Some(name.to_string()))
    }
}
