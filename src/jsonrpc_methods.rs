#![allow(dead_code)]

use crate::chat_component::{Component, ComponentArgument};

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
    }
}
