//! Port of `net.minecraft.util.datafix.fixes.BlockEntityIdFix`: legacy block
//! entity ids become namespaced ids.

use crate::datafix::dynamic::{get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::template::ChoiceSet;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `BlockEntityIdFix.ID_MAP`.
pub const ID_MAP: &[(&str, &str)] = &[
    ("Airportal", "minecraft:end_portal"),
    ("Banner", "minecraft:banner"),
    ("Beacon", "minecraft:beacon"),
    ("Cauldron", "minecraft:brewing_stand"),
    ("Chest", "minecraft:chest"),
    ("Comparator", "minecraft:comparator"),
    ("Control", "minecraft:command_block"),
    ("DLDetector", "minecraft:daylight_detector"),
    ("Dropper", "minecraft:dropper"),
    ("EnchantTable", "minecraft:enchanting_table"),
    ("EndGateway", "minecraft:end_gateway"),
    ("EnderChest", "minecraft:ender_chest"),
    ("FlowerPot", "minecraft:flower_pot"),
    ("Furnace", "minecraft:furnace"),
    ("Hopper", "minecraft:hopper"),
    ("MobSpawner", "minecraft:mob_spawner"),
    ("Music", "minecraft:noteblock"),
    ("Piston", "minecraft:piston"),
    ("RecordPlayer", "minecraft:jukebox"),
    ("Sign", "minecraft:sign"),
    ("Skull", "minecraft:skull"),
    ("Structure", "minecraft:structure_block"),
    ("Trap", "minecraft:dispenser"),
];

/// `new BlockEntityIdFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::sequence(
        "BlockEntityIdFix",
        vec![
            // `convertUnchecked`: a type-only conversion of the item stack, no data change.
            Fix::everywhere(
                "item stack block entity name hook converter",
                Target::Type(r::ITEM_STACK),
                |_| {},
            ),
            Fix::everywhere(
                "BlockEntityIdFix",
                Target::AnyChoice(ChoiceSet::BlockEntities),
                |block_entity| {
                    let renamed = get_str(block_entity, "id").and_then(|id| {
                        ID_MAP
                            .iter()
                            .find(|(old, _)| *old == id)
                            .map(|(_, new)| (*new).to_string())
                    });
                    if let Some(new_id) = renamed {
                        set(block_entity, "id", Tag::String(new_id));
                    }
                },
            ),
        ],
    )
}
