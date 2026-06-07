#![allow(dead_code)]

use crate::block_behavior::InteractionResult;
use crate::block_behaviour_properties::{
    BlockBehaviourPropertiesModel, BlockOffsetType, NoteBlockInstrumentModel, PostProcessModel,
    PushReactionModel, StatePredicateModel,
};
use crate::block_update::BlockPos;
use crate::fluid::FluidKind;

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

#[cfg(test)]
mod tests {
    use super::{
        cache_large_collision_shape, calculate_solid, can_be_replaced_by_fluid,
        can_be_replaced_by_item, get_destroy_progress, get_light_dampening, get_seed,
        get_shade_brightness, is_pathfindable, occlusion_faces_for_shape, propagates_skylight_down,
        BlockBehaviourDefaults, BlockStateBaseModel, OcclusionFacesModel, PathComputationTypeModel,
        RenderShapeModel, ShapeModel,
    };
    use crate::block_behavior::InteractionResult;
    use crate::block_behaviour_properties::{
        BlockBehaviourPropertiesModel, BlockOffsetType, NoteBlockInstrumentModel,
        PushReactionModel, StatePredicateModel,
    };
    use crate::block_update::BlockPos;
    use crate::fluid::FluidKind;

    #[test]
    fn block_behaviour_default_returns_match_java_noop_methods() {
        let behaviour = BlockBehaviourDefaults::new(true, true, "stone", 6.0, 1.5);
        assert_eq!(behaviour.update_shape("state"), "state");
        assert!(!behaviour.skip_rendering());
        assert_eq!(behaviour.use_without_item(), InteractionResult::Pass);
        assert_eq!(behaviour.use_item_on(), InteractionResult::TryWithEmptyHand);
        assert!(!behaviour.trigger_event());
        assert_eq!(behaviour.render_shape(), RenderShapeModel::Model);
        assert!(!behaviour.use_shape_for_light_occlusion());
        assert!(!behaviour.is_signal_source());
        assert!(!behaviour.has_analog_output_signal());
        assert!(!behaviour.should_changed_state_keep_block_entity());
        assert!(behaviour.can_survive());
        assert_eq!(behaviour.analog_output_signal(), 0);
        assert_eq!(behaviour.signal(), 0);
        assert_eq!(behaviour.direct_signal(), 0);
    }

    #[test]
    fn block_behaviour_constructor_backed_defaults_match_java_fields() {
        let behaviour = BlockBehaviourDefaults::new(false, true, "wood", 2.5, 2.0);
        assert!(behaviour.is_randomly_ticking());
        assert_eq!(behaviour.get_sound_type(), "wood");
        assert_eq!(behaviour.default_destroy_time(), 2.0);
        assert_eq!(behaviour.explosion_resistance, 2.5);
        assert_eq!(behaviour.max_horizontal_offset(), 0.25);
        assert_eq!(behaviour.max_vertical_offset(), 0.2);
        assert_eq!(
            behaviour.get_collision_shape(ShapeModel::Block),
            ShapeModel::Empty
        );
    }

    #[test]
    fn block_behaviour_pathfinding_replaceability_and_shapes_match_java_defaults() {
        assert!(!is_pathfindable(PathComputationTypeModel::Land, true, None));
        assert!(is_pathfindable(PathComputationTypeModel::Land, false, None));
        assert!(!is_pathfindable(PathComputationTypeModel::Air, true, None));
        assert!(is_pathfindable(PathComputationTypeModel::Air, false, None));
        assert!(is_pathfindable(
            PathComputationTypeModel::Water,
            true,
            Some(FluidKind::Water)
        ));
        assert!(!is_pathfindable(
            PathComputationTypeModel::Water,
            false,
            Some(FluidKind::Lava)
        ));

        assert!(can_be_replaced_by_item(true, true, true));
        assert!(can_be_replaced_by_item(true, false, false));
        assert!(!can_be_replaced_by_item(true, false, true));
        assert!(!can_be_replaced_by_item(false, true, false));
        assert!(can_be_replaced_by_fluid(false, false));
        assert!(can_be_replaced_by_fluid(true, true));
        assert!(!can_be_replaced_by_fluid(false, true));
    }

    #[test]
    fn block_behaviour_light_shade_and_skylight_defaults_match_java() {
        assert_eq!(get_light_dampening(true, true), 15);
        assert_eq!(get_light_dampening(false, true), 0);
        assert_eq!(get_light_dampening(false, false), 1);
        assert_eq!(get_shade_brightness(true), 0.2);
        assert_eq!(get_shade_brightness(false), 1.0);
        assert!(!propagates_skylight_down(true, true));
        assert!(!propagates_skylight_down(false, false));
        assert!(propagates_skylight_down(false, true));
    }

    #[test]
    fn block_behaviour_destroy_progress_and_seed_match_java_formulas() {
        assert_eq!(get_destroy_progress(-1.0, 8.0, true), 0.0);
        assert!((get_destroy_progress(1.5, 8.0, true) - 0.17777778).abs() < 1e-7);
        assert!((get_destroy_progress(1.5, 8.0, false) - 0.053333335).abs() < 1e-7);
        assert_eq!(
            get_seed(BlockPos { x: 1, y: 64, z: 2 }),
            -117_935_115_545_999
        );
    }

    #[test]
    fn block_state_base_solid_and_cache_helpers_match_java_cache_rules() {
        assert!(calculate_solid(true, false, false, ShapeModel::Empty));
        assert!(!calculate_solid(false, true, true, ShapeModel::Block));
        assert!(!calculate_solid(false, false, false, ShapeModel::Block));
        assert!(!calculate_solid(false, false, true, ShapeModel::Empty));
        assert!(calculate_solid(false, false, true, ShapeModel::Block));
        assert!(calculate_solid(
            false,
            false,
            true,
            ShapeModel::Custom {
                full_block: false,
                bounds_size_large_enough_for_solid: true,
                y_size_full: false,
                extends_outside_block: false,
            }
        ));
        assert!(!cache_large_collision_shape(ShapeModel::Block));
        assert!(cache_large_collision_shape(ShapeModel::Custom {
            full_block: false,
            bounds_size_large_enough_for_solid: false,
            y_size_full: false,
            extends_outside_block: true,
        }));
    }

    #[test]
    fn block_state_base_constructor_copies_java_property_fields() {
        let properties = BlockBehaviourPropertiesModel::of()
            .map_color("wood")
            .light_level(9)
            .strength(2.0, 3.0)
            .requires_correct_tool_for_drops()
            .push_reaction(PushReactionModel::Block)
            .air()
            .ignited_by_lava()
            .liquid()
            .no_occlusion()
            .redstone_conductor_predicate(StatePredicateModel::Custom)
            .suffocating_predicate(StatePredicateModel::False)
            .view_blocking_predicate(StatePredicateModel::False)
            .emissive_rendering(StatePredicateModel::Custom)
            .offset_type(BlockOffsetType::Xz)
            .no_terrain_particles()
            .instrument(NoteBlockInstrumentModel::Bell)
            .replaceable();
        let state = BlockStateBaseModel::from_properties("minecraft:test", &properties, true);

        assert_eq!(state.owner_id, "minecraft:test");
        assert_eq!(state.get_light_emission(), 9);
        assert!(state.use_shape_for_light_occlusion);
        assert!(state.is_air);
        assert!(state.ignited_by_lava);
        assert!(state.liquid);
        assert_eq!(state.get_piston_push_reaction(), PushReactionModel::Block);
        assert_eq!(state.map_color, "wood");
        assert_eq!(state.destroy_speed, 2.0);
        assert!(state.requires_correct_tool_for_drops);
        assert!(!state.can_occlude);
        assert_eq!(state.is_redstone_conductor, StatePredicateModel::Custom);
        assert_eq!(state.is_suffocating, StatePredicateModel::False);
        assert_eq!(state.is_view_blocking, StatePredicateModel::False);
        assert_eq!(state.emissive_rendering, StatePredicateModel::Custom);
        assert!(state.has_offset_function());
        assert!(!state.should_spawn_terrain_particles());
        assert_eq!(state.instrument, NoteBlockInstrumentModel::Bell);
        assert!(state.can_be_replaced());
    }

    #[test]
    fn block_state_base_init_cache_matches_java_cached_state_fields() {
        let properties = BlockBehaviourPropertiesModel::of();
        let mut state = BlockStateBaseModel::from_properties("minecraft:stone", &properties, false);
        state.init_cache(
            &properties,
            false,
            ShapeModel::Block,
            None,
            true,
            ShapeModel::Block,
        );

        assert_eq!(state.fluid, None);
        assert!(state.is_randomly_ticking);
        assert_eq!(
            state.get_collision_shape(ShapeModel::Empty),
            ShapeModel::Block
        );
        assert!(state.is_solid());
        assert!(state.blocks_motion());
        assert_eq!(state.occlusion_shape, ShapeModel::Block);
        assert!(state.solid_render);
        assert_eq!(state.occlusion_faces, OcclusionFacesModel::FullBlock);
        assert!(!state.propagates_skylight_down);
        assert_eq!(state.light_dampening, 15);
        assert!(!state.has_large_collision_shape());
    }

    #[test]
    fn block_state_base_dynamic_shape_cache_absence_matches_java() {
        let properties = BlockBehaviourPropertiesModel::of();
        let mut state =
            BlockStateBaseModel::from_properties("minecraft:moving_piston", &properties, false);
        state.init_cache(
            &properties,
            true,
            ShapeModel::Block,
            Some(FluidKind::Water),
            false,
            ShapeModel::Empty,
        );

        assert!(state.cache.is_none());
        assert!(!state.is_solid());
        assert!(state.has_large_collision_shape());
        assert_eq!(
            state.get_collision_shape(ShapeModel::Custom {
                full_block: false,
                bounds_size_large_enough_for_solid: false,
                y_size_full: false,
                extends_outside_block: true,
            }),
            ShapeModel::Custom {
                full_block: false,
                bounds_size_large_enough_for_solid: false,
                y_size_full: false,
                extends_outside_block: true,
            }
        );
    }

    #[test]
    fn block_state_base_blocks_motion_and_occlusion_face_rules_match_java() {
        let properties = BlockBehaviourPropertiesModel::of();
        let mut cobweb =
            BlockStateBaseModel::from_properties("minecraft:cobweb", &properties, false);
        cobweb.init_cache(
            &properties,
            false,
            ShapeModel::Block,
            None,
            false,
            ShapeModel::Block,
        );
        assert!(cobweb.is_solid());
        assert!(!cobweb.blocks_motion());

        let mut bamboo =
            BlockStateBaseModel::from_properties("minecraft:bamboo_sapling", &properties, false);
        bamboo.init_cache(
            &properties,
            false,
            ShapeModel::Block,
            None,
            false,
            ShapeModel::Block,
        );
        assert!(bamboo.is_solid());
        assert!(!bamboo.blocks_motion());

        assert_eq!(
            occlusion_faces_for_shape(ShapeModel::Empty),
            OcclusionFacesModel::Empty
        );
        assert_eq!(
            occlusion_faces_for_shape(ShapeModel::Block),
            OcclusionFacesModel::FullBlock
        );
        assert_eq!(
            occlusion_faces_for_shape(ShapeModel::Custom {
                full_block: false,
                bounds_size_large_enough_for_solid: false,
                y_size_full: false,
                extends_outside_block: false,
            }),
            OcclusionFacesModel::PerFace
        );
    }

    #[test]
    fn block_state_base_offset_accessor_matches_java_state_offset() {
        let properties = BlockBehaviourPropertiesModel::of().offset_type(BlockOffsetType::Xz);
        let state = BlockStateBaseModel::from_properties("minecraft:grass", &properties, false);
        assert_eq!(
            state.get_offset(BlockPos { x: 1, y: 99, z: 2 }),
            (-0.11666666666666667, 0.0, -0.18333333333333335)
        );
    }
}
