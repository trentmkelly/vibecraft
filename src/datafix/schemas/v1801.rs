//! Port of `net.minecraft.util.datafix.schemas.V1801`.

use crate::datafix::schema::Schema;

fn register_entities(schema: &mut Schema) {
    schema.register_simple_entity("minecraft:illager_beast");
}

/// Applies `V1801` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}

#[cfg(test)]
mod tests {
    use crate::datafix::schema::{Schema, VersionKey};
    use crate::datafix::template::ChoiceSet;

    use super::build;

    #[test]
    fn registers_illager_beast_as_a_simple_entity() {
        let mut schema = Schema::root(VersionKey::new(1801));
        build(&mut schema);

        assert!(schema.has_entity("minecraft:illager_beast"));
        assert!(schema
            .choice(ChoiceSet::Entities, "minecraft:illager_beast")
            .is_some());
    }
}
