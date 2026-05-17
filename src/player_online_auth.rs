#![allow(dead_code)]

use sha1::{Digest, Sha1};

use crate::player_access::NameAndId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasJoinedRequest {
    pub username: String,
    pub server_hash: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileResult {
    pub profile: NameAndId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionServiceResult {
    Joined(ProfileResult),
    NotJoined,
    AuthenticationUnavailable,
}

pub trait SessionService {
    fn has_joined_server(&mut self, request: HasJoinedRequest) -> SessionServiceResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnlineAuthOptions {
    pub online_mode: bool,
    pub singleplayer: bool,
    pub prevent_proxy_connections: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationDecision {
    Authenticated(NameAndId),
    StartOfflineMode(NameAndId),
    Disconnect(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnlineAuthenticator {
    options: OnlineAuthOptions,
}

impl OnlineAuthenticator {
    pub fn new(options: OnlineAuthOptions) -> Self {
        Self { options }
    }

    pub fn authenticate<S: SessionService>(
        &self,
        service: &mut S,
        username: &str,
        server_hash: String,
        remote_address: Option<&str>,
    ) -> AuthenticationDecision {
        if !self.options.online_mode {
            return AuthenticationDecision::StartOfflineMode(NameAndId::create_offline(username));
        }

        let request = HasJoinedRequest {
            username: username.to_string(),
            server_hash,
            address: self
                .options
                .prevent_proxy_connections
                .then(|| remote_address.map(str::to_string))
                .flatten(),
        };
        match service.has_joined_server(request) {
            SessionServiceResult::Joined(result) => {
                AuthenticationDecision::Authenticated(result.profile)
            }
            SessionServiceResult::NotJoined if self.options.singleplayer => {
                AuthenticationDecision::StartOfflineMode(NameAndId::create_offline(username))
            }
            SessionServiceResult::NotJoined => {
                AuthenticationDecision::Disconnect("multiplayer.disconnect.unverified_username")
            }
            SessionServiceResult::AuthenticationUnavailable if self.options.singleplayer => {
                AuthenticationDecision::StartOfflineMode(NameAndId::create_offline(username))
            }
            SessionServiceResult::AuthenticationUnavailable => {
                AuthenticationDecision::Disconnect("multiplayer.disconnect.authservers_down")
            }
        }
    }
}

pub fn minecraft_server_hash(server_id: &str, public_key: &[u8], shared_secret: &[u8]) -> String {
    let mut sha1 = Sha1::new();
    sha1.update(server_id.as_bytes());
    sha1.update(shared_secret);
    sha1.update(public_key);
    format_signed_sha1_digest(sha1.finalize().into())
}

fn format_signed_sha1_digest(mut digest: [u8; 20]) -> String {
    let negative = digest[0] & 0x80 != 0;
    if negative {
        twos_complement_abs(&mut digest);
    }
    let mut hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    while hex.starts_with('0') && hex.len() > 1 {
        hex.remove(0);
    }
    if negative {
        format!("-{hex}")
    } else {
        hex
    }
}

fn twos_complement_abs(bytes: &mut [u8]) {
    for byte in bytes.iter_mut() {
        *byte = !*byte;
    }
    for byte in bytes.iter_mut().rev() {
        let (next, overflow) = byte.overflowing_add(1);
        *byte = next;
        if !overflow {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct RecordingSessionService {
        next: SessionServiceResult,
        requests: Vec<HasJoinedRequest>,
    }

    impl RecordingSessionService {
        fn returning(next: SessionServiceResult) -> Self {
            Self {
                next,
                requests: Vec::new(),
            }
        }
    }

    impl SessionService for RecordingSessionService {
        fn has_joined_server(&mut self, request: HasJoinedRequest) -> SessionServiceResult {
            self.requests.push(request);
            self.next.clone()
        }
    }

    #[test]
    fn offline_mode_skips_session_service_and_uses_offline_uuid() {
        let auth = OnlineAuthenticator::new(OnlineAuthOptions {
            online_mode: false,
            singleplayer: false,
            prevent_proxy_connections: true,
        });
        let mut service = RecordingSessionService::returning(SessionServiceResult::NotJoined);

        assert_eq!(
            auth.authenticate(&mut service, "Steve", "hash".to_string(), Some("127.0.0.1")),
            AuthenticationDecision::StartOfflineMode(NameAndId::create_offline("Steve"))
        );
        assert!(service.requests.is_empty());
    }

    #[test]
    fn online_mode_accepts_session_service_profile_and_forwards_proxy_address_when_enabled() {
        let profile = NameAndId {
            uuid: "8667ba71-b85a-4004-af54-457a9734eed7".to_string(),
            name: "Alex".to_string(),
        };
        let mut service =
            RecordingSessionService::returning(SessionServiceResult::Joined(ProfileResult {
                profile: profile.clone(),
            }));
        let auth = OnlineAuthenticator::new(OnlineAuthOptions {
            online_mode: true,
            singleplayer: false,
            prevent_proxy_connections: true,
        });

        assert_eq!(
            auth.authenticate(
                &mut service,
                "Alex",
                "server-hash".to_string(),
                Some("203.0.113.7")
            ),
            AuthenticationDecision::Authenticated(profile)
        );
        assert_eq!(
            service.requests,
            vec![HasJoinedRequest {
                username: "Alex".to_string(),
                server_hash: "server-hash".to_string(),
                address: Some("203.0.113.7".to_string()),
            }]
        );
    }

    #[test]
    fn prevent_proxy_disabled_sends_no_address_to_session_service() {
        let mut service = RecordingSessionService::returning(SessionServiceResult::NotJoined);
        let auth = OnlineAuthenticator::new(OnlineAuthOptions {
            online_mode: true,
            singleplayer: false,
            prevent_proxy_connections: false,
        });

        assert_eq!(
            auth.authenticate(
                &mut service,
                "Alex",
                "hash".to_string(),
                Some("203.0.113.7")
            ),
            AuthenticationDecision::Disconnect("multiplayer.disconnect.unverified_username")
        );
        assert_eq!(service.requests[0].address, None);
    }

    #[test]
    fn online_mode_uses_vanilla_disconnect_keys_for_failed_or_unavailable_authentication() {
        let auth = OnlineAuthenticator::new(OnlineAuthOptions {
            online_mode: true,
            singleplayer: false,
            prevent_proxy_connections: false,
        });
        let mut invalid = RecordingSessionService::returning(SessionServiceResult::NotJoined);
        assert_eq!(
            auth.authenticate(&mut invalid, "BadName", "hash".to_string(), None),
            AuthenticationDecision::Disconnect("multiplayer.disconnect.unverified_username")
        );

        let mut unavailable =
            RecordingSessionService::returning(SessionServiceResult::AuthenticationUnavailable);
        assert_eq!(
            auth.authenticate(&mut unavailable, "Alex", "hash".to_string(), None),
            AuthenticationDecision::Disconnect("multiplayer.disconnect.authservers_down")
        );
    }

    #[test]
    fn singleplayer_falls_back_to_offline_profile_when_auth_fails_or_is_unavailable() {
        let auth = OnlineAuthenticator::new(OnlineAuthOptions {
            online_mode: true,
            singleplayer: true,
            prevent_proxy_connections: false,
        });
        let mut invalid = RecordingSessionService::returning(SessionServiceResult::NotJoined);
        assert_eq!(
            auth.authenticate(&mut invalid, "Local", "hash".to_string(), None),
            AuthenticationDecision::StartOfflineMode(NameAndId::create_offline("Local"))
        );

        let mut unavailable =
            RecordingSessionService::returning(SessionServiceResult::AuthenticationUnavailable);
        assert_eq!(
            auth.authenticate(&mut unavailable, "Local", "hash".to_string(), None),
            AuthenticationDecision::StartOfflineMode(NameAndId::create_offline("Local"))
        );
    }

    #[test]
    fn minecraft_server_hash_uses_signed_twos_complement_sha1_format() {
        assert_eq!(
            minecraft_server_hash("", b"public-key", b"shared-secret"),
            "22dad2606e21585dce2129957029ad597a754b3b"
        );
        assert_eq!(
            format_signed_sha1_digest([
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff
            ]),
            "-1"
        );
    }
}
