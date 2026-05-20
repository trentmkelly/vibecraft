#![allow(dead_code)]

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    West,
    East,
    Down,
    Up,
    North,
    South,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateFlags(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockChange {
    pub pos: BlockPos,
    pub old_block: &'static str,
    pub new_block: &'static str,
    pub flags: UpdateFlags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockUpdateAction {
    QueueLightCheck(BlockPos),
    RemoveBlockEntity(BlockPos),
    AffectNeighborsAfterRemoval {
        pos: BlockPos,
        moved_by_piston: bool,
    },
    OnPlace {
        pos: BlockPos,
        moved_by_piston: bool,
    },
    NotifyNeighbor {
        pos: BlockPos,
        source: BlockPos,
        direction: Direction,
    },
    MarkUnsaved(BlockPos),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeighborUpdateQueue {
    max_chained_updates: i32,
    running: bool,
    count: i32,
    stack: VecDeque<BlockUpdateAction>,
    added_this_layer: Vec<BlockUpdateAction>,
    skipped_first_pos: Option<BlockPos>,
    executed: Vec<BlockUpdateAction>,
}

pub const UPDATE_ORDER: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
];

impl UpdateFlags {
    pub const NOTIFY_NEIGHBORS: Self = Self(1);
    pub const MOVED_BY_PISTON: Self = Self(64);
    pub const SKIP_REDSTONE_WIRE_STATE_REPLACEMENT: Self = Self(128);
    pub const SUPPRESS_SIDE_EFFECTS: Self = Self(256);
    pub const SUPPRESS_ON_PLACE: Self = Self(512);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BlockPos {
    pub const fn relative(self, direction: Direction) -> Self {
        match direction {
            Direction::West => Self {
                x: self.x - 1,
                y: self.y,
                z: self.z,
            },
            Direction::East => Self {
                x: self.x + 1,
                y: self.y,
                z: self.z,
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y - 1,
                z: self.z,
            },
            Direction::Up => Self {
                x: self.x,
                y: self.y + 1,
                z: self.z,
            },
            Direction::North => Self {
                x: self.x,
                y: self.y,
                z: self.z - 1,
            },
            Direction::South => Self {
                x: self.x,
                y: self.y,
                z: self.z + 1,
            },
        }
    }
}

impl Direction {
    pub const fn opposite(self) -> Self {
        match self {
            Self::West => Self::East,
            Self::East => Self::West,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
        }
    }
}

impl BlockChange {
    pub fn changed(&self) -> bool {
        self.old_block != self.new_block
    }

    pub fn moved_by_piston(&self) -> bool {
        self.flags.contains(UpdateFlags::MOVED_BY_PISTON)
    }

    pub fn side_effects_enabled(&self) -> bool {
        !self.flags.contains(UpdateFlags::SUPPRESS_SIDE_EFFECTS)
    }
}

impl NeighborUpdateQueue {
    pub fn new(max_chained_updates: i32) -> Self {
        Self {
            max_chained_updates,
            running: false,
            count: 0,
            stack: VecDeque::new(),
            added_this_layer: Vec::new(),
            skipped_first_pos: None,
            executed: Vec::new(),
        }
    }

    pub fn add_and_run(&mut self, pos: BlockPos, update: BlockUpdateAction) {
        let running_already = self.running;
        let too_many_updates =
            self.max_chained_updates >= 0 && self.count >= self.max_chained_updates;
        self.count += 1;

        if !too_many_updates {
            if running_already {
                self.added_this_layer.push(update);
            } else {
                self.stack.push_front(update);
            }
        } else if self.skipped_first_pos.is_none() {
            self.skipped_first_pos = Some(pos);
        }

        if !running_already {
            self.run_updates();
        }
    }

    pub fn update_neighbors_at_except_from_facing(
        &mut self,
        source: BlockPos,
        skip_direction: Option<Direction>,
    ) {
        let running_already = self.running;
        let too_many_updates =
            self.max_chained_updates >= 0 && self.count >= self.max_chained_updates;
        self.count += 1;
        if too_many_updates {
            self.skipped_first_pos.get_or_insert(source);
            if !running_already {
                self.count = 0;
            }
            return;
        }

        for direction in UPDATE_ORDER {
            if Some(direction) != skip_direction {
                let pos = source.relative(direction);
                let update = BlockUpdateAction::NotifyNeighbor {
                    pos,
                    source,
                    direction,
                };
                if running_already {
                    self.added_this_layer.push(update);
                } else {
                    self.stack.push_back(update);
                }
            }
        }
        if !running_already {
            self.run_updates();
        }
    }

    pub fn update_shape_cascade(&mut self, source: BlockPos) {
        self.update_neighbors_at_except_from_facing(source, None);
        for direction in UPDATE_ORDER {
            let neighbor = source.relative(direction);
            self.update_neighbors_at_except_from_facing(neighbor, Some(direction.opposite()));
        }
    }

    pub fn executed(&self) -> &[BlockUpdateAction] {
        &self.executed
    }

    pub fn skipped_first_pos(&self) -> Option<BlockPos> {
        self.skipped_first_pos
    }

    fn run_updates(&mut self) {
        self.running = true;
        while !self.stack.is_empty() || !self.added_this_layer.is_empty() {
            for update in self.added_this_layer.drain(..).rev() {
                self.stack.push_front(update);
            }

            if let Some(update) = self.stack.pop_front() {
                self.executed.push(update);
            }
        }
        self.running = false;
        self.count = 0;
    }
}

pub fn plan_chunk_block_updates(
    changes: &[BlockChange],
    old_had_block_entity: impl Fn(&str) -> bool,
    new_has_block_entity: impl Fn(&str) -> bool,
) -> Vec<BlockUpdateAction> {
    plan_chunk_block_updates_with_limit(changes, -1, old_had_block_entity, new_has_block_entity)
}

pub fn plan_chunk_block_updates_with_limit(
    changes: &[BlockChange],
    max_chained_updates: i32,
    old_had_block_entity: impl Fn(&str) -> bool,
    new_has_block_entity: impl Fn(&str) -> bool,
) -> Vec<BlockUpdateAction> {
    let mut actions = Vec::new();
    let mut neighbors = NeighborUpdateQueue::new(max_chained_updates);

    for change in changes {
        actions.push(BlockUpdateAction::QueueLightCheck(change.pos));

        if change.changed()
            && old_had_block_entity(change.old_block)
            && !new_has_block_entity(change.new_block)
            && change.side_effects_enabled()
        {
            actions.push(BlockUpdateAction::RemoveBlockEntity(change.pos));
        }

        if change.changed()
            && (change.flags.contains(UpdateFlags::NOTIFY_NEIGHBORS) || change.moved_by_piston())
        {
            actions.push(BlockUpdateAction::AffectNeighborsAfterRemoval {
                pos: change.pos,
                moved_by_piston: change.moved_by_piston(),
            });
            neighbors.update_shape_cascade(change.pos);
        }

        if !change.flags.contains(UpdateFlags::SUPPRESS_ON_PLACE) {
            actions.push(BlockUpdateAction::OnPlace {
                pos: change.pos,
                moved_by_piston: change.moved_by_piston(),
            });
        }

        actions.push(BlockUpdateAction::MarkUnsaved(change.pos));
    }

    actions.extend(neighbors.executed().iter().cloned());
    actions
}

#[cfg(test)]
mod tests {
    use super::{
        plan_chunk_block_updates, plan_chunk_block_updates_with_limit, BlockChange, BlockPos,
        BlockUpdateAction, Direction, NeighborUpdateQueue, UpdateFlags, UPDATE_ORDER,
    };

    #[test]
    fn neighbor_update_order_matches_vanilla_order() {
        assert_eq!(
            UPDATE_ORDER,
            [
                Direction::West,
                Direction::East,
                Direction::Down,
                Direction::Up,
                Direction::North,
                Direction::South
            ]
        );

        let source = BlockPos {
            x: 10,
            y: 64,
            z: -3,
        };
        let mut queue = NeighborUpdateQueue::new(-1);
        queue.update_neighbors_at_except_from_facing(source, Some(Direction::Up));

        let positions = queue
            .executed()
            .iter()
            .map(|action| match action {
                BlockUpdateAction::NotifyNeighbor { pos, direction, .. } => (*pos, *direction),
                other => panic!("unexpected action: {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            positions,
            vec![
                (source.relative(Direction::West), Direction::West),
                (source.relative(Direction::East), Direction::East),
                (source.relative(Direction::Down), Direction::Down),
                (source.relative(Direction::North), Direction::North),
                (source.relative(Direction::South), Direction::South),
            ]
        );
    }

    #[test]
    fn chunk_block_change_queues_light_on_place_neighbors_and_unsaved() {
        let change = BlockChange {
            pos: BlockPos { x: 1, y: 2, z: 3 },
            old_block: "minecraft:air",
            new_block: "minecraft:stone",
            flags: UpdateFlags::NOTIFY_NEIGHBORS,
        };

        let actions = plan_chunk_block_updates(&[change], |_| false, |_| false);
        assert_eq!(actions[0], BlockUpdateAction::QueueLightCheck(change.pos));
        assert!(
            actions.contains(&BlockUpdateAction::AffectNeighborsAfterRemoval {
                pos: change.pos,
                moved_by_piston: false
            })
        );
        assert!(actions.contains(&BlockUpdateAction::OnPlace {
            pos: change.pos,
            moved_by_piston: false
        }));
        assert!(actions.contains(&BlockUpdateAction::MarkUnsaved(change.pos)));
        assert_eq!(
            actions
                .iter()
                .filter(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. }))
                .count(),
            36
        );
    }

    #[test]
    fn side_effect_and_on_place_flags_match_level_chunk_rules() {
        let flags = UpdateFlags::SUPPRESS_SIDE_EFFECTS
            .union(UpdateFlags::SUPPRESS_ON_PLACE)
            .union(UpdateFlags::MOVED_BY_PISTON);
        let change = BlockChange {
            pos: BlockPos { x: 0, y: 70, z: 0 },
            old_block: "minecraft:chest",
            new_block: "minecraft:air",
            flags,
        };

        let actions = plan_chunk_block_updates(
            &[change],
            |block| block == "minecraft:chest",
            |block| block == "minecraft:chest",
        );
        assert!(!actions
            .iter()
            .any(|action| matches!(action, BlockUpdateAction::RemoveBlockEntity(_))));
        assert!(!actions
            .iter()
            .any(|action| matches!(action, BlockUpdateAction::OnPlace { .. })));
        assert!(
            actions.contains(&BlockUpdateAction::AffectNeighborsAfterRemoval {
                pos: change.pos,
                moved_by_piston: true
            })
        );
    }

    #[test]
    fn block_entity_removal_respects_replacement_block_entity_type() {
        let pos = BlockPos { x: -2, y: 64, z: 5 };
        let removed = BlockChange {
            pos,
            old_block: "minecraft:chest",
            new_block: "minecraft:stone",
            flags: UpdateFlags::empty(),
        };
        let replaced = BlockChange {
            new_block: "minecraft:barrel",
            ..removed
        };

        let removal_actions = plan_chunk_block_updates(
            &[removed],
            |block| block == "minecraft:chest",
            |block| block == "minecraft:barrel",
        );
        assert!(removal_actions.contains(&BlockUpdateAction::RemoveBlockEntity(pos)));

        let replacement_actions = plan_chunk_block_updates(
            &[replaced],
            |block| block == "minecraft:chest",
            |block| block == "minecraft:barrel",
        );
        assert!(!replacement_actions
            .iter()
            .any(|action| matches!(action, BlockUpdateAction::RemoveBlockEntity(_))));
    }

    #[test]
    fn collecting_neighbor_updater_enforces_chain_limit() {
        let source = BlockPos { x: 0, y: 0, z: 0 };
        let mut queue = NeighborUpdateQueue::new(0);
        queue.update_neighbors_at_except_from_facing(source, None);

        assert!(queue.executed().is_empty());
        assert_eq!(queue.skipped_first_pos(), Some(source));
    }

    #[test]
    fn chunk_block_update_planner_uses_configured_neighbor_chain_limit() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let change = BlockChange {
            pos,
            old_block: "minecraft:stone",
            new_block: "minecraft:air",
            flags: UpdateFlags::NOTIFY_NEIGHBORS,
        };

        let limited = plan_chunk_block_updates_with_limit(&[change], 0, |_| false, |_| false);
        assert!(!limited
            .iter()
            .any(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. })));

        let unlimited = plan_chunk_block_updates_with_limit(&[change], -1, |_| false, |_| false);
        assert_eq!(
            unlimited
                .iter()
                .filter(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. }))
                .count(),
            36
        );
    }

    #[test]
    fn neighbor_cascade_notifies_adjacent_blocks_and_their_shape_neighbors() {
        use super::{
            plan_chunk_block_updates, BlockChange, BlockPos, BlockUpdateAction, Direction,
            UpdateFlags,
        };

        let pos = BlockPos { x: 5, y: 64, z: 5 };
        let changes = [BlockChange {
            pos,
            old_block: "minecraft:stone",
            new_block: "minecraft:air",
            flags: UpdateFlags::NOTIFY_NEIGHBORS,
        }];

        let actions = plan_chunk_block_updates(&changes, |_| false, |_| false);

        let neighbor_notifications: Vec<_> = actions
            .iter()
            .filter_map(|a| match a {
                BlockUpdateAction::NotifyNeighbor {
                    pos: neighbor_pos,
                    source,
                    direction,
                } => Some((*neighbor_pos, *source, *direction)),
                _ => None,
            })
            .collect();

        assert_eq!(
            neighbor_notifications.len(),
            36,
            "should notify all 6 adjacent blocks plus 5 shape neighbors around each"
        );

        let directions = [
            Direction::West,
            Direction::East,
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
        ];
        for dir in directions {
            let expected_pos = pos.relative(dir);
            assert!(
                neighbor_notifications
                    .iter()
                    .any(|(np, src, d)| { *np == expected_pos && *src == pos && *d == dir }),
                "missing notification for direction {:?}",
                dir
            );
        }

        for dir in directions {
            let neighbor = pos.relative(dir);
            let chained = neighbor_notifications
                .iter()
                .filter(|(_, src, _)| *src == neighbor)
                .count();
            assert_eq!(
                chained, 5,
                "neighbor {dir:?} should notify its adjacent shape-update chain except source"
            );
        }
    }
}
