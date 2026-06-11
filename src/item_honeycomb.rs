//! Java `HoneycombItem`: copper block waxing plus the `SignApplicator`
//! contract for waxing signs.

use crate::block_behavior::BlockStateModel;
use crate::block_entity::SignBlockEntityModel;
use crate::block_placement::default_state;
use crate::block_update::BlockPos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoneycombUse {
    WaxBlock { pos: BlockPos, state: BlockStateModel },
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "Java WAXED_RECIPES parity surface is used by tests until recipe generation consumes it"
)]
pub struct WaxedRecipe {
    pub category: &'static str,
    pub group: &'static str,
}

pub fn use_on(old_state: &BlockStateModel, pos: BlockPos) -> HoneycombUse {
    get_waxed(old_state)
        .map(|state| HoneycombUse::WaxBlock { pos, state })
        .unwrap_or(HoneycombUse::Pass)
}

/// Java `HoneycombItem.getWaxed`: `WAXABLES.get(oldState.getBlock())` followed
/// by `Block.withPropertiesOf(oldState)`.
pub fn get_waxed(old_state: &BlockStateModel) -> Option<BlockStateModel> {
    waxed_block(&old_state.registry_id).map(|target| with_properties_of(target, old_state))
}

/// Java `SignApplicator.tryApplyToSign`: `sign.setWaxed(true)`, then a level
/// event when the state changed.
#[allow(
    dead_code,
    reason = "live sign item application is deferred behind TODO(live-sign-applicator-use)"
)]
pub fn try_apply_to_sign(sign: &mut SignBlockEntityModel) -> bool {
    sign.set_waxed(true)
}

/// Java `HoneycombItem.canApplyToSign` always returns true.
#[allow(
    dead_code,
    reason = "live sign item application is deferred behind TODO(live-sign-applicator-use)"
)]
pub const fn can_apply_to_sign() -> bool {
    true
}

/// Java `HoneycombItem.WAXED_RECIPES`: only the extra recipe groups whose
/// grouped result id differs from the vanilla default family recipe names.
#[allow(
    dead_code,
    reason = "Java WAXED_RECIPES parity surface is used by tests until recipe generation consumes it"
)]
pub fn waxed_recipe(block: &str) -> Option<WaxedRecipe> {
    let block = block.strip_prefix("minecraft:").unwrap_or(block);
    let redstone = match block {
        "waxed_copper_bulb" => Some("waxed_copper_bulb"),
        "waxed_weathered_copper_bulb" => Some("waxed_weathered_copper_bulb"),
        "waxed_exposed_copper_bulb" => Some("waxed_exposed_copper_bulb"),
        "waxed_oxidized_copper_bulb" => Some("waxed_oxidized_copper_bulb"),
        "waxed_copper_door"
        | "waxed_weathered_copper_door"
        | "waxed_exposed_copper_door"
        | "waxed_oxidized_copper_door" => Some("waxed_copper_door"),
        "waxed_copper_trapdoor"
        | "waxed_weathered_copper_trapdoor"
        | "waxed_exposed_copper_trapdoor"
        | "waxed_oxidized_copper_trapdoor" => Some("waxed_copper_trapdoor"),
        _ => None,
    };
    if let Some(group) = redstone {
        return Some(WaxedRecipe {
            category: "redstone",
            group,
        });
    }

    let building = match block {
        "waxed_copper_golem_statue"
        | "waxed_weathered_copper_golem_statue"
        | "waxed_exposed_copper_golem_statue"
        | "waxed_oxidized_copper_golem_statue" => Some("waxed_copper_golem_statue"),
        "waxed_copper_chest"
        | "waxed_weathered_copper_chest"
        | "waxed_exposed_copper_chest"
        | "waxed_oxidized_copper_chest" => Some("waxed_copper_chest"),
        "waxed_lightning_rod"
        | "waxed_weathered_lightning_rod"
        | "waxed_exposed_lightning_rod"
        | "waxed_oxidized_lightning_rod" => Some("waxed_lightning_rod"),
        "waxed_copper_bars"
        | "waxed_weathered_copper_bars"
        | "waxed_exposed_copper_bars"
        | "waxed_oxidized_copper_bars" => Some("waxed_copper_bar"),
        "waxed_copper_chain"
        | "waxed_weathered_copper_chain"
        | "waxed_exposed_copper_chain"
        | "waxed_oxidized_copper_chain" => Some("waxed_copper_chain"),
        "waxed_copper_lantern"
        | "waxed_weathered_copper_lantern"
        | "waxed_exposed_copper_lantern"
        | "waxed_oxidized_copper_lantern" => Some("waxed_copper_lantern"),
        "waxed_copper_block"
        | "waxed_weathered_copper"
        | "waxed_exposed_copper"
        | "waxed_oxidized_copper" => Some("waxed_copper_block"),
        _ => None,
    };
    building.map(|group| WaxedRecipe {
        category: "building_blocks",
        group,
    })
}

fn with_properties_of(target_block: &str, old_state: &BlockStateModel) -> BlockStateModel {
    let mut state = default_state(target_block);
    for (name, value) in old_state.get_values() {
        if state.has_property(name) {
            state = state.try_set_property(name, value);
        }
    }
    state
}

fn waxed_block(block: &str) -> Option<&'static str> {
    Some(match block {
        "minecraft:copper_block" => "minecraft:waxed_copper_block",
        "minecraft:exposed_copper" => "minecraft:waxed_exposed_copper",
        "minecraft:weathered_copper" => "minecraft:waxed_weathered_copper",
        "minecraft:oxidized_copper" => "minecraft:waxed_oxidized_copper",
        "minecraft:cut_copper" => "minecraft:waxed_cut_copper",
        "minecraft:exposed_cut_copper" => "minecraft:waxed_exposed_cut_copper",
        "minecraft:weathered_cut_copper" => "minecraft:waxed_weathered_cut_copper",
        "minecraft:oxidized_cut_copper" => "minecraft:waxed_oxidized_cut_copper",
        "minecraft:cut_copper_slab" => "minecraft:waxed_cut_copper_slab",
        "minecraft:exposed_cut_copper_slab" => "minecraft:waxed_exposed_cut_copper_slab",
        "minecraft:weathered_cut_copper_slab" => "minecraft:waxed_weathered_cut_copper_slab",
        "minecraft:oxidized_cut_copper_slab" => "minecraft:waxed_oxidized_cut_copper_slab",
        "minecraft:cut_copper_stairs" => "minecraft:waxed_cut_copper_stairs",
        "minecraft:exposed_cut_copper_stairs" => "minecraft:waxed_exposed_cut_copper_stairs",
        "minecraft:weathered_cut_copper_stairs" => "minecraft:waxed_weathered_cut_copper_stairs",
        "minecraft:oxidized_cut_copper_stairs" => "minecraft:waxed_oxidized_cut_copper_stairs",
        "minecraft:chiseled_copper" => "minecraft:waxed_chiseled_copper",
        "minecraft:exposed_chiseled_copper" => "minecraft:waxed_exposed_chiseled_copper",
        "minecraft:weathered_chiseled_copper" => "minecraft:waxed_weathered_chiseled_copper",
        "minecraft:oxidized_chiseled_copper" => "minecraft:waxed_oxidized_chiseled_copper",
        "minecraft:copper_door" => "minecraft:waxed_copper_door",
        "minecraft:exposed_copper_door" => "minecraft:waxed_exposed_copper_door",
        "minecraft:weathered_copper_door" => "minecraft:waxed_weathered_copper_door",
        "minecraft:oxidized_copper_door" => "minecraft:waxed_oxidized_copper_door",
        "minecraft:copper_trapdoor" => "minecraft:waxed_copper_trapdoor",
        "minecraft:exposed_copper_trapdoor" => "minecraft:waxed_exposed_copper_trapdoor",
        "minecraft:weathered_copper_trapdoor" => "minecraft:waxed_weathered_copper_trapdoor",
        "minecraft:oxidized_copper_trapdoor" => "minecraft:waxed_oxidized_copper_trapdoor",
        "minecraft:copper_bars" => "minecraft:waxed_copper_bars",
        "minecraft:exposed_copper_bars" => "minecraft:waxed_exposed_copper_bars",
        "minecraft:weathered_copper_bars" => "minecraft:waxed_weathered_copper_bars",
        "minecraft:oxidized_copper_bars" => "minecraft:waxed_oxidized_copper_bars",
        "minecraft:copper_grate" => "minecraft:waxed_copper_grate",
        "minecraft:exposed_copper_grate" => "minecraft:waxed_exposed_copper_grate",
        "minecraft:weathered_copper_grate" => "minecraft:waxed_weathered_copper_grate",
        "minecraft:oxidized_copper_grate" => "minecraft:waxed_oxidized_copper_grate",
        "minecraft:copper_bulb" => "minecraft:waxed_copper_bulb",
        "minecraft:exposed_copper_bulb" => "minecraft:waxed_exposed_copper_bulb",
        "minecraft:weathered_copper_bulb" => "minecraft:waxed_weathered_copper_bulb",
        "minecraft:oxidized_copper_bulb" => "minecraft:waxed_oxidized_copper_bulb",
        "minecraft:copper_chest" => "minecraft:waxed_copper_chest",
        "minecraft:exposed_copper_chest" => "minecraft:waxed_exposed_copper_chest",
        "minecraft:weathered_copper_chest" => "minecraft:waxed_weathered_copper_chest",
        "minecraft:oxidized_copper_chest" => "minecraft:waxed_oxidized_copper_chest",
        "minecraft:copper_golem_statue" => "minecraft:waxed_copper_golem_statue",
        "minecraft:exposed_copper_golem_statue" => "minecraft:waxed_exposed_copper_golem_statue",
        "minecraft:weathered_copper_golem_statue" => "minecraft:waxed_weathered_copper_golem_statue",
        "minecraft:oxidized_copper_golem_statue" => "minecraft:waxed_oxidized_copper_golem_statue",
        "minecraft:lightning_rod" => "minecraft:waxed_lightning_rod",
        "minecraft:exposed_lightning_rod" => "minecraft:waxed_exposed_lightning_rod",
        "minecraft:weathered_lightning_rod" => "minecraft:waxed_weathered_lightning_rod",
        "minecraft:oxidized_lightning_rod" => "minecraft:waxed_oxidized_lightning_rod",
        "minecraft:copper_lantern" => "minecraft:waxed_copper_lantern",
        "minecraft:exposed_copper_lantern" => "minecraft:waxed_exposed_copper_lantern",
        "minecraft:weathered_copper_lantern" => "minecraft:waxed_weathered_copper_lantern",
        "minecraft:oxidized_copper_lantern" => "minecraft:waxed_oxidized_copper_lantern",
        "minecraft:copper_chain" => "minecraft:waxed_copper_chain",
        "minecraft:exposed_copper_chain" => "minecraft:waxed_exposed_copper_chain",
        "minecraft:weathered_copper_chain" => "minecraft:waxed_weathered_copper_chain",
        "minecraft:oxidized_copper_chain" => "minecraft:waxed_oxidized_copper_chain",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const POS: BlockPos = BlockPos { x: 0, y: 64, z: 0 };

    #[test]
    fn get_waxed_preserves_properties_like_java_with_properties_of() {
        let waxed = get_waxed(
            &BlockStateModel::new("minecraft:weathered_cut_copper_stairs")
                .with_property("facing", "east")
                .with_property("half", "top")
                .with_property("shape", "inner_left")
                .with_property("waterlogged", "true"),
        )
        .unwrap();
        assert_eq!(waxed.registry_id, "minecraft:waxed_weathered_cut_copper_stairs");
        assert_eq!(waxed.property("facing"), Some("east"));
        assert_eq!(waxed.property("half"), Some("top"));
        assert_eq!(waxed.property("shape"), Some("inner_left"));
        assert_eq!(waxed.property("waterlogged"), Some("true"));
    }

    #[test]
    fn use_on_passes_for_non_waxable_blocks() {
        assert_eq!(
            use_on(&BlockStateModel::new("minecraft:stone"), POS),
            HoneycombUse::Pass
        );
    }

    #[test]
    fn sign_applicator_waxes_only_when_sign_changes() {
        let mut sign = SignBlockEntityModel::default();
        assert!(can_apply_to_sign());
        assert!(try_apply_to_sign(&mut sign));
        assert!(sign.is_waxed);
        assert!(!try_apply_to_sign(&mut sign));
    }

    #[test]
    fn waxed_recipe_groups_match_java_static_map() {
        assert_eq!(
            waxed_recipe("minecraft:waxed_oxidized_copper_bulb"),
            Some(WaxedRecipe {
                category: "redstone",
                group: "waxed_oxidized_copper_bulb",
            })
        );
        assert_eq!(
            waxed_recipe("minecraft:waxed_weathered_copper_chest"),
            Some(WaxedRecipe {
                category: "building_blocks",
                group: "waxed_copper_chest",
            })
        );
        assert_eq!(waxed_recipe("minecraft:waxed_cut_copper"), None);
    }
}
