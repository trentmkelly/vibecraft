//! Port of `net.minecraft.util.datafix.schemas.V1904`.

use crate::datafix::schema::Schema;

fn register_entities(schema: &mut Schema) {
    schema.register_simple_entity("minecraft:cat");
}

/// Applies `V1904` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}

#[cfg(test)]
mod tests {
    use crate::datafix::schema::{Schema, VersionKey};
    use crate::datafix::template::ChoiceSet;

    use super::build;

    #[test]
    fn registers_cat_as_a_simple_entity() {
        let mut schema = Schema::root(VersionKey::new(1904));
        build(&mut schema);

        assert!(schema.has_entity("minecraft:cat"));
        assert!(schema
            .choice(ChoiceSet::Entities, "minecraft:cat")
            .is_some());
    }
}
