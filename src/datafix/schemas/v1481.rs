//! Port of `net.minecraft.util.datafix.schemas.V1481`.

use crate::datafix::schema::Schema;
fn register_block_entities(schema: &mut Schema) {
    schema.register_simple_block_entity("minecraft:conduit");
}

/// Applies `V1481` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
}
