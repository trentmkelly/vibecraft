#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::chat_component::{Component, ComponentArgument};
use crate::command::{LevelBasedPermissionSet, PermissionLevel};
use crate::game_rules::{
    GameRuleDefinition, GameRuleError, GameRuleType, GameRuleValue, GameRules,
    vanilla_game_rules,
};
use crate::jsonrpc_api::{JsonRpcSchema, MethodInfo, NamedMethodInfo, SchemaComponent};
use crate::network::play::GameMode;
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcRuntimeExceptionKind {
    Encode,
    InvalidParameter,
    InvalidRequest,
    MethodNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcRuntimeException {
    pub kind: JsonRpcRuntimeExceptionKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRpcErrorException {
    id: String,
    error: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcMethodMessage {
    pub literal: Option<String>,
    pub translatable: Option<String>,
    pub translatable_params: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientInfo {
    pub connection_id: i32,
}

pub struct JsonRpcLogger;

impl JsonRpcLogger {
    pub const PREFIX: &'static str = "RPC Connection #{}: ";

    pub fn log_message(client_info: ClientInfo, message: &str, args: &[&str]) -> (String, Vec<String>) {
        if args.is_empty() {
            (
                format!("RPC Connection #{{}}: {message}"),
                vec![client_info.connection_id.to_string()],
            )
        } else {
            let mut all_args = Vec::with_capacity(args.len() + 1);
            all_args.push(client_info.connection_id.to_string());
            all_args.extend(args.iter().map(|arg| (*arg).to_string()));
            (format!("RPC Connection #{{}}: {message}"), all_args)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcMethodAttributes {
    pub discoverable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverableJsonRpcMethod {
    pub id: Identifier,
    pub info: MethodInfo,
    pub attributes: JsonRpcMethodAttributes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverComponents {
    pub schemas: BTreeMap<String, JsonRpcSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverInfo {
    pub title: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverResponse {
    pub json_rpc_protocol_version: String,
    pub discover_info: DiscoverInfo,
    pub methods: Vec<NamedMethodInfo>,
    pub components: DiscoverComponents,
}

pub struct DiscoveryService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonRpcGameRuleUpdate {
    pub game_rule: GameRuleDefinition,
    pub value: GameRuleValue,
}

pub struct GameRulesService;

pub trait MinecraftGameRuleService {
    fn update_game_rule(
        &mut self,
        update: JsonRpcGameRuleUpdate,
        client_info: ClientInfo,
    ) -> Result<JsonRpcGameRuleUpdate, GameRuleError>;
    fn get_rule_value(&self, game_rule: GameRuleDefinition) -> Option<GameRuleValue>;
    fn get_typed_rule(
        &self,
        game_rule: GameRuleDefinition,
        value: GameRuleValue,
    ) -> JsonRpcGameRuleUpdate;
    fn get_available_game_rules(&self) -> Vec<GameRuleDefinition>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonRpcDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcServerSettings {
    pub autosave: bool,
    pub difficulty: JsonRpcDifficulty,
    pub enforce_allowlist: bool,
    pub using_allowlist: bool,
    pub max_players: i32,
    pub pause_when_empty: i32,
    pub player_idle_timeout: i32,
    pub allow_flight: bool,
    pub spawn_protection: i32,
    pub motd: String,
    pub force_game_mode: bool,
    pub game_mode: GameMode,
    pub view_distance: i32,
    pub simulation_distance: i32,
    pub accept_transfers: bool,
    pub status_heartbeat_interval: i32,
    pub operator_user_permissions: LevelBasedPermissionSet,
    pub hides_online_players: bool,
    pub replies_to_status: bool,
    pub entity_broadcast_range_percentage: i32,
}

pub struct ServerSettingsService;

impl JsonRpcRuntimeException {
    pub fn encode(message: impl Into<String>) -> Self {
        Self::new(JsonRpcRuntimeExceptionKind::Encode, message)
    }

    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::new(JsonRpcRuntimeExceptionKind::InvalidParameter, message)
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(JsonRpcRuntimeExceptionKind::InvalidRequest, message)
    }

    pub fn method_not_found(message: impl Into<String>) -> Self {
        Self::new(JsonRpcRuntimeExceptionKind::MethodNotFound, message)
    }

    fn new(kind: JsonRpcRuntimeExceptionKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl RemoteRpcErrorException {
    pub fn new(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            error: error.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn error(&self) -> &str {
        &self.error
    }
}

impl JsonRpcMethodMessage {
    pub fn new(
        literal: Option<String>,
        translatable: Option<String>,
        translatable_params: Option<Vec<String>>,
    ) -> Self {
        Self {
            literal,
            translatable,
            translatable_params,
        }
    }

    pub fn literal(value: impl Into<String>) -> Self {
        Self::new(Some(value.into()), None, None)
    }

    pub fn translatable(key: impl Into<String>, params: Option<Vec<String>>) -> Self {
        Self::new(None, Some(key.into()), params)
    }

    pub fn as_component(&self) -> Option<Component> {
        match &self.translatable {
            Some(key) => {
                let args = match &self.translatable_params {
                    Some(params) => params
                        .iter()
                        .map(|param| ComponentArgument::String(param.clone()))
                        .collect(),
                    None => Vec::new(),
                };
                Some(Component::translatable(key.clone(), args))
            }
            None => self.literal.clone().map(Component::literal),
        }
    }
}

impl ClientInfo {
    pub fn of(connection_id: i32) -> Self {
        Self { connection_id }
    }
}

impl JsonRpcMethodAttributes {
    pub fn new(discoverable: bool) -> Self {
        Self { discoverable }
    }
}

impl DiscoverableJsonRpcMethod {
    pub fn new(id: Identifier, info: MethodInfo, discoverable: bool) -> Self {
        Self {
            id,
            info,
            attributes: JsonRpcMethodAttributes::new(discoverable),
        }
    }
}

impl DiscoveryService {
    pub const OPENRPC_VERSION: &'static str = "1.3.2";
    pub const TITLE: &'static str = "Minecraft Server JSON-RPC";
    pub const VERSION: &'static str = "2.0.0";

    pub fn discover(
        schema_registry: &[SchemaComponent],
        incoming_methods: &[DiscoverableJsonRpcMethod],
        outgoing_methods: &[DiscoverableJsonRpcMethod],
    ) -> DiscoverResponse {
        let mut methods = Vec::with_capacity(incoming_methods.len() + outgoing_methods.len());
        add_discoverable_methods(&mut methods, incoming_methods);
        add_discoverable_methods(&mut methods, outgoing_methods);

        let schemas = schema_registry
            .iter()
            .map(|component| (component.name.clone(), component.schema.info()))
            .collect();

        DiscoverResponse {
            json_rpc_protocol_version: Self::OPENRPC_VERSION.to_string(),
            discover_info: DiscoverInfo {
                title: Self::TITLE.to_string(),
                version: Self::VERSION.to_string(),
            },
            methods,
            components: DiscoverComponents { schemas },
        }
    }
}

fn add_discoverable_methods(
    output: &mut Vec<NamedMethodInfo>,
    methods: &[DiscoverableJsonRpcMethod],
) {
    output.extend(
        methods
            .iter()
            .filter(|method| method.attributes.discoverable)
            .map(|method| method.info.named(method.id.clone())),
    );
}

impl JsonRpcGameRuleUpdate {
    pub fn new(game_rule: GameRuleDefinition, value: GameRuleValue) -> Self {
        Self { game_rule, value }
    }

    pub fn from_untyped(
        game_rule: GameRuleDefinition,
        read_type: GameRuleType,
        value: GameRuleValue,
    ) -> Result<Self, JsonRpcRuntimeException> {
        if game_rule.rule_type != read_type {
            Err(JsonRpcRuntimeException::invalid_parameter(format!(
                "Stated type \"{}\" mismatches with actual type \"{}\" of gamerule \"{}\"",
                game_rule_type_name(read_type),
                game_rule_type_name(game_rule.rule_type),
                game_rule.name
            )))
        } else {
            Ok(Self::new(game_rule, value))
        }
    }
}

impl GameRulesService {
    pub fn get(game_rules: &GameRules) -> Vec<JsonRpcGameRuleUpdate> {
        let mut rules = Vec::new();
        for game_rule in vanilla_game_rules() {
            if let Some(value) = game_rules.get(game_rule.name) {
                Self::add_game_rule(*game_rule, value, &mut rules);
            }
        }
        rules
    }

    fn add_game_rule(
        game_rule: GameRuleDefinition,
        value: GameRuleValue,
        rules: &mut Vec<JsonRpcGameRuleUpdate>,
    ) {
        rules.push(Self::get_typed_rule(game_rule, value));
    }

    pub fn get_typed_rule(
        game_rule: GameRuleDefinition,
        value: GameRuleValue,
    ) -> JsonRpcGameRuleUpdate {
        JsonRpcGameRuleUpdate::new(game_rule, value)
    }

    pub fn update(
        game_rules: &mut GameRules,
        update: JsonRpcGameRuleUpdate,
        _client_info: ClientInfo,
    ) -> Result<JsonRpcGameRuleUpdate, GameRuleError> {
        game_rules.set(update.game_rule.name, update.value.sync_value().as_str())?;
        Ok(update)
    }
}

impl MinecraftGameRuleService for GameRules {
    fn update_game_rule(
        &mut self,
        update: JsonRpcGameRuleUpdate,
        client_info: ClientInfo,
    ) -> Result<JsonRpcGameRuleUpdate, GameRuleError> {
        GameRulesService::update(self, update, client_info)
    }

    fn get_rule_value(&self, game_rule: GameRuleDefinition) -> Option<GameRuleValue> {
        self.get(game_rule.name)
    }

    fn get_typed_rule(
        &self,
        game_rule: GameRuleDefinition,
        value: GameRuleValue,
    ) -> JsonRpcGameRuleUpdate {
        GameRulesService::get_typed_rule(game_rule, value)
    }

    fn get_available_game_rules(&self) -> Vec<GameRuleDefinition> {
        vanilla_game_rules()
            .iter()
            .filter(|game_rule| self.get(game_rule.name).is_some())
            .copied()
            .collect()
    }
}

fn game_rule_type_name(rule_type: GameRuleType) -> &'static str {
    match rule_type {
        GameRuleType::Int => "integer",
        GameRuleType::Bool => "boolean",
    }
}

impl Default for JsonRpcServerSettings {
    fn default() -> Self {
        Self {
            autosave: true,
            difficulty: JsonRpcDifficulty::Normal,
            enforce_allowlist: false,
            using_allowlist: false,
            max_players: 20,
            pause_when_empty: 60,
            player_idle_timeout: 0,
            allow_flight: false,
            spawn_protection: 16,
            motd: "A Minecraft Server".to_string(),
            force_game_mode: false,
            game_mode: GameMode::Survival,
            view_distance: 10,
            simulation_distance: 10,
            accept_transfers: false,
            status_heartbeat_interval: 0,
            operator_user_permissions: LevelBasedPermissionSet::GAMEMASTER,
            hides_online_players: false,
            replies_to_status: true,
            entity_broadcast_range_percentage: 100,
        }
    }
}

impl ServerSettingsService {
    pub fn autosave(settings: &JsonRpcServerSettings) -> bool {
        settings.autosave
    }

    pub fn set_autosave(
        settings: &mut JsonRpcServerSettings,
        enabled: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.autosave = enabled;
        settings.autosave
    }

    pub fn difficulty(settings: &JsonRpcServerSettings) -> JsonRpcDifficulty {
        settings.difficulty
    }

    pub fn set_difficulty(
        settings: &mut JsonRpcServerSettings,
        difficulty: JsonRpcDifficulty,
        _client_info: ClientInfo,
    ) -> JsonRpcDifficulty {
        settings.difficulty = difficulty;
        settings.difficulty
    }

    pub fn enforce_allowlist(settings: &JsonRpcServerSettings) -> bool {
        settings.enforce_allowlist
    }

    pub fn set_enforce_allowlist(
        settings: &mut JsonRpcServerSettings,
        enforce: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.enforce_allowlist = enforce;
        settings.enforce_allowlist
    }

    pub fn using_allowlist(settings: &JsonRpcServerSettings) -> bool {
        settings.using_allowlist
    }

    pub fn set_using_allowlist(
        settings: &mut JsonRpcServerSettings,
        use_allowlist: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.using_allowlist = use_allowlist;
        settings.using_allowlist
    }

    pub fn max_players(settings: &JsonRpcServerSettings) -> i32 {
        settings.max_players
    }

    pub fn set_max_players(
        settings: &mut JsonRpcServerSettings,
        max_players: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.max_players = max_players;
        settings.max_players
    }

    pub fn pause_when_empty(settings: &JsonRpcServerSettings) -> i32 {
        settings.pause_when_empty
    }

    pub fn set_pause_when_empty(
        settings: &mut JsonRpcServerSettings,
        empty_seconds: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.pause_when_empty = empty_seconds;
        settings.pause_when_empty
    }

    pub fn player_idle_timeout(settings: &JsonRpcServerSettings) -> i32 {
        settings.player_idle_timeout
    }

    pub fn set_player_idle_timeout(
        settings: &mut JsonRpcServerSettings,
        idle_time: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.player_idle_timeout = idle_time;
        settings.player_idle_timeout
    }

    pub fn allow_flight(settings: &JsonRpcServerSettings) -> bool {
        settings.allow_flight
    }

    pub fn set_allow_flight(
        settings: &mut JsonRpcServerSettings,
        allow: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.allow_flight = allow;
        settings.allow_flight
    }

    pub fn spawn_protection(settings: &JsonRpcServerSettings) -> i32 {
        settings.spawn_protection
    }

    pub fn set_spawn_protection(
        settings: &mut JsonRpcServerSettings,
        spawn_protection: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.spawn_protection = spawn_protection;
        settings.spawn_protection
    }

    pub fn motd(settings: &JsonRpcServerSettings) -> String {
        settings.motd.clone()
    }

    pub fn set_motd(
        settings: &mut JsonRpcServerSettings,
        motd: impl Into<String>,
        _client_info: ClientInfo,
    ) -> String {
        settings.motd = motd.into();
        settings.motd.clone()
    }

    pub fn force_game_mode(settings: &JsonRpcServerSettings) -> bool {
        settings.force_game_mode
    }

    pub fn set_force_game_mode(
        settings: &mut JsonRpcServerSettings,
        force: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.force_game_mode = force;
        settings.force_game_mode
    }

    pub fn game_mode(settings: &JsonRpcServerSettings) -> GameMode {
        settings.game_mode
    }

    pub fn set_game_mode(
        settings: &mut JsonRpcServerSettings,
        game_mode: GameMode,
        _client_info: ClientInfo,
    ) -> GameMode {
        settings.game_mode = game_mode;
        settings.game_mode
    }

    pub fn view_distance(settings: &JsonRpcServerSettings) -> i32 {
        settings.view_distance
    }

    pub fn set_view_distance(
        settings: &mut JsonRpcServerSettings,
        view_distance: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.view_distance = view_distance;
        settings.view_distance
    }

    pub fn simulation_distance(settings: &JsonRpcServerSettings) -> i32 {
        settings.simulation_distance
    }

    pub fn set_simulation_distance(
        settings: &mut JsonRpcServerSettings,
        simulation_distance: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.simulation_distance = simulation_distance;
        settings.simulation_distance
    }

    pub fn accept_transfers(settings: &JsonRpcServerSettings) -> bool {
        settings.accept_transfers
    }

    pub fn set_accept_transfers(
        settings: &mut JsonRpcServerSettings,
        accept: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.accept_transfers = accept;
        settings.accept_transfers
    }

    pub fn status_heartbeat_interval(settings: &JsonRpcServerSettings) -> i32 {
        settings.status_heartbeat_interval
    }

    pub fn set_status_heartbeat_interval(
        settings: &mut JsonRpcServerSettings,
        status_heartbeat_interval: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.status_heartbeat_interval = status_heartbeat_interval;
        settings.status_heartbeat_interval
    }

    pub fn operator_user_permission_level(settings: &JsonRpcServerSettings) -> PermissionLevel {
        settings.operator_user_permissions.level()
    }

    pub fn set_operator_user_permission_level(
        settings: &mut JsonRpcServerSettings,
        level: PermissionLevel,
        _client_info: ClientInfo,
    ) -> PermissionLevel {
        settings.operator_user_permissions = LevelBasedPermissionSet::new(level);
        settings.operator_user_permissions.level()
    }

    pub fn hides_online_players(settings: &JsonRpcServerSettings) -> bool {
        settings.hides_online_players
    }

    pub fn set_hides_online_players(
        settings: &mut JsonRpcServerSettings,
        hide: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.hides_online_players = hide;
        settings.hides_online_players
    }

    pub fn replies_to_status(settings: &JsonRpcServerSettings) -> bool {
        settings.replies_to_status
    }

    pub fn set_replies_to_status(
        settings: &mut JsonRpcServerSettings,
        enable: bool,
        _client_info: ClientInfo,
    ) -> bool {
        settings.replies_to_status = enable;
        settings.replies_to_status
    }

    pub fn entity_broadcast_range_percentage(settings: &JsonRpcServerSettings) -> i32 {
        settings.entity_broadcast_range_percentage
    }

    pub fn set_entity_broadcast_range_percentage(
        settings: &mut JsonRpcServerSettings,
        percentage: i32,
        _client_info: ClientInfo,
    ) -> i32 {
        settings.entity_broadcast_range_percentage = percentage;
        settings.entity_broadcast_range_percentage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_runtime_exception_models_match_java_message_constructors() {
        assert_eq!(
            JsonRpcRuntimeException::encode("bad encoding"),
            JsonRpcRuntimeException {
                kind: JsonRpcRuntimeExceptionKind::Encode,
                message: "bad encoding".to_string(),
            }
        );
        assert_eq!(
            JsonRpcRuntimeException::invalid_parameter("bad param").kind,
            JsonRpcRuntimeExceptionKind::InvalidParameter
        );
        assert_eq!(
            JsonRpcRuntimeException::invalid_request("bad request").kind,
            JsonRpcRuntimeExceptionKind::InvalidRequest
        );
        assert_eq!(
            JsonRpcRuntimeException::method_not_found("missing").kind,
            JsonRpcRuntimeExceptionKind::MethodNotFound
        );
    }

    #[test]
    fn remote_rpc_error_exception_preserves_id_and_error_payloads() {
        let error = RemoteRpcErrorException::new("7", "{\"code\":-32603}");
        assert_eq!(error.id(), "7");
        assert_eq!(error.error(), "{\"code\":-32603}");
    }

    #[test]
    fn jsonrpc_method_message_as_component_matches_java_branch_order() {
        assert_eq!(
            JsonRpcMethodMessage::literal("Plain").as_component(),
            Some(Component::literal("Plain"))
        );
        assert_eq!(
            JsonRpcMethodMessage::translatable("chat.type.text", None).as_component(),
            Some(Component::translatable("chat.type.text", Vec::new()))
        );
        assert_eq!(
            JsonRpcMethodMessage::translatable(
                "chat.type.announcement",
                Some(vec!["Server".to_string(), "Restart".to_string()]),
            )
            .as_component(),
            Some(Component::translatable(
                "chat.type.announcement",
                vec![
                    ComponentArgument::String("Server".to_string()),
                    ComponentArgument::String("Restart".to_string()),
                ],
            ))
        );
        assert_eq!(
            JsonRpcMethodMessage::new(
                Some("ignored".to_string()),
                Some("translation.wins".to_string()),
                None,
            )
            .as_component(),
            Some(Component::translatable("translation.wins", Vec::new()))
        );
        assert_eq!(JsonRpcMethodMessage::new(None, None, None).as_component(), None);
    }

    #[test]
    fn client_info_of_matches_java_factory() {
        assert_eq!(ClientInfo::of(42), ClientInfo { connection_id: 42 });
    }

    #[test]
    fn jsonrpc_logger_prefix_and_varargs_match_java() {
        const JSON_RPC_LOGGER_JAVA: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/JsonRpcLogger.java");
        for sentinel in [
            "private static final String PREFIX = \"RPC Connection #{}: \";",
            "public void log(final ClientInfo clientInfo, final String message, final Object... args)",
            "if (args.length == 0)",
            "LOGGER.info(\"RPC Connection #{}: \" + message, clientInfo.connectionId());",
            "List<Object> list = new ArrayList<>(Arrays.asList(args));",
            "list.addFirst(clientInfo.connectionId());",
            "LOGGER.info(\"RPC Connection #{}: \" + message, list.toArray());",
        ] {
            assert!(
                JSON_RPC_LOGGER_JAVA.contains(sentinel),
                "JsonRpcLogger.java missing sentinel: {sentinel}"
            );
        }

        assert_eq!(JsonRpcLogger::PREFIX, "RPC Connection #{}: ");
        assert_eq!(
            JsonRpcLogger::log_message(ClientInfo::of(7), "connected", &[]),
            (
                "RPC Connection #{}: connected".to_string(),
                vec!["7".to_string()]
            )
        );
        assert_eq!(
            JsonRpcLogger::log_message(ClientInfo::of(9), "sent {}", &["payload"]),
            (
                "RPC Connection #{}: sent {}".to_string(),
                vec!["9".to_string(), "payload".to_string()]
            )
        );
    }

    #[test]
    fn discovery_service_matches_java_openrpc_metadata_and_filtering_order() {
        let schema = JsonRpcSchema::record("Test.CODEC")
            .with_field("field", JsonRpcSchema::of_type("string", "Codec.STRING"));
        let schema_registry = vec![SchemaComponent::new("test", schema)];
        let incoming_visible = method("minecraft:incoming/visible", "incoming visible", true);
        let incoming_hidden = method("minecraft:incoming/hidden", "incoming hidden", false);
        let outgoing_visible = method("minecraft:outgoing/visible", "outgoing visible", true);

        let response = DiscoveryService::discover(
            &schema_registry,
            &[incoming_visible.clone(), incoming_hidden],
            std::slice::from_ref(&outgoing_visible),
        );

        assert_eq!(response.json_rpc_protocol_version, "1.3.2");
        assert_eq!(response.discover_info.title, "Minecraft Server JSON-RPC");
        assert_eq!(response.discover_info.version, "2.0.0");
        assert_eq!(
            response
                .methods
                .iter()
                .map(|method| method.contents.description.as_str())
                .collect::<Vec<_>>(),
            vec!["incoming visible", "outgoing visible"]
        );
        assert_eq!(
            response.components.schemas.get("test").map(|schema| schema.codec.as_str()),
            Some("Test.CODEC")
        );
        assert_eq!(
            response
                .components
                .schemas
                .get("test")
                .and_then(|schema| schema.properties.get("field"))
                .map(|field| field.codec.as_str()),
            Some("Codec.STRING")
        );

        assert_eq!(incoming_visible.info.description, "incoming visible");
        assert_eq!(outgoing_visible.info.description, "outgoing visible");
    }

    #[test]
    fn game_rules_service_matches_java_get_update_and_untyped_validation() {
        let mut game_rules = GameRules::new(false);
        let keep_inventory = game_rule("keep_inventory");
        let max_entity_cramming = game_rule("max_entity_cramming");

        let rules = GameRulesService::get(&game_rules);
        assert!(rules.iter().any(|update| {
            update.game_rule.name == "keep_inventory" && update.value == GameRuleValue::Bool(false)
        }));

        assert_eq!(
            GameRulesService::get_typed_rule(keep_inventory, GameRuleValue::Bool(true)),
            JsonRpcGameRuleUpdate::new(keep_inventory, GameRuleValue::Bool(true))
        );

        let update =
            JsonRpcGameRuleUpdate::from_untyped(keep_inventory, GameRuleType::Bool, GameRuleValue::Bool(true));
        assert_eq!(
            update,
            Ok(JsonRpcGameRuleUpdate::new(
                keep_inventory,
                GameRuleValue::Bool(true)
            ))
        );

        let mismatch = JsonRpcGameRuleUpdate::from_untyped(
            keep_inventory,
            GameRuleType::Int,
            GameRuleValue::Bool(true),
        );
        assert_eq!(
            mismatch,
            Err(JsonRpcRuntimeException::invalid_parameter(
                "Stated type \"integer\" mismatches with actual type \"boolean\" of gamerule \"keep_inventory\""
            ))
        );

        let updated = GameRulesService::update(
            &mut game_rules,
            JsonRpcGameRuleUpdate::new(max_entity_cramming, GameRuleValue::Int(7)),
            ClientInfo::of(9),
        );
        assert_eq!(
            updated,
            Ok(JsonRpcGameRuleUpdate::new(
                max_entity_cramming,
                GameRuleValue::Int(7)
            ))
        );
        assert_eq!(game_rules.get("max_entity_cramming"), Some(GameRuleValue::Int(7)));
    }

    #[test]
    fn minecraft_game_rule_service_covers_typed_values_updates_and_availability() {
        let keep_inventory = game_rule("keep_inventory");
        let max_entity_cramming = game_rule("max_entity_cramming");
        let max_minecart_speed = game_rule("max_minecart_speed");
        let mut game_rules = GameRules::new(false);

        assert_eq!(
            game_rules.get_rule_value(keep_inventory),
            Some(GameRuleValue::Bool(false))
        );
        assert_eq!(
            game_rules.get_rule_value(max_entity_cramming),
            Some(GameRuleValue::Int(24))
        );
        assert_eq!(
            game_rules.get_typed_rule(keep_inventory, GameRuleValue::Bool(true)),
            JsonRpcGameRuleUpdate::new(keep_inventory, GameRuleValue::Bool(true))
        );
        assert_eq!(
            game_rules.update_game_rule(
                JsonRpcGameRuleUpdate::new(max_entity_cramming, GameRuleValue::Int(8)),
                ClientInfo::of(41),
            ),
            Ok(JsonRpcGameRuleUpdate::new(
                max_entity_cramming,
                GameRuleValue::Int(8)
            ))
        );
        assert_eq!(
            game_rules.get_rule_value(max_entity_cramming),
            Some(GameRuleValue::Int(8))
        );
        assert!(!game_rules
            .get_available_game_rules()
            .contains(&max_minecart_speed));
        assert!(GameRules::new(true)
            .get_available_game_rules()
            .contains(&max_minecart_speed));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn minecraft_game_rule_service_interface_matches_java_contract() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftGameRuleService.java"
        );
        for sentinel in [
            "import java.util.stream.Stream;",
            "import net.minecraft.server.jsonrpc.methods.ClientInfo;",
            "import net.minecraft.server.jsonrpc.methods.GameRulesService;",
            "import net.minecraft.world.level.gamerules.GameRule;",
            "<T> GameRulesService.GameRuleUpdate<T> updateGameRule(GameRulesService.GameRuleUpdate<T> update, ClientInfo clientInfo);",
            "<T> T getRuleValue(GameRule<T> gameRule);",
            "<T> GameRulesService.GameRuleUpdate<T> getTypedRule(GameRule<T> gameRule, T value);",
            "Stream<GameRule<?>> getAvailableGameRules();",
        ] {
            assert!(
                SOURCE.contains(sentinel),
                "MinecraftGameRuleService.java missing: {sentinel}"
            );
        }
    }

    #[test]
    fn server_settings_service_matches_java_boolean_forwarders() {
        let mut settings = JsonRpcServerSettings::default();
        let client_info = ClientInfo::of(11);

        assert!(ServerSettingsService::autosave(&settings));
        assert!(!ServerSettingsService::set_autosave(&mut settings, false, client_info));
        assert!(ServerSettingsService::set_enforce_allowlist(
            &mut settings,
            true,
            client_info
        ));
        assert!(ServerSettingsService::set_using_allowlist(
            &mut settings,
            true,
            client_info
        ));
        assert!(ServerSettingsService::set_allow_flight(
            &mut settings,
            true,
            client_info
        ));
        assert!(ServerSettingsService::set_force_game_mode(
            &mut settings,
            true,
            client_info
        ));
        assert!(ServerSettingsService::set_accept_transfers(
            &mut settings,
            true,
            client_info
        ));
        assert!(ServerSettingsService::set_hides_online_players(
            &mut settings,
            true,
            client_info
        ));
        assert!(!ServerSettingsService::set_replies_to_status(
            &mut settings,
            false,
            client_info
        ));

        assert!(ServerSettingsService::enforce_allowlist(&settings));
        assert!(ServerSettingsService::using_allowlist(&settings));
        assert!(ServerSettingsService::allow_flight(&settings));
        assert!(ServerSettingsService::force_game_mode(&settings));
        assert!(ServerSettingsService::accept_transfers(&settings));
        assert!(ServerSettingsService::hides_online_players(&settings));
        assert!(!ServerSettingsService::replies_to_status(&settings));
    }

    #[test]
    fn server_settings_service_matches_java_numeric_forwarders() {
        let mut settings = JsonRpcServerSettings::default();
        let client_info = ClientInfo::of(11);

        assert_eq!(
            ServerSettingsService::set_max_players(&mut settings, 77, client_info),
            77
        );
        assert_eq!(
            ServerSettingsService::set_pause_when_empty(&mut settings, 5, client_info),
            5
        );
        assert_eq!(
            ServerSettingsService::set_player_idle_timeout(&mut settings, 12, client_info),
            12
        );
        assert_eq!(
            ServerSettingsService::set_spawn_protection(&mut settings, 0, client_info),
            0
        );
        assert_eq!(
            ServerSettingsService::set_view_distance(&mut settings, 14, client_info),
            14
        );
        assert_eq!(
            ServerSettingsService::set_simulation_distance(&mut settings, 9, client_info),
            9
        );
        assert_eq!(
            ServerSettingsService::set_status_heartbeat_interval(&mut settings, 30, client_info),
            30
        );
        assert_eq!(
            ServerSettingsService::set_entity_broadcast_range_percentage(
                &mut settings,
                150,
                client_info,
            ),
            150
        );

        assert_eq!(ServerSettingsService::max_players(&settings), 77);
        assert_eq!(ServerSettingsService::pause_when_empty(&settings), 5);
        assert_eq!(ServerSettingsService::player_idle_timeout(&settings), 12);
        assert_eq!(ServerSettingsService::spawn_protection(&settings), 0);
        assert_eq!(ServerSettingsService::view_distance(&settings), 14);
        assert_eq!(ServerSettingsService::simulation_distance(&settings), 9);
        assert_eq!(ServerSettingsService::status_heartbeat_interval(&settings), 30);
        assert_eq!(
            ServerSettingsService::entity_broadcast_range_percentage(&settings),
            150
        );
    }

    #[test]
    fn server_settings_service_matches_java_enum_and_text_forwarders() {
        let mut settings = JsonRpcServerSettings::default();
        let client_info = ClientInfo::of(11);

        assert_eq!(
            ServerSettingsService::set_difficulty(
                &mut settings,
                JsonRpcDifficulty::Hard,
                client_info,
            ),
            JsonRpcDifficulty::Hard
        );
        assert_eq!(
            ServerSettingsService::set_game_mode(&mut settings, GameMode::Creative, client_info),
            GameMode::Creative
        );
        assert_eq!(
            ServerSettingsService::set_motd(&mut settings, "Managed", client_info),
            "Managed"
        );

        assert_eq!(
            ServerSettingsService::difficulty(&settings),
            JsonRpcDifficulty::Hard
        );
        assert_eq!(ServerSettingsService::game_mode(&settings), GameMode::Creative);
        assert_eq!(ServerSettingsService::motd(&settings), "Managed");
    }

    #[test]
    fn server_settings_service_matches_java_permission_level_conversion() {
        let mut settings = JsonRpcServerSettings::default();
        let client_info = ClientInfo::of(11);

        assert_eq!(
            ServerSettingsService::set_operator_user_permission_level(
                &mut settings,
                PermissionLevel::Admins,
                client_info,
            ),
            PermissionLevel::Admins
        );
        assert_eq!(
            ServerSettingsService::operator_user_permission_level(&settings),
            PermissionLevel::Admins
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn jsonrpc_exception_message_sources_match_java_26_1_2() {
        const ENCODE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/EncodeJsonRpcException.java");
        const INVALID_PARAMETER: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/InvalidParameterJsonRpcException.java");
        const INVALID_REQUEST: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/InvalidRequestJsonRpcException.java");
        const METHOD_NOT_FOUND: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/MethodNotFoundJsonRpcException.java");
        const REMOTE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/RemoteRpcErrorException.java");
        const MESSAGE: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/Message.java");
        const CLIENT_INFO: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/ClientInfo.java");

        for (name, source, sentinel) in [
            (
                "EncodeJsonRpcException.java",
                ENCODE,
                "public EncodeJsonRpcException(final String message)",
            ),
            (
                "InvalidParameterJsonRpcException.java",
                INVALID_PARAMETER,
                "public InvalidParameterJsonRpcException(final String message)",
            ),
            (
                "InvalidRequestJsonRpcException.java",
                INVALID_REQUEST,
                "public InvalidRequestJsonRpcException(final String message)",
            ),
            (
                "MethodNotFoundJsonRpcException.java",
                METHOD_NOT_FOUND,
                "public MethodNotFoundJsonRpcException(final String message)",
            ),
        ] {
            assert!(source.contains("extends RuntimeException"), "{name} runtime base missing");
            assert!(source.contains(sentinel), "{name} constructor missing");
            assert!(source.contains("super(message);"), "{name} super message missing");
        }

        assert_java_source_contains_all(
            "RemoteRpcErrorException.java",
            REMOTE,
            &[
                "private final JsonElement id;",
                "private final JsonObject error;",
                "public RemoteRpcErrorException(final JsonElement id, final JsonObject error)",
                "return this.error;",
                "return this.id;",
            ],
        );

        assert_java_source_contains_all(
            "Message.java",
            MESSAGE,
            &[
            "public record Message(Optional<String> literal, Optional<String> translatable, Optional<List<String>> translatableParams)",
            "Codec.STRING.optionalFieldOf(\"literal\").forGetter(Message::literal)",
            "Codec.STRING.optionalFieldOf(\"translatable\").forGetter(Message::translatable)",
            "Codec.STRING.listOf().lenientOptionalFieldOf(\"translatableParams\").forGetter(Message::translatableParams)",
            "if (this.translatable.isPresent())",
            "return Optional.of(Component.translatable(translationKey, translationArgs.toArray()));",
            "return Optional.of(Component.translatable(translationKey));",
            "return this.literal.map(Component::literal);",
        ],
        );

        assert_java_source_contains_all(
            "ClientInfo.java",
            CLIENT_INFO,
            &[
                "public record ClientInfo(Integer connectionId)",
                "public static ClientInfo of(final Integer connectionId)",
                "return new ClientInfo(connectionId);",
            ],
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn jsonrpc_service_sources_match_java_26_1_2() {
        const DISCOVERY: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/DiscoveryService.java");
        const GAME_RULES: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/GameRulesService.java");
        const SERVER_SETTINGS: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/ServerSettingsService.java");

        assert_java_source_contains_all(
            "DiscoveryService.java",
            DISCOVERY,
            &[
            "public static DiscoveryService.DiscoverResponse discover(final List<SchemaComponent<?>> schemaRegistry)",
            "new ArrayList<>(BuiltInRegistries.INCOMING_RPC_METHOD.size() + BuiltInRegistries.OUTGOING_RPC_METHOD.size())",
            "if (e.value().attributes().discoverable())",
            "methods.add(e.value().info().named(e.key().identifier()));",
            "schemas.put(component.name(), component.schema().info());",
            "new DiscoveryService.DiscoverInfo(\"Minecraft Server JSON-RPC\", \"2.0.0\")",
            "return new DiscoveryService.DiscoverResponse(\"1.3.2\", discoverInfo, methods, new DiscoveryService.DiscoverComponents(schemas));",
            "public record DiscoverComponents(Map<String, Schema<?>> schemas)",
            "public record DiscoverInfo(String title, String version)",
            "public record DiscoverResponse(",
            "Codec.STRING.fieldOf(\"openrpc\").forGetter(DiscoveryService.DiscoverResponse::jsonRpcProtocolVersion)",
        ],
        );

        assert_java_source_contains_all(
            "GameRulesService.java",
            GAME_RULES,
            &[
            "public static List<GameRulesService.GameRuleUpdate<?>> get(final MinecraftApi minecraftApi)",
            "minecraftApi.gameRuleService().getAvailableGameRules().forEach(gameRule -> addGameRule(minecraftApi, (GameRule<?>)gameRule, rules));",
            "rules.add(getTypedRule(minecraftApi, gameRule, value));",
            "return minecraftApi.gameRuleService().getTypedRule(gameRule, value);",
            "return minecraftApi.gameRuleService().updateGameRule(update, clientInfo);",
            "public record GameRuleUpdate<T>(GameRule<T> gameRule, T value)",
            "BuiltInRegistries.GAME_RULE",
            "dispatch(\"key\", GameRulesService.GameRuleUpdate::gameRule, GameRulesService.GameRuleUpdate::getValueAndTypeCodec)",
            "StringRepresentable.fromEnum(GameRuleType::values).fieldOf(\"type\")",
            "if (gameRule.gameRuleType() != readType)",
            "throw new InvalidParameterJsonRpcException(",
            "Stated type \\\"\" + readType + \"\\\" mismatches with actual type \\\"\" + gameRule.gameRuleType() + \"\\\" of gamerule \\\"\" + gameRule.id() + \"\\\"\"",
            "return new GameRulesService.GameRuleUpdate<>(gameRule, value);",
        ],
        );

        assert_server_settings_source_matches_java(SERVER_SETTINGS);
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    fn assert_server_settings_source_matches_java(source: &str) {
        assert_java_source_contains_all(
            "ServerSettingsService.java",
            source,
            &[
            "public class ServerSettingsService",
            "return minecraftApi.serverSettingsService().isAutoSave();",
            "return minecraftApi.serverSettingsService().setAutoSave(enabled, clientInfo);",
            "return minecraftApi.serverSettingsService().getDifficulty();",
            "return minecraftApi.serverSettingsService().setDifficulty(difficulty, clientInfo);",
            "return minecraftApi.serverSettingsService().isEnforceWhitelist();",
            "return minecraftApi.serverSettingsService().setEnforceWhitelist(enforce, clientInfo);",
            "return minecraftApi.serverSettingsService().isUsingWhitelist();",
            "return minecraftApi.serverSettingsService().setUsingWhitelist(use, clientInfo);",
            "return minecraftApi.serverSettingsService().getMaxPlayers();",
            "return minecraftApi.serverSettingsService().setMaxPlayers(maxPlayers, clientInfo);",
            "return minecraftApi.serverSettingsService().getPauseWhenEmptySeconds();",
            "return minecraftApi.serverSettingsService().setPauseWhenEmptySeconds(emptySeconds, clientInfo);",
            "return minecraftApi.serverSettingsService().getPlayerIdleTimeout();",
            "return minecraftApi.serverSettingsService().setPlayerIdleTimeout(idleTime, clientInfo);",
            "return minecraftApi.serverSettingsService().allowFlight();",
            "return minecraftApi.serverSettingsService().setAllowFlight(allow, clientInfo);",
            "return minecraftApi.serverSettingsService().getSpawnProtectionRadius();",
            "return minecraftApi.serverSettingsService().setSpawnProtectionRadius(spawnProtection, clientInfo);",
            "return minecraftApi.serverSettingsService().getMotd();",
            "return minecraftApi.serverSettingsService().setMotd(motd, clientInfo);",
            "return minecraftApi.serverSettingsService().forceGameMode();",
            "return minecraftApi.serverSettingsService().setForceGameMode(force, clientInfo);",
            "return minecraftApi.serverSettingsService().getGameMode();",
            "return minecraftApi.serverSettingsService().setGameMode(gameMode, clientInfo);",
            "return minecraftApi.serverSettingsService().getViewDistance();",
            "return minecraftApi.serverSettingsService().setViewDistance(viewDistance, clientInfo);",
            "return minecraftApi.serverSettingsService().getSimulationDistance();",
            "return minecraftApi.serverSettingsService().setSimulationDistance(simulationDistance, clientInfo);",
            "return minecraftApi.serverSettingsService().acceptsTransfers();",
            "return minecraftApi.serverSettingsService().setAcceptsTransfers(accept, clientInfo);",
            "return minecraftApi.serverSettingsService().getStatusHeartbeatInterval();",
            "return minecraftApi.serverSettingsService().setStatusHeartbeatInterval(statusHeartbeatInterval, clientInfo);",
            "return minecraftApi.serverSettingsService().getOperatorUserPermissions().level();",
            "return minecraftApi.serverSettingsService().setOperatorUserPermissions(LevelBasedPermissionSet.forLevel(level), clientInfo).level();",
            "return minecraftApi.serverSettingsService().hidesOnlinePlayers();",
            "return minecraftApi.serverSettingsService().setHidesOnlinePlayers(hide, clientInfo);",
            "return minecraftApi.serverSettingsService().repliesToStatus();",
            "return minecraftApi.serverSettingsService().setRepliesToStatus(enable, clientInfo);",
            "return minecraftApi.serverSettingsService().getEntityBroadcastRangePercentage();",
            "return minecraftApi.serverSettingsService().setEntityBroadcastRangePercentage(percentage, clientInfo);",
        ],
        );
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    fn assert_java_source_contains_all(name: &str, source: &str, sentinels: &[&str]) {
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "{name} is missing sentinel: {sentinel}"
            );
        }
    }

    fn method(id: &str, description: &str, discoverable: bool) -> DiscoverableJsonRpcMethod {
        let id = match Identifier::parse(id) {
            Ok(id) => id,
            Err(err) => panic!("test method id should parse: {err}"),
        };
        DiscoverableJsonRpcMethod::new(id, MethodInfo::new(description, None, None), discoverable)
    }

    fn game_rule(name: &str) -> GameRuleDefinition {
        match vanilla_game_rules()
            .iter()
            .find(|definition| definition.name == name)
        {
            Some(definition) => *definition,
            None => panic!("missing test game rule: {name}"),
        }
    }
}
