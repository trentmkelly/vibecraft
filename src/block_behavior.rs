#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::block_entity::has_block_entity_for_block;
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
        has_block_entity_for_block(self.registry_id.as_str())
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
    let actions =
        plan_chunk_block_updates(&[change], state_has_block_entity, state_has_block_entity);
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

/// 1:1 with the 26.1.2 `InteractionResult` sealed interface. The three `Success`
/// records differ only by swing source: `Success` swings on the client,
/// `SuccessServer` is server-authoritative, and `Consume` does not swing. All
/// three "consume the action"; `Fail`, `Pass`, and `TryWithEmptyHand` do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionResult {
    /// `InteractionResult.SUCCESS` — Success with swing source CLIENT.
    Success,
    /// `InteractionResult.SUCCESS_SERVER` — Success with swing source SERVER.
    SuccessServer,
    /// `InteractionResult.CONSUME` — Success with swing source NONE.
    Consume,
    /// `InteractionResult.PASS` — neither block nor item handled this; try the other.
    Pass,
    /// `InteractionResult.FAIL` — the interaction explicitly failed (e.g. locked chest).
    Fail,
    /// `InteractionResult.TRY_WITH_EMPTY_HAND` — retry as an empty-hand block use.
    TryWithEmptyHand,
}

impl InteractionResult {
    /// Whether this is a `Success` record (`SUCCESS`/`SUCCESS_SERVER`/`CONSUME`).
    pub fn is_success(self) -> bool {
        matches!(self, Self::Success | Self::SuccessServer | Self::Consume)
    }

    /// `Success.swingSource()` is CLIENT or SERVER (not the NONE of `CONSUME`).
    pub fn should_swing_hand(self) -> bool {
        matches!(self, Self::Success | Self::SuccessServer)
    }

    /// `InteractionResult.consumesAction()` — true only for the `Success` records.
    pub fn consumes_action(self) -> bool {
        matches!(self, Self::Success | Self::SuccessServer | Self::Consume)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockUseContext {
    pub pos: crate::block_update::BlockPos,
    pub face: crate::block_update::Direction,
    pub hand: BlockUseHand,
    /// `player.isSecondaryUseActive()` (sneaking).
    pub sneaking: bool,
    pub block_id: &'static str,
    /// The item in the acting hand (`None` = empty hand).
    pub held_item_id: Option<&'static str>,
    /// `!mainHand.isEmpty() || !offHand.isEmpty()` — whether either hand holds
    /// an item, used to compute `suppressUsingBlock`.
    pub has_item_in_hands: bool,
    /// `player.getCooldowns().isOnCooldown(itemStack)`.
    pub item_on_cooldown: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockUseHand {
    MainHand,
    OffHand,
}

/// Dispatch a right-click use action, 1:1 with `ServerPlayerGameMode.useItemOn`:
/// 1. `suppressUsingBlock = isSecondaryUseActive() && haveSomethingInOurHands`.
/// 2. If not suppressed: `block.useItemOn(item)` — if it `consumesAction`, return
///    it; otherwise if it is `TryWithEmptyHand` and this is the main hand, try
///    `block.useWithoutItem()` and return that if it `consumesAction`.
/// 3. If the held item is non-empty and not on cooldown: `item.useOn(context)`.
/// 4. Otherwise `PASS`.
///
/// `block_use_fn` models `blockState.useItemOn(...)`, `use_without_item_fn`
/// models `blockState.useWithoutItem(...)`, and `item_use_fn` models
/// `stack.useOn(context)`.
pub fn dispatch_block_use(
    ctx: &BlockUseContext,
    block_use_fn: impl FnOnce(&BlockUseContext) -> InteractionResult,
    use_without_item_fn: impl FnOnce(&BlockUseContext) -> InteractionResult,
    item_use_fn: impl FnOnce(&BlockUseContext) -> InteractionResult,
) -> InteractionResult {
    let suppress_using_block = ctx.sneaking && ctx.has_item_in_hands;
    if !suppress_using_block {
        let item_use = block_use_fn(ctx);
        if item_use.consumes_action() {
            return item_use;
        }
        if item_use == InteractionResult::TryWithEmptyHand && ctx.hand == BlockUseHand::MainHand {
            let without_item = use_without_item_fn(ctx);
            if without_item.consumes_action() {
                return without_item;
            }
        }
    }

    if ctx.held_item_id.is_some() && !ctx.item_on_cooldown {
        return item_use_fn(ctx);
    }

    InteractionResult::Pass
}

/// Dispatch a left-click attack action on a block.
///
/// In vanilla this triggers `blockState.attack(level, pos, player)` — used for
/// decorative/interactive on-attack effects (e.g. note block pitch display on attack).
/// Returns whether the block handled the attack (true = block consumed it).
pub fn dispatch_block_attack(_pos: crate::block_update::BlockPos, block_id: &str) -> bool {
    // In vanilla, only a small set of blocks have non-empty attack() implementations.
    // Note blocks trigger a sound on attack; no other common blocks do.
    matches!(block_id, "minecraft:note_block")
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacementValidationContext {
    pub target_pos: crate::block_update::BlockPos,
    pub player_pos: (f64, f64, f64),
    pub player_eye_pos: (f64, f64, f64),
    pub game_mode: PlacementGameMode,
    /// Whether the target block can survive at its position (block-specific check)
    pub can_survive: bool,
    /// Whether any entity occupies the placement position
    pub entity_collision: bool,
    /// Whether spawn protection covers the target position
    pub spawn_protected: bool,
    /// The player's `block_interaction_range` attribute (4.5 survival, 5.0
    /// creative). `isWithinBlockInteractionRange` adds a 1.0 buffer on top.
    pub block_interaction_range: f64,
    /// Inclusive build-height limits (`level.getMaxY()` / `getMinY()`).
    pub max_y: i32,
    pub min_y: i32,
    /// Existing block at target position (must be replaceable)
    pub existing_block_id: &'static str,
    /// Block being placed
    pub placed_block_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementGameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementDenyReason {
    TooFar,
    TooHigh,
    TooLow,
    SpawnProtected,
    CannotSurvive,
    EntityCollision,
    NotReplaceable,
    SpectatorMode,
}

/// Squared distance from `point` to the closest point of the 1×1×1 block AABB
/// at `pos` — `new AABB(pos).distanceToSqr(point)` in vanilla.
fn block_aabb_distance_sq(pos: crate::block_update::BlockPos, point: (f64, f64, f64)) -> f64 {
    let dx = ((pos.x as f64) - point.0).max(point.0 - (pos.x as f64 + 1.0)).max(0.0);
    let dy = ((pos.y as f64) - point.1).max(point.1 - (pos.y as f64 + 1.0)).max(0.0);
    let dz = ((pos.z as f64) - point.2).max(point.2 - (pos.z as f64 + 1.0)).max(0.0);
    dx * dx + dy * dy + dz * dz
}

/// Validate a block placement attempt server-side, consolidating the checks
/// vanilla spreads across `ServerGamePacketListenerImpl.handleUseItemOn`,
/// `ServerPlayerGameMode.useItemOn`, and `BlockItem.place`:
/// 1. Spectators do not place blocks.
/// 2. `isWithinBlockInteractionRange(pos, 1.0)`: squared eye→block-AABB distance
///    against `(block_interaction_range + 1.0)²`.
/// 3. Build-height limits (`pos.y > maxY` / `pos.y < minY`).
/// 4. Spawn protection.
/// 5. `place()` order — replaceable (`canPlace`), then `canSurvive`, then entity
///    collision (`isUnobstructed`).
pub fn validate_placement(ctx: &PlacementValidationContext) -> Result<(), PlacementDenyReason> {
    if ctx.game_mode == PlacementGameMode::Spectator {
        return Err(PlacementDenyReason::SpectatorMode);
    }

    let max_range = ctx.block_interaction_range + 1.0;
    if block_aabb_distance_sq(ctx.target_pos, ctx.player_eye_pos) >= max_range * max_range {
        return Err(PlacementDenyReason::TooFar);
    }

    if ctx.target_pos.y > ctx.max_y {
        return Err(PlacementDenyReason::TooHigh);
    }
    if ctx.target_pos.y < ctx.min_y {
        return Err(PlacementDenyReason::TooLow);
    }

    if ctx.spawn_protected {
        return Err(PlacementDenyReason::SpawnProtected);
    }

    // BlockItem.place: canPlace (replaceable) first, then getPlacementState /
    // canPlace(context, state) which is canSurvive && isUnobstructed.
    let state = BlockStateModel::new(ctx.existing_block_id);
    if !can_replace(&state, ctx.placed_block_id) {
        return Err(PlacementDenyReason::NotReplaceable);
    }
    if !ctx.can_survive {
        return Err(PlacementDenyReason::CannotSurvive);
    }
    if ctx.entity_collision {
        return Err(PlacementDenyReason::EntityCollision);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        can_replace, mirror_state, place_facing_opposite_player, placement_pos, plan_destroy_block,
        rotate_state, update_shape, update_shape_or_destroy, BlockStateModel, HorizontalFacing,
        InteractionResult, Mirror, PlacementContext, Rotation, ShapeUpdateContext,
        ShapeUpdateResult, SHAPE_UPDATE_ORDER,
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
        // 6 direct neighbors + 30 from shape update cascade (6 neighbors × 5 directions each)
        // mirrors Java Level.setBlock() calling updateNeighborsAt + updateNeighbourShapes
        assert_eq!(
            plan.actions
                .iter()
                .filter(|action| matches!(action, BlockUpdateAction::NotifyNeighbor { .. }))
                .count(),
            36
        );
    }

    #[test]
    fn interaction_result_predicates_match_vanilla_groupings() {
        // The three Success records are all "successes" and all consume the action.
        for success in [
            InteractionResult::Success,
            InteractionResult::SuccessServer,
            InteractionResult::Consume,
        ] {
            assert!(success.is_success(), "{success:?}");
            assert!(success.consumes_action(), "{success:?}");
        }
        // Fail, Pass, and TryWithEmptyHand are not successes and do not consume.
        for other in [
            InteractionResult::Pass,
            InteractionResult::Fail,
            InteractionResult::TryWithEmptyHand,
        ] {
            assert!(!other.is_success(), "{other:?}");
            assert!(!other.consumes_action(), "{other:?}");
        }
        // Swing happens for CLIENT/SERVER swing sources, not CONSUME (NONE).
        assert!(InteractionResult::Success.should_swing_hand());
        assert!(InteractionResult::SuccessServer.should_swing_hand());
        assert!(!InteractionResult::Consume.should_swing_hand());
    }

    #[test]
    fn block_use_dispatch_routes_block_first_then_item_respecting_pass() {
        use crate::block_behavior::{
            dispatch_block_use, BlockUseContext, BlockUseHand, InteractionResult,
        };
        use crate::block_update::{BlockPos, Direction};

        let ctx = BlockUseContext {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            face: Direction::Up,
            hand: BlockUseHand::MainHand,
            sneaking: false,
            block_id: "minecraft:crafting_table",
            held_item_id: Some("minecraft:stick"),
            has_item_in_hands: true,
            item_on_cooldown: false,
        };
        let no_without_item = |_: &BlockUseContext| InteractionResult::Pass;

        // Block consumes the action → item handler not called.
        let result = dispatch_block_use(
            &ctx,
            |_| InteractionResult::Success,
            no_without_item,
            |_| panic!("item handler should not be called"),
        );
        assert_eq!(result, InteractionResult::Success);

        // Block returns PASS → item handler called.
        let result = dispatch_block_use(
            &ctx,
            |_| InteractionResult::Pass,
            no_without_item,
            |_| InteractionResult::Consume,
        );
        assert_eq!(result, InteractionResult::Consume);

        // Block returns FAIL (does NOT consume the action) → vanilla still falls
        // through to the item use, unlike the old "return on any non-Pass" model.
        let result = dispatch_block_use(
            &ctx,
            |_| InteractionResult::Fail,
            no_without_item,
            |_| InteractionResult::Success,
        );
        assert_eq!(result, InteractionResult::Success);

        // Block returns TryWithEmptyHand on the main hand → useWithoutItem runs;
        // if it consumes, that result is returned and the item is not used.
        let result = dispatch_block_use(
            &ctx,
            |_| InteractionResult::TryWithEmptyHand,
            |_| InteractionResult::SuccessServer,
            |_| panic!("item handler should not be called when useWithoutItem consumes"),
        );
        assert_eq!(result, InteractionResult::SuccessServer);

        // TryWithEmptyHand but useWithoutItem passes → fall through to item use.
        let result = dispatch_block_use(
            &ctx,
            |_| InteractionResult::TryWithEmptyHand,
            |_| InteractionResult::Pass,
            |_| InteractionResult::Consume,
        );
        assert_eq!(result, InteractionResult::Consume);

        // Sneaking with an item → suppressUsingBlock, block handler skipped.
        let sneak_ctx = BlockUseContext {
            sneaking: true,
            ..ctx.clone()
        };
        let result = dispatch_block_use(
            &sneak_ctx,
            |_| panic!("block handler should not be called when suppressed"),
            no_without_item,
            |_| InteractionResult::Success,
        );
        assert_eq!(result, InteractionResult::Success);

        // Sneaking with empty hands → NOT suppressed, block handler still runs.
        let sneak_empty = BlockUseContext {
            sneaking: true,
            held_item_id: None,
            has_item_in_hands: false,
            ..ctx.clone()
        };
        let result = dispatch_block_use(
            &sneak_empty,
            |_| InteractionResult::Success,
            no_without_item,
            |_| panic!("no item to use"),
        );
        assert_eq!(result, InteractionResult::Success);

        // Held item on cooldown → item use skipped after a block PASS → PASS.
        let cooldown_ctx = BlockUseContext {
            item_on_cooldown: true,
            ..ctx.clone()
        };
        let result = dispatch_block_use(
            &cooldown_ctx,
            |_| InteractionResult::Pass,
            no_without_item,
            |_| panic!("item on cooldown must not be used"),
        );
        assert_eq!(result, InteractionResult::Pass);

        // No held item, block passes → PASS.
        let empty_ctx = BlockUseContext {
            held_item_id: None,
            has_item_in_hands: false,
            sneaking: false,
            ..ctx.clone()
        };
        let result = dispatch_block_use(
            &empty_ctx,
            |_| InteractionResult::Pass,
            no_without_item,
            |_| unreachable!(),
        );
        assert_eq!(result, InteractionResult::Pass);
    }

    #[test]
    fn placement_validation_enforces_all_deny_conditions() {
        use crate::block_behavior::{
            validate_placement, PlacementDenyReason, PlacementGameMode, PlacementValidationContext,
        };
        use crate::block_update::BlockPos;

        let ok_ctx = PlacementValidationContext {
            target_pos: BlockPos { x: 0, y: 64, z: 0 },
            player_pos: (0.5, 63.0, 0.5),
            player_eye_pos: (0.5, 64.62, 0.5),
            game_mode: PlacementGameMode::Survival,
            can_survive: true,
            entity_collision: false,
            spawn_protected: false,
            block_interaction_range: 4.5,
            max_y: 319,
            min_y: -64,
            existing_block_id: "minecraft:air",
            placed_block_id: "minecraft:stone",
        };
        assert!(validate_placement(&ok_ctx).is_ok());

        assert_eq!(
            validate_placement(&PlacementValidationContext {
                game_mode: PlacementGameMode::Spectator,
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::SpectatorMode)
        );
        // Survival reach is range 4.5 + 1.0 buffer = 5.5 to the block AABB. A
        // block one above (AABB face at y=65, eye at 64.62) is 0.38 away → in range.
        assert!(validate_placement(&PlacementValidationContext {
            target_pos: BlockPos { x: 0, y: 65, z: 0 },
            ..ok_ctx.clone()
        })
        .is_ok());
        // 6 blocks east: closest AABB face at x=6, eye at x=0.5 → 5.5 away,
        // which is NOT < 5.5 → too far.
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                target_pos: BlockPos { x: 6, y: 64, z: 0 },
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::TooFar)
        );
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                player_eye_pos: (100.0, 100.0, 100.0),
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::TooFar)
        );
        // Above/below the build limits.
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                target_pos: BlockPos { x: 0, y: 320, z: 0 },
                player_eye_pos: (0.5, 320.5, 0.5),
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::TooHigh)
        );
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                target_pos: BlockPos { x: 0, y: -65, z: 0 },
                player_eye_pos: (0.5, -64.5, 0.5),
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::TooLow)
        );
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                spawn_protected: true,
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::SpawnProtected)
        );
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                can_survive: false,
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::CannotSurvive)
        );
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                entity_collision: true,
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::EntityCollision)
        );
        assert_eq!(
            validate_placement(&PlacementValidationContext {
                existing_block_id: "minecraft:stone",
                ..ok_ctx.clone()
            }),
            Err(PlacementDenyReason::NotReplaceable)
        );
    }

    #[test]
    fn block_attack_dispatch_only_returns_true_for_note_block() {
        use crate::block_behavior::dispatch_block_attack;
        use crate::block_update::BlockPos;

        assert!(dispatch_block_attack(
            BlockPos { x: 0, y: 64, z: 0 },
            "minecraft:note_block"
        ));
        assert!(!dispatch_block_attack(
            BlockPos { x: 0, y: 64, z: 0 },
            "minecraft:stone"
        ));
        assert!(!dispatch_block_attack(
            BlockPos { x: 0, y: 64, z: 0 },
            "minecraft:crafting_table"
        ));
    }

    #[test]
    fn block_entity_lookup_is_authoritative_for_block_states() {
        assert!(BlockStateModel::new("minecraft:chest").has_block_entity());
        assert!(BlockStateModel::new("minecraft:oak_sign").has_block_entity());
        assert!(BlockStateModel::new("minecraft:chiseled_bookshelf").has_block_entity());
        assert!(BlockStateModel::new("minecraft:campfire").has_block_entity());
        assert!(!BlockStateModel::new("minecraft:stone").has_block_entity());
        assert!(!BlockStateModel::new("minecraft:dirt").has_block_entity());
    }
}
