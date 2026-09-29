//! Port of `net.minecraft.util.datafix.fixes.EntityIdFix`: legacy entity ids
//! become namespaced ids.

use crate::datafix::dynamic::{get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::template::ChoiceSet;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::entity_id_fix_ids::ID_MAP;

/// `new EntityIdFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::sequence(
        "EntityIdFix",
        vec![
            // `convertUnchecked`: a type-only conversion of the item stack, no data change.
            Fix::everywhere(
                "item stack entity name hook converter",
                Target::Type(r::ITEM_STACK),
                |_| {},
            ),
            Fix::everywhere(
                "EntityIdFix",
                Target::AnyChoice(ChoiceSet::Entities),
                |entity| {
                    let renamed = get_str(entity, "id").and_then(|id| {
                        ID_MAP
                            .iter()
                            .find(|(old, _)| *old == id)
                            .map(|(_, new)| (*new).to_string())
                    });
                    if let Some(new_id) = renamed {
                        set(entity, "id", Tag::String(new_id));
                    }
                },
            ),
        ],
    )
}
