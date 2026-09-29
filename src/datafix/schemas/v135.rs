//! Port of `net.minecraft.util.datafix.schemas.V135`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::PLAYER,
        dsl::optional_fields(vec![
            (
                "RootVehicle",
                dsl::optional_fields(vec![("Entity", dsl::reference(r::ENTITY_TREE))]),
            ),
            ("ender_pearls", dsl::list(dsl::reference(r::ENTITY_TREE))),
            ("Inventory", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("EnderItems", dsl::list(dsl::reference(r::ITEM_STACK))),
        ]),
    );
    schema.register_type(
        r::ENTITY_TREE,
        dsl::optional_fields_with_rest(
            vec![("Passengers", dsl::list(dsl::reference(r::ENTITY_TREE)))],
            dsl::reference(r::ENTITY),
        ),
    );
}

/// Applies `V135` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
