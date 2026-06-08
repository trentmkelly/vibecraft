use std::collections::BTreeMap;

use crate::registry::{feature_flags, FeatureFlagSet, Identifier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBuildContextModel {
    access: HolderLookupProviderModel,
    enabled_features: FeatureFlagSet,
}

impl CommandBuildContextModel {
    pub fn simple(access: HolderLookupProviderModel, enabled_features: FeatureFlagSet) -> Self {
        Self {
            access,
            enabled_features,
        }
    }

    pub fn list_registry_keys(&self) -> Vec<Identifier> {
        self.access.list_registry_keys()
    }

    pub fn lookup(&self, key: &Identifier) -> Option<RegistryLookupModel> {
        self.access
            .lookup(key)
            .map(|lookup| lookup.filter_features(self.enabled_features))
    }

    pub fn enabled_features(&self) -> FeatureFlagSet {
        self.enabled_features
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HolderLookupProviderModel {
    lookups: BTreeMap<Identifier, RegistryLookupModel>,
    registry_order: Vec<Identifier>,
}

impl HolderLookupProviderModel {
    pub fn create(lookups: impl IntoIterator<Item = RegistryLookupModel>) -> Self {
        let mut by_key = BTreeMap::new();
        let mut registry_order = Vec::new();
        for lookup in lookups {
            registry_order.push(lookup.key.clone());
            by_key.insert(lookup.key.clone(), lookup);
        }

        Self {
            lookups: by_key,
            registry_order,
        }
    }

    pub fn list_registry_keys(&self) -> Vec<Identifier> {
        self.registry_order.clone()
    }

    pub fn lookup(&self, key: &Identifier) -> Option<RegistryLookupModel> {
        self.lookups.get(key).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLookupModel {
    key: Identifier,
    elements: Vec<RegistryElementModel>,
    tags: Vec<Identifier>,
    filtered_delegate: bool,
}

impl RegistryLookupModel {
    pub fn new(
        key: Identifier,
        elements: impl IntoIterator<Item = RegistryElementModel>,
        tags: impl IntoIterator<Item = Identifier>,
    ) -> Self {
        Self {
            key,
            elements: elements.into_iter().collect(),
            tags: tags.into_iter().collect(),
            filtered_delegate: false,
        }
    }

    pub fn key(&self) -> &Identifier {
        &self.key
    }

    pub fn get(&self, id: &Identifier) -> Option<&RegistryElementModel> {
        self.elements.iter().find(|element| element.id() == id)
    }

    pub fn list_element_ids(&self) -> Vec<Identifier> {
        self.elements
            .iter()
            .map(|element| element.id().clone())
            .collect()
    }

    pub fn list_tag_ids(&self) -> Vec<Identifier> {
        self.tags.clone()
    }

    pub fn is_filtered_delegate(&self) -> bool {
        self.filtered_delegate
    }

    fn filter_features(self, enabled_features: FeatureFlagSet) -> Self {
        if !is_feature_filtered_registry(&self.key) {
            return self;
        }

        Self {
            key: self.key,
            elements: self
                .elements
                .into_iter()
                .filter(|element| element.is_enabled(enabled_features))
                .collect(),
            tags: self.tags,
            filtered_delegate: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryElementModel {
    id: Identifier,
    required_features: FeatureFlagSet,
}

impl RegistryElementModel {
    pub fn new(id: Identifier, required_features: FeatureFlagSet) -> Self {
        Self {
            id,
            required_features,
        }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn is_enabled(&self, enabled_features: FeatureFlagSet) -> bool {
        self.required_features.is_subset_of(enabled_features)
    }
}

fn is_feature_filtered_registry(key: &Identifier) -> bool {
    [
        "minecraft:item",
        "minecraft:block",
        "minecraft:entity_type",
        "minecraft:game_rule",
        "minecraft:menu",
        "minecraft:potion",
        "minecraft:mob_effect",
    ]
    .iter()
    .any(|registry| key == &id(registry))
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn flags(flags: &[crate::registry::FeatureFlag]) -> FeatureFlagSet {
    FeatureFlagSet::of(flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(value: &str, required_features: FeatureFlagSet) -> RegistryElementModel {
        RegistryElementModel::new(id(value), required_features)
    }

    fn provider() -> HolderLookupProviderModel {
        HolderLookupProviderModel::create([
            RegistryLookupModel::new(
                id("minecraft:item"),
                [
                    element("minecraft:stick", flags(&[feature_flags::VANILLA])),
                    element(
                        "minecraft:crafter",
                        flags(&[feature_flags::REDSTONE_EXPERIMENTS]),
                    ),
                ],
                [id("minecraft:tools"), id("minecraft:redstone")],
            ),
            RegistryLookupModel::new(
                id("minecraft:worldgen/biome"),
                [element(
                    "minecraft:experimental_biome",
                    flags(&[feature_flags::REDSTONE_EXPERIMENTS]),
                )],
                [id("minecraft:is_overworld")],
            ),
        ])
    }

    #[test]
    fn simple_delegates_registry_key_listing_and_exposes_enabled_features() {
        let context = CommandBuildContextModel::simple(
            provider(),
            flags(&[feature_flags::VANILLA, feature_flags::TRADE_REBALANCE]),
        );

        assert_eq!(
            context.list_registry_keys(),
            [id("minecraft:item"), id("minecraft:worldgen/biome")]
        );
        assert_eq!(
            context.enabled_features(),
            flags(&[feature_flags::VANILLA, feature_flags::TRADE_REBALANCE])
        );
    }

    #[test]
    fn lookup_returns_none_when_underlying_provider_has_no_registry() {
        let context =
            CommandBuildContextModel::simple(provider(), flags(&[feature_flags::VANILLA]));

        assert_eq!(context.lookup(&id("minecraft:unknown")), None);
    }

    #[test]
    fn lookup_filters_elements_for_java_feature_filtered_registries() {
        let context =
            CommandBuildContextModel::simple(provider(), flags(&[feature_flags::VANILLA]));
        let lookup = context.lookup(&id("minecraft:item")).unwrap();

        assert!(lookup.is_filtered_delegate());
        assert_eq!(lookup.key(), &id("minecraft:item"));
        assert_eq!(lookup.list_element_ids(), [id("minecraft:stick")]);
        assert!(lookup.get(&id("minecraft:stick")).is_some());
        assert_eq!(lookup.get(&id("minecraft:crafter")), None);
    }

    #[test]
    fn feature_filtering_keeps_tags_delegated_unchanged() {
        let context =
            CommandBuildContextModel::simple(provider(), flags(&[feature_flags::VANILLA]));
        let lookup = context.lookup(&id("minecraft:item")).unwrap();

        assert_eq!(
            lookup.list_tag_ids(),
            [id("minecraft:tools"), id("minecraft:redstone")]
        );
    }

    #[test]
    fn non_feature_filtered_registries_are_returned_without_element_filtering() {
        let context =
            CommandBuildContextModel::simple(provider(), flags(&[feature_flags::VANILLA]));
        let lookup = context.lookup(&id("minecraft:worldgen/biome")).unwrap();

        assert!(!lookup.is_filtered_delegate());
        assert_eq!(
            lookup.list_element_ids(),
            [id("minecraft:experimental_biome")]
        );
        assert!(lookup.get(&id("minecraft:experimental_biome")).is_some());
    }

    #[test]
    fn enabling_required_feature_makes_filtered_registry_element_visible() {
        let context = CommandBuildContextModel::simple(
            provider(),
            flags(&[feature_flags::VANILLA, feature_flags::REDSTONE_EXPERIMENTS]),
        );
        let lookup = context.lookup(&id("minecraft:item")).unwrap();

        assert_eq!(
            lookup.list_element_ids(),
            [id("minecraft:stick"), id("minecraft:crafter")]
        );
        assert_eq!(
            lookup
                .get(&id("minecraft:crafter"))
                .map(|element| element.id()),
            Some(&id("minecraft:crafter"))
        );
    }
}
