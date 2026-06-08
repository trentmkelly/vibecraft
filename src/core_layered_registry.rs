use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResourceKeyModel(String);

impl ResourceKeyModel {
    fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for ResourceKeyModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryModel {
    key: ResourceKeyModel,
    entries: Vec<&'static str>,
    frozen: bool,
}

impl RegistryModel {
    fn new(key: &str, entries: impl Into<Vec<&'static str>>) -> Self {
        Self {
            key: ResourceKeyModel::new(key),
            entries: entries.into(),
            frozen: false,
        }
    }

    fn freeze(&self) -> Self {
        let mut frozen = self.clone();
        frozen.frozen = true;
        frozen
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryEntryModel {
    key: ResourceKeyModel,
    value: RegistryModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryAccessModel {
    registries: BTreeMap<ResourceKeyModel, RegistryModel>,
    frozen: bool,
}

impl RegistryAccessModel {
    fn empty() -> Self {
        Self {
            registries: BTreeMap::new(),
            frozen: true,
        }
    }

    fn from_registries(
        registries: impl IntoIterator<Item = RegistryModel>,
    ) -> Result<Self, String> {
        let mut by_key = BTreeMap::new();
        for registry in registries {
            if by_key.insert(registry.key.clone(), registry).is_some() {
                return Err("Duplicate key in immutable registry access".to_string());
            }
        }
        Ok(Self {
            registries: by_key,
            frozen: false,
        })
    }

    fn from_map(registries: BTreeMap<ResourceKeyModel, RegistryModel>) -> Self {
        Self {
            registries,
            frozen: false,
        }
    }

    fn freeze(&self) -> Self {
        Self {
            registries: self
                .registries
                .iter()
                .map(|(key, registry)| (key.clone(), registry.freeze()))
                .collect(),
            frozen: true,
        }
    }

    fn lookup(&self, key: &ResourceKeyModel) -> Option<&RegistryModel> {
        self.registries.get(key)
    }

    fn lookup_or_throw(&self, key: &ResourceKeyModel) -> Result<&RegistryModel, String> {
        self.lookup(key)
            .ok_or_else(|| format!("Missing registry: {key}"))
    }

    fn registries(&self) -> Vec<RegistryEntryModel> {
        self.registries
            .iter()
            .map(|(key, value)| RegistryEntryModel {
                key: key.clone(),
                value: value.clone(),
            })
            .collect()
    }

    fn list_registry_keys(&self) -> Vec<ResourceKeyModel> {
        self.registries.keys().cloned().collect()
    }

    fn is_frozen(&self) -> bool {
        self.frozen && self.registries.values().all(|registry| registry.frozen)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryLayerModel {
    Static,
    Worldgen,
    Dimensions,
    Reloadable,
}

impl fmt::Display for RegistryLayerModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Static => "STATIC",
            Self::Worldgen => "WORLDGEN",
            Self::Dimensions => "DIMENSIONS",
            Self::Reloadable => "RELOADABLE",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LayeredRegistryAccessModel<T> {
    keys: Vec<T>,
    values: Vec<RegistryAccessModel>,
    composite: RegistryAccessModel,
}

impl<T> LayeredRegistryAccessModel<T>
where
    T: Clone + PartialEq + fmt::Display,
{
    fn new(keys: Vec<T>) -> Result<Self, String> {
        let values = vec![RegistryAccessModel::empty(); keys.len()];
        Self::with_values(keys, values)
    }

    fn with_values(keys: Vec<T>, values: Vec<RegistryAccessModel>) -> Result<Self, String> {
        let composite = RegistryAccessModel::from_map(collect_registries(values.iter())?).freeze();
        Ok(Self {
            keys,
            values,
            composite,
        })
    }

    fn get_layer_index_or_throw(&self, layer: T) -> Result<usize, String> {
        self.keys
            .iter()
            .position(|key| *key == layer)
            .ok_or_else(|| {
                format!(
                    "Can't find {layer} inside [{}]",
                    self.keys
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }

    fn get_layer(&self, layer: T) -> Result<&RegistryAccessModel, String> {
        let index = self.get_layer_index_or_throw(layer)?;
        Ok(&self.values[index])
    }

    fn get_access_for_loading(&self, for_layer: T) -> Result<RegistryAccessModel, String> {
        let index = self.get_layer_index_or_throw(for_layer)?;
        self.get_composite_access_for_layers(0, index)
    }

    fn get_access_from(&self, for_layer: T) -> Result<RegistryAccessModel, String> {
        let index = self.get_layer_index_or_throw(for_layer)?;
        self.get_composite_access_for_layers(index, self.values.len())
    }

    fn get_composite_access_for_layers(
        &self,
        from: usize,
        to: usize,
    ) -> Result<RegistryAccessModel, String> {
        Ok(
            RegistryAccessModel::from_map(collect_registries(self.values[from..to].iter())?)
                .freeze(),
        )
    }

    fn replace_from(
        &self,
        from_layer: T,
        layers: Vec<RegistryAccessModel>,
    ) -> Result<Self, String> {
        let index = self.get_layer_index_or_throw(from_layer)?;
        if layers.len() > self.values.len() - index {
            return Err("Too many values to replace".to_string());
        }

        let mut new_values = Vec::new();
        new_values.extend(self.values[..index].iter().cloned());
        new_values.extend(layers);
        while new_values.len() < self.values.len() {
            new_values.push(RegistryAccessModel::empty());
        }
        Self::with_values(self.keys.clone(), new_values)
    }

    fn composite_access(&self) -> &RegistryAccessModel {
        &self.composite
    }
}

fn collect_registries<'a>(
    registries: impl IntoIterator<Item = &'a RegistryAccessModel>,
) -> Result<BTreeMap<ResourceKeyModel, RegistryModel>, String> {
    let mut result = BTreeMap::new();
    for access in registries {
        for entry in access.registries() {
            if result.insert(entry.key.clone(), entry.value).is_some() {
                return Err(format!("Duplicated registry {}", entry.key));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn access(registries: impl IntoIterator<Item = RegistryModel>) -> RegistryAccessModel {
        RegistryAccessModel::from_registries(registries)
            .unwrap()
            .freeze()
    }

    fn layered() -> LayeredRegistryAccessModel<RegistryLayerModel> {
        LayeredRegistryAccessModel::new(vec![
            RegistryLayerModel::Static,
            RegistryLayerModel::Worldgen,
            RegistryLayerModel::Dimensions,
            RegistryLayerModel::Reloadable,
        ])
        .unwrap()
    }

    #[test]
    fn new_layered_access_starts_with_empty_frozen_layers_and_empty_composite() {
        let layered = layered();
        for layer in [
            RegistryLayerModel::Static,
            RegistryLayerModel::Worldgen,
            RegistryLayerModel::Dimensions,
            RegistryLayerModel::Reloadable,
        ] {
            assert!(layered.get_layer(layer).unwrap().is_frozen());
            assert!(layered.get_layer(layer).unwrap().registries().is_empty());
        }
        assert!(layered.composite_access().is_frozen());
        assert!(layered.composite_access().registries().is_empty());
    }

    #[test]
    fn replace_from_preserves_earlier_layers_replaces_supplied_layers_and_fills_tail_empty() {
        let static_access = access([RegistryModel::new("minecraft:root", ["registry"])]);
        let worldgen_access = access([RegistryModel::new("minecraft:biome", ["plains"])]);
        let dimension_access = access([RegistryModel::new(
            "minecraft:dimension_type",
            ["overworld"],
        )]);

        let with_static = layered()
            .replace_from(RegistryLayerModel::Static, vec![static_access.clone()])
            .unwrap();
        let replaced = with_static
            .replace_from(
                RegistryLayerModel::Worldgen,
                vec![worldgen_access.clone(), dimension_access.clone()],
            )
            .unwrap();

        assert_eq!(
            replaced.get_layer(RegistryLayerModel::Static).unwrap(),
            &static_access
        );
        assert_eq!(
            replaced.get_layer(RegistryLayerModel::Worldgen).unwrap(),
            &worldgen_access
        );
        assert_eq!(
            replaced.get_layer(RegistryLayerModel::Dimensions).unwrap(),
            &dimension_access
        );
        assert!(replaced
            .get_layer(RegistryLayerModel::Reloadable)
            .unwrap()
            .registries()
            .is_empty());
        assert!(
            with_static
                .get_layer(RegistryLayerModel::Worldgen)
                .unwrap()
                .registries()
                .is_empty(),
            "replaceFrom returns a new LayeredRegistryAccess instead of mutating the old one"
        );
    }

    #[test]
    fn access_for_loading_excludes_target_layer_and_access_from_includes_target_layer() {
        let replaced = layered()
            .replace_from(
                RegistryLayerModel::Static,
                vec![
                    access([RegistryModel::new("minecraft:root", ["registry"])]),
                    access([RegistryModel::new("minecraft:biome", ["plains"])]),
                    access([RegistryModel::new(
                        "minecraft:dimension_type",
                        ["overworld"],
                    )]),
                    access([RegistryModel::new(
                        "minecraft:loot_table",
                        ["chests/simple"],
                    )]),
                ],
            )
            .unwrap();

        let loading_dimensions = replaced
            .get_access_for_loading(RegistryLayerModel::Dimensions)
            .unwrap();
        assert_eq!(
            loading_dimensions.list_registry_keys(),
            vec![
                ResourceKeyModel::new("minecraft:biome"),
                ResourceKeyModel::new("minecraft:root"),
            ]
        );
        assert!(loading_dimensions
            .lookup(&ResourceKeyModel::new("minecraft:dimension_type"))
            .is_none());

        let from_dimensions = replaced
            .get_access_from(RegistryLayerModel::Dimensions)
            .unwrap();
        assert_eq!(
            from_dimensions.list_registry_keys(),
            vec![
                ResourceKeyModel::new("minecraft:dimension_type"),
                ResourceKeyModel::new("minecraft:loot_table"),
            ]
        );
        assert!(from_dimensions
            .lookup(&ResourceKeyModel::new("minecraft:biome"))
            .is_none());
    }

    #[test]
    fn composite_access_collects_all_layers_and_freezes_registries() {
        let replaced = layered()
            .replace_from(
                RegistryLayerModel::Static,
                vec![
                    access([RegistryModel::new("minecraft:root", ["registry"])]),
                    access([RegistryModel::new("minecraft:biome", ["plains"])]),
                ],
            )
            .unwrap();
        let composite = replaced.composite_access();

        assert!(composite.is_frozen());
        assert_eq!(
            composite
                .lookup_or_throw(&ResourceKeyModel::new("minecraft:root"))
                .unwrap()
                .entries,
            vec!["registry"]
        );
        assert_eq!(
            composite
                .lookup_or_throw(&ResourceKeyModel::new("minecraft:biome"))
                .unwrap()
                .entries,
            vec!["plains"]
        );
        assert_eq!(
            composite
                .lookup_or_throw(&ResourceKeyModel::new("minecraft:missing"))
                .unwrap_err(),
            "Missing registry: minecraft:missing"
        );
    }

    #[test]
    fn invalid_layer_and_too_many_replacements_match_java_errors() {
        let layered = layered();
        assert_eq!(
            layered
                .get_layer_index_or_throw(RegistryLayerModel::Reloadable)
                .unwrap(),
            3
        );
        assert_eq!(
            LayeredRegistryAccessModel::new(vec![RegistryLayerModel::Static])
                .unwrap()
                .get_layer_index_or_throw(RegistryLayerModel::Worldgen)
                .unwrap_err(),
            "Can't find WORLDGEN inside [STATIC]"
        );
        assert_eq!(
            layered
                .replace_from(
                    RegistryLayerModel::Dimensions,
                    vec![
                        RegistryAccessModel::empty(),
                        RegistryAccessModel::empty(),
                        RegistryAccessModel::empty(),
                    ],
                )
                .unwrap_err(),
            "Too many values to replace"
        );
    }

    #[test]
    fn duplicate_registry_keys_across_collected_layers_are_rejected() {
        let duplicate_a = access([RegistryModel::new("minecraft:biome", ["plains"])]);
        let duplicate_b = access([RegistryModel::new("minecraft:biome", ["desert"])]);
        assert_eq!(
            layered()
                .replace_from(RegistryLayerModel::Worldgen, vec![duplicate_a, duplicate_b])
                .unwrap_err(),
            "Duplicated registry minecraft:biome"
        );
    }
}
