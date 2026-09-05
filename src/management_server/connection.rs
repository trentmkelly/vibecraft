//! Connection lifecycle, pending calls, and response correlation.
use super::*;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcConnectionWrite {
    Single(String),
    Batch(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcConnectionEvent {
    Connected { client_info: i32, remote_address: String },
    Disconnected { client_info: i32, remote_address: String },
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcConnectionModel {
    pub client_info: i32,
    pub transaction_id: i32,
    pub pending_requests: BTreeMap<i32, PendingRpcRequest>,
    pub completed_requests: BTreeMap<i32, Result<String, String>>,
    pub writes: Vec<JsonRpcConnectionWrite>,
    pub events: Vec<JsonRpcConnectionEvent>,
    pub log_messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagementServerRuntimeModel {
    pub host: String,
    pub requested_port: u16,
    pub authentication_handler: String,
    pub server_channel_port: Option<u16>,
    pub nio_event_loop_group_closed: bool,
    pub connections: Vec<JsonRpcConnectionModel>,
    pub pipelines: Vec<ManagementServerPipeline>,
    pub log_messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagementServerPipeline {
    pub tls: bool,
    pub handlers: Vec<&'static str>,
    pub tcp_no_delay_attempted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JsonRpcConnectionDispatchError {
    error: JsonRpcError,
    data: String,
}

impl ManagementServerRuntimeModel {
    pub const THREAD_NAME_FORMAT: &'static str = "Management server IO #%d";
    pub const WEBSOCKET_PATH: &'static str = "/";
    pub const HTTP_OBJECT_AGGREGATOR_LIMIT: usize = 65_536;

    pub fn new(host: impl Into<String>, port: u16, authentication_handler: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            requested_port: port,
            authentication_handler: authentication_handler.into(),
            server_channel_port: None,
            nio_event_loop_group_closed: false,
            connections: Vec::new(),
            pipelines: Vec::new(),
            log_messages: Vec::new(),
        }
    }

    pub fn on_connected(&mut self, connection: JsonRpcConnectionModel) {
        if !self
            .connections
            .iter()
            .any(|existing| existing.client_info == connection.client_info)
        {
            self.connections.push(connection);
        }
    }

    pub fn on_disconnected(&mut self, client_info: i32) {
        self.connections
            .retain(|connection| connection.client_info != client_info);
    }

    pub fn start_without_tls(&mut self, bound_port: u16) {
        self.start(false, bound_port);
    }

    pub fn start_with_tls(&mut self, bound_port: u16) {
        self.start(true, bound_port);
    }

    fn start(&mut self, tls: bool, bound_port: u16) {
        self.server_channel_port = Some(bound_port);
        self.pipelines.push(ManagementServerPipeline::new(tls));
        self.log_messages.push(format!(
            "Json-RPC Management connection listening on {}:{}",
            self.host,
            self.get_port()
        ));
    }

    pub fn stop(&mut self, close_nio_event_loop_group: bool) {
        self.server_channel_port = None;
        self.connections.clear();
        if close_nio_event_loop_group {
            self.nio_event_loop_group_closed = true;
        }
    }

    pub fn tick(&mut self, current_time: u64) {
        self.for_each_connection_mut(|connection| connection.tick(current_time));
    }

    pub fn get_port(&self) -> u16 {
        self.server_channel_port.unwrap_or(self.requested_port)
    }

    pub fn for_each_connection_mut(&mut self, mut action: impl FnMut(&mut JsonRpcConnectionModel)) {
        for connection in &mut self.connections {
            action(connection);
        }
    }
}

impl ManagementServerPipeline {
    pub fn new(tls: bool) -> Self {
        let mut handlers = Vec::new();
        if tls {
            handlers.push("SslHandler");
        }
        handlers.extend([
            "HttpServerCodec",
            "HttpObjectAggregator(65536)",
            "AuthenticationHandler",
            "WebSocketServerProtocolHandler(/)",
            "WebSocketToJsonCodec",
            "JsonToWebSocketEncoder",
            "Connection",
        ]);
        Self {
            tls,
            handlers,
            tcp_no_delay_attempted: true,
        }
    }
}

impl JsonRpcConnectionModel {
    pub const REQUEST_TIMEOUT_MILLIS: u64 = 5_000;

    pub fn new(client_info: i32) -> Self {
        Self {
            client_info,
            transaction_id: 0,
            pending_requests: BTreeMap::new(),
            completed_requests: BTreeMap::new(),
            writes: Vec::new(),
            events: Vec::new(),
            log_messages: Vec::new(),
        }
    }

    pub fn tick(&mut self, current_time: u64) {
        let timed_out = self
            .pending_requests
            .iter()
            .filter_map(|(id, request)| request.timed_out(current_time).then_some(*id))
            .collect::<Vec<_>>();
        for id in timed_out {
            if let Some(mut request) = self.pending_requests.remove(&id) {
                request.result = Some(Err(format!(
                    "RPC method {} timed out waiting for response",
                    request.method
                )));
                if let Some(result) = request.result {
                    self.completed_requests.insert(id, result);
                }
            }
        }
    }

    pub fn channel_active(&mut self, remote_address: &str) {
        self.log_messages.push(format!(
            "Management connection opened for {remote_address}"
        ));
        self.events.push(JsonRpcConnectionEvent::Connected {
            client_info: self.client_info,
            remote_address: remote_address.to_string(),
        });
    }

    pub fn channel_inactive(&mut self, remote_address: &str) {
        self.log_messages.push(format!(
            "Management connection closed for {remote_address}"
        ));
        self.events.push(JsonRpcConnectionEvent::Disconnected {
            client_info: self.client_info,
            remote_address: remote_address.to_string(),
        });
    }

    pub fn exception_caught(&mut self, cause_message: &str, json_parse_exception: bool) {
        if json_parse_exception {
            self.writes
                .push(JsonRpcConnectionWrite::Single(
                    JsonRpcError::PARSE_ERROR
                        .create_with_unknown_id(Some(cause_message))
                        .to_json(),
                ));
        } else {
            self.events.push(JsonRpcConnectionEvent::Closed);
        }
    }

    pub fn read_json(&mut self, json: &serde_json::Value) {
        if let Some(object) = json.as_object() {
            if let Some(response) = self.handle_json_object(object) {
                self.writes.push(JsonRpcConnectionWrite::Single(response));
            }
        } else if let Some(array) = json.as_array() {
            let responses = array
                .iter()
                .filter_map(serde_json::Value::as_object)
                .filter_map(|object| self.handle_json_object(object))
                .collect::<Vec<_>>();
            self.writes.push(JsonRpcConnectionWrite::Batch(responses));
        } else {
            self.writes
                .push(JsonRpcConnectionWrite::Single(
                    JsonRpcError::INVALID_REQUEST
                        .create_with_unknown_id(None)
                        .to_json(),
                ));
        }
    }

    pub fn send_notification(&mut self, method: &str, params_json: &[&str]) {
        self.writes.push(JsonRpcConnectionWrite::Single(
            JsonRpcUtils::create_request(None, method, params_json),
        ));
    }

    pub fn send_request(&mut self, method: &str, params_json: &[&str], current_time: u64) -> i32 {
        self.transaction_id += 1;
        let id = self.transaction_id;
        self.pending_requests.insert(
            id,
            PendingRpcRequest::new(method, current_time + Self::REQUEST_TIMEOUT_MILLIS),
        );
        self.writes.push(JsonRpcConnectionWrite::Single(
            JsonRpcUtils::create_request(Some(id), method, params_json),
        ));
        id
    }

    pub fn handle_json_object(
        &mut self,
        json_object: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<String> {
        let id = json_object.get("id");
        let method = json_object
            .get("method")
            .and_then(serde_json::Value::as_str);
        let result = json_object.get("result");
        let params = json_object.get("params");
        let error = json_object
            .get("error")
            .and_then(serde_json::Value::as_object);

        if let (Some(method), None, None) = (method, result, error) {
            return if id.is_some_and(|id| !is_valid_request_id_value(id)) {
                Some(
                    JsonRpcError::INVALID_REQUEST
                        .create_with_unknown_id(Some(
                            "Invalid request id - only String, Number and NULL supported",
                        ))
                        .to_json(),
                )
            } else {
                self.handle_incoming_request(id, method, params)
            };
        }

        if method.is_none() && result.is_some() && error.is_none() && id.is_some() {
            let id = id?;
            if let Some(response_id) = valid_response_id_value(id) {
                if let Some(result) = result {
                    self.handle_request_response(response_id, result);
                }
            } else {
                self.log_messages.push(format!(
                    "Received respose {} with id {} we did not request",
                    result.map_or_else(|| "null".to_string(), ToString::to_string),
                    id
                ));
            }
            None
        } else if method.is_none() && result.is_none() && error.is_some() {
            if let Some(error) = error {
                self.handle_error(id, error);
            }
            None
        } else {
            Some(
                JsonRpcError::INVALID_REQUEST
                    .create_without_data(id.map_or(JsonRpcId::Null, json_rpc_id_from_value))
                    .to_json(),
            )
        }
    }

    fn handle_incoming_request(
        &mut self,
        id: Option<&serde_json::Value>,
        method: &str,
        params: Option<&serde_json::Value>,
    ) -> Option<String> {
        let send_response = id.is_some();
        match self.dispatch_incoming_request(method, params) {
            Ok(Some(result)) if send_response => id.map(|id| {
                JsonRpcUtils::create_success_result(json_rpc_id_from_value(id), &result.to_string())
            }),
            Ok(_) => None,
            Err(dispatch_error) if send_response => id.map(|id| {
                dispatch_error
                    .error
                    .create(
                        json_rpc_id_from_value(id),
                        &dispatch_error.data,
                    )
                    .to_json()
            }),
            Err(_) => None,
        }
    }

    fn dispatch_incoming_request(
        &self,
        method: &str,
        params: Option<&serde_json::Value>,
    ) -> Result<Option<serde_json::Value>, JsonRpcConnectionDispatchError> {
        if method.is_empty() || IdentifierLike::try_parse(method).is_none() {
            return Err(JsonRpcConnectionDispatchError {
                error: JsonRpcError::INVALID_REQUEST,
                data: format!("Failed to parse method value: {method}"),
            });
        }
        let Some(definition) = INCOMING_RPC_METHOD_DEFS
            .iter()
            .find(|definition| definition.method == method)
        else {
            return Err(JsonRpcConnectionDispatchError {
                error: JsonRpcError::METHOD_NOT_FOUND,
                data: format!("Method not found: {method}"),
            });
        };
        let model = if let Some((param_name, param_schema)) = definition.param {
            IncomingRpcMethodBuilderModel::method_with_params()
                .description(definition.description)
                .param(param_name, param_schema)
                .response(definition.result.0, definition.result.1)
                .build()
                .map_err(incoming_error_to_connection_dispatch_error)?
        } else {
            IncomingRpcMethodBuilderModel::method_parameterless()
                .description(definition.description)
                .response(definition.result.0, definition.result.1)
                .build()
                .map_err(incoming_error_to_connection_dispatch_error)?
        };
        model
            .apply(params, serde_json::json!(null))
            .map(Some)
            .map_err(incoming_error_to_connection_dispatch_error)
    }

    fn handle_request_response(&mut self, id: i32, result: &serde_json::Value) {
        if let Some(mut request) = self.pending_requests.remove(&id) {
            request.accept(result, |value| Ok(Some(value.to_string())));
            if let Some(result) = request.result {
                self.completed_requests.insert(id, result);
            }
        } else {
            self.log_messages
                .push(format!("Received unknown response (id: {id}): {result}"));
        }
    }

    fn handle_error(
        &mut self,
        id: Option<&serde_json::Value>,
        error: &serde_json::Map<String, serde_json::Value>,
    ) {
        if let Some(response_id) = id.and_then(valid_response_id_value) {
            if let Some(mut request) = self.pending_requests.remove(&response_id) {
                request.result = Some(Err(format!(
                    "Remote RPC error (id: {response_id}): {error:?}"
                )));
                if let Some(result) = request.result {
                    self.completed_requests.insert(response_id, result);
                }
            }
        }
        self.log_messages
            .push(format!("Received error (id: {:?}): {:?}", id, error));
    }
}

struct IdentifierLike;

impl IdentifierLike {
    fn try_parse(method: &str) -> Option<()> {
        (!method.is_empty()
            && !method.contains(' ')
            && !method.contains(':')
            && method
                .chars()
                .all(|ch| {
                    ch.is_ascii_lowercase()
                        || ch.is_ascii_digit()
                        || matches!(ch, '_' | '/' | '.')
                }))
        .then_some(())
    }
}

fn is_valid_request_id_value(id: &serde_json::Value) -> bool {
    id.is_null() || id.is_i64() || id.is_u64() || id.is_f64() || id.is_string()
}

fn valid_response_id_value(id: &serde_json::Value) -> Option<i32> {
    id.as_i64()
        .and_then(|id| i32::try_from(id).ok())
        .or_else(|| id.as_u64().and_then(|id| i32::try_from(id).ok()))
}

fn json_rpc_id_from_value(id: &serde_json::Value) -> JsonRpcId {
    if id.is_null() {
        JsonRpcId::Null
    } else if let Some(id) = id.as_i64() {
        JsonRpcId::Number(id)
    } else if let Some(id) = id.as_u64() {
        JsonRpcId::Number(i64::try_from(id).unwrap_or(i64::MAX))
    } else if let Some(id) = id.as_str() {
        JsonRpcId::String(id.to_string())
    } else {
        JsonRpcId::Null
    }
}

fn incoming_error_to_connection_dispatch_error(
    error: IncomingRpcMethodError,
) -> JsonRpcConnectionDispatchError {
    let rpc_error = match error.kind {
        IncomingRpcMethodErrorKind::InvalidParameter => JsonRpcError::INVALID_PARAMS,
        IncomingRpcMethodErrorKind::IllegalArgument | IncomingRpcMethodErrorKind::IllegalState => {
            JsonRpcError::INTERNAL_ERROR
        }
    };
    JsonRpcConnectionDispatchError {
        error: rpc_error,
        data: error.message,
    }
}
