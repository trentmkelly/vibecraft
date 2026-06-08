use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
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
struct ElementCodecModel {
    name: &'static str,
}

impl ElementCodecModel {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HolderElementCodecModel {
    RegistryFile {
        registry_key: ResourceKeyModel,
        element_codec: ElementCodecModel,
        allow_inline: bool,
    },
    RegistryFixed {
        registry_key: ResourceKeyModel,
    },
}

impl HolderElementCodecModel {
    fn registry_key(&self) -> &ResourceKeyModel {
        match self {
            Self::RegistryFile { registry_key, .. } | Self::RegistryFixed { registry_key } => {
                registry_key
            }
        }
    }

    fn java_to_string(&self) -> String {
        match self {
            Self::RegistryFile {
                registry_key,
                element_codec,
                ..
            } => format!("RegistryFileCodec[{registry_key} {}]", element_codec.name),
            Self::RegistryFixed { registry_key } => format!("RegistryFixedCodec[{registry_key}]"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderSetCodecModel {
    registry_key: ResourceKeyModel,
    element_codec: HolderElementCodecModel,
    always_use_list: bool,
    homogenous_list_shape: HomogenousListShape,
    registry_aware_shape: RegistryAwareShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HomogenousListShape {
    CompactList,
    ListOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryAwareShape {
    TagOrHomogenousList,
}

impl HolderSetCodecModel {
    fn create(
        registry_key: ResourceKeyModel,
        element_codec: HolderElementCodecModel,
        always_use_list: bool,
    ) -> Self {
        assert_eq!(
            &registry_key,
            element_codec.registry_key(),
            "HolderSetCodec.create receives holder element codecs for the same registry"
        );
        Self {
            registry_key,
            element_codec,
            always_use_list,
            homogenous_list_shape: if always_use_list {
                HomogenousListShape::ListOnly
            } else {
                HomogenousListShape::CompactList
            },
            registry_aware_shape: RegistryAwareShape::TagOrHomogenousList,
        }
    }
}

fn registry_file_codec_create(
    registry_key: ResourceKeyModel,
    element_codec: ElementCodecModel,
) -> HolderElementCodecModel {
    HolderElementCodecModel::RegistryFile {
        registry_key,
        element_codec,
        allow_inline: true,
    }
}

fn registry_fixed_codec_create(registry_key: ResourceKeyModel) -> HolderElementCodecModel {
    HolderElementCodecModel::RegistryFixed { registry_key }
}

fn homogeneous_list_with_element_codec(
    registry_key: ResourceKeyModel,
    element_codec: ElementCodecModel,
) -> HolderSetCodecModel {
    homogeneous_list_with_element_codec_and_flag(registry_key, element_codec, false)
}

fn homogeneous_list_with_element_codec_and_flag(
    registry_key: ResourceKeyModel,
    element_codec: ElementCodecModel,
    always_use_list: bool,
) -> HolderSetCodecModel {
    HolderSetCodecModel::create(
        registry_key.clone(),
        registry_file_codec_create(registry_key, element_codec),
        always_use_list,
    )
}

fn homogeneous_list(registry_key: ResourceKeyModel) -> HolderSetCodecModel {
    homogeneous_list_with_flag(registry_key, false)
}

fn homogeneous_list_with_flag(
    registry_key: ResourceKeyModel,
    always_use_list: bool,
) -> HolderSetCodecModel {
    HolderSetCodecModel::create(
        registry_key.clone(),
        registry_fixed_codec_create(registry_key),
        always_use_list,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry_key() -> ResourceKeyModel {
        ResourceKeyModel::new("minecraft:worldgen/biome")
    }

    fn element_codec() -> ElementCodecModel {
        ElementCodecModel::new("Biome.DIRECT_CODEC")
    }

    #[test]
    fn element_codec_overload_uses_registry_file_codec_and_defaults_compact_list() {
        let codec = homogeneous_list_with_element_codec(registry_key(), element_codec());

        assert_eq!(codec.registry_key, registry_key());
        assert!(!codec.always_use_list);
        assert_eq!(
            codec.homogenous_list_shape,
            HomogenousListShape::CompactList
        );
        assert_eq!(
            codec.registry_aware_shape,
            RegistryAwareShape::TagOrHomogenousList
        );
        assert_eq!(
            codec.element_codec,
            HolderElementCodecModel::RegistryFile {
                registry_key: registry_key(),
                element_codec: element_codec(),
                allow_inline: true,
            }
        );
        assert_eq!(
            codec.element_codec.java_to_string(),
            "RegistryFileCodec[minecraft:worldgen/biome Biome.DIRECT_CODEC]"
        );
    }

    #[test]
    fn element_codec_boolean_overload_forwards_always_use_list_to_holder_set_codec() {
        let compact =
            homogeneous_list_with_element_codec_and_flag(registry_key(), element_codec(), false);
        let list_only =
            homogeneous_list_with_element_codec_and_flag(registry_key(), element_codec(), true);

        assert_eq!(
            compact.homogenous_list_shape,
            HomogenousListShape::CompactList
        );
        assert_eq!(
            list_only.homogenous_list_shape,
            HomogenousListShape::ListOnly
        );
        assert!(!compact.always_use_list);
        assert!(list_only.always_use_list);
        assert!(matches!(
            list_only.element_codec,
            HolderElementCodecModel::RegistryFile {
                allow_inline: true,
                ..
            }
        ));
    }

    #[test]
    fn fixed_registry_overload_uses_registry_fixed_codec_and_defaults_compact_list() {
        let codec = homogeneous_list(registry_key());

        assert_eq!(codec.registry_key, registry_key());
        assert!(!codec.always_use_list);
        assert_eq!(
            codec.homogenous_list_shape,
            HomogenousListShape::CompactList
        );
        assert_eq!(
            codec.element_codec,
            HolderElementCodecModel::RegistryFixed {
                registry_key: registry_key()
            }
        );
        assert_eq!(
            codec.element_codec.java_to_string(),
            "RegistryFixedCodec[minecraft:worldgen/biome]"
        );
    }

    #[test]
    fn fixed_registry_boolean_overload_forwards_always_use_list_to_holder_set_codec() {
        let compact = homogeneous_list_with_flag(registry_key(), false);
        let list_only = homogeneous_list_with_flag(registry_key(), true);

        assert_eq!(
            compact.homogenous_list_shape,
            HomogenousListShape::CompactList
        );
        assert_eq!(
            list_only.homogenous_list_shape,
            HomogenousListShape::ListOnly
        );
        assert!(!compact.always_use_list);
        assert!(list_only.always_use_list);
        assert!(matches!(
            list_only.element_codec,
            HolderElementCodecModel::RegistryFixed { .. }
        ));
    }
}
