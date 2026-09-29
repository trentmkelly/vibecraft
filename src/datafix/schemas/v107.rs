//! Port of `net.minecraft.util.datafix.schemas.V107`.

use crate::datafix::schema::Schema;
fn register_entities(schema: &mut Schema) {
    schema.remove_entity("Minecart");
}

/// Applies `V107` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
