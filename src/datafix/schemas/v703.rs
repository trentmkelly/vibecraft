//! Port of `net.minecraft.util.datafix.schemas.V703`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_entities(schema: &mut Schema) {
    schema.remove_entity("EntityHorse");
    schema.register_entity(
        "Horse",
        dsl::optional_fields(vec![
            ("ArmorItem", dsl::reference(r::ITEM_STACK)),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_entity(
        "Donkey",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_entity(
        "Mule",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_entity(
        "ZombieHorse",
        dsl::optional_fields(vec![("SaddleItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_entity(
        "SkeletonHorse",
        dsl::optional_fields(vec![("SaddleItem", dsl::reference(r::ITEM_STACK))]),
    );
}

/// Applies `V703` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
