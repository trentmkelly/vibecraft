use super::*;

pub fn block_predicate_type(id: &str) -> Option<&'static BlockPredicateType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BLOCK_PREDICATE_TYPES.iter().find(|predicate_type| {
        predicate_type
            .id
            .strip_prefix("minecraft:")
            .unwrap_or(predicate_type.id)
            == name
    })
}

pub fn block_predicate_test(
    predicate: BlockPredicate,
    context: BlockPredicateContext,
    origin_y: i32,
) -> bool {
    block_predicate_test_with_vertical_context(predicate, context, context, origin_y)
}

pub fn block_predicate_test_with_vertical_context(
    predicate: BlockPredicate,
    origin_context: BlockPredicateContext,
    offset_context: BlockPredicateContext,
    origin_y: i32,
) -> bool {
    match predicate {
        BlockPredicate::MatchingBlocks { blocks } => blocks.contains(&origin_context.block),
        BlockPredicate::MatchingBlocksAt { offset_y, blocks } => {
            block_predicate_offset_context(offset_context, origin_y, offset_y)
                .is_some_and(|offset_context| blocks.contains(&offset_context.block))
        }
        BlockPredicate::MatchingBlockTag { tag } => block_matches_tag(origin_context.block, tag),
        BlockPredicate::MatchingFluids { fluids } => fluids.contains(&origin_context.fluid),
        BlockPredicate::MatchingFluidsAt { offset_y, fluids } => {
            block_predicate_offset_context(offset_context, origin_y, offset_y)
                .is_some_and(|offset_context| fluids.contains(&offset_context.fluid))
        }
        BlockPredicate::Solid => origin_context.solid,
        BlockPredicate::SolidAt { offset_y } => {
            block_predicate_offset_context(offset_context, origin_y, offset_y)
                .is_some_and(|offset_context| offset_context.solid)
        }
        BlockPredicate::Replaceable => origin_context.replaceable,
        BlockPredicate::ReplaceableAt { offset_y } => {
            block_predicate_offset_context(offset_context, origin_y, offset_y)
                .is_some_and(|offset_context| offset_context.replaceable)
        }
        BlockPredicate::WouldSurvive {
            offset_y,
            state: _,
            survives,
        } => block_predicate_offset_context(offset_context, origin_y, offset_y)
            .is_some_and(|_| survives),
        BlockPredicate::HasSturdyFace {
            offset_y,
            direction: _,
            sturdy,
        } => block_predicate_offset_context(offset_context, origin_y, offset_y)
            .is_some_and(|_| sturdy),
        BlockPredicate::InsideWorldBounds { offset_y } => {
            let y = origin_y + offset_y;
            y >= origin_context.min_y && y < origin_context.min_y + origin_context.height
        }
        BlockPredicate::AnyOf { predicates } => predicates.iter().any(|predicate| {
            block_predicate_test_with_vertical_context(
                *predicate,
                origin_context,
                offset_context,
                origin_y,
            )
        }),
        BlockPredicate::AllOf { predicates } => predicates.iter().all(|predicate| {
            block_predicate_test_with_vertical_context(
                *predicate,
                origin_context,
                offset_context,
                origin_y,
            )
        }),
        BlockPredicate::Not { predicate } => !block_predicate_test_with_vertical_context(
            *predicate,
            origin_context,
            offset_context,
            origin_y,
        ),
        BlockPredicate::True => true,
        BlockPredicate::Unobstructed => origin_context.unobstructed,
    }
}

fn block_predicate_offset_context(
    context: BlockPredicateContext,
    origin_y: i32,
    offset_y: i32,
) -> Option<BlockPredicateContext> {
    let y = origin_y + offset_y;
    (y >= context.min_y && y < context.min_y + context.height).then_some(context)
}

fn block_is_log(block: &str) -> bool {
    matches!(
        block,
        "minecraft:oak_log"
            | "minecraft:spruce_log"
            | "minecraft:birch_log"
            | "minecraft:jungle_log"
            | "minecraft:acacia_log"
            | "minecraft:dark_oak_log"
            | "minecraft:mangrove_log"
            | "minecraft:cherry_log"
            | "minecraft:pale_oak_log"
            | "minecraft:crimson_stem"
            | "minecraft:warped_stem"
            | "minecraft:stripped_oak_log"
            | "minecraft:stripped_spruce_log"
            | "minecraft:stripped_birch_log"
            | "minecraft:stripped_jungle_log"
            | "minecraft:stripped_acacia_log"
            | "minecraft:stripped_dark_oak_log"
            | "minecraft:stripped_mangrove_log"
            | "minecraft:stripped_cherry_log"
            | "minecraft:stripped_pale_oak_log"
            | "minecraft:stripped_crimson_stem"
            | "minecraft:stripped_warped_stem"
    )
}

fn block_is_leaf(block: &str) -> bool {
    matches!(
        block,
        "minecraft:oak_leaves"
            | "minecraft:spruce_leaves"
            | "minecraft:birch_leaves"
            | "minecraft:jungle_leaves"
            | "minecraft:acacia_leaves"
            | "minecraft:dark_oak_leaves"
            | "minecraft:mangrove_leaves"
            | "minecraft:cherry_leaves"
            | "minecraft:pale_oak_leaves"
            | "minecraft:azalea_leaves"
            | "minecraft:flowering_azalea_leaves"
    )
}

fn block_is_small_flower(block: &str) -> bool {
    matches!(
        block,
        "minecraft:dandelion"
            | "minecraft:poppy"
            | "minecraft:blue_orchid"
            | "minecraft:allium"
            | "minecraft:azure_bluet"
            | "minecraft:red_tulip"
            | "minecraft:orange_tulip"
            | "minecraft:white_tulip"
            | "minecraft:pink_tulip"
            | "minecraft:oxeye_daisy"
            | "minecraft:cornflower"
            | "minecraft:lily_of_the_valley"
            | "minecraft:wither_rose"
            | "minecraft:closed_eyeblossom"
            | "minecraft:open_eyeblossom"
    )
}

fn block_is_replaceable_by_tree(block: &str) -> bool {
    block_is_leaf(block)
        || block_is_small_flower(block)
        || matches!(
            block,
            "minecraft:pale_moss_carpet"
                | "minecraft:short_grass"
                | "minecraft:fern"
                | "minecraft:dead_bush"
                | "minecraft:vine"
                | "minecraft:glow_lichen"
                | "minecraft:sunflower"
                | "minecraft:lilac"
                | "minecraft:rose_bush"
                | "minecraft:peony"
                | "minecraft:tall_grass"
                | "minecraft:large_fern"
                | "minecraft:hanging_roots"
                | "minecraft:pitcher_plant"
                | "minecraft:water"
                | "minecraft:seagrass"
                | "minecraft:tall_seagrass"
                | "minecraft:bush"
                | "minecraft:firefly_bush"
                | "minecraft:warped_roots"
                | "minecraft:nether_sprouts"
                | "minecraft:crimson_roots"
                | "minecraft:leaf_litter"
                | "minecraft:short_dry_grass"
                | "minecraft:tall_dry_grass"
        )
}

pub(super) fn block_matches_tag(block: &str, tag: &str) -> bool {
    let tag = tag.strip_prefix("minecraft:").unwrap_or(tag);
    match tag {
        "air" => {
            block == "minecraft:air"
                || block == "minecraft:cave_air"
                || block == "minecraft:void_air"
        }
        "stone_ore_replaceables" => matches!(
            block,
            "minecraft:stone" | "minecraft:granite" | "minecraft:diorite" | "minecraft:andesite"
        ),
        "deepslate_ore_replaceables" => {
            matches!(block, "minecraft:deepslate" | "minecraft:tuff")
        }
        "base_stone_overworld" => {
            block_matches_tag(block, "minecraft:stone_ore_replaceables")
                || block_matches_tag(block, "minecraft:deepslate_ore_replaceables")
        }
        "base_stone_nether" => {
            matches!(
                block,
                "minecraft:netherrack" | "minecraft:basalt" | "minecraft:blackstone"
            )
        }
        "logs" => block_is_log(block),
        "leaves" => block_is_leaf(block),
        "small_flowers" => block_is_small_flower(block),
        "replaceable_by_trees" => block_is_replaceable_by_tree(block),
        "dirt" => matches!(
            block,
            "minecraft:dirt" | "minecraft:coarse_dirt" | "minecraft:rooted_dirt"
        ),
        "grass_blocks" => matches!(
            block,
            "minecraft:grass_block" | "minecraft:podzol" | "minecraft:mycelium"
        ),
        "mud" => matches!(block, "minecraft:mud" | "minecraft:muddy_mangrove_roots"),
        "moss_blocks" => matches!(block, "minecraft:moss_block" | "minecraft:pale_moss_block"),
        "substrate_overworld" => {
            block_matches_tag(block, "minecraft:dirt")
                || block_matches_tag(block, "minecraft:mud")
                || block_matches_tag(block, "minecraft:moss_blocks")
                || block_matches_tag(block, "minecraft:grass_blocks")
        }
        "supports_vegetation" => {
            block_matches_tag(block, "minecraft:substrate_overworld")
                || block == "minecraft:farmland"
        }
        "cannot_replace_below_tree_trunk" => {
            block_matches_tag(block, "minecraft:dirt")
                || block_matches_tag(block, "minecraft:mud")
                || block_matches_tag(block, "minecraft:moss_blocks")
                || block == "minecraft:podzol"
        }
        "replaceable" => {
            block == "minecraft:air"
                || block == "minecraft:cave_air"
                || block == "minecraft:void_air"
        }
        _ => false,
    }
}

pub const BLOCK_PREDICATE_TYPES: &[BlockPredicateType] = &[
    BlockPredicateType {
        id: "minecraft:matching_blocks",
    },
    BlockPredicateType {
        id: "minecraft:matching_block_tag",
    },
    BlockPredicateType {
        id: "minecraft:matching_fluids",
    },
    BlockPredicateType {
        id: "minecraft:has_sturdy_face",
    },
    BlockPredicateType {
        id: "minecraft:solid",
    },
    BlockPredicateType {
        id: "minecraft:replaceable",
    },
    BlockPredicateType {
        id: "minecraft:would_survive",
    },
    BlockPredicateType {
        id: "minecraft:inside_world_bounds",
    },
    BlockPredicateType {
        id: "minecraft:any_of",
    },
    BlockPredicateType {
        id: "minecraft:all_of",
    },
    BlockPredicateType {
        id: "minecraft:not",
    },
    BlockPredicateType {
        id: "minecraft:true",
    },
    BlockPredicateType {
        id: "minecraft:unobstructed",
    },
];
