#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::registry::Identifier;
use crate::resource_file_to_id_converter::FileToIdConverter;
use crate::resource_registry_load_task::{
    RegistryLoadPendingRegistration, RegistryLoadRegistrationInfo, RegistryLoadResource,
    RegistryLoadTaskData, RegistryLoadTaskLifecycle, RegistryLoadTaskModel,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KnownPackModel {
    pub id: &'static str,
    pub vanilla: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManagerRegistryResource {
    pub contents: String,
    pub source_pack_id: String,
    pub known_pack: Option<KnownPackModel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceManagerModel {
    resources: BTreeMap<Identifier, ResourceManagerRegistryResource>,
    tags: BTreeMap<Identifier, Vec<Identifier>>,
}

impl ResourceManagerModel {
    pub fn with_resource(
        mut self,
        id: &str,
        contents: &str,
        source_pack_id: &str,
        known_pack: Option<KnownPackModel>,
    ) -> Result<Self, String> {
        self.resources.insert(
            Identifier::parse(id)?,
            ResourceManagerRegistryResource {
                contents: contents.to_string(),
                source_pack_id: source_pack_id.to_string(),
                known_pack,
            },
        );
        Ok(self)
    }

    pub fn with_tag(mut self, id: &str, values: Vec<&str>) -> Result<Self, String> {
        let values = values
            .into_iter()
            .map(Identifier::parse)
            .collect::<Result<Vec<_>, _>>()?;
        self.tags.insert(Identifier::parse(id)?, values);
        Ok(self)
    }

    fn list_matching_resources(
        &self,
        converter: &FileToIdConverter,
    ) -> Vec<(&Identifier, &ResourceManagerRegistryResource)> {
        converter.matching_resources(self.resources.iter())
    }
}

pub fn resource_manager_registration_info(
    known_pack: Option<KnownPackModel>,
) -> RegistryLoadRegistrationInfo {
    RegistryLoadRegistrationInfo {
        source: known_pack.as_ref().map(|pack| pack.id.to_string()),
        lifecycle: if known_pack.is_some() {
            RegistryLoadTaskLifecycle::Stable
        } else {
            RegistryLoadTaskLifecycle::Experimental
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManagerRegistryLoadOutcome {
    pub registered_elements: Vec<(Identifier, String, RegistryLoadRegistrationInfo)>,
    pub loading_errors: BTreeMap<String, String>,
    pub registered_tags: BTreeMap<Identifier, Vec<Identifier>>,
    pub operations: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManagerRegistryLoadTaskModel {
    task: RegistryLoadTaskModel,
    resource_manager: ResourceManagerModel,
}

impl ResourceManagerRegistryLoadTaskModel {
    pub fn new(data: RegistryLoadTaskData, resource_manager: ResourceManagerModel) -> Self {
        Self {
            task: RegistryLoadTaskModel::new(data, RegistryLoadTaskLifecycle::Stable),
            resource_manager,
        }
    }

    pub fn load(mut self) -> ResourceManagerRegistryLoadOutcome {
        let mut operations = vec![
            "FileToIdConverter.registry(this.registryKey())",
            "CompletableFuture.supplyAsync(lister.listMatchingResources)",
            "RegistryOps.create(JsonOps.INSTANCE, context)",
            "ParallelMapTransform.schedule",
        ];
        let converter = FileToIdConverter::registry_key_id(self.task.registry_key());
        let mut pending = Vec::new();
        for (resource_id, resource) in self.resource_manager.list_matching_resources(&converter) {
            let element_id = match converter.file_to_id(resource_id) {
                Ok(id) => id,
                Err(error) => {
                    pending.push(RegistryLoadPendingRegistration {
                        key: resource_id.clone(),
                        value: Err(error),
                        registration_info: resource_manager_registration_info(
                            resource.known_pack.clone(),
                        ),
                    });
                    continue;
                }
            };
            let load_resource = RegistryLoadResource {
                contents: resource.contents.clone(),
                source_pack_id: resource.source_pack_id.clone(),
            };
            let mut registration = RegistryLoadPendingRegistration::load_from_resource(
                self.task.element_codec(),
                &element_id,
                &load_resource,
            );
            registration.registration_info =
                resource_manager_registration_info(resource.known_pack.clone());
            pending.push(registration);
        }

        operations.push("registerElements(sorted by resource id)");
        self.task.register_elements(pending);
        operations.push("TagLoader.ElementLookup.fromGetters");
        let _ = self.task.read_only_registry();
        operations.push("TagLoader.loadTagsForRegistry");
        self.task.register_tags(self.resource_manager.tags.clone());
        operations.push("registerTags");

        let registered_elements = self
            .task
            .read_only_registry()
            .unwrap_or(&[])
            .iter()
            .map(|element| {
                (
                    element.key.clone(),
                    element.value.clone(),
                    element.registration_info.clone(),
                )
            })
            .collect();
        ResourceManagerRegistryLoadOutcome {
            registered_elements,
            loading_errors: self.task.loading_errors().clone(),
            registered_tags: self.task.tags().clone(),
            operations,
        }
    }

}

trait FileToIdConverterRegistryKeyId {
    fn registry_key_id(registry: &Identifier) -> Self;
}

impl FileToIdConverterRegistryKeyId for FileToIdConverter {
    fn registry_key_id(registry: &Identifier) -> Self {
        FileToIdConverter::json(registry.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESOURCE_MANAGER_REGISTRY_LOAD_TASK_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/ResourceManagerRegistryLoadTask.java");

    #[test]
    fn registration_info_cache_uses_stable_lifecycle_for_any_known_pack() {
        assert_java_contains(
            RESOURCE_MANAGER_REGISTRY_LOAD_TASK_JAVA,
            &[
                "private static final Function<Optional<KnownPack>, RegistrationInfo> REGISTRATION_INFO_CACHE = Util.memoize",
                "knownPack.map(KnownPack::isVanilla).map(info -> Lifecycle.stable()).orElse(Lifecycle.experimental())",
                "return new RegistrationInfo(knownPack, lifecycle);",
            ],
        );

        assert_eq!(
            resource_manager_registration_info(Some(KnownPackModel {
                id: "vanilla",
                vanilla: true,
            })),
            RegistryLoadRegistrationInfo {
                source: Some("vanilla".to_string()),
                lifecycle: RegistryLoadTaskLifecycle::Stable,
            }
        );
        assert_eq!(
            resource_manager_registration_info(Some(KnownPackModel {
                id: "custom-known",
                vanilla: false,
            }))
            .lifecycle,
            RegistryLoadTaskLifecycle::Stable
        );
        assert_eq!(
            resource_manager_registration_info(None).lifecycle,
            RegistryLoadTaskLifecycle::Experimental
        );
    }

    #[test]
    fn load_lists_matching_resources_registers_sorted_entries_then_loads_tags() {
        assert_java_contains(
            RESOURCE_MANAGER_REGISTRY_LOAD_TASK_JAVA,
            &[
                "FileToIdConverter lister = FileToIdConverter.registry(this.registryKey());",
                "lister.listMatchingResources(this.resourceManager)",
                "RegistryOps.create(JsonOps.INSTANCE, context)",
                "ParallelMapTransform.schedule",
                "ResourceKey.create(this.registryKey(), lister.fileToId(resourceId))",
                "REGISTRATION_INFO_CACHE.apply(thunk.knownPackInfo())",
                "RegistryLoadTask.PendingRegistration.loadFromResource(this.data.elementCodec(), ops, elementKey, thunk)",
                "loadedEntries.entrySet().stream().sorted(Entry.comparingByKey()).map(Entry::getValue)",
                "TagLoader.ElementLookup.fromGetters",
                "TagLoader.loadTagsForRegistry(this.resourceManager, this.registryKey(), tagElementLookup)",
            ],
        );

        let manager = must(ResourceManagerModel::default().with_resource(
            "minecraft:chat_type/team_msg_command_incoming.json",
            "team",
            "vanilla",
            None,
        ));
        let manager = must(manager.with_resource(
            "minecraft:chat_type/chat.json",
            "chat",
            "vanilla",
            Some(KnownPackModel {
                id: "vanilla",
                vanilla: true,
            }),
        ));
        let manager = must(manager.with_resource(
            "minecraft:tags/chat_type/ignored.json",
            "ignored",
            "vanilla",
            None,
        ));
        let manager = must(manager.with_tag(
            "minecraft:network_safe",
            vec!["minecraft:chat", "minecraft:team_msg_command_incoming"],
        ));
        let task = ResourceManagerRegistryLoadTaskModel::new(
            must(RegistryLoadTaskData::new(
                "minecraft:chat_type",
                "ChatType.DIRECT_CODEC",
            )),
            manager,
        );

        let outcome = task.load();
        assert_eq!(
            outcome.operations,
            vec![
                "FileToIdConverter.registry(this.registryKey())",
                "CompletableFuture.supplyAsync(lister.listMatchingResources)",
                "RegistryOps.create(JsonOps.INSTANCE, context)",
                "ParallelMapTransform.schedule",
                "registerElements(sorted by resource id)",
                "TagLoader.ElementLookup.fromGetters",
                "TagLoader.loadTagsForRegistry",
                "registerTags",
            ]
        );
        assert_eq!(outcome.registered_elements.len(), 2);
        assert_eq!(
            outcome
                .registered_elements
                .iter()
                .map(|(id, _, _)| id.to_string())
                .collect::<Vec<_>>(),
            vec!["minecraft:chat", "minecraft:team_msg_command_incoming"]
        );
        assert_eq!(
            outcome.registered_elements[0].2,
            RegistryLoadRegistrationInfo {
                source: Some("vanilla".to_string()),
                lifecycle: RegistryLoadTaskLifecycle::Stable,
            }
        );
        assert_eq!(
            outcome.registered_elements[1].2.lifecycle,
            RegistryLoadTaskLifecycle::Experimental
        );
        assert_eq!(
            outcome
                .registered_tags
                .get(&must(Identifier::parse("minecraft:network_safe")))
                .cloned(),
            Some(vec![
                must(Identifier::parse("minecraft:chat")),
                must(Identifier::parse("minecraft:team_msg_command_incoming")),
            ])
        );
    }

    #[test]
    fn load_records_resource_parse_errors_as_pending_registration_failures() {
        let manager = must(ResourceManagerModel::default().with_resource(
            "minecraft:chat_type/chat.json",
            "codec-error",
            "vanilla",
            None,
        ));
        let task = ResourceManagerRegistryLoadTaskModel::new(
            must(RegistryLoadTaskData::new(
                "minecraft:chat_type",
                "ChatType.DIRECT_CODEC",
            )),
            manager,
        );
        let outcome = task.load();
        assert!(outcome.registered_elements.is_empty());
        assert_eq!(
            outcome.loading_errors,
            BTreeMap::from([(
                "ResourceKey[minecraft:chat_type / minecraft:chat]".to_string(),
                "Failed to parse minecraft:chat from pack vanilla".to_string(),
            )])
        );
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }

    fn must<T>(result: Result<T, String>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }
}
