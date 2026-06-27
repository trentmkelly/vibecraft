#![allow(dead_code)]

use crate::management_security::ManagementSecurityConfig;

pub const AUTHENTICATED_KEY: &str = "authenticated";
pub const ATTR_WEBSOCKET_ALLOWED: &str = "websocket_auth_allowed";
pub const SUBPROTOCOL_VALUE: &str = "minecraft-v1";
pub const SUBPROTOCOL_HEADER_PREFIX: &str = "minecraft-v1,";
pub const BEARER_PREFIX: &str = "Bearer ";
pub const UNAUTHORIZED_BODY_ERROR: &str = "Unauthorized";
pub const CONTENT_TYPE_JSON: &str = "application/json";
pub const CONNECTION_CLOSE: &str = "close";
pub const SWITCHING_PROTOCOLS_STATUS: u16 = 101;
pub const UNAUTHORIZED_STATUS: u16 = 401;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcAuthenticationHandler {
    security_config: ManagementSecurityConfig,
    allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcHttpRequest {
    pub authorization: Option<String>,
    pub sec_websocket_protocol: Option<String>,
    pub origin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcHttpResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub content_length: usize,
    pub connection: Option<String>,
    pub body: String,
    pub sec_websocket_protocol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonRpcAuthenticationState {
    pub authenticated: Option<bool>,
    pub websocket_allowed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcAuthenticationAction {
    Forward,
    Close,
    Unauthorized(JsonRpcHttpResponse),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcSecurityCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub token_sent_in_sec_websocket_protocol: bool,
}

impl JsonRpcAuthenticationHandler {
    pub fn new(security_config: ManagementSecurityConfig, allowed_origins: &str) -> Self {
        Self {
            security_config,
            allowed_origins: allowed_origins.split(',').map(ToOwned::to_owned).collect(),
        }
    }

    pub fn channel_read_http(
        &self,
        state: &mut JsonRpcAuthenticationState,
        request: &JsonRpcHttpRequest,
    ) -> JsonRpcAuthenticationAction {
        let result = self.perform_security_checks(request);
        if !result.allowed {
            state.authenticated = Some(false);
            let reason = result.reason.as_deref().map_or("", |reason| reason);
            return JsonRpcAuthenticationAction::Unauthorized(unauthorized_response(
                reason,
            ));
        }

        state.authenticated = Some(true);
        if result.token_sent_in_sec_websocket_protocol {
            state.websocket_allowed = Some(true);
        }
        JsonRpcAuthenticationAction::Forward
    }

    pub fn channel_read_non_http(
        &self,
        state: &JsonRpcAuthenticationState,
    ) -> JsonRpcAuthenticationAction {
        if state.authenticated == Some(true) {
            JsonRpcAuthenticationAction::Forward
        } else {
            JsonRpcAuthenticationAction::Close
        }
    }

    pub fn write_response(
        &self,
        state: &JsonRpcAuthenticationState,
        mut response: JsonRpcHttpResponse,
    ) -> JsonRpcHttpResponse {
        if response.status == SWITCHING_PROTOCOLS_STATUS && state.websocket_allowed == Some(true) {
            response.sec_websocket_protocol = Some(SUBPROTOCOL_VALUE.to_string());
        }
        response
    }

    pub fn perform_security_checks(
        &self,
        request: &JsonRpcHttpRequest,
    ) -> JsonRpcSecurityCheckResult {
        if let Some(token) = parse_token_in_authorization_header(request) {
            return if self.is_valid_api_key(&token) {
                JsonRpcSecurityCheckResult::allowed()
            } else {
                JsonRpcSecurityCheckResult::denied("Invalid API key")
            };
        }

        if let Some(token) = parse_token_in_sec_websocket_protocol_header(request) {
            if !self.is_allowed_origin_header(request) {
                return JsonRpcSecurityCheckResult::denied("Origin Not Allowed");
            }
            return if self.is_valid_api_key(&token) {
                JsonRpcSecurityCheckResult::allowed_with_websocket_protocol()
            } else {
                JsonRpcSecurityCheckResult::denied("Invalid API key")
            };
        }

        JsonRpcSecurityCheckResult::denied("Missing API key")
    }

    pub fn is_allowed_origin_header(&self, request: &JsonRpcHttpRequest) -> bool {
        let Some(origin) = &request.origin else {
            return false;
        };
        !origin.is_empty() && self.allowed_origins.contains(origin)
    }

    pub fn is_valid_api_key(&self, supplied_key: &str) -> bool {
        if supplied_key.is_empty() {
            return false;
        }
        constant_time_eq(
            supplied_key.as_bytes(),
            self.security_config.secret_key.as_bytes(),
        )
    }
}

impl JsonRpcSecurityCheckResult {
    fn allowed() -> Self {
        Self {
            allowed: true,
            reason: None,
            token_sent_in_sec_websocket_protocol: false,
        }
    }

    fn allowed_with_websocket_protocol() -> Self {
        Self {
            allowed: true,
            reason: None,
            token_sent_in_sec_websocket_protocol: true,
        }
    }

    fn denied(reason: &str) -> Self {
        Self {
            allowed: false,
            reason: Some(reason.to_string()),
            token_sent_in_sec_websocket_protocol: false,
        }
    }
}

pub fn parse_token_in_authorization_header(request: &JsonRpcHttpRequest) -> Option<String> {
    let header = request.authorization.as_ref()?;
    header
        .strip_prefix(BEARER_PREFIX)
        .map(|token| token.trim().to_string())
}

pub fn parse_token_in_sec_websocket_protocol_header(
    request: &JsonRpcHttpRequest,
) -> Option<String> {
    let header = request.sec_websocket_protocol.as_ref()?;
    header
        .strip_prefix(SUBPROTOCOL_HEADER_PREFIX)
        .map(|token| token.trim().to_string())
}

fn unauthorized_response(reason: &str) -> JsonRpcHttpResponse {
    let body = format!("{{\"error\":\"{UNAUTHORIZED_BODY_ERROR}\",\"message\":\"{reason}\"}}");
    JsonRpcHttpResponse {
        status: UNAUTHORIZED_STATUS,
        content_type: Some(CONTENT_TYPE_JSON.to_string()),
        content_length: body.len(),
        connection: Some(CONNECTION_CLOSE.to_string()),
        body,
        sec_websocket_protocol: None,
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let max_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();
    for index in 0..max_len {
        let a = match left.get(index) {
            Some(byte) => *byte,
            None => 0,
        };
        let b = match right.get(index) {
            Some(byte) => *byte,
            None => 0,
        };
        diff |= (a ^ b) as usize;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCD";

    fn handler() -> JsonRpcAuthenticationHandler {
        JsonRpcAuthenticationHandler::new(
            ManagementSecurityConfig {
                secret_key: SECRET.to_string(),
                tls_enabled: false,
                tls_keystore: None,
                tls_keystore_password: None,
            },
            "https://admin.example,https://other.example",
        )
    }

    #[test]
    fn authorization_header_takes_precedence_and_bypasses_origin_check() {
        let handler = handler();
        let request = JsonRpcHttpRequest {
            authorization: Some(format!("Bearer   {SECRET}  ")),
            sec_websocket_protocol: Some("minecraft-v1, wrong".to_string()),
            origin: Some("https://evil.example".to_string()),
        };

        assert_eq!(
            parse_token_in_authorization_header(&request),
            Some(SECRET.to_string())
        );
        assert_eq!(
            handler.perform_security_checks(&request),
            JsonRpcSecurityCheckResult {
                allowed: true,
                reason: None,
                token_sent_in_sec_websocket_protocol: false,
            }
        );
    }

    #[test]
    fn websocket_protocol_token_requires_allowed_non_empty_origin() {
        let handler = handler();
        let valid_protocol = Some(format!("minecraft-v1,  {SECRET}  "));
        let allowed = JsonRpcHttpRequest {
            sec_websocket_protocol: valid_protocol.clone(),
            origin: Some("https://admin.example".to_string()),
            ..JsonRpcHttpRequest::default()
        };
        assert_eq!(
            handler.perform_security_checks(&allowed),
            JsonRpcSecurityCheckResult {
                allowed: true,
                reason: None,
                token_sent_in_sec_websocket_protocol: true,
            }
        );

        for origin in [None, Some(String::new()), Some("https://evil.example".to_string())] {
            let denied = JsonRpcHttpRequest {
                sec_websocket_protocol: valid_protocol.clone(),
                origin,
                ..JsonRpcHttpRequest::default()
            };
            assert_eq!(
                handler.perform_security_checks(&denied),
                JsonRpcSecurityCheckResult::denied("Origin Not Allowed")
            );
        }
    }

    #[test]
    fn missing_or_invalid_api_key_returns_java_denial_reasons_and_response_shape() {
        let handler = handler();
        assert_eq!(
            handler.perform_security_checks(&JsonRpcHttpRequest::default()),
            JsonRpcSecurityCheckResult::denied("Missing API key")
        );
        assert_eq!(
            handler.perform_security_checks(&JsonRpcHttpRequest {
                authorization: Some("Bearer wrong".to_string()),
                ..JsonRpcHttpRequest::default()
            }),
            JsonRpcSecurityCheckResult::denied("Invalid API key")
        );
        assert!(!handler.is_valid_api_key(""));

        let mut state = JsonRpcAuthenticationState::default();
        let action = handler.channel_read_http(&mut state, &JsonRpcHttpRequest::default());
        assert_eq!(state.authenticated, Some(false));
        assert_eq!(
            action,
            JsonRpcAuthenticationAction::Unauthorized(JsonRpcHttpResponse {
                status: UNAUTHORIZED_STATUS,
                content_type: Some(CONTENT_TYPE_JSON.to_string()),
                content_length: 52,
                connection: Some(CONNECTION_CLOSE.to_string()),
                body: "{\"error\":\"Unauthorized\",\"message\":\"Missing API key\"}".to_string(),
                sec_websocket_protocol: None,
            })
        );
    }

    #[test]
    fn channel_state_forwards_authenticated_messages_and_closes_others() {
        let handler = handler();
        let mut state = JsonRpcAuthenticationState::default();
        assert_eq!(
            handler.channel_read_non_http(&state),
            JsonRpcAuthenticationAction::Close
        );

        assert_eq!(
            handler.channel_read_http(
                &mut state,
                &JsonRpcHttpRequest {
                    authorization: Some(format!("Bearer {SECRET}")),
                    ..JsonRpcHttpRequest::default()
                },
            ),
            JsonRpcAuthenticationAction::Forward
        );
        assert_eq!(state.authenticated, Some(true));
        assert_eq!(
            handler.channel_read_non_http(&state),
            JsonRpcAuthenticationAction::Forward
        );
    }

    #[test]
    fn websocket_auth_rewrites_switching_protocol_response_only_after_protocol_token_auth() {
        let handler = handler();
        let mut state = JsonRpcAuthenticationState::default();
        assert_eq!(
            handler.channel_read_http(
                &mut state,
                &JsonRpcHttpRequest {
                    sec_websocket_protocol: Some(format!("minecraft-v1,{SECRET}")),
                    origin: Some("https://admin.example".to_string()),
                    ..JsonRpcHttpRequest::default()
                },
            ),
            JsonRpcAuthenticationAction::Forward
        );

        let response = JsonRpcHttpResponse {
            status: SWITCHING_PROTOCOLS_STATUS,
            content_type: None,
            content_length: 0,
            connection: None,
            body: String::new(),
            sec_websocket_protocol: None,
        };
        assert_eq!(
            handler.write_response(&state, response).sec_websocket_protocol,
            Some(SUBPROTOCOL_VALUE.to_string())
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn authentication_handler_source_matches_java_26_1_2() {
        const AUTHENTICATION_HANDLER: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/security/AuthenticationHandler.java"
        );

        for sentinel in [
            "private static final AttributeKey<Boolean> AUTHENTICATED_KEY = AttributeKey.valueOf(\"authenticated\");",
            "private static final AttributeKey<Boolean> ATTR_WEBSOCKET_ALLOWED = AttributeKey.valueOf(\"websocket_auth_allowed\");",
            "private static final String SUBPROTOCOL_VALUE = \"minecraft-v1\";",
            "private static final String SUBPROTOCOL_HEADER_PREFIX = \"minecraft-v1,\";",
            "public static final String BEARER_PREFIX = \"Bearer \";",
            "this.allowedOrigins = Sets.newHashSet(allowedOrigins.split(\",\"));",
            "AuthenticationHandler.SecurityCheckResult result = this.performSecurityChecks(request);",
            "context.channel().attr(AUTHENTICATED_KEY).set(false);",
            "this.sendUnauthorizedResponse(context, result.getReason());",
            "context.channel().attr(AUTHENTICATED_KEY).set(true);",
            "context.channel().attr(ATTR_WEBSOCKET_ALLOWED).set(Boolean.TRUE);",
            "Boolean.TRUE.equals(isAuthenticated)",
            "response.headers().set(HttpHeaderNames.SEC_WEBSOCKET_PROTOCOL, \"minecraft-v1\");",
            "String tokenInAuthorizationHeader = this.parseTokenInAuthorizationHeader(request);",
            "String tokenInSecWebsocketProtocolHeader = this.parseTokenInSecWebsocketProtocolHeader(request);",
            "return AuthenticationHandler.SecurityCheckResult.denied(\"Origin Not Allowed\");",
            "return AuthenticationHandler.SecurityCheckResult.denied(\"Missing API key\");",
            "originHeader != null && !originHeader.isEmpty() ? this.allowedOrigins.contains(originHeader) : false;",
            "authHeader != null && authHeader.startsWith(\"Bearer \") ? authHeader.substring(\"Bearer \".length()).trim() : null;",
            "authHeader != null && authHeader.startsWith(\"minecraft-v1,\") ? authHeader.substring(\"minecraft-v1,\".length()).trim() : null;",
            "if (suppliedKey.isEmpty())",
            "return MessageDigest.isEqual(suppliedKeyBytes, configuredKeyBytes);",
            "String responseBody = \"{\\\"error\\\":\\\"Unauthorized\\\",\\\"message\\\":\\\"\" + reason + \"\\\"}\";",
            "response.headers().set(HttpHeaderNames.CONTENT_TYPE, \"application/json\");",
            "response.headers().set(HttpHeaderNames.CONNECTION, \"close\");",
            "private static class SecurityCheckResult",
        ] {
            assert!(
                AUTHENTICATION_HANDLER.contains(sentinel),
                "AuthenticationHandler.java is missing sentinel: {sentinel}"
            );
        }
    }
}
