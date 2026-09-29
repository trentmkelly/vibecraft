//! Port of `net.minecraft.util.datafix.schemas.V1458`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::{dsl, ChoiceSet, Tmpl};
fn register_block_entities(schema: &mut Schema) {
    schema.register_block_entity("minecraft:beacon", nameable());
    schema.register_block_entity("minecraft:banner", nameable());
    schema.register_block_entity("minecraft:brewing_stand", nameable_inventory());
    schema.register_block_entity("minecraft:chest", nameable_inventory());
    schema.register_block_entity("minecraft:trapped_chest", nameable_inventory());
    schema.register_block_entity("minecraft:dispenser", nameable_inventory());
    schema.register_block_entity("minecraft:dropper", nameable_inventory());
    schema.register_block_entity("minecraft:enchanting_table", nameable());
    schema.register_block_entity("minecraft:furnace", nameable_inventory());
    schema.register_block_entity("minecraft:hopper", nameable_inventory());
    schema.register_block_entity("minecraft:shulker_box", nameable_inventory());
}

fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::ENTITY,
        dsl::and(vec![
            dsl::reference(r::ENTITY_EQUIPMENT),
            dsl::optional_fields_with_rest(
                vec![("CustomName", dsl::reference(r::TEXT_COMPONENT))],
                dsl::namespaced_choice(ChoiceSet::Entities),
            ),
        ]),
    );
}

/// Applies `V1458` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
    register_types(schema);
}

/// `V1458.nameableInventory`.
pub fn nameable_inventory() -> Tmpl {
    dsl::optional_fields(vec![
        ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
        ("CustomName", dsl::reference(r::TEXT_COMPONENT)),
    ])
}

/// `V1458.nameable`.
pub fn nameable() -> Tmpl {
    dsl::optional_fields(vec![("CustomName", dsl::reference(r::TEXT_COMPONENT))])
}
