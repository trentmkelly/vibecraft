#![allow(dead_code)]

use std::collections::BTreeMap;

pub const VANILLA_BLOCK_REGISTRY_COUNT: usize = 1168;
pub const STATE_DEFINITION_NAME_PATTERN: &str = "^[a-z0-9_]+$";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRegistryEntry {
    pub numeric_id: usize,
    pub field_name: &'static str,
    pub registry_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Empty,
    FullCube,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockPhysicalProperties {
    pub map_color: &'static str,
    pub sound_type: &'static str,
    pub destroy_time: f32,
    pub explosion_resistance: f32,
    pub has_collision: bool,
    pub occludes: bool,
    pub light_emission: u8,
    pub pathfind_land: bool,
    pub pathfind_air: bool,
    pub can_survive_without_support: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPropertyDefinition {
    pub name: &'static str,
    pub values: &'static [&'static str],
    pub default_value: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStateDefinition {
    pub registry_id: &'static str,
    pub properties: &'static [BlockPropertyDefinition],
    pub physical: BlockPhysicalProperties,
    pub collision_shape: ShapeKind,
    pub occlusion_shape: ShapeKind,
}

pub(super) const fn block(
    numeric_id: usize,
    field_name: &'static str,
    registry_id: &'static str,
) -> BlockRegistryEntry {
    BlockRegistryEntry {
        numeric_id,
        field_name,
        registry_id,
    }
}

mod registry_data_a;
mod registry_data_b;
mod registry_data_c;

use std::sync::LazyLock;

pub static BLOCK_REGISTRY: LazyLock<&'static [BlockRegistryEntry]> = LazyLock::new(|| {
    let combined: Vec<BlockRegistryEntry> = registry_data_a::ENTRIES
        .iter()
        .chain(registry_data_b::ENTRIES.iter())
        .chain(registry_data_c::ENTRIES.iter())
        .copied()
        .collect();
    Box::leak(combined.into_boxed_slice())
});

const AXIS_VALUES: &[&str] = &["x", "y", "z"];
const BOOLEAN_VALUES: &[&str] = &["false", "true"];
const FACING_HORIZONTAL_VALUES: &[&str] = &["north", "south", "west", "east"];
const HALF_VALUES: &[&str] = &["top", "bottom"];
const STAIR_SHAPE_VALUES: &[&str] = &[
    "straight",
    "inner_left",
    "inner_right",
    "outer_left",
    "outer_right",
];
const CHEST_TYPE_VALUES: &[&str] = &["single", "left", "right"];
const AGE_0_7_VALUES: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7"];
const EMPTY_PROPERTIES: &[BlockPropertyDefinition] = &[];
const GRASS_BLOCK_PROPERTIES: &[BlockPropertyDefinition] = &[BlockPropertyDefinition {
    name: "snowy",
    values: BOOLEAN_VALUES,
    default_value: "false",
}];
const OAK_LOG_PROPERTIES: &[BlockPropertyDefinition] = &[BlockPropertyDefinition {
    name: "axis",
    values: AXIS_VALUES,
    default_value: "y",
}];
const OAK_STAIRS_PROPERTIES: &[BlockPropertyDefinition] = &[
    BlockPropertyDefinition {
        name: "facing",
        values: FACING_HORIZONTAL_VALUES,
        default_value: "north",
    },
    BlockPropertyDefinition {
        name: "half",
        values: HALF_VALUES,
        default_value: "bottom",
    },
    BlockPropertyDefinition {
        name: "shape",
        values: STAIR_SHAPE_VALUES,
        default_value: "straight",
    },
    BlockPropertyDefinition {
        name: "waterlogged",
        values: BOOLEAN_VALUES,
        default_value: "false",
    },
];
const CHEST_PROPERTIES: &[BlockPropertyDefinition] = &[
    BlockPropertyDefinition {
        name: "facing",
        values: FACING_HORIZONTAL_VALUES,
        default_value: "north",
    },
    BlockPropertyDefinition {
        name: "type",
        values: CHEST_TYPE_VALUES,
        default_value: "single",
    },
    BlockPropertyDefinition {
        name: "waterlogged",
        values: BOOLEAN_VALUES,
        default_value: "false",
    },
];
const WHEAT_PROPERTIES: &[BlockPropertyDefinition] = &[BlockPropertyDefinition {
    name: "age",
    values: AGE_0_7_VALUES,
    default_value: "0",
}];

const AIR_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "none",
    sound_type: "empty",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 0,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: true,
};
const STONE_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "stone",
    sound_type: "stone",
    destroy_time: 1.5,
    explosion_resistance: 6.0,
    has_collision: true,
    occludes: true,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const GRASS_BLOCK_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "grass",
    sound_type: "grass",
    destroy_time: 0.6,
    explosion_resistance: 0.6,
    has_collision: true,
    occludes: true,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const OAK_LOG_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "wood",
    sound_type: "wood",
    destroy_time: 2.0,
    explosion_resistance: 2.0,
    has_collision: true,
    occludes: true,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const OAK_STAIRS_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "wood",
    sound_type: "wood",
    destroy_time: 2.0,
    explosion_resistance: 3.0,
    has_collision: true,
    occludes: false,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const CHEST_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "wood",
    sound_type: "wood",
    destroy_time: 2.5,
    explosion_resistance: 2.5,
    has_collision: true,
    occludes: false,
    light_emission: 0,
    pathfind_land: false,
    pathfind_air: false,
    can_survive_without_support: true,
};
const WHEAT_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "plant",
    sound_type: "crop",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 0,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: false,
};
const TORCH_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "none",
    sound_type: "wood",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 14,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: false,
};
// All simple plants/flowers use .instabreak() in Java → destroyTime=0.0, explosionResistance=0.0,
// noCollision(), sound(SoundType.GRASS).
const INSTABREAK_PLANT_PHYSICAL: BlockPhysicalProperties = BlockPhysicalProperties {
    map_color: "plant",
    sound_type: "grass",
    destroy_time: 0.0,
    explosion_resistance: 0.0,
    has_collision: false,
    occludes: false,
    light_emission: 0,
    pathfind_land: true,
    pathfind_air: true,
    can_survive_without_support: false,
};

pub fn registry_entry_by_id(registry_id: &str) -> Option<&'static BlockRegistryEntry> {
    BLOCK_REGISTRY
        .iter()
        .find(|entry| entry.registry_id == registry_id)
}

pub fn representative_state_definition(registry_id: &str) -> Option<BlockStateDefinition> {
    Some(match registry_id {
        "minecraft:air" => BlockStateDefinition {
            registry_id: "minecraft:air",
            properties: EMPTY_PROPERTIES,
            physical: AIR_PHYSICAL,
            collision_shape: ShapeKind::Empty,
            occlusion_shape: ShapeKind::Empty,
        },
        "minecraft:stone" => BlockStateDefinition {
            registry_id: "minecraft:stone",
            properties: EMPTY_PROPERTIES,
            physical: STONE_PHYSICAL,
            collision_shape: ShapeKind::FullCube,
            occlusion_shape: ShapeKind::FullCube,
        },
        "minecraft:grass_block" => BlockStateDefinition {
            registry_id: "minecraft:grass_block",
            properties: GRASS_BLOCK_PROPERTIES,
            physical: GRASS_BLOCK_PHYSICAL,
            collision_shape: ShapeKind::FullCube,
            occlusion_shape: ShapeKind::FullCube,
        },
        "minecraft:oak_log" => BlockStateDefinition {
            registry_id: "minecraft:oak_log",
            properties: OAK_LOG_PROPERTIES,
            physical: OAK_LOG_PHYSICAL,
            collision_shape: ShapeKind::FullCube,
            occlusion_shape: ShapeKind::FullCube,
        },
        "minecraft:oak_stairs" => BlockStateDefinition {
            registry_id: "minecraft:oak_stairs",
            properties: OAK_STAIRS_PROPERTIES,
            physical: OAK_STAIRS_PHYSICAL,
            collision_shape: ShapeKind::Custom,
            occlusion_shape: ShapeKind::Custom,
        },
        "minecraft:chest" => BlockStateDefinition {
            registry_id: "minecraft:chest",
            properties: CHEST_PROPERTIES,
            physical: CHEST_PHYSICAL,
            collision_shape: ShapeKind::Custom,
            occlusion_shape: ShapeKind::Custom,
        },
        "minecraft:wheat" => BlockStateDefinition {
            registry_id: "minecraft:wheat",
            properties: WHEAT_PROPERTIES,
            physical: WHEAT_PHYSICAL,
            collision_shape: ShapeKind::Empty,
            occlusion_shape: ShapeKind::Empty,
        },
        "minecraft:torch" => BlockStateDefinition {
            registry_id: "minecraft:torch",
            properties: EMPTY_PROPERTIES,
            physical: TORCH_PHYSICAL,
            collision_shape: ShapeKind::Empty,
            occlusion_shape: ShapeKind::Empty,
        },
        // Instabreak plants — all use .instabreak() in Java (destroyTime=0.0, noCollision)
        "minecraft:short_grass"
        | "minecraft:fern"
        | "minecraft:dead_bush"
        | "minecraft:bush"
        | "minecraft:short_dry_grass"
        | "minecraft:dandelion"
        | "minecraft:golden_dandelion"
        | "minecraft:torchflower"
        | "minecraft:poppy"
        | "minecraft:blue_orchid"
        | "minecraft:allium"
        | "minecraft:azure_bluet"
        | "minecraft:red_tulip"
        | "minecraft:orange_tulip"
        | "minecraft:white_tulip"
        | "minecraft:pink_tulip"
        | "minecraft:oxeye_daisy"
        | "minecraft:cornflower"
        | "minecraft:wither_rose"
        | "minecraft:lily_of_the_valley"
        | "minecraft:brown_mushroom"
        | "minecraft:red_mushroom"
        | "minecraft:wildflowers"
        | "minecraft:firefly_bush"
        | "minecraft:tall_grass"
        | "minecraft:large_fern"
        | "minecraft:rose_bush"
        | "minecraft:peony"
        | "minecraft:lilac"
        | "minecraft:sunflower" => {
            // Use the static registry_id string so the lifetime requirement is satisfied
            let static_id = registry_entry_by_id(registry_id)?.registry_id;
            return Some(BlockStateDefinition {
                registry_id: static_id,
                properties: EMPTY_PROPERTIES,
                physical: INSTABREAK_PLANT_PHYSICAL,
                collision_shape: ShapeKind::Empty,
                occlusion_shape: ShapeKind::Empty,
            });
        }
        _ => return None,
    })
}

pub fn possible_state_count(definition: &BlockStateDefinition) -> usize {
    definition
        .properties
        .iter()
        .map(|property| property.values.len())
        .product::<usize>()
        .max(1)
}

pub fn is_valid_state_definition_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub fn validate_property_definition(
    owner: &str,
    property: &BlockPropertyDefinition,
    existing_names: &[&str],
) -> Result<(), String> {
    if !is_valid_state_definition_name(property.name) {
        return Err(format!(
            "{owner} has invalidly named property: {}",
            property.name
        ));
    }
    if property.values.len() <= 1 {
        return Err(format!(
            "{owner} attempted use property {} with <= 1 possible values",
            property.name
        ));
    }
    for value in property.values {
        if !is_valid_state_definition_name(value) {
            return Err(format!(
                "{owner} has property: {} with invalidly named value: {value}",
                property.name
            ));
        }
    }
    if existing_names.contains(&property.name) {
        return Err(format!("{owner} has duplicate property: {}", property.name));
    }
    Ok(())
}

impl BlockStateDefinition {
    pub fn get_owner(&self) -> &str {
        self.registry_id
    }

    pub fn get_properties(&self) -> Vec<&BlockPropertyDefinition> {
        let mut properties = self.properties.iter().collect::<Vec<_>>();
        properties.sort_by_key(|property| property.name);
        properties
    }

    pub fn get_property(&self, name: &str) -> Option<&BlockPropertyDefinition> {
        self.properties
            .iter()
            .find(|property| property.name == name)
    }

    pub fn is_singleton_state(&self) -> bool {
        self.properties.is_empty()
    }

    pub fn any_state(&self) -> BTreeMap<&'static str, &'static str> {
        default_state(self)
    }

    pub fn possible_states(&self) -> Vec<BTreeMap<&'static str, &'static str>> {
        let properties = self.get_properties();
        if properties.is_empty() {
            return vec![BTreeMap::new()];
        }

        let mut states = vec![BTreeMap::new()];
        for property in properties {
            let mut next_states = Vec::with_capacity(states.len() * property.values.len());
            for state in &states {
                for value in property.values {
                    let mut next = state.clone();
                    next.insert(property.name, *value);
                    next_states.push(next);
                }
            }
            states = next_states;
        }
        states
    }

    pub fn state_definition_string(&self) -> String {
        let properties = self
            .get_properties()
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "StateDefinition{{block={}, properties=[{properties}]}}",
            self.registry_id
        )
    }
}

pub fn default_state(definition: &BlockStateDefinition) -> BTreeMap<&'static str, &'static str> {
    definition
        .properties
        .iter()
        .map(|property| (property.name, property.default_value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        default_state, is_valid_state_definition_name, possible_state_count, registry_entry_by_id,
        representative_state_definition, validate_property_definition, BlockPropertyDefinition,
        BLOCK_REGISTRY, STATE_DEFINITION_NAME_PATTERN, VANILLA_BLOCK_REGISTRY_COUNT,
    };
    use crate::block_metadata::ShapeKind;
    use std::collections::BTreeSet;

    #[test]
    fn block_registry_order_matches_blocks_static_registration_surface() {
        assert_eq!(VANILLA_BLOCK_REGISTRY_COUNT, 1168);
        assert_eq!(BLOCK_REGISTRY.len(), VANILLA_BLOCK_REGISTRY_COUNT);
        assert_eq!(BLOCK_REGISTRY[0].registry_id, "minecraft:air");
        assert_eq!(BLOCK_REGISTRY[1].registry_id, "minecraft:stone");
        assert_eq!(BLOCK_REGISTRY[9].registry_id, "minecraft:dirt");
        assert_eq!(
            BLOCK_REGISTRY.last().unwrap().registry_id,
            "minecraft:firefly_bush"
        );
        assert!(BLOCK_REGISTRY
            .iter()
            .enumerate()
            .all(|(idx, entry)| entry.numeric_id == idx));
        let ids = BLOCK_REGISTRY
            .iter()
            .map(|entry| entry.registry_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), BLOCK_REGISTRY.len());
    }

    #[test]
    fn representative_state_counts_match_vanilla_property_products() {
        for (id, count) in [
            ("minecraft:stone", 1),
            ("minecraft:grass_block", 2),
            ("minecraft:oak_log", 3),
            ("minecraft:oak_stairs", 80),
            ("minecraft:chest", 24),
            ("minecraft:wheat", 8),
        ] {
            let definition =
                representative_state_definition(id).expect("known representative block");
            assert_eq!(possible_state_count(&definition), count, "{id}");
        }
    }

    #[test]
    fn default_states_expose_property_defaults() {
        let stairs = representative_state_definition("minecraft:oak_stairs").unwrap();
        let defaults = default_state(&stairs);
        assert_eq!(defaults["facing"], "north");
        assert_eq!(defaults["half"], "bottom");
        assert_eq!(defaults["shape"], "straight");
        assert_eq!(defaults["waterlogged"], "false");
    }

    #[test]
    fn state_definition_builder_validation_matches_java_name_rules() {
        assert_eq!(STATE_DEFINITION_NAME_PATTERN, "^[a-z0-9_]+$");
        assert!(is_valid_state_definition_name("waterlogged"));
        assert!(is_valid_state_definition_name("age_0"));
        assert!(!is_valid_state_definition_name(""));
        assert!(!is_valid_state_definition_name("Waterlogged"));
        assert!(!is_valid_state_definition_name("half-top"));

        let valid = BlockPropertyDefinition {
            name: "mode",
            values: &["low", "high"],
            default_value: "low",
        };
        assert!(validate_property_definition("minecraft:test", &valid, &[]).is_ok());
        assert!(validate_property_definition("minecraft:test", &valid, &["mode"]).is_err());
        assert!(validate_property_definition(
            "minecraft:test",
            &BlockPropertyDefinition {
                name: "Mode",
                values: &["low", "high"],
                default_value: "low",
            },
            &[],
        )
        .is_err());
        assert!(validate_property_definition(
            "minecraft:test",
            &BlockPropertyDefinition {
                name: "mode",
                values: &["only"],
                default_value: "only",
            },
            &[],
        )
        .is_err());
        assert!(validate_property_definition(
            "minecraft:test",
            &BlockPropertyDefinition {
                name: "mode",
                values: &["valid", "not-valid"],
                default_value: "valid",
            },
            &[],
        )
        .is_err());
    }

    #[test]
    fn state_definition_accessors_match_java_sorted_property_surface() {
        let stairs = representative_state_definition("minecraft:oak_stairs").unwrap();
        assert_eq!(stairs.get_owner(), "minecraft:oak_stairs");
        assert!(!stairs.is_singleton_state());
        assert_eq!(
            stairs
                .get_properties()
                .iter()
                .map(|property| property.name)
                .collect::<Vec<_>>(),
            vec!["facing", "half", "shape", "waterlogged"]
        );
        assert_eq!(
            stairs.get_property("shape").unwrap().values,
            &[
                "straight",
                "inner_left",
                "inner_right",
                "outer_left",
                "outer_right"
            ]
        );
        assert!(stairs.get_property("missing").is_none());
        assert_eq!(
            stairs.state_definition_string(),
            "StateDefinition{block=minecraft:oak_stairs, properties=[facing, half, shape, waterlogged]}"
        );

        let stone = representative_state_definition("minecraft:stone").unwrap();
        assert!(stone.is_singleton_state());
        assert!(stone.get_properties().is_empty());
        assert_eq!(stone.possible_states(), vec![Default::default()]);
    }

    #[test]
    fn state_definition_possible_states_match_java_cartesian_generation() {
        let chest = representative_state_definition("minecraft:chest").unwrap();
        let states = chest.possible_states();
        assert_eq!(states.len(), possible_state_count(&chest));
        assert_eq!(states.len(), 24);
        assert_eq!(states.first().unwrap()["facing"], "north");
        assert_eq!(states.first().unwrap()["type"], "single");
        assert_eq!(states.first().unwrap()["waterlogged"], "false");
        assert_eq!(states.last().unwrap()["facing"], "east");
        assert_eq!(states.last().unwrap()["type"], "right");
        assert_eq!(states.last().unwrap()["waterlogged"], "true");
        assert_eq!(chest.any_state(), default_state(&chest));
    }

    #[test]
    fn representative_properties_cover_physics_and_pathfinding_flags() {
        let air = representative_state_definition("minecraft:air").unwrap();
        assert!(!air.physical.has_collision);
        assert!(air.physical.pathfind_land);
        assert_eq!(air.collision_shape, ShapeKind::Empty);

        let stone = representative_state_definition("minecraft:stone").unwrap();
        assert_eq!(stone.physical.map_color, "stone");
        assert_eq!(stone.physical.destroy_time, 1.5);
        assert_eq!(stone.physical.explosion_resistance, 6.0);
        assert!(stone.physical.occludes);
        assert_eq!(stone.occlusion_shape, ShapeKind::FullCube);
        assert!(!stone.physical.pathfind_land);

        let torch = representative_state_definition("minecraft:torch").unwrap();
        assert_eq!(torch.physical.light_emission, 14);
        assert!(!torch.physical.can_survive_without_support);
    }

    #[test]
    fn registry_lookup_resolves_representative_ids() {
        assert_eq!(
            registry_entry_by_id("minecraft:oak_stairs")
                .unwrap()
                .field_name,
            "OAK_STAIRS"
        );
        assert_eq!(
            registry_entry_by_id("minecraft:chest").unwrap().field_name,
            "CHEST"
        );
        assert!(registry_entry_by_id("minecraft:not_a_block").is_none());
    }
}
