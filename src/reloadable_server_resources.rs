#![allow(dead_code)]

use crate::reloadable_server_registries::{
    HolderLookupProviderModel, ReloadableServerRegistriesHolder, ReloadableServerRegistriesModel,
};
use crate::server_function_library::ServerFunctionLibrary;
use crate::server_registry_layer::RegistryLayerAccess;
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeManagerModel {
    pub loading_context_registry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandsModel {
    pub command_selection: String,
    pub enabled_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerAdvancementManagerComponentModel {
    pub loading_context_registry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTagModel {
    pub id: String,
    applied: bool,
}

impl PendingTagModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            applied: false,
        }
    }

    pub fn apply(&mut self) {
        self.applied = true;
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingComponentModel {
    pub id: String,
    applied: bool,
}

impl PendingComponentModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            applied: false,
        }
    }

    pub fn apply(&mut self) {
        self.applied = true;
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadableServerResources {
    full_registry_holder: ReloadableServerRegistriesHolder,
    commands: CommandsModel,
    recipes: RecipeManagerModel,
    advancements: ServerAdvancementManagerComponentModel,
    function_library: ServerFunctionLibrary,
    postponed_tags: Vec<PendingTagModel>,
    new_components: Vec<PendingComponentModel>,
    listener_order: Vec<ReloadListenerKind>,
    reload_log: RefCell<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadListenerKind {
    Recipes,
    FunctionLibrary,
    Advancements,
}

impl ReloadableServerResources {
    pub fn new(
        full_layers_lookup: HolderLookupProviderModel,
        loading_context: HolderLookupProviderModel,
        enabled_features: Vec<String>,
        command_selection: impl Into<String>,
        postponed_tags: Vec<PendingTagModel>,
        function_library: ServerFunctionLibrary,
        new_components: Vec<PendingComponentModel>,
    ) -> Self {
        let loading_context_registry_count = loading_context.list_registries().count();
        Self {
            full_registry_holder: ReloadableServerRegistriesHolder::new(full_layers_lookup),
            postponed_tags,
            new_components,
            recipes: RecipeManagerModel {
                loading_context_registry_count,
            },
            commands: CommandsModel {
                command_selection: command_selection.into(),
                enabled_features,
            },
            advancements: ServerAdvancementManagerComponentModel {
                loading_context_registry_count,
            },
            function_library,
            listener_order: vec![
                ReloadListenerKind::Recipes,
                ReloadListenerKind::FunctionLibrary,
                ReloadListenerKind::Advancements,
            ],
            reload_log: RefCell::new(Vec::new()),
        }
    }

    pub fn load_resources(
        context_layers: RegistryLayerAccess,
        updated_context_tags: Vec<PendingTagModel>,
        enabled_features: Vec<String>,
        command_selection: impl Into<String>,
        function_library: ServerFunctionLibrary,
        new_components: Vec<PendingComponentModel>,
    ) -> Self {
        let tag_ids = updated_context_tags
            .iter()
            .map(|tag| tag.id.clone())
            .collect::<Vec<_>>();
        let registries =
            ReloadableServerRegistriesModel::reload(context_layers, tag_ids, BTreeMap::new());
        let result = Self::new(
            registries.lookup_with_updated_tags.clone(),
            registries.lookup_with_updated_tags,
            enabled_features,
            command_selection,
            updated_context_tags,
            function_library,
            new_components,
        );
        result.run_reload_listeners();
        result
    }

    fn run_reload_listeners(&self) {
        self.reload_log.borrow_mut().extend(
            self.listener_order
                .iter()
                .map(|listener| format!("{listener:?}")),
        );
    }

    pub fn get_function_library(&self) -> &ServerFunctionLibrary {
        &self.function_library
    }

    pub fn full_registries(&self) -> &ReloadableServerRegistriesHolder {
        &self.full_registry_holder
    }

    pub fn get_recipe_manager(&self) -> &RecipeManagerModel {
        &self.recipes
    }

    pub fn get_commands(&self) -> &CommandsModel {
        &self.commands
    }

    pub fn get_advancements(&self) -> &ServerAdvancementManagerComponentModel {
        &self.advancements
    }

    pub fn listeners(&self) -> &[ReloadListenerKind] {
        &self.listener_order
    }

    pub fn reload_log(&self) -> Vec<String> {
        self.reload_log.borrow().clone()
    }

    pub fn postponed_tags(&self) -> &[PendingTagModel] {
        &self.postponed_tags
    }

    pub fn new_components(&self) -> &[PendingComponentModel] {
        &self.new_components
    }

    pub fn update_components_and_static_registry_tags(&mut self) {
        self.postponed_tags.iter_mut().for_each(PendingTagModel::apply);
        self.new_components
            .iter_mut()
            .for_each(PendingComponentModel::apply);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_registry_layer::RegistryLayer;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/ReloadableServerResources.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn reloadable_server_resources_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public class ReloadableServerResources"));
        assert!(JAVA_SOURCE.contains("private static final CompletableFuture<Unit> DATA_RELOAD_INITIAL_TASK"));
        assert!(JAVA_SOURCE.contains("private final ReloadableServerRegistries.Holder fullRegistryHolder;"));
        assert!(JAVA_SOURCE.contains("private final Commands commands;"));
        assert!(JAVA_SOURCE.contains("private final RecipeManager recipes;"));
        assert!(JAVA_SOURCE.contains("private final ServerAdvancementManager advancements;"));
        assert!(JAVA_SOURCE.contains("private final ServerFunctionLibrary functionLibrary;"));
        assert!(JAVA_SOURCE.contains("private final List<Registry.PendingTags<?>> postponedTags;"));
        assert!(JAVA_SOURCE.contains("private final List<DataComponentInitializers.PendingComponents<?>> newComponents;"));
        assert!(JAVA_SOURCE.contains("this.fullRegistryHolder = new ReloadableServerRegistries.Holder(fullLayers.compositeAccess());"));
        assert!(JAVA_SOURCE.contains("this.recipes = new RecipeManager(loadingContext);"));
        assert!(JAVA_SOURCE.contains("this.commands = new Commands(commandSelection, CommandBuildContext.simple(loadingContext, enabledFeatures));"));
        assert!(JAVA_SOURCE.contains("this.advancements = new ServerAdvancementManager(loadingContext);"));
        assert!(JAVA_SOURCE.contains("this.functionLibrary = new ServerFunctionLibrary(functionCompilationPermissions, this.commands.getDispatcher());"));
        assert!(JAVA_SOURCE.contains("return List.of(this.recipes, this.functionLibrary, this.advancements);"));
        assert!(JAVA_SOURCE.contains("ReloadableServerRegistries.reload(contextLayers, updatedContextTags, resourceManager, backgroundExecutor)"));
        assert!(JAVA_SOURCE.contains("BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(fullRegistries.lookupWithUpdatedTags())"));
        assert!(JAVA_SOURCE.contains("SimpleReloadInstance.create("));
        assert!(JAVA_SOURCE.contains("this.postponedTags.forEach(Registry.PendingTags::apply);"));
        assert!(JAVA_SOURCE.contains("this.newComponents.forEach(DataComponentInitializers.PendingComponents::apply);"));
    }

    #[test]
    fn constructor_wires_accessors_and_listener_order_like_java() {
        let resources = ReloadableServerResources::new(
            HolderLookupProviderModel::default(),
            HolderLookupProviderModel::default(),
            vec!["vanilla".to_string()],
            "dedicated",
            vec![PendingTagModel::new("minecraft:test")],
            ServerFunctionLibrary::default(),
            vec![PendingComponentModel::new("minecraft:component")],
        );

        assert_eq!(resources.get_commands().command_selection, "dedicated");
        assert_eq!(resources.get_commands().enabled_features, ["vanilla"]);
        assert_eq!(resources.get_recipe_manager().loading_context_registry_count, 0);
        assert_eq!(resources.get_advancements().loading_context_registry_count, 0);
        assert!(resources
            .get_function_library()
            .get_available_tags()
            .next()
            .is_none());
        assert_eq!(
            resources.listeners(),
            &[
                ReloadListenerKind::Recipes,
                ReloadListenerKind::FunctionLibrary,
                ReloadListenerKind::Advancements,
            ]
        );
        assert_eq!(resources.postponed_tags().len(), 1);
        assert_eq!(resources.new_components().len(), 1);
    }

    #[test]
    fn load_resources_runs_registry_reload_then_listeners() {
        let resources = ReloadableServerResources::load_resources(
            RegistryLayer::create_registry_access(),
            vec![PendingTagModel::new("minecraft:context_tag")],
            vec!["vanilla".to_string(), "bundle".to_string()],
            "integrated",
            ServerFunctionLibrary::default(),
            vec![PendingComponentModel::new("minecraft:component")],
        );

        assert_eq!(
            resources.reload_log(),
            vec![
                "Recipes".to_string(),
                "FunctionLibrary".to_string(),
                "Advancements".to_string(),
            ]
        );
        assert!(resources
            .full_registries()
            .lookup()
            .lookup("loading:Some(Empty)")
            .is_some());
        assert_eq!(resources.get_commands().command_selection, "integrated");
    }

    #[test]
    fn update_components_and_static_registry_tags_applies_both_lists() {
        let mut resources = ReloadableServerResources::new(
            HolderLookupProviderModel::default(),
            HolderLookupProviderModel::default(),
            Vec::new(),
            "dedicated",
            vec![PendingTagModel::new("minecraft:test")],
            ServerFunctionLibrary::default(),
            vec![PendingComponentModel::new("minecraft:component")],
        );

        resources.update_components_and_static_registry_tags();

        assert!(resources.postponed_tags()[0].is_applied());
        assert!(resources.new_components()[0].is_applied());
    }
}
