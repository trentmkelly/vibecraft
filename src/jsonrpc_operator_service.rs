#![allow(dead_code)]

use crate::command::PermissionLevel;
use crate::jsonrpc_api::JsonRpcPlayerDto;
use crate::jsonrpc_methods::ClientInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcOperatorDto {
    pub player: JsonRpcPlayerDto,
    pub permission_level: Option<PermissionLevel>,
    pub bypasses_player_limit: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcOperatorOp {
    pub user: JsonRpcPlayerDto,
    pub permission_level: Option<PermissionLevel>,
    pub bypasses_player_limit: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcUserDirectory {
    pub users: Vec<JsonRpcPlayerDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcOperatorList {
    pub entries: Vec<JsonRpcOperatorOp>,
    pub events: Vec<JsonRpcOperatorEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcOperatorEvent {
    Clear { client_info: ClientInfo },
    Op {
        operator: JsonRpcOperatorOp,
        client_info: ClientInfo,
    },
    Deop {
        user: JsonRpcPlayerDto,
        client_info: ClientInfo,
    },
}

pub struct OperatorService;

impl JsonRpcOperatorDto {
    pub fn new(
        player: JsonRpcPlayerDto,
        permission_level: Option<PermissionLevel>,
        bypasses_player_limit: Option<bool>,
    ) -> Self {
        Self {
            player,
            permission_level,
            bypasses_player_limit,
        }
    }

    fn from_op(op: &JsonRpcOperatorOp) -> Self {
        Self {
            player: op.user.clone(),
            permission_level: op.permission_level,
            bypasses_player_limit: op.bypasses_player_limit,
        }
    }

    fn to_op(&self, user: JsonRpcPlayerDto) -> JsonRpcOperatorOp {
        JsonRpcOperatorOp {
            user,
            permission_level: self.permission_level,
            bypasses_player_limit: self.bypasses_player_limit,
        }
    }
}

impl JsonRpcUserDirectory {
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

impl JsonRpcOperatorList {
    pub fn new(entries: Vec<JsonRpcOperatorOp>) -> Self {
        Self {
            entries,
            events: Vec::new(),
        }
    }

    fn clear(&mut self, client_info: ClientInfo) {
        self.entries.clear();
        self.events.push(JsonRpcOperatorEvent::Clear { client_info });
    }

    fn deop(&mut self, user: &JsonRpcPlayerDto, client_info: ClientInfo) {
        self.entries.retain(|entry| entry.user != *user);
        self.events.push(JsonRpcOperatorEvent::Deop {
            user: user.clone(),
            client_info,
        });
    }

    fn op(&mut self, operator: JsonRpcOperatorOp, client_info: ClientInfo) {
        if !self.entries.contains(&operator) {
            self.entries.push(operator.clone());
        }
        self.events
            .push(JsonRpcOperatorEvent::Op { operator, client_info });
    }
}

impl OperatorService {
    pub fn get(operator_list: &JsonRpcOperatorList) -> Vec<JsonRpcOperatorDto> {
        operator_list
            .entries
            .iter()
            .map(JsonRpcOperatorDto::from_op)
            .collect()
    }

    pub fn clear(
        operator_list: &mut JsonRpcOperatorList,
        client_info: ClientInfo,
    ) -> Vec<JsonRpcOperatorDto> {
        operator_list.clear(client_info);
        Self::get(operator_list)
    }

    pub fn remove(
        users: &JsonRpcUserDirectory,
        operator_list: &mut JsonRpcOperatorList,
        player_dtos: &[JsonRpcPlayerDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcOperatorDto> {
        for player_dto in player_dtos {
            if let Some(user) = users.get_user(player_dto) {
                operator_list.deop(&user, client_info);
            }
        }

        Self::get(operator_list)
    }

    pub fn add(
        users: &JsonRpcUserDirectory,
        operator_list: &mut JsonRpcOperatorList,
        operators: &[JsonRpcOperatorDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcOperatorDto> {
        for operator in operators {
            if let Some(user) = users.get_user(&operator.player) {
                operator_list.op(operator.to_op(user), client_info);
            }
        }

        Self::get(operator_list)
    }

    pub fn set(
        users: &JsonRpcUserDirectory,
        operator_list: &mut JsonRpcOperatorList,
        operators: &[JsonRpcOperatorDto],
        client_info: ClientInfo,
    ) -> Vec<JsonRpcOperatorDto> {
        let final_operators = resolved_unique_operators(users, operators);
        let current_operators = unique_operators(&operator_list.entries);

        for current in &current_operators {
            if !final_operators.contains(current) {
                operator_list.deop(&current.user, client_info);
            }
        }
        for final_operator in final_operators {
            if !current_operators.contains(&final_operator) {
                operator_list.op(final_operator, client_info);
            }
        }

        Self::get(operator_list)
    }
}

fn resolved_unique_operators(
    users: &JsonRpcUserDirectory,
    operators: &[JsonRpcOperatorDto],
) -> Vec<JsonRpcOperatorOp> {
    let mut resolved = Vec::new();
    for operator in operators {
        if let Some(user) = users.get_user(&operator.player) {
            let op = operator.to_op(user);
            if !resolved.contains(&op) {
                resolved.push(op);
            }
        }
    }
    resolved
}

fn unique_operators(operators: &[JsonRpcOperatorOp]) -> Vec<JsonRpcOperatorOp> {
    let mut unique = Vec::new();
    for operator in operators {
        if !unique.contains(operator) {
            unique.push(operator.clone());
        }
    }
    unique
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_clear_match_java_operator_dto_mapping() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let mut operators = JsonRpcOperatorList::new(vec![op(
            steve.clone(),
            Some(PermissionLevel::Admins),
            Some(true),
        )]);

        assert_eq!(
            OperatorService::get(&operators),
            vec![JsonRpcOperatorDto::new(
                steve,
                Some(PermissionLevel::Admins),
                Some(true),
            )]
        );
        assert_eq!(
            OperatorService::clear(&mut operators, ClientInfo::of(7)),
            Vec::<JsonRpcOperatorDto>::new()
        );
        assert_eq!(
            operators.events,
            vec![JsonRpcOperatorEvent::Clear {
                client_info: ClientInfo::of(7),
            }]
        );
    }

    #[test]
    fn add_and_remove_resolve_users_before_mutating_operator_list() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex");
        let users = JsonRpcUserDirectory::new(vec![steve.clone(), alex.clone()]);
        let mut operators = JsonRpcOperatorList::default();

        let after_add = OperatorService::add(
            &users,
            &mut operators,
            &[
                JsonRpcOperatorDto::new(
                    JsonRpcPlayerDto::new(None, Some("Steve".to_string())),
                    Some(PermissionLevel::Admins),
                    Some(false),
                ),
                JsonRpcOperatorDto::new(
                    JsonRpcPlayerDto::new(None, Some("Missing".to_string())),
                    Some(PermissionLevel::Owners),
                    Some(true),
                ),
            ],
            ClientInfo::of(8),
        );
        assert_eq!(
            after_add,
            vec![JsonRpcOperatorDto::new(
                steve.clone(),
                Some(PermissionLevel::Admins),
                Some(false),
            )]
        );

        let after_remove = OperatorService::remove(
            &users,
            &mut operators,
            &[JsonRpcPlayerDto::new(
                Some("11111111-1111-1111-1111-111111111111".to_string()),
                Some("WrongName".to_string()),
            )],
            ClientInfo::of(9),
        );
        assert_eq!(after_remove, Vec::<JsonRpcOperatorDto>::new());
        assert_eq!(operators.entries, Vec::<JsonRpcOperatorOp>::new());
        assert_eq!(
            operators.events,
            vec![
                JsonRpcOperatorEvent::Op {
                    operator: op(steve.clone(), Some(PermissionLevel::Admins), Some(false)),
                    client_info: ClientInfo::of(8),
                },
                JsonRpcOperatorEvent::Deop {
                    user: steve,
                    client_info: ClientInfo::of(9),
                },
            ]
        );
    }

    #[test]
    fn set_matches_java_set_difference_on_user_permission_and_bypass() {
        let steve = player("11111111-1111-1111-1111-111111111111", "Steve");
        let alex = player("22222222-2222-2222-2222-222222222222", "Alex");
        let users = JsonRpcUserDirectory::new(vec![steve.clone(), alex.clone()]);
        let mut operators = JsonRpcOperatorList::new(vec![
            op(steve.clone(), Some(PermissionLevel::Gamemasters), Some(false)),
            op(alex.clone(), Some(PermissionLevel::Admins), Some(true)),
        ]);

        let result = OperatorService::set(
            &users,
            &mut operators,
            &[JsonRpcOperatorDto::new(
                JsonRpcPlayerDto::new(
                    Some("11111111-1111-1111-1111-111111111111".to_string()),
                    Some("Steve".to_string()),
                ),
                Some(PermissionLevel::Owners),
                Some(false),
            )],
            ClientInfo::of(10),
        );

        assert_eq!(
            result,
            vec![JsonRpcOperatorDto::new(
                steve.clone(),
                Some(PermissionLevel::Owners),
                Some(false),
            )]
        );
        assert_eq!(
            operators.events,
            vec![
                JsonRpcOperatorEvent::Deop {
                    user: steve.clone(),
                    client_info: ClientInfo::of(10),
                },
                JsonRpcOperatorEvent::Deop {
                    user: alex,
                    client_info: ClientInfo::of(10),
                },
                JsonRpcOperatorEvent::Op {
                    operator: op(steve, Some(PermissionLevel::Owners), Some(false)),
                    client_info: ClientInfo::of(10),
                },
            ]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn operator_service_source_matches_java_26_1_2() {
        const OPERATOR_SERVICE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/OperatorService.java");

        for sentinel in [
            "public static List<OperatorService.OperatorDto> get(final MinecraftApi minecraftApi)",
            "return minecraftApi.operatorListService().getEntries().stream().filter(u -> u.getUser() != null).map(OperatorService.OperatorDto::from).toList();",
            "minecraftApi.operatorListService().clear(clientInfo);",
            "minecraftApi.playerListService().getUser(playerDto.id(), playerDto.name())",
            "user.ifPresent(nameAndId -> minecraftApi.operatorListService().deop(nameAndId, clientInfo));",
            "thenApply(user -> user.map(nameAndId -> new OperatorService.Op(nameAndId, operator.permissionLevel(), operator.bypassesPlayerLimit())))",
            "operator -> minecraftApi.operatorListService().op(operator.user(), operator.permissionLevel(), operator.bypassesPlayerLimit(), clientInfo)",
            "Set<OperatorService.Op> finalOperators = Util.sequence(fetch).join().stream().flatMap(Optional::stream).collect(Collectors.toSet());",
            "Set<OperatorService.Op> currentOperators = minecraftApi.operatorListService()",
            "filter(operator -> !finalOperators.contains(operator))",
            "forEach(operator -> minecraftApi.operatorListService().deop(operator.user(), clientInfo));",
            "filter(operator -> !currentOperators.contains(operator))",
            "record Op(NameAndId user, Optional<PermissionLevel> permissionLevel, Optional<Boolean> bypassesPlayerLimit)",
            "public record OperatorDto(PlayerDto player, Optional<PermissionLevel> permissionLevel, Optional<Boolean> bypassesPlayerLimit)",
            "PermissionLevel.INT_CODEC.optionalFieldOf(\"permissionLevel\").forGetter(OperatorService.OperatorDto::permissionLevel)",
            "Codec.BOOL.optionalFieldOf(\"bypassesPlayerLimit\").forGetter(OperatorService.OperatorDto::bypassesPlayerLimit)",
            "PlayerDto.from(Objects.requireNonNull(serverOpListEntry.getUser()))",
            "Optional.of(serverOpListEntry.permissions().level())",
            "Optional.of(serverOpListEntry.getBypassesPlayerLimit())",
        ] {
            assert!(
                OPERATOR_SERVICE.contains(sentinel),
                "OperatorService.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn player(id: &str, name: &str) -> JsonRpcPlayerDto {
        JsonRpcPlayerDto::new(Some(id.to_string()), Some(name.to_string()))
    }

    fn op(
        user: JsonRpcPlayerDto,
        permission_level: Option<PermissionLevel>,
        bypasses_player_limit: Option<bool>,
    ) -> JsonRpcOperatorOp {
        JsonRpcOperatorOp {
            user,
            permission_level,
            bypasses_player_limit,
        }
    }
}
