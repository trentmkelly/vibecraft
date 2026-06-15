#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{self, Read, Write};
#[cfg(test)]
use std::io::Cursor;

use crate::network::varint::{read_var_i32, write_var_i32};

pub const COMMANDS_PACKAGE_NULL_MARKED: bool = true;
pub const COMMAND_SYNCHRONIZATION_PACKAGE_NULL_MARKED: bool = true;
pub const COMMAND_SYNCHRONIZATION_BRIGADIER_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericArgumentKind {
    Float,
    Double,
    Integer,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericArgumentTemplate {
    Float { min: f32, max: f32 },
    Double { min: f64, max: f64 },
    Integer { min: i32, max: i32 },
    Long { min: i64, max: i64 },
}

impl NumericArgumentTemplate {
    pub fn kind(self) -> NumericArgumentKind {
        match self {
            Self::Float { .. } => NumericArgumentKind::Float,
            Self::Double { .. } => NumericArgumentKind::Double,
            Self::Integer { .. } => NumericArgumentKind::Integer,
            Self::Long { .. } => NumericArgumentKind::Long,
        }
    }

    pub fn serialize_network<W: Write>(self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[create_number_flags(self.has_min(), self.has_max())])?;
        match self {
            Self::Float { min, max } => {
                if self.has_min() {
                    writer.write_all(&min.to_be_bytes())?;
                }
                if self.has_max() {
                    writer.write_all(&max.to_be_bytes())?;
                }
            }
            Self::Double { min, max } => {
                if self.has_min() {
                    writer.write_all(&min.to_be_bytes())?;
                }
                if self.has_max() {
                    writer.write_all(&max.to_be_bytes())?;
                }
            }
            Self::Integer { min, max } => {
                if self.has_min() {
                    writer.write_all(&min.to_be_bytes())?;
                }
                if self.has_max() {
                    writer.write_all(&max.to_be_bytes())?;
                }
            }
            Self::Long { min, max } => {
                if self.has_min() {
                    writer.write_all(&min.to_be_bytes())?;
                }
                if self.has_max() {
                    writer.write_all(&max.to_be_bytes())?;
                }
            }
        }
        Ok(())
    }

    pub fn deserialize_network<R: Read>(
        kind: NumericArgumentKind,
        reader: &mut R,
    ) -> io::Result<Self> {
        let mut flags = [0u8; 1];
        reader.read_exact(&mut flags)?;
        match kind {
            NumericArgumentKind::Float => {
                let min = if number_has_min(flags[0]) {
                    read_f32(reader)?
                } else {
                    -f32::MAX
                };
                let max = if number_has_max(flags[0]) {
                    read_f32(reader)?
                } else {
                    f32::MAX
                };
                Ok(Self::Float { min, max })
            }
            NumericArgumentKind::Double => {
                let min = if number_has_min(flags[0]) {
                    read_f64(reader)?
                } else {
                    -f64::MAX
                };
                let max = if number_has_max(flags[0]) {
                    read_f64(reader)?
                } else {
                    f64::MAX
                };
                Ok(Self::Double { min, max })
            }
            NumericArgumentKind::Integer => {
                let min = if number_has_min(flags[0]) {
                    read_i32(reader)?
                } else {
                    i32::MIN
                };
                let max = if number_has_max(flags[0]) {
                    read_i32(reader)?
                } else {
                    i32::MAX
                };
                Ok(Self::Integer { min, max })
            }
            NumericArgumentKind::Long => {
                let min = if number_has_min(flags[0]) {
                    read_i64(reader)?
                } else {
                    i64::MIN
                };
                let max = if number_has_max(flags[0]) {
                    read_i64(reader)?
                } else {
                    i64::MAX
                };
                Ok(Self::Long { min, max })
            }
        }
    }

    pub fn json_properties(self) -> BTreeMap<&'static str, String> {
        let mut properties = BTreeMap::new();
        match self {
            Self::Float { min, max } => {
                if min != -f32::MAX {
                    properties.insert("min", min.to_string());
                }
                if max != f32::MAX {
                    properties.insert("max", max.to_string());
                }
            }
            Self::Double { min, max } => {
                if min != -f64::MAX {
                    properties.insert("min", min.to_string());
                }
                if max != f64::MAX {
                    properties.insert("max", max.to_string());
                }
            }
            Self::Integer { min, max } => {
                if min != i32::MIN {
                    properties.insert("min", min.to_string());
                }
                if max != i32::MAX {
                    properties.insert("max", max.to_string());
                }
            }
            Self::Long { min, max } => {
                if min != i64::MIN {
                    properties.insert("min", min.to_string());
                }
                if max != i64::MAX {
                    properties.insert("max", max.to_string());
                }
            }
        }
        properties
    }

    fn has_min(self) -> bool {
        match self {
            Self::Float { min, .. } => min != -f32::MAX,
            Self::Double { min, .. } => min != -f64::MAX,
            Self::Integer { min, .. } => min != i32::MIN,
            Self::Long { min, .. } => min != i64::MIN,
        }
    }

    fn has_max(self) -> bool {
        match self {
            Self::Float { max, .. } => max != f32::MAX,
            Self::Double { max, .. } => max != f64::MAX,
            Self::Integer { max, .. } => max != i32::MAX,
            Self::Long { max, .. } => max != i64::MAX,
        }
    }
}

pub fn create_number_flags(has_min: bool, has_max: bool) -> u8 {
    u8::from(has_min) | (u8::from(has_max) << 1)
}

pub fn number_has_min(flags: u8) -> bool {
    flags & 1 != 0
}

pub fn number_has_max(flags: u8) -> bool {
    flags & 2 != 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrigadierStringType {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

impl BrigadierStringType {
    pub fn network_ordinal(self) -> i32 {
        match self {
            Self::SingleWord => 0,
            Self::QuotablePhrase => 1,
            Self::GreedyPhrase => 2,
        }
    }

    pub fn from_network_ordinal(ordinal: i32) -> Result<Self, String> {
        match ordinal {
            0 => Ok(Self::SingleWord),
            1 => Ok(Self::QuotablePhrase),
            2 => Ok(Self::GreedyPhrase),
            _ => Err(format!("Invalid StringArgumentType ordinal {ordinal}")),
        }
    }

    pub fn json_type(self) -> &'static str {
        match self {
            Self::SingleWord => "word",
            Self::QuotablePhrase => "phrase",
            Self::GreedyPhrase => "greedy",
        }
    }

    pub fn serialize_network<W: Write>(self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.network_ordinal())
    }

    pub fn deserialize_network<R: Read>(reader: &mut R) -> io::Result<Self> {
        let ordinal = read_var_i32(reader)?;
        Self::from_network_ordinal(ordinal)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentInfoKind {
    Singleton { context_aware: bool },
    Numeric(NumericArgumentKind),
    String,
    Entity,
    ScoreHolder,
    Time,
    RegistryBacked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgumentTypeRegistration {
    pub id: &'static str,
    pub class_name: &'static str,
    pub info: ArgumentInfoKind,
}

pub fn argument_type_bootstrap_order() -> &'static [ArgumentTypeRegistration] {
    &ARGUMENT_TYPE_BOOTSTRAP_ORDER
}

pub fn argument_type_by_class(
    class_name: &str,
) -> Result<&'static ArgumentTypeRegistration, String> {
    ARGUMENT_TYPE_BOOTSTRAP_ORDER
        .iter()
        .find(|registration| registration.class_name == class_name)
        .ok_or_else(|| format!("Unrecognized argument type {class_name} (class {class_name})"))
}

pub fn singleton_network_payload() -> Vec<u8> {
    Vec::new()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionProviderRegistryModel {
    providers_by_name: HashMap<String, String>,
}

impl Default for SuggestionProviderRegistryModel {
    fn default() -> Self {
        let mut registry = Self {
            providers_by_name: HashMap::new(),
        };
        registry
            .providers_by_name
            .insert("minecraft:ask_server".to_string(), "ask_server".to_string());
        registry.providers_by_name.insert(
            "minecraft:available_sounds".to_string(),
            "available_sounds".to_string(),
        );
        registry.providers_by_name.insert(
            "minecraft:summonable_entities".to_string(),
            "summonable_entities".to_string(),
        );
        registry
    }
}

impl SuggestionProviderRegistryModel {
    pub fn register(&mut self, name: &str, provider: &str) -> Result<(), String> {
        if self.providers_by_name.contains_key(name) {
            Err(format!(
                "A command suggestion provider is already registered with the name '{name}'"
            ))
        } else {
            self.providers_by_name
                .insert(name.to_string(), provider.to_string());
            Ok(())
        }
    }

    pub fn get_provider(&self, name: &str) -> &str {
        self.providers_by_name
            .get(name)
            .map(String::as_str)
            .unwrap_or("ask_server")
    }

    pub fn get_name(&self, provider: &str) -> &str {
        self.providers_by_name
            .iter()
            .find_map(|(name, registered)| (registered == provider).then_some(name.as_str()))
            .unwrap_or("minecraft:ask_server")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandNodeSyncKind {
    Root,
    Literal,
    Argument {
        parser: &'static str,
        properties: BTreeMap<&'static str, String>,
    },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandNodeSyncModel {
    pub name: String,
    pub kind: CommandNodeSyncKind,
    pub children: Vec<usize>,
    pub executable: bool,
    pub permissions: Option<String>,
    pub redirect: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedCommandNodeJson {
    pub node_type: &'static str,
    pub parser: Option<&'static str>,
    pub properties: BTreeMap<&'static str, String>,
    pub children: BTreeMap<String, SerializedCommandNodeJson>,
    pub executable: bool,
    pub permissions: Option<String>,
    pub redirect: Vec<String>,
}

pub fn serialize_node_to_json(
    nodes: &[CommandNodeSyncModel],
    node_index: usize,
) -> SerializedCommandNodeJson {
    let node = &nodes[node_index];
    let (node_type, parser, properties) = match &node.kind {
        CommandNodeSyncKind::Root => ("root", None, BTreeMap::new()),
        CommandNodeSyncKind::Literal => ("literal", None, BTreeMap::new()),
        CommandNodeSyncKind::Argument { parser, properties } => {
            ("argument", Some(*parser), properties.clone())
        }
        CommandNodeSyncKind::Unknown => ("unknown", None, BTreeMap::new()),
    };
    let children = node
        .children
        .iter()
        .map(|child| {
            (
                nodes[*child].name.clone(),
                serialize_node_to_json(nodes, *child),
            )
        })
        .collect();
    let redirect = node
        .redirect
        .map(|target| command_node_path(nodes, target))
        .unwrap_or_default();
    SerializedCommandNodeJson {
        node_type,
        parser,
        properties,
        children,
        executable: node.executable,
        permissions: node.permissions.clone(),
        redirect,
    }
}

pub fn find_used_argument_types(
    nodes: &[CommandNodeSyncModel],
    root: usize,
) -> BTreeSet<&'static str> {
    let mut used = BTreeSet::new();
    let mut visited = HashSet::new();
    find_used_argument_types_inner(nodes, root, &mut used, &mut visited);
    used
}

fn find_used_argument_types_inner(
    nodes: &[CommandNodeSyncModel],
    index: usize,
    used: &mut BTreeSet<&'static str>,
    visited: &mut HashSet<usize>,
) {
    if !visited.insert(index) {
        return;
    }
    if let CommandNodeSyncKind::Argument { parser, .. } = nodes[index].kind {
        used.insert(parser);
    }
    for child in &nodes[index].children {
        find_used_argument_types_inner(nodes, *child, used, visited);
    }
    if let Some(redirect) = nodes[index].redirect {
        find_used_argument_types_inner(nodes, redirect, used, visited);
    }
}

fn command_node_path(nodes: &[CommandNodeSyncModel], target: usize) -> Vec<String> {
    fn search(
        nodes: &[CommandNodeSyncModel],
        current: usize,
        target: usize,
        path: &mut Vec<String>,
    ) -> bool {
        if current == target {
            return true;
        }
        for child in &nodes[current].children {
            path.push(nodes[*child].name.clone());
            if search(nodes, *child, target, path) {
                return true;
            }
            path.pop();
        }
        false
    }

    let mut path = Vec::new();
    if search(nodes, 0, target, &mut path) {
        path
    } else {
        Vec::new()
    }
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

fn read_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_be_bytes(bytes))
}

fn read_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

const ARGUMENT_TYPE_BOOTSTRAP_ORDER: [ArgumentTypeRegistration; 57] = [
    ArgumentTypeRegistration {
        id: "brigadier:bool",
        class_name: "BoolArgumentType",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "brigadier:float",
        class_name: "FloatArgumentType",
        info: ArgumentInfoKind::Numeric(NumericArgumentKind::Float),
    },
    ArgumentTypeRegistration {
        id: "brigadier:double",
        class_name: "DoubleArgumentType",
        info: ArgumentInfoKind::Numeric(NumericArgumentKind::Double),
    },
    ArgumentTypeRegistration {
        id: "brigadier:integer",
        class_name: "IntegerArgumentType",
        info: ArgumentInfoKind::Numeric(NumericArgumentKind::Integer),
    },
    ArgumentTypeRegistration {
        id: "brigadier:long",
        class_name: "LongArgumentType",
        info: ArgumentInfoKind::Numeric(NumericArgumentKind::Long),
    },
    ArgumentTypeRegistration {
        id: "brigadier:string",
        class_name: "StringArgumentType",
        info: ArgumentInfoKind::String,
    },
    ArgumentTypeRegistration {
        id: "entity",
        class_name: "EntityArgument",
        info: ArgumentInfoKind::Entity,
    },
    ArgumentTypeRegistration {
        id: "game_profile",
        class_name: "GameProfileArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "block_pos",
        class_name: "BlockPosArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "column_pos",
        class_name: "ColumnPosArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "vec3",
        class_name: "Vec3Argument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "vec2",
        class_name: "Vec2Argument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "block_state",
        class_name: "BlockStateArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "block_predicate",
        class_name: "BlockPredicateArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "item_stack",
        class_name: "ItemArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "item_predicate",
        class_name: "ItemPredicateArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "color",
        class_name: "ColorArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "hex_color",
        class_name: "HexColorArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "component",
        class_name: "ComponentArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "style",
        class_name: "StyleArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "message",
        class_name: "MessageArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "nbt_compound_tag",
        class_name: "CompoundTagArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "nbt_tag",
        class_name: "NbtTagArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "nbt_path",
        class_name: "NbtPathArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "objective",
        class_name: "ObjectiveArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "objective_criteria",
        class_name: "ObjectiveCriteriaArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "operation",
        class_name: "OperationArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "particle",
        class_name: "ParticleArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "angle",
        class_name: "AngleArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "rotation",
        class_name: "RotationArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "scoreboard_slot",
        class_name: "ScoreboardSlotArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "score_holder",
        class_name: "ScoreHolderArgument",
        info: ArgumentInfoKind::ScoreHolder,
    },
    ArgumentTypeRegistration {
        id: "swizzle",
        class_name: "SwizzleArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "team",
        class_name: "TeamArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "item_slot",
        class_name: "SlotArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "item_slots",
        class_name: "SlotsArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "resource_location",
        class_name: "IdentifierArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "function",
        class_name: "FunctionArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "entity_anchor",
        class_name: "EntityAnchorArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "int_range",
        class_name: "RangeArgument.Ints",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "float_range",
        class_name: "RangeArgument.Floats",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "dimension",
        class_name: "DimensionArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "gamemode",
        class_name: "GameModeArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "time",
        class_name: "TimeArgument",
        info: ArgumentInfoKind::Time,
    },
    ArgumentTypeRegistration {
        id: "resource_or_tag",
        class_name: "ResourceOrTagArgument",
        info: ArgumentInfoKind::RegistryBacked,
    },
    ArgumentTypeRegistration {
        id: "resource_or_tag_key",
        class_name: "ResourceOrTagKeyArgument",
        info: ArgumentInfoKind::RegistryBacked,
    },
    ArgumentTypeRegistration {
        id: "resource",
        class_name: "ResourceArgument",
        info: ArgumentInfoKind::RegistryBacked,
    },
    ArgumentTypeRegistration {
        id: "resource_key",
        class_name: "ResourceKeyArgument",
        info: ArgumentInfoKind::RegistryBacked,
    },
    ArgumentTypeRegistration {
        id: "resource_selector",
        class_name: "ResourceSelectorArgument",
        info: ArgumentInfoKind::RegistryBacked,
    },
    ArgumentTypeRegistration {
        id: "template_mirror",
        class_name: "TemplateMirrorArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "template_rotation",
        class_name: "TemplateRotationArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "heightmap",
        class_name: "HeightmapTypeArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
    ArgumentTypeRegistration {
        id: "loot_table",
        class_name: "ResourceOrIdArgument.LootTableArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "loot_predicate",
        class_name: "ResourceOrIdArgument.LootPredicateArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "loot_modifier",
        class_name: "ResourceOrIdArgument.LootModifierArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "dialog",
        class_name: "ResourceOrIdArgument.DialogArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: true,
        },
    },
    ArgumentTypeRegistration {
        id: "uuid",
        class_name: "UuidArgument",
        info: ArgumentInfoKind::Singleton {
            context_aware: false,
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_argument_info_serializes_flags_bounds_and_json_like_java() {
        assert_eq!(create_number_flags(false, false), 0);
        assert_eq!(create_number_flags(true, false), 1);
        assert_eq!(create_number_flags(false, true), 2);
        assert_eq!(create_number_flags(true, true), 3);
        assert!(number_has_min(3));
        assert!(number_has_max(3));

        let template = NumericArgumentTemplate::Integer { min: -5, max: 10 };
        assert_eq!(template.kind(), NumericArgumentKind::Integer);
        let mut bytes = Vec::new();
        template.serialize_network(&mut bytes).unwrap();
        assert_eq!(bytes, [3, 0xff, 0xff, 0xff, 0xfb, 0, 0, 0, 10]);
        assert_eq!(
            NumericArgumentTemplate::deserialize_network(
                NumericArgumentKind::Integer,
                &mut Cursor::new(bytes)
            )
            .unwrap(),
            template
        );
        assert_eq!(
            template.json_properties(),
            BTreeMap::from([("max", "10".to_string()), ("min", "-5".to_string())])
        );

        let defaults = NumericArgumentTemplate::Double {
            min: -f64::MAX,
            max: f64::MAX,
        };
        let mut default_bytes = Vec::new();
        defaults.serialize_network(&mut default_bytes).unwrap();
        assert_eq!(default_bytes, [0]);
        assert!(defaults.json_properties().is_empty());
    }

    #[test]
    fn string_argument_serializer_uses_java_enum_ordinals_and_json_names() {
        for (argument_type, ordinal, json) in [
            (BrigadierStringType::SingleWord, 0, "word"),
            (BrigadierStringType::QuotablePhrase, 1, "phrase"),
            (BrigadierStringType::GreedyPhrase, 2, "greedy"),
        ] {
            let mut bytes = Vec::new();
            argument_type.serialize_network(&mut bytes).unwrap();
            assert_eq!(bytes, [ordinal]);
            assert_eq!(
                BrigadierStringType::deserialize_network(&mut Cursor::new(bytes)).unwrap(),
                argument_type
            );
            assert_eq!(argument_type.json_type(), json);
        }
        assert!(BrigadierStringType::from_network_ordinal(3).is_err());
    }

    #[test]
    fn argument_type_infos_bootstrap_order_and_class_lookup_match_java() {
        const { assert!(COMMANDS_PACKAGE_NULL_MARKED) };
        const { assert!(COMMAND_SYNCHRONIZATION_PACKAGE_NULL_MARKED) };
        const { assert!(COMMAND_SYNCHRONIZATION_BRIGADIER_PACKAGE_NULL_MARKED) };
        let registrations = argument_type_bootstrap_order();
        assert_eq!(registrations[0].id, "brigadier:bool");
        assert_eq!(registrations[5].id, "brigadier:string");
        assert_eq!(registrations[31].id, "score_holder");
        assert_eq!(registrations[43].id, "time");
        assert_eq!(registrations[52].id, "loot_table");
        assert_eq!(registrations[56].id, "uuid");
        assert_eq!(
            argument_type_by_class("StringArgumentType").unwrap(),
            &registrations[5]
        );
        assert_eq!(
            argument_type_by_class("BlockStateArgument").unwrap().info,
            ArgumentInfoKind::Singleton {
                context_aware: true
            }
        );
        assert!(argument_type_by_class("UnknownArgument").is_err());
        assert!(singleton_network_payload().is_empty());
    }

    #[test]
    fn suggestion_providers_register_lookup_and_default_like_java() {
        let mut registry = SuggestionProviderRegistryModel::default();
        assert_eq!(
            registry.get_provider("minecraft:available_sounds"),
            "available_sounds"
        );
        assert_eq!(registry.get_provider("missing:provider"), "ask_server");
        assert_eq!(
            registry.get_name("summonable_entities"),
            "minecraft:summonable_entities"
        );
        assert_eq!(registry.get_name("unregistered"), "minecraft:ask_server");
        assert!(registry
            .register("minecraft:ask_server", "replacement")
            .unwrap_err()
            .contains("already registered"));
        registry
            .register("custom:provider", "custom_provider")
            .unwrap();
        assert_eq!(registry.get_provider("custom:provider"), "custom_provider");
    }

    #[test]
    fn argument_utils_serializes_nodes_and_finds_redirect_argument_types() {
        let nodes = vec![
            CommandNodeSyncModel {
                name: String::new(),
                kind: CommandNodeSyncKind::Root,
                children: vec![1, 3],
                executable: false,
                permissions: None,
                redirect: None,
            },
            CommandNodeSyncModel {
                name: "literal".to_string(),
                kind: CommandNodeSyncKind::Literal,
                children: vec![2],
                executable: true,
                permissions: Some("op_level_2".to_string()),
                redirect: None,
            },
            CommandNodeSyncModel {
                name: "count".to_string(),
                kind: CommandNodeSyncKind::Argument {
                    parser: "brigadier:integer",
                    properties: NumericArgumentTemplate::Integer { min: 1, max: 64 }
                        .json_properties(),
                },
                children: Vec::new(),
                executable: true,
                permissions: None,
                redirect: None,
            },
            CommandNodeSyncModel {
                name: "alias".to_string(),
                kind: CommandNodeSyncKind::Argument {
                    parser: "brigadier:string",
                    properties: BTreeMap::from([("type", "word".to_string())]),
                },
                children: Vec::new(),
                executable: false,
                permissions: None,
                redirect: Some(2),
            },
            CommandNodeSyncModel {
                name: "unknown".to_string(),
                kind: CommandNodeSyncKind::Unknown,
                children: Vec::new(),
                executable: false,
                permissions: None,
                redirect: None,
            },
        ];

        let serialized = serialize_node_to_json(&nodes, 1);
        assert_eq!(serialized.node_type, "literal");
        assert!(serialized.executable);
        assert_eq!(serialized.permissions, Some("op_level_2".to_string()));
        assert_eq!(
            serialized.children["count"].properties,
            BTreeMap::from([("max", "64".to_string()), ("min", "1".to_string())])
        );

        let alias = serialize_node_to_json(&nodes, 3);
        assert_eq!(
            alias.redirect,
            vec!["literal".to_string(), "count".to_string()]
        );
        assert_eq!(serialize_node_to_json(&nodes, 4).node_type, "unknown");
        assert_eq!(
            find_used_argument_types(&nodes, 3),
            BTreeSet::from(["brigadier:integer", "brigadier:string"])
        );
    }
}
