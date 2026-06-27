#![allow(dead_code)]

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const TLS_PASSWORD_ENV: &str = "MINECRAFT_MANAGEMENT_TLS_KEYSTORE_PASSWORD";
pub const TLS_PASSWORD_SYSTEM_PROPERTY: &str = "management.tls.keystore.password";
pub const TLS_KEYSTORE_TYPE: &str = "PKCS12";
pub const TLS_KEYSTORE_MISSING_MESSAGE: &str = "TLS is enabled but keystore is not configured";
pub const TLS_KEYSTORE_NOT_FILE_PREFIX: &str =
    "Supplied keystore is not a file or does not exist: ";
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonRpcSslContextPlan {
    LoadPkcs12 {
        keystore_path: String,
        password: String,
        key_manager_algorithm: &'static str,
        trust_manager_algorithm: &'static str,
    },
    Error(String),
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
    let password = resolve_tls_password(
        env_password,
        system_property_password,
        config.tls_keystore_password.as_deref(),
    );
    ManagementSecurityDecision::TlsReady {
        keystore: keystore.clone(),
        password: password.map_or("", |password| password).to_string(),
    }
}

pub fn resolve_tls_password<'a>(
    env_password: Option<&'a str>,
    system_property_password: Option<&'a str>,
    server_property_password: Option<&'a str>,
) -> Option<&'a str> {
    env_password
        .or(system_property_password)
        .or(server_property_password)
}

pub fn create_jsonrpc_ssl_context_plan(
    keystore_path: &str,
    keystore_password_from_server_properties: &str,
    env_password: Option<&str>,
    system_property_password: Option<&str>,
) -> JsonRpcSslContextPlan {
    if keystore_path.is_empty() {
        return JsonRpcSslContextPlan::Error(TLS_KEYSTORE_MISSING_MESSAGE.to_string());
    }

    let path = Path::new(keystore_path);
    if !path.exists() || !path.is_file() {
        return JsonRpcSslContextPlan::Error(format!(
            "{TLS_KEYSTORE_NOT_FILE_PREFIX}'{keystore_path}'"
        ));
    }

    JsonRpcSslContextPlan::LoadPkcs12 {
        keystore_path: keystore_path.to_string(),
        password: get_keystore_password(
            keystore_password_from_server_properties,
            env_password,
            system_property_password,
        )
        .to_string(),
        key_manager_algorithm: "default",
        trust_manager_algorithm: "default",
    }
}

pub fn get_keystore_password<'a>(
    keystore_password_from_server_properties: &'a str,
    env_password: Option<&'a str>,
    system_property_password: Option<&'a str>,
) -> &'a str {
    resolve_tls_password(
        env_password,
        system_property_password,
        Some(keystore_password_from_server_properties),
    )
    .map_or("", |password| password)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let max_len = left.len().max(right.len());
    let mut diff = left.len() ^ right.len();
    for index in 0..max_len {
        let a = match left.get(index) {
            Some(byte) => *byte,
            None => 0,
        };
        let b = match right.get(index) {
            Some(byte) => *byte,
            None => 0,
        };
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

        let no_configured_password = ManagementSecurityConfig {
            tls_keystore_password: None,
            ..config
        };
        assert_eq!(
            tls_startup_decision(&no_configured_password, Some(""), Some("system")),
            ManagementSecurityDecision::TlsReady {
                keystore: "management.p12".to_string(),
                password: String::new(),
            }
        );
        assert_eq!(
            tls_startup_decision(&no_configured_password, None, None),
            ManagementSecurityDecision::TlsReady {
                keystore: "management.p12".to_string(),
                password: String::new(),
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
    fn jsonrpc_ssl_context_provider_validates_path_and_uses_java_password_precedence() {
        assert_eq!(
            get_keystore_password("server", Some(""), Some("system")),
            ""
        );
        assert_eq!(
            get_keystore_password("server", None, Some("system")),
            "system"
        );
        assert_eq!(get_keystore_password("server", None, None), "server");

        assert_eq!(
            create_jsonrpc_ssl_context_plan("", "server", None, None),
            JsonRpcSslContextPlan::Error(TLS_KEYSTORE_MISSING_MESSAGE.to_string())
        );
        assert_eq!(
            create_jsonrpc_ssl_context_plan("/definitely/not/a/keystore.p12", "server", None, None),
            JsonRpcSslContextPlan::Error(
                "Supplied keystore is not a file or does not exist: '/definitely/not/a/keystore.p12'"
                    .to_string()
            )
        );

        let path = temp_keystore_path();
        let file_result = File::create(&path);
        let _file = match file_result {
            Ok(file) => file,
            Err(error) => panic!("failed to create temp keystore marker: {error}"),
        };
        let path_string = path.to_string_lossy().into_owned();
        assert_eq!(
            create_jsonrpc_ssl_context_plan(&path_string, "server", None, Some("system")),
            JsonRpcSslContextPlan::LoadPkcs12 {
                keystore_path: path_string.clone(),
                password: "system".to_string(),
                key_manager_algorithm: "default",
                trust_manager_algorithm: "default",
            }
        );
        let _ = std::fs::remove_file(path);
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

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn jsonrpc_ssl_context_provider_source_matches_java_26_1_2() {
        const SSL_CONTEXT_PROVIDER: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/security/JsonRpcSslContextProvider.java"
        );

        for sentinel in [
            "private static final String PASSWORD_ENV_VARIABLE_KEY = \"MINECRAFT_MANAGEMENT_TLS_KEYSTORE_PASSWORD\";",
            "private static final String PASSWORD_SYSTEM_PROPERTY_KEY = \"management.tls.keystore.password\";",
            "public static SslContext createFrom(final String keystorePath, final String keystorePasswordFromServerProperties) throws Exception",
            "if (keystorePath.isEmpty())",
            "throw new IllegalArgumentException(\"TLS is enabled but keystore is not configured\");",
            "if (file.exists() && file.isFile())",
            "String keystorePassword = getKeystorePassword(keystorePasswordFromServerProperties);",
            "return loadKeystoreFromPath(file, keystorePassword);",
            "throw new IllegalArgumentException(\"Supplied keystore is not a file or does not exist: '\" + keystorePath + \"'\");",
            "String keystorePassword = System.getenv().get(\"MINECRAFT_MANAGEMENT_TLS_KEYSTORE_PASSWORD\");",
            "if (keystorePassword != null)",
            "String systemPropertyKeystorePassword = System.getProperty(\"management.tls.keystore.password\", null);",
            "return systemPropertyKeystorePassword != null ? systemPropertyKeystorePassword : keystorePasswordFromServerProperties;",
            "KeyStore keyStore = KeyStore.getInstance(\"PKCS12\");",
            "keyStore.load(keystoreStream, password.toCharArray());",
            "KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm())",
            "TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm())",
            "SslContextBuilder.forServer(keyManagerFactory).trustManager(trustManagerFactory).build();",
            "To use TLS for the management server, please follow these steps:",
            "management-server-tls-enabled",
            "management-server-tls-keystore",
            "management-server-tls-keystore-password",
        ] {
            assert!(
                SSL_CONTEXT_PROVIDER.contains(sentinel),
                "JsonRpcSslContextProvider.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn temp_keystore_path() -> std::path::PathBuf {
        let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(error) => error.duration().as_nanos(),
        };
        std::env::temp_dir().join(format!(
            "vibecraft-jsonrpc-keystore-{}-{nanos}.p12",
            std::process::id()
        ))
    }
}
