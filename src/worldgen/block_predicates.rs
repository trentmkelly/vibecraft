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

/// Java `BlockState.is(TagKey<Block>)`: membership is resolved against the vendored vanilla block
/// tags (with nested `#tag` references flattened), never hand-maintained lists. Property suffixes
/// (`[facing=north]`) are ignored because tags are keyed by block id.
pub(super) fn block_matches_tag(block: &str, tag: &str) -> bool {
    crate::block_tags::block_tag_contains(tag, block_state_id(block))
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
