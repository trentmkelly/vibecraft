#![allow(dead_code)]

use crate::registry::FeatureFlagSet;
use crate::resources::{PackFormat, PackFormatRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackTypeModel {
    ClientResources,
    ServerData,
}

impl PackTypeModel {
    pub fn directory(self) -> &'static str {
        match self {
            Self::ClientResources => "assets",
            Self::ServerData => "data",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackPositionModel {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackSelectionConfigModel {
    pub required: bool,
    pub default_position: PackPositionModel,
    pub fixed_position: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSectionKind {
    Features,
    OverlaysClient,
    OverlaysServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataSectionTypeModel {
    pub kind: MetadataSectionKind,
    pub name: &'static str,
    pub codec: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataSectionWithValue<T> {
    section_type: MetadataSectionTypeModel,
    value: T,
}

impl MetadataSectionTypeModel {
    pub fn with_value<T>(self, value: T) -> MetadataSectionWithValue<T> {
        MetadataSectionWithValue {
            section_type: self,
            value,
        }
    }
}

impl<T: Clone> MetadataSectionWithValue<T> {
    pub fn unwrap_to_type(&self, section_type: MetadataSectionTypeModel) -> Option<T> {
        if section_type.kind == self.section_type.kind {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

pub const FEATURE_FLAGS_METADATA_TYPE: MetadataSectionTypeModel = MetadataSectionTypeModel {
    kind: MetadataSectionKind::Features,
    name: "features",
    codec: "FeatureFlags.CODEC.fieldOf(\"enabled\")",
};

pub const OVERLAY_CLIENT_METADATA_TYPE: MetadataSectionTypeModel = MetadataSectionTypeModel {
    kind: MetadataSectionKind::OverlaysClient,
    name: "overlays",
    codec: "OverlayEntry.listCodecForPackType(PackType.CLIENT_RESOURCES).fieldOf(\"entries\")",
};

pub const OVERLAY_SERVER_METADATA_TYPE: MetadataSectionTypeModel = MetadataSectionTypeModel {
    kind: MetadataSectionKind::OverlaysServer,
    name: "overlays",
    codec: "OverlayEntry.listCodecForPackType(PackType.SERVER_DATA).fieldOf(\"entries\")",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureFlagsMetadataSectionModel {
    pub flags: FeatureFlagSet,
}

impl FeatureFlagsMetadataSectionModel {
    pub fn section_type() -> MetadataSectionTypeModel {
        FEATURE_FLAGS_METADATA_TYPE
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayMetadataSectionModel {
    overlays: Vec<OverlayEntryModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayEntryModel {
    pub format: PackFormatRange,
    pub overlay: String,
}

impl OverlayMetadataSectionModel {
    pub fn new(overlays: impl IntoIterator<Item = OverlayEntryModel>) -> Self {
        Self {
            overlays: overlays.into_iter().collect(),
        }
    }

    pub fn section_type_for_pack_type(pack_type: PackTypeModel) -> MetadataSectionTypeModel {
        match pack_type {
            PackTypeModel::ClientResources => OVERLAY_CLIENT_METADATA_TYPE,
            PackTypeModel::ServerData => OVERLAY_SERVER_METADATA_TYPE,
        }
    }

    pub fn overlays_for_version(&self, version: PackFormat) -> Vec<String> {
        self.overlays
            .iter()
            .filter(|entry| entry.is_applicable(version))
            .map(|entry| entry.overlay.clone())
            .collect()
    }

    pub fn overlays(&self) -> &[OverlayEntryModel] {
        &self.overlays
    }
}

impl OverlayEntryModel {
    pub fn new(format: PackFormatRange, overlay: impl Into<String>) -> Result<Self, String> {
        let overlay = validate_overlay_directory(overlay.into())?;
        Ok(Self { format, overlay })
    }

    pub fn is_applicable(&self, format_to_test: PackFormat) -> bool {
        self.format.min <= format_to_test && format_to_test <= self.format.max
    }
}

pub fn validate_overlay_directory(path: String) -> Result<String, String> {
    if !path.is_empty()
        && path
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        Ok(path)
    } else {
        Err(format!("{path} is not accepted directory name"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::feature_flags;

    const FEATURE_FLAGS_METADATA_SECTION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/FeatureFlagsMetadataSection.java");
    const OVERLAY_METADATA_SECTION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/OverlayMetadataSection.java");
    const PACK_SELECTION_CONFIG_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/PackSelectionConfig.java");
    const PACK_TYPE_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/packs/PackType.java");
    const METADATA_SECTION_TYPE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/metadata/MetadataSectionType.java");

    #[test]
    fn pack_type_directories_match_java() {
        assert_eq!(PackTypeModel::ClientResources.directory(), "assets");
        assert_eq!(PackTypeModel::ServerData.directory(), "data");
    }

    #[test]
    fn metadata_section_with_value_uses_java_reference_identity_semantics() {
        let section = FEATURE_FLAGS_METADATA_TYPE.with_value("payload");

        assert_eq!(
            section.unwrap_to_type(FEATURE_FLAGS_METADATA_TYPE),
            Some("payload")
        );
        assert_eq!(section.unwrap_to_type(OVERLAY_SERVER_METADATA_TYPE), None);

        assert_eq!(FEATURE_FLAGS_METADATA_TYPE.name, "features");
        assert_eq!(OVERLAY_CLIENT_METADATA_TYPE.name, "overlays");
        assert_eq!(OVERLAY_SERVER_METADATA_TYPE.name, "overlays");
        assert_ne!(
            OVERLAY_CLIENT_METADATA_TYPE.kind,
            OVERLAY_SERVER_METADATA_TYPE.kind
        );
    }

    #[test]
    fn feature_flags_metadata_and_selection_config_preserve_record_fields() {
        let flags = FeatureFlagSet::of(&[feature_flags::VANILLA, feature_flags::TRADE_REBALANCE]);
        let metadata = FeatureFlagsMetadataSectionModel { flags };
        let config = PackSelectionConfigModel {
            required: true,
            default_position: PackPositionModel::Bottom,
            fixed_position: false,
        };

        assert_eq!(FeatureFlagsMetadataSectionModel::section_type().name, "features");
        assert!(metadata.flags.contains(feature_flags::VANILLA));
        assert!(metadata.flags.contains(feature_flags::TRADE_REBALANCE));
        assert_eq!(
            config,
            PackSelectionConfigModel {
                required: true,
                default_position: PackPositionModel::Bottom,
                fixed_position: false,
            }
        );
    }

    #[test]
    fn overlay_metadata_validates_directories_and_filters_versions_like_java() {
        for valid in ["base", "modded.assets_1-20", ".", "A_Z-9"] {
            assert_eq!(validate_overlay_directory(valid.to_string()), Ok(valid.to_string()));
        }

        for invalid in ["", "nested/path", "with space", "accented_é"] {
            assert_eq!(
                validate_overlay_directory(invalid.to_string()),
                Err(format!("{invalid} is not accepted directory name"))
            );
        }

        let section = OverlayMetadataSectionModel::new([
            entry(81, 0, 82, u32::MAX, "legacy"),
            entry(83, 0, 83, 4, "current"),
            entry(83, 5, 84, 0, "future"),
        ]);

        assert_eq!(
            section.overlays_for_version(PackFormat {
                major: 83,
                minor: 4
            }),
            vec!["current".to_string()]
        );
        assert_eq!(
            section.overlays_for_version(PackFormat {
                major: 84,
                minor: 0
            }),
            vec!["future".to_string()]
        );
        assert_eq!(
            section.overlays_for_version(PackFormat {
                major: 81,
                minor: 2
            }),
            vec!["legacy".to_string()]
        );
    }

    #[test]
    fn overlay_section_types_are_selected_per_pack_type_like_java() {
        assert_eq!(
            OverlayMetadataSectionModel::section_type_for_pack_type(PackTypeModel::ClientResources),
            OVERLAY_CLIENT_METADATA_TYPE
        );
        assert_eq!(
            OverlayMetadataSectionModel::section_type_for_pack_type(PackTypeModel::ServerData),
            OVERLAY_SERVER_METADATA_TYPE
        );
    }

    #[test]
    fn pack_metadata_sources_match_java_26_1_2() {
        for needle in [
            "FeatureFlags.CODEC.fieldOf(\"enabled\")",
            "public static final MetadataSectionType<FeatureFlagsMetadataSection> TYPE = new MetadataSectionType<>(\"features\", CODEC)",
        ] {
            assert!(FEATURE_FLAGS_METADATA_SECTION_JAVA.contains(needle));
        }

        for needle in [
            "Pattern.compile(\"[-_a-zA-Z0-9.]+\")",
            "public static final MetadataSectionType<OverlayMetadataSection> CLIENT_TYPE = new MetadataSectionType<>(",
            "\"overlays\", codecForPackType(PackType.CLIENT_RESOURCES)",
            "new MetadataSectionType<>(\"overlays\", codecForPackType(PackType.SERVER_DATA))",
            "case CLIENT_RESOURCES -> CLIENT_TYPE",
            "case SERVER_DATA -> SERVER_TYPE",
            "this.overlays.stream().filter(entry -> entry.isApplicable(version)).map(OverlayMetadataSection.OverlayEntry::overlay).toList()",
            "return this.format.isValueInRange(formatToTest);",
            "fieldOf(\"directory\")",
        ] {
            assert!(OVERLAY_METADATA_SECTION_JAVA.contains(needle));
        }

        assert!(PACK_SELECTION_CONFIG_JAVA.contains(
            "public record PackSelectionConfig(boolean required, Pack.Position defaultPosition, boolean fixedPosition)"
        ));

        for needle in [
            "CLIENT_RESOURCES(\"assets\")",
            "SERVER_DATA(\"data\")",
            "public String getDirectory()",
        ] {
            assert!(PACK_TYPE_JAVA.contains(needle));
        }

        for needle in [
            "public record MetadataSectionType<T>(String name, Codec<T> codec)",
            "return new MetadataSectionType.WithValue<>(this, value);",
            "return type == this.type ? Optional.of(this.value) : Optional.empty();",
        ] {
            assert!(METADATA_SECTION_TYPE_JAVA.contains(needle));
        }
    }

    fn entry(
        min_major: u32,
        min_minor: u32,
        max_major: u32,
        max_minor: u32,
        overlay: &str,
    ) -> OverlayEntryModel {
        match OverlayEntryModel::new(
            PackFormatRange {
                min: PackFormat {
                    major: min_major,
                    minor: min_minor,
                },
                max: PackFormat {
                    major: max_major,
                    minor: max_minor,
                },
            },
            overlay,
        ) {
            Ok(entry) => entry,
            Err(error) => panic!("valid test overlay failed validation: {error}"),
        }
    }
}
