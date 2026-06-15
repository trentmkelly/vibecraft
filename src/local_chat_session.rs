//! Mirrors net.minecraft.network.chat.LocalChatSession.java and
//! net.minecraft.network.chat.RemoteChatSession.java.
//!
//! The real server uses `ProfileKeyPair` and a JVM `Signer` to create a
//! `SignedMessageChain.Encoder`. VibeCraft keeps the cryptographic primitive as
//! a testable descriptor here; `SignedMessageChain.java` remains tracked by its
//! own checklist row.

use crate::network::codec::{read_uuid, write_uuid, Uuid};
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::player_access::NameAndId;
use std::io::{self, Read, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileKeyPairModel {
    private_key: Vec<u8>,
    public_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePublicKeyData {
    pub expires_at: SystemTime,
    pub encoded_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChatSessionData {
    pub session_id: [u8; 16],
    pub profile_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalChatSessionModel {
    session_id: Uuid,
    key_pair: ProfileKeyPairModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChatSessionModel {
    session_id: Uuid,
    profile_public_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageValidatorModel {
    signature_validator_kind: &'static str,
    expires_after: Duration,
    public_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageDecoderModel {
    profile_id: Uuid,
    session_id: Uuid,
    profile_public_key: ProfilePublicKeyData,
    next_link_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteChatSessionValidationError {
    InvalidProfileKeySignature,
}

pub trait RemoteProfileKeySignatureValidator {
    fn validate(&self, payload: &[u8], signature: &[u8]) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageEncoderModel {
    profile_id: Uuid,
    session_id: Uuid,
    signer_algorithm: &'static str,
    private_key: Vec<u8>,
    next_link_index: i32,
}

impl ProfileKeyPairModel {
    pub fn new(private_key: Vec<u8>, public_key: ProfilePublicKeyData) -> Self {
        Self {
            private_key,
            public_key,
        }
    }

    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }

    pub fn public_key(&self) -> &ProfilePublicKeyData {
        &self.public_key
    }
}

impl LocalChatSessionModel {
    pub fn new(session_id: Uuid, key_pair: ProfileKeyPairModel) -> Self {
        Self {
            session_id,
            key_pair,
        }
    }

    pub fn create_with_session_id(session_id: Uuid, key_pair: ProfileKeyPairModel) -> Self {
        Self::new(session_id, key_pair)
    }

    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn key_pair(&self) -> &ProfileKeyPairModel {
        &self.key_pair
    }

    pub fn create_message_encoder(&self, profile_id: Uuid) -> SignedMessageEncoderModel {
        SignedMessageEncoderModel {
            profile_id,
            session_id: self.session_id,
            signer_algorithm: "SHA256withRSA",
            private_key: self.key_pair.private_key.clone(),
            next_link_index: 0,
        }
    }

    pub fn as_remote(&self) -> RemoteChatSessionModel {
        RemoteChatSessionModel {
            session_id: self.session_id,
            profile_public_key: self.key_pair.public_key.clone(),
        }
    }
}

impl RemoteChatSessionModel {
    pub fn new(session_id: Uuid, profile_public_key: ProfilePublicKeyData) -> Self {
        Self {
            session_id,
            profile_public_key,
        }
    }

    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn profile_public_key(&self) -> &ProfilePublicKeyData {
        &self.profile_public_key
    }

    pub fn as_data(&self) -> RemoteChatSessionData {
        RemoteChatSessionData {
            session_id: self.session_id.0,
            profile_key: self.profile_public_key.clone(),
        }
    }

    pub fn create_message_validator(&self, grace_period: Duration) -> SignedMessageValidatorModel {
        SignedMessageValidatorModel {
            signature_validator_kind: "ProfilePublicKey.createSignatureValidator",
            expires_after: grace_period,
            public_key: self.profile_public_key.clone(),
        }
    }

    pub fn create_message_decoder(&self, profile_id: Uuid) -> SignedMessageDecoderModel {
        SignedMessageDecoderModel {
            profile_id,
            session_id: self.session_id,
            profile_public_key: self.profile_public_key.clone(),
            next_link_index: 0,
        }
    }

    pub fn has_expired_at(&self, now: SystemTime) -> bool {
        self.profile_public_key.has_expired(Duration::ZERO, now)
    }
}

impl SignedMessageEncoderModel {
    pub fn profile_id(&self) -> Uuid {
        self.profile_id
    }

    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn signer_algorithm(&self) -> &'static str {
        self.signer_algorithm
    }

    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }

    pub fn next_link_index(&self) -> i32 {
        self.next_link_index
    }
}

impl SignedMessageValidatorModel {
    pub fn signature_validator_kind(&self) -> &'static str {
        self.signature_validator_kind
    }

    pub fn expires_after(&self) -> Duration {
        self.expires_after
    }

    pub fn public_key(&self) -> &ProfilePublicKeyData {
        &self.public_key
    }

    pub fn has_expired_at(&self, now: SystemTime) -> bool {
        self.public_key.has_expired(self.expires_after, now)
    }
}

impl SignedMessageDecoderModel {
    pub fn profile_id(&self) -> Uuid {
        self.profile_id
    }

    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn profile_public_key(&self) -> &ProfilePublicKeyData {
        &self.profile_public_key
    }

    pub fn next_link_index(&self) -> i32 {
        self.next_link_index
    }
}

impl ProfilePublicKeyData {
    pub const MAX_PUBLIC_KEY_BYTES: usize = 512;
    pub const MAX_SIGNATURE_BYTES: usize = 4096;

    pub fn has_expired(&self, grace_period: Duration, now: SystemTime) -> bool {
        self.expires_at + grace_period < now
    }

    pub fn expires_at_epoch_millis(&self) -> i64 {
        millis_since_epoch(self.expires_at)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.expires_at_epoch_millis().to_be_bytes())?;
        write_length_prefixed_bytes(writer, &self.encoded_key, Self::MAX_PUBLIC_KEY_BYTES)?;
        write_length_prefixed_bytes(writer, &self.key_signature, Self::MAX_SIGNATURE_BYTES)
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut expires_at = [0; 8];
        reader.read_exact(&mut expires_at)?;
        Ok(Self {
            expires_at: system_time_from_epoch_millis(i64::from_be_bytes(expires_at)),
            encoded_key: read_length_prefixed_bytes(reader, Self::MAX_PUBLIC_KEY_BYTES)?,
            key_signature: read_length_prefixed_bytes(reader, Self::MAX_SIGNATURE_BYTES)?,
        })
    }
}

impl RemoteChatSessionData {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            session_id: read_uuid(reader)?.0,
            profile_key: ProfilePublicKeyData::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_uuid(writer, Uuid(self.session_id))?;
        self.profile_key.write(writer)
    }

    pub fn validate<V: RemoteProfileKeySignatureValidator>(
        &self,
        profile: &NameAndId,
        service_signature_validator: &V,
    ) -> Result<RemoteChatSessionModel, RemoteChatSessionValidationError> {
        if service_signature_validator.validate(
            &profile_public_key_signed_payload(profile, &self.profile_key),
            &self.profile_key.key_signature,
        ) {
            Ok(RemoteChatSessionModel::new(
                Uuid(self.session_id),
                self.profile_key.clone(),
            ))
        } else {
            Err(RemoteChatSessionValidationError::InvalidProfileKeySignature)
        }
    }
}

fn read_length_prefixed_bytes<R: Read>(reader: &mut R, max_length: usize) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_length {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "byte array length exceeds vanilla limit",
        ));
    }
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_length_prefixed_bytes<W: Write>(
    writer: &mut W,
    bytes: &[u8],
    max_length: usize,
) -> io::Result<()> {
    if bytes.len() > max_length {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "byte array length exceeds vanilla limit",
        ));
    }
    write_var_i32(writer, bytes.len() as i32)?;
    writer.write_all(bytes)
}

fn millis_since_epoch(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_else(|err| -(err.duration().as_millis() as i64))
}

fn system_time_from_epoch_millis(millis: i64) -> SystemTime {
    if millis >= 0 {
        UNIX_EPOCH + Duration::from_millis(millis as u64)
    } else {
        UNIX_EPOCH - Duration::from_millis(millis.unsigned_abs())
    }
}

fn profile_public_key_signed_payload(profile: &NameAndId, key: &ProfilePublicKeyData) -> Vec<u8> {
    let uuid = parse_uuid_bytes(&profile.uuid).unwrap_or([0; 16]);
    let mut payload = Vec::with_capacity(24 + key.encoded_key.len());
    payload.extend_from_slice(&uuid);
    payload.extend_from_slice(&key.expires_at_epoch_millis().to_be_bytes());
    payload.extend_from_slice(&key.encoded_key);
    payload
}

fn parse_uuid_bytes(uuid: &str) -> Option<[u8; 16]> {
    let hex = uuid.replace('-', "");
    if hex.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    const LOCAL_CHAT_SESSION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/chat/LocalChatSession.java");
    const REMOTE_CHAT_SESSION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/chat/RemoteChatSession.java");
    const SIGNED_MESSAGE_CHAIN_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/chat/SignedMessageChain.java");

    fn key_data() -> ProfilePublicKeyData {
        ProfilePublicKeyData {
            expires_at: UNIX_EPOCH + Duration::from_secs(123),
            encoded_key: vec![1, 2, 3],
            key_signature: vec![4, 5, 6],
        }
    }

    fn key_pair() -> ProfileKeyPairModel {
        ProfileKeyPairModel::new(vec![9, 8, 7], key_data())
    }

    struct ExactValidator {
        expected_payload: Vec<u8>,
        expected_signature: Vec<u8>,
    }

    impl RemoteProfileKeySignatureValidator for ExactValidator {
        fn validate(&self, payload: &[u8], signature: &[u8]) -> bool {
            payload == self.expected_payload && signature == self.expected_signature
        }
    }

    fn profile() -> NameAndId {
        NameAndId {
            uuid: "00112233-4455-6677-8899-aabbccddeeff".to_string(),
            name: "Alex".to_string(),
        }
    }


    #[test]
    fn local_chat_session_java_source_contract_is_tracked() {
        for sentinel in [
            "public record LocalChatSession(UUID sessionId, ProfileKeyPair keyPair)",
            "public static LocalChatSession create(final ProfileKeyPair keyPair)",
            "return new LocalChatSession(UUID.randomUUID(), keyPair);",
            "public SignedMessageChain.Encoder createMessageEncoder(final UUID profileId)",
            "return new SignedMessageChain(profileId, this.sessionId).encoder(Signer.from(this.keyPair.privateKey(), \"SHA256withRSA\"));",
            "public RemoteChatSession asRemote()",
            "return new RemoteChatSession(this.sessionId, this.keyPair.publicKey());",
        ] {
            assert!(
                LOCAL_CHAT_SESSION_JAVA.contains(sentinel),
                "missing LocalChatSession Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record RemoteChatSession(UUID sessionId, ProfilePublicKey profilePublicKey)",
            "public SignedMessageValidator createMessageValidator(final Duration gracePeriod)",
            "public SignedMessageChain.Decoder createMessageDecoder(final UUID profileId)",
            "public RemoteChatSession.Data asData()",
            "return new RemoteChatSession.Data(this.sessionId, this.profilePublicKey.data());",
            "public boolean hasExpired()",
            "public record Data(UUID sessionId, ProfilePublicKey.Data profilePublicKey)",
            "return new RemoteChatSession.Data(input.readUUID(), new ProfilePublicKey.Data(input));",
            "output.writeUUID(data.sessionId);",
            "return new RemoteChatSession(this.sessionId, ProfilePublicKey.createValidated",
        ] {
            assert!(
                REMOTE_CHAT_SESSION_JAVA.contains(sentinel),
                "missing RemoteChatSession Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public SignedMessageChain(final UUID profileId, final UUID sessionId)",
            "this.nextLink = SignedMessageLink.root(profileId, sessionId);",
            "public SignedMessageChain.Encoder encoder(final Signer signer)",
            "return new MessageSignature(signer.sign(output -> PlayerChatMessage.updateSignature(output, link, body)));",
        ] {
            assert!(
                SIGNED_MESSAGE_CHAIN_JAVA.contains(sentinel),
                "missing SignedMessageChain Java sentinel used by LocalChatSession: {sentinel}"
            );
        }
    }

    #[test]
    fn local_session_encoder_uses_profile_session_and_rsa_signer_like_java() {
        let profile_id = Uuid([1; 16]);
        let session_id = Uuid([2; 16]);
        let session = LocalChatSessionModel::create_with_session_id(session_id, key_pair());
        let encoder = session.create_message_encoder(profile_id);

        assert_eq!(session.session_id(), session_id);
        assert_eq!(encoder.profile_id(), profile_id);
        assert_eq!(encoder.session_id(), session_id);
        assert_eq!(encoder.signer_algorithm(), "SHA256withRSA");
        assert_eq!(encoder.private_key(), &[9, 8, 7]);
        assert_eq!(encoder.next_link_index(), 0);
    }

    #[test]
    fn local_session_as_remote_preserves_session_and_public_key_data() {
        let session_id = Uuid([3; 16]);
        let public_key = key_data();
        let session = LocalChatSessionModel::new(
            session_id,
            ProfileKeyPairModel::new(vec![1, 1, 1], public_key.clone()),
        );
        let remote = session.as_remote();

        assert_eq!(remote.session_id(), session_id);
        assert_eq!(remote.profile_public_key(), &public_key);
        assert_eq!(
            remote.as_data(),
            RemoteChatSessionData {
                session_id: session_id.0,
                profile_key: public_key,
            }
        );
    }

    #[test]
    fn remote_session_validator_decoder_expiry_and_data_match_java() {
        let session_id = Uuid([4; 16]);
        let profile_id = Uuid([5; 16]);
        let public_key = key_data();
        let remote = RemoteChatSessionModel::new(session_id, public_key.clone());

        let validator = remote.create_message_validator(Duration::from_secs(8));
        assert_eq!(
            validator.signature_validator_kind(),
            "ProfilePublicKey.createSignatureValidator"
        );
        assert_eq!(validator.expires_after(), Duration::from_secs(8));
        assert_eq!(validator.public_key(), &public_key);
        assert!(!validator.has_expired_at(public_key.expires_at + Duration::from_secs(8)));
        assert!(validator.has_expired_at(public_key.expires_at + Duration::from_secs(9)));

        let decoder = remote.create_message_decoder(profile_id);
        assert_eq!(decoder.profile_id(), profile_id);
        assert_eq!(decoder.session_id(), session_id);
        assert_eq!(decoder.profile_public_key(), &public_key);
        assert_eq!(decoder.next_link_index(), 0);

        assert_eq!(
            remote.as_data(),
            RemoteChatSessionData {
                session_id: session_id.0,
                profile_key: public_key.clone(),
            }
        );
        assert!(!remote.has_expired_at(public_key.expires_at));
        assert!(remote.has_expired_at(public_key.expires_at + Duration::from_millis(1)));
    }

    #[test]
    fn remote_session_data_reads_writes_and_validates_like_java() {
        let session_id = Uuid([
            0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65,
            0x43, 0x21,
        ]);
        let profile_key = ProfilePublicKeyData {
            expires_at: UNIX_EPOCH + Duration::from_millis(1_717_171_717_171),
            encoded_key: vec![0x30, 0x82, 0x01, 0x0a],
            key_signature: vec![0xaa, 0xbb, 0xcc],
        };
        let data = RemoteChatSessionData {
            session_id: session_id.0,
            profile_key: profile_key.clone(),
        };

        let mut expected = Vec::new();
        write_uuid(&mut expected, session_id).unwrap_or_else(|err| panic!("{err}"));
        expected.extend_from_slice(&1_717_171_717_171_i64.to_be_bytes());
        write_var_i32(&mut expected, profile_key.encoded_key.len() as i32)
            .unwrap_or_else(|err| panic!("{err}"));
        expected.extend_from_slice(&profile_key.encoded_key);
        write_var_i32(&mut expected, profile_key.key_signature.len() as i32)
            .unwrap_or_else(|err| panic!("{err}"));
        expected.extend_from_slice(&profile_key.key_signature);

        let mut encoded = Vec::new();
        data.write(&mut encoded)
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(encoded, expected);
        assert_eq!(
            RemoteChatSessionData::read(&mut encoded.as_slice())
                .unwrap_or_else(|err| panic!("{err}")),
            data
        );

        let validator = ExactValidator {
            expected_payload: profile_public_key_signed_payload(&profile(), &profile_key),
            expected_signature: profile_key.key_signature.clone(),
        };
        assert_eq!(
            data.validate(&profile(), &validator)
                .unwrap_or_else(|err| panic!("{err:?}")),
            RemoteChatSessionModel::new(session_id, profile_key)
        );

        let bad_validator = ExactValidator {
            expected_payload: vec![0],
            expected_signature: vec![0],
        };
        assert_eq!(
            data.validate(&profile(), &bad_validator),
            Err(RemoteChatSessionValidationError::InvalidProfileKeySignature)
        );
    }

    #[test]
    fn remote_session_data_enforces_profile_key_wire_limits() {
        let data = RemoteChatSessionData {
            session_id: [0; 16],
            profile_key: ProfilePublicKeyData {
                expires_at: UNIX_EPOCH,
                encoded_key: vec![0; ProfilePublicKeyData::MAX_PUBLIC_KEY_BYTES + 1],
                key_signature: Vec::new(),
            },
        };
        assert!(data.write(&mut Vec::new()).is_err());

        let data = RemoteChatSessionData {
            session_id: [0; 16],
            profile_key: ProfilePublicKeyData {
                expires_at: UNIX_EPOCH,
                encoded_key: Vec::new(),
                key_signature: vec![0; ProfilePublicKeyData::MAX_SIGNATURE_BYTES + 1],
            },
        };
        assert!(data.write(&mut Vec::new()).is_err());

        let mut payload = Vec::new();
        write_uuid(&mut payload, Uuid([0; 16])).unwrap_or_else(|err| panic!("{err}"));
        payload.extend_from_slice(&0_i64.to_be_bytes());
        write_var_i32(
            &mut payload,
            (ProfilePublicKeyData::MAX_PUBLIC_KEY_BYTES + 1) as i32,
        )
        .unwrap_or_else(|err| panic!("{err}"));
        assert!(RemoteChatSessionData::read(&mut payload.as_slice()).is_err());
    }
}
