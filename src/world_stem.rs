#![allow(dead_code)]

use crate::server_registry_layer::RegistryLayerAccess;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CloseableResourceManagerModel {
    closed: bool,
}

impl CloseableResourceManagerModel {
    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadableServerResourcesModel {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDataAndGenSettingsModel {
    pub level_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldStem {
    resource_manager: CloseableResourceManagerModel,
    data_pack_resources: ReloadableServerResourcesModel,
    registries: RegistryLayerAccess,
    world_data_and_gen_settings: WorldDataAndGenSettingsModel,
}

impl WorldStem {
    pub fn new(
        resource_manager: CloseableResourceManagerModel,
        data_pack_resources: ReloadableServerResourcesModel,
        registries: RegistryLayerAccess,
        world_data_and_gen_settings: WorldDataAndGenSettingsModel,
    ) -> Self {
        Self {
            resource_manager,
            data_pack_resources,
            registries,
            world_data_and_gen_settings,
        }
    }

    pub fn resource_manager(&self) -> &CloseableResourceManagerModel {
        &self.resource_manager
    }

    pub fn data_pack_resources(&self) -> &ReloadableServerResourcesModel {
        &self.data_pack_resources
    }

    pub fn registries(&self) -> &RegistryLayerAccess {
        &self.registries
    }

    pub fn world_data_and_gen_settings(&self) -> &WorldDataAndGenSettingsModel {
        &self.world_data_and_gen_settings
    }

    pub fn close(&mut self) {
        self.resource_manager.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_registry_layer::RegistryLayer;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/WorldStem.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn world_stem_matches_java_record_shape() {
        assert!(JAVA_SOURCE.contains("public record WorldStem("));
        assert!(JAVA_SOURCE.contains("CloseableResourceManager resourceManager"));
        assert!(JAVA_SOURCE.contains("ReloadableServerResources dataPackResources"));
        assert!(JAVA_SOURCE.contains("LayeredRegistryAccess<RegistryLayer> registries"));
        assert!(JAVA_SOURCE.contains(
            "LevelDataAndDimensions.WorldDataAndGenSettings worldDataAndGenSettings"
        ));
        assert!(JAVA_SOURCE.contains(") implements AutoCloseable"));
        assert!(JAVA_SOURCE.contains("public void close()"));
        assert!(JAVA_SOURCE.contains("this.resourceManager.close();"));
    }

    #[test]
    fn world_stem_accessors_preserve_record_components() {
        let registries = RegistryLayer::create_registry_access();
        let stem = WorldStem::new(
            CloseableResourceManagerModel::default(),
            ReloadableServerResourcesModel {
                id: "datapack-resources".to_string(),
            },
            registries.clone(),
            WorldDataAndGenSettingsModel {
                level_name: "world".to_string(),
            },
        );

        assert!(!stem.resource_manager().is_closed());
        assert_eq!(stem.data_pack_resources().id, "datapack-resources");
        assert_eq!(stem.registries(), &registries);
        assert_eq!(stem.world_data_and_gen_settings().level_name, "world");
    }

    #[test]
    fn world_stem_close_delegates_to_resource_manager_only() {
        let mut stem = WorldStem::new(
            CloseableResourceManagerModel::default(),
            ReloadableServerResourcesModel {
                id: "datapack-resources".to_string(),
            },
            RegistryLayer::create_registry_access(),
            WorldDataAndGenSettingsModel {
                level_name: "world".to_string(),
            },
        );

        stem.close();

        assert!(stem.resource_manager().is_closed());
        assert_eq!(stem.data_pack_resources().id, "datapack-resources");
        assert_eq!(stem.world_data_and_gen_settings().level_name, "world");
    }
}
