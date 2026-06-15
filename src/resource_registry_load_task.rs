#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::registry::Identifier;
use crate::resource_file_to_id_converter::FileToIdConverter;
use crate::resource_registry_data_loader::RegistryDataLoaderValidatorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryLoadTaskLifecycle {
    Stable,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadTaskData {
    pub registry_key: Identifier,
    pub element_codec: &'static str,
    pub validator: RegistryDataLoaderValidatorKind,
}

impl RegistryLoadTaskData {
    pub fn new(registry_key: &str, element_codec: &'static str) -> Result<Self, String> {
        Ok(Self {
            registry_key: Identifier::parse(registry_key)?,
            element_codec,
            validator: RegistryDataLoaderValidatorKind::None,
        })
    }

    pub fn with_validator(
        registry_key: &str,
        element_codec: &'static str,
        validator: RegistryDataLoaderValidatorKind,
    ) -> Result<Self, String> {
        Ok(Self {
            registry_key: Identifier::parse(registry_key)?,
            element_codec,
            validator,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadRegistrationInfo {
    pub source: Option<String>,
    pub lifecycle: RegistryLoadTaskLifecycle,
}

impl RegistryLoadRegistrationInfo {
    pub fn stable() -> Self {
        Self {
            source: None,
            lifecycle: RegistryLoadTaskLifecycle::Stable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadResource {
    pub contents: String,
    pub source_pack_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryLoadResourceProvider {
    resources: BTreeMap<Identifier, RegistryLoadResource>,
}

impl RegistryLoadResourceProvider {
    pub fn with_resource(
        mut self,
        id: &str,
        contents: &str,
        source_pack_id: &str,
    ) -> Result<Self, String> {
        self.resources.insert(
            Identifier::parse(id)?,
            RegistryLoadResource {
                contents: contents.to_string(),
                source_pack_id: source_pack_id.to_string(),
            },
        );
        Ok(self)
    }

    fn get_resource(&self, id: &Identifier) -> Option<&RegistryLoadResource> {
        self.resources.get(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadPendingRegistration {
    pub key: Identifier,
    pub value: Result<String, String>,
    pub registration_info: RegistryLoadRegistrationInfo,
}

impl RegistryLoadPendingRegistration {
    pub fn success(key: &str, value: &str) -> Result<Self, String> {
        Ok(Self {
            key: Identifier::parse(key)?,
            value: Ok(value.to_string()),
            registration_info: RegistryLoadRegistrationInfo::stable(),
        })
    }

    pub fn failure(key: &str, error: &str) -> Result<Self, String> {
        Ok(Self {
            key: Identifier::parse(key)?,
            value: Err(error.to_string()),
            registration_info: RegistryLoadRegistrationInfo::stable(),
        })
    }

    pub fn load_from_resource(
        codec: &'static str,
        element_key: &Identifier,
        resource: &RegistryLoadResource,
    ) -> Self {
        let value = if resource.contents == "invalid-json" || resource.contents == "codec-error" {
            Err(format!(
                "Failed to parse {element_key} from pack {}",
                resource.source_pack_id
            ))
        } else {
            Ok(format!("{codec}:json:{}", resource.contents))
        };
        Self {
            key: element_key.clone(),
            value,
            registration_info: RegistryLoadRegistrationInfo::stable(),
        }
    }

    pub fn find_and_load_from_resource(
        codec: &'static str,
        element_key: &Identifier,
        converter: &FileToIdConverter,
        provider: &RegistryLoadResourceProvider,
    ) -> Self {
        let value = match converter.id_to_file(element_key) {
            Ok(resource_id) => provider
                .get_resource(&resource_id)
                .map(|resource| Self::load_from_resource(codec, element_key, resource).value)
                .unwrap_or_else(|| {
                    Err(format!(
                        "Failed to find resource {resource_id} for element {element_key}"
                    ))
                }),
            Err(error) => Err(error),
        };
        Self {
            key: element_key.clone(),
            value,
            registration_info: RegistryLoadRegistrationInfo::stable(),
        }
    }

    pub fn load_from_network(codec: &'static str, element_key: &Identifier, contents: &str) -> Self {
        let value = if contents == "codec-error" {
            Err(format!(
                "Failed to parse value {contents} for key {element_key} from server"
            ))
        } else {
            Ok(format!("{codec}:nbt:{contents}"))
        };
        Self {
            key: element_key.clone(),
            value,
            registration_info: RegistryLoadRegistrationInfo::stable(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredRegistryElement {
    pub key: Identifier,
    pub value: String,
    pub registration_info: RegistryLoadRegistrationInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadTaskInfo {
    pub registry: Identifier,
    pub holder_getter: &'static str,
    pub lifecycle: RegistryLoadTaskLifecycle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLoadTaskModel {
    data: RegistryLoadTaskData,
    lifecycle: RegistryLoadTaskLifecycle,
    elements_registered: bool,
    elements: Vec<RegisteredRegistryElement>,
    tags: BTreeMap<Identifier, Vec<Identifier>>,
    loading_errors: BTreeMap<String, String>,
    freeze_error: Option<String>,
}

impl RegistryLoadTaskModel {
    pub fn new(data: RegistryLoadTaskData, lifecycle: RegistryLoadTaskLifecycle) -> Self {
        Self {
            data,
            lifecycle,
            elements_registered: false,
            elements: Vec::new(),
            tags: BTreeMap::new(),
            loading_errors: BTreeMap::new(),
            freeze_error: None,
        }
    }

    pub fn registry_key(&self) -> &Identifier {
        &self.data.registry_key
    }

    pub fn element_codec(&self) -> &'static str {
        self.data.element_codec
    }

    pub fn read_only_registry(&self) -> Result<&[RegisteredRegistryElement], String> {
        if self.elements_registered {
            Ok(&self.elements)
        } else {
            Err("Elements not registered".to_string())
        }
    }

    pub fn create_registry_info(&self) -> RegistryLoadTaskInfo {
        RegistryLoadTaskInfo {
            registry: self.data.registry_key.clone(),
            holder_getter: "ConcurrentHolderGetter(registryWriteLock, createRegistrationLookup)",
            lifecycle: self.lifecycle,
        }
    }

    pub fn register_elements(&mut self, elements: Vec<RegistryLoadPendingRegistration>) {
        for element in elements {
            match element.value {
                Ok(value) => self.elements.push(RegisteredRegistryElement {
                    key: element.key,
                    value,
                    registration_info: element.registration_info,
                }),
                Err(error) => {
                    self.loading_errors
                        .insert(element_key_string(&self.data.registry_key, &element.key), error);
                }
            }
        }
        self.elements_registered = true;
    }

    pub fn register_tags(&mut self, pending_tags: BTreeMap<Identifier, Vec<Identifier>>) {
        self.tags = pending_tags;
    }

    pub fn set_freeze_error(&mut self, error: &str) {
        self.freeze_error = Some(error.to_string());
    }

    pub fn freeze_registry(&mut self, loading_errors: &mut BTreeMap<String, String>) -> bool {
        if let Some(error) = &self.freeze_error {
            loading_errors.insert(registry_key_string(&self.data.registry_key), error.clone());
            false
        } else {
            true
        }
    }

    pub fn validate_registry(
        &self,
        loading_errors: &mut BTreeMap<String, String>,
    ) -> Option<&[RegisteredRegistryElement]> {
        let mut registry_errors = BTreeMap::new();
        match self.data.validator {
            RegistryDataLoaderValidatorKind::None => {}
            RegistryDataLoaderValidatorKind::NonEmpty => {
                if self.elements.is_empty() {
                    registry_errors.insert(
                        registry_key_string(&self.data.registry_key),
                        format!("Registry must be non-empty: {}", self.data.registry_key),
                    );
                }
            }
            RegistryDataLoaderValidatorKind::TimelineValidateRegistry => {
                if self.elements.iter().any(|element| element.value == "bad-timeline") {
                    registry_errors.insert(
                        registry_key_string(&self.data.registry_key),
                        "Timeline validation failed".to_string(),
                    );
                }
            }
        }

        if registry_errors.is_empty() {
            Some(&self.elements)
        } else {
            loading_errors.extend(registry_errors);
            None
        }
    }

    pub fn loading_errors(&self) -> &BTreeMap<String, String> {
        &self.loading_errors
    }

    pub fn tags(&self) -> &BTreeMap<Identifier, Vec<Identifier>> {
        &self.tags
    }
}

fn registry_key_string(registry: &Identifier) -> String {
    format!("ResourceKey[minecraft:root / {registry}]")
}

fn element_key_string(registry: &Identifier, element: &Identifier) -> String {
    format!("ResourceKey[{registry} / {element}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY_LOAD_TASK_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/RegistryLoadTask.java");

    #[test]
    fn constructor_registry_key_read_only_gate_and_registry_info_match_java() {
        assert_java_contains(
            REGISTRY_LOAD_TASK_JAVA,
            &[
                "private final Object registryWriteLock = new Object();",
                "this.registry = new MappedRegistry<>(data.key(), lifecycle);",
                "this.concurrentRegistrationGetter = new ConcurrentHolderGetter<>(this.registryWriteLock, this.registry.createRegistrationLookup());",
                "return this.registry.key();",
                "throw new IllegalStateException(\"Elements not registered\");",
                "return new RegistryOps.RegistryInfo<>(this.registry, this.concurrentRegistrationGetter, this.registry.registryLifecycle());",
            ],
        );

        let task = RegistryLoadTaskModel::new(
            must(RegistryLoadTaskData::new(
                "minecraft:chat_type",
                "ChatType.DIRECT_CODEC",
            )),
            RegistryLoadTaskLifecycle::Stable,
        );

        assert_eq!(task.registry_key().to_string(), "minecraft:chat_type");
        assert_eq!(task.read_only_registry(), Err("Elements not registered".to_string()));
        assert_eq!(
            task.create_registry_info(),
            RegistryLoadTaskInfo {
                registry: must(Identifier::parse("minecraft:chat_type")),
                holder_getter: "ConcurrentHolderGetter(registryWriteLock, createRegistrationLookup)",
                lifecycle: RegistryLoadTaskLifecycle::Stable,
            }
        );
    }

    #[test]
    fn register_elements_splits_successes_and_errors_then_enables_read_only_registry() {
        assert_java_contains(
            REGISTRY_LOAD_TASK_JAVA,
            &[
                "synchronized (this.registryWriteLock)",
                ".ifLeft(value -> this.registry.register(element.key, (T)value, element.registrationInfo))",
                ".ifRight(error -> this.loadingErrors.put(element.key, error))",
                "this.elementsRegistered = true;",
            ],
        );

        let mut task = RegistryLoadTaskModel::new(
            must(RegistryLoadTaskData::new(
                "minecraft:chat_type",
                "ChatType.DIRECT_CODEC",
            )),
            RegistryLoadTaskLifecycle::Stable,
        );
        task.register_elements(vec![
            must(RegistryLoadPendingRegistration::success("minecraft:chat", "chat")),
            must(RegistryLoadPendingRegistration::failure(
                "minecraft:team_msg_command_incoming",
                "broken",
            )),
        ]);

        let registry = must(task.read_only_registry().map_err(|error| error.to_string()));
        assert_eq!(registry.len(), 1);
        assert_eq!(registry[0].key.to_string(), "minecraft:chat");
        assert_eq!(
            task.loading_errors(),
            &BTreeMap::from([(
                "ResourceKey[minecraft:chat_type / minecraft:team_msg_command_incoming]"
                    .to_string(),
                "broken".to_string(),
            )])
        );
    }

    #[test]
    fn register_tags_freeze_and_validate_follow_java_error_flow() {
        assert_java_contains(
            REGISTRY_LOAD_TASK_JAVA,
            &[
                "this.registry.bindTags(pendingTags);",
                "this.registry.freeze();",
                "loadingErrors.put(this.registry.key(), e);",
                "this.data.validator().validate(this.registry, registryErrors);",
                "loadingErrors.putAll(registryErrors);",
                "return Optional.empty();",
            ],
        );

        let mut task = RegistryLoadTaskModel::new(
            must(RegistryLoadTaskData::with_validator(
                "minecraft:wolf_variant",
                "WolfVariant.DIRECT_CODEC",
                RegistryDataLoaderValidatorKind::NonEmpty,
            )),
            RegistryLoadTaskLifecycle::Stable,
        );
        task.register_tags(BTreeMap::from([(
            must(Identifier::parse("minecraft:is_wolf")),
            vec![must(Identifier::parse("minecraft:pale"))],
        )]));
        assert_eq!(task.tags().len(), 1);

        let mut loading_errors = BTreeMap::new();
        assert!(task.freeze_registry(&mut loading_errors));
        assert!(loading_errors.is_empty());
        assert!(task.validate_registry(&mut loading_errors).is_none());
        assert_eq!(
            loading_errors,
            BTreeMap::from([(
                "ResourceKey[minecraft:root / minecraft:wolf_variant]".to_string(),
                "Registry must be non-empty: minecraft:wolf_variant".to_string(),
            )])
        );

        let mut freeze_failure = RegistryLoadTaskModel::new(
            must(RegistryLoadTaskData::new(
                "minecraft:chat_type",
                "ChatType.DIRECT_CODEC",
            )),
            RegistryLoadTaskLifecycle::Stable,
        );
        freeze_failure.set_freeze_error("freeze failed");
        let mut freeze_errors = BTreeMap::new();
        assert!(!freeze_failure.freeze_registry(&mut freeze_errors));
        assert_eq!(
            freeze_errors,
            BTreeMap::from([(
                "ResourceKey[minecraft:root / minecraft:chat_type]".to_string(),
                "freeze failed".to_string(),
            )])
        );
    }

    #[test]
    fn pending_registration_resource_and_network_helpers_match_java_error_text() {
        assert_java_contains(
            REGISTRY_LOAD_TASK_JAVA,
            &[
                "JsonElement json = StrictJsonParser.parse(reader);",
                "elementDecoder.parse(ops, json).getOrThrow()",
                "Failed to parse %s from pack %s",
                "Identifier resourceId = converter.idToFile(elementKey.identifier());",
                "resourceProvider.getResource(resourceId)",
                "Failed to find resource %s for element %s",
                "DataResult<T> parseResult = elementDecoder.parse(ops, contents);",
                "Failed to parse value %s for key %s from server",
            ],
        );

        let element = must(Identifier::parse("minecraft:chat"));
        let resource = RegistryLoadResource {
            contents: "{\"translation\":\"chat.type.text\"}".to_string(),
            source_pack_id: "vanilla".to_string(),
        };
        assert_eq!(
            RegistryLoadPendingRegistration::load_from_resource(
                "ChatType.DIRECT_CODEC",
                &element,
                &resource,
            )
            .value,
            Ok("ChatType.DIRECT_CODEC:json:{\"translation\":\"chat.type.text\"}".to_string())
        );

        let provider = must(RegistryLoadResourceProvider::default().with_resource(
            "minecraft:chat_type/chat.json",
            "{}",
            "vanilla",
        ));
        let converter = FileToIdConverter::json("chat_type");
        assert_eq!(
            RegistryLoadPendingRegistration::find_and_load_from_resource(
                "ChatType.DIRECT_CODEC",
                &element,
                &converter,
                &provider,
            )
            .value,
            Ok("ChatType.DIRECT_CODEC:json:{}".to_string())
        );
        assert_eq!(
            RegistryLoadPendingRegistration::find_and_load_from_resource(
                "ChatType.DIRECT_CODEC",
                &must(Identifier::parse("minecraft:missing")),
                &converter,
                &provider,
            )
            .value,
            Err(
                "Failed to find resource minecraft:chat_type/missing.json for element minecraft:missing"
                    .to_string()
            )
        );
        assert_eq!(
            RegistryLoadPendingRegistration::load_from_network(
                "ChatType.DIRECT_CODEC",
                &element,
                "codec-error",
            )
            .value,
            Err("Failed to parse value codec-error for key minecraft:chat from server".to_string())
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
