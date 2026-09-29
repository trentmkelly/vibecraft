//! Port of `net.minecraft.util.datafix.schemas.V102`.

use super::v99;
use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::ITEM_STACK,
        dsl::hook(
            dsl::optional_fields(vec![
                ("id", dsl::reference(r::ITEM_NAME)),
                ("tag", v99::item_stack_tag()),
            ]),
            v99::add_names,
        ),
    );
}

/// Applies `V102` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
