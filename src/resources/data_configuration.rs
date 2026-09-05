//! World pack selection and feature configuration, shared by resources and storage.
use super::*;

#[derive(Debug, Clone)]
pub struct DataPackConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    codec_default: bool,
}

impl DataPackConfig {
    /// Java `DataPackConfig(List<String>, List<String>)` takes immutable
    /// copies of both lists. Rust callers receive owned vectors, so this
    /// constructor performs the same boundary copy while retaining the
    /// existing field-based API used by repository selection.
    pub fn new<E, D, EI, DI>(enabled: EI, disabled: DI) -> Self
    where
        E: Into<String>,
        D: Into<String>,
        EI: IntoIterator<Item = E>,
        DI: IntoIterator<Item = D>,
    {
        Self {
            enabled: enabled.into_iter().map(Into::into).collect(),
            disabled: disabled.into_iter().map(Into::into).collect(),
            codec_default: false,
        }
    }

    pub fn default_26_1_2() -> Self {
        let mut config = Self::new([VANILLA_PACK_ID], std::iter::empty::<&str>());
        config.codec_default = true;
        config
    }

    pub fn from_properties(enabled: &str, disabled: &str) -> Self {
        Self::new(split_pack_list(enabled), split_pack_list(disabled))
    }

    pub fn enabled(&self) -> &[String] {
        &self.enabled
    }

    pub fn disabled(&self) -> &[String] {
        &self.disabled
    }

    /// Encodes Java `DataPackConfig.CODEC`'s exact field names and order.
    pub fn to_json(&self) -> Result<String, String> {
        let enabled = serde_json::to_string(&self.enabled).map_err(|error| error.to_string())?;
        let disabled = serde_json::to_string(&self.disabled).map_err(|error| error.to_string())?;
        Ok(format!(r#"{{"Enabled":{enabled},"Disabled":{disabled}}}"#))
    }

    /// Decodes the record codec shape used by level data and datapack
    /// configuration files. Unknown fields are ignored like Mojang's codec.
    pub fn from_json(raw: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|error| format!("invalid DataPackConfig JSON: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| "DataPackConfig must be a JSON object".to_string())?;
        let read_list = |name: &str| -> Result<Vec<String>, String> {
            object
                .get(name)
                .ok_or_else(|| format!("DataPackConfig missing {name}"))?
                .as_array()
                .ok_or_else(|| format!("DataPackConfig {name} must be an array"))?
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| format!("DataPackConfig {name} entries must be strings"))
                })
                .collect()
        };
        Ok(Self::new(read_list("Enabled")?, read_list("Disabled")?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDataConfiguration {
    pub data_packs: DataPackConfig,
    pub enabled_features: FeatureFlagSet,
}

impl WorldDataConfiguration {
    pub const ENABLED_FEATURES_ID: &'static str = "enabled_features";

    pub fn default_26_1_2() -> Self {
        Self {
            data_packs: DataPackConfig::default_26_1_2(),
            enabled_features: feature_flags::default_flags_26_1_2(),
        }
    }

    /// Java `WorldDataConfiguration#expandFeatures` joins the supplied flags
    /// without changing the selected datapacks.
    pub fn expand_features(&self, new_enabled_features: FeatureFlagSet) -> Self {
        Self {
            data_packs: self.data_packs.clone(),
            enabled_features: self.enabled_features.join(new_enabled_features),
        }
    }

    /// Encodes the Java record codec's optional `DataPacks` and
    /// `enabled_features` fields using the server's feature registry names.
    pub fn to_json(&self, registry: &FeatureFlagRegistry) -> Result<String, String> {
        let data_packs = self.data_packs.to_json()?;
        let features = registry
            .to_names(self.enabled_features)
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>();
        let mut fields = Vec::new();
        if !self.data_packs.is_codec_default() {
            fields.push(format!(r#""DataPacks":{data_packs}"#));
        }
        if self.enabled_features != feature_flags::default_flags_26_1_2() {
            let features = serde_json::to_string(&features).map_err(|error| error.to_string())?;
            fields.push(format!(r#""enabled_features":{features}"#));
        }
        Ok(format!("{{{}}}", fields.join(",")))
    }

    /// Decodes the Java record codec shape. Missing or malformed optional fields
    /// receive defaults independently, as lenientOptionalFieldOf specifies.
    pub fn from_json(raw: &str, registry: &FeatureFlagRegistry) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|error| format!("invalid WorldDataConfiguration JSON: {error}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| "WorldDataConfiguration must be a JSON object".to_string())?;
        let data_packs = object
            .get("DataPacks")
            .and_then(|value| DataPackConfig::from_json(&value.to_string()).ok())
            .unwrap_or_else(DataPackConfig::default_26_1_2);
        let read_features = || -> Result<FeatureFlagSet, String> {
            Ok(match object.get(Self::ENABLED_FEATURES_ID) {
                Some(value) => {
                    let values = value
                        .as_array()
                        .ok_or_else(|| "enabled_features must be an array".to_string())?;
                    let names = values
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .ok_or_else(|| {
                                    "enabled_features entries must be strings".to_string()
                                })
                                .and_then(Identifier::parse)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    registry
                        .resolve_names(&names)
                        .map_err(|unknown| format!("unknown feature flags: {unknown:?}"))?
                }
                None => feature_flags::default_flags_26_1_2(),
            })
        };
        let enabled_features =
            read_features().unwrap_or_else(|_| feature_flags::default_flags_26_1_2());
        Ok(Self {
            data_packs,
            enabled_features,
        })
    }
}

// Java's DataPackConfig has identity equality: DEFAULT can be omitted by the
// optional codec, while an explicit config with the same lists is still encoded.
// Rust value comparisons remain list-based; codec provenance is kept separately.
impl PartialEq for DataPackConfig {
    fn eq(&self, other: &Self) -> bool {
        self.enabled == other.enabled && self.disabled == other.disabled
    }
}
impl Eq for DataPackConfig {}
impl DataPackConfig {
    fn is_codec_default(&self) -> bool {
        self.codec_default && self.enabled == [VANILLA_PACK_ID] && self.disabled.is_empty()
    }
}
mod nbt;
#[cfg(test)]
mod tests;
