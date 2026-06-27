#![allow(dead_code)]

use crate::chat_component::Component;
use crate::jsonrpc_api::JsonRpcPlayerDto;
use crate::jsonrpc_methods::{ClientInfo, JsonRpcMethodMessage};

pub const DEFAULT_KICK_MESSAGE_KEY: &str = "multiplayer.disconnect.kicked";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcManagedPlayer {
    pub player: JsonRpcPlayerDto,
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

pub struct PlayerService;

impl JsonRpcManagedPlayer {
    pub fn new(id: Option<String>, name: Option<String>) -> Self {
        Self {
            player: JsonRpcPlayerDto::new(id, name),
        }
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
