//! Maps block-state names to [`LightBlockProperties`], exactly.
//!
//! Every field comes from the authoritative per-state tables in
//! [`crate::block_properties`], which were probed from the official server's
//! live registries (`tools/BlockPropertyDump.java`):
//!
//! - `opacity` <- `BlockState.getLightDampening()`
//! - `emission` <- `BlockState.getLightEmission()`
//! - `can_occlude` <- `BlockState.canOcclude()`
//! - `uses_shape_for_light_occlusion` <- `BlockState.useShapeForLightOcclusion()`
//! - `occlusion` <- `BlockState.getOcclusionShape()` as an occlusion-universe
//!   position whose vendored matrices answer `Shapes.mergedFaceOccludes` /
//!   `Shapes.faceShapeOccludes` exactly.
//!
//! State names with properties (e.g. `minecraft:redstone_torch[lit=true]`)
//! resolve to that exact state; bare block names resolve to the default state,
//! both matching `NbtUtils.readBlockState` semantics. Unknown names fall back to
//! the Bedrock default exactly like `LightEngine.getState` does for null chunks.

use crate::block_properties::{occlusion_position, state_physics_by_name};
use crate::lighting::light_chunk::LightBlockProperties;

/// Java equivalent: reading the light-relevant `BlockBehaviour.BlockStateBase`
/// cache fields for the state named `block_name`.
pub fn light_properties_for(block_name: &str) -> LightBlockProperties {
    let Some(state) = state_physics_by_name(block_name) else {
        return LightBlockProperties::BEDROCK_FALLBACK;
    };
    LightBlockProperties {
        opacity: state.light_dampening,
        emission: state.light_emission,
        can_occlude: state.can_occlude,
        uses_shape_for_light_occlusion: state.use_shape_for_light_occlusion,
        occlusion: occlusion_position(state.occlusion_shape),
    }
}

/// Java: `BlockState.getLightEmission` projected via `light_properties_for`.
pub fn emission_for(block_name: &str) -> u8 {
    light_properties_for(block_name).emission
}
