//! Tests for the Brigadier command graph, `ClientboundCommandsPacket` output and
//! server-side suggestions.

use std::io::Cursor;

use serde_json::Value;

use super::suggest::command_suggestions;
use super::{CommandGraph, NodeKind, ROOT};
use crate::network::play::{ClientboundCommandsPacket, CommandNodeStubData};

/// `--reports` `commands.json` of the official 26.1.2 server (path-expanded).
const REPORT: &str = include_str!("../../vanilla-data/reports/commands_26_1_2.json");

/// `ClientboundCommandsPacket` bytes captured from the official 26.1.2 server
/// code for a source at each `PermissionLevel` (0 = all .. 4 = owners).
const OFFICIAL_PACKETS: [&[u8]; 5] = [
    include_bytes!("../../vanilla-data/reports/commands_packet_level0_26_1_2.bin"),
    include_bytes!("../../vanilla-data/reports/commands_packet_level1_26_1_2.bin"),
    include_bytes!("../../vanilla-data/reports/commands_packet_level2_26_1_2.bin"),
    include_bytes!("../../vanilla-data/reports/commands_packet_level3_26_1_2.bin"),
    include_bytes!("../../vanilla-data/reports/commands_packet_level4_26_1_2.bin"),
];

fn permission_level_of(report_node: &Value) -> u8 {
    match report_node["permissions"]["permission"]["level"].as_str() {
        None => 0,
        Some("moderators") => 1,
        Some("gamemasters") => 2,
        Some("admins") => 3,
        Some("owners") => 4,
        Some(other) => panic!("unknown level {other}"),
    }
}

fn child_named(graph: &CommandGraph, parent: usize, name: &str) -> Option<usize> {
    graph
        .node(parent)
        .children
        .iter()
        .copied()
        .find(|c| graph.node(*c).kind.name() == name)
}

/// Compares one graph node (and its subtree) with the report node at `path`.
fn compare_with_report(graph: &CommandGraph, id: usize, report: &Value, path: &str) {
    let node = graph.node(id);
    let (kind, parser) = match &node.kind {
        NodeKind::Root => ("root", None),
        NodeKind::Literal { .. } => ("literal", None),
        NodeKind::Argument { parser, .. } => ("argument", Some(parser.as_str())),
    };
    let report_kind = report["type"].as_str().unwrap();
    assert_eq!(
        report_kind,
        if kind == "argument" { "argument" } else { kind },
        "{path}"
    );
    assert_eq!(report["parser"].as_str(), parser, "{path} parser");
    if let NodeKind::Argument { properties, .. } = &node.kind {
        let expected = report.get("properties").cloned().unwrap_or(Value::Null);
        assert_eq!(*properties, expected, "{path} properties");
    }
    assert_eq!(
        report["executable"].as_bool().unwrap_or(false),
        node.executable,
        "{path} executable"
    );
    assert_eq!(
        permission_level_of(report),
        node.permission_level,
        "{path} permission"
    );

    // Redirects: the report gives the target's path from the root and omits
    // redirects to the root itself (`CommandDispatcher.getPath` is empty).
    let report_redirect = report["redirect"].as_array().map(|steps| {
        steps.iter().fold(ROOT, |at, step| {
            child_named(graph, at, step.as_str().unwrap())
                .unwrap_or_else(|| panic!("{path}: redirect step {step} missing"))
        })
    });
    assert_eq!(
        report_redirect.unwrap_or(ROOT),
        node.redirect.unwrap_or(ROOT),
        "{path} redirect"
    );
    assert_eq!(
        report_redirect.is_some(),
        node.redirect.is_some_and(|r| r != ROOT),
        "{path} redirect presence"
    );

    let empty = serde_json::Map::new();
    let report_children = report["children"].as_object().unwrap_or(&empty);
    for &child in &node.children {
        let name = graph.node(child).kind.name();
        let child_report = report_children
            .get(name)
            .unwrap_or_else(|| panic!("{path}/{name} not in the official report"));
        compare_with_report(graph, child, child_report, &format!("{path}/{name}"));
    }
    for name in report_children.keys() {
        if child_named(graph, id, name).is_none() {
            // The report is generated with `CommandSelection.ALL`; the dedicated
            // server has no `/publish` (integrated-server only).
            assert_eq!(
                format!("{path}/{name}"),
                "/publish",
                "report node missing from graph"
            );
        }
    }
}

#[test]
fn vanilla_graph_matches_official_commands_report_node_for_node() {
    let report: Value = serde_json::from_str(REPORT).unwrap();
    compare_with_report(CommandGraph::vanilla(), ROOT, &report, "");
}

#[test]
fn packets_match_official_bytes_for_every_permission_level() {
    for (level, expected) in OFFICIAL_PACKETS.iter().enumerate() {
        let packet = CommandGraph::vanilla()
            .commands_packet_for_permission_level(level as u8)
            .unwrap();
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(bytes.len(), expected.len(), "level {level} length");
        assert!(
            bytes == *expected,
            "level {level} packet bytes differ from the official server"
        );
        // And the official bytes decode + validate with our reader.
        let decoded = ClientboundCommandsPacket::read(&mut Cursor::new(expected.to_vec())).unwrap();
        assert_eq!(decoded, packet, "level {level}");
    }
}

#[test]
fn restricted_flag_and_permissions_filter_the_tree() {
    let low = CommandGraph::vanilla()
        .commands_packet_for_permission_level(0)
        .unwrap();
    let owner = CommandGraph::vanilla()
        .commands_packet_for_permission_level(4)
        .unwrap();
    assert!(owner.entries.len() > low.entries.len());
    let names = |p: &ClientboundCommandsPacket| -> Vec<String> {
        p.entries[p.root_index as usize]
            .children
            .iter()
            .filter_map(|c| match &p.entries[*c as usize].stub {
                CommandNodeStubData::Literal { name } => Some(name.clone()),
                _ => None,
            })
            .collect()
    };
    assert!(!names(&low).contains(&"stop".to_string()));
    assert!(names(&owner).contains(&"stop".to_string()));
    assert!(owner.entries.iter().skip(1).any(|e| e.restricted));
    assert!(low.entries.iter().all(|e| !e.restricted));
}

#[test]
fn served_graph_adds_only_the_biome_debug_command() {
    let served = CommandGraph::served();
    let vanilla = CommandGraph::vanilla();
    assert_eq!(served.node_count(), vanilla.node_count() + 1);
    assert!(child_named(served, ROOT, "biome").is_some());
    assert!(child_named(vanilla, ROOT, "biome").is_none());
}

fn suggest(level: u8, input: &str) -> (usize, usize, Vec<String>) {
    let s = command_suggestions(CommandGraph::vanilla(), level, input);
    (s.start, s.end, s.texts)
}

#[test]
fn suggestions_complete_root_literals_with_replace_range() {
    let (start, end, texts) = suggest(4, "/gam");
    assert_eq!((start, end), (1, 4));
    assert_eq!(texts, ["gamemode", "gamerule"]);
    // Without the slash the range starts at 0.
    assert_eq!(suggest(4, "gam").0, 0);
}

#[test]
fn suggestions_walk_subcommands_in_sorted_order() {
    let (start, _, texts) = suggest(4, "/time ");
    assert_eq!(start, 6);
    assert_eq!(
        texts,
        ["add", "of", "pause", "query", "rate", "resume", "set"]
    );
    let (_, _, texts) = suggest(4, "/gamemode s");
    assert_eq!(texts, ["spectator", "survival"]);
}

#[test]
fn suggestions_offer_time_units_after_a_number() {
    let (start, end, texts) = suggest(4, "/time add 1");
    assert_eq!((start, end), (11, 11));
    assert_eq!(texts, ["d", "s", "t"]);
}

#[test]
fn suggestions_offer_coordinates_for_position_arguments() {
    assert_eq!(suggest(4, "/setworldspawn ").2, ["~", "~ ~", "~ ~ ~"]);
    assert_eq!(suggest(4, "/setworldspawn ~ ").2, ["~ ~", "~ ~ ~"]);
}

#[test]
fn suggestions_skip_an_exact_match_but_keep_longer_literals() {
    assert_eq!(suggest(4, "/stop").2, ["stopsound", "stopwatch"]);
}

#[test]
fn sound_providers_complete_both_commands_with_resource_matching() {
    for input in [
        "/playsound ambient.cave",
        "/stopsound @s master ambient.cave",
    ] {
        let s = command_suggestions(CommandGraph::vanilla(), 4, input);
        assert_eq!(s.texts, ["minecraft:ambient.cave"]);
        assert_eq!(s.tooltips, [None]);
        assert_eq!(s.start, input.rfind(' ').unwrap() + 1);
        assert_eq!(s.end, input.len());
    }
    assert_eq!(
        suggest(4, "/playsound minecraft:ambient.cave").2,
        Vec::<String>::new()
    );
    assert!(suggest(4, "/playsound craft:ambient").2.is_empty());
}

#[test]
fn summon_provider_filters_unsummonable_entities_and_preserves_tooltips() {
    for input in ["/summon pig", "/execute summon pig"] {
        let s = command_suggestions(CommandGraph::vanilla(), 4, input);
        assert!(s.texts.contains(&"minecraft:pig".to_owned()));
        assert_eq!(s.tooltips.len(), s.texts.len());
        for (text, tooltip) in s.texts.iter().zip(&s.tooltips) {
            assert_eq!(
                tooltip.as_deref(),
                Some(format!("entity.{}", text.replace(':', ".")).as_str())
            );
        }
    }
    assert!(suggest(4, "/summon player").2.is_empty());
    assert!(suggest(4, "/summon fishing_bobber").2.is_empty());
}

#[test]
fn anonymous_tick_providers_are_resolved_from_the_effective_command_path() {
    assert_eq!(suggest(4, "/tick rate ").2, ["20"]);
    assert_eq!(suggest(4, "/tick step 1").2, ["1s", "1t"]);
    assert_eq!(suggest(4, "/tick sprint ").2, ["1d", "3d", "60s", "stop"]);
    assert_eq!(suggest(4, "/execute run tick rate ").2, ["20"]);
    assert!(suggest(4, "/bossbar remove ").2.is_empty());
}

#[test]
fn provider_truncation_keeps_tooltips_aligned() {
    let sounds = command_suggestions(CommandGraph::vanilla(), 4, "/playsound ");
    assert_eq!(sounds.texts.len(), super::suggest::MAX_SUGGESTIONS);
    assert_eq!(sounds.tooltips.len(), sounds.texts.len());
    assert!(sounds.tooltips.iter().all(Option::is_none));
}

#[test]
fn served_graph_appends_the_biome_debug_command_last() {
    let served = CommandGraph::served();
    let last = served.node(ROOT).children.last().copied().unwrap();
    assert_eq!(served.node(last).kind.name(), "biome");
}

/// Suggestions recorded by running the official 26.1.2 `CommandDispatcher` with
/// a level-`n` source (`level`, `input`, resulting range and sorted texts).
const OFFICIAL_SUGGESTIONS: &str =
    include_str!("../../vanilla-data/reports/command_suggestions_26_1_2.json");

#[test]
fn suggestions_match_official_dispatcher_recordings() {
    let cases: Vec<Value> = serde_json::from_str(OFFICIAL_SUGGESTIONS).unwrap();
    let mut mismatches = Vec::new();
    for case in &cases {
        let input = case["input"].as_str().unwrap();
        let level = case["level"].as_u64().unwrap() as u8;
        let s = command_suggestions(CommandGraph::vanilla(), level, input);
        let texts: Vec<&str> = case["texts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap())
            .collect();
        let want = (
            case["start"].as_u64().unwrap() as usize,
            case["end"].as_u64().unwrap() as usize,
        );
        if s.texts != texts || (!texts.is_empty() && (s.start, s.end) != want) {
            mismatches.push(format!(
                "L{level} {input:?}: java {want:?} {texts:?} rust ({},{}) {:?}",
                s.start, s.end, s.texts
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} of {} differ:\n{}",
        mismatches.len(),
        cases.len(),
        mismatches.join("\n")
    );
}

/// A small graph whose parser must succeed before Brigadier can reach `next`.
/// The supplementary character in the leading literal makes the replacement
/// range assert Java's UTF-16 indexing instead of Rust's UTF-8 byte offsets.
fn parser_completion_graph(parser: &str, properties: Value) -> CommandGraph {
    let mut graph = CommandGraph::new();
    let head = graph.push(super::CommandGraphNode {
        kind: NodeKind::Literal {
            name: "😀".into()
        },
        executable: false,
        command_key: None,
        fork: false,
        permission_level: 0,
        redirect: None,
        children: Vec::new(),
    });
    let argument = graph.push(super::CommandGraphNode {
        kind: NodeKind::Argument {
            name: "value".into(),
            parser: parser.into(),
            properties,
            suggestions: None,
            type_key: 1,
        },
        executable: false,
        command_key: None,
        fork: false,
        permission_level: 0,
        redirect: None,
        children: Vec::new(),
    });
    let next = graph.push(super::CommandGraphNode {
        kind: NodeKind::Literal {
            name: "next".into(),
        },
        executable: true,
        command_key: Some(1),
        fork: false,
        permission_level: 0,
        redirect: None,
        children: Vec::new(),
    });
    graph.add_child(ROOT, head);
    graph.add_child(head, argument);
    graph.add_child(argument, next);
    graph
}

#[test]
fn stateless_minecraft_parsers_gate_following_literal_completion() {
    // These cases follow RangeArgument -> MinMaxBounds.Bounds.fromReader,
    // ColorArgument.parse, SwizzleArgument.parse, and TimeArgument.parse in
    // the official 26.1.2 decompilation.
    let cases: [(&str, Value, &[&str], &[&str]); 5] = [
        (
            "minecraft:int_range",
            Value::Null,
            &["1..2", "..5", "5.."],
            &["2..1", "..", "1.2", "2147483648", "1x"],
        ),
        (
            "minecraft:float_range",
            Value::Null,
            &["1.2..2.5", "..2.5", "2.5..", "-0.0..0.0", "0.0..0.0"],
            &["2.5..1.2", "..", "1.2.3", "0.0..-0.0", "+0.0", "1.2x"],
        ),
        (
            "minecraft:color",
            Value::Null,
            &["dark_blue", "DARK_BLUE", "d-a-r-k_b-l-u-e", "reset"],
            &["not_a_color", "bold"],
        ),
        (
            "minecraft:swizzle",
            Value::Null,
            &["xyz", "xz", "y"],
            &["xx", "X", "xY"],
        ),
        (
            "minecraft:time",
            serde_json::json!({"min": 0}),
            &["3s", "-0.5"],
            &["3w", "-0.51"],
        ),
    ];

    for (parser, properties, valid_cases, invalid_cases) in cases {
        let graph = parser_completion_graph(parser, properties);
        for valid in valid_cases {
            let valid_input = format!("😀 {valid} n");
            let valid_suggestions = command_suggestions(&graph, 0, &valid_input);
            assert_eq!(
                valid_suggestions.texts,
                ["next"],
                "{parser} valid {valid:?}"
            );
            assert_eq!(
                (valid_suggestions.start, valid_suggestions.end),
                (
                    valid_input.encode_utf16().count() - 1,
                    valid_input.encode_utf16().count()
                ),
                "{parser} replacement range for {valid:?}"
            );
        }

        for invalid in invalid_cases {
            let invalid_input = format!("😀 {invalid} n");
            let invalid_suggestions = command_suggestions(&graph, 0, &invalid_input);
            assert!(
                !invalid_suggestions.texts.iter().any(|text| text == "next"),
                "{parser} invalid {invalid:?} must not traverse to the following literal"
            );
        }
    }

    // Math.round(float) saturates at Integer.MAX_VALUE for large values, which
    // remains valid when the TimeArgument minimum is that same boundary.
    let large_time =
        parser_completion_graph("minecraft:time", serde_json::json!({"min": i32::MAX}));
    assert_eq!(
        command_suggestions(&large_time, 0, "😀 999999999999999999999999999999999999d n").texts,
        ["next"]
    );

    // At a one-tick minimum, 0.04s rounds up to one tick while 0.02s rounds
    // down to zero, matching Java Math.round(float).
    let one_tick_minimum = parser_completion_graph("minecraft:time", serde_json::json!({"min": 1}));
    assert_eq!(
        command_suggestions(&one_tick_minimum, 0, "😀 0.04s n").texts,
        ["next"]
    );
    assert!(
        command_suggestions(&one_tick_minimum, 0, "😀 0.02s n")
            .texts
            .is_empty()
    );
}

#[test]
fn vanilla_color_argument_suggests_chat_formatting_colors_and_reset() {
    // TeamCommand registers this argument with ColorArgument.color(); that
    // class delegates to ChatFormatting.getNames(true, false). SharedSuggestionProvider
    // also matches the word after an underscore, so `re` includes `dark_red`.
    assert_eq!(
        suggest(4, "/team modify sample color re").2,
        ["dark_red", "red", "reset"]
    );
}

#[test]
fn stateless_minecraft_parsers_preserve_java_reader_cursor_semantics() {
    use super::suggest::argument::{StringReader, parse};

    let parse_at_zero = |parser: &str, text: &str, properties: Value| {
        let input: Vec<u16> = text.encode_utf16().collect();
        let mut reader = StringReader::new(&input, 0);
        let success = parse(parser, &properties, &mut reader);
        (success, reader.cursor())
    };

    // Bounds.fromReader resets for malformed numbers; the Ints/Doubles
    // wrappers reset again when areSwapped() rejects a reversed range.
    // Successful parse stops before Brigadier's separating space.
    assert_eq!(
        parse_at_zero("minecraft:int_range", "1..2 tail", Value::Null),
        (true, 4)
    );
    assert_eq!(
        parse_at_zero("minecraft:int_range", "2..1", Value::Null),
        (false, 0)
    );
    assert_eq!(
        parse_at_zero("minecraft:int_range", "..", Value::Null),
        (false, 0)
    );
    assert_eq!(
        parse_at_zero("minecraft:int_range", "2147483648", Value::Null),
        (false, 0)
    );
    assert_eq!(
        parse_at_zero("minecraft:float_range", "2.5..1.2", Value::Null),
        (false, 0)
    );
    assert_eq!(
        parse_at_zero("minecraft:float_range", "0.0..-0.0", Value::Null),
        (false, 0)
    );
    assert_eq!(
        parse_at_zero("minecraft:float_range", "+0.0", Value::Null),
        (false, 0)
    );
    assert_eq!(
        parse_at_zero("minecraft:float_range", "1.2.3", Value::Null),
        (false, 0)
    );

    // ColorArgument and SwizzleArgument throw after consuming the bad token
    // or offending axis; the command dispatcher restores the argument start.
    assert_eq!(
        parse_at_zero("minecraft:color", "not_a_color!", Value::Null),
        (false, "not_a_color".encode_utf16().count())
    );
    assert_eq!(
        parse_at_zero("minecraft:swizzle", "xq", Value::Null),
        (false, 2)
    );

    // TimeArgument also retains its consumed cursor when the unit is invalid
    // or the computed tick count violates the argument's configured minimum.
    assert_eq!(
        parse_at_zero("minecraft:time", "3w", serde_json::json!({"min": 0})),
        (false, 2)
    );
    assert_eq!(
        parse_at_zero("minecraft:time", "3s", serde_json::json!({"min": 100})),
        (false, 2)
    );

    let malformed: Vec<u16> = "x".encode_utf16().collect();
    let mut reader = StringReader::new(&malformed, 0);
    assert_eq!(reader.read_float(), None);
    assert_eq!(
        reader.cursor(),
        0,
        "StringReader.readFloat resets on syntax error"
    );
}

/// Exercise the actual completion traversal, not only the argument helpers:
/// invalid syntax must keep the following literal inaccessible, and successful
/// arguments must retain Java's UTF-16 replacement ranges.
#[test]
fn minecraft_coordinate_parsers_gate_following_literal_completion() {
    let cases: &[(&str, &[&str], &[&str])] = &[
        (
            "minecraft:vec3",
            &["1 2 3", "~ ~1.5 ~", "^ ^2 ^-3"],
            &["x y z", "~ ~ ^", "1 2", "^ ^ 1"],
        ),
        (
            "minecraft:block_pos",
            &["1 2 3", "~1.5 ~ ~", "^ ^ ^"],
            &["1.5 2 3", "2147483648 2 3", "~ ~ ^"],
        ),
        (
            "minecraft:vec2",
            &["1.5 -2", "~ ~3.5"],
            &["^ ^", "1", "x y"],
        ),
        (
            "minecraft:column_pos",
            &["1 -2", "~1.5 ~"],
            &["1.5 2", "^ ^", "1"],
        ),
        (
            "minecraft:rotation",
            &["1.5 -2", "~ ~3.5"],
            &["^ ^", "1", "x y"],
        ),
    ];
    assert_parsers_gate_following_literal(cases);
}

#[test]
fn minecraft_simple_parsers_gate_following_literal_completion() {
    let cases: &[(&str, &[&str], &[&str])] = &[
        (
            "minecraft:resource_location",
            &["minecraft:foo/bar", ":foo", "foo"],
            &["..:foo", "foo::bar", "Foo"],
        ),
        (
            "minecraft:resource_key",
            &["minecraft:foo/bar", ":foo"],
            &["..:foo", "foo::bar", "Foo"],
        ),
        (
            "minecraft:resource_or_tag_key",
            &["minecraft:foo", "#custom:foo"],
            &["#..:foo", "foo::bar", "Foo"],
        ),
        (
            "minecraft:heightmap",
            &["world_surface", "motion_blocking_no_leaves"],
            &["WORLD_SURFACE", "ocean_floor_wg", "unknown"],
        ),
        (
            "minecraft:function",
            &["minecraft:foo", "#custom:foo"],
            &["#..:foo", "foo::bar", "Foo"],
        ),
        (
            "minecraft:uuid",
            &["dd12be42-52a9-4a91-a8a1-11c01849e498", "1-2-3-4-5"],
            &["not_uuid", "1-2-3-4", "1-2-3-4-5-6"],
        ),
        (
            "minecraft:angle",
            &["0", "~-5.5", "~"],
            &[
                "^",
                "angle",
                "99999999999999999999999999999999999999999999999",
            ],
        ),
        (
            "minecraft:gamemode",
            &["survival", "spectator"],
            &["Survival", "1", "not_a_mode"],
        ),
        (
            "minecraft:entity_anchor",
            &["feet", "eyes"],
            &["Feet", "head"],
        ),
        (
            "minecraft:hex_color",
            &["F0a", "FF0000", "FF-1+0"],
            &["FFFF", "GGG", "#F00"],
        ),
        (
            "minecraft:operation",
            &["=", "+=", "><"],
            &["==", "++", "+"],
        ),
    ];
    assert_parsers_gate_following_literal(cases);
}

fn assert_parsers_gate_following_literal(cases: &[(&str, &[&str], &[&str])]) {
    for &(parser, valid, invalid) in cases {
        let graph = parser_completion_graph(parser, Value::Null);
        for value in valid {
            let input = format!("😀 {value} n");
            let result = command_suggestions(&graph, 0, &input);
            assert_eq!(result.texts, ["next"], "{parser} valid {value:?}");
            let end = input.encode_utf16().count();
            assert_eq!((result.start, result.end), (end - 1, end));
        }
        for value in invalid {
            let input = format!("😀 {value} n");
            let result = command_suggestions(&graph, 0, &input);
            assert!(
                !result.texts.iter().any(|text| text == "next"),
                "{parser} invalid {value:?}"
            );
        }
    }
}

#[test]
fn minecraft_simple_arguments_offer_java_static_suggestions() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "minecraft:heightmap",
            "motion",
            &["motion_blocking", "motion_blocking_no_leaves"],
        ),
        ("minecraft:hex_color", "ff", &["FF0000"]),
        // SuggestionsBuilder drops a candidate equal to the remaining input.
        ("minecraft:operation", ">", &["><"]),
    ];
    for &(parser, prefix, expected) in cases {
        let graph = parser_completion_graph(parser, Value::Null);
        let input = format!("😀 {prefix}");
        let result = command_suggestions(&graph, 0, &input);
        assert_eq!(result.texts, expected, "{parser} {prefix:?}");
        assert_eq!(
            (result.start, result.end),
            (3, input.encode_utf16().count())
        );
    }
}
