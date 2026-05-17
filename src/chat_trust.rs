#![allow(dead_code)]

use std::collections::{BTreeMap, VecDeque};

use crate::network::codec::Uuid;

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
    fn signature_cache_keeps_vanilla_bounded_recent_signatures() {
        let mut cache = MessageSignatureCache::new(2);
        cache.push(MessageSignature(vec![1]));
        cache.push(MessageSignature(vec![2]));
        cache.push(MessageSignature(vec![3]));
        assert!(cache.contains_all(&[MessageSignature(vec![3]), MessageSignature(vec![2])]));
        assert!(!cache.contains_all(&[MessageSignature(vec![1])]));
    }
}
