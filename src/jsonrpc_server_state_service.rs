#![allow(dead_code)]

use crate::chat_component::Component;
use crate::jsonrpc_api::JsonRpcPlayerDto;
use crate::jsonrpc_methods::{ClientInfo, JsonRpcMethodMessage};

pub const SERVER_VERSION_NAME_26_1_2: &str = "26.1.2";
pub const SERVER_PROTOCOL_VERSION_26_1_2: i32 = 775;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcServerVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcServerState {
    pub started: bool,
    pub players: Vec<JsonRpcPlayerDto>,
    pub version: JsonRpcServerVersion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcSystemMessage {
    pub message: JsonRpcMethodMessage,
    pub overlay: bool,
    pub receiving_players: Option<Vec<JsonRpcPlayerDto>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcOnlinePlayer {
    pub player: JsonRpcPlayerDto,
    pub messages: Vec<JsonRpcDeliveredSystemMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcDeliveredSystemMessage {
    pub component: Component,
    pub overlay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcServerStateEvent {
    SaveEverything {
        suppress_logs: bool,
        flush: bool,
        force: bool,
        client_info: ClientInfo,
    },
    Halt {
        wait_for_shutdown: bool,
        client_info: ClientInfo,
    },
    BroadcastSystemMessage {
        component: Box<Component>,
        overlay: bool,
        client_info: ClientInfo,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcServerStateContext {
    pub ready: bool,
    pub players: Vec<JsonRpcOnlinePlayer>,
    pub save_result: bool,
    pub events: Vec<JsonRpcServerStateEvent>,
}

pub struct ServerStateService;

impl JsonRpcServerVersion {
    pub fn current() -> Self {
        Self {
            name: SERVER_VERSION_NAME_26_1_2.to_string(),
            protocol: SERVER_PROTOCOL_VERSION_26_1_2,
        }
    }
}

impl JsonRpcServerState {
    pub fn not_started() -> Self {
        Self {
            started: false,
            players: Vec::new(),
            version: JsonRpcServerVersion::current(),
        }
    }
}

impl JsonRpcSystemMessage {
    pub fn new(
        message: JsonRpcMethodMessage,
        overlay: bool,
        receiving_players: Option<Vec<JsonRpcPlayerDto>>,
    ) -> Self {
        Self {
            message,
            overlay,
            receiving_players,
        }
    }
}

impl JsonRpcOnlinePlayer {
    pub fn new(player: JsonRpcPlayerDto) -> Self {
        Self {
            player,
            messages: Vec::new(),
        }
    }
}

impl JsonRpcServerStateContext {
    pub fn new(ready: bool, players: Vec<JsonRpcOnlinePlayer>) -> Self {
        Self {
            ready,
            players,
            save_result: true,
            events: Vec::new(),
        }
    }

    fn player_dtos(&self) -> Vec<JsonRpcPlayerDto> {
        self.players
            .iter()
            .map(|player| player.player.clone())
            .collect()
    }

    fn get_player_index(&self, player: &JsonRpcPlayerDto) -> Option<usize> {
        if let Some(id) = &player.id {
            self.players
                .iter()
                .position(|online| online.player.id.as_ref() == Some(id))
        } else {
            player.name.as_ref().and_then(|name| {
                self.players
                    .iter()
                    .position(|online| online.player.name.as_ref() == Some(name))
            })
        }
    }
}

impl ServerStateService {
    pub fn status(context: &JsonRpcServerStateContext) -> JsonRpcServerState {
        if !context.ready {
            JsonRpcServerState::not_started()
        } else {
            JsonRpcServerState {
                started: true,
                players: context.player_dtos(),
                version: JsonRpcServerVersion::current(),
            }
        }
    }

    pub fn save(
        context: &mut JsonRpcServerStateContext,
        flush: bool,
        client_info: ClientInfo,
    ) -> bool {
        context.events.push(JsonRpcServerStateEvent::SaveEverything {
            suppress_logs: true,
            flush,
            force: true,
            client_info,
        });
        context.save_result
    }

    pub fn stop(context: &mut JsonRpcServerStateContext, client_info: ClientInfo) -> bool {
        context.events.push(JsonRpcServerStateEvent::Halt {
            wait_for_shutdown: false,
            client_info,
        });
        true
    }

    pub fn system_message(
        context: &mut JsonRpcServerStateContext,
        system_message: &JsonRpcSystemMessage,
        client_info: ClientInfo,
    ) -> bool {
        let Some(component) = system_message.message.as_component() else {
            return false;
        };

        match &system_message.receiving_players {
            Some(receiving_players) if receiving_players.is_empty() => false,
            Some(receiving_players) => {
                for player_dto in receiving_players {
                    if let Some(index) = context.get_player_index(player_dto) {
                        context.players[index]
                            .messages
                            .push(JsonRpcDeliveredSystemMessage {
                                component: component.clone(),
                                overlay: system_message.overlay,
                            });
                    }
                }
                true
            }
            None => {
                context
                    .events
                    .push(JsonRpcServerStateEvent::BroadcastSystemMessage {
                        component: Box::new(component),
                        overlay: system_message.overlay,
                        client_info,
                    });
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_matches_java_not_started_and_ready_shapes() {
        let not_ready = JsonRpcServerStateContext::new(false, vec![online_player("Steve")]);
        assert_eq!(ServerStateService::status(&not_ready), JsonRpcServerState::not_started());

        let ready = JsonRpcServerStateContext::new(true, vec![online_player("Steve")]);
        assert_eq!(
            ServerStateService::status(&ready),
            JsonRpcServerState {
                started: true,
                players: vec![player(None, "Steve")],
                version: JsonRpcServerVersion::current(),
            }
        );
    }

    #[test]
    fn save_and_stop_match_java_forwarding_arguments() {
        let mut context = JsonRpcServerStateContext::new(true, Vec::new());
        context.save_result = false;

        assert!(!ServerStateService::save(&mut context, true, ClientInfo::of(12)));
        assert!(ServerStateService::stop(&mut context, ClientInfo::of(13)));
        assert_eq!(
            context.events,
            vec![
                JsonRpcServerStateEvent::SaveEverything {
                    suppress_logs: true,
                    flush: true,
                    force: true,
                    client_info: ClientInfo::of(12),
                },
                JsonRpcServerStateEvent::Halt {
                    wait_for_shutdown: false,
                    client_info: ClientInfo::of(13),
                },
            ]
        );
    }

    #[test]
    fn system_message_matches_java_broadcast_and_targeted_delivery() {
        let mut context = JsonRpcServerStateContext::new(
            true,
            vec![online_player_with_id("11111111-1111-1111-1111-111111111111", "Steve")],
        );
        let broadcast = JsonRpcSystemMessage::new(JsonRpcMethodMessage::literal("Server"), false, None);
        assert!(ServerStateService::system_message(
            &mut context,
            &broadcast,
            ClientInfo::of(14),
        ));
        assert_eq!(
            context.events,
            vec![JsonRpcServerStateEvent::BroadcastSystemMessage {
                component: Box::new(Component::literal("Server")),
                overlay: false,
                client_info: ClientInfo::of(14),
            }]
        );

        let targeted = JsonRpcSystemMessage::new(
            JsonRpcMethodMessage::translatable("chat.type.text", None),
            true,
            Some(vec![JsonRpcPlayerDto::new(
                Some("11111111-1111-1111-1111-111111111111".to_string()),
                Some("WrongName".to_string()),
            )]),
        );
        assert!(ServerStateService::system_message(
            &mut context,
            &targeted,
            ClientInfo::of(15),
        ));
        assert_eq!(
            context.players[0].messages,
            vec![JsonRpcDeliveredSystemMessage {
                component: Component::translatable("chat.type.text", Vec::new()),
                overlay: true,
            }]
        );
    }

    #[test]
    fn system_message_rejects_empty_or_componentless_messages() {
        let mut context = JsonRpcServerStateContext::new(true, vec![online_player("Steve")]);
        let no_component = JsonRpcSystemMessage::new(
            JsonRpcMethodMessage::new(None, None, None),
            false,
            None,
        );
        assert!(!ServerStateService::system_message(
            &mut context,
            &no_component,
            ClientInfo::of(16),
        ));

        let empty_targets = JsonRpcSystemMessage::new(
            JsonRpcMethodMessage::literal("Nobody"),
            false,
            Some(Vec::new()),
        );
        assert!(!ServerStateService::system_message(
            &mut context,
            &empty_targets,
            ClientInfo::of(17),
        ));
        assert_eq!(context.events, Vec::<JsonRpcServerStateEvent>::new());
        assert_eq!(context.players[0].messages, Vec::<JsonRpcDeliveredSystemMessage>::new());
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn server_state_service_source_matches_java_26_1_2() {
        const SERVER_STATE_SERVICE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/ServerStateService.java");

        for sentinel in [
            "public static ServerStateService.ServerState status(final MinecraftApi minecraftApi)",
            "return !minecraftApi.serverStateService().isReady()",
            "? ServerStateService.ServerState.NOT_STARTED",
            ": new ServerStateService.ServerState(true, PlayerService.get(minecraftApi), ServerStatus.Version.current());",
            "return minecraftApi.serverStateService().saveEverything(true, flush, true, clientInfo);",
            "minecraftApi.submit(() -> minecraftApi.serverStateService().halt(false, clientInfo));",
            "return true;",
            "Component component = systemMessage.message().asComponent().orElse(null);",
            "if (component == null) {",
            "if (systemMessage.receivingPlayers().get().isEmpty()) {",
            "if (playerDto.id().isPresent())",
            "player = minecraftApi.playerListService().getPlayer(playerDto.id().get());",
            "player = minecraftApi.playerListService().getPlayerByName(playerDto.name().get());",
            "player.sendSystemMessage(component, systemMessage.overlay());",
            "minecraftApi.serverStateService().broadcastSystemMessage(component, systemMessage.overlay(), clientInfo);",
            "public record ServerState(boolean started, List<PlayerDto> players, ServerStatus.Version version)",
            "public static final ServerStateService.ServerState NOT_STARTED = new ServerStateService.ServerState(false, List.of(), ServerStatus.Version.current());",
            "public record SystemMessage(Message message, boolean overlay, Optional<List<PlayerDto>> receivingPlayers)",
            "PlayerDto.CODEC.codec().listOf().lenientOptionalFieldOf(\"receivingPlayers\").forGetter(ServerStateService.SystemMessage::receivingPlayers)",
        ] {
            assert!(
                SERVER_STATE_SERVICE.contains(sentinel),
                "ServerStateService.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn online_player(name: &str) -> JsonRpcOnlinePlayer {
        JsonRpcOnlinePlayer::new(player(None, name))
    }

    fn online_player_with_id(id: &str, name: &str) -> JsonRpcOnlinePlayer {
        JsonRpcOnlinePlayer::new(JsonRpcPlayerDto::new(
            Some(id.to_string()),
            Some(name.to_string()),
        ))
    }

    fn player(id: Option<&str>, name: &str) -> JsonRpcPlayerDto {
        JsonRpcPlayerDto::new(id.map(ToString::to_string), Some(name.to_string()))
    }
}
