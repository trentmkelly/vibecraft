//! Port of `net.minecraft.util.datafix.schemas.V808`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_block_entities(schema: &mut Schema) {
    schema.register_block_entity(
        "minecraft:shulker_box",
        dsl::optional_fields(vec![("Items", dsl::list(dsl::reference(r::ITEM_STACK)))]),
    );
}

/// Applies `V808` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
}
