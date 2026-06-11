//! Java tool `Item.useOn` behavior for block-state mutation tools:
//! `AxeItem`, `ShovelItem`, and `HoeItem`.
//!
//! This module is the pure decision half. Live world writes, packets, item
//! durability, and item-entity spawning are wired in `network::status::item_use_live`.

use crate::block_behavior::BlockStateModel;
use crate::block_placement::default_state;
use crate::block_placement::PlacementWorld;
use crate::block_update::{BlockPos, Direction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolUse {
    ChangeBlock {
        pos: BlockPos,
        state: BlockStateModel,
        drop: Option<ToolDrop>,
    },
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolDrop {
    pub item: &'static str,
    pub face: Direction,
}

pub fn axe_use_on(world: &impl PlacementWorld, pos: BlockPos) -> ToolUse {
    let old_state = world.state_at(pos);
    let Some(new_state) = axe_replacement_state(&old_state) else {
        return ToolUse::Pass;
    };
    ToolUse::ChangeBlock {
        pos,
        state: new_state,
        drop: None,
    }
}

pub fn shovel_use_on(
    world: &impl PlacementWorld,
    pos: BlockPos,
    clicked_face: Direction,
) -> ToolUse {
    let old_state = world.state_at(pos);
    if clicked_face != Direction::Down {
        if let Some(new_block) = shovel_flattened_block(&old_state.registry_id) {
            if world.state_at(pos.relative(Direction::Up)).is_air() {
                return ToolUse::ChangeBlock {
                    pos,
                    state: default_state(new_block),
                    drop: None,
                };
            }
        }
    }

    if block_type_is(&old_state, "campfire")
        && old_state.property("lit") == Some("true")
    {
        return ToolUse::ChangeBlock {
            pos,
            state: old_state.try_set_property("lit", "false"),
            drop: None,
        };
    }

    ToolUse::Pass
}

pub fn hoe_use_on(world: &impl PlacementWorld, pos: BlockPos, clicked_face: Direction) -> ToolUse {
    let old_state = world.state_at(pos);
    let Some((target, needs_air_above, drop_item)) = hoe_till_result(&old_state.registry_id) else {
        return ToolUse::Pass;
    };
    if needs_air_above
        && (clicked_face == Direction::Down || !world.state_at(pos.relative(Direction::Up)).is_air())
    {
        return ToolUse::Pass;
    }
    ToolUse::ChangeBlock {
        pos,
        state: default_state(target),
        drop: drop_item.map(|item| ToolDrop {
            item,
            face: clicked_face,
        }),
    }
}

pub fn axe_replacement_state(old_state: &BlockStateModel) -> Option<BlockStateModel> {
    if let Some(stripped) = stripped_block(&old_state.registry_id) {
        let mut state = default_state(stripped);
        if let Some(axis) = old_state.property("axis") {
            state = state.try_set_property("axis", axis);
        }
        return Some(state);
    }
    if let Some(previous) = previous_weathered_copper(&old_state.registry_id) {
        return Some(with_properties_of(previous, old_state));
    }
    if let Some(unwaxed) = wax_off_block(&old_state.registry_id) {
        return Some(with_properties_of(unwaxed, old_state));
    }
    None
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

fn stripped_block(block: &str) -> Option<&'static str> {
    Some(match block {
        "minecraft:oak_wood" => "minecraft:stripped_oak_wood",
        "minecraft:oak_log" => "minecraft:stripped_oak_log",
        "minecraft:dark_oak_wood" => "minecraft:stripped_dark_oak_wood",
        "minecraft:dark_oak_log" => "minecraft:stripped_dark_oak_log",
        "minecraft:pale_oak_wood" => "minecraft:stripped_pale_oak_wood",
        "minecraft:pale_oak_log" => "minecraft:stripped_pale_oak_log",
        "minecraft:acacia_wood" => "minecraft:stripped_acacia_wood",
        "minecraft:acacia_log" => "minecraft:stripped_acacia_log",
        "minecraft:cherry_wood" => "minecraft:stripped_cherry_wood",
        "minecraft:cherry_log" => "minecraft:stripped_cherry_log",
        "minecraft:birch_wood" => "minecraft:stripped_birch_wood",
        "minecraft:birch_log" => "minecraft:stripped_birch_log",
        "minecraft:jungle_wood" => "minecraft:stripped_jungle_wood",
        "minecraft:jungle_log" => "minecraft:stripped_jungle_log",
        "minecraft:spruce_wood" => "minecraft:stripped_spruce_wood",
        "minecraft:spruce_log" => "minecraft:stripped_spruce_log",
        "minecraft:warped_stem" => "minecraft:stripped_warped_stem",
        "minecraft:warped_hyphae" => "minecraft:stripped_warped_hyphae",
        "minecraft:crimson_stem" => "minecraft:stripped_crimson_stem",
        "minecraft:crimson_hyphae" => "minecraft:stripped_crimson_hyphae",
        "minecraft:mangrove_wood" => "minecraft:stripped_mangrove_wood",
        "minecraft:mangrove_log" => "minecraft:stripped_mangrove_log",
        "minecraft:bamboo_block" => "minecraft:stripped_bamboo_block",
        _ => return None,
    })
}

fn previous_weathered_copper(block: &str) -> Option<&'static str> {
    let rest = block.strip_prefix("minecraft:")?;
    let previous = match rest {
        "exposed_copper" => "copper_block",
        "weathered_copper" => "exposed_copper",
        "oxidized_copper" => "weathered_copper",
        "exposed_cut_copper" => "cut_copper",
        "weathered_cut_copper" => "exposed_cut_copper",
        "oxidized_cut_copper" => "weathered_cut_copper",
        "exposed_chiseled_copper" => "chiseled_copper",
        "weathered_chiseled_copper" => "exposed_chiseled_copper",
        "oxidized_chiseled_copper" => "weathered_chiseled_copper",
        "exposed_cut_copper_slab" => "cut_copper_slab",
        "weathered_cut_copper_slab" => "exposed_cut_copper_slab",
        "oxidized_cut_copper_slab" => "weathered_cut_copper_slab",
        "exposed_cut_copper_stairs" => "cut_copper_stairs",
        "weathered_cut_copper_stairs" => "exposed_cut_copper_stairs",
        "oxidized_cut_copper_stairs" => "weathered_cut_copper_stairs",
        "exposed_copper_door" => "copper_door",
        "weathered_copper_door" => "exposed_copper_door",
        "oxidized_copper_door" => "weathered_copper_door",
        "exposed_copper_trapdoor" => "copper_trapdoor",
        "weathered_copper_trapdoor" => "exposed_copper_trapdoor",
        "oxidized_copper_trapdoor" => "weathered_copper_trapdoor",
        "exposed_copper_bars" => "copper_bars",
        "weathered_copper_bars" => "exposed_copper_bars",
        "oxidized_copper_bars" => "weathered_copper_bars",
        "exposed_copper_grate" => "copper_grate",
        "weathered_copper_grate" => "exposed_copper_grate",
        "oxidized_copper_grate" => "weathered_copper_grate",
        "exposed_copper_bulb" => "copper_bulb",
        "weathered_copper_bulb" => "exposed_copper_bulb",
        "oxidized_copper_bulb" => "weathered_copper_bulb",
        "exposed_copper_lantern" => "copper_lantern",
        "weathered_copper_lantern" => "exposed_copper_lantern",
        "oxidized_copper_lantern" => "weathered_copper_lantern",
        "exposed_copper_chest" => "copper_chest",
        "weathered_copper_chest" => "exposed_copper_chest",
        "oxidized_copper_chest" => "weathered_copper_chest",
        "exposed_copper_golem_statue" => "copper_golem_statue",
        "weathered_copper_golem_statue" => "exposed_copper_golem_statue",
        "oxidized_copper_golem_statue" => "weathered_copper_golem_statue",
        "exposed_lightning_rod" => "lightning_rod",
        "weathered_lightning_rod" => "exposed_lightning_rod",
        "oxidized_lightning_rod" => "weathered_lightning_rod",
        "exposed_copper_chain" => "copper_chain",
        "weathered_copper_chain" => "exposed_copper_chain",
        "oxidized_copper_chain" => "weathered_copper_chain",
        _ => return None,
    };
    copper_block_id(previous)
}

fn wax_off_block(block: &str) -> Option<&'static str> {
    let rest = block.strip_prefix("minecraft:waxed_")?;
    copper_block_id(rest)
}

fn copper_block_id(name: &str) -> Option<&'static str> {
    Some(match name {
        "copper_block" => "minecraft:copper_block",
        "exposed_copper" => "minecraft:exposed_copper",
        "weathered_copper" => "minecraft:weathered_copper",
        "oxidized_copper" => "minecraft:oxidized_copper",
        "cut_copper" => "minecraft:cut_copper",
        "exposed_cut_copper" => "minecraft:exposed_cut_copper",
        "weathered_cut_copper" => "minecraft:weathered_cut_copper",
        "oxidized_cut_copper" => "minecraft:oxidized_cut_copper",
        "chiseled_copper" => "minecraft:chiseled_copper",
        "exposed_chiseled_copper" => "minecraft:exposed_chiseled_copper",
        "weathered_chiseled_copper" => "minecraft:weathered_chiseled_copper",
        "oxidized_chiseled_copper" => "minecraft:oxidized_chiseled_copper",
        "cut_copper_slab" => "minecraft:cut_copper_slab",
        "exposed_cut_copper_slab" => "minecraft:exposed_cut_copper_slab",
        "weathered_cut_copper_slab" => "minecraft:weathered_cut_copper_slab",
        "oxidized_cut_copper_slab" => "minecraft:oxidized_cut_copper_slab",
        "cut_copper_stairs" => "minecraft:cut_copper_stairs",
        "exposed_cut_copper_stairs" => "minecraft:exposed_cut_copper_stairs",
        "weathered_cut_copper_stairs" => "minecraft:weathered_cut_copper_stairs",
        "oxidized_cut_copper_stairs" => "minecraft:oxidized_cut_copper_stairs",
        "copper_door" => "minecraft:copper_door",
        "exposed_copper_door" => "minecraft:exposed_copper_door",
        "weathered_copper_door" => "minecraft:weathered_copper_door",
        "oxidized_copper_door" => "minecraft:oxidized_copper_door",
        "copper_trapdoor" => "minecraft:copper_trapdoor",
        "exposed_copper_trapdoor" => "minecraft:exposed_copper_trapdoor",
        "weathered_copper_trapdoor" => "minecraft:weathered_copper_trapdoor",
        "oxidized_copper_trapdoor" => "minecraft:oxidized_copper_trapdoor",
        "copper_bars" => "minecraft:copper_bars",
        "exposed_copper_bars" => "minecraft:exposed_copper_bars",
        "weathered_copper_bars" => "minecraft:weathered_copper_bars",
        "oxidized_copper_bars" => "minecraft:oxidized_copper_bars",
        "copper_grate" => "minecraft:copper_grate",
        "exposed_copper_grate" => "minecraft:exposed_copper_grate",
        "weathered_copper_grate" => "minecraft:weathered_copper_grate",
        "oxidized_copper_grate" => "minecraft:oxidized_copper_grate",
        "copper_bulb" => "minecraft:copper_bulb",
        "exposed_copper_bulb" => "minecraft:exposed_copper_bulb",
        "weathered_copper_bulb" => "minecraft:weathered_copper_bulb",
        "oxidized_copper_bulb" => "minecraft:oxidized_copper_bulb",
        "copper_lantern" => "minecraft:copper_lantern",
        "exposed_copper_lantern" => "minecraft:exposed_copper_lantern",
        "weathered_copper_lantern" => "minecraft:weathered_copper_lantern",
        "oxidized_copper_lantern" => "minecraft:oxidized_copper_lantern",
        "copper_chest" => "minecraft:copper_chest",
        "exposed_copper_chest" => "minecraft:exposed_copper_chest",
        "weathered_copper_chest" => "minecraft:weathered_copper_chest",
        "oxidized_copper_chest" => "minecraft:oxidized_copper_chest",
        "copper_golem_statue" => "minecraft:copper_golem_statue",
        "exposed_copper_golem_statue" => "minecraft:exposed_copper_golem_statue",
        "weathered_copper_golem_statue" => "minecraft:weathered_copper_golem_statue",
        "oxidized_copper_golem_statue" => "minecraft:oxidized_copper_golem_statue",
        "lightning_rod" => "minecraft:lightning_rod",
        "exposed_lightning_rod" => "minecraft:exposed_lightning_rod",
        "weathered_lightning_rod" => "minecraft:weathered_lightning_rod",
        "oxidized_lightning_rod" => "minecraft:oxidized_lightning_rod",
        "copper_chain" => "minecraft:copper_chain",
        "exposed_copper_chain" => "minecraft:exposed_copper_chain",
        "weathered_copper_chain" => "minecraft:weathered_copper_chain",
        "oxidized_copper_chain" => "minecraft:oxidized_copper_chain",
        _ => return None,
    })
}

fn shovel_flattened_block(block: &str) -> Option<&'static str> {
    Some(match block {
        "minecraft:grass_block"
        | "minecraft:dirt"
        | "minecraft:podzol"
        | "minecraft:coarse_dirt"
        | "minecraft:mycelium"
        | "minecraft:rooted_dirt" => "minecraft:dirt_path",
        _ => return None,
    })
}

fn hoe_till_result(block: &str) -> Option<(&'static str, bool, Option<&'static str>)> {
    Some(match block {
        "minecraft:grass_block" | "minecraft:dirt_path" | "minecraft:dirt" => {
            ("minecraft:farmland", true, None)
        }
        "minecraft:coarse_dirt" => ("minecraft:dirt", true, None),
        "minecraft:rooted_dirt" => ("minecraft:dirt", false, Some("minecraft:hanging_roots")),
        _ => return None,
    })
}

fn block_type_is(state: &BlockStateModel, block_type: &str) -> bool {
    crate::block_states::block_state_entry(&state.registry_id)
        .is_some_and(|entry| entry.block_type == block_type)
}

pub fn is_axe(item: &str) -> bool {
    crate::item_tags::item_in_tag(item, "minecraft:axes")
}

pub fn is_shovel(item: &str) -> bool {
    crate::item_tags::item_in_tag(item, "minecraft:shovels")
}

pub fn is_hoe(item: &str) -> bool {
    crate::item_tags::item_in_tag(item, "minecraft:hoes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_survival::SurvivalWorld;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestWorld {
        blocks: HashMap<(i32, i32, i32), BlockStateModel>,
    }

    impl TestWorld {
        fn with(mut self, pos: (i32, i32, i32), state: BlockStateModel) -> Self {
            self.blocks.insert(pos, state);
            self
        }
    }

    impl SurvivalWorld for TestWorld {
        fn state_at(&self, pos: BlockPos) -> BlockStateModel {
            self.blocks
                .get(&(pos.x, pos.y, pos.z))
                .cloned()
                .unwrap_or_else(BlockStateModel::air)
        }

        fn raw_brightness(&self, _pos: BlockPos) -> i32 {
            15
        }
    }

    impl PlacementWorld for TestWorld {
        fn has_neighbor_signal(&self, _pos: BlockPos) -> bool {
            false
        }
    }

    const POS: BlockPos = BlockPos { x: 1, y: 64, z: 1 };

    #[test]
    fn axe_strips_logs_and_preserves_axis_like_java() {
        let world = TestWorld::default().with(
            (1, 64, 1),
            BlockStateModel::new("minecraft:oak_log").with_property("axis", "x"),
        );
        let ToolUse::ChangeBlock { state, .. } = axe_use_on(&world, POS) else {
            panic!("expected axe change");
        };
        assert_eq!(state.registry_id, "minecraft:stripped_oak_log");
        assert_eq!(state.property("axis"), Some("x"));
    }

    #[test]
    fn axe_scrapes_copper_and_removes_wax_preserving_shared_properties() {
        let scraped = axe_replacement_state(
            &BlockStateModel::new("minecraft:oxidized_cut_copper_stairs")
                .with_property("facing", "east")
                .with_property("half", "top")
                .with_property("shape", "straight")
                .with_property("waterlogged", "true"),
        )
        .unwrap();
        assert_eq!(scraped.registry_id, "minecraft:weathered_cut_copper_stairs");
        assert_eq!(scraped.property("facing"), Some("east"));
        assert_eq!(scraped.property("waterlogged"), Some("true"));

        let unwaxed = axe_replacement_state(
            &BlockStateModel::new("minecraft:waxed_weathered_copper_bulb")
                .with_property("lit", "true")
                .with_property("powered", "false"),
        )
        .unwrap();
        assert_eq!(unwaxed.registry_id, "minecraft:weathered_copper_bulb");
        assert_eq!(unwaxed.property("lit"), Some("true"));
    }

    #[test]
    fn shovel_flattens_only_when_clicked_face_and_air_above_allow_it() {
        let world = TestWorld::default().with((1, 64, 1), BlockStateModel::new("minecraft:dirt"));
        assert!(matches!(
            shovel_use_on(&world, POS, Direction::Up),
            ToolUse::ChangeBlock { state, .. } if state.registry_id == "minecraft:dirt_path"
        ));
        assert_eq!(shovel_use_on(&world, POS, Direction::Down), ToolUse::Pass);

        let blocked = TestWorld::default()
            .with((1, 64, 1), BlockStateModel::new("minecraft:dirt"))
            .with((1, 65, 1), BlockStateModel::new("minecraft:stone"));
        assert_eq!(shovel_use_on(&blocked, POS, Direction::Up), ToolUse::Pass);
    }

    #[test]
    fn shovel_dowses_lit_campfires_even_when_clicked_down() {
        let world = TestWorld::default().with(
            (1, 64, 1),
            BlockStateModel::new("minecraft:campfire")
                .with_property("lit", "true")
                .with_property("waterlogged", "false"),
        );
        let ToolUse::ChangeBlock { state, .. } = shovel_use_on(&world, POS, Direction::Down) else {
            panic!("expected campfire dowse");
        };
        assert_eq!(state.property("lit"), Some("false"));
    }

    #[test]
    fn hoe_tills_air_gated_blocks_and_rooted_dirt_drop_like_java() {
        let world =
            TestWorld::default().with((1, 64, 1), BlockStateModel::new("minecraft:grass_block"));
        assert!(matches!(
            hoe_use_on(&world, POS, Direction::North),
            ToolUse::ChangeBlock { state, drop: None, .. } if state.registry_id == "minecraft:farmland"
        ));
        assert_eq!(hoe_use_on(&world, POS, Direction::Down), ToolUse::Pass);

        let rooted =
            TestWorld::default().with((1, 64, 1), BlockStateModel::new("minecraft:rooted_dirt"));
        assert_eq!(
            hoe_use_on(&rooted, POS, Direction::East),
            ToolUse::ChangeBlock {
                pos: POS,
                state: default_state("minecraft:dirt"),
                drop: Some(ToolDrop {
                    item: "minecraft:hanging_roots",
                    face: Direction::East,
                }),
            }
        );
    }
}
