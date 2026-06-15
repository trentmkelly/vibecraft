#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftSessionServiceModel {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfileRepositoryModel {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServicesKeyTypeModel {
    ProfileKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServicesKeyModel {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServicesKeySetModel {
    keys: BTreeMap<ServicesKeyTypeModel, Vec<ServicesKeyModel>>,
}

impl ServicesKeySetModel {
    pub fn with_keys(key_type: ServicesKeyTypeModel, keys: Vec<ServicesKeyModel>) -> Self {
        let mut by_type = BTreeMap::new();
        by_type.insert(key_type, keys);
        Self { keys: by_type }
    }

    pub fn keys(&self, key_type: ServicesKeyTypeModel) -> &[ServicesKeyModel] {
        self.keys.get(&key_type).map_or(&[], Vec::as_slice)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedUserNameToIdResolverModel {
    pub profile_repository: GameProfileRepositoryModel,
    pub cache_file: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileResolverModel {
    Cached {
        session_service: MinecraftSessionServiceModel,
        name_to_id_cache: CachedUserNameToIdResolverModel,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureValidatorModel {
    pub key_type: ServicesKeyTypeModel,
    pub key_ids: Vec<String>,
}

impl SignatureValidatorModel {
    pub fn from(
        key_set: &ServicesKeySetModel,
        key_type: ServicesKeyTypeModel,
    ) -> Option<SignatureValidatorModel> {
        let keys = key_set.keys(key_type.clone());
        if keys.is_empty() {
            return None;
        }

        Some(SignatureValidatorModel {
            key_type,
            key_ids: keys.iter().map(|key| key.id.clone()).collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YggdrasilAuthenticationServiceModel {
    pub session_service: MinecraftSessionServiceModel,
    pub services_key_set: ServicesKeySetModel,
    pub profile_repository: GameProfileRepositoryModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Services {
    pub session_service: MinecraftSessionServiceModel,
    pub services_key_set: ServicesKeySetModel,
    pub profile_repository: GameProfileRepositoryModel,
    pub name_to_id_cache: CachedUserNameToIdResolverModel,
    pub profile_resolver: ProfileResolverModel,
}

impl Services {
    pub const USERID_CACHE_FILE: &'static str = "usercache.json";

    pub fn create(
        service_access: &YggdrasilAuthenticationServiceModel,
        name_cache_dir: &Path,
    ) -> Self {
        let session_service = service_access.session_service.clone();
        let profile_repository = service_access.profile_repository.clone();
        let profile_cache = CachedUserNameToIdResolverModel {
            profile_repository: profile_repository.clone(),
            cache_file: name_cache_dir.join(Self::USERID_CACHE_FILE),
        };
        let profile_resolver = ProfileResolverModel::Cached {
            session_service: session_service.clone(),
            name_to_id_cache: profile_cache.clone(),
        };

        Self {
            session_service,
            services_key_set: service_access.services_key_set.clone(),
            profile_repository,
            name_to_id_cache: profile_cache,
            profile_resolver,
        }
    }

    pub fn profile_key_signature_validator(&self) -> Option<SignatureValidatorModel> {
        SignatureValidatorModel::from(&self.services_key_set, ServicesKeyTypeModel::ProfileKey)
    }

    pub fn can_validate_profile_keys(&self) -> bool {
        !self
            .services_key_set
            .keys(ServicesKeyTypeModel::ProfileKey)
            .is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/Services.java");

    fn auth_service_with_profile_keys(keys: Vec<&str>) -> YggdrasilAuthenticationServiceModel {
        YggdrasilAuthenticationServiceModel {
            session_service: MinecraftSessionServiceModel {
                id: "session".to_string(),
            },
            services_key_set: ServicesKeySetModel::with_keys(
                ServicesKeyTypeModel::ProfileKey,
                keys.into_iter()
                    .map(|id| ServicesKeyModel { id: id.to_string() })
                    .collect(),
            ),
            profile_repository: GameProfileRepositoryModel {
                id: "profiles".to_string(),
            },
        }
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_services_matches_java_record_and_factory_shape() {
        assert!(JAVA_SOURCE.contains("public record Services("));
        assert!(JAVA_SOURCE.contains("MinecraftSessionService sessionService"));
        assert!(JAVA_SOURCE.contains("ServicesKeySet servicesKeySet"));
        assert!(JAVA_SOURCE.contains("GameProfileRepository profileRepository"));
        assert!(JAVA_SOURCE.contains("UserNameToIdResolver nameToIdCache"));
        assert!(JAVA_SOURCE.contains("ProfileResolver profileResolver"));
        assert!(JAVA_SOURCE.contains("private static final String USERID_CACHE_FILE = \"usercache.json\";"));
        assert!(JAVA_SOURCE.contains("serviceAccess.createMinecraftSessionService();"));
        assert!(JAVA_SOURCE.contains("serviceAccess.createProfileRepository();"));
        assert!(JAVA_SOURCE.contains("new CachedUserNameToIdResolver(profileRepository, new File(nameCacheDir, \"usercache.json\"))"));
        assert!(JAVA_SOURCE.contains("new ProfileResolver.Cached(sessionService, profileCache)"));
        assert!(JAVA_SOURCE.contains("return new Services(sessionService, serviceAccess.getServicesKeySet(), profileRepository, profileCache, profileResolver);"));
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_services_matches_java_profile_key_validation_shape() {
        assert!(JAVA_SOURCE.contains("public @Nullable SignatureValidator profileKeySignatureValidator()"));
        assert!(JAVA_SOURCE.contains("SignatureValidator.from(this.servicesKeySet, ServicesKeyType.PROFILE_KEY)"));
        assert!(JAVA_SOURCE.contains("public boolean canValidateProfileKeys()"));
        assert!(JAVA_SOURCE.contains("!this.servicesKeySet.keys(ServicesKeyType.PROFILE_KEY).isEmpty()"));
    }

    #[test]
    fn server_utility_services_create_wires_java_factory_outputs() {
        let auth_service = auth_service_with_profile_keys(vec!["profile-key-a"]);
        let services = Services::create(&auth_service, Path::new("/tmp/cache"));

        assert_eq!(services.session_service.id, "session");
        assert_eq!(services.profile_repository.id, "profiles");
        assert_eq!(services.name_to_id_cache.profile_repository.id, "profiles");
        assert_eq!(
            services.name_to_id_cache.cache_file,
            Path::new("/tmp/cache").join("usercache.json")
        );
        assert_eq!(
            services.profile_resolver,
            ProfileResolverModel::Cached {
                session_service: MinecraftSessionServiceModel {
                    id: "session".to_string()
                },
                name_to_id_cache: services.name_to_id_cache.clone(),
            }
        );
    }

    #[test]
    fn server_utility_services_profile_key_validation_follows_key_set() {
        let without_keys = Services::create(&auth_service_with_profile_keys(Vec::new()), Path::new("."));
        assert!(!without_keys.can_validate_profile_keys());
        assert_eq!(without_keys.profile_key_signature_validator(), None);

        let with_keys = Services::create(
            &auth_service_with_profile_keys(vec!["profile-key-a", "profile-key-b"]),
            Path::new("."),
        );
        assert!(with_keys.can_validate_profile_keys());
        assert_eq!(
            with_keys.profile_key_signature_validator(),
            Some(SignatureValidatorModel {
                key_type: ServicesKeyTypeModel::ProfileKey,
                key_ids: vec!["profile-key-a".to_string(), "profile-key-b".to_string()],
            })
        );
    }
}
