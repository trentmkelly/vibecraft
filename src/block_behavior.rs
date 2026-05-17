#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::block_metadata::{default_state, representative_state_definition};
use crate::block_update::{
    plan_chunk_block_updates, BlockChange, BlockPos, BlockUpdateAction, Direction, UpdateFlags,
};

pub const SHAPE_UPDATE_ORDER: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::North,
    Direction::South,
    Direction::Down,
    Direction::Up,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalFacing {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateModel {
    pub registry_id: String,
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementContext {
    pub clicked_pos: BlockPos,
    pub clicked_face: Direction,
    pub player_horizontal_facing: HorizontalFacing,
    pub replacing_clicked_block: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapeUpdateContext {
    pub pos: BlockPos,
    pub neighbor_pos: BlockPos,
    pub direction_to_neighbor: Direction,
    pub neighbor_signal: bool,
    pub can_survive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeUpdateResult {
    Keep(BlockStateModel),
    Replace(BlockStateModel),
    Destroy(BlockStateModel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroyPlan {
    pub pos: BlockPos,
    pub old_state: BlockStateModel,
    pub replacement: BlockStateModel,
    pub drops: Vec<String>,
    pub actions: Vec<BlockUpdateAction>,
}

impl HorizontalFacing {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => self,
            Rotation::Clockwise90 => match self {
                Self::North => Self::East,
                Self::East => Self::South,
                Self::South => Self::West,
                Self::West => Self::North,
            },
            Rotation::Clockwise180 => self.opposite(),
            Rotation::CounterClockwise90 => match self {
                Self::North => Self::West,
                Self::West => Self::South,
                Self::South => Self::East,
                Self::East => Self::North,
            },
        }
    }

    pub fn mirror(self, mirror: Mirror) -> Self {
        match mirror {
            Mirror::None => self,
            Mirror::LeftRight => match self {
                Self::North => Self::South,
                Self::South => Self::North,
                other => other,
            },
            Mirror::FrontBack => match self {
                Self::East => Self::West,
                Self::West => Self::East,
                other => other,
            },
        }
    }
}

impl TryFrom<&str> for HorizontalFacing {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "north" => Ok(Self::North),
            "east" => Ok(Self::East),
            "south" => Ok(Self::South),
            "west" => Ok(Self::West),
            _ => Err(format!("not a horizontal facing value: {value}")),
        }
    }
}

impl BlockStateModel {
    pub fn new(registry_id: impl Into<String>) -> Self {
        Self {
            registry_id: registry_id.into(),
            properties: BTreeMap::new(),
        }
    }

    pub fn default_for(registry_id: &str) -> Option<Self> {
        let definition = representative_state_definition(registry_id)?;
        let properties = default_state(&definition)
            .into_iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        Some(Self {
            registry_id: registry_id.to_string(),
            properties,
        })
    }

    pub fn air() -> Self {
        Self::new("minecraft:air")
    }

    pub fn with_property(mut self, name: &str, value: impl Into<String>) -> Self {
        self.properties.insert(name.to_string(), value.into());
        self
    }

    pub fn property(&self, name: &str) -> Option<&str> {
        self.properties.get(name).map(String::as_str)
    }

    pub fn is_air(&self) -> bool {
        self.registry_id == "minecraft:air"
    }

    pub fn has_block_entity(&self) -> bool {
        matches!(
            self.registry_id.as_str(),
            "minecraft:chest"
                | "minecraft:trapped_chest"
                | "minecraft:barrel"
                | "minecraft:furnace"
                | "minecraft:blast_furnace"
                | "minecraft:smoker"
                | "minecraft:beehive"
                | "minecraft:lectern"
                | "minecraft:sign"
                | "minecraft:oak_sign"
                | "minecraft:oak_hanging_sign"
        )
    }
}

pub fn placement_pos(context: PlacementContext) -> BlockPos {
    if context.replacing_clicked_block {
        context.clicked_pos
    } else {
        context.clicked_pos.relative(context.clicked_face)
    }
}

pub fn can_replace(existing: &BlockStateModel, held_block_id: &str) -> bool {
    existing.is_air()
        || existing.registry_id == "minecraft:water"
        || existing.property("replaceable") == Some("true")
        || (existing.registry_id == "minecraft:snow" && held_block_id != "minecraft:snow")
}

pub fn place_facing_opposite_player(
    default_state: &BlockStateModel,
    context: PlacementContext,
) -> BlockStateModel {
    default_state.clone().with_property(
        "facing",
        context.player_horizontal_facing.opposite().as_str(),
    )
}

pub fn rotate_state(state: &BlockStateModel, rotation: Rotation) -> BlockStateModel {
    transform_facing(state, |facing| facing.rotate(rotation))
}

pub fn mirror_state(state: &BlockStateModel, mirror: Mirror) -> BlockStateModel {
    transform_facing(state, |facing| facing.mirror(mirror))
}

pub fn update_shape(
    state: &BlockStateModel,
    _context: ShapeUpdateContext,
    updater: impl Fn(&BlockStateModel) -> BlockStateModel,
) -> ShapeUpdateResult {
    let updated = updater(state);
    if !updated.is_air() {
        ShapeUpdateResult::Replace(updated)
    } else {
        ShapeUpdateResult::Destroy(BlockStateModel::air())
    }
}

pub fn update_shape_or_destroy(
    state: &BlockStateModel,
    context: ShapeUpdateContext,
) -> ShapeUpdateResult {
    if context.can_survive {
        ShapeUpdateResult::Keep(state.clone())
    } else {
        ShapeUpdateResult::Destroy(BlockStateModel::air())
    }
}

pub fn plan_destroy_block(
    pos: BlockPos,
    old_state: BlockStateModel,
    flags: UpdateFlags,
    drop_resources: bool,
) -> DestroyPlan {
    let replacement = BlockStateModel::air();
    let old_block_for_update = if old_state.has_block_entity() {
        "minecraft:chest"
    } else {
        "minecraft:stone"
    };
    let change = BlockChange {
        pos,
        old_block: old_block_for_update,
        new_block: "minecraft:air",
        flags,
    };
    let actions = plan_chunk_block_updates(
        &[change],
        |id| state_has_block_entity(id),
        |id| state_has_block_entity(id),
    );
    let drops = if drop_resources && !old_state.is_air() {
        vec![old_state.registry_id.clone()]
    } else {
        Vec::new()
    };

    DestroyPlan {
        pos,
        old_state,
        replacement,
        drops,
        actions,
    }
}

fn transform_facing(
    state: &BlockStateModel,
    f: impl FnOnce(HorizontalFacing) -> HorizontalFacing,
) -> BlockStateModel {
    let Some(facing) = state
        .property("facing")
        .and_then(|value| HorizontalFacing::try_from(value).ok())
    else {
        return state.clone();
    };

    state.clone().with_property("facing", f(facing).as_str())
}

fn state_has_block_entity(registry_id: &str) -> bool {
    BlockStateModel::new(registry_id).has_block_entity()
}

#[cfg(test)]
mod tests {
    use super::{
        can_replace, mirror_state, place_facing_opposite_player, placement_pos, plan_destroy_block,
        rotate_state, update_shape, update_shape_or_destroy, BlockStateModel, HorizontalFacing,
        Mirror, PlacementContext, Rotation, ShapeUpdateContext, ShapeUpdateResult,
        SHAPE_UPDATE_ORDER,
    };
    use crate::block_update::{
        BlockPos, BlockUpdateAction, Direction, UpdateFlags, UPDATE_ORDER as NEIGHBOR_UPDATE_ORDER,
    };

    #[test]
    fn shape_update_order_matches_vanilla_block_behaviour_order() {
        assert_eq!(
            SHAPE_UPDATE_ORDER,
            [
                Direction::West,
                Direction::East,
                Direction::North,
                Direction::South,
                Direction::Down,
                Direction::Up
            ]
        );
        assert_ne!(SHAPE_UPDATE_ORDER, NEIGHBOR_UPDATE_ORDER);
    }

    #[test]
    fn placement_uses_clicked_face_unless_replacing_clicked_block() {
        let context = PlacementContext {
            clicked_pos: BlockPos { x: 4, y: 64, z: -3 },
            clicked_face: Direction::Up,
            player_horizontal_facing: HorizontalFacing::North,
            replacing_clicked_block: false,
        };
        assert_eq!(placement_pos(context), BlockPos { x: 4, y: 65, z: -3 });

        assert_eq!(
            placement_pos(PlacementContext {
                replacing_clicked_block: true,
                ..context
            }),
            context.clicked_pos
        );
    }

    #[test]
    fn replaceability_covers_air_liquid_snow_and_explicit_replaceable_blocks() {
        assert!(can_replace(&BlockStateModel::air(), "minecraft:stone"));
        assert!(can_replace(
            &BlockStateModel::new("minecraft:water"),
            "minecraft:stone"
        ));
        assert!(can_replace(
            &BlockStateModel::new("minecraft:tall_grass").with_property("replaceable", "true"),
            "minecraft:stone"
        ));
        assert!(can_replace(
            &BlockStateModel::new("minecraft:snow"),
            "minecraft:dirt"
        ));
        assert!(!can_replace(
            &BlockStateModel::new("minecraft:stone"),
            "minecraft:dirt"
        ));
    }

    #[test]
    fn horizontal_placement_faces_opposite_player_direction() {
        let chest = BlockStateModel::default_for("minecraft:chest").unwrap();
        let placed = place_facing_opposite_player(
            &chest,
            PlacementContext {
                clicked_pos: BlockPos { x: 0, y: 64, z: 0 },
                clicked_face: Direction::Up,
                player_horizontal_facing: HorizontalFacing::East,
                replacing_clicked_block: false,
            },
        );
        assert_eq!(placed.property("facing"), Some("west"));
        assert_eq!(placed.property("type"), Some("single"));
        assert_eq!(placed.property("waterlogged"), Some("false"));
    }

    #[test]
    fn rotation_and_mirroring_transform_facing_and_preserve_other_properties() {
        let stairs = BlockStateModel::default_for("minecraft:oak_stairs")
            .unwrap()
            .with_property("facing", "north")
            .with_property("waterlogged", "true");

        let rotated = rotate_state(&stairs, Rotation::Clockwise90);
        assert_eq!(rotated.property("facing"), Some("east"));
        assert_eq!(rotated.property("waterlogged"), Some("true"));

        let mirrored = mirror_state(&rotated, Mirror::FrontBack);
        assert_eq!(mirrored.property("facing"), Some("west"));
        assert_eq!(mirrored.property("waterlogged"), Some("true"));

        let unchanged = rotate_state(
            &BlockStateModel::new("minecraft:stone"),
            Rotation::Clockwise90,
        );
        assert_eq!(unchanged.registry_id, "minecraft:stone");
    }

    #[test]
    fn shape_update_keeps_surviving_state_and_destroys_unsupported_state() {
        let torch = BlockStateModel::default_for("minecraft:torch").unwrap();
        let context = ShapeUpdateContext {
            pos: BlockPos { x: 0, y: 65, z: 0 },
            neighbor_pos: BlockPos { x: 0, y: 64, z: 0 },
            direction_to_neighbor: Direction::Down,
            neighbor_signal: false,
            can_survive: true,
        };
        assert_eq!(
            update_shape_or_destroy(&torch, context),
            ShapeUpdateResult::Keep(torch.clone())
        );
        assert_eq!(
            update_shape_or_destroy(
                &torch,
                ShapeUpdateContext {
                    can_survive: false,
                    ..context
                }
            ),
            ShapeUpdateResult::Destroy(BlockStateModel::air())
        );
    }

    #[test]
    fn custom_shape_update_replaces_or_destroys_state() {
        let stone = BlockStateModel::new("minecraft:stone");
        let context = ShapeUpdateContext {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            neighbor_pos: BlockPos { x: 1, y: 65, z: 1 },
            direction_to_neighbor: Direction::Up,
            neighbor_signal: false,
            can_survive: true,
        };
        assert_eq!(
            update_shape(&stone, context, |state| state
                .clone()
                .with_property("updated", "true")),
            ShapeUpdateResult::Replace(stone.clone().with_property("updated", "true"))
        );
        assert_eq!(
            update_shape(&stone, context, |_| BlockStateModel::air()),
            ShapeUpdateResult::Destroy(BlockStateModel::air())
        );
    }

    #[test]
    fn destruction_plan_removes_block_entities_drops_and_notifies_neighbors() {
        let pos = BlockPos { x: 8, y: 70, z: -2 };
        let plan = plan_destroy_block(
            pos,
            BlockStateModel::default_for("minecraft:chest").unwrap(),
            UpdateFlags::NOTIFY_NEIGHBORS,
            true,
        );

        assert_eq!(plan.replacement, BlockStateModel::air());
        assert_eq!(plan.drops, vec!["minecraft:chest"]);
        assert!(plan
            .actions
            .contains(&BlockUpdateAction::RemoveBlockEntity(pos)));
        assert!(plan.actions.contains(&BlockUpdateAction::MarkUnsaved(pos)));
        assert!(plan.actions.iter().any(|action| matches!(
            action,
            BlockUpdateAction::AffectNeighborsAfterRemoval { pos: affected, .. } if *affected == pos
        )));
        assert_eq!(
            plan.actions
                .iter()
                .filter(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. }))
                .count(),
            6
        );
    }
}
