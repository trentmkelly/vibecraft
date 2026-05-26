#![allow(dead_code)]

use std::collections::{BTreeMap, VecDeque};

use crate::network::codec::Uuid;
use crate::server_properties::ServerProperties;

pub const SIGNATURE_CACHE_SIZE: usize = 20;
pub const CHAT_CHAIN_BROKEN: &str = "multiplayer.disconnect.chat_validation_failed";
pub const INVALID_COMMAND_SIGNATURE: &str = "chat.disabled.invalid_signature";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageSignature(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageBody {
    pub content: String,
    pub timestamp_millis: i64,
    pub salt: i64,
    pub last_seen: Vec<MessageSignature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedMessageLink {
    pub sender: Uuid,
    pub session_id: Uuid,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessage {
    pub link: SignedMessageLink,
    pub body: SignedMessageBody,
    pub signature: MessageSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatChain {
    sender: Uuid,
    session_id: Uuid,
    next_index: u32,
    previous_signature: Option<MessageSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatTrustDecision {
    Accept,
    Disconnect(&'static str),
}

pub trait MessageSignatureValidator {
    fn validate(&self, payload: &[u8], signature: &MessageSignature) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignatureCache {
    signatures: VecDeque<MessageSignature>,
    capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientChatPresentation {
    Signed { decorated: String },
    Unsigned { decorated: String },
    Modified { decorated: String, unsigned: String },
    Filtered,
    Deleted { signature: MessageSignature },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedCommandArguments {
    pub command: String,
    pub signed_arguments: BTreeMap<String, MessageSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextFilterResult {
    PassThrough(String),
    FullyFiltered,
    PartiallyFiltered { raw: String, mask: Vec<bool> },
    ServiceUnavailableFallback(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextFilterConfig {
    Disabled,
    Legacy(LegacyTextFilterConfig),
    PlayerSafety(PlayerSafetyTextFilterConfig),
    UnsupportedVersion(u32),
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyTextFilterConfig {
    pub api_server: String,
    pub api_key: String,
    pub rule_id: i32,
    pub server_id: String,
    pub room_id: String,
    pub hashes_to_drop: i32,
    pub max_concurrent_requests: u32,
    pub chat_endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSafetyTextFilterConfig {
    pub api_server: String,
    pub api_path: String,
    pub scope: String,
    pub server_id: String,
    pub application_id: String,
    pub tenant_id: String,
    pub room_id: String,
    pub certificate_path: String,
    pub hashes_to_drop: i32,
    pub max_concurrent_requests: u32,
    pub connection_read_timeout_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerReportMetadata {
    pub sender: Uuid,
    pub session_id: Option<Uuid>,
    pub message_index: Option<u32>,
    pub signature: Option<MessageSignature>,
    pub signed_body: Option<SignedMessageBody>,
    pub reported_text: String,
    pub filter_result: Option<TextFilterResult>,
}

impl ChatChain {
    pub fn new(sender: Uuid, session_id: Uuid) -> Self {
        Self {
            sender,
            session_id,
            next_index: 0,
            previous_signature: None,
        }
    }

    pub fn validate<V: MessageSignatureValidator>(
        &mut self,
        message: SignedMessage,
        cache: &mut MessageSignatureCache,
        validator: &V,
    ) -> ChatTrustDecision {
        if message.link.sender != self.sender
            || message.link.session_id != self.session_id
            || message.link.index != self.next_index
        {
            return ChatTrustDecision::Disconnect(CHAT_CHAIN_BROKEN);
        }
        if !cache.contains_all(&message.body.last_seen) {
            return ChatTrustDecision::Disconnect(CHAT_CHAIN_BROKEN);
        }

        let payload = signed_message_payload(
            &message.link,
            &message.body,
            self.previous_signature.as_ref(),
        );
        if !validator.validate(&payload, &message.signature) {
            return ChatTrustDecision::Disconnect(CHAT_CHAIN_BROKEN);
        }

        self.next_index += 1;
        self.previous_signature = Some(message.signature.clone());
        cache.push(message.signature);
        ChatTrustDecision::Accept
    }
}

impl MessageSignatureCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            signatures: VecDeque::new(),
            capacity,
        }
    }

    pub fn vanilla() -> Self {
        Self::new(SIGNATURE_CACHE_SIZE)
    }

    pub fn push(&mut self, signature: MessageSignature) {
        if self.signatures.contains(&signature) {
            return;
        }
        self.signatures.push_front(signature);
        while self.signatures.len() > self.capacity {
            self.signatures.pop_back();
        }
    }

    pub fn contains_all(&self, signatures: &[MessageSignature]) -> bool {
        signatures
            .iter()
            .all(|signature| self.signatures.contains(signature))
    }
}

impl SignedCommandArguments {
    pub fn validate_required(&self, required_arguments: &[&str]) -> Result<(), &'static str> {
        if required_arguments
            .iter()
            .all(|argument| self.signed_arguments.contains_key(*argument))
        {
            Ok(())
        } else {
            Err(INVALID_COMMAND_SIGNATURE)
        }
    }
}

pub fn present_player_chat(
    signed_body: Option<&SignedMessageBody>,
    decorated: Option<String>,
    filtered: bool,
    deleted: Option<MessageSignature>,
) -> ClientChatPresentation {
    if let Some(signature) = deleted {
        return ClientChatPresentation::Deleted { signature };
    }
    if filtered {
        return ClientChatPresentation::Filtered;
    }
    match (signed_body, decorated) {
        (Some(body), Some(decorated)) if decorated != body.content => {
            ClientChatPresentation::Modified {
                decorated,
                unsigned: body.content.clone(),
            }
        }
        (Some(_body), Some(decorated)) => ClientChatPresentation::Signed { decorated },
        (Some(body), None) => ClientChatPresentation::Signed {
            decorated: body.content.clone(),
        },
        (None, Some(decorated)) => ClientChatPresentation::Unsigned { decorated },
        (None, None) => ClientChatPresentation::Unsigned {
            decorated: String::new(),
        },
    }
}

pub fn apply_text_filter(
    raw: impl Into<String>,
    blocked_character_mask: Option<Vec<bool>>,
    service_available: bool,
) -> TextFilterResult {
    let raw = raw.into();
    if !service_available {
        return TextFilterResult::ServiceUnavailableFallback(raw);
    }
    let Some(mask) = blocked_character_mask else {
        return TextFilterResult::PassThrough(raw);
    };
    if mask.iter().all(|filtered| *filtered) {
        TextFilterResult::FullyFiltered
    } else if mask.iter().any(|filtered| *filtered) {
        TextFilterResult::PartiallyFiltered { raw, mask }
    } else {
        TextFilterResult::PassThrough(raw)
    }
}

pub fn text_filter_config_from_properties(properties: &ServerProperties) -> TextFilterConfig {
    let config = properties.text_filtering_config.trim();
    if config.is_empty() {
        return TextFilterConfig::Disabled;
    }
    match properties.text_filtering_version {
        0 => parse_legacy_text_filter_config(config),
        1 => parse_player_safety_text_filter_config(config),
        version => TextFilterConfig::UnsupportedVersion(version),
    }
}

fn parse_legacy_text_filter_config(config: &str) -> TextFilterConfig {
    let parsed = match serde_json::from_str::<serde_json::Value>(config) {
        Ok(parsed) => parsed,
        Err(err) => return TextFilterConfig::Invalid(err.to_string()),
    };
    let Some(api_server) = json_string(&parsed, "apiServer") else {
        return TextFilterConfig::Invalid("missing apiServer".to_string());
    };
    let Some(api_key) = json_string(&parsed, "apiKey") else {
        return TextFilterConfig::Invalid("missing apiKey".to_string());
    };
    if api_key.is_empty() {
        return TextFilterConfig::Invalid("missing apiKey".to_string());
    }
    let endpoints = parsed.get("endpoints");
    TextFilterConfig::Legacy(LegacyTextFilterConfig {
        api_server,
        api_key,
        rule_id: json_i32(&parsed, "ruleId", 1),
        server_id: json_string(&parsed, "serverId").unwrap_or_default(),
        room_id: json_string(&parsed, "roomId").unwrap_or_else(|| "Java:Chat".to_string()),
        hashes_to_drop: json_i32(&parsed, "hashesToDrop", -1),
        max_concurrent_requests: json_u32(&parsed, "maxConcurrentRequests", 7),
        chat_endpoint: endpoints
            .and_then(|value| value.get("chat"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("v1/chat")
            .to_string(),
    })
}

fn parse_player_safety_text_filter_config(config: &str) -> TextFilterConfig {
    let parsed = match serde_json::from_str::<serde_json::Value>(config) {
        Ok(parsed) => parsed,
        Err(err) => return TextFilterConfig::Invalid(err.to_string()),
    };
    match player_safety_config_from_json(&parsed) {
        Ok(config) => TextFilterConfig::PlayerSafety(config),
        Err(err) => TextFilterConfig::Invalid(err),
    }
}

fn player_safety_config_from_json(
    parsed: &serde_json::Value,
) -> Result<PlayerSafetyTextFilterConfig, String> {
    Ok(PlayerSafetyTextFilterConfig {
        api_server: required_json_string(parsed, "apiServer")?,
        api_path: required_json_string(parsed, "apiPath")?,
        scope: required_json_string(parsed, "scope")?,
        server_id: json_string(parsed, "serverId").unwrap_or_default(),
        application_id: required_json_string(parsed, "applicationId")?,
        tenant_id: required_json_string(parsed, "tenantId")?,
        room_id: json_string(parsed, "roomId").unwrap_or_else(|| "Java:Chat".to_string()),
        certificate_path: required_json_string(parsed, "certificatePath")?,
        hashes_to_drop: json_i32(parsed, "hashesToDrop", -1),
        max_concurrent_requests: json_u32(parsed, "maxConcurrentRequests", 7),
        connection_read_timeout_ms: json_u32(parsed, "connectionReadTimeoutMs", 2000),
    })
}

fn required_json_string(value: &serde_json::Value, key: &str) -> Result<String, String> {
    json_string(value, key).ok_or_else(|| format!("missing {key}"))
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn json_i32(value: &serde_json::Value, key: &str, default: i32) -> i32 {
    value
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .unwrap_or(default)
}

fn json_u32(value: &serde_json::Value, key: &str, default: u32) -> u32 {
    value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(default)
}

pub fn report_metadata_for_message(
    message: &SignedMessage,
    reported_text: impl Into<String>,
    filter_result: Option<TextFilterResult>,
) -> PlayerReportMetadata {
    PlayerReportMetadata {
        sender: message.link.sender,
        session_id: Some(message.link.session_id),
        message_index: Some(message.link.index),
        signature: Some(message.signature.clone()),
        signed_body: Some(message.body.clone()),
        reported_text: reported_text.into(),
        filter_result,
    }
}

pub fn report_metadata_for_unsigned(
    sender: Uuid,
    reported_text: impl Into<String>,
    filter_result: Option<TextFilterResult>,
) -> PlayerReportMetadata {
    PlayerReportMetadata {
        sender,
        session_id: None,
        message_index: None,
        signature: None,
        signed_body: None,
        reported_text: reported_text.into(),
        filter_result,
    }
}

pub fn signed_message_payload(
    link: &SignedMessageLink,
    body: &SignedMessageBody,
    previous_signature: Option<&MessageSignature>,
) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&link.sender.0);
    payload.extend_from_slice(&link.session_id.0);
    payload.extend_from_slice(&link.index.to_be_bytes());
    payload.extend_from_slice(&body.salt.to_be_bytes());
    payload.extend_from_slice(&body.timestamp_millis.to_be_bytes());
    payload.extend_from_slice(&(body.content.len() as i32).to_be_bytes());
    payload.extend_from_slice(body.content.as_bytes());
    if let Some(previous) = previous_signature {
        payload.extend_from_slice(&(previous.0.len() as i32).to_be_bytes());
        payload.extend_from_slice(&previous.0);
    } else {
        payload.extend_from_slice(&0_i32.to_be_bytes());
    }
    payload.extend_from_slice(&(body.last_seen.len() as i32).to_be_bytes());
    for signature in &body.last_seen {
        payload.extend_from_slice(&(signature.0.len() as i32).to_be_bytes());
        payload.extend_from_slice(&signature.0);
    }
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ExactValidator {
        payload: Vec<u8>,
        signature: MessageSignature,
    }

    impl MessageSignatureValidator for ExactValidator {
        fn validate(&self, payload: &[u8], signature: &MessageSignature) -> bool {
            payload == self.payload && signature == &self.signature
        }
    }

    fn body(content: &str) -> SignedMessageBody {
        SignedMessageBody {
            content: content.to_string(),
            timestamp_millis: 123,
            salt: 456,
            last_seen: Vec::new(),
        }
    }

    #[test]
    fn chat_chain_accepts_next_signed_message_and_remembers_signature() {
        let sender = Uuid([1; 16]);
        let session = Uuid([2; 16]);
        let link = SignedMessageLink {
            sender,
            session_id: session,
            index: 0,
        };
        let body = body("hello");
        let signature = MessageSignature(vec![9]);
        let validator = ExactValidator {
            payload: signed_message_payload(&link, &body, None),
            signature: signature.clone(),
        };
        let mut chain = ChatChain::new(sender, session);
        let mut cache = MessageSignatureCache::vanilla();

        assert_eq!(
            chain.validate(
                SignedMessage {
                    link,
                    body,
                    signature: signature.clone(),
                },
                &mut cache,
                &validator,
            ),
            ChatTrustDecision::Accept
        );
        assert!(cache.contains_all(&[signature]));
    }

    #[test]
    fn chat_chain_rejects_wrong_index_bad_signature_or_unknown_last_seen() {
        let sender = Uuid([1; 16]);
        let session = Uuid([2; 16]);
        let link = SignedMessageLink {
            sender,
            session_id: session,
            index: 1,
        };
        let bad = SignedMessage {
            link,
            body: body("hello"),
            signature: MessageSignature(vec![0]),
        };
        let validator = ExactValidator {
            payload: Vec::new(),
            signature: MessageSignature(vec![1]),
        };
        let mut chain = ChatChain::new(sender, session);
        let mut cache = MessageSignatureCache::vanilla();
        assert_eq!(
            chain.validate(bad, &mut cache, &validator),
            ChatTrustDecision::Disconnect(CHAT_CHAIN_BROKEN)
        );

        let link = SignedMessageLink {
            sender,
            session_id: session,
            index: 0,
        };
        let mut body = body("hello");
        body.last_seen.push(MessageSignature(vec![7]));
        assert_eq!(
            chain.validate(
                SignedMessage {
                    link,
                    body,
                    signature: MessageSignature(vec![1]),
                },
                &mut cache,
                &validator,
            ),
            ChatTrustDecision::Disconnect(CHAT_CHAIN_BROKEN)
        );
    }

    #[test]
    fn chat_presentation_distinguishes_signed_unsigned_modified_filtered_and_deleted() {
        let signed = body("hello");
        assert_eq!(
            present_player_chat(Some(&signed), Some("hello".to_string()), false, None),
            ClientChatPresentation::Signed {
                decorated: "hello".to_string()
            }
        );
        assert_eq!(
            present_player_chat(Some(&signed), Some("* hello".to_string()), false, None),
            ClientChatPresentation::Modified {
                decorated: "* hello".to_string(),
                unsigned: "hello".to_string(),
            }
        );
        assert!(matches!(
            present_player_chat(None, Some("server".to_string()), false, None),
            ClientChatPresentation::Unsigned { .. }
        ));
        assert_eq!(
            present_player_chat(Some(&signed), None, true, None),
            ClientChatPresentation::Filtered
        );
        assert_eq!(
            present_player_chat(Some(&signed), None, false, Some(MessageSignature(vec![3]))),
            ClientChatPresentation::Deleted {
                signature: MessageSignature(vec![3])
            }
        );
    }

    #[test]
    fn signed_command_arguments_require_signatures_for_signable_arguments() {
        let mut signed_arguments = BTreeMap::new();
        signed_arguments.insert("message".to_string(), MessageSignature(vec![1]));
        let command = SignedCommandArguments {
            command: "say hello".to_string(),
            signed_arguments,
        };
        assert_eq!(command.validate_required(&["message"]), Ok(()));
        assert_eq!(
            command.validate_required(&["message", "target"]),
            Err(INVALID_COMMAND_SIGNATURE)
        );
    }

    #[test]
    fn text_filter_results_preserve_vanilla_fallback_and_mask_shapes() {
        assert_eq!(
            apply_text_filter("hello", None, true),
            TextFilterResult::PassThrough("hello".to_string())
        );
        assert_eq!(
            apply_text_filter("bad", Some(vec![true, true, true]), true),
            TextFilterResult::FullyFiltered
        );
        assert_eq!(
            apply_text_filter("bad", Some(vec![true, false, true]), true),
            TextFilterResult::PartiallyFiltered {
                raw: "bad".to_string(),
                mask: vec![true, false, true],
            }
        );
        assert_eq!(
            apply_text_filter("hello", Some(vec![true]), false),
            TextFilterResult::ServiceUnavailableFallback("hello".to_string())
        );
    }

    #[test]
    fn text_filter_config_uses_server_properties_version_dispatch() {
        let mut properties = ServerProperties::load_or_default(std::path::Path::new(
            "definitely-missing-test-server.properties",
        ))
        .unwrap();
        assert_eq!(
            text_filter_config_from_properties(&properties),
            TextFilterConfig::Disabled
        );

        properties.set(
            "text-filtering-config",
            r#"{"apiServer":"https://filter.example","apiKey":"secret","ruleId":2,"serverId":"srv","roomId":"room","hashesToDrop":1,"maxConcurrentRequests":3,"endpoints":{"chat":"v2/chat"}}"#,
        );
        properties.set("text-filtering-version", "0");
        assert_eq!(
            text_filter_config_from_properties(&properties),
            TextFilterConfig::Legacy(LegacyTextFilterConfig {
                api_server: "https://filter.example".to_string(),
                api_key: "secret".to_string(),
                rule_id: 2,
                server_id: "srv".to_string(),
                room_id: "room".to_string(),
                hashes_to_drop: 1,
                max_concurrent_requests: 3,
                chat_endpoint: "v2/chat".to_string(),
            })
        );

        properties.set(
            "text-filtering-config",
            r#"{"apiServer":"https://safety.example","apiPath":"/chat","scope":"scope","applicationId":"app","tenantId":"tenant","certificatePath":"cert.pem","connectionReadTimeoutMs":5000}"#,
        );
        properties.set("text-filtering-version", "1");
        match text_filter_config_from_properties(&properties) {
            TextFilterConfig::PlayerSafety(config) => {
                assert_eq!(config.api_path, "/chat");
                assert_eq!(config.room_id, "Java:Chat");
                assert_eq!(config.connection_read_timeout_ms, 5000);
            }
            other => panic!("unexpected config: {other:?}"),
        }

        properties.set("text-filtering-version", "2");
        assert_eq!(
            text_filter_config_from_properties(&properties),
            TextFilterConfig::UnsupportedVersion(2)
        );
    }

    #[test]
    fn player_report_metadata_keeps_signed_context_when_available() {
        let message = SignedMessage {
            link: SignedMessageLink {
                sender: Uuid([1; 16]),
                session_id: Uuid([2; 16]),
                index: 5,
            },
            body: body("evidence"),
            signature: MessageSignature(vec![8]),
        };
        let metadata = report_metadata_for_message(
            &message,
            "evidence",
            Some(TextFilterResult::PassThrough("evidence".to_string())),
        );
        assert_eq!(metadata.sender, Uuid([1; 16]));
        assert_eq!(metadata.session_id, Some(Uuid([2; 16])));
        assert_eq!(metadata.message_index, Some(5));
        assert_eq!(metadata.signature, Some(MessageSignature(vec![8])));
        assert_eq!(metadata.signed_body.unwrap().content, "evidence");

        let unsigned = report_metadata_for_unsigned(Uuid([3; 16]), "system", None);
        assert_eq!(unsigned.session_id, None);
        assert_eq!(unsigned.signature, None);
    }

    #[test]
    fn signature_cache_keeps_vanilla_bounded_recent_signatures() {
        let mut cache = MessageSignatureCache::new(2);
        cache.push(MessageSignature(vec![1]));
        cache.push(MessageSignature(vec![2]));
        cache.push(MessageSignature(vec![3]));
        assert!(cache.contains_all(&[MessageSignature(vec![3]), MessageSignature(vec![2])]));
        assert!(!cache.contains_all(&[MessageSignature(vec![1])]));
    }
}
