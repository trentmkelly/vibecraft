//! Port of `net.minecraft.util.datafix.fixes.ItemLoreFix`.

use crate::datafix::fix::Fix;
use crate::datafix::legacy_component_data_fix_utils::create_text_component_json;
use crate::datafix::references as r;
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ItemLoreFix(schema)`: each typed lore component's legacy string is
/// wrapped in a plain text JSON component, matching the Java pair's second
/// string update through `LegacyComponentDataFixUtils.createTextComponentJson`.
pub fn fix() -> Fix {
    Fix::everywhere(
        "Item Lore componentize",
        Target::Type(r::ITEM_STACK),
        |stack| {
            let Some(tag) = typed_mut(stack, "tag") else {
                return;
            };
            let Some(display) = typed_mut(tag, "display") else {
                return;
            };
            let Some(Tag::List(lore)) = typed_mut(display, "Lore") else {
                return;
            };

            for component in lore {
                if let Tag::String(text) = component {
                    *text = create_text_component_json(text);
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::fix;
    use crate::datafix::fix::FixContext;
    use crate::datafix::references as r;
    use crate::datafix::schema::{Schema, VersionKey};
    use crate::datafix::schemas::v99;
    use crate::storage::nbt::Tag;

    #[test]
    fn componentizes_each_lore_string() {
        let mut schema = Schema::root(VersionKey::new(99));
        v99::build(&mut schema);
        let context = FixContext {
            input: &schema,
            output: &schema,
        };
        let mut stack = Tag::Compound(vec![(
            "tag".to_string(),
            Tag::Compound(vec![(
                "display".to_string(),
                Tag::Compound(vec![(
                    "Lore".to_string(),
                    Tag::List(vec![
                        Tag::String("first line".to_string()),
                        Tag::String("second line".to_string()),
                    ]),
                )]),
            )]),
        )]);

        assert!(fix().apply(&context, r::ITEM_STACK, &mut stack));

        let Tag::Compound(item_fields) = stack else {
            panic!("item stack should remain a compound");
        };
        let Tag::Compound(tag_fields) = &item_fields[0].1 else {
            panic!("item tag should remain a compound");
        };
        let Tag::Compound(display_fields) = &tag_fields[0].1 else {
            panic!("display should remain a compound");
        };
        let Tag::List(lore) = &display_fields[0].1 else {
            panic!("Lore should remain a list");
        };
        assert_eq!(lore[0], Tag::String(r#"{"text":"first line"}"#.to_string()));
        assert_eq!(
            lore[1],
            Tag::String(r#"{"text":"second line"}"#.to_string())
        );
    }

    #[test]
    fn missing_display_or_lore_is_left_untouched() {
        let mut schema = Schema::root(VersionKey::new(99));
        v99::build(&mut schema);
        let context = FixContext {
            input: &schema,
            output: &schema,
        };
        let mut stack = Tag::Compound(vec![("tag".to_string(), Tag::Compound(Vec::new()))]);

        assert!(fix().apply(&context, r::ITEM_STACK, &mut stack));
        let Tag::Compound(fields) = stack else {
            panic!("item stack should remain a compound");
        };
        assert_eq!(fields[0].1, Tag::Compound(Vec::new()));
    }
}
