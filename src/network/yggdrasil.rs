//! HTTPS `hasJoinedServer` client for the Mojang session server.
//!
//! Mirrors authlib's `YggdrasilMinecraftSessionService.hasJoinedServer`: a GET to
//! `<session host>/session/minecraft/hasJoined?username=..&serverId=..[&ip=..]`.
//! `200` carries the profile JSON, `204` means the client never called `joinServer`
//! (Java returns `null`), and transport/server failures raise
//! `AuthenticationUnavailableException`.

use std::time::Duration;

use serde_json::Value;

use crate::player_access::NameAndId;
use crate::player_online_auth::{
    HasJoinedRequest, ProfileProperty, ProfileResult, SessionService, SessionServiceResult,
};

const DEFAULT_SESSION_HOST: &str = "https://sessionserver.mojang.com";
/// authlib `YggdrasilEnvironment` system property overriding the session server host.
const SESSION_HOST_PROPERTY: &str = "minecraft.api.session.host";
/// Environment-variable spelling of [`SESSION_HOST_PROPERTY`] (Rust has no JVM `-D`
/// system properties, so the launch flag `-Dminecraft.api.session.host=<url>` and this
/// variable are both honoured).
const SESSION_HOST_ENV: &str = "MINECRAFT_API_SESSION_HOST";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// [`SessionService`] backed by the live Yggdrasil session server.
pub struct YggdrasilSessionService {
    agent: ureq::Agent,
    base_url: String,
}

impl YggdrasilSessionService {
    /// Session service for the configured session host: authlib's
    /// `minecraft.api.session.host` override when present (see
    /// [`session_host_override`]), otherwise the official Mojang session host.
    pub fn new() -> Self {
        let host = session_host_override(
            std::env::args().skip(1),
            std::env::var(SESSION_HOST_ENV).ok(),
        );
        Self::with_base_url(host.as_deref().unwrap_or(DEFAULT_SESSION_HOST))
    }

    /// Session service for an alternate host (authlib's `minecraft.api.session.host`).
    pub fn with_base_url(base_url: &str) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .http_status_as_error(false)
            .build()
            .into();
        Self {
            agent,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

/// Resolves authlib's `minecraft.api.session.host` override: a `-D<property>=<url>`
/// launch argument wins over the environment variable; empty values are ignored
/// (authlib treats a blank property as unset).
pub fn session_host_override(
    args: impl IntoIterator<Item = String>,
    env_value: Option<String>,
) -> Option<String> {
    let flag = format!("-D{SESSION_HOST_PROPERTY}=");
    args.into_iter()
        .find_map(|arg| arg.strip_prefix(&flag).map(str::to_string))
        .or(env_value)
        .filter(|host| !host.trim().is_empty())
}

impl SessionService for YggdrasilSessionService {
    fn has_joined_server(&mut self, request: HasJoinedRequest) -> SessionServiceResult {
        let url = format!("{}/session/minecraft/hasJoined", self.base_url);
        let mut call = self
            .agent
            .get(&url)
            .query("username", &request.username)
            .query("serverId", &request.server_hash);
        if let Some(address) = &request.address {
            call = call.query("ip", address);
        }
        let mut response = match call.call() {
            Ok(response) => response,
            Err(_) => return SessionServiceResult::AuthenticationUnavailable,
        };
        match response.status().as_u16() {
            200 => {}
            204 => return SessionServiceResult::NotJoined,
            _ => return SessionServiceResult::AuthenticationUnavailable,
        }
        match response.body_mut().read_to_string() {
            Ok(body) => parse_has_joined_response(&body),
            Err(_) => SessionServiceResult::AuthenticationUnavailable,
        }
    }
}

/// Parses the `hasJoined` `200` body (`{"id":"<undashed uuid>","name":..,"properties":[..]}`).
pub fn parse_has_joined_response(body: &str) -> SessionServiceResult {
    let Ok(json) = serde_json::from_str::<Value>(body) else {
        return SessionServiceResult::AuthenticationUnavailable;
    };
    let (Some(id), Some(name)) = (
        json.get("id").and_then(Value::as_str),
        json.get("name").and_then(Value::as_str),
    ) else {
        return SessionServiceResult::NotJoined;
    };
    let Some(uuid) = hyphenate_undashed_uuid(id) else {
        return SessionServiceResult::AuthenticationUnavailable;
    };
    let properties = json
        .get("properties")
        .and_then(Value::as_array)
        .map(|entries| entries.iter().filter_map(parse_property).collect())
        .unwrap_or_default();
    SessionServiceResult::Joined(ProfileResult {
        profile: NameAndId {
            uuid,
            name: name.to_string(),
        },
        properties,
    })
}

fn parse_property(entry: &Value) -> Option<ProfileProperty> {
    Some(ProfileProperty {
        name: entry.get("name")?.as_str()?.to_string(),
        value: entry.get("value")?.as_str()?.to_string(),
        signature: entry
            .get("signature")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// Converts a 32-hex-digit UUID (authlib `UndashedUuid`) to the canonical dashed form.
fn hyphenate_undashed_uuid(id: &str) -> Option<String> {
    let id = id.replace('-', "").to_ascii_lowercase();
    if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!(
        "{}-{}-{}-{}-{}",
        &id[0..8],
        &id[8..12],
        &id[12..16],
        &id[16..20],
        &id[20..32]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_profile_with_signed_texture_property() {
        let body = r#"{"id":"069a79f444e94726a5befca90e38aaf5","name":"Notch",
            "properties":[{"name":"textures","value":"abc","signature":"sig"},
                          {"name":"other","value":"v"}]}"#;
        let SessionServiceResult::Joined(result) = parse_has_joined_response(body) else {
            panic!("expected joined");
        };
        assert_eq!(result.profile.name, "Notch");
        assert_eq!(result.profile.uuid, "069a79f4-44e9-4726-a5be-fca90e38aaf5");
        assert_eq!(result.properties.len(), 2);
        assert_eq!(result.properties[0].signature.as_deref(), Some("sig"));
        assert_eq!(result.properties[1].signature, None);
    }

    #[test]
    fn session_host_override_prefers_launch_flag_then_env_and_ignores_blank() {
        let args = |list: &[&str]| list.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
        assert_eq!(session_host_override(args(&[]), None), None);
        assert_eq!(
            session_host_override(args(&["nogui"]), Some("http://env".to_string())),
            Some("http://env".to_string())
        );
        assert_eq!(
            session_host_override(
                args(&["-Dminecraft.api.session.host=http://flag"]),
                Some("http://env".to_string())
            ),
            Some("http://flag".to_string())
        );
        assert_eq!(
            session_host_override(args(&["-Dminecraft.api.session.host="]), None),
            None
        );
    }

    #[test]
    fn malformed_bodies_map_to_unavailable_or_not_joined() {
        assert_eq!(
            parse_has_joined_response("not json"),
            SessionServiceResult::AuthenticationUnavailable
        );
        assert_eq!(
            parse_has_joined_response("{}"),
            SessionServiceResult::NotJoined
        );
        assert_eq!(
            parse_has_joined_response(r#"{"id":"zz","name":"A"}"#),
            SessionServiceResult::AuthenticationUnavailable
        );
    }
}
