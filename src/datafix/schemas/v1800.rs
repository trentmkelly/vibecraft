//! Port of `net.minecraft.util.datafix.schemas.V1800`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;

fn register_entities(schema: &mut Schema) {
    schema.register_simple_entity("minecraft:panda");
    schema.register_entity(
        "minecraft:pillager",
        dsl::optional_fields(vec![(
            "Inventory",
            dsl::list(dsl::reference(r::ITEM_STACK)),
        )]),
    );
}

/// Applies `V1800` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}

#[cfg(test)]
mod tests {
    use crate::datafix::schema::Schema;
    use crate::datafix::schema::VersionKey;
    use crate::datafix::template::{ChoiceSet, Tmpl};

    use super::build;

    #[test]
    fn registers_panda_and_pillager_inventory_shape() {
        let mut schema = Schema::root(VersionKey::new(1800));
        build(&mut schema);

        assert!(schema.has_entity("minecraft:panda"));
        let Some(Tmpl::Fields(fields)) = schema.choice(ChoiceSet::Entities, "minecraft:pillager")
        else {
            panic!("pillager must have optional fields");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "Inventory");
        assert!(fields[0].optional);
        assert!(matches!(
            &fields[0].tmpl,
            Tmpl::List(item_stack) if matches!(item_stack.as_ref(), Tmpl::Ref(crate::datafix::references::ITEM_STACK))
        ));
    }
}
