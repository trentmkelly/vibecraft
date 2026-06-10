#![allow(dead_code)]

use crate::block_behavior::BlockStateModel;
use crate::block_update::{BlockPos, BlockUpdateAction, Direction};

pub const FALLING_BLOCK_TICK_DELAY: i32 = 2;
pub const MAX_FALLING_BLOCK_PUSH_DEPTH: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallingKind {
    SandLike,
    ConcretePowder,
    Anvil,
    DragonEgg,
    PointedDripstone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GravityAction {
    Stay,
    ScheduleFall { delay: i32 },
    SpawnFallingEntity(FallingBlockEntityModel),
    Land { state: BlockStateModel },
    Break { drops: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallingBlockEntityModel {
    pub block: BlockStateModel,
    pub pos: BlockPos,
    pub kind: FallingKind,
    pub drop_item: bool,
    pub cancel_drop: bool,
    pub hurt_entities: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GravityPlan {
    pub action: GravityAction,
    pub updates: Vec<BlockUpdateAction>,
}

pub fn falling_kind(state: &BlockStateModel) -> Option<FallingKind> {
    match state.registry_id.as_str() {
        // BrushableBlock (suspicious sand/gravel) falls exactly like
        // FallingBlock (its tick calls FallingBlockEntity.fall).
        "minecraft:sand"
        | "minecraft:red_sand"
        | "minecraft:gravel"
        | "minecraft:suspicious_sand"
        | "minecraft:suspicious_gravel" => Some(FallingKind::SandLike),
        id if id.ends_with("_concrete_powder") => Some(FallingKind::ConcretePowder),
        "minecraft:anvil" | "minecraft:chipped_anvil" | "minecraft:damaged_anvil" => {
            Some(FallingKind::Anvil)
        }
        "minecraft:dragon_egg" => Some(FallingKind::DragonEgg),
        "minecraft:pointed_dripstone" => Some(FallingKind::PointedDripstone),
        _ => None,
    }
}

pub fn is_free_for_falling(state: &BlockStateModel) -> bool {
    // Java FallingBlock.isFree: isAir || is(BlockTags.FIRE) || liquid ||
    // canBeReplaced. `replaceable` is a per-state flag (not a state
    // property), so consult the authoritative physics tables.
    if state.is_air() {
        return true;
    }
    if crate::block_tags::block_tag_contains("fire", &state.registry_id) {
        return true;
    }
    crate::block_properties::state_physics_by_name(&state.state_name())
        .is_some_and(|physics| physics.liquid || physics.replaceable)
}

pub fn schedule_gravity_tick(
    state: &BlockStateModel,
    below: &BlockStateModel,
    pos: BlockPos,
) -> GravityPlan {
    if falling_kind(state).is_some() && is_free_for_falling(below) {
        GravityPlan {
            action: GravityAction::ScheduleFall {
                delay: FALLING_BLOCK_TICK_DELAY,
            },
            updates: vec![BlockUpdateAction::NotifyNeighbor {
                pos: pos.relative(Direction::Down),
                source: pos,
                direction: Direction::Down,
            }],
        }
    } else {
        GravityPlan {
            action: GravityAction::Stay,
            updates: Vec::new(),
        }
    }
}

pub fn start_falling(state: BlockStateModel, pos: BlockPos, min_y: i32) -> GravityPlan {
    let Some(kind) = falling_kind(&state) else {
        return GravityPlan {
            action: GravityAction::Stay,
            updates: Vec::new(),
        };
    };

    if pos.y < min_y {
        return GravityPlan {
            action: GravityAction::Break {
                drops: if kind == FallingKind::DragonEgg {
                    Vec::new()
                } else {
                    vec![state.registry_id]
                },
            },
            updates: vec![BlockUpdateAction::MarkUnsaved(pos)],
        };
    }

    let entity = FallingBlockEntityModel {
        block: state,
        pos,
        kind,
        drop_item: true,
        cancel_drop: false,
        hurt_entities: matches!(kind, FallingKind::Anvil | FallingKind::PointedDripstone),
    };
    GravityPlan {
        action: GravityAction::SpawnFallingEntity(entity),
        updates: vec![
            BlockUpdateAction::QueueLightCheck(pos),
            BlockUpdateAction::MarkUnsaved(pos),
        ],
    }
}

pub fn land_falling_block(
    entity: &FallingBlockEntityModel,
    landing_pos: BlockPos,
    replaced: &BlockStateModel,
    adjacent_to_water: bool,
    fall_distance: f32,
) -> GravityPlan {
    if !is_free_for_falling(replaced) && !replaced.is_air() {
        return break_after_fall(entity, landing_pos);
    }

    let mut landed_state = entity.block.clone();
    if entity.kind == FallingKind::ConcretePowder && adjacent_to_water {
        landed_state.registry_id = landed_state
            .registry_id
            .replace("_concrete_powder", "_concrete");
    }

    if entity.kind == FallingKind::Anvil && fall_distance > 1.0 {
        landed_state.registry_id = match landed_state.registry_id.as_str() {
            "minecraft:anvil" => "minecraft:chipped_anvil".to_string(),
            "minecraft:chipped_anvil" => "minecraft:damaged_anvil".to_string(),
            "minecraft:damaged_anvil" => return break_after_fall(entity, landing_pos),
            _ => landed_state.registry_id,
        };
    }

    GravityPlan {
        action: GravityAction::Land {
            state: landed_state,
        },
        updates: vec![
            BlockUpdateAction::QueueLightCheck(landing_pos),
            BlockUpdateAction::MarkUnsaved(landing_pos),
        ],
    }
}

pub fn break_after_fall(entity: &FallingBlockEntityModel, pos: BlockPos) -> GravityPlan {
    GravityPlan {
        action: GravityAction::Break {
            drops: if entity.drop_item && !entity.cancel_drop {
                vec![entity.block.registry_id.clone()]
            } else {
                Vec::new()
            },
        },
        updates: vec![BlockUpdateAction::MarkUnsaved(pos)],
    }
}

#[cfg(test)]
mod tests {
    use super::{
        break_after_fall, falling_kind, is_free_for_falling, land_falling_block,
        schedule_gravity_tick, start_falling, FallingBlockEntityModel, FallingKind, GravityAction,
        FALLING_BLOCK_TICK_DELAY,
    };
    use crate::block_behavior::BlockStateModel;
    use crate::block_update::{BlockPos, BlockUpdateAction};

    #[test]
    fn identifies_vanilla_gravity_block_families() {
        assert_eq!(
            falling_kind(&BlockStateModel::new("minecraft:sand")),
            Some(FallingKind::SandLike)
        );
        assert_eq!(
            falling_kind(&BlockStateModel::new("minecraft:white_concrete_powder")),
            Some(FallingKind::ConcretePowder)
        );
        assert_eq!(
            falling_kind(&BlockStateModel::new("minecraft:anvil")),
            Some(FallingKind::Anvil)
        );
        assert_eq!(
            falling_kind(&BlockStateModel::new("minecraft:dragon_egg")),
            Some(FallingKind::DragonEgg)
        );
        assert_eq!(
            falling_kind(&BlockStateModel::new("minecraft:pointed_dripstone")),
            Some(FallingKind::PointedDripstone)
        );
        assert_eq!(falling_kind(&BlockStateModel::new("minecraft:stone")), None);
    }

    #[test]
    fn free_blocks_match_falling_block_pass_through_cases() {
        assert!(is_free_for_falling(&BlockStateModel::air()));
        assert!(is_free_for_falling(&BlockStateModel::new(
            "minecraft:water"
        )));
        assert!(is_free_for_falling(
            &BlockStateModel::new("minecraft:tall_grass").with_property("replaceable", "true")
        ));
        assert!(!is_free_for_falling(&BlockStateModel::new(
            "minecraft:stone"
        )));
    }

    #[test]
    fn gravity_tick_is_scheduled_when_support_is_free() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let plan = schedule_gravity_tick(
            &BlockStateModel::new("minecraft:sand"),
            &BlockStateModel::air(),
            pos,
        );
        assert_eq!(
            plan.action,
            GravityAction::ScheduleFall {
                delay: FALLING_BLOCK_TICK_DELAY
            }
        );
        assert_eq!(plan.updates.len(), 1);

        assert_eq!(
            schedule_gravity_tick(
                &BlockStateModel::new("minecraft:sand"),
                &BlockStateModel::new("minecraft:stone"),
                pos,
            )
            .action,
            GravityAction::Stay
        );
    }

    #[test]
    fn falling_tick_spawns_entity_and_removes_world_block() {
        let pos = BlockPos { x: 4, y: 80, z: -1 };
        let plan = start_falling(BlockStateModel::new("minecraft:anvil"), pos, -64);
        let GravityAction::SpawnFallingEntity(entity) = plan.action else {
            panic!("expected falling entity")
        };
        assert_eq!(entity.kind, FallingKind::Anvil);
        assert!(entity.hurt_entities);
        assert!(plan
            .updates
            .contains(&BlockUpdateAction::QueueLightCheck(pos)));
        assert!(plan.updates.contains(&BlockUpdateAction::MarkUnsaved(pos)));
    }

    #[test]
    fn falling_below_world_breaks_without_spawning_entity() {
        let pos = BlockPos { x: 0, y: -65, z: 0 };
        let plan = start_falling(BlockStateModel::new("minecraft:gravel"), pos, -64);
        assert_eq!(
            plan.action,
            GravityAction::Break {
                drops: vec!["minecraft:gravel".to_string()]
            }
        );
    }

    #[test]
    fn concrete_powder_hardens_when_landing_next_to_water() {
        let entity = FallingBlockEntityModel {
            block: BlockStateModel::new("minecraft:blue_concrete_powder"),
            pos: BlockPos { x: 0, y: 70, z: 0 },
            kind: FallingKind::ConcretePowder,
            drop_item: true,
            cancel_drop: false,
            hurt_entities: false,
        };
        let plan = land_falling_block(
            &entity,
            BlockPos { x: 0, y: 64, z: 0 },
            &BlockStateModel::air(),
            true,
            6.0,
        );
        assert_eq!(
            plan.action,
            GravityAction::Land {
                state: BlockStateModel::new("minecraft:blue_concrete")
            }
        );
    }

    #[test]
    fn anvils_degrade_then_break_after_falling() {
        let entity = FallingBlockEntityModel {
            block: BlockStateModel::new("minecraft:anvil"),
            pos: BlockPos { x: 0, y: 70, z: 0 },
            kind: FallingKind::Anvil,
            drop_item: true,
            cancel_drop: false,
            hurt_entities: true,
        };
        assert_eq!(
            land_falling_block(
                &entity,
                BlockPos { x: 0, y: 64, z: 0 },
                &BlockStateModel::air(),
                false,
                3.0,
            )
            .action,
            GravityAction::Land {
                state: BlockStateModel::new("minecraft:chipped_anvil")
            }
        );

        let damaged = FallingBlockEntityModel {
            block: BlockStateModel::new("minecraft:damaged_anvil"),
            ..entity
        };
        assert_eq!(
            land_falling_block(
                &damaged,
                BlockPos { x: 0, y: 64, z: 0 },
                &BlockStateModel::air(),
                false,
                3.0,
            )
            .action,
            GravityAction::Break {
                drops: vec!["minecraft:damaged_anvil".to_string()]
            }
        );
    }

    #[test]
    fn blocked_landing_breaks_with_configured_drop_rules() {
        let entity = FallingBlockEntityModel {
            block: BlockStateModel::new("minecraft:sand"),
            pos: BlockPos { x: 0, y: 70, z: 0 },
            kind: FallingKind::SandLike,
            drop_item: true,
            cancel_drop: false,
            hurt_entities: false,
        };
        assert_eq!(
            land_falling_block(
                &entity,
                BlockPos { x: 0, y: 64, z: 0 },
                &BlockStateModel::new("minecraft:stone"),
                false,
                4.0,
            )
            .action,
            GravityAction::Break {
                drops: vec!["minecraft:sand".to_string()]
            }
        );
        assert_eq!(
            break_after_fall(
                &FallingBlockEntityModel {
                    cancel_drop: true,
                    ..entity
                },
                BlockPos { x: 0, y: 64, z: 0 },
            )
            .action,
            GravityAction::Break { drops: Vec::new() }
        );
    }
}
