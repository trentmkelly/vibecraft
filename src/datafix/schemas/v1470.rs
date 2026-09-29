//! Port of `net.minecraft.util.datafix.schemas.V1470`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_entities(schema: &mut Schema) {
    schema.register_simple_entity("minecraft:turtle");
    schema.register_simple_entity("minecraft:cod_mob");
    schema.register_simple_entity("minecraft:tropical_fish");
    schema.register_simple_entity("minecraft:salmon_mob");
    schema.register_simple_entity("minecraft:puffer_fish");
    schema.register_simple_entity("minecraft:phantom");
    schema.register_simple_entity("minecraft:dolphin");
    schema.register_simple_entity("minecraft:drowned");
    schema.register_entity(
        "minecraft:trident",
        dsl::optional_fields(vec![
            ("inBlockState", dsl::reference(r::BLOCK_STATE)),
            ("Trident", dsl::reference(r::ITEM_STACK)),
        ]),
    );
}

/// Applies `V1470` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
