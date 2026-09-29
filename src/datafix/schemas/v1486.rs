//! Port of `net.minecraft.util.datafix.schemas.V1486`.

use crate::datafix::schema::Schema;
fn register_entities(schema: &mut Schema) {
    schema.rename_entity("minecraft:cod_mob", "minecraft:cod");
    schema.rename_entity("minecraft:salmon_mob", "minecraft:salmon");
}

/// Applies `V1486` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
