//! Port of `net.minecraft.util.datafix.schemas.V1451_1`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::CHUNK,
        dsl::fields(vec![(
            "Level",
            dsl::optional_fields(vec![
                ("Entities", dsl::list(dsl::reference(r::ENTITY_TREE))),
                (
                    "TileEntities",
                    dsl::list(dsl::or(dsl::reference(r::BLOCK_ENTITY), dsl::remainder())),
                ),
                (
                    "TileTicks",
                    dsl::list(dsl::fields(vec![("i", dsl::reference(r::BLOCK_NAME))])),
                ),
                (
                    "Sections",
                    dsl::list(dsl::optional_fields(vec![(
                        "Palette",
                        dsl::list(dsl::reference(r::BLOCK_STATE)),
                    )])),
                ),
            ]),
        )]),
    );
}

/// Applies `V1451_1` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
