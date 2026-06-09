const TAG_APPENDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/tags/TagAppender.java");
const TAGS_PROVIDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/tags/TagsProvider.java");
const HOLDER_TAG_PROVIDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/tags/HolderTagProvider.java");
const KEY_TAG_PROVIDER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/tags/KeyTagProvider.java");
const INTRINSIC_HOLDER_TAGS_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/tags/IntrinsicHolderTagsProvider.java"
);
const DATA_TAGS_PACKAGE_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/tags/package-info.java");

const TAG_APPENDER_SENTINELS: &[&str] = &[
    "public interface TagAppender<E, T>",
    "default TagAppender<E, T> add(final E... elements)",
    "return this.addAll(Arrays.stream(elements));",
    "default TagAppender<E, T> addAll(final Collection<E> elements)",
    "elements.forEach(this::add);",
    "default TagAppender<E, T> addAll(final Stream<E> elements)",
    "TagAppender<E, T> addOptional(E element);",
    "TagAppender<E, T> addTag(TagKey<T> tag);",
    "TagAppender<E, T> addOptionalTag(TagKey<T> tag);",
    "builder.addElement(element.identifier());",
    "builder.addOptionalElement(element.identifier());",
    "builder.addTag(tag.location());",
    "builder.addOptionalTag(tag.location());",
    "default <U> TagAppender<U, T> map(final Function<U, E> converter)",
    "original.add(converter.apply(element));",
    "original.addTag(tag);",
    "original.addOptionalTag(tag);",
];

const TAGS_PROVIDER_SENTINELS: &[&str] = &[
    "public abstract class TagsProvider<T> implements DataProvider",
    "protected final PackOutput.PathProvider pathProvider;",
    "private final CompletableFuture<Void> contentsDone = new CompletableFuture<>();",
    "private final CompletableFuture<TagsProvider.TagLookup<T>> parentProvider;",
    "private final Map<Identifier, TagBuilder> builders = Maps.newLinkedHashMap();",
    "this.pathProvider = output.createRegistryTagsPathProvider(registryKey);",
    "return \"Tags for \" + this.registryKey.identifier();",
    "protected abstract void addTags(HolderLookup.Provider registries);",
    "record CombinedData<T>(HolderLookup.Provider contents, TagsProvider.TagLookup<T> parent)",
    "this.contentsDone.complete(null);",
    "HolderLookup.RegistryLookup<T> lookup = c.contents.lookupOrThrow(this.registryKey);",
    "Predicate<Identifier> elementCheck = id -> lookup.get(ResourceKey.create(this.registryKey, id)).isPresent();",
    "Predicate<Identifier> tagCheck = id -> this.builders.containsKey(id) || c.parent.contains(TagKey.create(this.registryKey, id));",
    "List<TagEntry> unresolvedEntries = entries.stream().filter(e -> !e.verifyIfPresent(elementCheck, tagCheck)).toList();",
    "\"Couldn't define tag %s as it is missing following references: %s\"",
    "Path path = this.pathProvider.json(id);",
    "DataProvider.saveStable(cache, c.contents, TagFile.CODEC, new TagFile(entries, builder.shouldReplace()), path)",
    "return this.builders.computeIfAbsent(tag.location(), k -> TagBuilder.create());",
    "return this.contentsDone.thenApply(ignore -> id -> Optional.ofNullable(this.builders.get(id.location())));",
    "this.builders.clear();",
    "this.addTags(registries);",
    "static <T> TagsProvider.TagLookup<T> empty()",
    "default boolean contains(final TagKey<T> key)",
];

const ADAPTER_PROVIDER_SENTINELS: &[(&str, &[&str])] = &[
    (
        HOLDER_TAG_PROVIDER_JAVA,
        &[
            "public abstract class HolderTagProvider<T> extends TagsProvider<T>",
            "protected TagAppender<Holder.Reference<T>, T> tag(final TagKey<T> tag)",
            "return TagAppender.<T>forBuilder(builder).map(Holder.Reference::key);",
        ],
    ),
    (
        KEY_TAG_PROVIDER_JAVA,
        &[
            "public abstract class KeyTagProvider<T> extends TagsProvider<T>",
            "protected TagAppender<ResourceKey<T>, T> tag(final TagKey<T> tag)",
            "return TagAppender.forBuilder(builder);",
            "protected TagAppender<ResourceKey<T>, T> tag(final TagKey<T> tag, final boolean replace)",
            "builder.setReplace(replace);",
        ],
    ),
    (
        INTRINSIC_HOLDER_TAGS_PROVIDER_JAVA,
        &[
            "public abstract class IntrinsicHolderTagsProvider<T> extends TagsProvider<T>",
            "private final Function<T, ResourceKey<T>> keyExtractor;",
            "final CompletableFuture<TagsProvider.TagLookup<T>> parentProvider",
            "this.keyExtractor = keyExtractor;",
            "return TagAppender.<T>forBuilder(builder).map(this.keyExtractor);",
        ],
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum TagEntryModel {
    Element(String),
    OptionalElement(String),
    Tag(String),
    OptionalTag(String),
}

impl TagEntryModel {
    fn verify_if_present(
        &self,
        element_exists: impl Fn(&str) -> bool,
        tag_exists: impl Fn(&str) -> bool,
    ) -> bool {
        match self {
            Self::Element(id) => element_exists(id),
            Self::OptionalElement(_) => true,
            Self::Tag(id) => tag_exists(id),
            Self::OptionalTag(_) => true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct TagBuilderModel {
    entries: Vec<TagEntryModel>,
    replace: bool,
}

impl TagBuilderModel {
    fn add_element(&mut self, id: &str) {
        self.entries.push(TagEntryModel::Element(id.to_string()));
    }

    fn add_optional_element(&mut self, id: &str) {
        self.entries
            .push(TagEntryModel::OptionalElement(id.to_string()));
    }

    fn add_tag(&mut self, id: &str) {
        self.entries.push(TagEntryModel::Tag(id.to_string()));
    }

    fn add_optional_tag(&mut self, id: &str) {
        self.entries
            .push(TagEntryModel::OptionalTag(id.to_string()));
    }

    fn set_replace(&mut self, replace: bool) {
        self.replace = replace;
    }
}

fn mapped_add_optional_matches_java(builder: &mut TagBuilderModel, element: &str) {
    // TagAppender.map(...).addOptional delegates to original.add(converter.apply(element)).
    builder.add_element(element);
}

fn unresolved_required_entries<'a>(
    entries: &'a [TagEntryModel],
    known_elements: &[&str],
    known_tags: &[&str],
    parent_tags: &[&str],
) -> Vec<&'a TagEntryModel> {
    entries
        .iter()
        .filter(|entry| {
            !entry.verify_if_present(
                |id| known_elements.contains(&id),
                |id| known_tags.contains(&id) || parent_tags.contains(&id),
            )
        })
        .collect()
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_tags_tag_appender_matches_java_builder_contract() {
        assert_source_contains_all(TAG_APPENDER_JAVA, TAG_APPENDER_SENTINELS);
        assert_eq!(TAG_APPENDER_JAVA.lines().count(), 88);
        assert_eq!(count_occurrences(TAG_APPENDER_JAVA, "return this;"), 10);

        let mut builder = TagBuilderModel::default();
        builder.add_element("minecraft:stone");
        builder.add_optional_element("minecraft:maybe");
        builder.add_tag("minecraft:mineable/pickaxe");
        builder.add_optional_tag("minecraft:optional_group");
        assert_eq!(
            builder.entries,
            vec![
                TagEntryModel::Element("minecraft:stone".to_string()),
                TagEntryModel::OptionalElement("minecraft:maybe".to_string()),
                TagEntryModel::Tag("minecraft:mineable/pickaxe".to_string()),
                TagEntryModel::OptionalTag("minecraft:optional_group".to_string()),
            ]
        );
    }

    #[test]
    fn data_tags_mapped_add_optional_uses_required_add_like_java() {
        let mut builder = TagBuilderModel::default();
        mapped_add_optional_matches_java(&mut builder, "minecraft:mapped");
        assert_eq!(
            builder.entries,
            vec![TagEntryModel::Element("minecraft:mapped".to_string())]
        );
    }

    #[test]
    fn data_tags_tags_provider_run_validation_matches_java() {
        assert_source_contains_all(TAGS_PROVIDER_JAVA, TAGS_PROVIDER_SENTINELS);
        assert_eq!(TAGS_PROVIDER_JAVA.lines().count(), 133);
        assert_eq!(
            count_occurrences(TAGS_PROVIDER_JAVA, "CompletableFuture"),
            14
        );
        assert_eq!(count_occurrences(TAGS_PROVIDER_JAVA, "TagLookup"), 8);

        let entries = vec![
            TagEntryModel::Element("minecraft:stone".to_string()),
            TagEntryModel::Element("minecraft:missing".to_string()),
            TagEntryModel::OptionalElement("minecraft:optional_missing".to_string()),
            TagEntryModel::Tag("minecraft:known_tag".to_string()),
            TagEntryModel::Tag("minecraft:parent_tag".to_string()),
            TagEntryModel::Tag("minecraft:missing_tag".to_string()),
            TagEntryModel::OptionalTag("minecraft:optional_missing_tag".to_string()),
        ];
        let unresolved = unresolved_required_entries(
            &entries,
            &["minecraft:stone"],
            &["minecraft:known_tag"],
            &["minecraft:parent_tag"],
        );
        assert_eq!(
            unresolved,
            vec![
                &TagEntryModel::Element("minecraft:missing".to_string()),
                &TagEntryModel::Tag("minecraft:missing_tag".to_string()),
            ]
        );
    }

    #[test]
    fn data_tags_adapter_providers_match_java_contracts() {
        for (source, sentinels) in ADAPTER_PROVIDER_SENTINELS {
            assert_source_contains_all(source, sentinels);
        }
        assert_eq!(HOLDER_TAG_PROVIDER_JAVA.lines().count(), 23);
        assert_eq!(KEY_TAG_PROVIDER_JAVA.lines().count(), 28);
        assert_eq!(INTRINSIC_HOLDER_TAGS_PROVIDER_JAVA.lines().count(), 40);

        let mut replace_builder = TagBuilderModel::default();
        replace_builder.set_replace(false);
        assert!(!replace_builder.replace);
    }

    #[test]
    fn data_tags_package_is_null_marked() {
        assert!(DATA_TAGS_PACKAGE_JAVA.contains("@NullMarked"));
        assert!(DATA_TAGS_PACKAGE_JAVA.contains("package net.minecraft.data.tags;"));
        assert!(DATA_TAGS_PACKAGE_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
        assert_eq!(count_occurrences(DATA_TAGS_PACKAGE_JAVA, "@NullMarked"), 1);
    }
}
