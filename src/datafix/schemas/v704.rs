//! Port of `net.minecraft.util.datafix.schemas.V704`.

use crate::datafix::fixes::block_entity_id_fix::ID_MAP;
use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::{dsl, ChoiceSet};
use crate::storage::nbt::Tag;

use super::v704_item_to_block_entity::ITEM_TO_BLOCK_ENTITY;
use super::v99;

/// `V704.ADD_NAMES`: `V99.addNames` with the namespaced block entity table.
pub fn add_names(input: &mut Tag) {
    v99::add_names_with(input, ITEM_TO_BLOCK_ENTITY, v99::ITEM_TO_ENTITY);
}

/// `V704.registerBlockEntities`: legacy ids are re-keyed through
/// `BlockEntityIdFix.ID_MAP`.
fn register_block_entities(schema: &mut Schema) {
    for (old_id, new_id) in ID_MAP {
        let template = schema
            .remove_block_entity(old_id)
            .unwrap_or_else(|| panic!("Didn't find {old_id} in schema"));
        schema.register_block_entity(new_id, template);
    }
}

/// `V704.registerTypes`.
fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::BLOCK_ENTITY,
        dsl::optional_fields_with_rest(
            vec![("components", dsl::reference(r::DATA_COMPONENTS))],
            dsl::namespaced_choice(ChoiceSet::BlockEntities),
        ),
    );
    schema.register_type(
        r::ITEM_STACK,
        dsl::hook(
            dsl::optional_fields(vec![
                ("id", dsl::reference(r::ITEM_NAME)),
                ("tag", v99::item_stack_tag()),
            ]),
            add_names,
        ),
    );
}

/// Applies `V704` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_block_entities(schema);
    register_types(schema);
}
