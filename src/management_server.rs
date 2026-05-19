#![allow(dead_code)]

use crate::management_security::{
    authenticate_management_secret, is_valid_management_secret, tls_startup_decision,
    ManagementSecurityConfig, ManagementSecurityDecision,
};
use crate::network::play::GameMode;
use crate::player_access::NameAndId;
use crate::server_properties::ServerProperties;

pub const DEFAULT_KICK_MESSAGE: &str = "multiplayer.disconnect.kicked";

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
    Any,
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
        } else if entries.iter().any(|entry| entry == "*") {
            Self::Any
        } else {
            Self::Exact(entries)
        }
    }

    pub fn accepts(&self, origin_header: Option<&str>) -> bool {
        match (self, origin_header) {
            (_, None) => true,
            (Self::Any, Some(_)) => true,
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
    fn to_json(&self) -> String {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcRequest {
    pub id: JsonRpcId,
    pub method: String,
    pub params: JsonRpcParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcResult {
    Null,
    Players(Vec<PlayerDto>),
    Discovery(DiscoveryDocument),
    NotificationQueued(String),
}

impl JsonRpcResult {
    fn to_json(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Players(players) => json_array(players.iter().map(PlayerDto::to_json).collect()),
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
                "{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{}}}",
                self.id.to_json(),
                result.to_json()
            ),
            Err(error) => format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{},\"error\":{}}}",
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
}

impl JsonRpcError {
    pub const PARSE_ERROR: Self = Self {
        code: -32700,
        message: "Parse error",
    };
    pub const INVALID_REQUEST: Self = Self {
        code: -32600,
        message: "Invalid Request",
    };
    pub const METHOD_NOT_FOUND: Self = Self {
        code: -32601,
        message: "Method not found",
    };
    pub const INVALID_PARAMS: Self = Self {
        code: -32602,
        message: "Invalid params",
    };
    pub const INTERNAL_ERROR: Self = Self {
        code: -32603,
        message: "Internal error",
    };

    fn to_json(&self) -> String {
        format!(
            "{{\"code\":{},\"message\":{}}}",
            self.code,
            json_string(self.message)
        )
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManagementServerState {
    pub online_players: Vec<NameAndId>,
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
        let result = match request.method.as_str() {
            "rpc/discover" => Ok(JsonRpcResult::Discovery(DiscoveryDocument::vanilla())),
            "players/get" => Ok(JsonRpcResult::Players(self.players())),
            "players/kick" => self
                .kick_players(request.params)
                .map(JsonRpcResult::Players),
            _ => Err(JsonRpcError::METHOD_NOT_FOUND),
        };
        JsonRpcResponse {
            id: request.id,
            result,
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

fn incoming_result_schema(method: &str) -> Option<&'static str> {
    match method {
        "players/get" | "players/kick" => Some("PlayerDto[]"),
        "rpc/discover" => Some("DiscoveryDocument"),
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
        "server/activity" => Some("string"),
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
mod tests {
    use super::*;

    const VALID_SECRET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCD";

    fn config() -> ManagementServerConfig {
        ManagementServerConfig {
            enabled: true,
            host: "localhost".to_string(),
            port: 25585,
            allowed_origins: AllowedOrigins::parse("https://admin.example"),
            security: ManagementSecurityConfig {
                secret_key: VALID_SECRET.to_string(),
                tls_enabled: false,
                tls_keystore: None,
                tls_keystore_password: None,
            },
        }
    }

    fn player(name: &str) -> NameAndId {
        NameAndId::create_offline(name)
    }

    #[test]
    fn startup_refuses_invalid_secret_and_builds_plain_or_tls_listener() {
        let disabled = ManagementServerConfig {
            enabled: false,
            ..config()
        };
        assert_eq!(
            startup_plan(&disabled, None, None),
            ManagementStartupPlan::Disabled
        );

        let invalid_secret = ManagementServerConfig {
            security: ManagementSecurityConfig {
                secret_key: "bad".to_string(),
                ..config().security
            },
            ..config()
        };
        assert_eq!(
            startup_plan(&invalid_secret, None, None),
            ManagementStartupPlan::Refused(ManagementSecurityDecision::DisabledInvalidSecret)
        );

        assert!(matches!(
            startup_plan(&config(), None, None),
            ManagementStartupPlan::Listen { tls: None, .. }
        ));

        let tls = ManagementServerConfig {
            security: ManagementSecurityConfig {
                tls_enabled: true,
                tls_keystore: Some("management.p12".to_string()),
                tls_keystore_password: Some("server-pass".to_string()),
                ..config().security
            },
            ..config()
        };
        assert_eq!(
            startup_plan(&tls, Some("env-pass"), None),
            ManagementStartupPlan::Listen {
                host: "localhost".to_string(),
                port: 25585,
                tls: Some(TlsEndpoint {
                    keystore: "management.p12".to_string(),
                    password: "env-pass".to_string(),
                }),
                allowed_origins: AllowedOrigins::parse("https://admin.example"),
            }
        );
    }

    #[test]
    fn origins_parse_exact_wildcard_and_empty_policy() {
        let exact = AllowedOrigins::parse("https://a.example, https://b.example");
        assert!(exact.accepts(None));
        assert!(exact.accepts(Some("https://b.example")));
        assert!(!exact.accepts(Some("https://c.example")));
        assert!(AllowedOrigins::parse("*").accepts(Some("https://anything.example")));
        assert!(!AllowedOrigins::parse("").accepts(Some("https://a.example")));
    }

    #[test]
    fn authorization_checks_origin_before_constant_time_secret() {
        let cfg = config();
        assert_eq!(
            authorize_request(&cfg, Some("https://evil.example"), Some(VALID_SECRET)),
            ManagementAccessDecision::RejectedOrigin
        );
        assert_eq!(
            authorize_request(&cfg, Some("https://admin.example"), Some("wrong")),
            ManagementAccessDecision::Unauthorized
        );
        assert_eq!(
            authorize_request(&cfg, Some("https://admin.example"), Some(VALID_SECRET)),
            ManagementAccessDecision::Accepted
        );
    }

    #[test]
    fn player_dto_resolves_by_id_before_name_fallback() {
        let steve = player("Steve");
        let alex = player("Alex");
        let dto = PlayerDto {
            id: Some(alex.uuid.clone()),
            name: Some("Steve".to_string()),
            latency: 0,
            game_mode: GameMode::Survival,
        };
        assert_eq!(dto.resolve(&[steve.clone(), alex.clone()]), Some(&alex));

        let by_name = PlayerDto {
            id: None,
            name: Some("steve".to_string()),
            latency: 0,
            game_mode: GameMode::Survival,
        };
        assert_eq!(
            by_name.resolve(&[steve.clone(), alex.clone()]),
            Some(&steve)
        );
    }

    #[test]
    fn player_dto_json_includes_identity_latency_and_game_mode() {
        let steve = player("Steve");
        let dto = PlayerDto::with_state(&steve, 47, GameMode::Creative);
        assert_eq!(
            dto.to_json(),
            format!(
                "{{\"id\":\"{}\",\"name\":\"Steve\",\"latency\":47,\"gameMode\":\"creative\"}}",
                steve.uuid
            )
        );
    }

    #[test]
    fn discovery_exposes_management_methods_notifications_and_schema_refs() {
        let discovery = DiscoveryDocument::vanilla();
        assert!(discovery
            .incoming_methods
            .iter()
            .any(|method| method.method == "players/kick"
                && method.params_schema.as_deref() == Some("KickDto[]")
                && method.result_schema.as_deref() == Some("PlayerDto[]")));
        assert!(discovery
            .outgoing_notifications
            .iter()
            .any(|method| method.method == "server/status"
                && method.params_schema.as_deref() == Some("ServerStatusDto")));
        assert!(discovery
            .incoming_methods
            .iter()
            .any(|method| method.method == "server/metrics"
                && method.result_schema.as_deref() == Some("ServerMetricsDto")));
        assert!(discovery
            .schemas
            .iter()
            .any(|schema| schema.name == "PlayerDto"
                && schema.fields == ["id", "name", "latency", "gameMode"]));
        assert!(discovery
            .schemas
            .iter()
            .any(|schema| schema.name == "ServerMetricsDto"
                && schema.fields.contains(&"tickDurationNanos".to_string())));
    }

    #[test]
    fn jmx_property_uses_management_metrics_export_as_rust_equivalent() {
        let mut properties = ServerProperties::load_or_default(std::path::Path::new(
            "definitely-missing-test-server.properties",
        ))
        .unwrap();
        let management_plan = ManagementStartupPlan::Listen {
            host: "localhost".to_string(),
            port: 25585,
            tls: None,
            allowed_origins: AllowedOrigins::Empty,
        };

        assert_eq!(
            jmx_monitoring_plan(&properties, &management_plan),
            JmxMonitoringPlan::Disabled
        );

        properties.set("enable-jmx-monitoring", "true");
        assert_eq!(
            jmx_monitoring_plan(&properties, &management_plan),
            JmxMonitoringPlan::EquivalentMetricsExport {
                endpoint: "server/metrics",
                transport: "json-rpc-management",
            }
        );
        assert_eq!(
            jmx_monitoring_plan(&properties, &ManagementStartupPlan::Disabled),
            JmxMonitoringPlan::RefusedManagementDisabled
        );
    }

    #[test]
    fn player_get_and_kick_match_vanilla_resolution_and_default_message() {
        let steve = player("Steve");
        let alex = player("Alex");
        let mut state = ManagementServerState {
            online_players: vec![steve.clone(), alex.clone()],
            ..Default::default()
        };
        state.connect_client("admin");

        let get = state.handle_request(JsonRpcRequest {
            id: JsonRpcId::Number(1),
            method: "players/get".to_string(),
            params: JsonRpcParams::None,
        });
        assert_eq!(
            get.result,
            Ok(JsonRpcResult::Players(vec![
                PlayerDto::from_profile(&steve),
                PlayerDto::from_profile(&alex)
            ]))
        );

        let kick = state.handle_request(JsonRpcRequest {
            id: JsonRpcId::String("kick".to_string()),
            method: "players/kick".to_string(),
            params: JsonRpcParams::Kick(vec![
                KickDto {
                    player: PlayerDto {
                        id: Some(steve.uuid.clone()),
                        name: None,
                        latency: 0,
                        game_mode: GameMode::Survival,
                    },
                    message: None,
                },
                KickDto {
                    player: PlayerDto {
                        id: None,
                        name: Some("Alex".to_string()),
                        latency: 0,
                        game_mode: GameMode::Survival,
                    },
                    message: Some("Go away".to_string()),
                },
            ]),
        });

        assert_eq!(
            kick.result,
            Ok(JsonRpcResult::Players(vec![
                PlayerDto::from_profile(&steve),
                PlayerDto::from_profile(&alex)
            ]))
        );
        assert!(state.online_players.is_empty());
        assert_eq!(state.disconnected_players[0].message, DEFAULT_KICK_MESSAGE);
        assert_eq!(state.disconnected_players[1].message, "Go away");
        assert_eq!(
            state.notifications,
            vec![QueuedNotification {
                client_id: "admin".to_string(),
                method: "players/left".to_string(),
            }]
        );
    }

    #[test]
    fn request_tracking_correlates_responses_and_cleans_up_on_disconnect() {
        let steve = player("Steve");
        let mut state = ManagementServerState {
            online_players: vec![steve],
            ..Default::default()
        };
        state.connect_client("admin");

        let get = JsonRpcRequest {
            id: JsonRpcId::String("req-1".to_string()),
            method: "players/get".to_string(),
            params: JsonRpcParams::None,
        };
        let response = state.handle_client_request("admin", get);
        assert_eq!(response.id, JsonRpcId::String("req-1".to_string()));
        assert_eq!(state.pending_request_count(), 0);

        let slow = JsonRpcRequest {
            id: JsonRpcId::Number(2),
            method: "server/metrics".to_string(),
            params: JsonRpcParams::None,
        };
        state.begin_request("admin", &slow);
        assert_eq!(state.pending_request_count(), 1);
        state.disconnect_client("admin");
        assert_eq!(state.pending_request_count(), 0);
    }

    #[test]
    fn json_rpc_errors_and_envelopes_use_standard_codes() {
        let mut state = ManagementServerState::default();
        let unknown = state.handle_request(JsonRpcRequest {
            id: JsonRpcId::Number(99),
            method: "missing/method".to_string(),
            params: JsonRpcParams::None,
        });
        assert_eq!(unknown.result, Err(JsonRpcError::METHOD_NOT_FOUND));
        assert_eq!(
            unknown.to_json(),
            "{\"jsonrpc\":\"2.0\",\"id\":99,\"error\":{\"code\":-32601,\"message\":\"Method not found\"}}"
        );

        let invalid_params = state.handle_request(JsonRpcRequest {
            id: JsonRpcId::Null,
            method: "players/kick".to_string(),
            params: JsonRpcParams::None,
        });
        assert_eq!(invalid_params.result, Err(JsonRpcError::INVALID_PARAMS));
    }

    #[test]
    fn outgoing_notification_methods_cover_vanilla_set_and_broadcast_to_clients() {
        let notifications = vec![
            OutgoingNotification::ServerStarted,
            OutgoingNotification::ServerStopping,
            OutgoingNotification::ServerSaving,
            OutgoingNotification::ServerSaved,
            OutgoingNotification::ServerActivity("tick".to_string()),
            OutgoingNotification::PlayersJoined(vec![]),
            OutgoingNotification::PlayersLeft(vec![]),
            OutgoingNotification::OperatorsAdded(vec![]),
            OutgoingNotification::OperatorsRemoved(vec![]),
            OutgoingNotification::AllowlistAdded(vec![]),
            OutgoingNotification::AllowlistRemoved(vec![]),
            OutgoingNotification::IpBansAdded(vec![]),
            OutgoingNotification::IpBansRemoved(vec![]),
            OutgoingNotification::BansAdded(vec![]),
            OutgoingNotification::BansRemoved(vec![]),
            OutgoingNotification::GameRulesUpdated(vec![]),
            OutgoingNotification::ServerStatus("running".to_string()),
        ];
        assert_eq!(
            notifications
                .iter()
                .map(OutgoingNotification::method)
                .collect::<Vec<_>>(),
            OUTGOING_METHODS
        );

        let mut state = ManagementServerState::default();
        state.connect_client("first");
        state.connect_client("second");
        state.broadcast(OutgoingNotification::ServerStarted);
        assert_eq!(
            state.notifications,
            vec![
                QueuedNotification {
                    client_id: "first".to_string(),
                    method: "server/started".to_string(),
                },
                QueuedNotification {
                    client_id: "second".to_string(),
                    method: "server/started".to_string(),
                },
            ]
        );
    }
}
