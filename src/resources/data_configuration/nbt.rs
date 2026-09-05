//! NbtOps forms of the same pack and feature codecs used by JSON resources.
use super::*;
use crate::storage::nbt::Tag;

impl DataPackConfig {
    pub fn to_nbt(&self) -> Tag {
        Tag::Compound(vec![
            ("Enabled".to_owned(), strings(&self.enabled)),
            ("Disabled".to_owned(), strings(&self.disabled)),
        ])
    }

    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let Tag::Compound(fields) = tag else {
            return Err("DataPacks must be a compound".to_owned());
        };
        let get = |key: &str| {
            fields
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value)
        };
        Ok(Self::new(
            string_list(get("Enabled"))?,
            string_list(get("Disabled"))?,
        ))
    }
}

impl WorldDataConfiguration {
    pub fn to_nbt(&self, registry: &FeatureFlagRegistry) -> Tag {
        let mut fields = Vec::new();
        if !self.data_packs.is_codec_default() {
            fields.push(("DataPacks".to_owned(), self.data_packs.to_nbt()));
        }
        if self.enabled_features != feature_flags::default_flags_26_1_2() {
            let names = registry
                .to_names(self.enabled_features)
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            fields.push((Self::ENABLED_FEATURES_ID.to_owned(), strings(&names)));
        }
        Tag::Compound(fields)
    }

    /// Lenient fields default independently; unknown feature IDs invalidate the
    /// complete feature field, rather than retaining the codec's partial set.
    pub fn from_nbt(tag: &Tag, registry: &FeatureFlagRegistry) -> Result<Self, String> {
        let Tag::Compound(fields) = tag else {
            return Err("world data configuration must be a compound".to_owned());
        };
        Ok(Self::from_nbt_fields(fields, registry))
    }

    pub(crate) fn from_nbt_fields(
        fields: &[(String, Tag)],
        registry: &FeatureFlagRegistry,
    ) -> Self {
        let get = |key: &str| {
            fields
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value)
        };
        let data_packs = get("DataPacks")
            .and_then(|tag| DataPackConfig::from_nbt(tag).ok())
            .unwrap_or_else(DataPackConfig::default_26_1_2);
        let enabled_features = string_list(get(Self::ENABLED_FEATURES_ID))
            .ok()
            .and_then(|names| {
                names
                    .iter()
                    .map(|name| Identifier::parse(name))
                    .collect::<Result<Vec<_>, _>>()
                    .ok()
            })
            .and_then(|names| registry.resolve_names(&names).ok())
            .unwrap_or_else(feature_flags::default_flags_26_1_2);
        Self {
            data_packs,
            enabled_features,
        }
    }
}

fn strings(values: &[String]) -> Tag {
    Tag::List(values.iter().cloned().map(Tag::String).collect())
}

fn string_list(tag: Option<&Tag>) -> Result<Vec<String>, String> {
    // NbtOps accepts an empty CollectionTag as a list, regardless of array type.
    match tag {
        Some(Tag::List(values)) => values
            .iter()
            .map(|value| match value {
                Tag::String(value) => Ok(value.clone()),
                _ => Err("list entries must be strings".to_owned()),
            })
            .collect(),
        Some(Tag::ByteArray(values)) if values.is_empty() => Ok(Vec::new()),
        Some(Tag::IntArray(values)) if values.is_empty() => Ok(Vec::new()),
        Some(Tag::LongArray(values)) if values.is_empty() => Ok(Vec::new()),
        _ => Err("expected a string list".to_owned()),
    }
}
