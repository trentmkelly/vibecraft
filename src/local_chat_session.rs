//! Mirrors net.minecraft.network.chat.LocalChatSession.java.
//!
//! The real server uses `ProfileKeyPair` and a JVM `Signer` to create a
//! `SignedMessageChain.Encoder`. VibeCraft keeps the cryptographic primitive as
//! a testable descriptor here; `SignedMessageChain.java` remains tracked by its
//! own checklist row.

use crate::network::codec::Uuid;
use std::time::SystemTime;

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
}
