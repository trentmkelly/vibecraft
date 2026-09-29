//! Port of `net.minecraft.util.datafix.schemas.V1022`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_types(schema: &mut Schema) {
    schema.register_type(r::RECIPE, dsl::namespaced_string());
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
            ("ShoulderEntityLeft", dsl::reference(r::ENTITY_TREE)),
            ("ShoulderEntityRight", dsl::reference(r::ENTITY_TREE)),
            (
                "recipeBook",
                dsl::optional_fields(vec![
                    ("recipes", dsl::list(dsl::reference(r::RECIPE))),
                    ("toBeDisplayed", dsl::list(dsl::reference(r::RECIPE))),
                ]),
            ),
        ]),
    );
    schema.register_type(
        r::HOTBAR,
        dsl::compound_list(dsl::list(dsl::reference(r::ITEM_STACK))),
    );
}

/// Applies `V1022` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
