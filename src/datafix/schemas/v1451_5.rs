//! Port of `net.minecraft.util.datafix.schemas.V1451_5`.

use crate::datafix::schema::Schema;
fn register_block_entities(schema: &mut Schema) {
    schema.remove_block_entity("minecraft:flower_pot");
    schema.remove_block_entity("minecraft:noteblock");
}

/// Applies `V1451_5` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
}
