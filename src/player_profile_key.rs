#![allow(dead_code)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::player_access::NameAndId;

pub const EXPIRY_GRACE_PERIOD: Duration = Duration::from_secs(8 * 60 * 60);
pub const EXPIRED_PROFILE_PUBLIC_KEY: &str = "multiplayer.disconnect.expired_public_key";
pub const INVALID_PUBLIC_KEY_SIGNATURE: &str =
    "multiplayer.disconnect.invalid_public_key_signature";
pub const MISSING_PROFILE_KEY: &str = "chat.disabled.missingProfileKey";
pub const INVALID_COMMAND_SIGNATURE: &str = "chat.disabled.invalid_signature";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecureProfileConfig {
    pub enforce_secure_profile: bool,
    pub online_mode: bool,
    pub can_validate_profile_keys: bool,
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
pub struct ValidatedRemoteChatSession {
    pub session_id: [u8; 16],
    pub profile_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileKeyDecision {
    Accepted(ValidatedRemoteChatSession),
    IgnoreMissingServicesKey,
    Disconnect(&'static str),
}

pub trait ProfileKeySignatureValidator {
    fn validate(&self, payload: &[u8], signature: &[u8]) -> bool;
}

pub fn enforce_secure_profile(config: SecureProfileConfig) -> bool {
    config.enforce_secure_profile && config.online_mode && config.can_validate_profile_keys
}

pub fn unsigned_chat_key_decision(config: SecureProfileConfig) -> Result<(), &'static str> {
    if enforce_secure_profile(config) {
        Err(MISSING_PROFILE_KEY)
    } else {
        Ok(())
    }
}

pub fn unsigned_signed_command_decision(
    config: SecureProfileConfig,
    has_signable_arguments: bool,
) -> Result<(), &'static str> {
    if enforce_secure_profile(config) && has_signable_arguments {
        Err(INVALID_COMMAND_SIGNATURE)
    } else {
        Ok(())
    }
}

pub fn validate_chat_session_update<V: ProfileKeySignatureValidator>(
    profile: &NameAndId,
    old: Option<&ValidatedRemoteChatSession>,
    new_session: RemoteChatSessionData,
    validator: Option<&V>,
    now: SystemTime,
) -> ProfileKeyDecision {
    if let Some(old) = old {
        if new_session.profile_key.expires_at < old.profile_key.expires_at {
            return ProfileKeyDecision::Disconnect(EXPIRED_PROFILE_PUBLIC_KEY);
        }
    }

    let Some(validator) = validator else {
        return ProfileKeyDecision::IgnoreMissingServicesKey;
    };
    if new_session
        .profile_key
        .has_expired(EXPIRY_GRACE_PERIOD, now)
    {
        return ProfileKeyDecision::Disconnect(EXPIRED_PROFILE_PUBLIC_KEY);
    }

    let payload = signed_payload(profile, &new_session.profile_key);
    if !validator.validate(&payload, &new_session.profile_key.key_signature) {
        return ProfileKeyDecision::Disconnect(INVALID_PUBLIC_KEY_SIGNATURE);
    }

    ProfileKeyDecision::Accepted(ValidatedRemoteChatSession {
        session_id: new_session.session_id,
        profile_key: new_session.profile_key,
    })
}

pub fn signed_payload(profile: &NameAndId, key: &ProfilePublicKeyData) -> Vec<u8> {
    let uuid = parse_uuid_bytes(&profile.uuid).unwrap_or([0; 16]);
    let mut payload = Vec::with_capacity(24 + key.encoded_key.len());
    payload.extend_from_slice(&uuid);
    payload.extend_from_slice(&millis_since_epoch(key.expires_at).to_be_bytes());
    payload.extend_from_slice(&key.encoded_key);
    payload
}

impl ProfilePublicKeyData {
    pub fn has_expired(&self, grace_period: Duration, now: SystemTime) -> bool {
        self.expires_at + grace_period < now
    }
}

fn millis_since_epoch(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_else(|err| -(err.duration().as_millis() as i64))
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

#[cfg(test)]
mod tests {
    use super::*;

    struct ExactValidator {
        expected_payload: Vec<u8>,
        expected_signature: Vec<u8>,
    }

    impl ProfileKeySignatureValidator for ExactValidator {
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

    fn key(expires_at: SystemTime) -> ProfilePublicKeyData {
        ProfilePublicKeyData {
            expires_at,
            encoded_key: vec![1, 2, 3, 4],
            key_signature: vec![9, 8, 7],
        }
    }

    #[test]
    fn enforce_secure_profile_requires_property_online_mode_and_services_keys() {
        assert!(enforce_secure_profile(SecureProfileConfig {
            enforce_secure_profile: true,
            online_mode: true,
            can_validate_profile_keys: true,
        }));
        assert!(!enforce_secure_profile(SecureProfileConfig {
            enforce_secure_profile: true,
            online_mode: false,
            can_validate_profile_keys: true,
        }));
        assert!(!enforce_secure_profile(SecureProfileConfig {
            enforce_secure_profile: true,
            online_mode: true,
            can_validate_profile_keys: false,
        }));
        assert!(!enforce_secure_profile(SecureProfileConfig {
            enforce_secure_profile: false,
            online_mode: true,
            can_validate_profile_keys: true,
        }));
    }

    #[test]
    fn secure_profile_blocks_unsigned_chat_and_signable_unsigned_commands() {
        let enforced = SecureProfileConfig {
            enforce_secure_profile: true,
            online_mode: true,
            can_validate_profile_keys: true,
        };
        assert_eq!(
            unsigned_chat_key_decision(enforced),
            Err(MISSING_PROFILE_KEY)
        );
        assert_eq!(
            unsigned_signed_command_decision(enforced, true),
            Err(INVALID_COMMAND_SIGNATURE)
        );
        assert_eq!(unsigned_signed_command_decision(enforced, false), Ok(()));

        let relaxed = SecureProfileConfig {
            enforce_secure_profile: true,
            online_mode: false,
            can_validate_profile_keys: true,
        };
        assert_eq!(unsigned_chat_key_decision(relaxed), Ok(()));
        assert_eq!(unsigned_signed_command_decision(relaxed, true), Ok(()));
    }

    #[test]
    fn profile_key_signed_payload_uses_uuid_expires_millis_and_encoded_key() {
        let expires_at = UNIX_EPOCH + Duration::from_millis(0x0102_0304_0506_0708);
        let payload = signed_payload(&profile(), &key(expires_at));
        assert_eq!(
            payload,
            vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 1, 2, 3, 4
            ]
        );
    }

    #[test]
    fn valid_chat_session_update_accepts_matching_service_signature() {
        let now = UNIX_EPOCH + Duration::from_secs(100);
        let session = RemoteChatSessionData {
            session_id: [5; 16],
            profile_key: key(now + Duration::from_secs(60)),
        };
        let validator = ExactValidator {
            expected_payload: signed_payload(&profile(), &session.profile_key),
            expected_signature: session.profile_key.key_signature.clone(),
        };

        assert_eq!(
            validate_chat_session_update(&profile(), None, session.clone(), Some(&validator), now),
            ProfileKeyDecision::Accepted(ValidatedRemoteChatSession {
                session_id: session.session_id,
                profile_key: session.profile_key
            })
        );
    }

    #[test]
    fn chat_session_update_disconnects_on_older_expiry_expired_key_or_bad_signature() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000_000);
        let old = ValidatedRemoteChatSession {
            session_id: [1; 16],
            profile_key: key(now + Duration::from_secs(600)),
        };
        let newer_but_already_expired = RemoteChatSessionData {
            session_id: [2; 16],
            profile_key: key(now - EXPIRY_GRACE_PERIOD - Duration::from_secs(1)),
        };
        let bad_validator = ExactValidator {
            expected_payload: vec![0],
            expected_signature: vec![0],
        };

        let older = RemoteChatSessionData {
            session_id: [2; 16],
            profile_key: key(now + Duration::from_secs(1)),
        };
        assert_eq!(
            validate_chat_session_update(&profile(), Some(&old), older, Some(&bad_validator), now),
            ProfileKeyDecision::Disconnect(EXPIRED_PROFILE_PUBLIC_KEY)
        );
        assert_eq!(
            validate_chat_session_update(
                &profile(),
                None,
                newer_but_already_expired,
                Some(&bad_validator),
                now
            ),
            ProfileKeyDecision::Disconnect(EXPIRED_PROFILE_PUBLIC_KEY)
        );

        let bad_signature = RemoteChatSessionData {
            session_id: [3; 16],
            profile_key: key(now + Duration::from_secs(60)),
        };
        assert_eq!(
            validate_chat_session_update(
                &profile(),
                None,
                bad_signature,
                Some(&bad_validator),
                now
            ),
            ProfileKeyDecision::Disconnect(INVALID_PUBLIC_KEY_SIGNATURE)
        );
    }

    #[test]
    fn missing_services_key_ignores_chat_session_update_like_vanilla_warning_path() {
        let now = UNIX_EPOCH + Duration::from_secs(100);
        let session = RemoteChatSessionData {
            session_id: [4; 16],
            profile_key: key(now + Duration::from_secs(60)),
        };
        assert_eq!(
            validate_chat_session_update::<ExactValidator>(&profile(), None, session, None, now),
            ProfileKeyDecision::IgnoreMissingServicesKey
        );
    }
}
