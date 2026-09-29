//! Port of `net.minecraft.util.datafix.schemas.V700`.

use crate::datafix::schema::Schema;
fn register_entities(schema: &mut Schema) {
    schema.register_simple_entity("ElderGuardian");
}

/// Applies `V700` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
