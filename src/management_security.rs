#![allow(dead_code)]

pub const TLS_PASSWORD_ENV: &str = "MINECRAFT_MANAGEMENT_TLS_KEYSTORE_PASSWORD";
pub const TLS_PASSWORD_SYSTEM_PROPERTY: &str = "management.tls.keystore.password";

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
    secret_key.len() == 40 && secret_key.bytes().all(|byte| byte.is_ascii_alphanumeric())
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
        assert_eq!(VALID_SECRET.len(), 40);
        assert!(is_valid_management_secret(VALID_SECRET));
        assert!(!is_valid_management_secret(""));
        assert!(!is_valid_management_secret("short"));
        assert!(!is_valid_management_secret(
            "0123456789abcdefghijklmnopqrstuvwxyzABC!"
        ));
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
}
