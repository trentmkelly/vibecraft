use crate::chat_component::{Component, NbtSource, ResolutionContext};

pub const NBT_CONTENTS_CODEC_FIELDS: [&str; 5] =
    ["nbt", "interpret", "plain", "separator", "source"];
pub const DATA_SOURCE_CODEC_TYPES: [&str; 3] = ["entity", "block", "storage"];
pub const DEFAULT_NBT_SEPARATOR: &str = ", ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtContentsModel {
    nbt_path: String,
    interpreting: bool,
    plain: bool,
    separator: Option<Component>,
    data_source: NbtDataSourceModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtDataSourceModel {
    Block { coordinates: String },
    Entity { selector: String },
    Storage { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtContentsError {
    InterpretAndPlain,
}

impl NbtContentsModel {
    pub fn new(
        nbt_path: impl Into<String>,
        interpreting: bool,
        plain: bool,
        separator: Option<Component>,
        data_source: NbtDataSourceModel,
    ) -> Result<Self, NbtContentsError> {
        if interpreting && plain {
            return Err(NbtContentsError::InterpretAndPlain);
        }
        Ok(Self {
            nbt_path: nbt_path.into(),
            interpreting,
            plain,
            separator,
            data_source,
        })
    }

    pub fn nbt_path(&self) -> &str {
        &self.nbt_path
    }

    pub fn interpreting(&self) -> bool {
        self.interpreting
    }

    pub fn plain(&self) -> bool {
        self.plain
    }

    pub fn separator(&self) -> Option<&Component> {
        self.separator.as_ref()
    }

    pub fn data_source(&self) -> &NbtDataSourceModel {
        &self.data_source
    }

    pub fn codec_fields(&self) -> [&'static str; 5] {
        NBT_CONTENTS_CODEC_FIELDS
    }

    pub fn data_source_codec_types(&self) -> [&'static str; 3] {
        DATA_SOURCE_CODEC_TYPES
    }

    pub fn resolve(&self, context: &ResolutionContext, _recursion_depth: i32) -> Component {
        if context.source.is_none() {
            return Component::empty();
        }

        let values = context.nbt(&self.data_source.as_nbt_source(), &self.nbt_path);
        if values.is_empty() {
            return Component::empty();
        }

        let separator = self
            .separator
            .clone()
            .unwrap_or_else(|| Component::literal(DEFAULT_NBT_SEPARATOR));

        let components = if self.interpreting {
            values
                .iter()
                .filter_map(|value| parse_component_payload(value))
                .collect::<Vec<_>>()
        } else {
            values
                .into_iter()
                .map(Component::literal)
                .collect::<Vec<_>>()
        };

        join_components(components, separator)
    }
}

impl NbtDataSourceModel {
    pub fn block(coordinates: impl Into<String>) -> Self {
        Self::Block {
            coordinates: coordinates.into(),
        }
    }

    pub fn entity(selector: impl Into<String>) -> Self {
        Self::Entity {
            selector: selector.into(),
        }
    }

    pub fn storage(id: impl Into<String>) -> Self {
        Self::Storage { id: id.into() }
    }

    pub fn codec_field(&self) -> &'static str {
        match self {
            Self::Block { .. } => "block",
            Self::Entity { .. } => "entity",
            Self::Storage { .. } => "storage",
        }
    }

    pub fn source_codec_field(&self) -> &'static str {
        "source"
    }

    pub fn source_value(&self) -> &str {
        match self {
            Self::Block { coordinates } => coordinates,
            Self::Entity { selector } => selector,
            Self::Storage { id } => id,
        }
    }

    pub fn get_data(&self, context: &ResolutionContext, path: &str) -> Vec<String> {
        context.nbt(&self.as_nbt_source(), path)
    }

    pub fn as_nbt_source(&self) -> NbtSource {
        match self {
            Self::Block { coordinates } => NbtSource::Block(coordinates.clone()),
            Self::Entity { selector } => NbtSource::Entity(selector.clone()),
            Self::Storage { id } => NbtSource::Storage(id.clone()),
        }
    }
}

impl std::fmt::Display for NbtDataSourceModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block { coordinates } => write!(formatter, "block={coordinates}"),
            Self::Entity { selector } => write!(formatter, "entity={selector}"),
            Self::Storage { id } => write!(formatter, "storage={id}"),
        }
    }
}

fn parse_component_payload(payload: &str) -> Option<Component> {
    let value = serde_json::from_str::<serde_json::Value>(payload).ok()?;
    match value {
        serde_json::Value::String(text) => Some(Component::literal(text)),
        serde_json::Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(serde_json::Value::as_str) {
                Some(Component::literal(text))
            } else {
                object
                    .get("translate")
                    .and_then(serde_json::Value::as_str)
                    .map(|key| Component::translatable(key, Vec::new()))
            }
        }
        _ => None,
    }
}

fn join_components(mut components: Vec<Component>, separator: Component) -> Component {
    if components.is_empty() {
        return Component::empty();
    }

    let mut result = components.remove(0);
    for component in components {
        result = result.append(separator.clone()).append(component);
    }
    result
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::resolution_context::ResolutionSourceModel;
    use crate::chat_component::TranslationTable;

    const NBT_CONTENTS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/contents/NbtContents.java");
    const DATA_SOURCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/contents/data/DataSource.java");
    const DATA_SOURCES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/contents/data/DataSources.java");
    const BLOCK_DATA_SOURCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/contents/data/BlockDataSource.java");
    const ENTITY_DATA_SOURCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/contents/data/EntityDataSource.java");
    const STORAGE_DATA_SOURCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/contents/data/StorageDataSource.java");

    fn source_context() -> ResolutionContext {
        ResolutionContext::create(ResolutionSourceModel { entity: None })
    }

    fn storage(id: &str) -> NbtDataSourceModel {
        NbtDataSourceModel::storage(id)
    }

    fn valid(result: Result<NbtContentsModel, NbtContentsError>) -> NbtContentsModel {
        match result {
            Ok(contents) => contents,
            Err(error) => panic!("expected valid NBT contents, got {error:?}"),
        }
    }

    #[test]
    fn nbt_contents_java_source_contract_is_tracked() {
        for sentinel in [
            "public record NbtContents(",
            "NBT_PATH_CODEC.fieldOf(\"nbt\").forGetter(NbtContents::nbtPath)",
            "Codec.BOOL.lenientOptionalFieldOf(\"interpret\", false).forGetter(NbtContents::interpreting)",
            "Codec.BOOL.lenientOptionalFieldOf(\"plain\", false).forGetter(NbtContents::plain)",
            "ComponentSerialization.CODEC.lenientOptionalFieldOf(\"separator\").forGetter(NbtContents::separator)",
            "DataSources.CODEC.forGetter(NbtContents::dataSource)",
            "'interpret' and 'plain' flags can't be both on",
            "CommandSourceStack source = context.source();",
            "if (source == null)",
            "ComponentUtils.resolve(context, this.separator, recursionDepth)",
            "ComponentUtils.DEFAULT_NO_STYLE_SEPARATOR",
            "if (this.interpreting)",
            "ComponentSerialization.CODEC.parse(registryOps, tag).getOrThrow()",
            "TextComponentTagVisitor.PlainStyling.INSTANCE",
            "TextComponentTagVisitor.RichStyling.INSTANCE",
            "left.append(resolvedSeparator).append(right)",
            "orElseGet(Component::empty)",
        ] {
            assert!(
                NBT_CONTENTS_JAVA.contains(sentinel),
                "missing NbtContents Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "Stream<CompoundTag> getData(final CommandSourceStack sender) throws CommandSyntaxException;",
            "MapCodec<? extends DataSource> codec();",
        ] {
            assert!(
                DATA_SOURCE_JAVA.contains(sentinel),
                "missing DataSource Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "ComponentSerialization.createLegacyComponentMatcher(ID_MAPPER, DataSource::codec, \"source\")",
            "ID_MAPPER.put(\"entity\", EntityDataSource.MAP_CODEC);",
            "ID_MAPPER.put(\"block\", BlockDataSource.MAP_CODEC);",
            "ID_MAPPER.put(\"storage\", StorageDataSource.MAP_CODEC);",
        ] {
            assert!(
                DATA_SOURCES_JAVA.contains(sentinel),
                "missing DataSources Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn nbt_data_sources_java_source_contract_is_tracked() {
        for sentinel in [
            "public interface DataSource",
            "Stream<CompoundTag> getData(final CommandSourceStack sender) throws CommandSyntaxException;",
            "MapCodec<? extends DataSource> codec();",
        ] {
            assert!(
                DATA_SOURCE_JAVA.contains(sentinel),
                "missing DataSource Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "ComponentSerialization.createLegacyComponentMatcher(ID_MAPPER, DataSource::codec, \"source\")",
            "ID_MAPPER.put(\"entity\", EntityDataSource.MAP_CODEC);",
            "ID_MAPPER.put(\"block\", BlockDataSource.MAP_CODEC);",
            "ID_MAPPER.put(\"storage\", StorageDataSource.MAP_CODEC);",
        ] {
            assert!(
                DATA_SOURCES_JAVA.contains(sentinel),
                "missing DataSources Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record BlockDataSource(CompilableString<Coordinates> coordinates) implements DataSource",
            "BLOCK_POS_CODEC.fieldOf(\"block\").forGetter(BlockDataSource::coordinates)",
            "BlockPosArgument.blockPos().parse(reader)",
            "Invalid coordinates path: ",
            "level.isLoaded(pos)",
            "entity.saveWithFullMetadata(sender.registryAccess())",
            "return Stream.empty();",
            "return MAP_CODEC;",
        ] {
            assert!(
                BLOCK_DATA_SOURCE_JAVA.contains(sentinel),
                "missing BlockDataSource Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record EntityDataSource(CompilableString<EntitySelector> selector) implements DataSource",
            "EntitySelector.COMPILABLE_CODEC.fieldOf(\"entity\").forGetter(EntityDataSource::selector)",
            "this.selector.compiled().findEntities(sender)",
            "entities.stream().map(NbtPredicate::getEntityTagToCompare)",
            "return MAP_CODEC;",
        ] {
            assert!(
                ENTITY_DATA_SOURCE_JAVA.contains(sentinel),
                "missing EntityDataSource Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record StorageDataSource(Identifier id) implements DataSource",
            "Identifier.CODEC.fieldOf(\"storage\").forGetter(StorageDataSource::id)",
            "sender.getServer().getCommandStorage().get(this.id)",
            "return Stream.of(tag);",
            "return \"storage=\" + this.id;",
        ] {
            assert!(
                STORAGE_DATA_SOURCE_JAVA.contains(sentinel),
                "missing StorageDataSource Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn nbt_contents_rejects_interpret_and_plain_together_like_java() {
        assert_eq!(
            NbtContentsModel::new("path", true, true, None, storage("minecraft:test")),
            Err(NbtContentsError::InterpretAndPlain)
        );
    }

    #[test]
    fn nbt_contents_resolves_source_null_empty_and_joins_selected_values() {
        let contents = valid(NbtContentsModel::new(
            "Items[].id",
            false,
            false,
            None,
            storage("minecraft:test"),
        ));
        let without_source = ResolutionContext::default().with_nbt(
            NbtSource::Storage("minecraft:test".to_string()),
            "Items[].id",
            vec!["minecraft:stone"],
        );
        assert_eq!(contents.resolve(&without_source, 0).get_string(), "");

        let with_values = source_context().with_nbt(
            NbtSource::Storage("minecraft:test".to_string()),
            "Items[].id",
            vec!["minecraft:stone", "minecraft:dirt"],
        );
        assert_eq!(
            contents.resolve(&with_values, 0).get_string(),
            "minecraft:stone, minecraft:dirt"
        );

        let custom_separator = valid(NbtContentsModel::new(
            "Items[].id",
            false,
            true,
            Some(Component::literal(" | ")),
            storage("minecraft:test"),
        ));
        assert_eq!(
            custom_separator.resolve(&with_values, 0).get_string(),
            "minecraft:stone | minecraft:dirt"
        );
    }

    #[test]
    fn nbt_contents_interprets_component_payloads_and_skips_invalid_values() {
        let contents = valid(NbtContentsModel::new(
            "lines",
            true,
            false,
            None,
            storage("minecraft:sign"),
        ));
        let context = source_context().with_nbt(
            NbtSource::Storage("minecraft:sign".to_string()),
            "lines",
            vec![
                "{\"text\":\"First\"}",
                "\"Second\"",
                "not a component",
                "{\"translate\":\"chat.type.text\"}",
            ],
        );
        let translations = TranslationTable::default().with("chat.type.text", "Translated");

        assert_eq!(
            contents
                .resolve(&context, 0)
                .render_plain(&translations, &context),
            "First, Second, Translated"
        );
    }

    #[test]
    fn nbt_contents_accessors_and_data_source_surface_match_java() {
        let contents = valid(NbtContentsModel::new(
            "Inventory[0]",
            false,
            true,
            Some(Component::literal("; ")),
            NbtDataSourceModel::block("~ ~ ~"),
        ));

        assert_eq!(
            contents.codec_fields(),
            ["nbt", "interpret", "plain", "separator", "source"]
        );
        assert_eq!(
            contents.data_source_codec_types(),
            ["entity", "block", "storage"]
        );
        assert_eq!(contents.nbt_path(), "Inventory[0]");
        assert!(!contents.interpreting());
        assert!(contents.plain());
        assert_eq!(
            contents.separator().map(Component::get_string).as_deref(),
            Some("; ")
        );
        assert_eq!(contents.data_source(), &NbtDataSourceModel::block("~ ~ ~"));
        assert_eq!(contents.data_source().codec_field(), "block");
        assert_eq!(contents.data_source().source_codec_field(), "source");
        assert_eq!(contents.data_source().source_value(), "~ ~ ~");
        assert_eq!(contents.data_source().to_string(), "block=~ ~ ~");
        assert_eq!(
            NbtDataSourceModel::entity("@s").as_nbt_source(),
            NbtSource::Entity("@s".to_string())
        );
        assert_eq!(
            storage("minecraft:test").to_string(),
            "storage=minecraft:test"
        );
    }

    #[test]
    fn nbt_data_sources_get_context_data_by_java_source_kind() {
        let context = source_context()
            .with_nbt(
                NbtSource::Block("0 64 0".to_string()),
                "Items",
                vec!["block"],
            )
            .with_nbt(
                NbtSource::Entity("@s".to_string()),
                "SelectedItem",
                vec!["entity"],
            )
            .with_nbt(
                NbtSource::Storage("minecraft:test".to_string()),
                "Root",
                vec!["storage"],
            );

        assert_eq!(NbtDataSourceModel::block("0 64 0").codec_field(), "block");
        assert_eq!(
            NbtDataSourceModel::block("0 64 0").get_data(&context, "Items"),
            vec!["block".to_string()]
        );
        assert_eq!(NbtDataSourceModel::entity("@s").codec_field(), "entity");
        assert_eq!(
            NbtDataSourceModel::entity("@s").get_data(&context, "SelectedItem"),
            vec!["entity".to_string()]
        );
        assert_eq!(
            NbtDataSourceModel::storage("minecraft:test").codec_field(),
            "storage"
        );
        assert_eq!(
            NbtDataSourceModel::storage("minecraft:test").get_data(&context, "Root"),
            vec!["storage".to_string()]
        );
        assert!(
            NbtDataSourceModel::block("1 64 0")
                .get_data(&context, "Items")
                .is_empty(),
            "Java block source returns an empty stream for unloaded/missing block entity data"
        );
    }
}
