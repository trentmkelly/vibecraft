//! Port of `net.minecraft.util.datafix.schemas.V1488`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_block_entities(schema: &mut Schema) {
    schema.register_block_entity(
        "minecraft:command_block",
        dsl::optional_fields(vec![
            ("CustomName", dsl::reference(r::TEXT_COMPONENT)),
            ("LastOutput", dsl::reference(r::TEXT_COMPONENT)),
        ]),
    );
}

/// Applies `V1488` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
}
