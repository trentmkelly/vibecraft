use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifecycleModel {
    Stable,
    Experimental,
}

impl LifecycleModel {
    fn stable() -> Self {
        Self::Stable
    }

    fn experimental() -> Self {
        Self::Experimental
    }

    fn add(self, other: Self) -> Self {
        if matches!(self, Self::Experimental) || matches!(other, Self::Experimental) {
            Self::Experimental
        } else {
            Self::Stable
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RegistryKeyModel(String);

impl RegistryKeyModel {
    fn new(value: &str) -> Self {
        Self(value.to_string())
    }

    fn identifier(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegistryKeyModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceKeyModel {
    registry: RegistryKeyModel,
    location: String,
}

impl ResourceKeyModel {
    fn new(registry: &RegistryKeyModel, location: &str) -> Self {
        Self {
            registry: registry.clone(),
            location: location.to_string(),
        }
    }

    fn is_for(&self, registry: &RegistryKeyModel) -> bool {
        &self.registry == registry
    }
}

impl fmt::Display for ResourceKeyModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "ResourceKey[{} / {}]",
            self.registry, self.location
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderModel {
    owner: usize,
    key: ResourceKeyModel,
    value: Option<String>,
    lazy: Option<LazyCloneModel>,
}

impl HolderModel {
    fn stand_alone(owner: usize, key: ResourceKeyModel) -> Self {
        Self {
            owner,
            key,
            value: None,
            lazy: None,
        }
    }

    fn bound(owner: usize, key: ResourceKeyModel, value: &str) -> Self {
        Self {
            owner,
            key,
            value: Some(value.to_string()),
            lazy: None,
        }
    }

    fn lazy(
        owner: usize,
        key: ResourceKeyModel,
        source_provider: ProviderKind,
        value: &str,
    ) -> Self {
        Self {
            owner,
            key,
            value: None,
            lazy: Some(LazyCloneModel {
                source_provider,
                source_value: value.to_string(),
                calls: Rc::new(Cell::new(0)),
            }),
        }
    }

    fn bind_value(&mut self, value: String) {
        self.value = Some(value);
        self.lazy = None;
    }

    fn value(&mut self) -> &str {
        if let Some(lazy) = self.lazy.take() {
            lazy.calls.set(lazy.calls.get() + 1);
            self.bind_value(format!(
                "cloned:{}:{}",
                lazy.source_provider.as_str(),
                lazy.source_value
            ));
        }
        match self.value.as_deref() {
            Some(value) => value,
            None => panic!("holder value is bound"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderKind {
    Patch,
    Fallback,
}

impl ProviderKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Patch => "patch",
            Self::Fallback => "fallback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LazyCloneModel {
    source_provider: ProviderKind,
    source_value: String,
    calls: Rc<Cell<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryLookupModel {
    key: RegistryKeyModel,
    lifecycle: LifecycleModel,
    owner: usize,
    entries: BTreeMap<ResourceKeyModel, HolderModel>,
    tags_available: bool,
}

impl RegistryLookupModel {
    fn new(key: RegistryKeyModel, lifecycle: LifecycleModel, entries: Vec<(&str, &str)>) -> Self {
        let owner = 1;
        let entries = entries
            .into_iter()
            .map(|(location, value)| {
                let key = ResourceKeyModel::new(&key, location);
                (key.clone(), HolderModel::bound(owner, key, value))
            })
            .collect();
        Self {
            key,
            lifecycle,
            owner,
            entries,
            tags_available: true,
        }
    }

    fn empty_tags(mut self) -> Self {
        self.tags_available = false;
        self
    }

    fn get(&self, key: &ResourceKeyModel) -> Option<HolderModel> {
        self.entries.get(key).cloned()
    }

    fn get_tag(&self, tag: &str) -> Vec<HolderModel> {
        let _ = tag;
        Vec::new()
    }

    fn list_tags(&self) -> Result<Vec<String>, String> {
        if self.tags_available {
            Ok(Vec::new())
        } else {
            Err("Tags are not available in datagen".to_string())
        }
    }

    fn list_elements(&self) -> Vec<HolderModel> {
        self.entries.values().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderModel {
    lookups: BTreeMap<RegistryKeyModel, RegistryLookupModel>,
}

impl ProviderModel {
    fn new(registries: impl IntoIterator<Item = RegistryLookupModel>) -> Self {
        Self {
            lookups: registries
                .into_iter()
                .map(|lookup| (lookup.key.clone(), lookup))
                .collect(),
        }
    }

    fn lookup(&self, key: &RegistryKeyModel) -> Option<&RegistryLookupModel> {
        self.lookups.get(key)
    }

    fn lookup_or_throw(&self, key: &RegistryKeyModel) -> Result<&RegistryLookupModel, String> {
        self.lookup(key)
            .ok_or_else(|| format!("Missing registry: {key}"))
    }

    fn list_registry_keys(&self) -> Vec<RegistryKeyModel> {
        self.lookups.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryAccessModel {
    registries: BTreeMap<RegistryKeyModel, RegistryLookupModel>,
}

impl RegistryAccessModel {
    fn new(registries: impl IntoIterator<Item = RegistryLookupModel>) -> Self {
        Self {
            registries: registries
                .into_iter()
                .map(|lookup| (lookup.key.clone(), lookup))
                .collect(),
        }
    }

    fn registries(&self) -> Vec<RegistryLookupModel> {
        self.registries.values().cloned().collect()
    }

    fn list_registry_keys(&self) -> BTreeSet<RegistryKeyModel> {
        self.registries.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegisteredValueModel {
    value: String,
    lifecycle: LifecycleModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueAndHolderModel {
    value: RegisteredValueModel,
    holder: Option<HolderModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryContentsModel {
    key: RegistryKeyModel,
    lifecycle: LifecycleModel,
    values: BTreeMap<ResourceKeyModel, ValueAndHolderModel>,
}

impl RegistryContentsModel {
    fn build_as_lookup(&self, owner: usize) -> RegistryLookupModel {
        let entries = self
            .values
            .iter()
            .map(|(key, entry)| {
                let mut holder = entry
                    .holder
                    .clone()
                    .unwrap_or_else(|| HolderModel::stand_alone(owner, key.clone()));
                holder.bind_value(entry.value.value.clone());
                (key.clone(), holder)
            })
            .collect();
        RegistryLookupModel {
            key: self.key.clone(),
            lifecycle: self.lifecycle,
            owner,
            entries,
            tags_available: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BootstrapActionModel {
    Register {
        key: ResourceKeyModel,
        value: String,
        lifecycle: LifecycleModel,
    },
    LookupElement {
        registry: RegistryKeyModel,
        key: ResourceKeyModel,
    },
    LookupTag {
        registry: RegistryKeyModel,
        tag: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryStubModel {
    key: RegistryKeyModel,
    lifecycle: LifecycleModel,
    bootstrap: Vec<BootstrapActionModel>,
}

impl RegistryStubModel {
    fn apply(&self, state: &mut BuildStateModel) {
        for action in &self.bootstrap {
            match action {
                BootstrapActionModel::Register {
                    key,
                    value,
                    lifecycle,
                } => state.register(key.clone(), value.clone(), *lifecycle),
                BootstrapActionModel::LookupElement { registry, key } => {
                    let _ = state.lookup(registry, key);
                }
                BootstrapActionModel::LookupTag { registry, tag } => {
                    let _ = state.lookup_tag(registry, tag);
                }
            }
        }
    }

    fn collect_registered_values(&self, state: &mut BuildStateModel) -> RegistryContentsModel {
        let mut result = BTreeMap::new();
        let matching_keys = state
            .registered_values
            .keys()
            .filter(|key| key.is_for(&self.key))
            .cloned()
            .collect::<Vec<_>>();
        for key in matching_keys {
            let Some(value) = state.registered_values.remove(&key) else {
                continue;
            };
            let holder = state.holders.remove(&key);
            result.insert(key, ValueAndHolderModel { value, holder });
        }
        RegistryContentsModel {
            key: self.key.clone(),
            lifecycle: self.lifecycle,
            values: result,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BuildStateModel {
    owner: usize,
    context: RegistryAccessModel,
    new_registries: BTreeSet<RegistryKeyModel>,
    holders: BTreeMap<ResourceKeyModel, HolderModel>,
    registered_values: BTreeMap<ResourceKeyModel, RegisteredValueModel>,
    errors: Vec<String>,
}

impl BuildStateModel {
    fn create(
        context: RegistryAccessModel,
        new_registries: impl IntoIterator<Item = RegistryKeyModel>,
    ) -> Self {
        Self {
            owner: 2,
            context,
            new_registries: new_registries.into_iter().collect(),
            holders: BTreeMap::new(),
            registered_values: BTreeMap::new(),
            errors: Vec::new(),
        }
    }

    fn register(&mut self, key: ResourceKeyModel, value: String, lifecycle: LifecycleModel) {
        let new_value = value.clone();
        let previous = self
            .registered_values
            .insert(key.clone(), RegisteredValueModel { value, lifecycle });
        if let Some(previous) = previous {
            self.errors.push(format!(
                "Duplicate registration for {key}, new={}, old={}",
                new_value, previous.value
            ));
        }
        self.get_or_create_holder(key);
    }

    fn lookup(
        &mut self,
        registry: &RegistryKeyModel,
        key: &ResourceKeyModel,
    ) -> Option<HolderModel> {
        if let Some(context_lookup) = self.context.registries.get(registry) {
            return context_lookup.get(key);
        }
        Some(self.get_or_create_holder(key.clone()))
    }

    fn lookup_tag(&mut self, registry: &RegistryKeyModel, tag: &str) -> Vec<HolderModel> {
        if let Some(context_lookup) = self.context.registries.get(registry) {
            return context_lookup.clone().empty_tags().get_tag(tag);
        }
        let _ = self.new_registries.contains(registry);
        Vec::new()
    }

    fn get_or_create_holder(&mut self, key: ResourceKeyModel) -> HolderModel {
        self.holders
            .entry(key.clone())
            .or_insert_with(|| HolderModel::stand_alone(self.owner, key))
            .clone()
    }

    fn report_unclaimed_registered_values(&mut self) {
        for (key, registered_value) in &self.registered_values {
            self.errors.push(format!(
                "Orpaned value {} for key {}",
                registered_value.value, key
            ));
        }
    }

    fn report_not_collected_holders(&mut self) {
        for key in self.holders.keys() {
            self.errors.push(format!("Unreferenced key: {key}"));
        }
    }

    fn throw_on_error(&self) -> Result<(), String> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Errors during registry creation: {}",
                self.errors.join(" | ")
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistrySetBuilderModel {
    entries: Vec<RegistryStubModel>,
}

impl RegistrySetBuilderModel {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add(
        mut self,
        key: RegistryKeyModel,
        lifecycle: LifecycleModel,
        bootstrap: Vec<BootstrapActionModel>,
    ) -> Self {
        self.entries.push(RegistryStubModel {
            key,
            lifecycle,
            bootstrap,
        });
        self
    }

    fn add_stable(self, key: RegistryKeyModel, bootstrap: Vec<BootstrapActionModel>) -> Self {
        self.add(key, LifecycleModel::stable(), bootstrap)
    }

    fn create_state(&self, context: RegistryAccessModel) -> BuildStateModel {
        let mut state =
            BuildStateModel::create(context, self.entries.iter().map(|entry| entry.key.clone()));
        for entry in &self.entries {
            entry.apply(&mut state);
        }
        state
    }

    fn build(&self, context: RegistryAccessModel) -> Result<ProviderModel, String> {
        let mut state = self.create_state(context.clone());
        let new_registries = self
            .entries
            .iter()
            .map(|stub| {
                stub.collect_registered_values(&mut state)
                    .build_as_lookup(state.owner)
            })
            .collect::<Vec<_>>();
        let result = build_provider_with_context(state.owner, &context, new_registries);
        state.report_not_collected_holders();
        state.report_unclaimed_registered_values();
        state.throw_on_error()?;
        Ok(result)
    }

    fn build_patch(
        &self,
        context: RegistryAccessModel,
        fallback_provider: ProviderModel,
        cloner_factory: ClonerFactoryModel,
    ) -> Result<PatchedRegistriesModel, String> {
        let mut state = self.create_state(context.clone());
        let mut new_registries = BTreeMap::new();
        for stub in &self.entries {
            let contents = stub.collect_registered_values(&mut state);
            new_registries.insert(contents.key.clone(), contents);
        }
        let context_registries = context.list_registry_keys();
        for key in fallback_provider.list_registry_keys() {
            if !context_registries.contains(&key) {
                new_registries
                    .entry(key.clone())
                    .or_insert(RegistryContentsModel {
                        key,
                        lifecycle: LifecycleModel::stable(),
                        values: BTreeMap::new(),
                    });
            }
        }
        let dynamic_registries = new_registries
            .values()
            .map(|contents| contents.build_as_lookup(state.owner))
            .collect::<Vec<_>>();
        let patch_only = build_provider_with_context(state.owner, &context, dynamic_registries);
        state.report_unclaimed_registered_values();
        state.throw_on_error()?;
        let full = create_lazy_full_patched_registries(
            &context,
            &fallback_provider,
            &cloner_factory,
            new_registries.keys().cloned().collect(),
            &patch_only,
        )?;
        Ok(PatchedRegistriesModel {
            full,
            patches: patch_only,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClonerFactoryModel {
    cloners: BTreeSet<RegistryKeyModel>,
}

impl ClonerFactoryModel {
    fn new(keys: impl IntoIterator<Item = RegistryKeyModel>) -> Self {
        Self {
            cloners: keys.into_iter().collect(),
        }
    }

    fn cloner(&self, key: &RegistryKeyModel) -> bool {
        self.cloners.contains(key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatchedRegistriesModel {
    full: ProviderModel,
    patches: ProviderModel,
}

fn build_provider_with_context(
    new_owner: usize,
    context: &RegistryAccessModel,
    new_registries: impl IntoIterator<Item = RegistryLookupModel>,
) -> ProviderModel {
    let mut lookups = BTreeMap::new();
    for context_registry in context.registries() {
        lookups.insert(context_registry.key.clone(), context_registry.empty_tags());
    }
    for mut new_registry in new_registries {
        new_registry.owner = new_owner;
        new_registry.tags_available = false;
        lookups.insert(new_registry.key.clone(), new_registry);
    }
    ProviderModel { lookups }
}

fn create_lazy_full_patched_registries(
    context: &RegistryAccessModel,
    fallback_provider: &ProviderModel,
    cloner_factory: &ClonerFactoryModel,
    registry_keys: Vec<RegistryKeyModel>,
    patch_provider: &ProviderModel,
) -> Result<ProviderModel, String> {
    let owner = 3;
    let mut lazy_full_registries = Vec::new();
    for registry_key in registry_keys {
        if !cloner_factory.cloner(&registry_key) {
            return Err(format!("No cloner for {}", registry_key.identifier()));
        }
        let patch_contents = patch_provider.lookup_or_throw(&registry_key)?;
        let fallback_contents = fallback_provider.lookup_or_throw(&registry_key)?;
        let mut entries = BTreeMap::new();
        for mut holder in patch_contents.list_elements() {
            let value = holder.value().to_string();
            entries.insert(
                holder.key.clone(),
                HolderModel::lazy(owner, holder.key.clone(), ProviderKind::Patch, &value),
            );
        }
        for mut holder in fallback_contents.list_elements() {
            let value = holder.value().to_string();
            entries.entry(holder.key.clone()).or_insert_with(|| {
                HolderModel::lazy(owner, holder.key.clone(), ProviderKind::Fallback, &value)
            });
        }
        lazy_full_registries.push(RegistryLookupModel {
            key: registry_key,
            lifecycle: patch_contents.lifecycle.add(fallback_contents.lifecycle),
            owner,
            entries,
            tags_available: false,
        });
    }
    Ok(build_provider_with_context(
        owner,
        context,
        lazy_full_registries,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry(id: &str) -> RegistryKeyModel {
        RegistryKeyModel::new(id)
    }

    fn key(registry: &RegistryKeyModel, location: &str) -> ResourceKeyModel {
        ResourceKeyModel::new(registry, location)
    }

    fn register(registry: &RegistryKeyModel, location: &str, value: &str) -> BootstrapActionModel {
        BootstrapActionModel::Register {
            key: key(registry, location),
            value: value.to_string(),
            lifecycle: LifecycleModel::stable(),
        }
    }

    fn must_ok<T, E: fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("expected Ok(..), got Err({error:?})"),
        }
    }

    fn must_err<T: fmt::Debug, E>(result: Result<T, E>) -> E {
        match result {
            Ok(value) => panic!("expected Err(..), got Ok({value:?})"),
            Err(error) => error,
        }
    }

    fn must_some<T>(option: Option<T>, label: &str) -> T {
        match option {
            Some(value) => value,
            None => panic!("{label}"),
        }
    }

    #[test]
    fn build_collects_registered_values_and_wraps_context_with_empty_tags() {
        let biomes = registry("minecraft:worldgen/biome");
        let damage_types = registry("minecraft:damage_type");
        let context = RegistryAccessModel::new([RegistryLookupModel::new(
            damage_types.clone(),
            LifecycleModel::experimental(),
            vec![("minecraft:lava", "lava")],
        )]);
        let provider = RegistrySetBuilderModel::new()
            .add_stable(
                biomes.clone(),
                vec![
                    BootstrapActionModel::LookupTag {
                        registry: damage_types.clone(),
                        tag: "minecraft:is_fire".to_string(),
                    },
                    register(&biomes, "minecraft:plains", "plains"),
                ],
            )
            .build(context);
        let provider = must_ok(provider);

        assert_eq!(
            provider.list_registry_keys(),
            vec![damage_types.clone(), biomes.clone()]
        );
        let biome_lookup = must_ok(provider.lookup_or_throw(&biomes));
        assert_eq!(biome_lookup.lifecycle, LifecycleModel::stable());
        assert_eq!(
            must_err(biome_lookup.list_tags()),
            "Tags are not available in datagen"
        );
        let mut plains = must_some(
            biome_lookup.get(&key(&biomes, "minecraft:plains")),
            "plains holder should exist",
        );
        assert_eq!(plains.owner, 2);
        assert_eq!(plains.value(), "plains");

        let damage_lookup = must_ok(provider.lookup_or_throw(&damage_types));
        assert_eq!(damage_lookup.lifecycle, LifecycleModel::experimental());
        assert_eq!(damage_lookup.get_tag("minecraft:any"), Vec::new());
        assert_eq!(
            must_err(damage_lookup.list_tags()),
            "Tags are not available in datagen"
        );
    }

    #[test]
    fn same_registry_lookup_holders_are_claimed_during_collection() {
        let biomes = registry("minecraft:worldgen/biome");
        let plains = key(&biomes, "minecraft:plains");
        let provider = RegistrySetBuilderModel::new()
            .add_stable(
                biomes.clone(),
                vec![
                    BootstrapActionModel::LookupElement {
                        registry: biomes.clone(),
                        key: plains.clone(),
                    },
                    register(&biomes, "minecraft:plains", "plains"),
                ],
            )
            .build(RegistryAccessModel::new([]));
        let provider = must_ok(provider);

        assert_eq!(
            must_some(
                must_ok(provider.lookup_or_throw(&biomes)).get(&plains),
                "plains holder should be collected"
            )
            .value,
            Some("plains".to_string())
        );
    }

    #[test]
    fn build_reports_duplicate_orphaned_and_unreferenced_values_like_java() {
        let biomes = registry("minecraft:worldgen/biome");
        let damage_types = registry("minecraft:damage_type");
        let err = RegistrySetBuilderModel::new()
            .add_stable(
                biomes.clone(),
                vec![
                    register(&biomes, "minecraft:plains", "first"),
                    register(&biomes, "minecraft:plains", "second"),
                    register(&damage_types, "minecraft:lava", "lava"),
                    BootstrapActionModel::LookupElement {
                        registry: biomes.clone(),
                        key: key(&biomes, "minecraft:desert"),
                    },
                ],
            )
            .build(RegistryAccessModel::new([]));
        let err = must_err(err);

        assert!(err.starts_with("Errors during registry creation: "));
        assert!(err.contains("Duplicate registration for ResourceKey[minecraft:worldgen/biome / minecraft:plains], new=second, old=first"));
        assert!(err.contains(
            "Orpaned value lava for key ResourceKey[minecraft:damage_type / minecraft:lava]"
        ));
        assert!(err.contains(
            "Unreferenced key: ResourceKey[minecraft:worldgen/biome / minecraft:desert]"
        ));
    }

    #[test]
    fn build_patch_skips_unreferenced_holders_and_exposes_patch_only_values() {
        let biomes = registry("minecraft:worldgen/biome");
        let fallback = ProviderModel::new([RegistryLookupModel::new(
            biomes.clone(),
            LifecycleModel::stable(),
            vec![("minecraft:forest", "old_forest")],
        )]);
        let patched = RegistrySetBuilderModel::new()
            .add(
                biomes.clone(),
                LifecycleModel::experimental(),
                vec![
                    BootstrapActionModel::LookupElement {
                        registry: biomes.clone(),
                        key: key(&biomes, "minecraft:unused_reference"),
                    },
                    register(&biomes, "minecraft:plains", "new_plains"),
                ],
            )
            .build_patch(
                RegistryAccessModel::new([]),
                fallback,
                ClonerFactoryModel::new([biomes.clone()]),
            );
        let patched = must_ok(patched);

        let patch_lookup = must_ok(patched.patches.lookup_or_throw(&biomes));
        assert!(patch_lookup
            .get(&key(&biomes, "minecraft:forest"))
            .is_none());
        assert!(patch_lookup
            .get(&key(&biomes, "minecraft:unused_reference"))
            .is_none());
        assert_eq!(patch_lookup.lifecycle, LifecycleModel::experimental());
    }

    #[test]
    fn full_patched_registries_lazily_clone_patch_values_before_fallback_values() {
        let biomes = registry("minecraft:worldgen/biome");
        let fallback = ProviderModel::new([RegistryLookupModel::new(
            biomes.clone(),
            LifecycleModel::stable(),
            vec![
                ("minecraft:plains", "old_plains"),
                ("minecraft:forest", "old_forest"),
            ],
        )]);
        let patched = RegistrySetBuilderModel::new()
            .add(
                biomes.clone(),
                LifecycleModel::experimental(),
                vec![register(&biomes, "minecraft:plains", "new_plains")],
            )
            .build_patch(
                RegistryAccessModel::new([]),
                fallback,
                ClonerFactoryModel::new([biomes.clone()]),
            );
        let mut patched = must_ok(patched);
        let full_lookup = must_some(
            patched.full.lookups.get_mut(&biomes),
            "full patched biome registry should exist",
        );

        assert_eq!(
            full_lookup.lifecycle,
            LifecycleModel::experimental().add(LifecycleModel::stable())
        );
        let mut plains = must_some(
            full_lookup.get(&key(&biomes, "minecraft:plains")),
            "patch value should override fallback",
        );
        let lazy = must_some(plains.lazy.clone(), "plains should be lazy");
        assert_eq!(lazy.calls.get(), 0);
        assert_eq!(plains.value(), "cloned:patch:new_plains");
        assert_eq!(plains.value(), "cloned:patch:new_plains");
        assert_eq!(lazy.calls.get(), 1);

        let mut forest = must_some(
            full_lookup.get(&key(&biomes, "minecraft:forest")),
            "fallback value should fill missing patch value",
        );
        assert_eq!(forest.value(), "cloned:fallback:old_forest");
    }

    #[test]
    fn build_patch_adds_fallback_only_registries_and_requires_cloners() {
        let biomes = registry("minecraft:worldgen/biome");
        let damage_types = registry("minecraft:damage_type");
        let fallback = ProviderModel::new([
            RegistryLookupModel::new(
                biomes.clone(),
                LifecycleModel::stable(),
                vec![("minecraft:forest", "old_forest")],
            ),
            RegistryLookupModel::new(
                damage_types.clone(),
                LifecycleModel::stable(),
                vec![("minecraft:lava", "lava")],
            ),
        ]);
        let err = RegistrySetBuilderModel::new()
            .add_stable(
                biomes.clone(),
                vec![register(&biomes, "minecraft:plains", "plains")],
            )
            .build_patch(
                RegistryAccessModel::new([]),
                fallback.clone(),
                ClonerFactoryModel::new([biomes.clone()]),
            );
        let err = must_err(err);
        assert_eq!(err, "No cloner for minecraft:damage_type");

        let patched = RegistrySetBuilderModel::new()
            .add_stable(
                biomes.clone(),
                vec![register(&biomes, "minecraft:plains", "plains")],
            )
            .build_patch(
                RegistryAccessModel::new([]),
                fallback,
                ClonerFactoryModel::new([biomes.clone(), damage_types.clone()]),
            );
        let patched = must_ok(patched);
        assert_eq!(
            patched.patches.list_registry_keys(),
            vec![damage_types.clone(), biomes.clone()]
        );
        assert!(must_ok(patched.patches.lookup_or_throw(&damage_types))
            .list_elements()
            .is_empty());
    }
}
