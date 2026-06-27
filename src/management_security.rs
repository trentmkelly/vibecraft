#![allow(dead_code)]

use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TLS_PASSWORD_ENV: &str = "MINECRAFT_MANAGEMENT_TLS_KEYSTORE_PASSWORD";
pub const TLS_PASSWORD_SYSTEM_PROPERTY: &str = "management.tls.keystore.password";
pub const MANAGEMENT_SECRET_KEY_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
pub const MANAGEMENT_SECRET_KEY_LEN: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagementSecurityConfig {
    pub secret_key: String,
    pub tls_enabled: bool,
    pub tls_keystore: Option<String>,
    pub tls_keystore_password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagementSecurityDecision {
    Accepted,
    Unauthorized,
    DisabledInvalidSecret,
    TlsReady { keystore: String, password: String },
    TlsMisconfigured(&'static str),
}

pub fn is_valid_management_secret(secret_key: &str) -> bool {
    secret_key.len() == MANAGEMENT_SECRET_KEY_LEN
        && secret_key.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

pub fn generate_management_secret_key() -> String {
    let mut key = String::with_capacity(MANAGEMENT_SECRET_KEY_LEN);
    if let Ok(mut random) = File::open("/dev/urandom") {
        fill_secret_key_from_reader(&mut key, &mut random);
    }
    while key.len() < MANAGEMENT_SECRET_KEY_LEN {
        key.push(MANAGEMENT_SECRET_KEY_CHARS[fallback_random_index(key.len())] as char);
    }
    key
}

fn fill_secret_key_from_reader(key: &mut String, reader: &mut impl Read) {
    while key.len() < MANAGEMENT_SECRET_KEY_LEN {
        let mut byte = [0_u8; 1];
        if reader.read_exact(&mut byte).is_err() {
            return;
        }
        let max_unbiased = u8::MAX - (u8::MAX % MANAGEMENT_SECRET_KEY_CHARS.len() as u8);
        if byte[0] < max_unbiased {
            let index = usize::from(byte[0] % MANAGEMENT_SECRET_KEY_CHARS.len() as u8);
            key.push(MANAGEMENT_SECRET_KEY_CHARS[index] as char);
        }
    }
}

fn fallback_random_index(offset: usize) -> usize {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(error) => error.duration().as_nanos(),
    };
    (nanos.wrapping_add(offset as u128) % MANAGEMENT_SECRET_KEY_CHARS.len() as u128) as usize
}

pub fn authenticate_management_secret(
    configured: &str,
    provided: &str,
) -> ManagementSecurityDecision {
    if !is_valid_management_secret(configured) {
        return ManagementSecurityDecision::DisabledInvalidSecret;
    }
    if constant_time_eq(configured.as_bytes(), provided.as_bytes()) {
        ManagementSecurityDecision::Accepted
    } else {
        ManagementSecurityDecision::Unauthorized
    }
}

pub fn tls_startup_decision(
    config: &ManagementSecurityConfig,
    env_password: Option<&str>,
    system_property_password: Option<&str>,
) -> ManagementSecurityDecision {
    if !config.tls_enabled {
        return ManagementSecurityDecision::Accepted;
    }
    let Some(keystore) = config
        .tls_keystore
        .as_ref()
        .filter(|value| !value.is_empty())
    else {
        return ManagementSecurityDecision::TlsMisconfigured(
            "TLS is enabled but keystore is not configured",
        );
    };
    let Some(password) = resolve_tls_password(
        env_password,
        system_property_password,
        config.tls_keystore_password.as_deref(),
    ) else {
        return ManagementSecurityDecision::TlsMisconfigured(
            "TLS is enabled but keystore password is not configured",
        );
    };
    ManagementSecurityDecision::TlsReady {
        keystore: keystore.clone(),
        password: password.to_string(),
    }
}

pub fn resolve_tls_password<'a>(
    env_password: Option<&'a str>,
    system_property_password: Option<&'a str>,
    server_property_password: Option<&'a str>,
) -> Option<&'a str> {
    env_password
        .filter(|value| !value.is_empty())
        .or_else(|| system_property_password.filter(|value| !value.is_empty()))
        .or_else(|| server_property_password.filter(|value| !value.is_empty()))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let max_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();
    for index in 0..max_len {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        diff |= (a ^ b) as usize;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SECRET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCD";

    #[test]
    fn management_secret_requires_exact_vanilla_shape() {
        assert_eq!(VALID_SECRET.len(), MANAGEMENT_SECRET_KEY_LEN);
        assert!(is_valid_management_secret(VALID_SECRET));
        assert!(!is_valid_management_secret(""));
        assert!(!is_valid_management_secret("short"));
        assert!(!is_valid_management_secret(
            "0123456789abcdefghijklmnopqrstuvwxyzABC!"
        ));
    }

    #[test]
    fn generated_management_secret_matches_java_shape() {
        let generated = generate_management_secret_key();
        assert_eq!(generated.len(), MANAGEMENT_SECRET_KEY_LEN);
        assert!(is_valid_management_secret(&generated));
        assert!(generated
            .bytes()
            .all(|byte| MANAGEMENT_SECRET_KEY_CHARS.contains(&byte)));
    }

    #[test]
    fn management_auth_accepts_only_matching_valid_secret() {
        assert_eq!(
            authenticate_management_secret(VALID_SECRET, VALID_SECRET),
            ManagementSecurityDecision::Accepted
        );
        assert_eq!(
            authenticate_management_secret(VALID_SECRET, "nope"),
            ManagementSecurityDecision::Unauthorized
        );
        assert_eq!(
            authenticate_management_secret("bad", "bad"),
            ManagementSecurityDecision::DisabledInvalidSecret
        );
    }

    #[test]
    fn tls_requires_keystore_and_uses_env_system_property_server_property_order() {
        let config = ManagementSecurityConfig {
            secret_key: VALID_SECRET.to_string(),
            tls_enabled: true,
            tls_keystore: Some("management.p12".to_string()),
            tls_keystore_password: Some("server".to_string()),
        };
        assert_eq!(
            tls_startup_decision(&config, Some("env"), Some("system")),
            ManagementSecurityDecision::TlsReady {
                keystore: "management.p12".to_string(),
                password: "env".to_string(),
            }
        );
        assert_eq!(
            tls_startup_decision(&config, None, Some("system")),
            ManagementSecurityDecision::TlsReady {
                keystore: "management.p12".to_string(),
                password: "system".to_string(),
            }
        );
        assert_eq!(
            tls_startup_decision(&config, None, None),
            ManagementSecurityDecision::TlsReady {
                keystore: "management.p12".to_string(),
                password: "server".to_string(),
            }
        );
    }

    #[test]
    fn tls_disabled_accepts_and_tls_enabled_reports_missing_inputs() {
        let disabled = ManagementSecurityConfig {
            secret_key: VALID_SECRET.to_string(),
            tls_enabled: false,
            tls_keystore: None,
            tls_keystore_password: None,
        };
        assert_eq!(
            tls_startup_decision(&disabled, None, None),
            ManagementSecurityDecision::Accepted
        );

        let missing_keystore = ManagementSecurityConfig {
            tls_enabled: true,
            ..disabled.clone()
        };
        assert!(matches!(
            tls_startup_decision(&missing_keystore, None, None),
            ManagementSecurityDecision::TlsMisconfigured(_)
        ));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn security_config_source_matches_java_26_1_2() {
        const SECURITY_CONFIG: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/security/SecurityConfig.java");

        for sentinel in [
            "public record SecurityConfig(String secretKey)",
            "private static final String SECRET_KEY_CHARS = \"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789\";",
            "return secretKey.isEmpty() ? false : secretKey.matches(\"^[a-zA-Z0-9]{40}$\");",
            "SecureRandom random = new SecureRandom();",
            "StringBuilder key = new StringBuilder(40);",
            "for (int i = 0; i < 40; i++)",
            ".charAt(random.nextInt(\"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789\".length()))",
            "return key.toString();",
        ] {
            assert!(
                SECURITY_CONFIG.contains(sentinel),
                "SecurityConfig.java is missing sentinel: {sentinel}"
            );
        }
    }
}
