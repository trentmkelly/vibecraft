#![allow(dead_code)]

use crate::chat_component::component_utils::wrap_in_square_brackets;
use crate::chat_component::{Component, ComponentArgument, HoverEvent, Style, TextColor};
use crate::chat_formatting::ChatFormatting;
use crate::network::configuration::KnownPack;
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
pub enum PackSourceModel {
    Default,
    BuiltIn,
    Feature,
    World,
    Server,
}

impl PackSourceModel {
    pub fn decorate(self, pack_description: Component) -> Component {
        match self.description_id() {
            Some(description_id) => Component::translatable(
                "pack.nameAndSource",
                vec![
                    ComponentArgument::Component(Box::new(pack_description)),
                    ComponentArgument::Component(Box::new(Component::translatable(
                        description_id,
                        Vec::new(),
                    ))),
                ],
            )
            .styled(Style::empty().with_legacy_color(Some(ChatFormatting::Gray))),
            None => pack_description,
        }
    }

    pub fn should_add_automatically(self) -> bool {
        !matches!(self, Self::Feature)
    }

    fn description_id(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::BuiltIn => Some("pack.source.builtin"),
            Self::Feature => Some("pack.source.feature"),
            Self::World => Some("pack.source.world"),
            Self::Server => Some("pack.source.server"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackLocationInfoModel {
    pub id: String,
    pub title: Component,
    pub source: PackSourceModel,
    pub known_pack_info: Option<KnownPack>,
}

impl PackLocationInfoModel {
    pub fn new(
        id: impl Into<String>,
        title: Component,
        source: PackSourceModel,
        known_pack_info: Option<KnownPack>,
    ) -> Self {
        Self {
            id: id.into(),
            title,
            source,
            known_pack_info,
        }
    }

    pub fn create_chat_link(&self, enabled: bool, description: Component) -> Component {
        wrap_in_square_brackets(self.source.decorate(Component::literal(&self.id))).styled(
            Style::empty()
                .with_color(link_color(enabled))
                .with_insertion(escape_if_required(&self.id))
                .with_hover_event(HoverEvent::Text(Box::new(
                    Component::empty()
                        .append(self.title.clone())
                        .append(Component::literal("\n"))
                        .append(description),
                ))),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSectionKind {
    Features,
    OverlaysClient,
    OverlaysServer,
    PackClient,
    PackServer,
    PackFallback,
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

pub const PACK_CLIENT_METADATA_TYPE: MetadataSectionTypeModel = MetadataSectionTypeModel {
    kind: MetadataSectionKind::PackClient,
    name: "pack",
    codec: "PackMetadataSection.codecForPackType(PackType.CLIENT_RESOURCES)",
};

pub const PACK_SERVER_METADATA_TYPE: MetadataSectionTypeModel = MetadataSectionTypeModel {
    kind: MetadataSectionKind::PackServer,
    name: "pack",
    codec: "PackMetadataSection.codecForPackType(PackType.SERVER_DATA)",
};

pub const PACK_FALLBACK_METADATA_TYPE: MetadataSectionTypeModel = MetadataSectionTypeModel {
    kind: MetadataSectionKind::PackFallback,
    name: "pack",
    codec: "PackMetadataSection.FALLBACK_CODEC",
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
pub struct PackMetadataSectionModel {
    pub description: Component,
    pub supported_formats: PackFormatRange,
}

impl PackMetadataSectionModel {
    pub fn new(description: Component, supported_formats: PackFormatRange) -> Self {
        Self {
            description,
            supported_formats,
        }
    }

    pub fn fallback(description: Component) -> Self {
        let unknown = PackFormat {
            major: u32::MAX,
            minor: 0,
        };
        Self::new(
            description,
            PackFormatRange {
                min: unknown,
                max: unknown,
            },
        )
    }

    pub fn section_type_for_pack_type(pack_type: PackTypeModel) -> MetadataSectionTypeModel {
        match pack_type {
            PackTypeModel::ClientResources => PACK_CLIENT_METADATA_TYPE,
            PackTypeModel::ServerData => PACK_SERVER_METADATA_TYPE,
        }
    }

    pub fn fallback_section_type() -> MetadataSectionTypeModel {
        PACK_FALLBACK_METADATA_TYPE
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

fn link_color(enabled: bool) -> TextColor {
    if enabled {
        legacy_color(ChatFormatting::Green)
    } else {
        legacy_color(ChatFormatting::Red)
    }
}

fn legacy_color(format: ChatFormatting) -> TextColor {
    match TextColor::from_legacy_format(format) {
        Some(color) => color,
        None => unreachable!("pack location uses only Java color formatting values"),
    }
}

fn escape_if_required(input: &str) -> String {
    if !input.is_empty() && input.chars().all(is_allowed_in_unquoted_string) {
        input.to_string()
    } else {
        let mut escaped = String::with_capacity(input.len() + 2);
        escaped.push('"');
        for ch in input.chars() {
            if matches!(ch, '\\' | '"') {
                escaped.push('\\');
            }
            escaped.push(ch);
        }
        escaped.push('"');
        escaped
    }
}

fn is_allowed_in_unquoted_string(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '+')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::ComponentContent;
    use crate::registry::feature_flags;

    const FEATURE_FLAGS_METADATA_SECTION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/FeatureFlagsMetadataSection.java");
    const OVERLAY_METADATA_SECTION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/OverlayMetadataSection.java");
    const PACK_LOCATION_INFO_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/PackLocationInfo.java");
    const PACK_SELECTION_CONFIG_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/PackSelectionConfig.java");
    const PACK_TYPE_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/packs/PackType.java");
    const METADATA_SECTION_TYPE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/metadata/MetadataSectionType.java");
    const PACK_METADATA_SECTION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/metadata/pack/PackMetadataSection.java");
    const PACK_SOURCE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/packs/repository/PackSource.java");

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
        assert_ne!(PACK_CLIENT_METADATA_TYPE.kind, PACK_SERVER_METADATA_TYPE.kind);
        assert_ne!(PACK_CLIENT_METADATA_TYPE.kind, PACK_FALLBACK_METADATA_TYPE.kind);
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
    fn pack_metadata_section_preserves_description_range_and_types_like_java() {
        let description = Component::literal("Pack description");
        let supported_formats = PackFormatRange {
            min: PackFormat {
                major: 101,
                minor: 0,
            },
            max: PackFormat {
                major: 101,
                minor: 1,
            },
        };
        let section = PackMetadataSectionModel::new(description.clone(), supported_formats);

        assert_eq!(section.description, description);
        assert_eq!(section.supported_formats, supported_formats);
        assert_eq!(
            PackMetadataSectionModel::section_type_for_pack_type(PackTypeModel::ClientResources),
            PACK_CLIENT_METADATA_TYPE
        );
        assert_eq!(
            PackMetadataSectionModel::section_type_for_pack_type(PackTypeModel::ServerData),
            PACK_SERVER_METADATA_TYPE
        );
        assert_eq!(
            PackMetadataSectionModel::fallback_section_type(),
            PACK_FALLBACK_METADATA_TYPE
        );
    }

    #[test]
    fn pack_metadata_fallback_uses_unknown_single_format_like_java() {
        let fallback = PackMetadataSectionModel::fallback(Component::literal("missing format"));

        assert_eq!(fallback.description.get_string(), "missing format");
        assert_eq!(
            fallback.supported_formats,
            PackFormatRange {
                min: PackFormat {
                    major: u32::MAX,
                    minor: 0,
                },
                max: PackFormat {
                    major: u32::MAX,
                    minor: 0,
                },
            }
        );
    }

    #[test]
    fn pack_source_decoration_and_auto_add_flags_match_java() {
        assert_eq!(
            PackSourceModel::Default
                .decorate(Component::literal("plain"))
                .get_string(),
            "plain"
        );

        let built_in = PackSourceModel::BuiltIn.decorate(Component::literal("vanilla"));
        assert_eq!(
            built_in.get_style().get_color().map(TextColor::serialize),
            Some("gray".to_string())
        );
        match built_in.get_contents() {
            ComponentContent::Translatable { key, args, .. } => {
                assert_eq!(key, "pack.nameAndSource");
                assert_eq!(args.len(), 2);
            }
            other => panic!("pack source decoration should be translatable, got {other:?}"),
        }

        assert!(PackSourceModel::Default.should_add_automatically());
        assert!(PackSourceModel::BuiltIn.should_add_automatically());
        assert!(!PackSourceModel::Feature.should_add_automatically());
        assert!(PackSourceModel::World.should_add_automatically());
        assert!(PackSourceModel::Server.should_add_automatically());
    }

    #[test]
    fn pack_location_chat_link_matches_java_component_shape() {
        let location = PackLocationInfoModel::new(
            "file/My Pack",
            Component::literal("Display Name"),
            PackSourceModel::BuiltIn,
            Some(KnownPack::vanilla("core")),
        );
        let link = location.create_chat_link(false, Component::literal("Description"));

        assert_eq!(
            link.style.get_color().map(TextColor::serialize),
            Some("red".to_string())
        );
        assert_eq!(link.style.get_insertion(), Some("\"file/My Pack\""));
        match link.get_contents() {
            ComponentContent::Translatable { key, args, .. } => {
                assert_eq!(key, "chat.square_brackets");
                assert_eq!(args.len(), 1);
            }
            other => panic!("chat link should be square-brackets translation, got {other:?}"),
        }
        match link.style.get_hover_event() {
            Some(HoverEvent::Text(text)) => {
                assert_eq!(text.get_string(), "Display Name\nDescription");
            }
            other => panic!("chat link should use show_text hover, got {other:?}"),
        }

        let enabled = PackLocationInfoModel::new(
            "simple_id",
            Component::literal("Title"),
            PackSourceModel::Default,
            None,
        )
        .create_chat_link(true, Component::literal("Desc"));
        assert_eq!(
            enabled.style.get_color().map(TextColor::serialize),
            Some("green".to_string())
        );
        assert_eq!(enabled.style.get_insertion(), Some("simple_id"));
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

        for needle in [
            "ComponentUtils.wrapInSquareBrackets(this.source.decorate(Component.literal(this.id)))",
            "enabled ? ChatFormatting.GREEN : ChatFormatting.RED",
            "StringArgumentType.escapeIfRequired(this.id)",
            "new HoverEvent.ShowText(Component.empty().append(this.title).append(\"\\n\").append(description))",
        ] {
            assert!(PACK_LOCATION_INFO_JAVA.contains(needle));
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

        for needle in [
            "public record PackMetadataSection(Component description, InclusiveRange<PackFormat> supportedFormats)",
            "ComponentSerialization.CODEC.fieldOf(\"description\").forGetter(PackMetadataSection::description)",
            "new PackMetadataSection(description, new InclusiveRange<>(PackFormat.of(Integer.MAX_VALUE)))",
            "new MetadataSectionType<>(\"pack\", codecForPackType(PackType.CLIENT_RESOURCES))",
            "new MetadataSectionType<>(\"pack\", codecForPackType(PackType.SERVER_DATA))",
            "public static final MetadataSectionType<PackMetadataSection> FALLBACK_TYPE = new MetadataSectionType<>(\"pack\", FALLBACK_CODEC);",
            "PackFormat.packCodec(packType).forGetter(PackMetadataSection::supportedFormats)",
            "case CLIENT_RESOURCES -> CLIENT_TYPE",
            "case SERVER_DATA -> SERVER_TYPE",
        ] {
            assert!(PACK_METADATA_SECTION_JAVA.contains(needle));
        }

        for needle in [
            "PackSource DEFAULT = create(NO_DECORATION, true);",
            "PackSource BUILT_IN = create(decorateWithSource(\"pack.source.builtin\"), true);",
            "PackSource FEATURE = create(decorateWithSource(\"pack.source.feature\"), false);",
            "PackSource WORLD = create(decorateWithSource(\"pack.source.world\"), true);",
            "PackSource SERVER = create(decorateWithSource(\"pack.source.server\"), true);",
            "Component.translatable(\"pack.nameAndSource\", packDescription, description).withStyle(ChatFormatting.GRAY)",
        ] {
            assert!(PACK_SOURCE_JAVA.contains(needle));
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
