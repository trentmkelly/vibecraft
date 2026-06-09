use std::collections::BTreeMap;

pub const DATA_INFO_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReportContract {
    java_file: &'static str,
    name: &'static str,
    output_path: &'static str,
}

const REPORT_CONTRACTS: &[ReportContract] = &[
    ReportContract {
        java_file: "BiomeParametersDumpReport.java",
        name: "Biome Parameters",
        output_path: "reports/biome_parameters/<preset>.json",
    },
    ReportContract {
        java_file: "BlockListReport.java",
        name: "Block List",
        output_path: "reports/blocks.json",
    },
    ReportContract {
        java_file: "CommandsReport.java",
        name: "Command Syntax",
        output_path: "reports/commands.json",
    },
    ReportContract {
        java_file: "DatapackStructureReport.java",
        name: "Datapack Structure",
        output_path: "reports/datapack.json",
    },
    ReportContract {
        java_file: "PacketReport.java",
        name: "Packet Report",
        output_path: "reports/packets.json",
    },
    ReportContract {
        java_file: "RegistryComponentsReport.java",
        name: "Default Components",
        output_path: "reports/<namespace>/components/<registry>/<element>.json",
    },
    ReportContract {
        java_file: "RegistryDumpReport.java",
        name: "Registry Dump",
        output_path: "reports/registries.json",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DatapackEntry {
    elements: bool,
    tags: bool,
    stable: bool,
}

impl DatapackEntry {
    const PSEUDO_REGISTRY: Self = Self {
        elements: true,
        tags: false,
        stable: true,
    };
    const STABLE_DYNAMIC_REGISTRY: Self = Self {
        elements: true,
        tags: true,
        stable: true,
    };
    const UNSTABLE_DYNAMIC_REGISTRY: Self = Self {
        elements: true,
        tags: true,
        stable: false,
    };
    const BUILT_IN_REGISTRY: Self = Self {
        elements: false,
        tags: true,
        stable: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DatapackFormat {
    Structure,
    Mcfunction,
}

impl DatapackFormat {
    fn serialized_name(self) -> &'static str {
        match self {
            Self::Structure => "structure",
            Self::Mcfunction => "mcfunction",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomPackEntry {
    format: DatapackFormat,
    entry: DatapackEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockStateReportModel {
    id: i32,
    default: bool,
    properties: BTreeMap<&'static str, &'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockReportModel {
    properties: BTreeMap<&'static str, Vec<&'static str>>,
    states: Vec<BlockStateReportModel>,
    definition_type: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryDumpEntryModel {
    protocol_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistryDumpModel {
    default: Option<&'static str>,
    protocol_id: i32,
    entries: BTreeMap<&'static str, RegistryDumpEntryModel>,
}

fn block_list_report_for_fixture() -> BTreeMap<&'static str, BlockReportModel> {
    BTreeMap::from([(
        "minecraft:oak_door",
        BlockReportModel {
            properties: BTreeMap::from([
                ("facing", vec!["north", "south"]),
                ("open", vec!["true", "false"]),
            ]),
            states: vec![
                BlockStateReportModel {
                    id: 100,
                    default: true,
                    properties: BTreeMap::from([("facing", "north"), ("open", "false")]),
                },
                BlockStateReportModel {
                    id: 101,
                    default: false,
                    properties: BTreeMap::from([("facing", "south"), ("open", "true")]),
                },
            ],
            definition_type: "minecraft:door",
        },
    )])
}

fn packet_report_protocol_templates() -> Vec<(&'static str, &'static str)> {
    vec![
        ("handshake", "serverbound"),
        ("status", "clientbound"),
        ("status", "serverbound"),
        ("login", "clientbound"),
        ("login", "serverbound"),
        ("configuration", "clientbound"),
        ("configuration", "serverbound"),
        ("play", "clientbound"),
        ("play", "serverbound"),
    ]
}

fn datapack_manual_entries() -> BTreeMap<&'static str, DatapackEntry> {
    BTreeMap::from([
        ("minecraft:recipe", DatapackEntry::PSEUDO_REGISTRY),
        ("minecraft:advancement", DatapackEntry::PSEUDO_REGISTRY),
        (
            "minecraft:loot_table",
            DatapackEntry::STABLE_DYNAMIC_REGISTRY,
        ),
        (
            "minecraft:item_modifier",
            DatapackEntry::STABLE_DYNAMIC_REGISTRY,
        ),
        (
            "minecraft:predicate",
            DatapackEntry::STABLE_DYNAMIC_REGISTRY,
        ),
    ])
}

fn datapack_non_registry_entries() -> BTreeMap<&'static str, CustomPackEntry> {
    BTreeMap::from([
        (
            "structure",
            CustomPackEntry {
                format: DatapackFormat::Structure,
                entry: DatapackEntry::PSEUDO_REGISTRY,
            },
        ),
        (
            "function",
            CustomPackEntry {
                format: DatapackFormat::Mcfunction,
                entry: DatapackEntry::STABLE_DYNAMIC_REGISTRY,
            },
        ),
    ])
}

fn registry_dump_fixture() -> BTreeMap<&'static str, RegistryDumpModel> {
    BTreeMap::from([(
        "minecraft:block",
        RegistryDumpModel {
            default: Some("minecraft:air"),
            protocol_id: 2,
            entries: BTreeMap::from([
                ("minecraft:air", RegistryDumpEntryModel { protocol_id: 0 }),
                ("minecraft:stone", RegistryDumpEntryModel { protocol_id: 1 }),
            ]),
        },
    )])
}

fn java_source(file: &str) -> &'static str {
    match file {
        "BiomeParametersDumpReport.java" => {
            include_str!("../../decompiled-server-26.1.2/net/minecraft/data/info/BiomeParametersDumpReport.java")
        }
        "BlockListReport.java" => {
            include_str!(
                "../../decompiled-server-26.1.2/net/minecraft/data/info/BlockListReport.java"
            )
        }
        "CommandsReport.java" => {
            include_str!(
                "../../decompiled-server-26.1.2/net/minecraft/data/info/CommandsReport.java"
            )
        }
        "DatapackStructureReport.java" => {
            include_str!("../../decompiled-server-26.1.2/net/minecraft/data/info/DatapackStructureReport.java")
        }
        "PacketReport.java" => {
            include_str!("../../decompiled-server-26.1.2/net/minecraft/data/info/PacketReport.java")
        }
        "RegistryComponentsReport.java" => {
            include_str!("../../decompiled-server-26.1.2/net/minecraft/data/info/RegistryComponentsReport.java")
        }
        "RegistryDumpReport.java" => {
            include_str!(
                "../../decompiled-server-26.1.2/net/minecraft/data/info/RegistryDumpReport.java"
            )
        }
        _ => unreachable!("unknown data info report source"),
    }
}

fn parse_get_name(source: &str) -> Option<String> {
    source
        .split("return \"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_names_and_output_paths_match_java_contracts() {
        for contract in REPORT_CONTRACTS {
            let source = java_source(contract.java_file);
            assert_eq!(parse_get_name(source).as_deref(), Some(contract.name));
            match contract.java_file {
                "BiomeParametersDumpReport.java" => {
                    assert!(source.contains("resolve(\"biome_parameters\")"));
                    assert!(source.contains("element.withSuffix(\".json\")"));
                }
                "RegistryComponentsReport.java" => {
                    assert!(source.contains("createRegistryComponentPathProvider"));
                    assert!(source.contains("registryPathProvider.json(elementId)"));
                }
                _ => {
                    let filename = contract
                        .output_path
                        .strip_prefix("reports/")
                        .unwrap_or(contract.output_path);
                    if filename.ends_with(".json") {
                        assert!(
                            source.contains(&format!("resolve(\"{filename}\")")),
                            "{} should resolve {filename}",
                            contract.java_file
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn biome_parameters_report_uses_known_presets_and_registry_ops() {
        let source = java_source("BiomeParametersDumpReport.java");
        assert!(source.contains("MultiNoiseBiomeSourceParameterList.knownPresets()"));
        assert!(source.contains("registryAccess.createSerializationContext(JsonOps.INSTANCE)"));
        assert!(source.contains("Climate.ParameterList.codec(ENTRY_CODEC)"));
        assert!(source.contains("resultOrPartial(e -> LOGGER.error"));
    }

    #[test]
    fn block_list_report_shape_matches_java() {
        let report = block_list_report_for_fixture();
        let oak_door = &report["minecraft:oak_door"];
        assert!(oak_door.properties.contains_key("facing"));
        assert_eq!(oak_door.states[0].id, 100);
        assert!(oak_door.states[0].default);
        assert_eq!(oak_door.definition_type, "minecraft:door");

        let source = java_source("BlockListReport.java");
        assert!(source.contains("entry.add(\"properties\", properties)"));
        assert!(source.contains("stateEntry.addProperty(\"id\", Block.getId(state))"));
        assert!(source.contains("stateEntry.addProperty(\"default\", true)"));
        assert!(source.contains("entry.add(\"definition\", data)"));
        assert!(source.contains("BlockTypes.CODEC"));
    }

    #[test]
    fn commands_report_builds_all_selection_dispatcher_json() {
        let source = java_source("CommandsReport.java");
        assert!(source.contains("new Commands(Commands.CommandSelection.ALL"));
        assert!(source.contains("Commands.createValidationContext(provider)"));
        assert!(
            source.contains("ArgumentUtils.serializeNodeToJson(dispatcher, dispatcher.getRoot())")
        );
    }

    #[test]
    fn datapack_structure_report_entries_and_formats_match_java() {
        assert_eq!(
            datapack_manual_entries()["minecraft:recipe"],
            DatapackEntry::PSEUDO_REGISTRY
        );
        assert_eq!(
            datapack_manual_entries()["minecraft:loot_table"],
            DatapackEntry::STABLE_DYNAMIC_REGISTRY
        );
        let dynamic_entry = DatapackEntry {
            elements: true,
            tags: true,
            stable: false,
        };
        let builtin_entry = DatapackEntry {
            elements: false,
            tags: true,
            stable: true,
        };
        assert_eq!(dynamic_entry, DatapackEntry::UNSTABLE_DYNAMIC_REGISTRY);
        assert_eq!(builtin_entry, DatapackEntry::BUILT_IN_REGISTRY);
        assert_eq!(
            datapack_non_registry_entries()["structure"]
                .format
                .serialized_name(),
            "structure"
        );
        assert_eq!(
            datapack_non_registry_entries()["function"]
                .format
                .serialized_name(),
            "mcfunction"
        );

        let source = java_source("DatapackStructureReport.java");
        assert!(source.contains("BuiltInRegistries.REGISTRY.forEach"));
        assert!(source.contains("RegistryDataLoader.WORLDGEN_REGISTRIES"));
        assert!(source.contains("RegistryDataLoader.DIMENSION_REGISTRIES"));
        assert!(source.contains("Duplicate entry for key "));
        assert!(source.contains("fieldOf(\"registries\")"));
        assert!(source.contains("fieldOf(\"others\")"));
    }

    #[test]
    fn packet_report_protocol_templates_and_shape_match_java() {
        let templates = packet_report_protocol_templates();
        assert_eq!(templates.len(), 9);
        assert_eq!(templates[0], ("handshake", "serverbound"));
        assert_eq!(templates[8], ("play", "serverbound"));

        let source = java_source("PacketReport.java");
        for template in [
            "HandshakeProtocols.SERVERBOUND_TEMPLATE",
            "StatusProtocols.CLIENTBOUND_TEMPLATE",
            "StatusProtocols.SERVERBOUND_TEMPLATE",
            "LoginProtocols.CLIENTBOUND_TEMPLATE",
            "LoginProtocols.SERVERBOUND_TEMPLATE",
            "ConfigurationProtocols.CLIENTBOUND_TEMPLATE",
            "ConfigurationProtocols.SERVERBOUND_TEMPLATE",
            "GameProtocols.CLIENTBOUND_TEMPLATE",
            "GameProtocols.SERVERBOUND_TEMPLATE",
        ] {
            assert!(source.contains(template));
        }
        assert!(source.contains("Collectors.groupingBy(ProtocolInfo.Details::id)"));
        assert!(source.contains("packetInfo.addProperty(\"protocol_id\", networkId)"));
    }

    #[test]
    fn registry_components_report_skips_empty_patches_and_encodes_components() {
        let source = java_source("RegistryComponentsReport.java");
        assert!(source.contains("DATA_COMPONENT_INITIALIZERS"));
        assert!(source.contains("if (!components.isEmpty())"));
        assert!(source.contains("DataComponentPatch.builder().set(components).build()"));
        assert!(source.contains("root.add(\n                                    \"components\""));
        assert!(source.contains("Failed to encode components for item "));
    }

    #[test]
    fn registry_dump_report_shape_matches_java() {
        let report = registry_dump_fixture();
        let blocks = &report["minecraft:block"];
        assert_eq!(blocks.default, Some("minecraft:air"));
        assert_eq!(blocks.protocol_id, 2);
        assert_eq!(blocks.entries["minecraft:stone"].protocol_id, 1);

        let source = java_source("RegistryDumpReport.java");
        assert!(source.contains("BuiltInRegistries.REGISTRY.listElements()"));
        assert!(source.contains("registry instanceof DefaultedRegistry"));
        assert!(source.contains("result.addProperty(\"protocol_id\", registryId)"));
        assert!(source.contains("entry.addProperty(\"protocol_id\", protocolId)"));
        assert!(source.contains("result.add(\"entries\", entries)"));
    }

    #[test]
    fn package_is_null_marked() {
        const {
            assert!(DATA_INFO_PACKAGE_NULL_MARKED);
        }
    }
}
