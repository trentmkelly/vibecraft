//! Port of `net.minecraft.util.datafix.fixes.EntityCatSplitFix`.

use crate::datafix::dynamic::{get_i32_or, get_str_or, set};
use crate::datafix::fix::Fix;
use crate::storage::nbt::Tag;

use super::simple_entity_rename_fix::simple_entity_rename_fix;

/// `new EntityCatSplitFix(schema, changesType)`.
pub fn fix() -> Fix {
    simple_entity_rename_fix("EntityCatSplitFix", |name, tag| {
        if name == "minecraft:ocelot" {
            let cat_type = get_i32_or(tag, "CatType", 0);
            // For type zero Java calls Dynamic.set for Trusting but discards the
            // immutable Dynamic it returns, so that branch leaves the tag unchanged.
            if (1..4).contains(&cat_type) {
                set(tag, "CatType", Tag::Int(cat_type));
                let owner_uuid = get_str_or(tag, "OwnerUUID", "").to_string();
                set(tag, "OwnerUUID", Tag::String(owner_uuid));
                return "minecraft:cat".to_string();
            }
        }

        name.to_string()
    })
}

#[cfg(test)]
mod tests {
    use crate::datafix::dynamic::equal_unordered;
    use crate::datafix::fixer::DataFixerBuilder;
    use crate::datafix::references as r;
    use crate::datafix::template::{dsl, ChoiceSet};
    use crate::storage::nbt::Tag;

    use super::fix;
    use crate::datafix::schemas::v1904;

    fn fixer() -> crate::datafix::fixer::DataFixer {
        let mut builder = DataFixerBuilder::new();
        builder.add_schema(1800, 0, |schema| {
            schema.register_type(r::ENTITY, dsl::namespaced_choice(ChoiceSet::Entities));
            schema.register_simple_entity("minecraft:ocelot");
        });
        let cat_schema = builder.add_schema(1904, 0, v1904::build);
        builder.add_fixer(cat_schema, false, fix());
        builder.build(1904)
    }

    fn entity(fields: Vec<(&str, Tag)>) -> Tag {
        let mut entries = vec![(
            "id".to_string(),
            Tag::String("minecraft:ocelot".to_string()),
        )];
        entries.extend(
            fields
                .into_iter()
                .map(|(name, value)| (name.to_string(), value)),
        );
        Tag::Compound(entries)
    }

    fn update(input: &Tag) -> Tag {
        fixer().update(r::ENTITY, input, 1800, 1904).unwrap()
    }

    fn assert_updated(input: &Tag, expected: &Tag) {
        let actual = update(input);
        // NBT compounds are maps. The shared entity rename fix removes and
        // reinserts the discriminator, which may move `id` within Rust's Vec.
        assert!(
            equal_unordered(&actual, expected),
            "expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn cat_types_one_through_three_become_cats_and_preserve_other_data() {
        for cat_type in 1..4 {
            let input = entity(vec![
                ("CatType", Tag::Short(cat_type)),
                ("OwnerUUID", Tag::Int(42)),
                ("Owner", Tag::String("Mittens".to_string())),
                ("Custom", Tag::String("kept".to_string())),
            ]);

            assert_updated(
                &input,
                &entity_with_id(
                    "minecraft:cat",
                    vec![
                        ("CatType", Tag::Int(i32::from(cat_type))),
                        ("OwnerUUID", Tag::String(String::new())),
                        ("Owner", Tag::String("Mittens".to_string())),
                        ("Custom", Tag::String("kept".to_string())),
                    ],
                ),
            );
        }
    }

    #[test]
    fn type_zero_preserves_ocelot_and_does_not_persist_the_java_dynamic_set_call() {
        let input = entity(vec![
            ("CatType", Tag::Int(0)),
            ("Owner", Tag::String("Mittens".to_string())),
            (
                "OwnerUUID",
                Tag::String("01234567-89ab-cdef-0123-456789abcdef".to_string()),
            ),
        ]);

        assert_updated(&input, &input);
    }

    #[test]
    fn cat_type_conversion_keeps_a_string_owner_uuid() {
        let owner_uuid = "01234567-89ab-cdef-0123-456789abcdef";
        let input = entity(vec![
            ("CatType", Tag::Int(2)),
            ("OwnerUUID", Tag::String(owner_uuid.to_string())),
        ]);

        assert_updated(
            &input,
            &entity_with_id(
                "minecraft:cat",
                vec![
                    ("CatType", Tag::Int(2)),
                    ("OwnerUUID", Tag::String(owner_uuid.to_string())),
                ],
            ),
        );
    }

    #[test]
    fn other_cat_types_remain_ocelots() {
        for cat_type in [-1, 4, 9] {
            let input = entity(vec![("CatType", Tag::Int(cat_type))]);
            assert_updated(&input, &input);
        }
    }

    fn entity_with_id(id: &str, fields: Vec<(&str, Tag)>) -> Tag {
        let mut entries = vec![("id".to_string(), Tag::String(id.to_string()))];
        entries.extend(
            fields
                .into_iter()
                .map(|(name, value)| (name.to_string(), value)),
        );
        Tag::Compound(entries)
    }
}
