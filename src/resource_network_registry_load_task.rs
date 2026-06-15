#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::registry::{Identifier, Registry, ResourceKey};
use crate::resource_file_to_id_converter::FileToIdConverter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryLoadLifecycle {
    Stable,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationInfoModel {
    pub source: Option<String>,
    pub lifecycle: RegistryLoadLifecycle,
}

pub fn network_registration_info() -> RegistrationInfoModel {
    RegistrationInfoModel {
        source: None,
        lifecycle: RegistryLoadLifecycle::Experimental,
    }
}

#[derive(Debug, Clone)]
pub struct RegistryDataModel {
    pub key: ResourceKey<Registry<String>>,
    pub element_codec: ElementCodecModel,
}

impl RegistryDataModel {
    pub fn new(registry: &str, codec_name: &'static str) -> Result<Self, String> {
        Ok(Self {
            key: ResourceKey::<String>::create_registry_key(Identifier::parse(registry)?),
            element_codec: ElementCodecModel::new(codec_name),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementCodecModel {
    name: &'static str,
}

impl ElementCodecModel {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn parse_network(&self, element: &ResourceKey<String>, tag: &str) -> Result<String, String> {
        if tag == "error" {
            Err(format!(
                "Failed to parse value {tag} for key {} from server",
                element.location()
            ))
        } else {
            Ok(format!("{}:nbt:{tag}", self.name))
        }
    }

    fn parse_json(
        &self,
        element: &ResourceKey<String>,
        resource: &ResourceModel,
    ) -> Result<String, String> {
        if resource.contents == "error" {
            Err(format!(
                "Failed to parse {} from pack {}",
                element.location(),
                resource.source_pack_id
            ))
        } else {
            Ok(format!("{}:json:{}", self.name, resource.contents))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedRegistryEntryModel {
    pub id: Identifier,
    pub data: Option<String>,
}

impl PackedRegistryEntryModel {
    pub fn network(id: &str, data: &str) -> Result<Self, String> {
        Ok(Self {
            id: Identifier::parse(id)?,
            data: Some(data.to_string()),
        })
    }

    pub fn known_data(id: &str) -> Result<Self, String> {
        Ok(Self {
            id: Identifier::parse(id)?,
            data: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPayloadModel {
    pub tags: Vec<NetworkTagModel>,
}

impl NetworkPayloadModel {
    pub fn empty() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn resolve(
        &self,
        registry_key: &ResourceKey<Registry<String>>,
        registered_order: &[ResourceKey<String>],
    ) -> BTreeMap<TagKeyModel, Vec<HolderModel>> {
        let mut tags = BTreeMap::new();
        for tag in &self.tags {
            let values = tag
                .ids
                .iter()
                .filter_map(|id| registered_order.get(*id))
                .map(|key| HolderModel {
                    key: key.location().clone(),
                })
                .collect();
            tags.insert(
                TagKeyModel {
                    registry: registry_key.location().clone(),
                    location: tag.id.clone(),
                },
                values,
            );
        }
        tags
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkTagModel {
    pub id: Identifier,
    pub ids: Vec<usize>,
}

impl NetworkTagModel {
    pub fn new(id: &str, ids: Vec<usize>) -> Result<Self, String> {
        Ok(Self {
            id: Identifier::parse(id)?,
            ids,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkedRegistryDataModel {
    pub elements: Vec<PackedRegistryEntryModel>,
    pub tags: NetworkPayloadModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceModel {
    pub contents: String,
    pub source_pack_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceProviderModel {
    resources: BTreeMap<Identifier, ResourceModel>,
}

impl ResourceProviderModel {
    pub fn with_resource(
        mut self,
        id: &str,
        contents: &str,
        source_pack_id: &str,
    ) -> Result<Self, String> {
        self.resources.insert(
            Identifier::parse(id)?,
            ResourceModel {
                contents: contents.to_string(),
                source_pack_id: source_pack_id.to_string(),
            },
        );
        Ok(self)
    }

    fn get_resource(&self, id: &Identifier) -> Option<&ResourceModel> {
        self.resources.get(id)
    }
}

#[derive(Debug, Clone)]
pub struct PendingRegistrationModel {
    pub key: ResourceKey<String>,
    pub value: Result<String, String>,
    pub registration_info: RegistrationInfoModel,
}

impl PendingRegistrationModel {
    fn load_from_network(
        codec: &ElementCodecModel,
        key: ResourceKey<String>,
        tag: &str,
    ) -> Self {
        Self {
            value: codec.parse_network(&key, tag),
            key,
            registration_info: network_registration_info(),
        }
    }

    fn find_and_load_from_resource(
        codec: &ElementCodecModel,
        key: ResourceKey<String>,
        converter: &FileToIdConverter,
        provider: &ResourceProviderModel,
    ) -> Self {
        let value = match converter.id_to_file(key.location()) {
            Ok(resource_id) => provider
                .get_resource(&resource_id)
                .map(|resource| codec.parse_json(&key, resource))
                .unwrap_or_else(|| {
                    Err(format!(
                        "Failed to find resource {resource_id} for element {}",
                        key.location()
                    ))
                }),
            Err(error) => Err(error),
        };
        Self {
            key,
            value,
            registration_info: network_registration_info(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagKeyModel {
    pub registry: Identifier,
    pub location: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderModel {
    pub key: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRegistryLoadOutcome {
    pub completed_noop: bool,
    pub registered_elements: Vec<(Identifier, String, RegistrationInfoModel)>,
    pub loading_errors: BTreeMap<String, String>,
    pub registered_tags: BTreeMap<TagKeyModel, Vec<HolderModel>>,
    pub operations: Vec<&'static str>,
}

#[derive(Debug, Clone)]
pub struct NetworkRegistryLoadTaskModel {
    pub data: RegistryDataModel,
    pub lifecycle: RegistryLoadLifecycle,
    pub entries: BTreeMap<Identifier, NetworkedRegistryDataModel>,
    pub known_data_source: ResourceProviderModel,
}

impl NetworkRegistryLoadTaskModel {
    pub fn new(data: RegistryDataModel, known_data_source: ResourceProviderModel) -> Self {
        Self {
            data,
            lifecycle: RegistryLoadLifecycle::Stable,
            entries: BTreeMap::new(),
            known_data_source,
        }
    }

    pub fn with_entries(mut self, entries: NetworkedRegistryDataModel) -> Self {
        self.entries.insert(self.data.key.location().clone(), entries);
        self
    }

    pub fn load(&self) -> NetworkRegistryLoadOutcome {
        let Some(registry_entries) = self.entries.get(self.data.key.location()) else {
            return NetworkRegistryLoadOutcome {
                completed_noop: true,
                registered_elements: Vec::new(),
                loading_errors: BTreeMap::new(),
                registered_tags: BTreeMap::new(),
                operations: vec!["completedFuture(null)"],
            };
        };

        let converter = FileToIdConverter::registry(&self.data.key);
        let mut operations = vec![
            "RegistryOps.create(NbtOps.INSTANCE, context)",
            "RegistryOps.create(JsonOps.INSTANCE, context)",
            "FileToIdConverter.registry(this.registryKey())",
        ];
        let pending = registry_entries
            .elements
            .iter()
            .map(|entry| {
                let key = ResourceKey::<String>::create(&self.data.key, entry.id.clone());
                if let Some(tag) = &entry.data {
                    operations.push("PendingRegistration.loadFromNetwork");
                    PendingRegistrationModel::load_from_network(&self.data.element_codec, key, tag)
                } else {
                    operations.push("PendingRegistration.findAndLoadFromResource");
                    PendingRegistrationModel::find_and_load_from_resource(
                        &self.data.element_codec,
                        key,
                        &converter,
                        &self.known_data_source,
                    )
                }
            })
            .collect::<Vec<_>>();

        operations.push("registerElements");
        let mut registered_order = Vec::new();
        let mut registered_elements = Vec::new();
        let mut loading_errors = BTreeMap::new();
        for element in pending {
            match element.value {
                Ok(value) => {
                    registered_order.push(element.key.clone());
                    registered_elements.push((
                        element.key.location().clone(),
                        value,
                        element.registration_info,
                    ));
                }
                Err(error) => {
                    loading_errors.insert(element.key.java_to_string(), error);
                }
            }
        }

        operations.push("TagLoader.loadTagsFromNetwork");
        let registered_tags = registry_entries
            .tags
            .resolve(&self.data.key, &registered_order);
        operations.push("registerTags");

        NetworkRegistryLoadOutcome {
            completed_noop: false,
            registered_elements,
            loading_errors,
            registered_tags,
            operations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NETWORK_REGISTRY_LOAD_TASK_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/NetworkRegistryLoadTask.java");
    const REGISTRY_LOAD_TASK_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/RegistryLoadTask.java");
    const TAG_NETWORK_SERIALIZATION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/tags/TagNetworkSerialization.java");

    #[test]
    fn absent_network_registry_entries_complete_without_registering() {
        assert_java_contains(
            NETWORK_REGISTRY_LOAD_TASK_JAVA,
            &[
                "RegistryDataLoader.NetworkedRegistryData registryEntries = this.entries.get(this.registryKey());",
                "return CompletableFuture.completedFuture(null);",
            ],
        );

        let data = must(RegistryDataModel::new(
            "minecraft:worldgen/biome",
            "Biome.NETWORK_CODEC",
        ));
        let task = NetworkRegistryLoadTaskModel::new(data, ResourceProviderModel::default());
        let outcome = task.load();
        assert!(outcome.completed_noop);
        assert!(outcome.registered_elements.is_empty());
        assert!(outcome.registered_tags.is_empty());
        assert_eq!(outcome.operations, vec!["completedFuture(null)"]);
    }

    #[test]
    fn load_registers_network_values_then_known_data_fallbacks_then_tags() {
        assert_java_contains(
            NETWORK_REGISTRY_LOAD_TASK_JAVA,
            &[
                "new RegistrationInfo(Optional.empty(), Lifecycle.experimental())",
                "RegistryOps.create(NbtOps.INSTANCE, context)",
                "RegistryOps.create(JsonOps.INSTANCE, context)",
                "FileToIdConverter.registry(this.registryKey())",
                "RegistryLoadTask.PendingRegistration.loadFromNetwork",
                "RegistryLoadTask.PendingRegistration.findAndLoadFromResource",
                "this.registerElements(pendingRegistrations.stream())",
                "TagLoader.loadTagsFromNetwork(registryEntries.tags(), this.readOnlyRegistry())",
                "this.registerTags(pendingTags)",
            ],
        );
        assert_java_contains(
            REGISTRY_LOAD_TASK_JAVA,
            &[
                "this.registry.register(element.key, (T)value, element.registrationInfo)",
                "this.loadingErrors.put(element.key, error)",
                "this.elementsRegistered = true",
            ],
        );
        assert_java_contains(
            TAG_NETWORK_SERIALIZATION_JAVA,
            &[
                "ids.intStream().mapToObj(registry::get).flatMap(Optional::stream)",
                "TagKey.create(registryKey, key)",
            ],
        );

        let data = must(RegistryDataModel::new(
            "minecraft:worldgen/biome",
            "Biome.NETWORK_CODEC",
        ));
        let provider = must(ResourceProviderModel::default().with_resource(
            "minecraft:worldgen/biome/plains.json",
            "{\"temperature\":0.8}",
            "vanilla",
        ));
        let task = NetworkRegistryLoadTaskModel::new(data, provider).with_entries(
            NetworkedRegistryDataModel {
                elements: vec![
                    must(PackedRegistryEntryModel::network("minecraft:forest", "{nbt}")),
                    must(PackedRegistryEntryModel::known_data("minecraft:plains")),
                ],
                tags: NetworkPayloadModel {
                    tags: vec![must(NetworkTagModel::new(
                        "minecraft:is_overworld",
                        vec![0, 1, 99],
                    ))],
                },
            },
        );

        let outcome = task.load();
        assert!(!outcome.completed_noop);
        assert_eq!(
            outcome.operations,
            vec![
                "RegistryOps.create(NbtOps.INSTANCE, context)",
                "RegistryOps.create(JsonOps.INSTANCE, context)",
                "FileToIdConverter.registry(this.registryKey())",
                "PendingRegistration.loadFromNetwork",
                "PendingRegistration.findAndLoadFromResource",
                "registerElements",
                "TagLoader.loadTagsFromNetwork",
                "registerTags",
            ]
        );
        assert_eq!(outcome.loading_errors, BTreeMap::new());
        assert_eq!(outcome.registered_elements.len(), 2);
        assert_eq!(outcome.registered_elements[0].0.to_string(), "minecraft:forest");
        assert_eq!(outcome.registered_elements[0].1, "Biome.NETWORK_CODEC:nbt:{nbt}");
        assert_eq!(
            outcome.registered_elements[0].2,
            network_registration_info()
        );
        assert_eq!(outcome.registered_elements[1].0.to_string(), "minecraft:plains");
        assert_eq!(
            outcome.registered_elements[1].1,
            "Biome.NETWORK_CODEC:json:{\"temperature\":0.8}"
        );

        let tag_key = TagKeyModel {
            registry: must(Identifier::parse("minecraft:worldgen/biome")),
            location: must(Identifier::parse("minecraft:is_overworld")),
        };
        let tag_values = outcome
            .registered_tags
            .get(&tag_key)
            .unwrap_or_else(|| panic!("missing tag {tag_key:?}"));
        assert_eq!(
            tag_values
                .iter()
                .map(|holder| holder.key.to_string())
                .collect::<Vec<_>>(),
            vec!["minecraft:forest", "minecraft:plains"]
        );
    }

    #[test]
    fn load_records_element_errors_and_resolves_tags_against_successful_registry() {
        let data = must(RegistryDataModel::new(
            "minecraft:chat_type",
            "ChatType.DIRECT_CODEC",
        ));
        let task = NetworkRegistryLoadTaskModel::new(data, ResourceProviderModel::default())
            .with_entries(NetworkedRegistryDataModel {
                elements: vec![
                    must(PackedRegistryEntryModel::network("minecraft:chat", "ok")),
                    must(PackedRegistryEntryModel::network(
                        "minecraft:msg_command_outgoing",
                        "error",
                    )),
                    must(PackedRegistryEntryModel::known_data(
                        "minecraft:team_msg_command_incoming",
                    )),
                ],
                tags: NetworkPayloadModel {
                    tags: vec![must(NetworkTagModel::new(
                        "minecraft:network_safe",
                        vec![0, 1, 2],
                    ))],
                },
            });

        let outcome = task.load();
        assert_eq!(outcome.registered_elements.len(), 1);
        assert_eq!(outcome.loading_errors.len(), 2);
        assert!(outcome
            .loading_errors
            .values()
            .any(|error| error.contains("Failed to parse value error")));
        assert!(outcome
            .loading_errors
            .values()
            .any(|error| error.contains("Failed to find resource")));

        let holders = outcome
            .registered_tags
            .get(&TagKeyModel {
                registry: must(Identifier::parse("minecraft:chat_type")),
                location: must(Identifier::parse("minecraft:network_safe")),
            })
            .unwrap_or_else(|| panic!("missing network_safe tag"));
        assert_eq!(holders.len(), 1);
        assert_eq!(holders[0].key.to_string(), "minecraft:chat");
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
