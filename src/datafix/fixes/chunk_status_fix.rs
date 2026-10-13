//! Port of `net.minecraft.util.datafix.fixes.ChunkStatusFix`.

use crate::datafix::dynamic::{get_str_or, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ChunkStatusFix(schema, changesType)`: the legacy `postprocessed` chunk
/// status becomes `fullchunk`; all other values remain as stored.
pub fn fix() -> Fix {
    Fix::everywhere("ChunkStatusFix", Target::Type(r::CHUNK), |chunk| {
        let Some(level) = typed_mut(chunk, "Level") else {
            return;
        };
        if get_str_or(level, "Status", "empty") == "postprocessed" {
            set(level, "Status", Tag::String("fullchunk".to_string()));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::fix;
    use crate::datafix::dynamic::get;
    use crate::datafix::fix::FixContext;
    use crate::datafix::references as r;
    use crate::datafix::schema::{Schema, VersionKey};
    use crate::datafix::schemas::v99;
    use crate::storage::nbt::Tag;

    fn context() -> Schema {
        let mut schema = Schema::root(VersionKey::new(99));
        v99::build(&mut schema);
        schema
    }

    #[test]
    fn changes_postprocessed_to_fullchunk() {
        let schema = context();
        let fix_context = FixContext {
            input: &schema,
            output: &schema,
        };
        let mut chunk = Tag::Compound(vec![(
            "Level".to_string(),
            Tag::Compound(vec![(
                "Status".to_string(),
                Tag::String("postprocessed".to_string()),
            )]),
        )]);

        assert!(fix().apply(&fix_context, r::CHUNK, &mut chunk));

        assert_eq!(
            get(get(&chunk, "Level").unwrap(), "Status"),
            Some(&Tag::String("fullchunk".to_string()))
        );
    }

    #[test]
    fn leaves_other_and_non_string_status_values_unchanged() {
        let schema = context();
        let fix_context = FixContext {
            input: &schema,
            output: &schema,
        };
        for status in [Tag::String("full".to_string()), Tag::Int(3)] {
            let mut chunk = Tag::Compound(vec![(
                "Level".to_string(),
                Tag::Compound(vec![("Status".to_string(), status.clone())]),
            )]);
            assert!(fix().apply(&fix_context, r::CHUNK, &mut chunk));
            assert_eq!(get(get(&chunk, "Level").unwrap(), "Status"), Some(&status));
        }
    }
}
