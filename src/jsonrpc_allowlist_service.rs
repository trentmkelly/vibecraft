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

pub trait MinecraftAllowListService {
    fn get_entries(&self) -> Vec<JsonRpcPlayerDto>;
    fn add(&mut self, infos: JsonRpcPlayerDto, client_info: ClientInfo) -> bool;
    fn clear(&mut self, client_info: ClientInfo);
    fn remove(&mut self, name_and_id: &JsonRpcPlayerDto, client_info: ClientInfo);
    fn kick_unlisted_players(&mut self, client_info: ClientInfo);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftAllowListServiceImplModel {
    pub server_allowlist: JsonRpcAllowlist,
    pub log_messages: Vec<(ClientInfo, String)>,
    pub server_events: Vec<MinecraftAllowListServerEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinecraftAllowListServerEvent {
    KickUnlistedPlayers,
}

impl MinecraftAllowListServiceImplModel {
    pub fn new(server_allowlist: JsonRpcAllowlist) -> Self {
        Self {
            server_allowlist,
            log_messages: Vec::new(),
            server_events: Vec::new(),
        }
    }

    fn log(&mut self, client_info: ClientInfo, message: String) {
        self.log_messages.push((client_info, message));
    }
}

impl MinecraftAllowListService for MinecraftAllowListServiceImplModel {
    fn get_entries(&self) -> Vec<JsonRpcPlayerDto> {
        MinecraftAllowListService::get_entries(&self.server_allowlist)
    }

    fn add(&mut self, infos: JsonRpcPlayerDto, client_info: ClientInfo) -> bool {
        self.log(
            client_info,
            format!("Add player '{}' to allowlist", player_display(&infos)),
        );
        MinecraftAllowListService::add(&mut self.server_allowlist, infos, client_info)
    }

    fn clear(&mut self, client_info: ClientInfo) {
        self.log(client_info, "Clear allowlist".to_string());
        MinecraftAllowListService::clear(&mut self.server_allowlist, client_info);
    }

    fn remove(&mut self, name_and_id: &JsonRpcPlayerDto, client_info: ClientInfo) {
        self.log(
            client_info,
            format!(
                "Remove player '{}' from allowlist",
                player_display(name_and_id)
            ),
        );
        MinecraftAllowListService::remove(&mut self.server_allowlist, name_and_id, client_info);
    }

    fn kick_unlisted_players(&mut self, client_info: ClientInfo) {
        self.log(client_info, "Kick unlisted players".to_string());
        self.server_events
            .push(MinecraftAllowListServerEvent::KickUnlistedPlayers);
    }
}

impl MinecraftAllowListService for JsonRpcAllowlist {
    fn get_entries(&self) -> Vec<JsonRpcPlayerDto> {
        self.entries.clone()
    }

    fn add(&mut self, infos: JsonRpcPlayerDto, client_info: ClientInfo) -> bool {
        let inserted = !self.entries.contains(&infos);
        if inserted {
            self.entries.push(infos.clone());
        }
        self.events.push(JsonRpcAllowlistEvent::Add {
            user: infos,
            client_info,
        });
        inserted
    }

    fn clear(&mut self, client_info: ClientInfo) {
        self.entries.clear();
        self.events
            .push(JsonRpcAllowlistEvent::Clear { client_info });
    }

    fn remove(&mut self, name_and_id: &JsonRpcPlayerDto, client_info: ClientInfo) {
        self.entries.retain(|entry| entry != name_and_id);
        self.events.push(JsonRpcAllowlistEvent::Remove {
            user: name_and_id.clone(),
            client_info,
        });
    }

    fn kick_unlisted_players(&mut self, client_info: ClientInfo) {
        self.events
            .push(JsonRpcAllowlistEvent::KickUnlistedPlayers { client_info });
    }
}

fn player_display(player: &JsonRpcPlayerDto) -> String {
    // Java's logger receives a NameAndId record. Records use this exact generated
    // toString shape; retain `null` handling because PlayerDto represents both
    // fields as optional at the JSON-RPC boundary.
    let id = player.id.as_deref().unwrap_or("null");
    let name = player.name.as_deref().unwrap_or("null");
    format!("NameAndId[id={id}, name={name}]")
}

impl JsonRpcAllowlist {
    pub fn new(entries: Vec<JsonRpcPlayerDto>) -> Self {
        Self {
            entries,
            events: Vec::new(),
        }
    }

    fn add(&mut self, user: JsonRpcPlayerDto, client_info: ClientInfo) {
        <Self as MinecraftAllowListService>::add(self, user, client_info);
    }

    fn clear(&mut self, client_info: ClientInfo) {
        <Self as MinecraftAllowListService>::clear(self, client_info);
    }

    fn remove(&mut self, user: &JsonRpcPlayerDto, client_info: ClientInfo) {
        <Self as MinecraftAllowListService>::remove(self, user, client_info);
    }

    fn kick_unlisted_players(&mut self, client_info: ClientInfo) {
        <Self as MinecraftAllowListService>::kick_unlisted_players(self, client_info);
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
    fn minecraft_allow_list_service_interface_matches_java_contract() {
        const MINECRAFT_ALLOW_LIST_SERVICE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftAllowListService.java"
        );
        for sentinel in [
            "public interface MinecraftAllowListService",
            "Collection<UserWhiteListEntry> getEntries();",
            "boolean add(UserWhiteListEntry infos, ClientInfo clientInfo);",
            "void clear(ClientInfo clientInfo);",
            "void remove(NameAndId nameAndId, ClientInfo clientInfo);",
            "void kickUnlistedPlayers(ClientInfo clientInfo);",
            "import net.minecraft.server.jsonrpc.methods.ClientInfo;",
            "import net.minecraft.server.players.NameAndId;",
            "import net.minecraft.server.players.UserWhiteListEntry;",
        ] {
            assert!(
                MINECRAFT_ALLOW_LIST_SERVICE.contains(sentinel),
                "MinecraftAllowListService.java is missing sentinel: {sentinel}"
            );
        }

        let steve = player("00000000-0000-0000-0000-000000000001", "Steve");
        let alex = player("00000000-0000-0000-0000-000000000002", "Alex");
        let mut allowlist = JsonRpcAllowlist::new(vec![steve.clone()]);
        let client_info = ClientInfo::of(9);

        assert_eq!(
            MinecraftAllowListService::get_entries(&allowlist),
            vec![steve.clone()]
        );
        assert!(!MinecraftAllowListService::add(
            &mut allowlist,
            steve.clone(),
            client_info
        ));
        assert!(MinecraftAllowListService::add(
            &mut allowlist,
            alex.clone(),
            client_info
        ));
        assert_eq!(
            MinecraftAllowListService::get_entries(&allowlist),
            vec![steve.clone(), alex.clone()]
        );

        MinecraftAllowListService::remove(&mut allowlist, &steve, client_info);
        MinecraftAllowListService::kick_unlisted_players(&mut allowlist, client_info);
        MinecraftAllowListService::clear(&mut allowlist, client_info);
        assert_eq!(MinecraftAllowListService::get_entries(&allowlist), vec![]);
        assert_eq!(
            allowlist.events,
            vec![
                JsonRpcAllowlistEvent::Add {
                    user: steve.clone(),
                    client_info,
                },
                JsonRpcAllowlistEvent::Add {
                    user: alex,
                    client_info,
                },
                JsonRpcAllowlistEvent::Remove {
                    user: steve,
                    client_info,
                },
                JsonRpcAllowlistEvent::KickUnlistedPlayers { client_info },
                JsonRpcAllowlistEvent::Clear { client_info },
            ]
        );
    }

    #[test]
    fn minecraft_allow_list_service_impl_logs_and_delegates_like_java() {
        const MINECRAFT_ALLOW_LIST_SERVICE_IMPL: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftAllowListServiceImpl.java"
        );
        for sentinel in [
            "public class MinecraftAllowListServiceImpl implements MinecraftAllowListService",
            "private final DedicatedServer server;",
            "private final JsonRpcLogger jsonrpcLogger;",
            "public MinecraftAllowListServiceImpl(final DedicatedServer server, final JsonRpcLogger jsonrpcLogger)",
            "this.server = server;",
            "this.jsonrpcLogger = jsonrpcLogger;",
            "return this.server.getPlayerList().getWhiteList().getEntries();",
            "this.jsonrpcLogger.log(clientInfo, \"Add player '{}' to allowlist\", infos.getUser());",
            "return this.server.getPlayerList().getWhiteList().add(infos);",
            "this.jsonrpcLogger.log(clientInfo, \"Clear allowlist\");",
            "this.server.getPlayerList().getWhiteList().clear();",
            "this.jsonrpcLogger.log(clientInfo, \"Remove player '{}' from allowlist\", nameAndId);",
            "this.server.getPlayerList().getWhiteList().remove(nameAndId);",
            "this.jsonrpcLogger.log(clientInfo, \"Kick unlisted players\");",
            "this.server.kickUnlistedPlayers();",
        ] {
            assert!(
                MINECRAFT_ALLOW_LIST_SERVICE_IMPL.contains(sentinel),
                "MinecraftAllowListServiceImpl.java is missing sentinel: {sentinel}"
            );
        }

        let steve = player("00000000-0000-0000-0000-000000000001", "Steve");
        let alex = player("00000000-0000-0000-0000-000000000002", "Alex");
        let client_info = ClientInfo::of(11);
        let mut service =
            MinecraftAllowListServiceImplModel::new(JsonRpcAllowlist::new(vec![steve.clone()]));

        assert_eq!(
            MinecraftAllowListService::get_entries(&service),
            vec![steve.clone()]
        );
        assert!(MinecraftAllowListService::add(
            &mut service,
            alex.clone(),
            client_info
        ));
        assert!(!MinecraftAllowListService::add(
            &mut service,
            alex.clone(),
            client_info
        ));
        MinecraftAllowListService::remove(&mut service, &steve, client_info);
        MinecraftAllowListService::clear(&mut service, client_info);
        MinecraftAllowListService::kick_unlisted_players(&mut service, client_info);

        assert_eq!(MinecraftAllowListService::get_entries(&service), vec![]);
        assert_eq!(
            service.log_messages,
            vec![
                (
                    client_info,
                    "Add player 'NameAndId[id=00000000-0000-0000-0000-000000000002, name=Alex]' to allowlist".to_string(),
                ),
                (
                    client_info,
                    "Add player 'NameAndId[id=00000000-0000-0000-0000-000000000002, name=Alex]' to allowlist".to_string(),
                ),
                (
                    client_info,
                    "Remove player 'NameAndId[id=00000000-0000-0000-0000-000000000001, name=Steve]' from allowlist".to_string(),
                ),
                (client_info, "Clear allowlist".to_string()),
                (client_info, "Kick unlisted players".to_string()),
            ]
        );
        assert_eq!(
            service.server_allowlist.events,
            vec![
                JsonRpcAllowlistEvent::Add {
                    user: alex.clone(),
                    client_info,
                },
                JsonRpcAllowlistEvent::Add {
                    user: alex,
                    client_info,
                },
                JsonRpcAllowlistEvent::Remove {
                    user: steve,
                    client_info,
                },
                JsonRpcAllowlistEvent::Clear { client_info },
            ]
        );
        assert_eq!(
            service.server_events,
            vec![MinecraftAllowListServerEvent::KickUnlistedPlayers]
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
