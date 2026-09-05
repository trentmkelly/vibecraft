#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::chat_component::{Component, ComponentArgument};
use crate::command::{LevelBasedPermissionSet, PermissionLevel};
use crate::game_rules::{
    GameRuleDefinition, GameRuleError, GameRuleType, GameRuleValue, GameRules,
    vanilla_game_rules,
};
use crate::jsonrpc_api::{JsonRpcSchema, MethodInfo, NamedMethodInfo, SchemaComponent};
use crate::jsonrpc_minecraft_api::DedicatedServerIdentity;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftGameRuleServiceImplModel {
    pub server: DedicatedServerIdentity,
    pub game_rules: GameRules,
    pub log_messages: Vec<(ClientInfo, String)>,
}

impl MinecraftGameRuleServiceImplModel {
    pub const fn new(server: DedicatedServerIdentity, game_rules: GameRules) -> Self {
        Self {
            server,
            game_rules,
            log_messages: Vec::new(),
        }
    }
}

impl MinecraftGameRuleService for MinecraftGameRuleServiceImplModel {
    fn update_game_rule(
        &mut self,
        update: JsonRpcGameRuleUpdate,
        client_info: ClientInfo,
    ) -> Result<JsonRpcGameRuleUpdate, GameRuleError> {
        let old_value = self.game_rules.get(update.game_rule.name);
        let result = self
            .game_rules
            .set(update.game_rule.name, &update.value.sync_value());
        result?;
        let old_value = old_value.ok_or(GameRuleError::DisabledByFeature)?;
        self.log_messages.push((
            client_info,
            format!(
                "Game rule 'minecraft:{}' updated from '{}' to '{}'",
                update.game_rule.name,
                old_value.sync_value(),
                update.value.sync_value()
            ),
        ));
        Ok(update)
    }

    fn get_rule_value(&self, game_rule: GameRuleDefinition) -> Option<GameRuleValue> {
        self.game_rules.get(game_rule.name)
    }

    fn get_typed_rule(
        &self,
        game_rule: GameRuleDefinition,
        value: GameRuleValue,
    ) -> JsonRpcGameRuleUpdate {
        JsonRpcGameRuleUpdate::new(game_rule, value)
    }

    fn get_available_game_rules(&self) -> Vec<GameRuleDefinition> {
        self.game_rules.get_available_game_rules()
    }
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

pub trait MinecraftServerSettingsService {
    fn is_auto_save(&self) -> bool;
    fn set_auto_save(&mut self, enabled: bool, client_info: ClientInfo) -> bool;
    fn get_difficulty(&self) -> JsonRpcDifficulty;
    fn set_difficulty(
        &mut self,
        difficulty: JsonRpcDifficulty,
        client_info: ClientInfo,
    ) -> JsonRpcDifficulty;
    fn is_enforce_whitelist(&self) -> bool;
    fn set_enforce_whitelist(&mut self, enforce: bool, client_info: ClientInfo) -> bool;
    fn is_using_whitelist(&self) -> bool;
    fn set_using_whitelist(&mut self, use_whitelist: bool, client_info: ClientInfo) -> bool;
    fn get_max_players(&self) -> i32;
    fn set_max_players(&mut self, max_players: i32, client_info: ClientInfo) -> i32;
    fn get_pause_when_empty_seconds(&self) -> i32;
    fn set_pause_when_empty_seconds(&mut self, seconds: i32, client_info: ClientInfo) -> i32;
    fn get_player_idle_timeout(&self) -> i32;
    fn set_player_idle_timeout(&mut self, minutes: i32, client_info: ClientInfo) -> i32;
    fn allow_flight(&self) -> bool;
    fn set_allow_flight(&mut self, allow: bool, client_info: ClientInfo) -> bool;
    fn get_spawn_protection_radius(&self) -> i32;
    fn set_spawn_protection_radius(&mut self, radius: i32, client_info: ClientInfo) -> i32;
    fn get_motd(&self) -> String;
    fn set_motd(&mut self, motd: String, client_info: ClientInfo) -> String;
    fn force_game_mode(&self) -> bool;
    fn set_force_game_mode(&mut self, force: bool, client_info: ClientInfo) -> bool;
    fn get_game_mode(&self) -> GameMode;
    fn set_game_mode(&mut self, game_mode: GameMode, client_info: ClientInfo) -> GameMode;
    fn get_view_distance(&self) -> i32;
    fn set_view_distance(&mut self, distance: i32, client_info: ClientInfo) -> i32;
    fn get_simulation_distance(&self) -> i32;
    fn set_simulation_distance(&mut self, distance: i32, client_info: ClientInfo) -> i32;
    fn accepts_transfers(&self) -> bool;
    fn set_accepts_transfers(&mut self, accept: bool, client_info: ClientInfo) -> bool;
    fn get_status_heartbeat_interval(&self) -> i32;
    fn set_status_heartbeat_interval(&mut self, interval: i32, client_info: ClientInfo) -> i32;
    fn get_operator_user_permissions(&self) -> LevelBasedPermissionSet;
    fn set_operator_user_permissions(
        &mut self,
        permissions: LevelBasedPermissionSet,
        client_info: ClientInfo,
    ) -> LevelBasedPermissionSet;
    fn hides_online_players(&self) -> bool;
    fn set_hides_online_players(&mut self, hide: bool, client_info: ClientInfo) -> bool;
    fn replies_to_status(&self) -> bool;
    fn set_replies_to_status(&mut self, enable: bool, client_info: ClientInfo) -> bool;
    fn get_entity_broadcast_range_percentage(&self) -> i32;
    fn set_entity_broadcast_range_percentage(
        &mut self,
        percentage: i32,
        client_info: ClientInfo,
    ) -> i32;
}

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

impl MinecraftServerSettingsService for JsonRpcServerSettings {
    fn is_auto_save(&self) -> bool {
        ServerSettingsService::autosave(self)
    }

    fn set_auto_save(&mut self, enabled: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_autosave(self, enabled, client_info)
    }

    fn get_difficulty(&self) -> JsonRpcDifficulty {
        ServerSettingsService::difficulty(self)
    }

    fn set_difficulty(
        &mut self,
        difficulty: JsonRpcDifficulty,
        client_info: ClientInfo,
    ) -> JsonRpcDifficulty {
        ServerSettingsService::set_difficulty(self, difficulty, client_info)
    }

    fn is_enforce_whitelist(&self) -> bool {
        ServerSettingsService::enforce_allowlist(self)
    }

    fn set_enforce_whitelist(&mut self, enforce: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_enforce_allowlist(self, enforce, client_info)
    }

    fn is_using_whitelist(&self) -> bool {
        ServerSettingsService::using_allowlist(self)
    }

    fn set_using_whitelist(&mut self, use_whitelist: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_using_allowlist(self, use_whitelist, client_info)
    }

    fn get_max_players(&self) -> i32 {
        ServerSettingsService::max_players(self)
    }

    fn set_max_players(&mut self, max_players: i32, client_info: ClientInfo) -> i32 {
        ServerSettingsService::set_max_players(self, max_players, client_info)
    }

    fn get_pause_when_empty_seconds(&self) -> i32 {
        ServerSettingsService::pause_when_empty(self)
    }

    fn set_pause_when_empty_seconds(&mut self, seconds: i32, client_info: ClientInfo) -> i32 {
        ServerSettingsService::set_pause_when_empty(self, seconds, client_info)
    }

    fn get_player_idle_timeout(&self) -> i32 {
        ServerSettingsService::player_idle_timeout(self)
    }

    fn set_player_idle_timeout(&mut self, minutes: i32, client_info: ClientInfo) -> i32 {
        ServerSettingsService::set_player_idle_timeout(self, minutes, client_info)
    }

    fn allow_flight(&self) -> bool {
        ServerSettingsService::allow_flight(self)
    }

    fn set_allow_flight(&mut self, allow: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_allow_flight(self, allow, client_info)
    }

    fn get_spawn_protection_radius(&self) -> i32 {
        ServerSettingsService::spawn_protection(self)
    }

    fn set_spawn_protection_radius(&mut self, radius: i32, client_info: ClientInfo) -> i32 {
        ServerSettingsService::set_spawn_protection(self, radius, client_info)
    }

    fn get_motd(&self) -> String {
        ServerSettingsService::motd(self)
    }

    fn set_motd(&mut self, motd: String, client_info: ClientInfo) -> String {
        ServerSettingsService::set_motd(self, motd, client_info)
    }

    fn force_game_mode(&self) -> bool {
        ServerSettingsService::force_game_mode(self)
    }

    fn set_force_game_mode(&mut self, force: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_force_game_mode(self, force, client_info)
    }

    fn get_game_mode(&self) -> GameMode {
        ServerSettingsService::game_mode(self)
    }

    fn set_game_mode(&mut self, game_mode: GameMode, client_info: ClientInfo) -> GameMode {
        ServerSettingsService::set_game_mode(self, game_mode, client_info)
    }

    fn get_view_distance(&self) -> i32 {
        ServerSettingsService::view_distance(self)
    }

    fn set_view_distance(&mut self, distance: i32, client_info: ClientInfo) -> i32 {
        ServerSettingsService::set_view_distance(self, distance, client_info)
    }

    fn get_simulation_distance(&self) -> i32 {
        ServerSettingsService::simulation_distance(self)
    }

    fn set_simulation_distance(&mut self, distance: i32, client_info: ClientInfo) -> i32 {
        ServerSettingsService::set_simulation_distance(self, distance, client_info)
    }

    fn accepts_transfers(&self) -> bool {
        ServerSettingsService::accept_transfers(self)
    }

    fn set_accepts_transfers(&mut self, accept: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_accept_transfers(self, accept, client_info)
    }

    fn get_status_heartbeat_interval(&self) -> i32 {
        ServerSettingsService::status_heartbeat_interval(self)
    }

    fn set_status_heartbeat_interval(&mut self, interval: i32, client_info: ClientInfo) -> i32 {
        ServerSettingsService::set_status_heartbeat_interval(self, interval, client_info)
    }

    fn get_operator_user_permissions(&self) -> LevelBasedPermissionSet {
        self.operator_user_permissions
    }

    fn set_operator_user_permissions(
        &mut self,
        permissions: LevelBasedPermissionSet,
        _client_info: ClientInfo,
    ) -> LevelBasedPermissionSet {
        self.operator_user_permissions = permissions;
        self.operator_user_permissions
    }

    fn hides_online_players(&self) -> bool {
        ServerSettingsService::hides_online_players(self)
    }

    fn set_hides_online_players(&mut self, hide: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_hides_online_players(self, hide, client_info)
    }

    fn replies_to_status(&self) -> bool {
        ServerSettingsService::replies_to_status(self)
    }

    fn set_replies_to_status(&mut self, enable: bool, client_info: ClientInfo) -> bool {
        ServerSettingsService::set_replies_to_status(self, enable, client_info)
    }

    fn get_entity_broadcast_range_percentage(&self) -> i32 {
        ServerSettingsService::entity_broadcast_range_percentage(self)
    }

    fn set_entity_broadcast_range_percentage(
        &mut self,
        percentage: i32,
        client_info: ClientInfo,
    ) -> i32 {
        ServerSettingsService::set_entity_broadcast_range_percentage(
            self,
            percentage,
            client_info,
        )
    }
}

#[cfg(test)]
mod tests;
