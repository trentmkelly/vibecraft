#![allow(dead_code)]

use crate::block_behavior::InteractionResult;
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

#[cfg(test)]
mod tests {
    use super::{
        cache_large_collision_shape, calculate_solid, can_be_replaced_by_fluid,
        can_be_replaced_by_item, get_destroy_progress, get_light_dampening, get_seed,
        get_shade_brightness, is_pathfindable, propagates_skylight_down, BlockBehaviourDefaults,
        PathComputationTypeModel, RenderShapeModel, ShapeModel,
    };
    use crate::block_behavior::InteractionResult;
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
}
