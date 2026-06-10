//! Physical properties of every vanilla 26.1.2 block state, captured directly from
//! the official server's live registries.
//!
//! The data is NOT parsed from decompiled source: `tools/BlockPropertyDump.java`
//! bootstraps the real server jar's registries and dumps each `BlockState`'s
//! accessor surface (`BlockBehaviour.BlockStateBase`: `getDestroySpeed`,
//! `getMapColor`, `getSoundType`, `getLightEmission`, `getLightDampening`,
//! `blocksMotion`, `isSolidRender`, `propagatesSkylightDown`, voxel shapes, fluid
//! state, pathfindability, ...) plus block-level constants (`getExplosionResistance`,
//! `getFriction`, `getSpeedFactor`, `getJumpFactor`). The dump is vendored at
//! `vanilla-data/reports/block_properties_26_1_2.json.gz` and decoded once into
//! dense tables indexed by the global block-state id from [`crate::block_states`].
//!
//! Voxel shapes are interned (942 unique shapes across 29873 states) exactly as the
//! probe emitted them: each shape is a list of AABBs in block-local coordinates,
//! matching Java `VoxelShape.toAabbs()`.

#![allow(dead_code)]

use std::collections::HashMap;
use std::io::Read;
use std::sync::LazyLock;

use serde_json::Value;

use crate::block_states::{block_state_entries, VANILLA_BLOCK_STATE_COUNT_26_1_2};

/// One axis-aligned box of a voxel shape, `[min_x, min_y, min_z, max_x, max_y, max_z]`
/// in block-local coordinates (a full cube is `[0, 0, 0, 1, 1, 1]`).
pub type ShapeBox = [f64; 6];

/// Block-level physical constants shared by all states of a block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockPhysics {
    pub explosion_resistance: f32,
    pub friction: f32,
    pub speed_factor: f32,
    pub jump_factor: f32,
}

/// The five sound events plus volume/pitch of a Java `SoundType` constant.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundTypeInfo {
    pub name: String,
    pub volume: f32,
    pub pitch: f32,
    pub break_sound: String,
    pub step_sound: String,
    pub place_sound: String,
    pub hit_sound: String,
    pub fall_sound: String,
}

/// Java `PushReaction` for piston interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushReaction {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

/// The fluid contained in a block state (`BlockStateBase.getFluidState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateFluid {
    Empty,
    Water { amount: u8, source: bool },
    Lava { amount: u8, source: bool },
}

/// Per-state physical properties, mirroring `BlockBehaviour.BlockStateBase`.
#[derive(Debug, Clone, PartialEq)]
pub struct StatePhysics {
    pub destroy_speed: f32,
    /// Index into the vanilla `MapColor` id space (see [`map_color_name`]).
    pub map_color: u8,
    /// Index into [`sound_types`].
    pub sound_type: u16,
    pub light_emission: u8,
    pub light_dampening: u8,
    pub is_air: bool,
    pub liquid: bool,
    pub blocks_motion: bool,
    pub is_solid: bool,
    pub propagates_skylight_down: bool,
    pub can_occlude: bool,
    pub is_solid_render: bool,
    pub use_shape_for_light_occlusion: bool,
    pub large_collision_shape: bool,
    pub requires_correct_tool_for_drops: bool,
    pub ignited_by_lava: bool,
    pub push_reaction: PushReaction,
    pub is_signal_source: bool,
    pub is_randomly_ticking: bool,
    pub has_block_entity: bool,
    pub replaceable: bool,
    /// Java `NoteBlockInstrument` constant name.
    pub instrument: &'static str,
    /// Java `RenderShape` constant name.
    pub render_shape: &'static str,
    pub pathfind_land: bool,
    pub pathfind_air: bool,
    pub pathfind_water: bool,
    pub suffocating: bool,
    pub view_blocking: bool,
    pub fluid: StateFluid,
    /// Indices into [`shape`].
    pub shape: u16,
    pub collision_shape: u16,
    pub occlusion_shape: u16,
    pub interaction_shape: u16,
    pub support_shape: u16,
}

struct PropertyTables {
    blocks: Vec<BlockPhysics>,
    states: Vec<StatePhysics>,
    shapes: Vec<Vec<ShapeBox>>,
    sound_types: Vec<SoundTypeInfo>,
    map_color_names: HashMap<u8, String>,
}

static TABLES: LazyLock<PropertyTables> = LazyLock::new(|| {
    let raw = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vanilla-data/reports/block_properties_26_1_2.json.gz"
    ));
    let mut decoded = String::new();
    flate2::read::GzDecoder::new(raw.as_slice())
        .read_to_string(&mut decoded)
        .unwrap_or_else(|err| panic!("vendored block properties report must be valid gzip: {err}"));
    let root: Value =
        serde_json::from_str(&decoded)
        .unwrap_or_else(|err| panic!("vendored block properties report must be valid JSON: {err}"));
    assert_eq!(
        root["format"].as_str(),
        Some("vibecraft-block-properties-v1"),
        "unexpected block properties report format"
    );
    parse_tables(&root)
});

fn parse_tables(root: &Value) -> PropertyTables {
    let shapes = root["shapes"]
        .as_array()
        .unwrap_or_else(|| panic!("shapes array"))
        .iter()
        .map(|shape| {
            shape
                .as_array()
                .unwrap_or_else(|| panic!("shape box list"))
                .iter()
                .map(|aabb| {
                    let coordinates = aabb.as_array().unwrap_or_else(|| panic!("shape box"));
                    let mut shape_box = [0.0; 6];
                    for (slot, value) in shape_box.iter_mut().zip(coordinates) {
                        *slot = value.as_f64().unwrap_or_else(|| panic!("shape coordinate"));
                    }
                    shape_box
                })
                .collect()
        })
        .collect();

    let mut sound_types = Vec::new();
    let mut sound_type_indices = HashMap::new();
    for (name, entry) in root["sound_types"].as_object().unwrap_or_else(|| panic!("sound types")) {
        sound_type_indices.insert(name.clone(), sound_types.len() as u16);
        sound_types.push(SoundTypeInfo {
            name: name.clone(),
            volume: float(entry, "volume"),
            pitch: float(entry, "pitch"),
            break_sound: string(entry, "break"),
            step_sound: string(entry, "step"),
            place_sound: string(entry, "place"),
            hit_sound: string(entry, "hit"),
            fall_sound: string(entry, "fall"),
        });
    }

    let map_color_names = root["map_colors"]
        .as_object()
        .unwrap_or_else(|| panic!("map colors"))
        .iter()
        .map(|(id, name)| {
            (
                id.parse::<u8>().unwrap_or_else(|err| panic!("map color id: {err}")),
                name.as_str().unwrap_or_else(|| panic!("map color name")).to_string(),
            )
        })
        .collect();

    let report_blocks = root["blocks"].as_object().unwrap_or_else(|| panic!("blocks object"));
    let mut blocks = Vec::with_capacity(block_state_entries().len());
    let mut states: Vec<Option<StatePhysics>> = vec![None; VANILLA_BLOCK_STATE_COUNT_26_1_2];
    for entry in block_state_entries() {
        let block = report_blocks
            .get(entry.registry_id)
            .unwrap_or_else(|| panic!("missing block {} in properties report", entry.registry_id));
        blocks.push(BlockPhysics {
            explosion_resistance: float(block, "explosion_resistance"),
            friction: float(block, "friction"),
            speed_factor: float(block, "speed_factor"),
            jump_factor: float(block, "jump_factor"),
        });
        for state in block["states"].as_array().unwrap_or_else(|| panic!("states array")) {
            let id = state["id"].as_i64().unwrap_or_else(|| panic!("state id")) as usize;
            states[id] = Some(parse_state(state, &sound_type_indices));
        }
    }

    PropertyTables {
        blocks,
        states: states
            .into_iter()
            .enumerate()
            .map(|(id, state)| state.unwrap_or_else(|| panic!("state {id} missing from report")))
            .collect(),
        shapes,
        sound_types,
        map_color_names,
    }
}

fn parse_state(state: &Value, sound_type_indices: &HashMap<String, u16>) -> StatePhysics {
    let fluid = &state["fluid"];
    let fluid = match fluid["type"].as_str().unwrap_or_else(|| panic!("fluid type")) {
        "minecraft:empty" => StateFluid::Empty,
        "minecraft:water" | "minecraft:flowing_water" => StateFluid::Water {
            amount: fluid["amount"].as_u64().unwrap_or_else(|| panic!("fluid amount")) as u8,
            source: fluid["source"].as_bool().unwrap_or_else(|| panic!("fluid source")),
        },
        "minecraft:lava" | "minecraft:flowing_lava" => StateFluid::Lava {
            amount: fluid["amount"].as_u64().unwrap_or_else(|| panic!("fluid amount")) as u8,
            source: fluid["source"].as_bool().unwrap_or_else(|| panic!("fluid source")),
        },
        other => panic!("unexpected fluid type {other}"),
    };
    StatePhysics {
        destroy_speed: float(state, "destroy_speed"),
        map_color: state["map_color"].as_u64().unwrap_or_else(|| panic!("map color")) as u8,
        sound_type: sound_type_indices[state["sound_type"].as_str().unwrap_or_else(|| panic!("sound type"))],
        light_emission: state["light_emission"].as_u64().unwrap_or_else(|| panic!("light emission")) as u8,
        light_dampening: state["light_dampening"].as_u64().unwrap_or_else(|| panic!("light dampening")) as u8,
        is_air: boolean(state, "is_air"),
        liquid: boolean(state, "liquid"),
        blocks_motion: boolean(state, "blocks_motion"),
        is_solid: boolean(state, "is_solid"),
        propagates_skylight_down: boolean(state, "propagates_skylight_down"),
        can_occlude: boolean(state, "can_occlude"),
        is_solid_render: boolean(state, "is_solid_render"),
        use_shape_for_light_occlusion: boolean(state, "use_shape_for_light_occlusion"),
        large_collision_shape: boolean(state, "large_collision_shape"),
        requires_correct_tool_for_drops: boolean(state, "requires_correct_tool_for_drops"),
        ignited_by_lava: boolean(state, "ignited_by_lava"),
        push_reaction: match state["push_reaction"].as_str().unwrap_or_else(|| panic!("push reaction")) {
            "NORMAL" => PushReaction::Normal,
            "DESTROY" => PushReaction::Destroy,
            "BLOCK" => PushReaction::Block,
            "IGNORE" => PushReaction::Ignore,
            "PUSH_ONLY" => PushReaction::PushOnly,
            other => panic!("unexpected push reaction {other}"),
        },
        is_signal_source: boolean(state, "is_signal_source"),
        is_randomly_ticking: boolean(state, "is_randomly_ticking"),
        has_block_entity: boolean(state, "has_block_entity"),
        replaceable: boolean(state, "replaceable"),
        instrument: intern_constant(state["instrument"].as_str().unwrap_or_else(|| panic!("instrument"))),
        render_shape: intern_constant(state["render_shape"].as_str().unwrap_or_else(|| panic!("render shape"))),
        pathfind_land: boolean(state, "pathfind_land"),
        pathfind_air: boolean(state, "pathfind_air"),
        pathfind_water: boolean(state, "pathfind_water"),
        suffocating: boolean(state, "suffocating"),
        view_blocking: boolean(state, "view_blocking"),
        fluid,
        shape: shape_index(state, "shape"),
        collision_shape: shape_index(state, "collision_shape"),
        occlusion_shape: shape_index(state, "occlusion_shape"),
        interaction_shape: shape_index(state, "interaction_shape"),
        support_shape: shape_index(state, "support_shape"),
    }
}

fn float(value: &Value, key: &str) -> f32 {
    value[key].as_f64().unwrap_or_else(|| panic!("missing float {key}")) as f32
}

fn boolean(value: &Value, key: &str) -> bool {
    value[key].as_bool().unwrap_or_else(|| panic!("missing bool {key}"))
}

fn string(value: &Value, key: &str) -> String {
    value[key]
        .as_str()
        .unwrap_or_else(|| panic!("missing string {key}"))
        .to_string()
}

fn shape_index(state: &Value, key: &str) -> u16 {
    state[key].as_u64().unwrap_or_else(|| panic!("missing shape index {key}")) as u16
}

/// Interns Java enum constant names so [`StatePhysics`] stays `Copy`-friendly and
/// comparisons are cheap. The constant sets are tiny (`NoteBlockInstrument`,
/// `RenderShape`), so leaking each distinct name once is fine.
fn intern_constant(name: &str) -> &'static str {
    static INTERNED: LazyLock<std::sync::Mutex<HashMap<String, &'static str>>> =
        LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
    let mut interned = INTERNED.lock().unwrap_or_else(|err| panic!("intern lock poisoned: {err}"));
    if let Some(existing) = interned.get(name) {
        return existing;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    interned.insert(name.to_string(), leaked);
    leaked
}

/// Block-level physics for the block owning a registry index (protocol order).
pub fn block_physics_by_registry_index(index: usize) -> Option<&'static BlockPhysics> {
    TABLES.blocks.get(index)
}

/// Block-level physics by registry id (e.g. `minecraft:slime_block`).
pub fn block_physics(registry_id: &str) -> Option<&'static BlockPhysics> {
    let entry = crate::block_states::block_state_entry(registry_id)?;
    let index = block_state_entries()
        .iter()
        .position(|candidate| std::ptr::eq(candidate, entry))?;
    TABLES.blocks.get(index)
}

/// Per-state physics for a global block-state id.
pub fn state_physics(state_id: i32) -> Option<&'static StatePhysics> {
    usize::try_from(state_id).ok().and_then(|id| TABLES.states.get(id))
}

/// Per-state physics for a `block[prop=value,...]` state name.
pub fn state_physics_by_name(name: &str) -> Option<&'static StatePhysics> {
    state_physics(crate::block_states::network_id_for_block_state(name)?)
}

/// The interned voxel shape boxes for a shape index from [`StatePhysics`].
pub fn shape(index: u16) -> &'static [ShapeBox] {
    &TABLES.shapes[index as usize]
}

/// All Java `SoundType` constants captured by the probe.
pub fn sound_types() -> &'static [SoundTypeInfo] {
    &TABLES.sound_types
}

/// The `SoundType` of a state.
pub fn state_sound_type(state: &StatePhysics) -> &'static SoundTypeInfo {
    &TABLES.sound_types[state.sound_type as usize]
}

/// Java `MapColor` constant name for a map-color id.
pub fn map_color_name(id: u8) -> Option<&'static str> {
    TABLES.map_color_names.get(&id).map(String::as_str)
}

#[cfg(test)]
mod tests;
