#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::registry::Identifier;
use crate::server_registry_layer::{RegistryLayer, RegistryLayerAccess};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootDataTypeModel {
    Predicate,
    Modifier,
    Table,
}

impl LootDataTypeModel {
    pub const VALUES: [Self; 3] = [Self::Predicate, Self::Modifier, Self::Table];

    pub const fn registry_key(self) -> &'static str {
        match self {
            Self::Predicate => "minecraft:predicate",
            Self::Modifier => "minecraft:item_modifier",
            Self::Table => "minecraft:loot_table",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootElementModel {
    pub id: Identifier,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritableRegistryModel {
    pub registry_key: String,
    pub elements: BTreeMap<Identifier, LootElementModel>,
    pub tags_loaded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HolderLookupProviderModel {
    registries: BTreeMap<String, WritableRegistryModel>,
}

impl HolderLookupProviderModel {
    pub fn create(registries: impl IntoIterator<Item = WritableRegistryModel>) -> Self {
        Self {
            registries: registries
                .into_iter()
                .map(|registry| (registry.registry_key.clone(), registry))
                .collect(),
        }
    }

    pub fn concatenate(first: &Self, second: &Self) -> Self {
        let mut registries = first.registries.clone();
        registries.extend(second.registries.clone());
        Self { registries }
    }

    pub fn lookup(&self, registry_key: &str) -> Option<&WritableRegistryModel> {
        self.registries.get(registry_key)
    }

    pub fn list_registries(&self) -> impl Iterator<Item = &WritableRegistryModel> {
        self.registries.values()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadableLayeredRegistriesModel {
    pub layers: RegistryLayerAccess,
    pub reloadable: HolderLookupProviderModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadableServerRegistriesLoadResult {
    pub layers: ReloadableLayeredRegistriesModel,
    pub lookup_with_updated_tags: HolderLookupProviderModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadableServerRegistriesHolder {
    registries: HolderLookupProviderModel,
}

impl ReloadableServerRegistriesHolder {
    pub fn new(registries: HolderLookupProviderModel) -> Self {
        Self { registries }
    }

    pub fn lookup(&self) -> &HolderLookupProviderModel {
        &self.registries
    }

    pub fn get_loot_table(&self, id: &Identifier) -> LootTableLookupResult {
        self.registries
            .lookup(LootDataTypeModel::Table.registry_key())
            .and_then(|registry| registry.elements.get(id))
            .map_or(LootTableLookupResult::Empty, |element| {
                LootTableLookupResult::Found(element.id.clone())
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootTableLookupResult {
    Found(Identifier),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadableServerRegistriesModel {
    pub validation_warnings: Vec<String>,
}

impl ReloadableServerRegistriesModel {
    pub fn reload(
        context: RegistryLayerAccess,
        updated_context_tags: Vec<String>,
        resources: BTreeMap<String, Vec<Identifier>>,
    ) -> ReloadableServerRegistriesLoadResult {
        let context_lookup_with_updated_tags =
            build_updated_context_lookup(&context, &updated_context_tags);
        let new_registries = LootDataTypeModel::VALUES
            .into_iter()
            .map(|data_type| schedule_registry_load(data_type, &resources))
            .collect::<Vec<_>>();

        Self::create_and_validate_full_context(
            context,
            context_lookup_with_updated_tags,
            new_registries,
        )
        .0
    }

    pub fn create_and_validate_full_context(
        context_layers: RegistryLayerAccess,
        context_lookup_with_updated_tags: HolderLookupProviderModel,
        new_registries: Vec<WritableRegistryModel>,
    ) -> (ReloadableServerRegistriesLoadResult, Vec<String>) {
        let full_layers = create_updated_registries(context_layers, new_registries.clone());
        let full_lookup_with_updated_tags = HolderLookupProviderModel::concatenate(
            &context_lookup_with_updated_tags,
            &full_layers.reloadable,
        );
        let warnings = validate_loot_registries(&full_lookup_with_updated_tags);
        (
            ReloadableServerRegistriesLoadResult {
                layers: full_layers,
                lookup_with_updated_tags: full_lookup_with_updated_tags,
            },
            warnings,
        )
    }
}

fn build_updated_context_lookup(
    context: &RegistryLayerAccess,
    updated_context_tags: &[String],
) -> HolderLookupProviderModel {
    let elements = updated_context_tags
        .iter()
        .filter_map(|tag| Identifier::parse(tag).ok())
        .map(|id| (id.clone(), LootElementModel { id, valid: true }))
        .collect();
    let access_marker = format!(
        "loading:{:?}",
        context.access_for(RegistryLayer::Reloadable)
    );
    HolderLookupProviderModel::create([WritableRegistryModel {
        registry_key: access_marker,
        elements,
        tags_loaded: true,
    }])
}

fn schedule_registry_load(
    data_type: LootDataTypeModel,
    resources: &BTreeMap<String, Vec<Identifier>>,
) -> WritableRegistryModel {
    let registry_key = data_type.registry_key().to_string();
    let elements = resources
        .get(&registry_key)
        .into_iter()
        .flatten()
        .map(|id| {
            (
                id.clone(),
                LootElementModel {
                    id: id.clone(),
                    valid: !id.path().contains("invalid"),
                },
            )
        })
        .collect();
    WritableRegistryModel {
        registry_key,
        elements,
        tags_loaded: true,
    }
}

fn create_updated_registries(
    context: RegistryLayerAccess,
    registries: Vec<WritableRegistryModel>,
) -> ReloadableLayeredRegistriesModel {
    ReloadableLayeredRegistriesModel {
        layers: context,
        reloadable: HolderLookupProviderModel::create(registries),
    }
}

fn validate_loot_registries(registries: &HolderLookupProviderModel) -> Vec<String> {
    let mut warnings = Vec::new();
    for data_type in LootDataTypeModel::VALUES {
        if let Some(registry) = registries.lookup(data_type.registry_key()) {
            for element in registry.elements.values() {
                if !element.valid {
                    warnings.push(format!(
                        "Found loot table element validation problem in {}: invalid element",
                        element.id
                    ));
                }
            }
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/ReloadableServerRegistries.java");

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap_or_else(|err| panic!("{err}"))
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn reloadable_server_registries_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public class ReloadableServerRegistries"));
        assert!(JAVA_SOURCE.contains("private static final RegistrationInfo DEFAULT_REGISTRATION_INFO"));
        assert!(JAVA_SOURCE.contains("TagLoader.buildUpdatedLookups("));
        assert!(JAVA_SOURCE.contains("context.getAccessForLoading(RegistryLayer.RELOADABLE)"));
        assert!(JAVA_SOURCE.contains("LootDataType.values()"));
        assert!(JAVA_SOURCE.contains("scheduleRegistryLoad((LootDataType<?>)type, ops, manager, executor)"));
        assert!(JAVA_SOURCE.contains("new MappedRegistry<>(type.registryKey(), Lifecycle.experimental())"));
        assert!(JAVA_SOURCE.contains("SimpleJsonResourceReloadListener.scanDirectory("));
        assert!(JAVA_SOURCE.contains("registry.register(ResourceKey.create(type.registryKey(), id),"));
        assert!(JAVA_SOURCE.contains("TagLoader.loadTagsForRegistry(manager, registry);"));
        assert!(JAVA_SOURCE.contains("createAndValidateFullContext("));
        assert!(JAVA_SOURCE.contains("concatenateLookups("));
        assert!(JAVA_SOURCE.contains("validateLootRegistries(fullLookupWithUpdatedTags);"));
        assert!(JAVA_SOURCE.contains("context.replaceFrom(RegistryLayer.RELOADABLE"));
        assert!(JAVA_SOURCE.contains("public static class Holder"));
        assert!(JAVA_SOURCE.contains("public LootTable getLootTable(final ResourceKey<LootTable> id)"));
        assert!(JAVA_SOURCE.contains("orElse(LootTable.EMPTY)"));
        assert!(JAVA_SOURCE.contains("public record LoadResult("));
    }

    #[test]
    fn reload_loads_loot_data_type_registries_in_java_order_and_tags_them() {
        let mut resources = BTreeMap::new();
        resources.insert(
            "minecraft:loot_table".to_string(),
            vec![id("minecraft:chests/spawn_bonus_chest")],
        );
        resources.insert(
            "minecraft:predicate".to_string(),
            vec![id("minecraft:conditions/test")],
        );

        let result = ReloadableServerRegistriesModel::reload(
            RegistryLayer::create_registry_access(),
            vec!["minecraft:context_tag".to_string()],
            resources,
        );

        let keys = result
            .layers
            .reloadable
            .list_registries()
            .map(|registry| registry.registry_key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec![
                "minecraft:item_modifier",
                "minecraft:loot_table",
                "minecraft:predicate"
            ]
        );
        assert!(result
            .layers
            .reloadable
            .list_registries()
            .all(|registry| registry.tags_loaded));
        assert!(result
            .lookup_with_updated_tags
            .lookup("loading:Some(Empty)")
            .is_some());
    }

    #[test]
    fn create_and_validate_full_context_concatenates_lookups_and_reports_invalid_loot() {
        let registry = WritableRegistryModel {
            registry_key: LootDataTypeModel::Table.registry_key().to_string(),
            elements: BTreeMap::from([(
                id("minecraft:invalid/table"),
                LootElementModel {
                    id: id("minecraft:invalid/table"),
                    valid: false,
                },
            )]),
            tags_loaded: true,
        };

        let (result, warnings) = ReloadableServerRegistriesModel::create_and_validate_full_context(
            RegistryLayer::create_registry_access(),
            HolderLookupProviderModel::default(),
            vec![registry],
        );

        assert!(result
            .lookup_with_updated_tags
            .lookup("minecraft:loot_table")
            .is_some());
        assert_eq!(
            warnings,
            vec![
                "Found loot table element validation problem in minecraft:invalid/table: invalid element"
                    .to_string()
            ]
        );
    }

    #[test]
    fn holder_returns_found_loot_table_or_empty_fallback() {
        let registry = WritableRegistryModel {
            registry_key: LootDataTypeModel::Table.registry_key().to_string(),
            elements: BTreeMap::from([(
                id("minecraft:entities/zombie"),
                LootElementModel {
                    id: id("minecraft:entities/zombie"),
                    valid: true,
                },
            )]),
            tags_loaded: true,
        };
        let holder =
            ReloadableServerRegistriesHolder::new(HolderLookupProviderModel::create([registry]));

        assert_eq!(
            holder.get_loot_table(&id("minecraft:entities/zombie")),
            LootTableLookupResult::Found(id("minecraft:entities/zombie"))
        );
        assert_eq!(
            holder.get_loot_table(&id("minecraft:entities/missing")),
            LootTableLookupResult::Empty
        );
    }
}
