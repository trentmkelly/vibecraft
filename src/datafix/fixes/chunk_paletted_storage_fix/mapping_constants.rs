//! `ChunkPalettedStorageFix.MappingConstants`: the block states the fix produces
//! for blocks whose new state depends on neighbours or block entities.
//!
//! The Java palette is identity based, so every constant `Dynamic` instance gets
//! its own identity number here (after the identities of `BlockStateData.MAP`).

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::datafix::fixes::block_state_data::{map_group_count, BlockState};

/// Identity of a block state instance.
pub type StateId = u32;

/// `ChunkPalettedStorageFix.MappingConstants.FIX`: blocks that need a fix-up pass.
pub const FIX: &[usize] = &[
    2, 3, 110, 140, 144, 25, 86, 26, 176, 177, 175, 64, 71, 193, 194, 195, 196, 197,
];

/// `MappingConstants.VIRTUAL`: blocks whose state depends on neighbouring chunks.
pub const VIRTUAL: &[usize] = &[
    54, 146, 25, 26, 51, 53, 67, 108, 109, 114, 128, 134, 135, 136, 156, 163, 164, 180, 203, 55,
    85, 113, 188, 189, 190, 191, 192, 93, 94, 101, 102, 160, 106, 107, 183, 184, 185, 186, 187,
    132, 139, 199,
];

const DYE_COLORS: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

const FACINGS: [&str; 4] = ["east", "north", "south", "west"];

/// The constant states and their lookup tables.
pub struct MappingConstants {
    states: Vec<BlockState>,
    base: StateId,
    pub air: StateId,
    pub pumpkin: StateId,
    pub snowy_podzol: StateId,
    pub snowy_grass: StateId,
    pub snowy_mycelium: StateId,
    pub upper_sunflower: StateId,
    pub upper_lilac: StateId,
    pub upper_tall_grass: StateId,
    pub upper_large_fern: StateId,
    pub upper_rose_bush: StateId,
    pub upper_peony: StateId,
    pub flower_pot: HashMap<String, StateId>,
    pub skull: HashMap<String, StateId>,
    pub door: HashMap<String, StateId>,
    pub note_block: HashMap<String, StateId>,
    pub bed: HashMap<String, StateId>,
    pub banner: HashMap<String, StateId>,
}

/// `ExtraDataFixUtils.blockState(id, properties)`.
fn block_state(name: &str, properties: &[(&str, &str)]) -> BlockState {
    let mut props: Vec<(String, String)> = properties
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect();
    props.sort();
    (name.to_string(), props)
}

struct Builder {
    states: Vec<BlockState>,
    base: StateId,
}

impl Builder {
    fn add(&mut self, name: &str, properties: &[(&str, &str)]) -> StateId {
        self.states.push(block_state(name, properties));
        self.base + (self.states.len() as StateId - 1)
    }
}

impl MappingConstants {
    /// The process-wide constants.
    pub fn get() -> &'static MappingConstants {
        static CONSTANTS: OnceLock<MappingConstants> = OnceLock::new();
        CONSTANTS.get_or_init(MappingConstants::build)
    }

    /// The state behind an identity number.
    pub fn state(&self, id: StateId) -> &BlockState {
        &self.states[(id - self.base) as usize]
    }

    /// Whether `id` refers to a constant (as opposed to a `BlockStateData.MAP` entry).
    pub fn is_constant(&self, id: StateId) -> bool {
        id >= self.base
    }

    #[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
    fn build() -> Self {
        let mut b = Builder {
            states: Vec::new(),
            base: map_group_count(),
        };
        let pumpkin = b.add("minecraft:pumpkin", &[]);
        let snowy_podzol = b.add("minecraft:podzol", &[("snowy", "true")]);
        let snowy_grass = b.add("minecraft:grass_block", &[("snowy", "true")]);
        let snowy_mycelium = b.add("minecraft:mycelium", &[("snowy", "true")]);
        let upper = [("half", "upper")];
        let upper_sunflower = b.add("minecraft:sunflower", &upper);
        let upper_lilac = b.add("minecraft:lilac", &upper);
        let upper_tall_grass = b.add("minecraft:tall_grass", &upper);
        let upper_large_fern = b.add("minecraft:large_fern", &upper);
        let upper_rose_bush = b.add("minecraft:rose_bush", &upper);
        let upper_peony = b.add("minecraft:peony", &upper);

        let mut flower_pot = HashMap::new();
        for (key, name) in FLOWER_POTS {
            flower_pot.insert((*key).to_string(), b.add(name, &[]));
        }
        let mut skull = HashMap::new();
        for (index, name, kind) in [
            (0, "skeleton", "skull"),
            (1, "wither_skeleton", "skull"),
            (2, "zombie", "head"),
            (3, "player", "head"),
            (4, "creeper", "head"),
            (5, "dragon", "head"),
        ] {
            for facing in FACINGS {
                let id = b.add(
                    &format!("minecraft:{name}_wall_{kind}"),
                    &[("facing", facing)],
                );
                skull.insert(format!("{index}{facing}"), id);
            }
            for rotation in 0..16 {
                let id = b.add(
                    &format!("minecraft:{name}_{kind}"),
                    &[("rotation", &rotation.to_string())],
                );
                skull.insert(format!("{index}{rotation}"), id);
            }
        }
        let mut door = HashMap::new();
        for kind in [
            "oak_door",
            "iron_door",
            "spruce_door",
            "birch_door",
            "jungle_door",
            "acacia_door",
            "dark_oak_door",
        ] {
            let id = format!("minecraft:{kind}");
            for facing in FACINGS {
                for half in ["lower", "upper"] {
                    for hinge in ["left", "right"] {
                        for open in ["false", "true"] {
                            for powered in ["false", "true"] {
                                let state = b.add(
                                    &id,
                                    &[
                                        ("facing", facing),
                                        ("half", half),
                                        ("hinge", hinge),
                                        ("open", open),
                                        ("powered", powered),
                                    ],
                                );
                                door.insert(
                                    format!("{id}{facing}{half}{hinge}{open}{powered}"),
                                    state,
                                );
                            }
                        }
                    }
                }
            }
        }
        let mut note_block = HashMap::new();
        for note in 0..26 {
            let note_text = note.to_string();
            let powered = b.add(
                "minecraft:note_block",
                &[("powered", "true"), ("note", &note_text)],
            );
            note_block.insert(format!("true{note}"), powered);
            let unpowered = b.add(
                "minecraft:note_block",
                &[("powered", "false"), ("note", &note_text)],
            );
            note_block.insert(format!("false{note}"), unpowered);
        }
        let mut bed = HashMap::new();
        for (color_id, color) in DYE_COLORS.iter().enumerate() {
            if *color == "red" {
                continue;
            }
            add_beds(&mut b, &mut bed, color_id, color);
        }
        let mut banner = HashMap::new();
        for (color_id, color) in DYE_COLORS.iter().enumerate() {
            if *color == "white" {
                continue;
            }
            add_banners(&mut b, &mut banner, 15 - color_id, color);
        }
        let air = b.add("minecraft:air", &[]);
        MappingConstants {
            states: b.states,
            base: b.base,
            air,
            pumpkin,
            snowy_podzol,
            snowy_grass,
            snowy_mycelium,
            upper_sunflower,
            upper_lilac,
            upper_tall_grass,
            upper_large_fern,
            upper_rose_bush,
            upper_peony,
            flower_pot,
            skull,
            door,
            note_block,
            bed,
            banner,
        }
    }
}

/// `MappingConstants.addBeds`.
fn add_beds(b: &mut Builder, map: &mut HashMap<String, StateId>, color_id: usize, color: &str) {
    let name = format!("minecraft:{color}_bed");
    for facing in ["south", "west", "north", "east"] {
        let foot = b.add(
            &name,
            &[("facing", facing), ("occupied", "false"), ("part", "foot")],
        );
        map.insert(format!("{facing}falsefoot{color_id}"), foot);
        let head = b.add(
            &name,
            &[("facing", facing), ("occupied", "false"), ("part", "head")],
        );
        map.insert(format!("{facing}falsehead{color_id}"), head);
        let occupied = b.add(
            &name,
            &[("facing", facing), ("occupied", "true"), ("part", "head")],
        );
        map.insert(format!("{facing}truehead{color_id}"), occupied);
    }
}

/// `MappingConstants.addBanners`.
fn add_banners(b: &mut Builder, map: &mut HashMap<String, StateId>, color_id: usize, color: &str) {
    for rotation in 0..16 {
        let id = b.add(
            &format!("minecraft:{color}_banner"),
            &[("rotation", &rotation.to_string())],
        );
        map.insert(format!("{rotation}_{color_id}"), id);
    }
    for facing in ["north", "south", "west", "east"] {
        let id = b.add(
            &format!("minecraft:{color}_wall_banner"),
            &[("facing", facing)],
        );
        map.insert(format!("{facing}_{color_id}"), id);
    }
}

/// `MappingConstants.FLOWER_POT_MAP` (`"<item><data>"` to the potted block).
const FLOWER_POTS: &[(&str, &str)] = &[
    ("minecraft:air0", "minecraft:flower_pot"),
    ("minecraft:red_flower0", "minecraft:potted_poppy"),
    ("minecraft:red_flower1", "minecraft:potted_blue_orchid"),
    ("minecraft:red_flower2", "minecraft:potted_allium"),
    ("minecraft:red_flower3", "minecraft:potted_azure_bluet"),
    ("minecraft:red_flower4", "minecraft:potted_red_tulip"),
    ("minecraft:red_flower5", "minecraft:potted_orange_tulip"),
    ("minecraft:red_flower6", "minecraft:potted_white_tulip"),
    ("minecraft:red_flower7", "minecraft:potted_pink_tulip"),
    ("minecraft:red_flower8", "minecraft:potted_oxeye_daisy"),
    ("minecraft:yellow_flower0", "minecraft:potted_dandelion"),
    ("minecraft:sapling0", "minecraft:potted_oak_sapling"),
    ("minecraft:sapling1", "minecraft:potted_spruce_sapling"),
    ("minecraft:sapling2", "minecraft:potted_birch_sapling"),
    ("minecraft:sapling3", "minecraft:potted_jungle_sapling"),
    ("minecraft:sapling4", "minecraft:potted_acacia_sapling"),
    ("minecraft:sapling5", "minecraft:potted_dark_oak_sapling"),
    ("minecraft:red_mushroom0", "minecraft:potted_red_mushroom"),
    (
        "minecraft:brown_mushroom0",
        "minecraft:potted_brown_mushroom",
    ),
    ("minecraft:deadbush0", "minecraft:potted_dead_bush"),
    ("minecraft:tallgrass2", "minecraft:potted_fern"),
    ("minecraft:cactus0", "minecraft:potted_cactus"),
];
