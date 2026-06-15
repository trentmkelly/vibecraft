use crate::chat_component::filter_mask::FilterMask;
use crate::chat_component::Component;
use crate::chat_trust::{
    MessageSignature, MessageSignatureValidator, SignedMessageBody, SignedMessageLink,
};
use crate::network::codec::Uuid;

pub const SYSTEM_SENDER: Uuid = Uuid([0; 16]);
pub const MESSAGE_EXPIRES_AFTER_SERVER_MILLIS: i64 = 5 * 60 * 1000;
pub const MESSAGE_EXPIRES_AFTER_CLIENT_MILLIS: i64 = 7 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerChatMessage {
    pub link: SignedMessageLink,
    pub signature: Option<MessageSignature>,
    pub signed_body: SignedMessageBody,
    pub unsigned_content: Option<Component>,
    pub filter_mask: FilterMask,
}

impl PlayerChatMessage {
    pub const MAP_CODEC_FIELDS: [&'static str; 5] = [
        "link",
        "signature",
        "signed_body",
        "unsigned_content",
        "filter_mask",
    ];

    pub fn system(content: impl Into<String>) -> Self {
        Self::unsigned(SYSTEM_SENDER, content)
    }

    pub fn unsigned(sender: Uuid, content: impl Into<String>) -> Self {
        let signed_body = SignedMessageBody::unsigned(content);
        let link = SignedMessageLink::unsigned(sender);
        Self {
            link,
            signature: None,
            signed_body,
            unsigned_content: None,
            filter_mask: FilterMask::pass_through(),
        }
    }

    pub fn unsigned_at(sender: Uuid, content: impl Into<String>, epoch_millis: i64) -> Self {
        let signed_body = SignedMessageBody::unsigned_at(content, epoch_millis);
        let link = SignedMessageLink::unsigned(sender);
        Self {
            link,
            signature: None,
            signed_body,
            unsigned_content: None,
            filter_mask: FilterMask::pass_through(),
        }
    }

    pub fn with_unsigned_content(&self, content: Component) -> Self {
        let unsigned_content = (content != Component::literal(self.signed_content().to_string()))
            .then_some(content);
        Self {
            unsigned_content,
            ..self.clone()
        }
    }

    pub fn remove_unsigned_content(&self) -> Self {
        if self.unsigned_content.is_some() {
            Self {
                unsigned_content: None,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }

    pub fn filter(&self, filter_mask: FilterMask) -> Self {
        if self.filter_mask == filter_mask {
            self.clone()
        } else {
            Self {
                filter_mask,
                ..self.clone()
            }
        }
    }

    pub fn filter_for_recipient(&self, filtered: bool) -> Self {
        if filtered {
            self.filter(self.filter_mask.clone())
        } else {
            self.filter(FilterMask::pass_through())
        }
    }

    pub fn remove_signature(&self) -> Self {
        self.remove_signature_with_body(SignedMessageBody::unsigned(self.signed_content()))
    }

    pub fn remove_signature_with_body(&self, signed_body: SignedMessageBody) -> Self {
        Self {
            link: SignedMessageLink::unsigned(self.sender()),
            signature: None,
            signed_body,
            unsigned_content: self.unsigned_content.clone(),
            filter_mask: self.filter_mask.clone(),
        }
    }

    pub fn verify<V: MessageSignatureValidator>(&self, signature_validator: &V) -> bool {
        self.signature.as_ref().is_some_and(|signature| {
            signature_validator.validate(
                &player_chat_message_signature_payload(&self.link, &self.signed_body),
                signature,
            )
        })
    }

    pub fn signed_content(&self) -> &str {
        &self.signed_body.content
    }

    pub fn decorated_content(&self) -> Component {
        self.unsigned_content
            .clone()
            .unwrap_or_else(|| Component::literal(self.signed_content().to_string()))
    }

    pub fn time_stamp_epoch_millis(&self) -> i64 {
        self.signed_body.epoch_millis
    }

    pub fn salt(&self) -> i64 {
        self.signed_body.salt
    }

    pub fn has_expired_server_at(&self, now_epoch_millis: i64) -> bool {
        now_epoch_millis
            > self
                .time_stamp_epoch_millis()
                .saturating_add(MESSAGE_EXPIRES_AFTER_SERVER_MILLIS)
    }

    pub fn has_expired_client_at(&self, now_epoch_millis: i64) -> bool {
        now_epoch_millis
            > self
                .time_stamp_epoch_millis()
                .saturating_add(MESSAGE_EXPIRES_AFTER_CLIENT_MILLIS)
    }

    pub fn sender(&self) -> Uuid {
        self.link.sender
    }

    pub fn is_system(&self) -> bool {
        self.sender() == SYSTEM_SENDER
    }

    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    pub fn has_signature_from(&self, profile_id: Uuid) -> bool {
        self.has_signature() && self.link.sender == profile_id
    }

    pub fn is_fully_filtered(&self) -> bool {
        self.filter_mask.is_fully_filtered()
    }

    pub fn describe_signed(message: &PlayerChatMessage) -> String {
        let mut last_seen = String::new();
        for signature in &message.signed_body.last_seen.entries {
            last_seen.push_str("     ");
            last_seen.push_str(&MessageSignature::describe(Some(signature)));
            last_seen.push('\n');
        }

        format!(
            "'{}' @ {}\n - From: {}/{}, message #{}\n - Salt: {}\n - Signature: {}\n - Last Seen: [\n{} ]\n",
            message.signed_body.content,
            epoch_millis_to_java_instant_string(message.signed_body.epoch_millis),
            uuid_to_hyphenated(message.link.sender),
            uuid_to_hyphenated(message.link.session_id),
            message.link.index,
            message.signed_body.salt,
            MessageSignature::describe(message.signature.as_ref()),
            last_seen
        )
    }
}

pub fn player_chat_message_signature_payload(
    link: &SignedMessageLink,
    body: &SignedMessageBody,
) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1_i32.to_be_bytes());
    link.update_signature(&mut payload);
    body.update_signature(&mut payload);
    payload
}

pub fn uuid_to_hyphenated(uuid: Uuid) -> String {
    let b = uuid.0;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b[6],
        b[7],
        b[8],
        b[9],
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}

fn epoch_millis_to_java_instant_string(epoch_millis: i64) -> String {
    let seconds = epoch_millis.div_euclid(1000);
    let millis = epoch_millis.rem_euclid(1000);
    let days = seconds.div_euclid(86_400);
    let second_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = second_of_day / 3_600;
    let minute = (second_of_day % 3_600) / 60;
    let second = second_of_day % 60;

    if millis == 0 {
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    } else {
        format!(
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z"
        )
    }
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_trust::LastSeenMessages;

    struct ExactValidator {
        payload: Vec<u8>,
        signature: MessageSignature,
    }

    impl MessageSignatureValidator for ExactValidator {
        fn validate(&self, payload: &[u8], signature: &MessageSignature) -> bool {
            payload == self.payload && signature == &self.signature
        }
    }

    fn uuid(byte: u8) -> Uuid {
        Uuid([byte; 16])
    }

    fn signature(byte: u8) -> MessageSignature {
        MessageSignature(vec![byte; MessageSignature::BYTES])
    }

    fn signed_body(content: &str) -> SignedMessageBody {
        SignedMessageBody {
            content: content.to_string(),
            epoch_seconds: 123,
            epoch_millis: 123_456,
            salt: 77,
            last_seen: LastSeenMessages::empty(),
        }
    }

    fn signed_message() -> PlayerChatMessage {
        let link = SignedMessageLink {
            sender: uuid(1),
            session_id: uuid(2),
            index: 3,
        };
        let body = signed_body("hello");
        let signature = signature(9);
        PlayerChatMessage {
            link,
            signature: Some(signature),
            signed_body: body,
            unsigned_content: None,
            filter_mask: FilterMask::pass_through(),
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn player_chat_message_java_contract_is_tracked() {
        const PLAYER_CHAT_MESSAGE_JAVA: &str =
            vibecraft_java_source!("/net/minecraft/network/chat/PlayerChatMessage.java");

        for sentinel in [
            "SignedMessageLink.CODEC.fieldOf(\"link\").forGetter(PlayerChatMessage::link)",
            "MessageSignature.CODEC.optionalFieldOf(\"signature\")",
            "SignedMessageBody.MAP_CODEC.forGetter(PlayerChatMessage::signedBody)",
            "ComponentSerialization.CODEC",
            "FilterMask.CODEC.optionalFieldOf(\"filter_mask\", FilterMask.PASS_THROUGH)",
            "private static final UUID SYSTEM_SENDER = Util.NIL_UUID;",
            "public static final Duration MESSAGE_EXPIRES_AFTER_SERVER = Duration.ofMinutes(5L);",
            "public static final Duration MESSAGE_EXPIRES_AFTER_CLIENT = MESSAGE_EXPIRES_AFTER_SERVER.plus(Duration.ofMinutes(2L));",
            "SignedMessageBody body = SignedMessageBody.unsigned(content);",
            "SignedMessageLink link = SignedMessageLink.unsigned(sender);",
            "Component unsignedContent = !content.equals(Component.literal(this.signedContent())) ? content : null;",
            "return this.filter(filtered ? this.filterMask : FilterMask.PASS_THROUGH);",
            "output.update(Ints.toByteArray(1));",
            "return this.signature != null && this.signature.verify(signatureValidator, output -> updateSignature(output, this.link, this.signedBody));",
            "return Objects.requireNonNullElseGet(this.unsignedContent, () -> Component.literal(this.signedContent()));",
            "return now.isAfter(this.timeStamp().plus(MESSAGE_EXPIRES_AFTER_SERVER));",
            "return this.hasSignature() && this.link.sender().equals(profileId);",
            "return this.filterMask.isFullyFiltered();",
            "public static String describeSigned(final PlayerChatMessage message)",
        ] {
            assert!(
                PLAYER_CHAT_MESSAGE_JAVA.contains(sentinel),
                "missing PlayerChatMessage sentinel: {sentinel}"
            );
        }
        assert_eq!(
            PlayerChatMessage::MAP_CODEC_FIELDS,
            [
                "link",
                "signature",
                "signed_body",
                "unsigned_content",
                "filter_mask"
            ]
        );
    }

    #[test]
    fn unsigned_and_system_factories_match_java_defaults() {
        let message = PlayerChatMessage::unsigned_at(uuid(4), "hello", 12_345);
        assert_eq!(message.link, SignedMessageLink::unsigned(uuid(4)));
        assert_eq!(message.signature, None);
        assert_eq!(message.signed_content(), "hello");
        assert_eq!(message.signed_body.epoch_millis, 12_345);
        assert_eq!(message.signed_body.epoch_seconds, 12);
        assert_eq!(message.signed_body.salt, 0);
        assert_eq!(message.signed_body.last_seen, LastSeenMessages::empty());
        assert_eq!(message.unsigned_content, None);
        assert_eq!(message.filter_mask, FilterMask::pass_through());
        assert!(!message.is_system());

        let system = PlayerChatMessage::system("notice");
        assert_eq!(system.sender(), SYSTEM_SENDER);
        assert!(system.is_system());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn unsigned_content_filter_and_remove_signature_match_java() {
        let message = signed_message();
        assert_eq!(message.decorated_content(), Component::literal("hello"));
        assert_eq!(
            message.with_unsigned_content(Component::literal("hello")).unsigned_content,
            None
        );
        let decorated = message.with_unsigned_content(Component::literal("* hello"));
        assert_eq!(decorated.decorated_content(), Component::literal("* hello"));
        assert_eq!(
            decorated.remove_unsigned_content().unsigned_content,
            None
        );
        assert_eq!(
            message.remove_unsigned_content(),
            message,
            "Java returns this when unsignedContent is already null"
        );

        let mut partial = FilterMask::partially_filtered(5);
        partial.set_filtered(0);
        assert_eq!(
            message.filter(FilterMask::pass_through()),
            message,
            "same filter mask returns the same value"
        );
        assert_eq!(message.filter(partial.clone()).filter_mask, partial);
        assert_eq!(
            message.filter(partial.clone()).filter_for_recipient(false).filter_mask,
            FilterMask::pass_through()
        );
        assert_eq!(
            message.filter(partial.clone()).filter_for_recipient(true).filter_mask,
            partial
        );

        let replacement_body = SignedMessageBody::unsigned_at("hello", 99_000);
        let unsigned = decorated.remove_signature_with_body(replacement_body.clone());
        assert_eq!(unsigned.link, SignedMessageLink::unsigned(uuid(1)));
        assert_eq!(unsigned.signature, None);
        assert_eq!(unsigned.signed_body, replacement_body);
        assert_eq!(unsigned.unsigned_content, Some(Component::literal("* hello")));
    }

    #[test]
    fn signature_payload_verify_and_signature_queries_match_java() {
        let message = signed_message();
        let payload = player_chat_message_signature_payload(&message.link, &message.signed_body);
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_i32.to_be_bytes());
        message.link.update_signature(&mut expected);
        message.signed_body.update_signature(&mut expected);
        assert_eq!(payload, expected);

        let validator = ExactValidator {
            payload,
            signature: message
                .signature
                .clone()
                .unwrap_or_else(|| panic!("fixture message has a signature")),
        };
        assert!(message.verify(&validator));
        assert!(message.has_signature());
        assert!(message.has_signature_from(uuid(1)));
        assert!(!message.has_signature_from(uuid(9)));
        assert!(!PlayerChatMessage {
            signature: None,
            ..message.clone()
        }
        .verify(&validator));
    }

    #[test]
    fn expiry_filter_and_description_match_java_surface() {
        let mut message = signed_message();
        assert_eq!(message.time_stamp_epoch_millis(), 123_456);
        assert_eq!(message.salt(), 77);
        assert!(!message.has_expired_server_at(123_456 + MESSAGE_EXPIRES_AFTER_SERVER_MILLIS));
        assert!(message.has_expired_server_at(
            123_456 + MESSAGE_EXPIRES_AFTER_SERVER_MILLIS + 1
        ));
        assert!(!message.has_expired_client_at(123_456 + MESSAGE_EXPIRES_AFTER_CLIENT_MILLIS));
        assert!(message.has_expired_client_at(
            123_456 + MESSAGE_EXPIRES_AFTER_CLIENT_MILLIS + 1
        ));
        assert!(!message.is_fully_filtered());
        message.filter_mask = FilterMask::fully_filtered();
        assert!(message.is_fully_filtered());

        let previous = signature(7);
        message.signed_body.last_seen.entries.push(previous.clone());
        let description = PlayerChatMessage::describe_signed(&message);
        assert!(description.contains("'hello' @ 1970-01-01T00:02:03.456Z"));
        assert!(description.contains(
            " - From: 01010101-0101-0101-0101-010101010101/02020202-0202-0202-0202-020202020202, message #3"
        ));
        assert!(description.contains(" - Salt: 77"));
        assert!(description.contains(&format!(
            " - Signature: {}",
            MessageSignature::describe(message.signature.as_ref())
        )));
        assert!(description.contains(&format!(
            "     {}\n",
            MessageSignature::describe(Some(&previous))
        )));
    }
}
