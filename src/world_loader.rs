#![allow(dead_code)]

use crate::reloadable_server_resources::{
    PendingComponentModel, PendingTagModel, ReloadableServerResources,
};
use crate::server_function_library::ServerFunctionLibrary;
use crate::server_registry_layer::{RegistryLayer, RegistryLayerAccess};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDataConfigurationModel {
    pub enabled_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackRepositoryModel {
    pub selected_packs: Vec<String>,
    pub configured: bool,
}

impl PackRepositoryModel {
    pub fn open_all_selected(&self) -> Vec<String> {
        self.selected_packs.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseableResourceManagerModel {
    pub pack_type: String,
    pub opened_packs: Vec<String>,
    closed: bool,
}

impl CloseableResourceManagerModel {
    pub fn new(opened_packs: Vec<String>) -> Self {
        Self {
            pack_type: "SERVER_DATA".to_string(),
            opened_packs,
            closed: false,
        }
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackConfigModel {
    pub pack_repository: PackRepositoryModel,
    pub initial_data_config: WorldDataConfigurationModel,
    pub safe_mode: bool,
    pub init_mode: bool,
}

impl PackConfigModel {
    pub fn create_resource_manager(
        &self,
    ) -> (WorldDataConfigurationModel, CloseableResourceManagerModel) {
        let mut data_config = self.initial_data_config.clone();
        if self.safe_mode && !data_config.enabled_features.contains(&"safe_mode".to_string()) {
            data_config.enabled_features.push("safe_mode".to_string());
        }
        if self.init_mode && !data_config.enabled_features.contains(&"init_mode".to_string()) {
            data_config.enabled_features.push("init_mode".to_string());
        }
        (
            data_config,
            CloseableResourceManagerModel::new(self.pack_repository.open_all_selected()),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLoaderInitConfigModel {
    pub pack_config: PackConfigModel,
    pub command_selection: String,
    pub function_compilation_permissions: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLoaderDataLoadContextModel {
    pub resource_pack_count: usize,
    pub data_configuration: WorldDataConfigurationModel,
    pub datapack_worldgen_registry_count: usize,
    pub datapack_dimensions_registry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLoaderDataLoadOutputModel<D> {
    pub cookie: D,
    pub final_dimensions_registry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLoaderResultModel<D> {
    pub resources_closed: bool,
    pub managers_reloaded: bool,
    pub registry_layers: Vec<&'static str>,
    pub cookie: D,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldLoaderEvent {
    CreateResourceManager,
    LoadStaticTags,
    LoadWorldgenRegistries,
    LoadDimensionRegistries,
    InvokeWorldDataSupplier,
    ReplaceWorldgenLayer,
    LoadReloadableServerResources,
    UpdateComponentsAndStaticRegistryTags,
    InvokeResultFactory,
    CloseResourcesAfterFailure,
}

#[derive(Debug, Default)]
pub struct WorldLoaderModel {
    events: RefCell<Vec<WorldLoaderEvent>>,
}

impl WorldLoaderModel {
    pub fn events(&self) -> Vec<WorldLoaderEvent> {
        self.events.borrow().clone()
    }

    pub fn load<D>(
        &self,
        config: WorldLoaderInitConfigModel,
        world_data_supplier: impl FnOnce(WorldLoaderDataLoadContextModel) -> Result<WorldLoaderDataLoadOutputModel<D>, String>,
        result_factory: impl FnOnce(
            &CloseableResourceManagerModel,
            &ReloadableServerResources,
            &RegistryLayerAccess,
            D,
        ) -> WorldLoaderResultModel<D>,
    ) -> Result<WorldLoaderResultModel<D>, String> {
        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::CreateResourceManager);
        let (world_data_configuration, mut resources) = config.pack_config.create_resource_manager();
        let initial_layers = RegistryLayer::create_registry_access();

        self.events.borrow_mut().push(WorldLoaderEvent::LoadStaticTags);
        let static_layer_tags = vec![PendingTagModel::new("minecraft:static")];

        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::LoadWorldgenRegistries);
        let worldgen_context_registry_count = 1 + static_layer_tags.len();

        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::LoadDimensionRegistries);
        let dimension_context_registry_count = worldgen_context_registry_count + 1;

        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::InvokeWorldDataSupplier);
        let world_data_and_registries = match world_data_supplier(WorldLoaderDataLoadContextModel {
            resource_pack_count: resources.opened_packs.len(),
            data_configuration: world_data_configuration.clone(),
            datapack_worldgen_registry_count: worldgen_context_registry_count,
            datapack_dimensions_registry_count: dimension_context_registry_count,
        }) {
            Ok(output) => output,
            Err(err) => {
                resources.close();
                self.events
                    .borrow_mut()
                    .push(WorldLoaderEvent::CloseResourcesAfterFailure);
                return Err(err);
            }
        };

        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::ReplaceWorldgenLayer);
        let resources_load_context = initial_layers;
        let _final_dimensions_registry_count = world_data_and_registries.final_dimensions_registry_count;

        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::LoadReloadableServerResources);
        let mut managers = ReloadableServerResources::load_resources(
            resources_load_context.clone(),
            static_layer_tags,
            world_data_configuration.enabled_features.clone(),
            config.command_selection,
            ServerFunctionLibrary::default(),
            vec![PendingComponentModel::new("minecraft:data_components")],
        );

        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::UpdateComponentsAndStaticRegistryTags);
        managers.update_components_and_static_registry_tags();

        self.events
            .borrow_mut()
            .push(WorldLoaderEvent::InvokeResultFactory);
        Ok(result_factory(
            &resources,
            &managers,
            &resources_load_context,
            world_data_and_registries.cookie,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/WorldLoader.java");

    fn init_config() -> WorldLoaderInitConfigModel {
        WorldLoaderInitConfigModel {
            pack_config: PackConfigModel {
                pack_repository: PackRepositoryModel {
                    selected_packs: vec!["vanilla".to_string()],
                    configured: false,
                },
                initial_data_config: WorldDataConfigurationModel {
                    enabled_features: vec!["base".to_string()],
                },
                safe_mode: true,
                init_mode: false,
            },
            command_selection: "dedicated".to_string(),
            function_compilation_permissions: "function".to_string(),
        }
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn world_loader_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public class WorldLoader"));
        assert!(JAVA_SOURCE.contains("public static <D, R> CompletableFuture<R> load("));
        assert!(JAVA_SOURCE.contains("CompletableFuture.supplyAsync(config.packConfig::createResourceManager"));
        assert!(JAVA_SOURCE.contains("RegistryLayer.createRegistryAccess();"));
        assert!(JAVA_SOURCE.contains("TagLoader.loadTagsForExistingRegistries(resources, initialLayers.getLayer(RegistryLayer.STATIC));"));
        assert!(JAVA_SOURCE.contains("initialLayers.getAccessForLoading(RegistryLayer.WORLDGEN);"));
        assert!(JAVA_SOURCE.contains("RegistryDataLoader.load(resources, worldgenContextRegistries, RegistryDataLoader.WORLDGEN_REGISTRIES"));
        assert!(JAVA_SOURCE.contains("RegistryDataLoader.load(resources, dimensionContextRegistries, RegistryDataLoader.DIMENSION_REGISTRIES"));
        assert!(JAVA_SOURCE.contains("worldDataSupplier.get("));
        assert!(JAVA_SOURCE.contains("new WorldLoader.DataLoadContext("));
        assert!(JAVA_SOURCE.contains("initialLayers.replaceFrom("));
        assert!(JAVA_SOURCE.contains("RegistryLayer.WORLDGEN, loadedWorldgenRegistries, worldDataAndRegistries.finalDimensions"));
        assert!(JAVA_SOURCE.contains("ReloadableServerResources.loadResources("));
        assert!(JAVA_SOURCE.contains("if (throwable != null) {\n                                          resources.close();"));
        assert!(JAVA_SOURCE.contains("managers.updateComponentsAndStaticRegistryTags();"));
        assert!(JAVA_SOURCE.contains("resultFactory.create(resources, managers, resourcesLoadContext, worldDataAndRegistries.cookie);"));
        assert!(JAVA_SOURCE.contains("public record DataLoadContext("));
        assert!(JAVA_SOURCE.contains("public record DataLoadOutput<D>(D cookie, RegistryAccess.Frozen finalDimensions)"));
        assert!(JAVA_SOURCE.contains("public record InitConfig("));
        assert!(JAVA_SOURCE.contains("public record PackConfig("));
        assert!(JAVA_SOURCE.contains("public interface ResultFactory<D, R>"));
        assert!(JAVA_SOURCE.contains("public interface WorldDataSupplier<D>"));
    }

    #[test]
    fn pack_config_configures_repository_and_opens_selected_packs() {
        let (data_config, resources) = init_config().pack_config.create_resource_manager();

        assert_eq!(data_config.enabled_features, vec!["base", "safe_mode"]);
        assert_eq!(resources.pack_type, "SERVER_DATA");
        assert_eq!(resources.opened_packs, vec!["vanilla"]);
        assert!(!resources.is_closed());
    }

    #[test]
    fn world_loader_runs_java_load_steps_in_order_and_builds_result() {
        let loader = WorldLoaderModel::default();
        let result = loader
            .load(
                init_config(),
                |context| {
                    assert_eq!(context.resource_pack_count, 1);
                    assert_eq!(context.data_configuration.enabled_features, vec!["base", "safe_mode"]);
                    assert_eq!(context.datapack_worldgen_registry_count, 2);
                    assert_eq!(context.datapack_dimensions_registry_count, 3);
                    Ok(WorldLoaderDataLoadOutputModel {
                        cookie: "cookie".to_string(),
                        final_dimensions_registry_count: 1,
                    })
                },
                |resources, managers, registries, cookie| WorldLoaderResultModel {
                    resources_closed: resources.is_closed(),
                    managers_reloaded: !managers.reload_log().is_empty()
                        && managers.postponed_tags().iter().all(PendingTagModel::is_applied)
                        && managers
                            .new_components()
                            .iter()
                            .all(PendingComponentModel::is_applied),
                    registry_layers: registries
                        .layers()
                        .iter()
                        .map(|(layer, _)| layer.serialized_name())
                        .collect(),
                    cookie,
                },
            )
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            loader.events(),
            vec![
                WorldLoaderEvent::CreateResourceManager,
                WorldLoaderEvent::LoadStaticTags,
                WorldLoaderEvent::LoadWorldgenRegistries,
                WorldLoaderEvent::LoadDimensionRegistries,
                WorldLoaderEvent::InvokeWorldDataSupplier,
                WorldLoaderEvent::ReplaceWorldgenLayer,
                WorldLoaderEvent::LoadReloadableServerResources,
                WorldLoaderEvent::UpdateComponentsAndStaticRegistryTags,
                WorldLoaderEvent::InvokeResultFactory,
            ]
        );
        assert!(!result.resources_closed);
        assert!(result.managers_reloaded);
        assert_eq!(result.registry_layers[0], "STATIC");
        assert_eq!(result.cookie, "cookie");
    }

    #[test]
    fn world_loader_closes_resources_when_data_supplier_fails() {
        let loader = WorldLoaderModel::default();
        let result = loader.load(
            init_config(),
            |_context| Err("failed world data".to_string()),
            |_resources, _managers, _registries, cookie: String| WorldLoaderResultModel {
                resources_closed: false,
                managers_reloaded: false,
                registry_layers: Vec::new(),
                cookie,
            },
        );

        assert_eq!(result, Err("failed world data".to_string()));
        assert!(loader
            .events()
            .contains(&WorldLoaderEvent::CloseResourcesAfterFailure));
    }
}
