//! Incoming and outgoing RPC method declarations and builders.

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

pub const INCOMING_RPC_METHOD_DEFS: &[IncomingRpcMethodDef] = &[
    IncomingRpcMethodDef::parameterless(
        "allowlist",
        "Get the allowlist",
        "allowlist",
        "Schema.PLAYER_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "allowlist/set",
        "Set the allowlist",
        "players",
        "Schema.PLAYER_SCHEMA.asArray()",
        "allowlist",
        "Schema.PLAYER_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "allowlist/add",
        "Add players to allowlist",
        "add",
        "Schema.PLAYER_SCHEMA.asArray()",
        "allowlist",
        "Schema.PLAYER_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "allowlist/remove",
        "Remove players from allowlist",
        "remove",
        "Schema.PLAYER_SCHEMA.asArray()",
        "allowlist",
        "Schema.PLAYER_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "allowlist/clear",
        "Clear all players in allowlist",
        "allowlist",
        "Schema.PLAYER_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "bans",
        "Get the ban list",
        "banlist",
        "Schema.PLAYER_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "bans/set",
        "Set the banlist",
        "bans",
        "Schema.PLAYER_BAN_SCHEMA.asArray()",
        "banlist",
        "Schema.PLAYER_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "bans/add",
        "Add players to ban list",
        "add",
        "Schema.PLAYER_BAN_SCHEMA.asArray()",
        "banlist",
        "Schema.PLAYER_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "bans/remove",
        "Remove players from ban list",
        "remove",
        "Schema.PLAYER_SCHEMA.asArray()",
        "banlist",
        "Schema.PLAYER_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "bans/clear",
        "Clear all players in ban list",
        "banlist",
        "Schema.PLAYER_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "ip_bans",
        "Get the ip ban list",
        "banlist",
        "Schema.IP_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "ip_bans/set",
        "Set the ip banlist",
        "banlist",
        "Schema.IP_BAN_SCHEMA.asArray()",
        "banlist",
        "Schema.IP_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "ip_bans/add",
        "Add ip to ban list",
        "add",
        "Schema.INCOMING_IP_BAN_SCHEMA.asArray()",
        "banlist",
        "Schema.IP_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "ip_bans/remove",
        "Remove ip from ban list",
        "ip",
        "Schema.STRING_SCHEMA.asArray()",
        "banlist",
        "Schema.IP_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "ip_bans/clear",
        "Clear all ips in ban list",
        "banlist",
        "Schema.IP_BAN_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "players",
        "Get all connected players",
        "players",
        "Schema.PLAYER_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "players/kick",
        "Kick players",
        "kick",
        "Schema.KICK_PLAYER_SCHEMA.asArray()",
        "kicked",
        "Schema.PLAYER_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "operators",
        "Get all oped players",
        "operators",
        "Schema.OPERATOR_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "operators/set",
        "Set all oped players",
        "operators",
        "Schema.OPERATOR_SCHEMA.asArray()",
        "operators",
        "Schema.OPERATOR_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "operators/add",
        "Op players",
        "add",
        "Schema.OPERATOR_SCHEMA.asArray()",
        "operators",
        "Schema.OPERATOR_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "operators/remove",
        "Deop players",
        "remove",
        "Schema.PLAYER_SCHEMA.asArray()",
        "operators",
        "Schema.OPERATOR_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "operators/clear",
        "Deop all players",
        "operators",
        "Schema.OPERATOR_SCHEMA.asArray()",
    ),
    IncomingRpcMethodDef::parameterless(
        "server/status",
        "Get server status",
        "status",
        "Schema.SERVER_STATE_SCHEMA.asRef()",
    ),
    IncomingRpcMethodDef::with_param(
        "server/save",
        "Save server state",
        "flush",
        "Schema.BOOL_SCHEMA",
        "saving",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "server/stop",
        "Stop server",
        "stopping",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "server/system_message",
        "Send a system message",
        "message",
        "Schema.SYSTEM_MESSAGE_SCHEMA.asRef()",
        "sent",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/autosave",
        "Get whether automatic world saving is enabled on the server",
        "enabled",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/autosave/set",
        "Enable or disable automatic world saving on the server",
        "enable",
        "Schema.BOOL_SCHEMA",
        "enabled",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/difficulty",
        "Get the current difficulty level of the server",
        "difficulty",
        "Schema.DIFFICULTY_SCHEMA.asRef()",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/difficulty/set",
        "Set the difficulty level of the server",
        "difficulty",
        "Schema.DIFFICULTY_SCHEMA.asRef()",
        "difficulty",
        "Schema.DIFFICULTY_SCHEMA.asRef()",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/enforce_allowlist",
        "Get whether allowlist enforcement is enabled (kicks players immediately when removed from allowlist)",
        "enforced",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/enforce_allowlist/set",
        "Enable or disable allowlist enforcement (when enabled, players are kicked immediately upon removal from allowlist)",
        "enforce",
        "Schema.BOOL_SCHEMA",
        "enforced",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/use_allowlist",
        "Get whether the allowlist is enabled on the server",
        "used",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/use_allowlist/set",
        "Enable or disable the allowlist on the server (controls whether only allowlisted players can join)",
        "use",
        "Schema.BOOL_SCHEMA",
        "used",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/max_players",
        "Get the maximum number of players allowed to connect to the server",
        "max",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/max_players/set",
        "Set the maximum number of players allowed to connect to the server",
        "max",
        "Schema.INT_SCHEMA",
        "max",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/pause_when_empty_seconds",
        "Get the number of seconds before the game is automatically paused when no players are online",
        "seconds",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/pause_when_empty_seconds/set",
        "Set the number of seconds before the game is automatically paused when no players are online",
        "seconds",
        "Schema.INT_SCHEMA",
        "seconds",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/player_idle_timeout",
        "Get the number of seconds before idle players are automatically kicked from the server",
        "seconds",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/player_idle_timeout/set",
        "Set the number of seconds before idle players are automatically kicked from the server",
        "seconds",
        "Schema.INT_SCHEMA",
        "seconds",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/allow_flight",
        "Get whether flight is allowed for players in Survival mode",
        "allowed",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/allow_flight/set",
        "Allow or disallow flight for players in Survival mode",
        "allow",
        "Schema.BOOL_SCHEMA",
        "allowed",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/motd",
        "Get the server's message of the day displayed to players",
        "message",
        "Schema.STRING_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/motd/set",
        "Set the server's message of the day displayed to players",
        "message",
        "Schema.STRING_SCHEMA",
        "message",
        "Schema.STRING_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/spawn_protection_radius",
        "Get the spawn protection radius in blocks (only operators can edit within this area)",
        "radius",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/spawn_protection_radius/set",
        "Set the spawn protection radius in blocks (only operators can edit within this area)",
        "radius",
        "Schema.INT_SCHEMA",
        "radius",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/force_game_mode",
        "Get whether players are forced to use the server's default game mode",
        "forced",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/force_game_mode/set",
        "Enable or disable forcing players to use the server's default game mode",
        "force",
        "Schema.BOOL_SCHEMA",
        "forced",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/game_mode",
        "Get the server's default game mode",
        "mode",
        "Schema.GAME_TYPE_SCHEMA.asRef()",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/game_mode/set",
        "Set the server's default game mode",
        "mode",
        "Schema.GAME_TYPE_SCHEMA.asRef()",
        "mode",
        "Schema.GAME_TYPE_SCHEMA.asRef()",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/view_distance",
        "Get the server's view distance in chunks",
        "distance",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/view_distance/set",
        "Set the server's view distance in chunks",
        "distance",
        "Schema.INT_SCHEMA",
        "distance",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/simulation_distance",
        "Get the server's simulation distance in chunks",
        "distance",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/simulation_distance/set",
        "Set the server's simulation distance in chunks",
        "distance",
        "Schema.INT_SCHEMA",
        "distance",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/accept_transfers",
        "Get whether the server accepts player transfers from other servers",
        "accepted",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/accept_transfers/set",
        "Enable or disable accepting player transfers from other servers",
        "accept",
        "Schema.BOOL_SCHEMA",
        "accepted",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/status_heartbeat_interval",
        "Get the interval in seconds between server status heartbeats",
        "seconds",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/status_heartbeat_interval/set",
        "Set the interval in seconds between server status heartbeats",
        "seconds",
        "Schema.INT_SCHEMA",
        "seconds",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/operator_user_permission_level",
        "Get default operator permission level",
        "level",
        "Schema.PERMISSION_LEVEL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/operator_user_permission_level/set",
        "Set default operator permission level",
        "level",
        "Schema.PERMISSION_LEVEL_SCHEMA",
        "level",
        "Schema.PERMISSION_LEVEL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/hide_online_players",
        "Get whether the server hides online player information from status queries",
        "hidden",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/hide_online_players/set",
        "Enable or disable hiding online player information from status queries",
        "hide",
        "Schema.BOOL_SCHEMA",
        "hidden",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/status_replies",
        "Get whether the server responds to connection status requests",
        "enabled",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/status_replies/set",
        "Enable or disable the server responding to connection status requests",
        "enable",
        "Schema.BOOL_SCHEMA",
        "enabled",
        "Schema.BOOL_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "serversettings/entity_broadcast_range",
        "Get the entity broadcast range as a percentage",
        "percentage_points",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::with_param(
        "serversettings/entity_broadcast_range/set",
        "Set the entity broadcast range as a percentage",
        "percentage_points",
        "Schema.INT_SCHEMA",
        "percentage_points",
        "Schema.INT_SCHEMA",
    ),
    IncomingRpcMethodDef::parameterless(
        "gamerules",
        "Get the available game rule keys and their current values",
        "gamerules",
        "Schema.TYPED_GAME_RULE_SCHEMA.asRef().asArray()",
    ),
    IncomingRpcMethodDef::with_param(
        "gamerules/update",
        "Update game rule value",
        "gamerule",
        "Schema.UNTYPED_GAME_RULE_SCHEMA.asRef()",
        "gamerule",
        "Schema.TYPED_GAME_RULE_SCHEMA.asRef()",
    ),
    IncomingRpcMethodDef {
        method: "rpc.discover",
        description: "",
        param: None,
        result: ("result", "Schema.DISCOVERY_SCHEMA"),
        run_on_main_thread: false,
        discoverable: false,
    },
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
pub struct IncomingRpcMethodDef {
    pub method: &'static str,
    pub description: &'static str,
    pub param: Option<(&'static str, &'static str)>,
    pub result: (&'static str, &'static str),
    pub run_on_main_thread: bool,
    pub discoverable: bool,
}

impl IncomingRpcMethodDef {
    pub const fn parameterless(
        method: &'static str,
        description: &'static str,
        result_name: &'static str,
        result_schema: &'static str,
    ) -> Self {
        Self {
            method,
            description,
            param: None,
            result: (result_name, result_schema),
            run_on_main_thread: true,
            discoverable: true,
        }
    }

    pub const fn with_param(
        method: &'static str,
        description: &'static str,
        param_name: &'static str,
        param_schema: &'static str,
        result_name: &'static str,
        result_schema: &'static str,
    ) -> Self {
        Self {
            method,
            description,
            param: Some((param_name, param_schema)),
            result: (result_name, result_schema),
            run_on_main_thread: true,
            discoverable: true,
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutgoingRpcMethodKind {
    Method,
    Notification,
    ParameterlessMethod,
    ParameterlessNotification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingRpcMethodModel {
    pub description: String,
    pub param: Option<(String, String)>,
    pub result: Option<(String, String)>,
    pub discoverable: bool,
    pub kind: OutgoingRpcMethodKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingRpcMethodBuilderModel {
    description: String,
    param: Option<(String, String)>,
    result: Option<(String, String)>,
    kind: OutgoingRpcMethodKind,
}

impl OutgoingRpcMethodModel {
    pub fn default_encode_params(&self) -> Option<serde_json::Value> {
        None
    }

    pub fn default_decode_result(&self) -> Option<serde_json::Value> {
        None
    }

    pub fn encode_params(
        &self,
        params: serde_json::Value,
    ) -> Result<Option<serde_json::Value>, String> {
        match self.kind {
            OutgoingRpcMethodKind::Method | OutgoingRpcMethodKind::Notification => {
                if self.param.is_none() {
                    Err("Method defined as having no parameters".to_string())
                } else {
                    Ok(Some(params))
                }
            }
            OutgoingRpcMethodKind::ParameterlessMethod
            | OutgoingRpcMethodKind::ParameterlessNotification => Ok(None),
        }
    }

    pub fn decode_result(
        &self,
        result: serde_json::Value,
    ) -> Result<Option<serde_json::Value>, String> {
        match self.kind {
            OutgoingRpcMethodKind::Method | OutgoingRpcMethodKind::ParameterlessMethod => {
                if self.result.is_none() {
                    Err("Method defined as having no result".to_string())
                } else {
                    Ok(Some(result))
                }
            }
            OutgoingRpcMethodKind::Notification
            | OutgoingRpcMethodKind::ParameterlessNotification => Ok(None),
        }
    }
}

impl OutgoingRpcMethodBuilderModel {
    pub const DEFAULT_DISCOVERABLE: bool = true;

    pub fn notification() -> Self {
        Self::new(OutgoingRpcMethodKind::ParameterlessNotification)
    }

    pub fn notification_with_params() -> Self {
        Self::new(OutgoingRpcMethodKind::Notification)
    }

    pub fn request() -> Self {
        Self::new(OutgoingRpcMethodKind::ParameterlessMethod)
    }

    pub fn request_with_params() -> Self {
        Self::new(OutgoingRpcMethodKind::Method)
    }

    fn new(kind: OutgoingRpcMethodKind) -> Self {
        Self {
            description: String::new(),
            param: None,
            result: None,
            kind,
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn response(mut self, result_name: &str, result_schema: &str) -> Self {
        self.result = Some((result_name.to_string(), result_schema.to_string()));
        self
    }

    pub fn param(mut self, param_name: &str, param_schema: &str) -> Self {
        self.param = Some((param_name.to_string(), param_schema.to_string()));
        self
    }

    pub fn build(self) -> OutgoingRpcMethodModel {
        OutgoingRpcMethodModel {
            description: self.description,
            param: self.param,
            result: self.result,
            discoverable: Self::DEFAULT_DISCOVERABLE,
            kind: self.kind,
        }
    }

    pub fn register_key(key: &str) -> String {
        format!("{}{}", OutgoingRpcMethodDef::NOTIFICATION_PREFIX, key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingRpcMethodKind {
    Method,
    ParameterlessMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingRpcMethodBuilderKind {
    ParameterlessFunction,
    ParameterFunction,
    ApiSupplier,
    MissingFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingRpcMethodAttributes {
    pub run_on_main_thread: bool,
    pub discoverable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingRpcMethodModel {
    pub description: String,
    pub param: Option<(String, String)>,
    pub result: Option<(String, String)>,
    pub attributes: IncomingRpcMethodAttributes,
    pub kind: IncomingRpcMethodKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingRpcMethodBuilderModel {
    description: String,
    param: Option<(String, String)>,
    result: Option<(String, String)>,
    discoverable: bool,
    run_on_main_thread: bool,
    kind: IncomingRpcMethodBuilderKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomingRpcMethodErrorKind {
    IllegalArgument,
    IllegalState,
    InvalidParameter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingRpcMethodError {
    pub kind: IncomingRpcMethodErrorKind,
    pub message: String,
}

impl IncomingRpcMethodBuilderModel {
    pub const DEFAULT_DISCOVERABLE: bool = true;
    pub const DEFAULT_RUN_ON_MAIN_THREAD: bool = true;

    pub fn method_parameterless() -> Self {
        Self::new(IncomingRpcMethodBuilderKind::ParameterlessFunction)
    }

    pub fn method_with_params() -> Self {
        Self::new(IncomingRpcMethodBuilderKind::ParameterFunction)
    }

    pub fn method_api_supplier() -> Self {
        Self::new(IncomingRpcMethodBuilderKind::ApiSupplier)
    }

    pub fn missing_function_for_test() -> Self {
        Self::new(IncomingRpcMethodBuilderKind::MissingFunction)
    }

    fn new(kind: IncomingRpcMethodBuilderKind) -> Self {
        Self {
            description: String::new(),
            param: None,
            result: None,
            discoverable: Self::DEFAULT_DISCOVERABLE,
            run_on_main_thread: Self::DEFAULT_RUN_ON_MAIN_THREAD,
            kind,
        }
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn response(mut self, result_name: &str, result_schema: &str) -> Self {
        self.result = Some((result_name.to_string(), result_schema.to_string()));
        self
    }

    pub fn param(mut self, param_name: &str, param_schema: &str) -> Self {
        self.param = Some((param_name.to_string(), param_schema.to_string()));
        self
    }

    pub fn undiscoverable(mut self) -> Self {
        self.discoverable = false;
        self
    }

    pub fn not_on_main_thread(mut self) -> Self {
        self.run_on_main_thread = false;
        self
    }

    pub fn build(self) -> Result<IncomingRpcMethodModel, IncomingRpcMethodError> {
        if self.result.is_none() {
            return Err(IncomingRpcMethodError::illegal_state("No response defined"));
        }

        let attributes = IncomingRpcMethodAttributes {
            run_on_main_thread: self.run_on_main_thread,
            discoverable: self.discoverable,
        };
        let kind = match self.kind {
            IncomingRpcMethodBuilderKind::ParameterlessFunction
            | IncomingRpcMethodBuilderKind::ApiSupplier => IncomingRpcMethodKind::ParameterlessMethod,
            IncomingRpcMethodBuilderKind::ParameterFunction => {
                if self.param.is_none() {
                    return Err(IncomingRpcMethodError::illegal_state(
                        "No param schema defined",
                    ));
                }
                IncomingRpcMethodKind::Method
            }
            IncomingRpcMethodBuilderKind::MissingFunction => {
                return Err(IncomingRpcMethodError::illegal_state("No method defined"));
            }
        };

        Ok(IncomingRpcMethodModel {
            description: self.description,
            param: self.param,
            result: self.result,
            attributes,
            kind,
        })
    }

    pub fn register_key(key: &str) -> String {
        key.to_string()
    }
}

impl IncomingRpcMethodModel {
    pub fn apply(
        &self,
        params_json: Option<&serde_json::Value>,
        result: serde_json::Value,
    ) -> Result<serde_json::Value, IncomingRpcMethodError> {
        match self.kind {
            IncomingRpcMethodKind::Method => {
                self.extract_param(params_json)?;
                self.encode_result(result)
            }
            IncomingRpcMethodKind::ParameterlessMethod => {
                self.validate_parameterless_params(params_json)?;
                self.encode_result(result)
            }
        }
    }

    pub fn extract_param(
        &self,
        params_json: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, IncomingRpcMethodError> {
        let Some(params_json) = params_json else {
            return Err(IncomingRpcMethodError::invalid_parameter(
                "Expected params as array or named",
            ));
        };
        if !(params_json.is_array() || params_json.is_object()) {
            return Err(IncomingRpcMethodError::invalid_parameter(
                "Expected params as array or named",
            ));
        }
        let Some((parameter_name, _)) = &self.param else {
            return Err(IncomingRpcMethodError::illegal_argument(
                "Method defined as having parameters without describing them",
            ));
        };

        if let Some(object) = params_json.as_object() {
            object.get(parameter_name).cloned().ok_or_else(|| {
                IncomingRpcMethodError::invalid_parameter(format!(
                    "Params passed by-name, but expected param [{parameter_name}] does not exist"
                ))
            })
        } else if let Some(array) = params_json.as_array() {
            if array.is_empty() || array.len() > 1 {
                Err(IncomingRpcMethodError::invalid_parameter(
                    "Expected exactly one element in the params array",
                ))
            } else {
                Ok(array[0].clone())
            }
        } else {
            Err(IncomingRpcMethodError::invalid_parameter(
                "Expected params as array or named",
            ))
        }
    }

    pub fn validate_parameterless_params(
        &self,
        params_json: Option<&serde_json::Value>,
    ) -> Result<(), IncomingRpcMethodError> {
        if params_json.is_none()
            || params_json
                .and_then(serde_json::Value::as_array)
                .is_some_and(Vec::is_empty)
        {
            if self.param.is_some() {
                Err(IncomingRpcMethodError::illegal_argument(
                    "Parameterless method unexpectedly has parameter description",
                ))
            } else {
                Ok(())
            }
        } else {
            Err(IncomingRpcMethodError::invalid_parameter(
                "Expected no params, or an empty array",
            ))
        }
    }

    pub fn encode_result(
        &self,
        result: serde_json::Value,
    ) -> Result<serde_json::Value, IncomingRpcMethodError> {
        if self.result.is_none() {
            Err(IncomingRpcMethodError::illegal_state(
                "No result codec defined",
            ))
        } else {
            Ok(result)
        }
    }
}

impl IncomingRpcMethodError {
    fn illegal_argument(message: impl Into<String>) -> Self {
        Self {
            kind: IncomingRpcMethodErrorKind::IllegalArgument,
            message: message.into(),
        }
    }

    fn illegal_state(message: impl Into<String>) -> Self {
        Self {
            kind: IncomingRpcMethodErrorKind::IllegalState,
            message: message.into(),
        }
    }

    fn invalid_parameter(message: impl Into<String>) -> Self {
        Self {
            kind: IncomingRpcMethodErrorKind::InvalidParameter,
            message: message.into(),
        }
    }
}
