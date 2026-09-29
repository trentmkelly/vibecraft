//! Port of `net.minecraft.util.datafix.schemas.V100`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::ENTITY_EQUIPMENT,
        dsl::and(vec![
            dsl::optional_fields(vec![(
                "ArmorItems",
                dsl::list(dsl::reference(r::ITEM_STACK)),
            )]),
            dsl::optional_fields(vec![(
                "HandItems",
                dsl::list(dsl::reference(r::ITEM_STACK)),
            )]),
            dsl::optional_fields(vec![("body_armor_item", dsl::reference(r::ITEM_STACK))]),
            dsl::optional_fields(vec![("saddle", dsl::reference(r::ITEM_STACK))]),
        ]),
    );
}

/// Applies `V100` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
