//! Port of `net.minecraft.util.datafix.schemas.V1125`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_block_entities(schema: &mut Schema) {
    schema.register_simple_block_entity("minecraft:bed");
}

fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::ADVANCEMENTS,
        dsl::optional_fields(vec![
            (
                "minecraft:adventure/adventuring_time",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::BIOME), dsl::string()),
                )]),
            ),
            (
                "minecraft:adventure/kill_a_mob",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::string()),
                )]),
            ),
            (
                "minecraft:adventure/kill_all_mobs",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::string()),
                )]),
            ),
            (
                "minecraft:husbandry/bred_all_animals",
                dsl::optional_fields(vec![(
                    "criteria",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::string()),
                )]),
            ),
        ]),
    );
    schema.register_type(r::BIOME, dsl::namespaced_string());
    schema.register_type(r::ENTITY_NAME, dsl::namespaced_string());
}

/// Applies `V1125` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
    register_types(schema);
}
