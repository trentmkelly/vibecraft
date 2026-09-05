//! Management request, response, discovery, and player data codecs.
use super::*;

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

    pub(super) fn to_json(&self) -> String {
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
    pub(super) fn to_json(&self) -> String {
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
    pub(super) fn to_json(&self) -> String {
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
    pub(super) fn to_json(&self) -> String {
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
    pub(super) fn to_json(&self) -> String {
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
    pub(super) fn to_json(&self) -> String {
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

    pub(super) fn with_data(mut self, data: Option<&str>) -> Self {
        self.data = data
            .filter(|data| !data.trim().is_empty())
            .map(ToOwned::to_owned);
        self
    }

    pub(super) fn to_json(&self) -> String {
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

    pub(super) fn to_json(&self) -> String {
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
    pub(super) fn incoming(method: &str) -> Self {
        Self {
            method: method.to_string(),
            params_schema: incoming_params_schema(method).map(ToOwned::to_owned),
            result_schema: incoming_result_schema(method).map(ToOwned::to_owned),
        }
    }

    pub(super) fn outgoing(method: &str) -> Self {
        Self {
            method: method.to_string(),
            params_schema: outgoing_params_schema(method).map(ToOwned::to_owned),
            result_schema: None,
        }
    }

    pub(super) fn to_json(&self) -> String {
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
    pub(super) fn object(name: &str, fields: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            fields: fields.iter().map(|field| field.to_string()).collect(),
        }
    }

    pub(super) fn to_json(&self) -> String {
        format!(
            "{{\"name\":{},\"type\":\"object\",\"fields\":{}}}",
            json_string(&self.name),
            json_array(self.fields.iter().map(|field| json_string(field)).collect())
        )
    }
}
