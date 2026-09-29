//! Port of `net.minecraft.util.datafix.schemas.V1451_4`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_types(schema: &mut Schema) {
    schema.register_type(r::BLOCK_NAME, dsl::namespaced_string());
}

/// Applies `V1451_4` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
