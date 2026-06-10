#![allow(dead_code)]

use std::collections::{BTreeMap, VecDeque};

use crate::network::codec::Uuid;
use crate::server_properties::ServerProperties;

pub const SIGNATURE_CACHE_SIZE: usize = 20;
pub const CHAT_CHAIN_BROKEN: &str = "multiplayer.disconnect.chat_validation_failed";
pub const INVALID_COMMAND_SIGNATURE: &str = "chat.disabled.invalid_signature";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageSignature(pub Vec<u8>);

impl MessageSignature {
    pub const BYTES: usize = 256;

    pub fn checksum(&self) -> i32 {
        self.0.iter().fold(1_i32, |result, byte| {
            result.wrapping_mul(31).wrapping_add(i32::from(*byte as i8))
        })
    }

    pub fn pack(&self, cache: &MessageSignatureCache) -> PackedMessageSignatureModel {
        let packed_id = cache.pack(self);
        if packed_id != MessageSignatureCache::NOT_FOUND {
            PackedMessageSignatureModel::Id(packed_id)
        } else {
            PackedMessageSignatureModel::Full(self.clone())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessages {
    pub entries: Vec<MessageSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedLastSeenMessages {
    pub entries: Vec<PackedMessageSignatureModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedMessageSignatureModel {
    Full(MessageSignature),
    Id(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessagesUpdateModel {
    pub offset: i32,
    pub acknowledged: Vec<bool>,
    pub checksum: u8,
}

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
    pub const NOT_FOUND: i32 = -1;

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

    pub fn pack(&self, signature: &MessageSignature) -> i32 {
        self.signatures
            .iter()
            .position(|entry| entry == signature)
            .map(|index| index as i32)
            .unwrap_or(Self::NOT_FOUND)
    }

    pub fn unpack(&self, id: i32) -> Option<MessageSignature> {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.signatures.get(index))
            .cloned()
    }

    pub fn push_body(&mut self, body: &SignedMessageBody, signature: Option<MessageSignature>) {
        let mut queue: VecDeque<MessageSignature> = body.last_seen.iter().cloned().collect();
        if let Some(signature) = signature {
            queue.push_back(signature);
        }
        let new_entries: Vec<MessageSignature> = queue.iter().cloned().collect();

        let mut rebuilt = VecDeque::new();
        while let Some(entry) = queue.pop_back() {
            if rebuilt.len() < self.capacity {
                rebuilt.push_back(entry);
            }
        }
        for entry in self.signatures.iter().cloned() {
            if rebuilt.len() >= self.capacity {
                break;
            }
            if !new_entries.contains(&entry) {
                rebuilt.push_back(entry);
            }
        }
        self.signatures = rebuilt;
    }
}

impl LastSeenMessages {
    pub const LAST_SEEN_MESSAGES_MAX_LENGTH: usize = 20;

    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn update_signature_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.entries.len() as i32).to_be_bytes());
        for entry in &self.entries {
            bytes.extend_from_slice(&entry.0);
        }
        bytes
    }

    pub fn pack(&self, cache: &MessageSignatureCache) -> PackedLastSeenMessages {
        PackedLastSeenMessages {
            entries: self.entries.iter().map(|entry| entry.pack(cache)).collect(),
        }
    }

    pub fn compute_checksum(&self) -> u8 {
        let checksum = self.entries.iter().fold(1_i32, |result, entry| {
            result.wrapping_mul(31).wrapping_add(entry.checksum())
        });
        let checksum_byte = checksum as i8;
        if checksum_byte == 0 {
            1
        } else {
            checksum_byte as u8
        }
    }
}

impl PackedLastSeenMessages {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn unpack(&self, cache: &MessageSignatureCache) -> Option<LastSeenMessages> {
        let mut unpacked = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            unpacked.push(entry.unpack(cache)?);
        }
        Some(LastSeenMessages { entries: unpacked })
    }
}

impl PackedMessageSignatureModel {
    pub const FULL_SIGNATURE: i32 = -1;

    pub fn unpack(&self, cache: &MessageSignatureCache) -> Option<MessageSignature> {
        match self {
            Self::Full(signature) => Some(signature.clone()),
            Self::Id(id) => cache.unpack(*id),
        }
    }
}

impl LastSeenMessagesUpdateModel {
    pub const ACKNOWLEDGED_BITS: usize = 20;
    pub const IGNORE_CHECKSUM: u8 = 0;

    pub fn verify_checksum(&self, last_seen: &LastSeenMessages) -> bool {
        self.checksum == Self::IGNORE_CHECKSUM || self.checksum == last_seen.compute_checksum()
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

    #[test]
    fn last_seen_messages_match_java_checksum_pack_and_update_contracts() {
        const LAST_SEEN_MESSAGES_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/LastSeenMessages.java"
        );
        const MESSAGE_SIGNATURE_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/MessageSignature.java"
        );
        const MESSAGE_SIGNATURE_CACHE_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/MessageSignatureCache.java"
        );

        for sentinel in [
            "public static final LastSeenMessages EMPTY = new LastSeenMessages(List.of());",
            "public static final int LAST_SEEN_MESSAGES_MAX_LENGTH = 20;",
            "output.update(Ints.toByteArray(this.entries.size()));",
            "output.update(entry.bytes());",
            "this.entries.stream().map(entry -> entry.pack(cache)).toList()",
            "checksum = 31 * checksum + entry.checksum();",
            "return checksumByte == 0 ? 1 : checksumByte;",
            "input.readCollection(FriendlyByteBuf.limitValue(ArrayList::new, 20), MessageSignature.Packed::read)",
            "Optional<MessageSignature> entry = packed.unpack(cache);",
            "public static final byte IGNORE_CHECKSUM = 0;",
            "return this.checksum == 0 || this.checksum == lastSeen.computeChecksum();",
        ] {
            assert!(
                LAST_SEEN_MESSAGES_JAVA.contains(sentinel),
                "missing LastSeenMessages sentinel {sentinel}"
            );
        }
        for sentinel in [
            "public static final int BYTES = 256;",
            "Preconditions.checkState(bytes.length == 256, \"Invalid message signature size\");",
            "return packedId != -1 ? new MessageSignature.Packed(packedId) : new MessageSignature.Packed(this);",
            "public int checksum()",
            "return Arrays.hashCode(this.bytes);",
            "public static final int FULL_SIGNATURE = -1;",
            "int id = input.readVarInt() - 1;",
        ] {
            assert!(
                MESSAGE_SIGNATURE_JAVA.contains(sentinel),
                "missing MessageSignature sentinel {sentinel}"
            );
        }
        for sentinel in [
            "public static final int NOT_FOUND = -1;",
            "private static final int DEFAULT_CAPACITY = 128;",
            "return new MessageSignatureCache(128);",
            "if (signature.equals(this.entries[i]))",
            "return this.entries[id];",
            "queue.addAll(lastSeen);",
            "this.entries[i] = queue.removeLast();",
        ] {
            assert!(
                MESSAGE_SIGNATURE_CACHE_JAVA.contains(sentinel),
                "missing MessageSignatureCache sentinel {sentinel}"
            );
        }

        let first = MessageSignature(vec![1; MessageSignature::BYTES]);
        let second = MessageSignature(vec![2; MessageSignature::BYTES]);
        let full = MessageSignature(vec![3; MessageSignature::BYTES]);
        let last_seen = LastSeenMessages {
            entries: vec![first.clone(), second.clone()],
        };

        let mut expected_update_bytes = Vec::new();
        expected_update_bytes.extend_from_slice(&2_i32.to_be_bytes());
        expected_update_bytes.extend_from_slice(&first.0);
        expected_update_bytes.extend_from_slice(&second.0);
        assert_eq!(last_seen.update_signature_bytes(), expected_update_bytes);

        let checksum = last_seen.compute_checksum();
        assert_ne!(checksum, 0);
        assert!(LastSeenMessagesUpdateModel {
            offset: 4,
            acknowledged: vec![true, false, true],
            checksum,
        }
        .verify_checksum(&last_seen));
        assert!(LastSeenMessagesUpdateModel {
            offset: 4,
            acknowledged: Vec::new(),
            checksum: LastSeenMessagesUpdateModel::IGNORE_CHECKSUM,
        }
        .verify_checksum(&last_seen));
        assert!(!LastSeenMessagesUpdateModel {
            offset: 4,
            acknowledged: Vec::new(),
            checksum: checksum.wrapping_add(1),
        }
        .verify_checksum(&last_seen));

        let mut cache = MessageSignatureCache::new(3);
        cache.push(first.clone());
        cache.push(second.clone());
        assert_eq!(cache.pack(&second), 0);
        assert_eq!(cache.pack(&first), 1);
        assert_eq!(cache.pack(&full), MessageSignatureCache::NOT_FOUND);

        let packed = LastSeenMessages {
            entries: vec![second.clone(), full.clone()],
        }
        .pack(&cache);
        assert_eq!(
            packed.entries,
            vec![
                PackedMessageSignatureModel::Id(0),
                PackedMessageSignatureModel::Full(full.clone())
            ]
        );
        assert_eq!(
            packed.unpack(&cache),
            Some(LastSeenMessages {
                entries: vec![second.clone(), full]
            })
        );
        assert_eq!(
            PackedLastSeenMessages {
                entries: vec![PackedMessageSignatureModel::Id(99)]
            }
            .unpack(&cache),
            None
        );

        let body = SignedMessageBody {
            content: "hello".to_string(),
            timestamp_millis: 1,
            salt: 2,
            last_seen: vec![first.clone(), second.clone()],
        };
        let signature = MessageSignature(vec![9; MessageSignature::BYTES]);
        cache.push_body(&body, Some(signature.clone()));
        assert_eq!(cache.unpack(0), Some(signature));
        assert_eq!(cache.unpack(1), Some(second));
        assert_eq!(cache.unpack(2), Some(first));
    }
}
