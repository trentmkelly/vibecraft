#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::chat_component::{Component, ComponentArgument};
use crate::game_rules::{
    GameRuleDefinition, GameRuleError, GameRuleType, GameRuleValue, GameRules,
    vanilla_game_rules,
};
use crate::jsonrpc_api::{JsonRpcSchema, MethodInfo, NamedMethodInfo, SchemaComponent};
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

fn game_rule_type_name(rule_type: GameRuleType) -> &'static str {
    match rule_type {
        GameRuleType::Int => "integer",
        GameRuleType::Bool => "boolean",
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
    #[cfg(vibecraft_has_decompiled_sources)]
    fn jsonrpc_method_sources_match_java_26_1_2() {
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
        const DISCOVERY: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/DiscoveryService.java");
        const GAME_RULES: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/GameRulesService.java");

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

        for sentinel in [
            "private final JsonElement id;",
            "private final JsonObject error;",
            "public RemoteRpcErrorException(final JsonElement id, final JsonObject error)",
            "return this.error;",
            "return this.id;",
        ] {
            assert!(
                REMOTE.contains(sentinel),
                "RemoteRpcErrorException.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record Message(Optional<String> literal, Optional<String> translatable, Optional<List<String>> translatableParams)",
            "Codec.STRING.optionalFieldOf(\"literal\").forGetter(Message::literal)",
            "Codec.STRING.optionalFieldOf(\"translatable\").forGetter(Message::translatable)",
            "Codec.STRING.listOf().lenientOptionalFieldOf(\"translatableParams\").forGetter(Message::translatableParams)",
            "if (this.translatable.isPresent())",
            "return Optional.of(Component.translatable(translationKey, translationArgs.toArray()));",
            "return Optional.of(Component.translatable(translationKey));",
            "return this.literal.map(Component::literal);",
        ] {
            assert!(
                MESSAGE.contains(sentinel),
                "Message.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record ClientInfo(Integer connectionId)",
            "public static ClientInfo of(final Integer connectionId)",
            "return new ClientInfo(connectionId);",
        ] {
            assert!(
                CLIENT_INFO.contains(sentinel),
                "ClientInfo.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
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
        ] {
            assert!(
                DISCOVERY.contains(sentinel),
                "DiscoveryService.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
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
        ] {
            assert!(
                GAME_RULES.contains(sentinel),
                "GameRulesService.java is missing sentinel: {sentinel}"
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
