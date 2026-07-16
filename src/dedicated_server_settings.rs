//! Persistent settings wrapper corresponding to `DedicatedServerSettings.java`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::server_properties::ServerProperties;

#[derive(Debug, Clone)]
pub struct DedicatedServerSettings {
    source: PathBuf,
    properties: ServerProperties,
}

impl DedicatedServerSettings {
    pub fn new(source: impl Into<PathBuf>) -> Result<Self, String> {
        let source = source.into();
        let properties = ServerProperties::load_or_default(&source)?;
        Ok(Self { source, properties })
    }

    pub fn properties(&self) -> &ServerProperties {
        &self.properties
    }

    pub fn force_save(&mut self) -> Result<(), String> {
        self.properties.save(&self.source)
    }

    pub fn update<F>(&mut self, mutator: F) -> Result<&mut Self, String>
    where
        F: FnOnce(ServerProperties) -> ServerProperties,
    {
        self.properties = mutator(self.properties.clone());
        self.force_save()?;
        Ok(self)
    }

    pub fn source(&self) -> &Path {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/dedicated/DedicatedServerSettings.java");

    fn path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("vibecraft-dedicated-settings-{name}-{}.properties", std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source_matches_dedicated_server_settings_lifecycle() {
        for fragment in [
            "private final Path source",
            "private DedicatedServerProperties properties",
            "this.properties = DedicatedServerProperties.fromFile(source)",
            "public DedicatedServerProperties getProperties()",
            "public void forceSave()",
            "this.properties.store(this.source)",
            "public DedicatedServerSettings update(final UnaryOperator<DedicatedServerProperties> mutator)",
            "(this.properties = mutator.apply(this.properties)).store(this.source)",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn missing_source_uses_defaults_until_force_save() {
        let source = path("defaults");
        let mut settings = DedicatedServerSettings::new(&source).expect("default settings");
        assert_eq!(settings.properties().motd, "A Minecraft Server");
        assert!(!source.exists());
        settings.force_save().expect("force save");
        assert!(source.is_file());
        let reloaded = ServerProperties::load_or_default(&source).expect("reload settings");
        assert_eq!(reloaded.motd, "A Minecraft Server");
        let _ignored = fs::remove_file(source);
    }

    #[test]
    fn update_replaces_properties_and_persists_immediately() {
        let source = path("update");
        let mut settings = DedicatedServerSettings::new(&source).expect("default settings");
        settings
            .update(|mut properties| {
                properties.set("motd", "A Changed Server");
                properties
            })
            .expect("update settings");
        assert_eq!(settings.properties().motd, "A Changed Server");
        let reloaded = ServerProperties::load_or_default(&source).expect("reload settings");
        assert_eq!(reloaded.motd, "A Changed Server");
        assert_eq!(settings.source(), source.as_path());
        let _ignored = fs::remove_file(source);
    }
}
