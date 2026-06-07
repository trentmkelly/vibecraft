#![allow(dead_code)]

// TODO(block-behaviour-live-side-effects): replace these action plans with live
// ServerLevel/loot/block-entity operations once those subsystems are wired into
// the shared block-state behavior path.

use crate::block_behavior::InteractionResult;
use crate::block_behaviour_properties::{
    BlockBehaviourPropertiesModel, BlockOffsetType, NoteBlockInstrumentModel, PostProcessModel,
    PushReactionModel, StatePredicateModel,
};
use crate::block_update::BlockPos;
use crate::fluid::FluidKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaDirectionModel {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl JavaDirectionModel {
    fn ordinal(self) -> usize {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

pub const BLOCK_BEHAVIOUR_UPDATE_SHAPE_ORDER: [JavaDirectionModel; 6] = [
    JavaDirectionModel::West,
    JavaDirectionModel::East,
    JavaDirectionModel::North,
    JavaDirectionModel::South,
    JavaDirectionModel::Down,
    JavaDirectionModel::Up,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportTypeModel {
    Full,
    Center,
    Rigid,
}

impl SupportTypeModel {
    fn ordinal(self) -> usize {
        match self {
            Self::Full => 0,
            Self::Center => 1,
            Self::Rigid => 2,
        }
    }
}

const SUPPORT_TYPE_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionBlockInteractionModel {
    Keep,
    Destroy,
    DestroyWithDecay,
    TriggerBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplosionActionModel {
    SpawnAfterBreak {
        drop_experience_hack: bool,
    },
    GetDrops {
        include_block_entity: bool,
        include_this_entity: bool,
        include_explosion_radius: bool,
    },
    EmitDrop {
        item: &'static str,
        pos: BlockPos,
    },
    SetAir {
        flags: i32,
    },
    WasExploded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExplosionHitPlanInput<'a> {
    pub state_is_air: bool,
    pub block_interaction: ExplosionBlockInteractionModel,
    pub drop_from_explosion: bool,
    pub has_block_entity: bool,
    pub has_direct_source_entity: bool,
    pub indirect_source_is_player: bool,
    pub drops: &'a [&'static str],
    pub pos: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegatedBlockCallModel {
    UpdateIndirectNeighbourShapes { update_limit: i32 },
    TriggerEvent { b0: i32, b1: i32 },
    HandleNeighborChanged { moved_by_piston: bool },
    OnPlace { moved_by_piston: bool },
    AffectNeighborsAfterRemoval { moved_by_piston: bool },
    Tick,
    RandomTick,
    EntityInside { is_precise: bool },
    SpawnAfterBreak { drop_experience: bool },
    GetDrops,
    UseItemOn,
    UseWithoutItem,
    Attack,
    UpdateShape,
    CanBeReplacedByItem,
    CanBeReplacedByFluid,
    CanSurvive,
    GetMenuProvider,
    GetTicker,
    GetCloneItemStack { include_data: bool },
    OnProjectileHit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeighborShapeUpdateModel {
    pub direction_to_neighbor: JavaDirectionModel,
    pub direction_from_neighbor: JavaDirectionModel,
    pub update_limit: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathComputationTypeModel {
    Land,
    Water,
    Air,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderShapeModel {
    Model,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeModel {
    Empty,
    Block,
    Custom {
        full_block: bool,
        bounds_size_large_enough_for_solid: bool,
        y_size_full: bool,
        extends_outside_block: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockBehaviourDefaults {
    pub has_collision: bool,
    pub is_randomly_ticking: bool,
    pub sound_type: &'static str,
    pub explosion_resistance: f32,
    pub destroy_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcclusionFacesModel {
    Empty,
    FullBlock,
    PerFace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockStateBaseCacheModel {
    pub collision_shape: ShapeModel,
    pub large_collision_shape: bool,
    pub face_sturdy: [bool; SUPPORT_TYPE_COUNT * 6],
    pub is_collision_shape_full_block: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStateBaseModel {
    pub owner_id: String,
    pub light_emission: u8,
    pub use_shape_for_light_occlusion: bool,
    pub is_air: bool,
    pub ignited_by_lava: bool,
    pub liquid: bool,
    pub legacy_solid: bool,
    pub push_reaction: PushReactionModel,
    pub map_color: String,
    pub destroy_speed: f32,
    pub requires_correct_tool_for_drops: bool,
    pub can_occlude: bool,
    pub is_redstone_conductor: StatePredicateModel,
    pub is_suffocating: StatePredicateModel,
    pub is_view_blocking: StatePredicateModel,
    pub post_process: PostProcessModel,
    pub emissive_rendering: StatePredicateModel,
    pub offset_type: BlockOffsetType,
    pub spawn_terrain_particles: bool,
    pub instrument: NoteBlockInstrumentModel,
    pub replaceable: bool,
    pub cache: Option<BlockStateBaseCacheModel>,
    pub fluid: Option<FluidKind>,
    pub is_randomly_ticking: bool,
    pub solid_render: bool,
    pub occlusion_shape: ShapeModel,
    pub occlusion_faces: OcclusionFacesModel,
    pub propagates_skylight_down: bool,
    pub light_dampening: i32,
}

impl BlockBehaviourDefaults {
    pub fn new(
        has_collision: bool,
        is_randomly_ticking: bool,
        sound_type: &'static str,
        explosion_resistance: f32,
        destroy_time: f32,
    ) -> Self {
        Self {
            has_collision,
            is_randomly_ticking,
            sound_type,
            explosion_resistance,
            destroy_time,
        }
    }

    pub fn update_shape<T>(&self, state: T) -> T {
        state
    }

    pub fn skip_rendering(&self) -> bool {
        false
    }

    pub fn use_without_item(&self) -> InteractionResult {
        InteractionResult::Pass
    }

    pub fn use_item_on(&self) -> InteractionResult {
        InteractionResult::TryWithEmptyHand
    }

    pub fn trigger_event(&self) -> bool {
        false
    }

    pub fn render_shape(&self) -> RenderShapeModel {
        RenderShapeModel::Model
    }

    pub fn use_shape_for_light_occlusion(&self) -> bool {
        false
    }

    pub fn is_signal_source(&self) -> bool {
        false
    }

    pub fn has_analog_output_signal(&self) -> bool {
        false
    }

    pub fn max_horizontal_offset(&self) -> f32 {
        0.25
    }

    pub fn max_vertical_offset(&self) -> f32 {
        0.2
    }

    pub fn should_changed_state_keep_block_entity(&self) -> bool {
        false
    }

    pub fn can_survive(&self) -> bool {
        true
    }

    pub fn analog_output_signal(&self) -> i32 {
        0
    }

    pub fn signal(&self) -> i32 {
        0
    }

    pub fn direct_signal(&self) -> i32 {
        0
    }

    pub fn is_randomly_ticking(&self) -> bool {
        self.is_randomly_ticking
    }

    pub fn get_sound_type(&self) -> &'static str {
        self.sound_type
    }

    pub fn default_destroy_time(&self) -> f32 {
        self.destroy_time
    }

    pub fn get_collision_shape(&self, shape: ShapeModel) -> ShapeModel {
        if self.has_collision {
            shape
        } else {
            ShapeModel::Empty
        }
    }

    pub fn get_visual_shape(&self, shape: ShapeModel) -> ShapeModel {
        self.get_collision_shape(shape)
    }

    pub fn get_block_support_shape(&self, shape: ShapeModel) -> ShapeModel {
        self.get_collision_shape(shape)
    }
}

impl BlockStateBaseModel {
    pub fn from_properties(
        owner_id: impl Into<String>,
        properties: &BlockBehaviourPropertiesModel,
        use_shape_for_light_occlusion: bool,
    ) -> Self {
        Self {
            owner_id: owner_id.into(),
            light_emission: properties.light_emission,
            use_shape_for_light_occlusion,
            is_air: properties.is_air,
            ignited_by_lava: properties.ignited_by_lava,
            liquid: properties.liquid,
            legacy_solid: false,
            push_reaction: properties.push_reaction,
            map_color: properties.map_color.clone(),
            destroy_speed: properties.destroy_time,
            requires_correct_tool_for_drops: properties.requires_correct_tool_for_drops,
            can_occlude: properties.can_occlude,
            is_redstone_conductor: properties.is_redstone_conductor,
            is_suffocating: properties.is_suffocating,
            is_view_blocking: properties.is_view_blocking,
            post_process: properties.post_process,
            emissive_rendering: properties.emissive_rendering,
            offset_type: properties.offset_type,
            spawn_terrain_particles: properties.spawn_terrain_particles,
            instrument: properties.instrument,
            replaceable: properties.replaceable,
            cache: None,
            fluid: None,
            is_randomly_ticking: false,
            solid_render: false,
            occlusion_shape: ShapeModel::Empty,
            occlusion_faces: OcclusionFacesModel::Empty,
            propagates_skylight_down: false,
            light_dampening: 0,
        }
    }

    pub fn init_cache(
        &mut self,
        properties: &BlockBehaviourPropertiesModel,
        owner_dynamic_shape: bool,
        collision_shape: ShapeModel,
        fluid: Option<FluidKind>,
        is_randomly_ticking: bool,
        owner_occlusion_shape: ShapeModel,
    ) {
        self.fluid = fluid;
        self.is_randomly_ticking = is_randomly_ticking;
        self.cache = (!owner_dynamic_shape).then(|| BlockStateBaseCacheModel {
            collision_shape,
            large_collision_shape: cache_large_collision_shape(collision_shape),
            face_sturdy: face_sturdy_table(collision_shape),
            is_collision_shape_full_block: collision_shape.is_full_block(),
        });
        self.legacy_solid = calculate_solid(
            properties.force_solid_on,
            properties.force_solid_off,
            self.cache.is_some(),
            collision_shape,
        );
        self.occlusion_shape = if self.can_occlude {
            owner_occlusion_shape
        } else {
            ShapeModel::Empty
        };
        self.solid_render = self.occlusion_shape.is_full_block();
        self.occlusion_faces = occlusion_faces_for_shape(self.occlusion_shape);
        self.propagates_skylight_down =
            propagates_skylight_down(owner_occlusion_shape.is_full_block(), fluid.is_none());
        self.light_dampening =
            get_light_dampening(self.solid_render, self.propagates_skylight_down);
    }

    pub fn blocks_motion(&self) -> bool {
        self.owner_id != "minecraft:cobweb"
            && self.owner_id != "minecraft:bamboo_sapling"
            && self.legacy_solid
    }

    pub fn is_solid(&self) -> bool {
        self.legacy_solid
    }

    pub fn has_large_collision_shape(&self) -> bool {
        self.cache
            .map(|cache| cache.large_collision_shape)
            .unwrap_or(true)
    }

    pub fn get_collision_shape(&self, fallback_shape: ShapeModel) -> ShapeModel {
        self.cache
            .map(|cache| cache.collision_shape)
            .unwrap_or(fallback_shape)
    }

    pub fn has_offset_function(&self) -> bool {
        self.offset_type != BlockOffsetType::None
    }

    pub fn get_offset(&self, pos: BlockPos) -> (f64, f64, f64) {
        crate::block_behaviour_properties::offset_for_type(self.offset_type, pos, 0.25, 0.2)
    }

    pub fn get_light_emission(&self) -> u8 {
        self.light_emission
    }

    pub fn can_be_replaced(&self) -> bool {
        self.replaceable
    }

    pub fn should_spawn_terrain_particles(&self) -> bool {
        self.spawn_terrain_particles
    }

    pub fn get_piston_push_reaction(&self) -> PushReactionModel {
        self.push_reaction
    }

    pub fn delegate_trigger_event(&self, b0: i32, b1: i32) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::TriggerEvent { b0, b1 }
    }

    pub fn delegate_neighbor_changed(&self, moved_by_piston: bool) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::HandleNeighborChanged { moved_by_piston }
    }

    pub fn delegate_on_place(&self, moved_by_piston: bool) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::OnPlace { moved_by_piston }
    }

    pub fn delegate_affect_neighbors_after_removal(
        &self,
        moved_by_piston: bool,
    ) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::AffectNeighborsAfterRemoval { moved_by_piston }
    }

    pub fn delegate_update_indirect_neighbour_shapes(
        &self,
        update_limit: i32,
    ) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::UpdateIndirectNeighbourShapes { update_limit }
    }

    pub fn delegate_tick(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::Tick
    }

    pub fn delegate_random_tick(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::RandomTick
    }

    pub fn delegate_entity_inside(&self, is_precise: bool) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::EntityInside { is_precise }
    }

    pub fn delegate_spawn_after_break(&self, drop_experience: bool) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::SpawnAfterBreak { drop_experience }
    }

    pub fn delegate_get_drops(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::GetDrops
    }

    pub fn delegate_use_item_on(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::UseItemOn
    }

    pub fn delegate_use_without_item(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::UseWithoutItem
    }

    pub fn delegate_attack(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::Attack
    }

    pub fn delegate_update_shape(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::UpdateShape
    }

    pub fn delegate_can_be_replaced_by_item(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::CanBeReplacedByItem
    }

    pub fn delegate_can_be_replaced_by_fluid(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::CanBeReplacedByFluid
    }

    pub fn delegate_can_survive(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::CanSurvive
    }

    pub fn delegate_get_menu_provider(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::GetMenuProvider
    }

    pub fn delegate_get_ticker(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::GetTicker
    }

    pub fn delegate_get_clone_item_stack(&self, include_data: bool) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::GetCloneItemStack { include_data }
    }

    pub fn delegate_on_projectile_hit(&self) -> DelegatedBlockCallModel {
        DelegatedBlockCallModel::OnProjectileHit
    }

    pub fn is_collision_shape_full_block(&self, fallback_shape_full_block: bool) -> bool {
        self.cache
            .map(|cache| cache.is_collision_shape_full_block)
            .unwrap_or(fallback_shape_full_block)
    }

    pub fn is_face_sturdy_cached(
        &self,
        direction: JavaDirectionModel,
        support_type: SupportTypeModel,
        fallback: bool,
    ) -> bool {
        if self.cache.is_some() {
            self.cache
                .map(|cache| cache.face_sturdy[face_support_index(direction, support_type)])
                .unwrap_or(fallback)
        } else {
            fallback
        }
    }
}

pub fn is_pathfindable(
    path_type: PathComputationTypeModel,
    collision_shape_full_block: bool,
    fluid: Option<FluidKind>,
) -> bool {
    match path_type {
        PathComputationTypeModel::Land | PathComputationTypeModel::Air => {
            !collision_shape_full_block
        }
        PathComputationTypeModel::Water => fluid == Some(FluidKind::Water),
    }
}

pub fn can_be_replaced_by_item(
    state_replaceable: bool,
    item_in_hand_empty: bool,
    item_is_this_block_item: bool,
) -> bool {
    state_replaceable && (item_in_hand_empty || !item_is_this_block_item)
}

pub fn can_be_replaced_by_fluid(state_replaceable: bool, state_solid: bool) -> bool {
    state_replaceable || !state_solid
}

pub fn get_light_dampening(solid_render: bool, propagates_skylight_down: bool) -> i32 {
    if solid_render {
        15
    } else if propagates_skylight_down {
        0
    } else {
        1
    }
}

pub fn get_shade_brightness(collision_shape_full_block: bool) -> f32 {
    if collision_shape_full_block {
        0.2
    } else {
        1.0
    }
}

pub fn propagates_skylight_down(shape_full_block: bool, fluid_empty: bool) -> bool {
    !shape_full_block && fluid_empty
}

pub fn get_destroy_progress(
    destroy_speed: f32,
    player_destroy_speed: f32,
    has_correct_tool_for_drops: bool,
) -> f32 {
    if destroy_speed == -1.0 {
        return 0.0;
    }
    let modifier = if has_correct_tool_for_drops {
        30.0
    } else {
        100.0
    };
    player_destroy_speed / destroy_speed / modifier
}

pub fn calculate_solid(
    force_solid_on: bool,
    force_solid_off: bool,
    cache_present: bool,
    collision_shape: ShapeModel,
) -> bool {
    if force_solid_on {
        return true;
    }
    if force_solid_off || !cache_present {
        return false;
    }
    match collision_shape {
        ShapeModel::Empty => false,
        ShapeModel::Block => true,
        ShapeModel::Custom {
            bounds_size_large_enough_for_solid,
            y_size_full,
            ..
        } => bounds_size_large_enough_for_solid || y_size_full,
    }
}

pub fn cache_large_collision_shape(shape: ShapeModel) -> bool {
    match shape {
        ShapeModel::Custom {
            extends_outside_block,
            ..
        } => extends_outside_block,
        ShapeModel::Empty | ShapeModel::Block => false,
    }
}

pub fn get_seed(pos: BlockPos) -> i64 {
    crate::block_behaviour_properties::mth_get_seed(pos.x, pos.y, pos.z)
}

pub fn plan_update_neighbour_shapes(update_limit: i32) -> Vec<NeighborShapeUpdateModel> {
    BLOCK_BEHAVIOUR_UPDATE_SHAPE_ORDER
        .iter()
        .map(|direction| NeighborShapeUpdateModel {
            direction_to_neighbor: *direction,
            direction_from_neighbor: direction.opposite(),
            update_limit,
        })
        .collect()
}

pub fn face_support_index(direction: JavaDirectionModel, support_type: SupportTypeModel) -> usize {
    direction.ordinal() * SUPPORT_TYPE_COUNT + support_type.ordinal()
}

pub fn face_sturdy_table(shape: ShapeModel) -> [bool; SUPPORT_TYPE_COUNT * 6] {
    let supported = match shape {
        ShapeModel::Block => [true, true, true],
        ShapeModel::Empty => [false, false, false],
        ShapeModel::Custom { full_block, .. } => [full_block, full_block, full_block],
    };
    let mut table = [false; SUPPORT_TYPE_COUNT * 6];
    for direction in [
        JavaDirectionModel::Down,
        JavaDirectionModel::Up,
        JavaDirectionModel::North,
        JavaDirectionModel::South,
        JavaDirectionModel::West,
        JavaDirectionModel::East,
    ] {
        for support_type in [
            SupportTypeModel::Full,
            SupportTypeModel::Center,
            SupportTypeModel::Rigid,
        ] {
            table[face_support_index(direction, support_type)] = supported[support_type.ordinal()];
        }
    }
    table
}

pub fn plan_on_explosion_hit(input: ExplosionHitPlanInput<'_>) -> Vec<ExplosionActionModel> {
    if input.state_is_air || input.block_interaction == ExplosionBlockInteractionModel::TriggerBlock
    {
        return Vec::new();
    }

    let mut actions = Vec::new();
    if input.drop_from_explosion {
        actions.push(ExplosionActionModel::SpawnAfterBreak {
            drop_experience_hack: input.indirect_source_is_player,
        });
        actions.push(ExplosionActionModel::GetDrops {
            include_block_entity: input.has_block_entity,
            include_this_entity: input.has_direct_source_entity,
            include_explosion_radius: input.block_interaction
                == ExplosionBlockInteractionModel::DestroyWithDecay,
        });
        actions.extend(
            input
                .drops
                .iter()
                .map(|item| ExplosionActionModel::EmitDrop {
                    item,
                    pos: input.pos,
                }),
        );
    }
    actions.push(ExplosionActionModel::SetAir { flags: 3 });
    actions.push(ExplosionActionModel::WasExploded);
    actions
}

impl ShapeModel {
    fn is_full_block(self) -> bool {
        matches!(
            self,
            ShapeModel::Block
                | ShapeModel::Custom {
                    full_block: true,
                    ..
                }
        )
    }
}

pub fn occlusion_faces_for_shape(shape: ShapeModel) -> OcclusionFacesModel {
    match shape {
        ShapeModel::Empty => OcclusionFacesModel::Empty,
        shape if shape.is_full_block() => OcclusionFacesModel::FullBlock,
        ShapeModel::Custom { .. } => OcclusionFacesModel::PerFace,
        ShapeModel::Block => OcclusionFacesModel::FullBlock,
    }
}
