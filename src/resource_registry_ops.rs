#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::registry::{Identifier, Registry, ResourceKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryOpsLifecycle {
    Stable,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryOpsInfo {
    pub owner: String,
    pub getter: RegistryOpsGetter,
    pub elements_lifecycle: RegistryOpsLifecycle,
}

impl RegistryOpsInfo {
    pub fn from_registry_lookup(lookup: RegistryLookupModel) -> Self {
        Self {
            owner: lookup.owner.clone(),
            getter: RegistryOpsGetter {
                registry: lookup.registry,
                owner: lookup.owner,
                elements: lookup.elements,
            },
            elements_lifecycle: lookup.lifecycle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryOpsGetter {
    pub registry: Identifier,
    pub owner: String,
    pub elements: BTreeMap<Identifier, String>,
}

impl RegistryOpsGetter {
    pub fn get(&self, key: &ResourceKey<String>) -> Option<HolderReferenceModel> {
        (key.registry() == &self.registry)
            .then(|| self.elements.get(key.location()))
            .flatten()
            .map(|value| HolderReferenceModel {
                key: key.clone(),
                value: value.clone(),
                owner: self.owner.clone(),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderReferenceModel {
    pub key: ResourceKey<String>,
    pub value: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLookupModel {
    pub registry: Identifier,
    pub owner: String,
    pub lifecycle: RegistryOpsLifecycle,
    pub elements: BTreeMap<Identifier, String>,
}

impl RegistryLookupModel {
    pub fn new(
        registry: &str,
        owner: &str,
        lifecycle: RegistryOpsLifecycle,
        elements: &[(&str, &str)],
    ) -> Result<Self, String> {
        let elements = elements
            .iter()
            .map(|(id, value)| Ok((Identifier::parse(id)?, (*value).to_string())))
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        Ok(Self {
            registry: Identifier::parse(registry)?,
            owner: owner.to_string(),
            lifecycle,
            elements,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderLookupProviderModel {
    id: String,
    lookups: BTreeMap<Identifier, RegistryLookupModel>,
}

impl HolderLookupProviderModel {
    pub fn new(id: &str, lookups: Vec<RegistryLookupModel>) -> Self {
        Self {
            id: id.to_string(),
            lookups: lookups
                .into_iter()
                .map(|lookup| (lookup.registry.clone(), lookup))
                .collect(),
        }
    }

    pub fn lookup(&self, registry: &Identifier) -> Option<RegistryLookupModel> {
        self.lookups.get(registry).cloned()
    }

    pub fn create_serialization_context(&self, parent_ops: &str) -> String {
        format!("RegistryOps({parent_ops}, provider={})", self.id)
    }
}

#[derive(Debug, Clone)]
pub struct HolderLookupAdapterModel {
    provider: HolderLookupProviderModel,
    cache: BTreeMap<Identifier, Option<RegistryOpsInfo>>,
}

impl HolderLookupAdapterModel {
    pub fn new(provider: HolderLookupProviderModel) -> Self {
        Self {
            provider,
            cache: BTreeMap::new(),
        }
    }

    pub fn lookup(&mut self, registry: &Identifier) -> Option<RegistryOpsInfo> {
        if !self.cache.contains_key(registry) {
            let value = self
                .provider
                .lookup(registry)
                .map(RegistryOpsInfo::from_registry_lookup);
            self.cache.insert(registry.clone(), value);
        }
        self.cache.get(registry).cloned().flatten()
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    pub fn provider_id(&self) -> &str {
        &self.provider.id
    }
}

impl PartialEq for HolderLookupAdapterModel {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider
    }
}

impl Eq for HolderLookupAdapterModel {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryInfoLookupModel {
    Direct {
        id: String,
        infos: BTreeMap<Identifier, RegistryOpsInfo>,
    },
    Adapter(HolderLookupAdapterModel),
}

impl RegistryInfoLookupModel {
    pub fn direct(id: &str, infos: Vec<RegistryOpsInfo>) -> Self {
        Self::Direct {
            id: id.to_string(),
            infos: infos
                .into_iter()
                .map(|info| (info.getter.registry.clone(), info))
                .collect(),
        }
    }

    pub fn from_provider(provider: HolderLookupProviderModel) -> Self {
        Self::Adapter(HolderLookupAdapterModel::new(provider))
    }

    pub fn lookup(&mut self, registry: &Identifier) -> Option<RegistryOpsInfo> {
        match self {
            Self::Direct { infos, .. } => infos.get(registry).cloned(),
            Self::Adapter(adapter) => adapter.lookup(registry),
        }
    }

    fn stable_identity(&self) -> String {
        match self {
            Self::Direct { id, .. } => format!("direct:{id}"),
            Self::Adapter(adapter) => format!("adapter:{}", adapter.provider_id()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryOpsModel {
    delegate: String,
    lookup_provider: RegistryInfoLookupModel,
}

impl RegistryOpsModel {
    pub fn create(parent: &str, lookup_provider: RegistryInfoLookupModel) -> Self {
        Self {
            delegate: parent.to_string(),
            lookup_provider,
        }
    }

    pub fn create_from_holder_provider(parent: &str, provider: HolderLookupProviderModel) -> Self {
        Self::create(parent, RegistryInfoLookupModel::from_provider(provider))
    }

    pub fn inject_registry_context(dynamic: DynamicModel, provider: &HolderLookupProviderModel) -> DynamicModel {
        DynamicModel {
            ops: provider.create_serialization_context(&dynamic.ops),
            value: dynamic.value,
        }
    }

    pub fn with_parent(&self, parent: &str) -> Self {
        if parent == self.delegate {
            self.clone()
        } else {
            Self {
                delegate: parent.to_string(),
                lookup_provider: self.lookup_provider.clone(),
            }
        }
    }

    pub fn owner(&mut self, registry_key: &Identifier) -> Option<String> {
        self.lookup_provider
            .lookup(registry_key)
            .map(|info| info.owner)
    }

    pub fn getter(&mut self, registry_key: &Identifier) -> Option<RegistryOpsGetter> {
        self.lookup_provider
            .lookup(registry_key)
            .map(|info| info.getter)
    }

    pub fn retrieve_getter(&mut self, registry_key: &Identifier) -> RegistryOpsResult<RegistryOpsGetter> {
        match self.lookup_provider.lookup(registry_key) {
            Some(info) => RegistryOpsResult::success(info.getter, info.elements_lifecycle),
            None => RegistryOpsResult::error(format!(
                "Unknown registry: {}",
                registry_key_string(registry_key)
            )),
        }
    }

    pub fn retrieve_element(&mut self, key: &ResourceKey<String>) -> RegistryOpsResult<HolderReferenceModel> {
        let registry_key = key.registry().clone();
        let value = self
            .lookup_provider
            .lookup(&registry_key)
            .and_then(|info| info.getter.get(key));
        match value {
            Some(holder) => RegistryOpsResult::success(holder, RegistryOpsLifecycle::Stable),
            None => RegistryOpsResult::error(format!("Can't find value: {}", key.java_to_string())),
        }
    }

    pub fn java_hash_key(&self) -> String {
        format!(
            "{}*31+{}",
            self.delegate,
            self.lookup_provider.stable_identity()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicModel {
    pub ops: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryOpsResult<T> {
    Success {
        value: T,
        lifecycle: RegistryOpsLifecycle,
    },
    Error(String),
}

impl<T> RegistryOpsResult<T> {
    pub fn success(value: T, lifecycle: RegistryOpsLifecycle) -> Self {
        Self::Success { value, lifecycle }
    }

    pub fn error(message: String) -> Self {
        Self::Error(message)
    }
}

pub fn retrieve_getter_from_plain_ops<E>(
    _registry_key: &ResourceKey<Registry<E>>,
) -> RegistryOpsResult<RegistryOpsGetter> {
    RegistryOpsResult::error("Not a registry ops".to_string())
}

pub fn retrieve_element_from_plain_ops<E>(
    _key: &ResourceKey<E>,
) -> RegistryOpsResult<HolderReferenceModel> {
    RegistryOpsResult::error("Not a registry ops".to_string())
}

fn registry_key_string(registry: &Identifier) -> String {
    format!("ResourceKey[minecraft:root / {registry}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY_OPS_JAVA: &str = vibecraft_java_source!("/net/minecraft/resources/RegistryOps.java");

    #[test]
    fn create_inject_with_parent_owner_getter_and_equality_follow_java_contract() {
        assert_java_contains(
            REGISTRY_OPS_JAVA,
            &[
                "public class RegistryOps<T> extends DelegatingOps<T>",
                "return create(parent, new RegistryOps.HolderLookupAdapter(lookupProvider));",
                "return new Dynamic(lookupProvider.createSerializationContext(dynamic.getOps()), dynamic.getValue());",
                "return (RegistryOps<U>)(parent == this.delegate ? this : new RegistryOps((DynamicOps<T>)parent, this.lookupProvider));",
                "return this.lookupProvider.lookup(registryKey).map(RegistryOps.RegistryInfo::owner);",
                "return this.lookupProvider.lookup(registryKey).map(RegistryOps.RegistryInfo::getter);",
                "return this.delegate.equals(ops.delegate) && this.lookupProvider.equals(ops.lookupProvider);",
                "return this.delegate.hashCode() * 31 + this.lookupProvider.hashCode();",
            ],
        );

        let provider = HolderLookupProviderModel::new(
            "provider",
            vec![must(RegistryLookupModel::new(
                "minecraft:chat_type",
                "chat-owner",
                RegistryOpsLifecycle::Stable,
                &[("minecraft:chat", "chat")],
            ))],
        );
        let dynamic = DynamicModel {
            ops: "json".to_string(),
            value: "{\"value\":1}".to_string(),
        };
        assert_eq!(
            RegistryOpsModel::inject_registry_context(dynamic, &provider),
            DynamicModel {
                ops: "RegistryOps(json, provider=provider)".to_string(),
                value: "{\"value\":1}".to_string(),
            }
        );

        let mut ops = RegistryOpsModel::create_from_holder_provider("json", provider);
        assert_eq!(ops.with_parent("json"), ops);
        assert_eq!(ops.with_parent("nbt").delegate, "nbt");
        assert_eq!(
            ops.owner(&must(Identifier::parse("minecraft:chat_type"))),
            Some("chat-owner".to_string())
        );
        assert_eq!(
            ops.getter(&must(Identifier::parse("minecraft:chat_type")))
                .map(|getter| getter.registry.to_string()),
            Some("minecraft:chat_type".to_string())
        );
        assert_eq!(ops.java_hash_key(), "json*31+adapter:provider");
    }

    #[test]
    fn retrieve_getter_returns_lifecycle_success_unknown_registry_or_plain_ops_error() {
        assert_java_contains(
            REGISTRY_OPS_JAVA,
            &[
                "public static <E, O> RecordCodecBuilder<O, HolderGetter<E>> retrieveGetter",
                "ops -> ops instanceof RegistryOps<?> registryOps",
                ".map(r -> DataResult.success(r.getter(), r.elementsLifecycle()))",
                "DataResult.error(() -> \"Unknown registry: \" + registryKey)",
                "DataResult.error(() -> \"Not a registry ops\")",
            ],
        );

        let info = RegistryOpsInfo::from_registry_lookup(must(RegistryLookupModel::new(
            "minecraft:chat_type",
            "chat-owner",
            RegistryOpsLifecycle::Experimental,
            &[("minecraft:chat", "chat")],
        )));
        let mut ops = RegistryOpsModel::create("json", RegistryInfoLookupModel::direct("lookup", vec![info]));
        let result = ops.retrieve_getter(&must(Identifier::parse("minecraft:chat_type")));
        match result {
            RegistryOpsResult::Success { value, lifecycle } => {
                assert_eq!(value.registry.to_string(), "minecraft:chat_type");
                assert_eq!(lifecycle, RegistryOpsLifecycle::Experimental);
            }
            RegistryOpsResult::Error(error) => panic!("{error}"),
        }
        assert_eq!(
            ops.retrieve_getter(&must(Identifier::parse("minecraft:missing"))),
            RegistryOpsResult::Error(
                "Unknown registry: ResourceKey[minecraft:root / minecraft:missing]".to_string()
            )
        );
        assert_eq!(
            retrieve_getter_from_plain_ops(&ResourceKey::<Registry<String>>::create_registry_key(
                must(Identifier::parse("minecraft:chat_type")),
            )),
            RegistryOpsResult::Error("Not a registry ops".to_string())
        );
    }

    #[test]
    fn retrieve_element_builds_registry_key_and_reports_missing_values_like_java() {
        assert_java_contains(
            REGISTRY_OPS_JAVA,
            &[
                "public static <E, O> RecordCodecBuilder<O, Holder.Reference<E>> retrieveElement",
                "ResourceKey<? extends Registry<E>> registryKey = ResourceKey.createRegistryKey(key.registry());",
                ".flatMap(r -> r.getter().get(key))",
                ".<DataResult<E>>map(DataResult::success)",
                "DataResult.error(() -> \"Can't find value: \" + key)",
                "DataResult.error(() -> \"Not a registry ops\")",
            ],
        );

        let info = RegistryOpsInfo::from_registry_lookup(must(RegistryLookupModel::new(
            "minecraft:chat_type",
            "chat-owner",
            RegistryOpsLifecycle::Stable,
            &[("minecraft:chat", "chat")],
        )));
        let mut ops = RegistryOpsModel::create("json", RegistryInfoLookupModel::direct("lookup", vec![info]));
        let chat_key = ResourceKey::<String>::new(
            must(Identifier::parse("minecraft:chat_type")),
            must(Identifier::parse("minecraft:chat")),
        );
        let result = ops.retrieve_element(&chat_key);
        match result {
            RegistryOpsResult::Success { value, .. } => {
                assert_eq!(value.key.location().to_string(), "minecraft:chat");
                assert_eq!(value.value, "chat");
            }
            RegistryOpsResult::Error(error) => panic!("{error}"),
        }

        let missing_key = ResourceKey::<String>::new(
            must(Identifier::parse("minecraft:chat_type")),
            must(Identifier::parse("minecraft:missing")),
        );
        assert_eq!(
            ops.retrieve_element(&missing_key),
            RegistryOpsResult::Error(
                "Can't find value: ResourceKey[minecraft:chat_type / minecraft:missing]"
                    .to_string()
            )
        );
        assert_eq!(
            retrieve_element_from_plain_ops(&missing_key),
            RegistryOpsResult::Error("Not a registry ops".to_string())
        );
    }

    #[test]
    fn holder_lookup_adapter_caches_present_and_absent_lookups_and_compares_by_provider() {
        assert_java_contains(
            REGISTRY_OPS_JAVA,
            &[
                "private final Map<ResourceKey<? extends Registry<?>>, Optional<? extends RegistryOps.RegistryInfo<?>>> lookups = new ConcurrentHashMap<>();",
                "return (Optional<RegistryOps.RegistryInfo<E>>)this.lookups.computeIfAbsent(registryKey, this::createLookup);",
                "return this.lookupProvider.lookup(key).map(RegistryOps.RegistryInfo::fromRegistryLookup);",
                "obj instanceof RegistryOps.HolderLookupAdapter adapter && this.lookupProvider.equals(adapter.lookupProvider)",
                "return this.lookupProvider.hashCode();",
                "return new RegistryOps.RegistryInfo<>(registry, registry, registry.registryLifecycle());",
            ],
        );

        let provider = HolderLookupProviderModel::new(
            "provider",
            vec![must(RegistryLookupModel::new(
                "minecraft:chat_type",
                "chat-owner",
                RegistryOpsLifecycle::Stable,
                &[("minecraft:chat", "chat")],
            ))],
        );
        let mut adapter = HolderLookupAdapterModel::new(provider.clone());
        assert_eq!(adapter.cache_size(), 0);
        assert!(adapter
            .lookup(&must(Identifier::parse("minecraft:chat_type")))
            .is_some());
        assert_eq!(adapter.cache_size(), 1);
        assert!(adapter
            .lookup(&must(Identifier::parse("minecraft:chat_type")))
            .is_some());
        assert_eq!(adapter.cache_size(), 1);
        assert!(adapter
            .lookup(&must(Identifier::parse("minecraft:missing")))
            .is_none());
        assert_eq!(adapter.cache_size(), 2);
        assert_eq!(adapter, HolderLookupAdapterModel::new(provider));
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
