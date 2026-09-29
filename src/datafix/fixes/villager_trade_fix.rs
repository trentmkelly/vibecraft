//! Port of `net.minecraft.util.datafix.fixes.VillagerTradeFix`.

use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new VillagerTradeFix(schema)`: carved pumpkins in trades become pumpkins.
pub fn fix() -> Fix {
    Fix::everywhere(
        "Villager trade fix",
        Target::Type(r::VILLAGER_TRADE),
        |trade| {
            for field in ["buy", "buyB", "sell"] {
                let Some(item_stack) = typed_mut(trade, field) else {
                    continue;
                };
                if let Some(Tag::String(id)) = typed_mut(item_stack, "id") {
                    if id == "minecraft:carved_pumpkin" {
                        *id = "minecraft:pumpkin".to_string();
                    }
                }
            }
        },
    )
}
