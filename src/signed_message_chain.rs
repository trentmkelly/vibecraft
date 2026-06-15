use crate::chat_component::filter_mask::FilterMask;
use crate::chat_component::player_chat_message::{
    player_chat_message_signature_payload, PlayerChatMessage,
};
use crate::chat_trust::{
    MessageSignature, MessageSignatureValidator, SignedMessageBody, SignedMessageLink,
};
use crate::network::codec::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageChainModel {
    next_link: Option<SignedMessageLink>,
    last_timestamp_epoch_millis: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignedMessageChainDecodeError {
    MissingProfileKey,
    ChainBroken,
    ExpiredProfileKey,
    InvalidSignature,
    OutOfOrderChat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignedMessageChainEncodeError {
    ChainBroken,
    SigningFailed(String),
}

pub trait SignedMessageSigner {
    fn sign(&self, payload: &[u8]) -> Result<MessageSignature, String>;
}

pub trait ProfileKeyExpiry {
    fn has_expired(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageChainEncoderModel<S> {
    next_link: Option<SignedMessageLink>,
    signer: S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageChainDecoderModel<V, E> {
    next_link: Option<SignedMessageLink>,
    last_timestamp_epoch_millis: i64,
    signature_validator: V,
    profile_key_expiry: E,
}

impl SignedMessageChainModel {
    pub fn new(profile_id: Uuid, session_id: Uuid) -> Self {
        Self {
            next_link: Some(SignedMessageLink::root(profile_id, session_id)),
            last_timestamp_epoch_millis: 0,
        }
    }

    pub fn next_link(&self) -> Option<SignedMessageLink> {
        self.next_link
    }

    pub fn last_timestamp_epoch_millis(&self) -> i64 {
        self.last_timestamp_epoch_millis
    }

    pub fn encoder<S: SignedMessageSigner>(&self, signer: S) -> SignedMessageChainEncoderModel<S> {
        SignedMessageChainEncoderModel {
            next_link: self.next_link,
            signer,
        }
    }

    pub fn decoder<V, E>(
        &self,
        signature_validator: V,
        profile_key_expiry: E,
    ) -> SignedMessageChainDecoderModel<V, E>
    where
        V: MessageSignatureValidator,
        E: ProfileKeyExpiry,
    {
        SignedMessageChainDecoderModel {
            next_link: self.next_link,
            last_timestamp_epoch_millis: self.last_timestamp_epoch_millis,
            signature_validator,
            profile_key_expiry,
        }
    }
}

impl<S: SignedMessageSigner> SignedMessageChainEncoderModel<S> {
    pub fn unsigned() -> SignedMessageChainEncoderModel<UnsignedSigner> {
        SignedMessageChainEncoderModel {
            next_link: None,
            signer: UnsignedSigner,
        }
    }

    pub fn next_link(&self) -> Option<SignedMessageLink> {
        self.next_link
    }

    pub fn pack(
        &mut self,
        body: &SignedMessageBody,
    ) -> Result<Option<MessageSignature>, SignedMessageChainEncodeError> {
        let Some(link) = self.next_link else {
            return Ok(None);
        };
        self.next_link = link.advance();
        let payload = player_chat_message_signature_payload(&link, body);
        self.signer
            .sign(&payload)
            .map(Some)
            .map_err(SignedMessageChainEncodeError::SigningFailed)
    }
}

impl<V, E> SignedMessageChainDecoderModel<V, E>
where
    V: MessageSignatureValidator,
    E: ProfileKeyExpiry,
{
    pub fn unsigned(profile_id: Uuid, enforces_secure_chat: bool) -> UnsignedDecoderModel {
        UnsignedDecoderModel {
            profile_id,
            enforces_secure_chat,
        }
    }

    pub fn next_link(&self) -> Option<SignedMessageLink> {
        self.next_link
    }

    pub fn last_timestamp_epoch_millis(&self) -> i64 {
        self.last_timestamp_epoch_millis
    }

    pub fn unpack(
        &mut self,
        signature: Option<MessageSignature>,
        body: SignedMessageBody,
    ) -> Result<PlayerChatMessage, SignedMessageChainDecodeError> {
        let signature = signature.ok_or(SignedMessageChainDecodeError::MissingProfileKey)?;
        if self.profile_key_expiry.has_expired() {
            return Err(SignedMessageChainDecodeError::ExpiredProfileKey);
        }
        let Some(link) = self.next_link else {
            return Err(SignedMessageChainDecodeError::ChainBroken);
        };
        if body.epoch_millis < self.last_timestamp_epoch_millis {
            self.set_chain_broken();
            return Err(SignedMessageChainDecodeError::OutOfOrderChat);
        }

        self.last_timestamp_epoch_millis = body.epoch_millis;
        let unpacked = PlayerChatMessage {
            link,
            signature: Some(signature),
            signed_body: body,
            unsigned_content: None,
            filter_mask: FilterMask::pass_through(),
        };
        if !unpacked.verify(&self.signature_validator) {
            self.set_chain_broken();
            return Err(SignedMessageChainDecodeError::InvalidSignature);
        }

        self.next_link = link.advance();
        Ok(unpacked)
    }

    pub fn set_chain_broken(&mut self) {
        self.next_link = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsignedDecoderModel {
    profile_id: Uuid,
    enforces_secure_chat: bool,
}

impl UnsignedDecoderModel {
    pub fn unpack(
        &self,
        _signature: Option<MessageSignature>,
        body: SignedMessageBody,
    ) -> Result<PlayerChatMessage, SignedMessageChainDecodeError> {
        if self.enforces_secure_chat {
            Err(SignedMessageChainDecodeError::MissingProfileKey)
        } else {
            Ok(PlayerChatMessage::unsigned(self.profile_id, body.content))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsignedSigner;

impl SignedMessageSigner for UnsignedSigner {
    fn sign(&self, _payload: &[u8]) -> Result<MessageSignature, String> {
        Err("unsigned encoder does not sign".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageValidatorModel<V, E> {
    validator: V,
    expired: E,
    last_message: Option<PlayerChatMessage>,
    is_chain_valid: bool,
}

impl<V, E> SignedMessageValidatorModel<V, E>
where
    V: MessageSignatureValidator,
    E: ProfileKeyExpiry,
{
    pub fn new(validator: V, expired: E) -> Self {
        Self {
            validator,
            expired,
            last_message: None,
            is_chain_valid: true,
        }
    }

    pub fn accept_unsigned(message: &PlayerChatMessage) -> PlayerChatMessage {
        message.remove_signature()
    }

    pub fn reject_all(_message: &PlayerChatMessage) -> Option<PlayerChatMessage> {
        None
    }

    pub fn update_and_validate(
        &mut self,
        message: PlayerChatMessage,
    ) -> Option<PlayerChatMessage> {
        self.is_chain_valid = self.is_chain_valid && self.validate(&message);
        if !self.is_chain_valid {
            return None;
        }
        self.last_message = Some(message.clone());
        Some(message)
    }

    pub fn is_chain_valid(&self) -> bool {
        self.is_chain_valid
    }

    pub fn last_message(&self) -> Option<&PlayerChatMessage> {
        self.last_message.as_ref()
    }

    fn validate(&self, message: &PlayerChatMessage) -> bool {
        !self.expired.has_expired()
            && message.verify(&self.validator)
            && self.validate_chain(message)
    }

    fn validate_chain(&self, message: &PlayerChatMessage) -> bool {
        if Some(message) == self.last_message.as_ref() {
            true
        } else if let Some(last_message) = &self.last_message {
            message.link.is_descendant_of(&last_message.link)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIGNED_MESSAGE_CHAIN_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/chat/SignedMessageChain.java");
    const SIGNED_MESSAGE_VALIDATOR_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/chat/SignedMessageValidator.java");

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordingSigner;

    impl SignedMessageSigner for RecordingSigner {
        fn sign(&self, payload: &[u8]) -> Result<MessageSignature, String> {
            let mut bytes = vec![0; MessageSignature::BYTES];
            bytes[..payload.len().min(MessageSignature::BYTES)]
                .copy_from_slice(&payload[..payload.len().min(MessageSignature::BYTES)]);
            MessageSignature::new(bytes)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ExactValidator {
        payload: Vec<u8>,
    }

    impl MessageSignatureValidator for ExactValidator {
        fn validate(&self, payload: &[u8], signature: &MessageSignature) -> bool {
            payload == self.payload && signature.bytes().starts_with(payload)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Expired(bool);

    impl ProfileKeyExpiry for Expired {
        fn has_expired(&self) -> bool {
            self.0
        }
    }

    fn uuid(byte: u8) -> Uuid {
        Uuid([byte; 16])
    }

    fn body(content: &str, epoch_millis: i64) -> SignedMessageBody {
        SignedMessageBody::unsigned_at(content, epoch_millis)
    }

    #[test]
    fn signed_message_chain_java_surface_is_tracked() {
        for sentinel in [
            "private @Nullable SignedMessageLink nextLink;",
            "private Instant lastTimeStamp = Instant.EPOCH;",
            "this.nextLink = SignedMessageLink.root(profileId, sessionId);",
            "this.nextLink = link.advance();",
            "return new MessageSignature(signer.sign(output -> PlayerChatMessage.updateSignature(output, link, body)));",
            "throw new SignedMessageChain.DecodeException(SignedMessageChain.DecodeException.MISSING_PROFILE_KEY);",
            "if (body.timeStamp().isBefore(SignedMessageChain.this.lastTimeStamp))",
            "SignedMessageChain.this.lastTimeStamp = body.timeStamp();",
            "new PlayerChatMessage(link, signature, body, null, FilterMask.PASS_THROUGH);",
            "if (!unpacked.verify(signatureValidator))",
            "SignedMessageChain.this.nextLink = link.advance();",
            "SignedMessageChain.Encoder UNSIGNED = body -> null;",
        ] {
            assert!(
                SIGNED_MESSAGE_CHAIN_JAVA.contains(sentinel),
                "missing SignedMessageChain.java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "SignedMessageValidator ACCEPT_UNSIGNED = PlayerChatMessage::removeSignature;",
            "SignedMessageValidator REJECT_ALL = message ->",
            "private @Nullable PlayerChatMessage lastMessage;",
            "private boolean isChainValid = true;",
            "if (message.equals(this.lastMessage))",
            "!message.link().isDescendantOf(this.lastMessage.link())",
            "this.isChainValid = this.isChainValid && this.validate(message);",
            "this.lastMessage = message;",
        ] {
            assert!(
                SIGNED_MESSAGE_VALIDATOR_JAVA.contains(sentinel),
                "missing SignedMessageValidator.java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn encoder_signs_current_link_payload_then_advances_next_link() {
        let profile = uuid(1);
        let session = uuid(2);
        let chain = SignedMessageChainModel::new(profile, session);
        let mut encoder = chain.encoder(RecordingSigner);
        let first_body = body("hello", 10_000);
        let first_link = SignedMessageLink::root(profile, session);

        let signature = encoder.pack(&first_body).unwrap().unwrap();
        assert_eq!(
            signature.bytes()[..player_chat_message_signature_payload(&first_link, &first_body).len()],
            player_chat_message_signature_payload(&first_link, &first_body)
        );
        assert_eq!(encoder.next_link(), first_link.advance());

        let second_body = body("again", 11_000);
        let second_link = first_link.advance().unwrap();
        let signature = encoder.pack(&second_body).unwrap().unwrap();
        assert_eq!(
            signature.bytes()[..player_chat_message_signature_payload(&second_link, &second_body).len()],
            player_chat_message_signature_payload(&second_link, &second_body)
        );
        assert_eq!(encoder.next_link(), second_link.advance());
    }

    #[test]
    fn decoder_rejects_missing_expired_out_of_order_and_bad_signature_like_java() {
        let profile = uuid(3);
        let session = uuid(4);
        let chain = SignedMessageChainModel::new(profile, session);
        let first_body = body("hello", 20_000);
        let first_link = SignedMessageLink::root(profile, session);
        let payload = player_chat_message_signature_payload(&first_link, &first_body);
        let signature = RecordingSigner.sign(&payload).unwrap();

        let mut missing_decoder = chain.decoder(ExactValidator { payload: payload.clone() }, Expired(false));
        assert_eq!(
            missing_decoder.unpack(None, first_body.clone()),
            Err(SignedMessageChainDecodeError::MissingProfileKey)
        );

        let mut expired_decoder = chain.decoder(ExactValidator { payload: payload.clone() }, Expired(true));
        assert_eq!(
            expired_decoder.unpack(Some(signature.clone()), first_body.clone()),
            Err(SignedMessageChainDecodeError::ExpiredProfileKey)
        );

        let mut decoder = chain.decoder(ExactValidator { payload: payload.clone() }, Expired(false));
        let unpacked = decoder
            .unpack(Some(signature), first_body.clone())
            .expect("valid first message");
        assert_eq!(unpacked.link, first_link);
        assert_eq!(decoder.last_timestamp_epoch_millis(), 20_000);
        assert_eq!(decoder.next_link(), first_link.advance());

        let second_link = first_link.advance().unwrap();
        let older_body = body("older", 19_999);
        let older_payload = player_chat_message_signature_payload(&second_link, &older_body);
        let older_signature = RecordingSigner.sign(&older_payload).unwrap();
        assert_eq!(
            decoder.unpack(Some(older_signature), older_body),
            Err(SignedMessageChainDecodeError::OutOfOrderChat)
        );
        assert_eq!(decoder.next_link(), None);

        let mut bad_decoder = chain.decoder(ExactValidator { payload: Vec::new() }, Expired(false));
        let bad_signature = MessageSignature::new(vec![7; MessageSignature::BYTES]).unwrap();
        assert_eq!(
            bad_decoder.unpack(Some(bad_signature), first_body),
            Err(SignedMessageChainDecodeError::InvalidSignature)
        );
        assert_eq!(bad_decoder.next_link(), None);
    }

    #[test]
    fn unsigned_decoder_and_validator_match_java_fallbacks() {
        let profile = uuid(5);
        let decoded =
            SignedMessageChainDecoderModel::<ExactValidator, Expired>::unsigned(profile, false)
                .unpack(None, body("plain", 30_000))
                .unwrap();
        assert_eq!(decoded, PlayerChatMessage::unsigned(profile, "plain"));

        assert_eq!(
            SignedMessageChainDecoderModel::<ExactValidator, Expired>::unsigned(profile, true)
                .unpack(None, body("plain", 30_000)),
            Err(SignedMessageChainDecodeError::MissingProfileKey)
        );

        let signed_body = body("signed", 40_000);
        let link = SignedMessageLink::root(profile, uuid(6));
        let payload = player_chat_message_signature_payload(&link, &signed_body);
        let signature = RecordingSigner.sign(&payload).unwrap();
        let signed = PlayerChatMessage {
            link,
            signature: Some(signature.clone()),
            signed_body,
            unsigned_content: None,
            filter_mask: FilterMask::pass_through(),
        };
        assert_eq!(
            SignedMessageValidatorModel::<ExactValidator, Expired>::accept_unsigned(&signed),
            signed.remove_signature()
        );
        assert_eq!(
            SignedMessageValidatorModel::<ExactValidator, Expired>::reject_all(&signed),
            None
        );

        let mut validator = SignedMessageValidatorModel::new(ExactValidator { payload }, Expired(false));
        assert_eq!(validator.update_and_validate(signed.clone()), Some(signed.clone()));
        assert_eq!(validator.update_and_validate(signed.clone()), Some(signed));

        let out_of_order = PlayerChatMessage {
            link,
            signature: Some(signature),
            signed_body: body("old", 41_000),
            unsigned_content: None,
            filter_mask: FilterMask::pass_through(),
        };
        assert_eq!(validator.update_and_validate(out_of_order), None);
        assert!(!validator.is_chain_valid());
    }
}
