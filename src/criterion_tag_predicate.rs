use std::collections::BTreeSet;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagPredicateModel {
    pub tag: TagKeyModel,
    pub expected: bool,
}

impl TagPredicateModel {
    pub fn new(tag: TagKeyModel, expected: bool) -> Self {
        Self { tag, expected }
    }

    pub fn codec_shape(registry_key: Identifier) -> CodecShapeModel {
        CodecShapeModel {
            registry_key,
            id_codec: "TagKey.codec(registryKey)",
            expected_codec: "Codec.BOOL",
            fields: ["id", "expected"],
        }
    }

    pub fn is(tag: TagKeyModel) -> Self {
        Self::new(tag, true)
    }

    pub fn is_not(tag: TagKeyModel) -> Self {
        Self::new(tag, false)
    }

    pub fn matches<T>(&self, holder: &HolderModel<T>) -> bool {
        holder.is(&self.tag) == self.expected
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecShapeModel {
    pub registry_key: Identifier,
    pub id_codec: &'static str,
    pub expected_codec: &'static str,
    pub fields: [&'static str; 2],
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagKeyModel {
    registry_key: Identifier,
    location: Identifier,
}

impl TagKeyModel {
    pub fn new(registry_key: Identifier, location: Identifier) -> Self {
        Self {
            registry_key,
            location,
        }
    }

    pub fn registry_key(&self) -> &Identifier {
        &self.registry_key
    }

    pub fn location(&self) -> &Identifier {
        &self.location
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderModel<T> {
    value: T,
    tags: BTreeSet<TagKeyModel>,
}

impl<T> HolderModel<T> {
    pub fn new(value: T, tags: impl IntoIterator<Item = TagKeyModel>) -> Self {
        Self {
            value,
            tags: tags.into_iter().collect(),
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    fn is(&self, tag: &TagKeyModel) -> bool {
        self.tags.contains(tag)
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_tag(location: &str) -> TagKeyModel {
        TagKeyModel::new(id("minecraft:item"), id(location))
    }

    fn block_tag(location: &str) -> TagKeyModel {
        TagKeyModel::new(id("minecraft:block"), id(location))
    }

    #[test]
    fn tag_predicate_codec_uses_registry_specific_tag_key_and_required_expected_bool() {
        let shape = TagPredicateModel::codec_shape(id("minecraft:damage_type"));

        assert_eq!(shape.registry_key, id("minecraft:damage_type"));
        assert_eq!(shape.id_codec, "TagKey.codec(registryKey)");
        assert_eq!(shape.expected_codec, "Codec.BOOL");
        assert_eq!(shape.fields, ["id", "expected"]);
    }

    #[test]
    fn tag_predicate_factories_store_expected_flag_exactly() {
        let tag = item_tag("minecraft:axes");

        assert_eq!(
            TagPredicateModel::is(tag.clone()),
            TagPredicateModel::new(tag.clone(), true)
        );
        assert_eq!(
            TagPredicateModel::is_not(tag.clone()),
            TagPredicateModel::new(tag, false)
        );
    }

    #[test]
    fn tag_key_preserves_registry_key_and_location_from_codec_field() {
        let tag = TagKeyModel::new(id("minecraft:item"), id("minecraft:pickaxes"));

        assert_eq!(tag.registry_key(), &id("minecraft:item"));
        assert_eq!(tag.location(), &id("minecraft:pickaxes"));
    }

    #[test]
    fn tag_predicate_matches_holder_membership_equal_to_expected() {
        let axes = item_tag("minecraft:axes");
        let pickaxes = item_tag("minecraft:pickaxes");
        let holder = HolderModel::new("minecraft:diamond_axe", [axes.clone()]);

        assert!(TagPredicateModel::is(axes.clone()).matches(&holder));
        assert!(!TagPredicateModel::is_not(axes).matches(&holder));
        assert!(!TagPredicateModel::is(pickaxes.clone()).matches(&holder));
        assert!(TagPredicateModel::is_not(pickaxes).matches(&holder));
    }

    #[test]
    fn tag_predicate_uses_full_tag_key_identity_including_registry() {
        let item_axes = item_tag("minecraft:axes");
        let block_axes = block_tag("minecraft:axes");
        let holder = HolderModel::new("minecraft:diamond_axe", [item_axes.clone()]);

        assert!(TagPredicateModel::is(item_axes).matches(&holder));
        assert!(!TagPredicateModel::is(block_axes.clone()).matches(&holder));
        assert!(TagPredicateModel::is_not(block_axes).matches(&holder));
        assert_eq!(holder.value(), &"minecraft:diamond_axe");
    }
}
