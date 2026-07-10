#![allow(dead_code)]

use crate::chat_component::Component;
use crate::jsonrpc_api::JsonRpcPlayerDto;
use crate::jsonrpc_minecraft_api::MinecraftCompletedFuture;
use crate::jsonrpc_minecraft_api::DedicatedServerIdentity;
use crate::jsonrpc_methods::{ClientInfo, JsonRpcMethodMessage};

pub const DEFAULT_KICK_MESSAGE_KEY: &str = "multiplayer.disconnect.kicked";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcManagedPlayer {
    pub player: JsonRpcPlayerDto,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcKickDto {
    pub player: JsonRpcPlayerDto,
    pub message: Option<JsonRpcMethodMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcKickedPlayer {
    pub player: JsonRpcPlayerDto,
    pub message: Component,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcPlayerList {
    pub players: Vec<JsonRpcManagedPlayer>,
    pub kicked: Vec<JsonRpcKickedPlayer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcRemovedPlayer {
    pub player: JsonRpcPlayerDto,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinecraftPlayerLookupTask {
    FetchById(String),
    FetchByName(String),
}

pub trait MinecraftPlayerListService {
    fn get_players(&self) -> Vec<JsonRpcManagedPlayer>;
    fn get_player(&self, uuid: &str) -> Option<JsonRpcManagedPlayer>;
    fn get_user(
        &mut self,
        id: Option<&str>,
        name: Option<&str>,
    ) -> MinecraftCompletedFuture<Option<JsonRpcPlayerDto>>;
    fn fetch_user_by_name(&self, name: &str) -> Option<JsonRpcPlayerDto>;
    fn fetch_user_by_id(&self, id: &str) -> Option<JsonRpcPlayerDto>;
    fn get_cached_user_by_id(&self, id: &str) -> Option<JsonRpcPlayerDto>;
    fn get_player_optional(
        &self,
        id: Option<&str>,
        name: Option<&str>,
    ) -> Option<JsonRpcManagedPlayer>;
    fn get_players_with_address(&self, ip: &str) -> Vec<JsonRpcManagedPlayer>;
    fn get_player_by_name(&self, name: &str) -> Option<JsonRpcManagedPlayer>;
    fn remove(&mut self, player: &JsonRpcManagedPlayer, client_info: ClientInfo);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftPlayerListModel {
    pub player_list: JsonRpcPlayerList,
    pub cached_users: Vec<JsonRpcPlayerDto>,
    pub session_users: Vec<JsonRpcPlayerDto>,
    pub lookup_tasks: Vec<MinecraftPlayerLookupTask>,
    pub removed_players: Vec<JsonRpcRemovedPlayer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftPlayerListServiceImplModel {
    pub server: DedicatedServerIdentity,
    pub player_list_service: MinecraftPlayerListModel,
    pub log_messages: Vec<(ClientInfo, String)>,
    pub events: Vec<MinecraftPlayerListImplEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinecraftPlayerListImplEvent {
    PlayerRemoved(JsonRpcPlayerDto),
    RemovalLogged(String),
}

impl MinecraftPlayerListServiceImplModel {
    pub fn new(server: DedicatedServerIdentity, player_list_service: MinecraftPlayerListModel) -> Self {
        Self {
            server,
            player_list_service,
            log_messages: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl MinecraftPlayerListModel {
    pub fn new(
        player_list: JsonRpcPlayerList,
        cached_users: Vec<JsonRpcPlayerDto>,
        session_users: Vec<JsonRpcPlayerDto>,
    ) -> Self {
        Self {
            player_list,
            cached_users,
            session_users,
            lookup_tasks: Vec::new(),
            removed_players: Vec::new(),
        }
    }
}

pub struct PlayerService;

impl JsonRpcManagedPlayer {
    pub fn new(id: Option<String>, name: Option<String>) -> Self {
        Self {
            player: JsonRpcPlayerDto::new(id, name),
            address: None,
        }
    }

    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }
}

impl JsonRpcKickDto {
    pub fn new(player: JsonRpcPlayerDto, message: Option<JsonRpcMethodMessage>) -> Self {
        Self { player, message }
    }
}

impl JsonRpcPlayerList {
    pub fn new(players: Vec<JsonRpcManagedPlayer>) -> Self {
        Self {
            players,
            kicked: Vec::new(),
        }
    }

    fn find_server_player_index(&self, player: &JsonRpcPlayerDto) -> Option<usize> {
        if let Some(id) = &player.id {
            self.players
                .iter()
                .position(|server_player| server_player.player.id.as_ref() == Some(id))
        } else {
            player.name.as_ref().and_then(|name| {
                self.players
                    .iter()
                    .position(|server_player| server_player.player.name.as_ref() == Some(name))
            })
        }
    }
}

impl MinecraftPlayerListService for MinecraftPlayerListModel {
    fn get_players(&self) -> Vec<JsonRpcManagedPlayer> {
        self.player_list.players.clone()
    }

    fn get_player(&self, uuid: &str) -> Option<JsonRpcManagedPlayer> {
        self.player_list
            .players
            .iter()
            .find(|player| player.player.id.as_deref() == Some(uuid))
            .cloned()
    }

    fn get_user(
        &mut self,
        id: Option<&str>,
        name: Option<&str>,
    ) -> MinecraftCompletedFuture<Option<JsonRpcPlayerDto>> {
        let value = if let Some(id) = id {
            match self.get_cached_user_by_id(id) {
                some @ Some(_) => some,
                None => {
                    self.lookup_tasks
                        .push(MinecraftPlayerLookupTask::FetchById(id.to_string()));
                    self.fetch_user_by_id(id)
                }
            }
        } else if let Some(name) = name {
            self.lookup_tasks
                .push(MinecraftPlayerLookupTask::FetchByName(name.to_string()));
            self.fetch_user_by_name(name)
        } else {
            None
        };
        MinecraftCompletedFuture::completed(value)
    }

    fn fetch_user_by_name(&self, name: &str) -> Option<JsonRpcPlayerDto> {
        self.cached_users
            .iter()
            .find(|user| {
                user.name
                    .as_deref()
                    .is_some_and(|cached_name| cached_name.eq_ignore_ascii_case(name))
            })
            .cloned()
    }

    fn fetch_user_by_id(&self, id: &str) -> Option<JsonRpcPlayerDto> {
        self.session_users
            .iter()
            .find(|user| user.id.as_deref() == Some(id))
            .cloned()
    }

    fn get_cached_user_by_id(&self, id: &str) -> Option<JsonRpcPlayerDto> {
        self.cached_users
            .iter()
            .find(|user| user.id.as_deref() == Some(id))
            .cloned()
    }

    fn get_player_optional(
        &self,
        id: Option<&str>,
        name: Option<&str>,
    ) -> Option<JsonRpcManagedPlayer> {
        if let Some(id) = id {
            self.get_player(id)
        } else {
            name.and_then(|name| self.get_player_by_name(name))
        }
    }

    fn get_players_with_address(&self, ip: &str) -> Vec<JsonRpcManagedPlayer> {
        self.player_list
            .players
            .iter()
            .filter(|player| player.address.as_deref() == Some(ip))
            .cloned()
            .collect()
    }

    fn get_player_by_name(&self, name: &str) -> Option<JsonRpcManagedPlayer> {
        self.player_list
            .players
            .iter()
            .find(|player| {
                player
                    .player
                    .name
                    .as_deref()
                    .is_some_and(|player_name| player_name.eq_ignore_ascii_case(name))
            })
            .cloned()
    }

    fn remove(&mut self, player: &JsonRpcManagedPlayer, client_info: ClientInfo) {
        self.player_list
            .players
            .retain(|candidate| candidate.player != player.player);
        self.removed_players.push(JsonRpcRemovedPlayer {
            player: player.player.clone(),
            client_info,
        });
    }
}

impl MinecraftPlayerListService for MinecraftPlayerListServiceImplModel {
    fn get_players(&self) -> Vec<JsonRpcManagedPlayer> {
        self.player_list_service.get_players()
    }

    fn get_player(&self, uuid: &str) -> Option<JsonRpcManagedPlayer> {
        self.player_list_service.get_player(uuid)
    }

    fn get_user(
        &mut self,
        id: Option<&str>,
        name: Option<&str>,
    ) -> MinecraftCompletedFuture<Option<JsonRpcPlayerDto>> {
        self.player_list_service.get_user(id, name)
    }

    fn fetch_user_by_name(&self, name: &str) -> Option<JsonRpcPlayerDto> {
        self.player_list_service.fetch_user_by_name(name)
    }

    fn fetch_user_by_id(&self, id: &str) -> Option<JsonRpcPlayerDto> {
        self.player_list_service.fetch_user_by_id(id)
    }

    fn get_cached_user_by_id(&self, id: &str) -> Option<JsonRpcPlayerDto> {
        self.player_list_service.get_cached_user_by_id(id)
    }

    fn get_player_optional(
        &self,
        id: Option<&str>,
        name: Option<&str>,
    ) -> Option<JsonRpcManagedPlayer> {
        self.player_list_service.get_player_optional(id, name)
    }

    fn get_players_with_address(&self, ip: &str) -> Vec<JsonRpcManagedPlayer> {
        self.player_list_service.get_players_with_address(ip)
    }

    fn get_player_by_name(&self, name: &str) -> Option<JsonRpcManagedPlayer> {
        self.player_list_service.get_player_by_name(name)
    }

    fn remove(&mut self, player: &JsonRpcManagedPlayer, client_info: ClientInfo) {
        self.player_list_service.remove(player, client_info);
        self.events
            .push(MinecraftPlayerListImplEvent::PlayerRemoved(player.player.clone()));
        let name = player.player.name.as_deref().unwrap_or("");
        let message = format!("Remove player '{name}'");
        self.log_messages.push((client_info, message.clone()));
        self.events
            .push(MinecraftPlayerListImplEvent::RemovalLogged(message));
    }
}

impl PlayerService {
    pub fn get(player_list: &JsonRpcPlayerList) -> Vec<JsonRpcPlayerDto> {
        player_list
            .players
            .iter()
            .map(|server_player| server_player.player.clone())
            .collect()
    }

    pub fn kick(
        player_list: &mut JsonRpcPlayerList,
        kicks: &[JsonRpcKickDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcPlayerDto> {
        let mut kicked = Vec::new();

        for kick in kicks {
            if let Some(index) = player_list.find_server_player_index(&kick.player) {
                let server_player = player_list.players.remove(index);
                let message = match kick
                    .message
                    .as_ref()
                    .and_then(JsonRpcMethodMessage::as_component)
                {
                    Some(message) => message,
                    None => default_kick_message(),
                };
                player_list.kicked.push(JsonRpcKickedPlayer {
                    player: server_player.player,
                    message,
                    client_info,
                });
                kicked.push(kick.player.clone());
            }
        }

        kicked
    }
}

fn default_kick_message() -> Component {
    Component::translatable(DEFAULT_KICK_MESSAGE_KEY, Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_current_players_as_json_rpc_dtos() {
        let player_list = JsonRpcPlayerList::new(vec![
            player("11111111-1111-1111-1111-111111111111", "Steve"),
            player("22222222-2222-2222-2222-222222222222", "Alex"),
        ]);

        assert_eq!(
            PlayerService::get(&player_list),
            vec![
                JsonRpcPlayerDto::new(
                    Some("11111111-1111-1111-1111-111111111111".to_string()),
                    Some("Steve".to_string()),
                ),
                JsonRpcPlayerDto::new(
                    Some("22222222-2222-2222-2222-222222222222".to_string()),
                    Some("Alex".to_string()),
                ),
            ]
        );
    }

    #[test]
    fn kick_matches_java_resolution_order_and_return_shape() {
        let mut player_list = JsonRpcPlayerList::new(vec![
            player("11111111-1111-1111-1111-111111111111", "Steve"),
            player("22222222-2222-2222-2222-222222222222", "Alex"),
        ]);
        let requested = JsonRpcPlayerDto::new(
            Some("11111111-1111-1111-1111-111111111111".to_string()),
            Some("WrongName".to_string()),
        );

        let kicked = PlayerService::kick(
            &mut player_list,
            &[JsonRpcKickDto::new(
                requested.clone(),
                Some(JsonRpcMethodMessage::literal("Go away")),
            )],
            ClientInfo::of(3),
        );

        assert_eq!(kicked, vec![requested]);
        assert_eq!(PlayerService::get(&player_list), vec![player_dto("22222222-2222-2222-2222-222222222222", "Alex")]);
        assert_eq!(player_list.kicked.len(), 1);
        assert_eq!(player_list.kicked[0].message, Component::literal("Go away"));
        assert_eq!(player_list.kicked[0].client_info, ClientInfo::of(3));
    }

    #[test]
    fn kick_uses_name_only_when_id_is_absent_and_ignores_missing_targets() {
        let mut player_list = JsonRpcPlayerList::new(vec![player(
            "22222222-2222-2222-2222-222222222222",
            "Alex",
        )]);
        let by_name = JsonRpcPlayerDto::new(None, Some("Alex".to_string()));
        let missing = JsonRpcPlayerDto::new(None, Some("Steve".to_string()));

        let kicked = PlayerService::kick(
            &mut player_list,
            &[
                JsonRpcKickDto::new(missing, None),
                JsonRpcKickDto::new(by_name.clone(), None),
            ],
            ClientInfo::of(4),
        );

        assert_eq!(kicked, vec![by_name]);
        assert_eq!(PlayerService::get(&player_list), Vec::<JsonRpcPlayerDto>::new());
        assert_eq!(
            player_list.kicked[0].message,
            Component::translatable(DEFAULT_KICK_MESSAGE_KEY, Vec::new())
        );
    }

    #[test]
    fn minecraft_player_list_service_matches_lookup_and_async_cache_contract() {
        let steve_dto = player_dto("11111111-1111-1111-1111-111111111111", "Steve");
        let alex_dto = player_dto("22222222-2222-2222-2222-222222222222", "Alex");
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve")
            .with_address("192.0.2.10");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex")
            .with_address("192.0.2.10");
        let mut service = MinecraftPlayerListModel::new(
            JsonRpcPlayerList::new(vec![steve.clone(), alex.clone()]),
            vec![steve_dto.clone()],
            vec![alex_dto.clone()],
        );

        assert_eq!(service.get_players(), vec![steve.clone(), alex.clone()]);
        assert_eq!(
            service.get_player("11111111-1111-1111-1111-111111111111"),
            Some(steve.clone())
        );
        assert_eq!(service.get_player_by_name("aLeX"), Some(alex.clone()));
        assert_eq!(
            service.get_players_with_address("192.0.2.10"),
            vec![steve.clone(), alex.clone()]
        );
        assert_eq!(
            service.get_player_optional(
                Some("11111111-1111-1111-1111-111111111111"),
                Some("Alex")
            ),
            Some(steve.clone())
        );

        assert_eq!(
            service
                .get_user(
                    Some("11111111-1111-1111-1111-111111111111"),
                    Some("WrongName")
                )
                .into_inner(),
            Some(steve_dto.clone())
        );
        assert!(service.lookup_tasks.is_empty());
        assert_eq!(
            service
                .get_user(
                    Some("22222222-2222-2222-2222-222222222222"),
                    None
                )
                .into_inner(),
            Some(alex_dto)
        );
        assert_eq!(
            service.get_user(None, Some("sTeVe")).into_inner(),
            Some(steve_dto)
        );
        assert_eq!(service.get_user(None, None).into_inner(), None);
        assert_eq!(
            service.lookup_tasks,
            vec![
                MinecraftPlayerLookupTask::FetchById(
                    "22222222-2222-2222-2222-222222222222".to_string()
                ),
                MinecraftPlayerLookupTask::FetchByName("sTeVe".to_string()),
            ]
        );

        service.remove(&steve, ClientInfo::of(81));
        assert_eq!(service.get_players(), vec![alex]);
        assert_eq!(
            service.removed_players,
            vec![JsonRpcRemovedPlayer {
                player: steve.player,
                client_info: ClientInfo::of(81),
            }]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn minecraft_player_list_service_interface_matches_java_contract() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftPlayerListService.java"
        );
        for sentinel in [
            "import java.util.List;",
            "import java.util.Optional;",
            "import java.util.UUID;",
            "import java.util.concurrent.CompletableFuture;",
            "import net.minecraft.util.Util;",
            "List<ServerPlayer> getPlayers();",
            "@Nullable ServerPlayer getPlayer(UUID uuid);",
            "default CompletableFuture<Optional<NameAndId>> getUser(final Optional<UUID> id, final Optional<String> name)",
            "Optional<NameAndId> nameAndId = this.getCachedUserById(id.get());",
            "CompletableFuture.completedFuture(nameAndId)",
            "CompletableFuture.supplyAsync(() -> this.fetchUserById(id.get()), Util.nonCriticalIoPool())",
            "CompletableFuture.supplyAsync(() -> this.fetchUserByName(name.get()), Util.nonCriticalIoPool())",
            "CompletableFuture.completedFuture(Optional.empty())",
            "Optional<NameAndId> fetchUserByName(String name);",
            "Optional<NameAndId> fetchUserById(UUID id);",
            "Optional<NameAndId> getCachedUserById(UUID id);",
            "Optional<ServerPlayer> getPlayer(Optional<UUID> id, Optional<String> name);",
            "List<ServerPlayer> getPlayersWithAddress(String ip);",
            "@Nullable ServerPlayer getPlayerByName(String name);",
            "void remove(ServerPlayer player, ClientInfo clientInfo);",
        ] {
            assert!(
                SOURCE.contains(sentinel),
                "MinecraftPlayerListService.java missing: {sentinel}"
            );
        }
    }

    #[test]
    fn minecraft_player_list_service_impl_forwards_and_removes_before_logging() {
        let server = DedicatedServerIdentity(91);
        let steve_dto = player_dto("11111111-1111-1111-1111-111111111111", "Steve");
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve")
            .with_address("192.0.2.91");
        let mut service = MinecraftPlayerListServiceImplModel::new(
            server,
            MinecraftPlayerListModel::new(
                JsonRpcPlayerList::new(vec![steve.clone()]),
                vec![steve_dto.clone()],
                vec![steve_dto.clone()],
            ),
        );

        assert_eq!(service.server, server);
        assert_eq!(service.get_players(), vec![steve.clone()]);
        assert_eq!(
            service.get_player("11111111-1111-1111-1111-111111111111"),
            Some(steve.clone())
        );
        assert_eq!(service.get_player_by_name("STEVE"), Some(steve.clone()));
        assert_eq!(
            service.get_players_with_address("192.0.2.91"),
            vec![steve.clone()]
        );
        assert_eq!(
            service
                .get_user(None, Some("Steve"))
                .into_inner(),
            Some(steve_dto)
        );

        service.remove(&steve, ClientInfo::of(92));
        assert!(service.get_players().is_empty());
        assert_eq!(
            service.log_messages,
            vec![(ClientInfo::of(92), "Remove player 'Steve'".to_string())]
        );
        assert_eq!(
            service.events,
            vec![
                MinecraftPlayerListImplEvent::PlayerRemoved(steve.player),
                MinecraftPlayerListImplEvent::RemovalLogged(
                    "Remove player 'Steve'".to_string()
                ),
            ]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn minecraft_player_list_service_impl_matches_java_contract() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftPlayerListServiceImpl.java"
        );
        for sentinel in [
            "private final JsonRpcLogger jsonRpcLogger;",
            "private final DedicatedServer server;",
            "public MinecraftPlayerListServiceImpl(final DedicatedServer server, final JsonRpcLogger jsonRpcLogger)",
            "this.jsonRpcLogger = jsonRpcLogger;",
            "this.server = server;",
            "return this.server.getPlayerList().getPlayers();",
            "return this.server.getPlayerList().getPlayer(uuid);",
            "return this.server.services().nameToIdCache().get(name);",
            "return Optional.ofNullable(this.server.services().sessionService().fetchProfile(id, true)).map(profile -> new NameAndId(profile.profile()));",
            "return this.server.services().nameToIdCache().get(id);",
            "return Optional.ofNullable(this.server.getPlayerList().getPlayer(id.get()));",
            "Optional.ofNullable(this.server.getPlayerList().getPlayerByName(name.get()))",
            "return this.server.getPlayerList().getPlayersWithAddress(ip);",
            "this.server.getPlayerList().remove(serverPlayer);",
            "this.jsonRpcLogger.log(clientInfo, \"Remove player '{}'\", serverPlayer.getPlainTextName());",
            "return this.server.getPlayerList().getPlayerByName(name);",
        ] {
            assert!(
                SOURCE.contains(sentinel),
                "MinecraftPlayerListServiceImpl.java missing: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn player_service_source_matches_java_26_1_2() {
        const PLAYER_SERVICE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/PlayerService.java");

        for sentinel in [
            "private static final Component DEFAULT_KICK_MESSAGE = Component.translatable(\"multiplayer.disconnect.kicked\");",
            "public static List<PlayerDto> get(final MinecraftApi minecraftApi)",
            "return minecraftApi.playerListService().getPlayers().stream().map(PlayerDto::from).toList();",
            "public static List<PlayerDto> kick(final MinecraftApi minecraftApi, final List<PlayerService.KickDto> kick, final ClientInfo clientInfo)",
            "List<PlayerDto> kicked = new ArrayList<>();",
            "ServerPlayer serverPlayer = getServerPlayer(minecraftApi, kickDto.player());",
            "minecraftApi.playerListService().remove(serverPlayer, clientInfo);",
            "serverPlayer.connection.disconnect(kickDto.message.flatMap(Message::asComponent).orElse(DEFAULT_KICK_MESSAGE));",
            "kicked.add(kickDto.player());",
            "if (playerDto.id().isPresent())",
            "return minecraftApi.playerListService().getPlayer(playerDto.id().get());",
            "return playerDto.name().isPresent() ? minecraftApi.playerListService().getPlayerByName(playerDto.name().get()) : null;",
            "public record KickDto(PlayerDto player, Optional<Message> message)",
            "Message.CODEC.optionalFieldOf(\"message\").forGetter(PlayerService.KickDto::message)",
        ] {
            assert!(
                PLAYER_SERVICE.contains(sentinel),
                "PlayerService.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn player(id: &str, name: &str) -> JsonRpcManagedPlayer {
        JsonRpcManagedPlayer::new(Some(id.to_string()), Some(name.to_string()))
    }

    fn player_dto(id: &str, name: &str) -> JsonRpcPlayerDto {
        JsonRpcPlayerDto::new(Some(id.to_string()), Some(name.to_string()))
    }
}
