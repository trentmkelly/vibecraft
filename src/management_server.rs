#![allow(dead_code)]

use crate::management_security::{
    authenticate_management_secret, is_valid_management_secret, tls_startup_decision,
    ManagementSecurityConfig, ManagementSecurityDecision,
};
use crate::network::play::GameMode;
use crate::player_access::NameAndId;
use crate::server_properties::ServerProperties;

pub const DEFAULT_KICK_MESSAGE: &str = "multiplayer.disconnect.kicked";
pub const JSON_RPC_VERSION: &str = "2.0";
pub const OPEN_RPC_VERSION: &str = "1.3.2";

pub const INCOMING_METHODS: &[&str] = &[
    "rpc/discover",
    "players/get",
    "players/kick",
    "operators/add",
    "operators/remove",
    "allowlist/add",
    "allowlist/remove",
    "bans/add",
    "bans/remove",
    "ip_bans/add",
    "ip_bans/remove",
    "gamerules/get",
    "gamerules/update",
    "server/settings",
    "server/state",
    "server/metrics",
];

pub const OUTGOING_METHODS: &[&str] = &[
    "server/started",
    "server/stopping",
    "server/saving",
    "server/saved",
    "server/activity",
    "players/joined",
    "players/left",
    "operators/added",
    "operators/removed",
    "allowlist/added",
    "allowlist/removed",
    "ip_bans/added",
    "ip_bans/removed",
    "bans/added",
    "bans/removed",
    "gamerules/updated",
    "server/status",
];

pub const OUTGOING_RPC_METHOD_DEFS: &[OutgoingRpcMethodDef] = &[
    OutgoingRpcMethodDef::notification("server/started", "Server started", None),
    OutgoingRpcMethodDef::notification("server/stopping", "Server shutting down", None),
    OutgoingRpcMethodDef::notification("server/saving", "Server save started", None),
    OutgoingRpcMethodDef::notification("server/saved", "Server save completed", None),
    OutgoingRpcMethodDef::notification(
        "server/activity",
        "Server activity occurred. Rate limited to 1 notification per 30 seconds",
        None,
    ),
    OutgoingRpcMethodDef::notification(
        "players/joined",
        "Player joined",
        Some(("player", "PlayerDto")),
    ),
    OutgoingRpcMethodDef::notification("players/left", "Player left", Some(("player", "PlayerDto"))),
    OutgoingRpcMethodDef::notification(
        "operators/added",
        "Player was oped",
        Some(("player", "OperatorDto")),
    ),
    OutgoingRpcMethodDef::notification(
        "operators/removed",
        "Player was deoped",
        Some(("player", "OperatorDto")),
    ),
    OutgoingRpcMethodDef::notification(
        "allowlist/added",
        "Player was added to allowlist",
        Some(("player", "PlayerDto")),
    ),
    OutgoingRpcMethodDef::notification(
        "allowlist/removed",
        "Player was removed from allowlist",
        Some(("player", "PlayerDto")),
    ),
    OutgoingRpcMethodDef::notification(
        "ip_bans/added",
        "Ip was added to ip ban list",
        Some(("player", "IpBanDto")),
    ),
    OutgoingRpcMethodDef::notification(
        "ip_bans/removed",
        "Ip was removed from ip ban list",
        Some(("player", "string")),
    ),
    OutgoingRpcMethodDef::notification(
        "bans/added",
        "Player was added to ban list",
        Some(("player", "UserBanDto")),
    ),
    OutgoingRpcMethodDef::notification(
        "bans/removed",
        "Player was removed from ban list",
        Some(("player", "PlayerDto")),
    ),
    OutgoingRpcMethodDef::notification(
        "gamerules/updated",
        "Gamerule was changed",
        Some(("gamerule", "GameRuleUpdate")),
    ),
    OutgoingRpcMethodDef::notification(
        "server/status",
        "Server status heartbeat",
        Some(("status", "ServerState")),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutgoingRpcMethodDef {
    pub method: &'static str,
    pub description: &'static str,
    pub param: Option<(&'static str, &'static str)>,
    pub discoverable: bool,
}

impl OutgoingRpcMethodDef {
    pub const NOTIFICATION_PREFIX: &'static str = "notification/";

    pub const fn notification(
        method: &'static str,
        description: &'static str,
        param: Option<(&'static str, &'static str)>,
    ) -> Self {
        Self {
            method,
            description,
            param,
            discoverable: true,
        }
    }

    pub fn registry_key(self) -> String {
        format!("{}{}", Self::NOTIFICATION_PREFIX, self.method)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagementServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub allowed_origins: AllowedOrigins,
    pub security: ManagementSecurityConfig,
}

impl ManagementServerConfig {
    pub fn from_properties(properties: &ServerProperties) -> Self {
        Self {
            enabled: properties.management_server_enabled,
            host: properties.management_server_host.clone(),
            port: properties.management_server_port,
            allowed_origins: AllowedOrigins::parse(&properties.management_server_allowed_origins),
            security: ManagementSecurityConfig {
                secret_key: properties.management_server_secret.clone(),
                tls_enabled: properties.management_server_tls_enabled,
                tls_keystore: empty_to_none(&properties.management_server_tls_keystore),
                tls_keystore_password: empty_to_none(
                    &properties.management_server_tls_keystore_password,
                ),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagementStartupPlan {
    Disabled,
    Refused(ManagementSecurityDecision),
    Listen {
        host: String,
        port: u16,
        tls: Option<TlsEndpoint>,
        allowed_origins: AllowedOrigins,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsEndpoint {
    pub keystore: String,
    pub password: String,
}

pub fn startup_plan(
    config: &ManagementServerConfig,
    env_password: Option<&str>,
    system_property_password: Option<&str>,
) -> ManagementStartupPlan {
    if !config.enabled {
        return ManagementStartupPlan::Disabled;
    }
    if !is_valid_management_secret(&config.security.secret_key) {
        return ManagementStartupPlan::Refused(ManagementSecurityDecision::DisabledInvalidSecret);
    }
    match tls_startup_decision(&config.security, env_password, system_property_password) {
        ManagementSecurityDecision::Accepted => ManagementStartupPlan::Listen {
            host: config.host.clone(),
            port: config.port,
            tls: None,
            allowed_origins: config.allowed_origins.clone(),
        },
        ManagementSecurityDecision::TlsReady { keystore, password } => {
            ManagementStartupPlan::Listen {
                host: config.host.clone(),
                port: config.port,
                tls: Some(TlsEndpoint { keystore, password }),
                allowed_origins: config.allowed_origins.clone(),
            }
        }
        decision => ManagementStartupPlan::Refused(decision),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JmxMonitoringPlan {
    Disabled,
    EquivalentMetricsExport {
        endpoint: &'static str,
        transport: &'static str,
    },
    RefusedManagementDisabled,
}

pub fn jmx_monitoring_plan(
    properties: &ServerProperties,
    management_plan: &ManagementStartupPlan,
) -> JmxMonitoringPlan {
    if !properties.enable_jmx_monitoring {
        return JmxMonitoringPlan::Disabled;
    }
    match management_plan {
        ManagementStartupPlan::Listen { .. } => JmxMonitoringPlan::EquivalentMetricsExport {
            endpoint: "server/metrics",
            transport: "json-rpc-management",
        },
        ManagementStartupPlan::Disabled | ManagementStartupPlan::Refused(_) => {
            JmxMonitoringPlan::RefusedManagementDisabled
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowedOrigins {
    Empty,
    Exact(Vec<String>),
}

impl AllowedOrigins {
    pub fn parse(raw: &str) -> Self {
        let entries: Vec<String> = raw
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        if entries.is_empty() {
            Self::Empty
        } else {
            Self::Exact(entries)
        }
    }

    pub fn accepts(&self, origin_header: Option<&str>) -> bool {
        match (self, origin_header) {
            (_, None) => true,
            (Self::Exact(allowed), Some(origin)) => allowed.iter().any(|entry| entry == origin),
            (Self::Empty, Some(_)) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagementAccessDecision {
    Accepted,
    RejectedOrigin,
    Unauthorized,
    DisabledInvalidSecret,
}

pub fn authorize_request(
    config: &ManagementServerConfig,
    origin_header: Option<&str>,
    bearer_secret: Option<&str>,
) -> ManagementAccessDecision {
    if !config.allowed_origins.accepts(origin_header) {
        return ManagementAccessDecision::RejectedOrigin;
    }
    match authenticate_management_secret(
        &config.security.secret_key,
        bearer_secret.unwrap_or_default(),
    ) {
        ManagementSecurityDecision::Accepted => ManagementAccessDecision::Accepted,
        ManagementSecurityDecision::DisabledInvalidSecret => {
            ManagementAccessDecision::DisabledInvalidSecret
        }
        _ => ManagementAccessDecision::Unauthorized,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDto {
    pub id: Option<String>,
    pub name: Option<String>,
    pub latency: i32,
    pub game_mode: GameMode,
}

impl PlayerDto {
    pub fn from_profile(profile: &NameAndId) -> Self {
        Self {
            id: Some(profile.uuid.clone()),
            name: Some(profile.name.clone()),
            latency: 0,
            game_mode: GameMode::Survival,
        }
    }

    pub fn with_state(profile: &NameAndId, latency: i32, game_mode: GameMode) -> Self {
        Self {
            id: Some(profile.uuid.clone()),
            name: Some(profile.name.clone()),
            latency,
            game_mode,
        }
    }

    pub fn resolve<'a>(&self, players: &'a [NameAndId]) -> Option<&'a NameAndId> {
        if let Some(id) = &self.id {
            if let Some(player) = players.iter().find(|player| &player.uuid == id) {
                return Some(player);
            }
        }
        self.name.as_ref().and_then(|name| {
            players
                .iter()
                .find(|player| player.name.eq_ignore_ascii_case(name))
        })
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"id\":{},\"name\":{},\"latency\":{},\"gameMode\":{}}}",
            json_option(self.id.as_deref()),
            json_option(self.name.as_deref()),
            self.latency,
            json_string(game_mode_name(self.game_mode))
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KickDto {
    pub player: PlayerDto,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpBanDto {
    pub ip: String,
    pub reason: Option<String>,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleDto {
    pub key: String,
    pub value: String,
}

impl GameRuleDto {
    fn to_json(&self) -> String {
        format!(
            "{{\"key\":{},\"value\":{}}}",
            json_string(&self.key),
            json_string(&self.value)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSettingsDto {
    pub online_mode: bool,
    pub max_players: i32,
    pub motd: String,
}

impl ServerSettingsDto {
    fn to_json(&self) -> String {
        format!(
            "{{\"onlineMode\":{},\"maxPlayers\":{},\"motd\":{}}}",
            self.online_mode,
            self.max_players,
            json_string(&self.motd)
        )
    }
}

impl Default for ServerSettingsDto {
    fn default() -> Self {
        Self {
            online_mode: false,
            max_players: 20,
            motd: "A Minecraft Server".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerStatusDto {
    pub running: bool,
    pub player_count: i32,
}

impl ServerStatusDto {
    fn to_json(&self) -> String {
        format!(
            "{{\"running\":{},\"playerCount\":{}}}",
            self.running, self.player_count
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerMetricsDto {
    pub tick: i64,
    pub tick_duration_nanos: i64,
    pub over_budget_nanos: i64,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub packets_in: i64,
    pub packets_out: i64,
}

impl ServerMetricsDto {
    fn to_json(&self) -> String {
        format!(
            "{{\"tick\":{},\"tickDurationNanos\":{},\"overBudgetNanos\":{},\"bytesIn\":{},\"bytesOut\":{},\"packetsIn\":{},\"packetsOut\":{}}}",
            self.tick,
            self.tick_duration_nanos,
            self.over_budget_nanos,
            self.bytes_in,
            self.bytes_out,
            self.packets_in,
            self.packets_out
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDisconnectEvent {
    pub player: NameAndId,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcId {
    Number(i64),
    String(String),
    Null,
}

impl JsonRpcId {
    pub fn to_json(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::String(value) => json_string(value),
            Self::Null => "null".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcParams {
    None,
    Kick(Vec<KickDto>),
    Players(Vec<PlayerDto>),
    IpBans(Vec<IpBanDto>),
    GameRules(Vec<GameRuleDto>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcRequest {
    pub id: JsonRpcId,
    pub method: String,
    pub params: JsonRpcParams,
}

pub struct JsonRpcUtils;

impl JsonRpcUtils {
    pub fn create_success_result(id: JsonRpcId, result_json: &str) -> String {
        format!(
            "{{\"jsonrpc\":{},\"id\":{},\"result\":{}}}",
            json_string(JSON_RPC_VERSION),
            id.to_json(),
            result_json
        )
    }

    pub fn create_request(id: Option<i32>, method: &str, params_json: &[&str]) -> String {
        let mut request = format!(
            "{{\"jsonrpc\":{},",
            json_string(JSON_RPC_VERSION),
        );
        if let Some(id) = id {
            request.push_str("\"id\":");
            request.push_str(&id.to_string());
            request.push(',');
        }
        request.push_str("\"method\":");
        request.push_str(&json_string(method));
        if !params_json.is_empty() {
            request.push_str(",\"params\":[");
            request.push_str(&params_json.join(","));
            request.push(']');
        }
        request.push('}');
        request
    }

    pub fn create_error(
        id: JsonRpcId,
        message: &str,
        error_code: i32,
        data: Option<&str>,
    ) -> String {
        let mut error = format!(
            "{{\"jsonrpc\":{},\"id\":{},\"error\":{{\"code\":{},\"message\":{}",
            json_string(JSON_RPC_VERSION),
            id.to_json(),
            error_code,
            json_string(message)
        );
        if let Some(data) = data.filter(|data| !data.trim().is_empty()) {
            error.push_str(",\"data\":");
            error.push_str(&json_string(data));
        }
        error.push_str("}}");
        error
    }

    pub fn get_request_id(json_object: &serde_json::Value) -> Option<&serde_json::Value> {
        json_object.get("id")
    }

    pub fn get_method_name(json_object: &serde_json::Value) -> Option<&str> {
        json_object.get("method").and_then(serde_json::Value::as_str)
    }

    pub fn get_params(json_object: &serde_json::Value) -> Option<&serde_json::Value> {
        json_object.get("params")
    }

    pub fn get_result(json_object: &serde_json::Value) -> Option<&serde_json::Value> {
        json_object.get("result")
    }

    pub fn get_error(json_object: &serde_json::Value) -> Option<&serde_json::Map<String, serde_json::Value>> {
        json_object.get("error").and_then(serde_json::Value::as_object)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcResult {
    Null,
    Players(Vec<PlayerDto>),
    GameRules(Vec<GameRuleDto>),
    ServerSettings(ServerSettingsDto),
    ServerStatus(ServerStatusDto),
    ServerMetrics(ServerMetricsDto),
    Discovery(DiscoveryDocument),
    NotificationQueued(String),
}

impl JsonRpcResult {
    fn to_json(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Players(players) => json_array(players.iter().map(PlayerDto::to_json).collect()),
            Self::GameRules(rules) => json_array(rules.iter().map(GameRuleDto::to_json).collect()),
            Self::ServerSettings(settings) => settings.to_json(),
            Self::ServerStatus(status) => status.to_json(),
            Self::ServerMetrics(metrics) => metrics.to_json(),
            Self::Discovery(document) => document.to_json(),
            Self::NotificationQueued(method) => {
                format!("{{\"queued\":{}}}", json_string(method))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcResponse {
    pub id: JsonRpcId,
    pub result: Result<JsonRpcResult, JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn to_json(&self) -> String {
        match &self.result {
            Ok(result) => format!(
                "{{\"jsonrpc\":{},\"id\":{},\"result\":{}}}",
                json_string(JSON_RPC_VERSION),
                self.id.to_json(),
                result.to_json()
            ),
            Err(error) => format!(
                "{{\"jsonrpc\":{},\"id\":{},\"error\":{}}}",
                json_string(JSON_RPC_VERSION),
                self.id.to_json(),
                error.to_json()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: &'static str,
    pub data: Option<String>,
}

impl JsonRpcError {
    pub const PARSE_ERROR: Self = Self {
        code: -32700,
        message: "Parse error",
        data: None,
    };
    pub const INVALID_REQUEST: Self = Self {
        code: -32600,
        message: "Invalid Request",
        data: None,
    };
    pub const METHOD_NOT_FOUND: Self = Self {
        code: -32601,
        message: "Method not found",
        data: None,
    };
    pub const INVALID_PARAMS: Self = Self {
        code: -32602,
        message: "Invalid params",
        data: None,
    };
    pub const INTERNAL_ERROR: Self = Self {
        code: -32603,
        message: "Internal error",
        data: None,
    };

    pub fn create_with_unknown_id(self, data: Option<&str>) -> JsonRpcResponse {
        JsonRpcResponse {
            id: JsonRpcId::Null,
            result: Err(self.with_data(data)),
        }
    }

    pub fn create_without_data(self, id: JsonRpcId) -> JsonRpcResponse {
        JsonRpcResponse {
            id,
            result: Err(self),
        }
    }

    pub fn create(self, id: JsonRpcId, data: &str) -> JsonRpcResponse {
        JsonRpcResponse {
            id,
            result: Err(self.with_data(Some(data))),
        }
    }

    fn with_data(mut self, data: Option<&str>) -> Self {
        self.data = data
            .filter(|data| !data.trim().is_empty())
            .map(ToOwned::to_owned);
        self
    }

    fn to_json(&self) -> String {
        let mut out = format!(
            "{{\"code\":{},\"message\":{}}}",
            self.code,
            json_string(self.message)
        );
        if let Some(data) = &self.data {
            out.pop();
            out.push_str(",\"data\":");
            out.push_str(&json_string(data));
            out.push('}');
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryDocument {
    pub incoming_methods: Vec<MethodSchema>,
    pub outgoing_notifications: Vec<MethodSchema>,
    pub schemas: Vec<SchemaDescriptor>,
}

impl DiscoveryDocument {
    pub fn vanilla() -> Self {
        Self {
            incoming_methods: INCOMING_METHODS
                .iter()
                .map(|method| MethodSchema::incoming(method))
                .collect(),
            outgoing_notifications: OUTGOING_METHODS
                .iter()
                .map(|method| MethodSchema::outgoing(method))
                .collect(),
            schemas: vec![
                SchemaDescriptor::object("PlayerDto", &["id", "name", "latency", "gameMode"]),
                SchemaDescriptor::object("KickDto", &["player", "message"]),
                SchemaDescriptor::object("ServerStatusDto", &["running", "playerCount"]),
                SchemaDescriptor::object(
                    "ServerMetricsDto",
                    &[
                        "tick",
                        "tickDurationNanos",
                        "overBudgetNanos",
                        "bytesIn",
                        "bytesOut",
                        "packetsIn",
                        "packetsOut",
                    ],
                ),
                SchemaDescriptor::object("GameRuleDto", &["key", "value"]),
                SchemaDescriptor::object("BanDto", &["player", "reason", "expires"]),
                SchemaDescriptor::object("IpBanDto", &["ip", "reason", "expires"]),
            ],
        }
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"methods\":{},\"notifications\":{},\"schemas\":{}}}",
            json_array(
                self.incoming_methods
                    .iter()
                    .map(MethodSchema::to_json)
                    .collect()
            ),
            json_array(
                self.outgoing_notifications
                    .iter()
                    .map(MethodSchema::to_json)
                    .collect()
            ),
            json_array(self.schemas.iter().map(SchemaDescriptor::to_json).collect())
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSchema {
    pub method: String,
    pub params_schema: Option<String>,
    pub result_schema: Option<String>,
}

impl MethodSchema {
    fn incoming(method: &str) -> Self {
        Self {
            method: method.to_string(),
            params_schema: incoming_params_schema(method).map(ToOwned::to_owned),
            result_schema: incoming_result_schema(method).map(ToOwned::to_owned),
        }
    }

    fn outgoing(method: &str) -> Self {
        Self {
            method: method.to_string(),
            params_schema: outgoing_params_schema(method).map(ToOwned::to_owned),
            result_schema: None,
        }
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"method\":{},\"paramsSchema\":{},\"resultSchema\":{}}}",
            json_string(&self.method),
            json_option(self.params_schema.as_deref()),
            json_option(self.result_schema.as_deref())
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDescriptor {
    pub name: String,
    pub fields: Vec<String>,
}

impl SchemaDescriptor {
    fn object(name: &str, fields: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            fields: fields.iter().map(|field| field.to_string()).collect(),
        }
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"name\":{},\"type\":\"object\",\"fields\":{}}}",
            json_string(&self.name),
            json_array(self.fields.iter().map(|field| json_string(field)).collect())
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutgoingNotification {
    ServerStarted,
    ServerStopping,
    ServerSaving,
    ServerSaved,
    ServerActivity(String),
    PlayersJoined(Vec<PlayerDto>),
    PlayersLeft(Vec<PlayerDto>),
    OperatorsAdded(Vec<PlayerDto>),
    OperatorsRemoved(Vec<PlayerDto>),
    AllowlistAdded(Vec<PlayerDto>),
    AllowlistRemoved(Vec<PlayerDto>),
    IpBansAdded(Vec<String>),
    IpBansRemoved(Vec<String>),
    BansAdded(Vec<PlayerDto>),
    BansRemoved(Vec<PlayerDto>),
    GameRulesUpdated(Vec<String>),
    ServerStatus(String),
}

impl OutgoingNotification {
    pub fn method(&self) -> &'static str {
        match self {
            Self::ServerStarted => "server/started",
            Self::ServerStopping => "server/stopping",
            Self::ServerSaving => "server/saving",
            Self::ServerSaved => "server/saved",
            Self::ServerActivity(_) => "server/activity",
            Self::PlayersJoined(_) => "players/joined",
            Self::PlayersLeft(_) => "players/left",
            Self::OperatorsAdded(_) => "operators/added",
            Self::OperatorsRemoved(_) => "operators/removed",
            Self::AllowlistAdded(_) => "allowlist/added",
            Self::AllowlistRemoved(_) => "allowlist/removed",
            Self::IpBansAdded(_) => "ip_bans/added",
            Self::IpBansRemoved(_) => "ip_bans/removed",
            Self::BansAdded(_) => "bans/added",
            Self::BansRemoved(_) => "bans/removed",
            Self::GameRulesUpdated(_) => "gamerules/updated",
            Self::ServerStatus(_) => "server/status",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedNotification {
    pub client_id: String,
    pub method: String,
}

pub struct JsonRpcNotificationService<'a> {
    management_server: &'a mut ManagementServerState,
}

impl<'a> JsonRpcNotificationService<'a> {
    pub fn new(management_server: &'a mut ManagementServerState) -> Self {
        Self { management_server }
    }

    pub fn player_joined(&mut self, player: PlayerDto) {
        self.broadcast(OutgoingNotification::PlayersJoined(vec![player]));
    }

    pub fn player_left(&mut self, player: PlayerDto) {
        self.broadcast(OutgoingNotification::PlayersLeft(vec![player]));
    }

    pub fn server_started(&mut self) {
        self.broadcast(OutgoingNotification::ServerStarted);
    }

    pub fn server_shutting_down(&mut self) {
        self.broadcast(OutgoingNotification::ServerStopping);
    }

    pub fn server_save_started(&mut self) {
        self.broadcast(OutgoingNotification::ServerSaving);
    }

    pub fn server_save_completed(&mut self) {
        self.broadcast(OutgoingNotification::ServerSaved);
    }

    pub fn server_activity_occured(&mut self) {
        self.broadcast(OutgoingNotification::ServerActivity(String::new()));
    }

    pub fn player_oped(&mut self, operator: PlayerDto) {
        self.broadcast(OutgoingNotification::OperatorsAdded(vec![operator]));
    }

    pub fn player_deoped(&mut self, operator: PlayerDto) {
        self.broadcast(OutgoingNotification::OperatorsRemoved(vec![operator]));
    }

    pub fn player_added_to_allowlist(&mut self, player: PlayerDto) {
        self.broadcast(OutgoingNotification::AllowlistAdded(vec![player]));
    }

    pub fn player_removed_from_allowlist(&mut self, player: PlayerDto) {
        self.broadcast(OutgoingNotification::AllowlistRemoved(vec![player]));
    }

    pub fn ip_banned(&mut self, ban: String) {
        self.broadcast(OutgoingNotification::IpBansAdded(vec![ban]));
    }

    pub fn ip_unbanned(&mut self, ip: String) {
        self.broadcast(OutgoingNotification::IpBansRemoved(vec![ip]));
    }

    pub fn player_banned(&mut self, ban: PlayerDto) {
        self.broadcast(OutgoingNotification::BansAdded(vec![ban]));
    }

    pub fn player_unbanned(&mut self, player: PlayerDto) {
        self.broadcast(OutgoingNotification::BansRemoved(vec![player]));
    }

    pub fn on_game_rule_changed(&mut self, update: String) {
        self.broadcast(OutgoingNotification::GameRulesUpdated(vec![update]));
    }

    pub fn status_heartbeat(&mut self, status: String) {
        self.broadcast(OutgoingNotification::ServerStatus(status));
    }

    fn broadcast(&mut self, notification: OutgoingNotification) {
        self.management_server.broadcast(notification);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManagementServerState {
    pub online_players: Vec<NameAndId>,
    pub operators: Vec<NameAndId>,
    pub allowlist: Vec<NameAndId>,
    pub banned_players: Vec<NameAndId>,
    pub ip_bans: Vec<IpBanDto>,
    pub game_rules: Vec<GameRuleDto>,
    pub settings: ServerSettingsDto,
    pub metrics: ServerMetricsDto,
    connected_clients: Vec<String>,
    pending_requests: Vec<PendingManagementRequest>,
    pub disconnected_players: Vec<PlayerDisconnectEvent>,
    pub notifications: Vec<QueuedNotification>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingManagementRequest {
    pub client_id: String,
    pub id: JsonRpcId,
    pub method: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRpcRequest {
    pub method: String,
    pub timeout_time: u64,
    pub result: Option<Result<String, String>>,
}

impl PendingRpcRequest {
    pub fn new(method: impl Into<String>, timeout_time: u64) -> Self {
        Self {
            method: method.into(),
            timeout_time,
            result: None,
        }
    }

    pub fn accept(
        &mut self,
        response: &serde_json::Value,
        decode_result: impl FnOnce(&serde_json::Value) -> Result<Option<String>, String>,
    ) {
        self.result = Some(match decode_result(response) {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err("decoded result was null".to_string()),
            Err(error) => Err(error),
        });
    }

    pub fn timed_out(&self, current_time: u64) -> bool {
        current_time > self.timeout_time
    }
}

impl ManagementServerState {
    pub fn connect_client(&mut self, client_id: impl Into<String>) {
        let client_id = client_id.into();
        if !self
            .connected_clients
            .iter()
            .any(|client| client == &client_id)
        {
            self.connected_clients.push(client_id);
        }
    }

    pub fn disconnect_client(&mut self, client_id: &str) {
        self.connected_clients.retain(|client| client != client_id);
        self.pending_requests
            .retain(|request| request.client_id != client_id);
    }

    pub fn begin_request(&mut self, client_id: &str, request: &JsonRpcRequest) {
        self.pending_requests.push(PendingManagementRequest {
            client_id: client_id.to_string(),
            id: request.id.clone(),
            method: request.method.clone(),
        });
    }

    pub fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }

    pub fn shutdown(&mut self) -> Vec<JsonRpcResponse> {
        self.broadcast(OutgoingNotification::ServerStopping);
        let rejected = self
            .pending_requests
            .drain(..)
            .map(|request| JsonRpcResponse {
                id: request.id,
                result: Err(JsonRpcError::INTERNAL_ERROR),
            })
            .collect();
        self.connected_clients.clear();
        rejected
    }

    pub fn handle_client_request(
        &mut self,
        client_id: &str,
        request: JsonRpcRequest,
    ) -> JsonRpcResponse {
        self.begin_request(client_id, &request);
        let response = self.handle_request(request);
        self.complete_request(client_id, &response.id);
        response
    }

    fn complete_request(&mut self, client_id: &str, id: &JsonRpcId) {
        if let Some(index) = self
            .pending_requests
            .iter()
            .position(|request| request.client_id == client_id && request.id == *id)
        {
            self.pending_requests.remove(index);
        }
    }

    pub fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = self.dispatch_request(&request).and_then(|result| {
            validate_result_schema(&request.method, &result)?;
            Ok(result)
        });
        JsonRpcResponse {
            id: request.id,
            result,
        }
    }

    fn dispatch_request(
        &mut self,
        request: &JsonRpcRequest,
    ) -> Result<JsonRpcResult, JsonRpcError> {
        validate_params_schema(&request.method, &request.params)?;
        match request.method.as_str() {
            "rpc/discover" => Ok(JsonRpcResult::Discovery(DiscoveryDocument::vanilla())),
            "players/get" => Ok(JsonRpcResult::Players(self.players())),
            "players/kick" => self
                .kick_players(request.params.clone())
                .map(JsonRpcResult::Players),
            "operators/add" => self.update_player_set(
                request.params.clone(),
                PlayerSetKind::Operators,
                PlayerSetOperation::Add,
            ),
            "operators/remove" => self.update_player_set(
                request.params.clone(),
                PlayerSetKind::Operators,
                PlayerSetOperation::Remove,
            ),
            "allowlist/add" => self.update_player_set(
                request.params.clone(),
                PlayerSetKind::Allowlist,
                PlayerSetOperation::Add,
            ),
            "allowlist/remove" => self.update_player_set(
                request.params.clone(),
                PlayerSetKind::Allowlist,
                PlayerSetOperation::Remove,
            ),
            "bans/add" => self.update_player_set(
                request.params.clone(),
                PlayerSetKind::Bans,
                PlayerSetOperation::Add,
            ),
            "bans/remove" => self.update_player_set(
                request.params.clone(),
                PlayerSetKind::Bans,
                PlayerSetOperation::Remove,
            ),
            "ip_bans/add" => self.update_ip_bans(request.params.clone(), IpBanOperation::Add),
            "ip_bans/remove" => self.update_ip_bans(request.params.clone(), IpBanOperation::Remove),
            "gamerules/get" => Ok(JsonRpcResult::GameRules(self.game_rules.clone())),
            "gamerules/update" => self.update_game_rules(request.params.clone()),
            "server/settings" => Ok(JsonRpcResult::ServerSettings(self.settings.clone())),
            "server/state" => Ok(JsonRpcResult::ServerStatus(ServerStatusDto {
                running: true,
                player_count: self.online_players.len() as i32,
            })),
            "server/metrics" => Ok(JsonRpcResult::ServerMetrics(self.metrics.clone())),
            _ => Err(JsonRpcError::METHOD_NOT_FOUND),
        }
    }

    pub fn players(&self) -> Vec<PlayerDto> {
        self.online_players
            .iter()
            .map(PlayerDto::from_profile)
            .collect()
    }

    pub fn broadcast(&mut self, notification: OutgoingNotification) {
        let method = notification.method().to_string();
        for client_id in &self.connected_clients {
            self.notifications.push(QueuedNotification {
                client_id: client_id.clone(),
                method: method.clone(),
            });
        }
    }

    fn kick_players(&mut self, params: JsonRpcParams) -> Result<Vec<PlayerDto>, JsonRpcError> {
        let JsonRpcParams::Kick(kicks) = params else {
            return Err(JsonRpcError::INVALID_PARAMS);
        };
        let mut kicked = Vec::new();
        for kick in kicks {
            let Some(player) = kick.player.resolve(&self.online_players).cloned() else {
                continue;
            };
            self.online_players
                .retain(|online| online.uuid != player.uuid);
            self.disconnected_players.push(PlayerDisconnectEvent {
                player: player.clone(),
                message: kick
                    .message
                    .unwrap_or_else(|| DEFAULT_KICK_MESSAGE.to_string()),
            });
            kicked.push(PlayerDto::from_profile(&player));
        }
        if !kicked.is_empty() {
            self.broadcast(OutgoingNotification::PlayersLeft(kicked.clone()));
        }
        Ok(kicked)
    }

    fn update_player_set(
        &mut self,
        params: JsonRpcParams,
        kind: PlayerSetKind,
        operation: PlayerSetOperation,
    ) -> Result<JsonRpcResult, JsonRpcError> {
        let JsonRpcParams::Players(players) = params else {
            return Err(JsonRpcError::INVALID_PARAMS);
        };
        let mut changed = Vec::new();
        for dto in players {
            let Some(profile) = dto.to_name_and_id() else {
                continue;
            };
            let list = match kind {
                PlayerSetKind::Operators => &mut self.operators,
                PlayerSetKind::Allowlist => &mut self.allowlist,
                PlayerSetKind::Bans => &mut self.banned_players,
            };
            match operation {
                PlayerSetOperation::Add => {
                    if !contains_profile(list, &profile) {
                        list.push(profile.clone());
                        changed.push(PlayerDto::from_profile(&profile));
                    }
                }
                PlayerSetOperation::Remove => {
                    let before = list.len();
                    list.retain(|existing| !profile_matches(existing, &profile));
                    if list.len() != before {
                        changed.push(PlayerDto::from_profile(&profile));
                    }
                }
            }
        }
        if !changed.is_empty() {
            self.broadcast(kind.notification(operation, changed));
        }
        Ok(JsonRpcResult::Null)
    }

    fn update_ip_bans(
        &mut self,
        params: JsonRpcParams,
        operation: IpBanOperation,
    ) -> Result<JsonRpcResult, JsonRpcError> {
        let JsonRpcParams::IpBans(ip_bans) = params else {
            return Err(JsonRpcError::INVALID_PARAMS);
        };
        let mut changed = Vec::new();
        for ban in ip_bans {
            match operation {
                IpBanOperation::Add => {
                    if let Some(existing) = self.ip_bans.iter_mut().find(|entry| entry.ip == ban.ip)
                    {
                        *existing = ban.clone();
                    } else {
                        self.ip_bans.push(ban.clone());
                    }
                    changed.push(ban.ip);
                }
                IpBanOperation::Remove => {
                    let before = self.ip_bans.len();
                    self.ip_bans.retain(|entry| entry.ip != ban.ip);
                    if self.ip_bans.len() != before {
                        changed.push(ban.ip);
                    }
                }
            }
        }
        if !changed.is_empty() {
            self.broadcast(match operation {
                IpBanOperation::Add => OutgoingNotification::IpBansAdded(changed),
                IpBanOperation::Remove => OutgoingNotification::IpBansRemoved(changed),
            });
        }
        Ok(JsonRpcResult::Null)
    }

    fn update_game_rules(&mut self, params: JsonRpcParams) -> Result<JsonRpcResult, JsonRpcError> {
        let JsonRpcParams::GameRules(rules) = params else {
            return Err(JsonRpcError::INVALID_PARAMS);
        };
        let mut changed = Vec::new();
        for rule in rules {
            if let Some(existing) = self
                .game_rules
                .iter_mut()
                .find(|entry| entry.key == rule.key)
            {
                if existing.value != rule.value {
                    existing.value = rule.value.clone();
                    changed.push(rule.key);
                }
            } else {
                changed.push(rule.key.clone());
                self.game_rules.push(rule);
            }
        }
        if !changed.is_empty() {
            self.broadcast(OutgoingNotification::GameRulesUpdated(changed));
        }
        Ok(JsonRpcResult::Null)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerSetKind {
    Operators,
    Allowlist,
    Bans,
}

impl PlayerSetKind {
    fn notification(
        self,
        operation: PlayerSetOperation,
        players: Vec<PlayerDto>,
    ) -> OutgoingNotification {
        match (self, operation) {
            (Self::Operators, PlayerSetOperation::Add) => {
                OutgoingNotification::OperatorsAdded(players)
            }
            (Self::Operators, PlayerSetOperation::Remove) => {
                OutgoingNotification::OperatorsRemoved(players)
            }
            (Self::Allowlist, PlayerSetOperation::Add) => {
                OutgoingNotification::AllowlistAdded(players)
            }
            (Self::Allowlist, PlayerSetOperation::Remove) => {
                OutgoingNotification::AllowlistRemoved(players)
            }
            (Self::Bans, PlayerSetOperation::Add) => OutgoingNotification::BansAdded(players),
            (Self::Bans, PlayerSetOperation::Remove) => OutgoingNotification::BansRemoved(players),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerSetOperation {
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IpBanOperation {
    Add,
    Remove,
}

impl PlayerDto {
    fn to_name_and_id(&self) -> Option<NameAndId> {
        match (&self.name, &self.id) {
            (Some(name), Some(uuid)) => Some(NameAndId {
                name: name.clone(),
                uuid: uuid.clone(),
            }),
            _ => None,
        }
    }
}

fn contains_profile(players: &[NameAndId], profile: &NameAndId) -> bool {
    players
        .iter()
        .any(|existing| profile_matches(existing, profile))
}

fn profile_matches(left: &NameAndId, right: &NameAndId) -> bool {
    left.uuid == right.uuid || left.name.eq_ignore_ascii_case(&right.name)
}

fn empty_to_none(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn incoming_params_schema(method: &str) -> Option<&'static str> {
    match method {
        "players/kick" => Some("KickDto[]"),
        "operators/add" | "operators/remove" | "allowlist/add" | "allowlist/remove"
        | "bans/add" | "bans/remove" => Some("PlayerDto[]"),
        "ip_bans/add" | "ip_bans/remove" => Some("IpBanDto[]"),
        "gamerules/update" => Some("GameRuleDto[]"),
        _ => None,
    }
}

fn validate_params_schema(method: &str, params: &JsonRpcParams) -> Result<(), JsonRpcError> {
    let Some(schema) = incoming_params_schema(method) else {
        return if matches!(params, JsonRpcParams::None) {
            Ok(())
        } else {
            Err(JsonRpcError::INVALID_PARAMS)
        };
    };
    match (schema, params) {
        ("KickDto[]", JsonRpcParams::Kick(_)) => Ok(()),
        ("PlayerDto[]", JsonRpcParams::Players(_)) => Ok(()),
        ("IpBanDto[]", JsonRpcParams::IpBans(_)) => Ok(()),
        ("GameRuleDto[]", JsonRpcParams::GameRules(_)) => Ok(()),
        _ => Err(JsonRpcError::INVALID_PARAMS),
    }
}

fn validate_result_schema(method: &str, result: &JsonRpcResult) -> Result<(), JsonRpcError> {
    let Some(schema) = incoming_result_schema(method) else {
        return Err(JsonRpcError::METHOD_NOT_FOUND);
    };
    match (schema, result) {
        ("DiscoveryDocument", JsonRpcResult::Discovery(_)) => Ok(()),
        ("PlayerDto[]", JsonRpcResult::Players(_)) => Ok(()),
        ("GameRuleDto[]", JsonRpcResult::GameRules(_)) => Ok(()),
        ("void", JsonRpcResult::Null) => Ok(()),
        ("ServerSettingsDto", JsonRpcResult::ServerSettings(_)) => Ok(()),
        ("ServerStatusDto", JsonRpcResult::ServerStatus(_)) => Ok(()),
        ("ServerMetricsDto", JsonRpcResult::ServerMetrics(_)) => Ok(()),
        _ => Err(JsonRpcError::INTERNAL_ERROR),
    }
}

fn incoming_result_schema(method: &str) -> Option<&'static str> {
    match method {
        "players/get" | "players/kick" => Some("PlayerDto[]"),
        "rpc/discover" => Some("DiscoveryDocument"),
        "gamerules/get" => Some("GameRuleDto[]"),
        "server/settings" => Some("ServerSettingsDto"),
        "server/state" => Some("ServerStatusDto"),
        "server/metrics" => Some("ServerMetricsDto"),
        _ => Some("void"),
    }
}

fn outgoing_params_schema(method: &str) -> Option<&'static str> {
    match method {
        "players/joined" | "players/left" | "operators/added" | "operators/removed"
        | "allowlist/added" | "allowlist/removed" | "bans/added" | "bans/removed" => {
            Some("PlayerDto[]")
        }
        "ip_bans/added" | "ip_bans/removed" => Some("IpBanDto[]"),
        "gamerules/updated" => Some("GameRuleDto[]"),
        "server/status" => Some("ServerStatusDto"),
        _ => None,
    }
}

fn game_mode_name(game_mode: GameMode) -> &'static str {
    match game_mode {
        GameMode::Survival => "survival",
        GameMode::Creative => "creative",
        GameMode::Adventure => "adventure",
        GameMode::Spectator => "spectator",
    }
}

fn json_array(items: Vec<String>) -> String {
    format!("[{}]", items.join(","))
}

fn json_option(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".to_string())
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests;
