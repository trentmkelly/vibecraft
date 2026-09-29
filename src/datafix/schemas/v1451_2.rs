//! Port of `net.minecraft.util.datafix.schemas.V1451_2`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_block_entities(schema: &mut Schema) {
    schema.register_block_entity(
        "minecraft:piston",
        dsl::optional_fields(vec![("blockState", dsl::reference(r::BLOCK_STATE))]),
    );
}

/// Applies `V1451_2` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
}
