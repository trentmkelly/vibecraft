//! Port of `net.minecraft.util.datafix.schemas.V1483`.

use crate::datafix::schema::Schema;
fn register_entities(schema: &mut Schema) {
    schema.rename_entity("minecraft:puffer_fish", "minecraft:pufferfish");
}

/// Applies `V1483` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
