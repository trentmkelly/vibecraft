//! Port of `net.minecraft.util.datafix.schemas.V106`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::UNTAGGED_SPAWNER,
        dsl::optional_fields(vec![
            (
                "SpawnPotentials",
                dsl::list(dsl::fields(vec![(
                    "Entity",
                    dsl::reference(r::ENTITY_TREE),
                )])),
            ),
            ("SpawnData", dsl::reference(r::ENTITY_TREE)),
        ]),
    );
}

/// Applies `V106` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
