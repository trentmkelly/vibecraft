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
