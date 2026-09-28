//! Command parse-tree parity against the official 26.1.2 `commands.json` report
//! (`--reports` output of the vanilla server, the `/brigadier dump` equivalent),
//! vendored at `vanilla-data/reports/commands_26_1_2.json`.
//!
//! The report was cross-checked against `Commands.java` and every
//! `net/minecraft/server/commands/*Command.register`. VibeCraft does not (yet)
//! own a full Brigadier tree: it dispatches on string slices (see `dispatch.rs`)
//! and only sends a `/biome` debug tree in `ClientboundCommands`. These tests
//! therefore pin what *can* be compared (root literals, permission gates,
//! sub-command permission overrides, signed-argument locations, redirects) and
//! pin the known gaps explicitly so that closing one forces a test update.

use super::*;
use serde_json::Value;

const REPORT: &str = include_str!("../../../vanilla-data/reports/commands_26_1_2.json");

fn report() -> Value {
    serde_json::from_str(REPORT).expect("commands report is valid JSON")
}

fn children(node: &Value) -> impl Iterator<Item = (&String, &Value)> {
    node["children"].as_object().into_iter().flatten()
}

/// Required level name of a node (`gamemasters`, `admins`, `owners`), if it has
/// its own `minecraft:require` gate. Nodes without one inherit or are open.
fn own_level(node: &Value) -> Option<&str> {
    node["permissions"]["permission"]["level"].as_str()
}

fn count_nodes(node: &Value) -> usize {
    1 + children(node)
        .map(|(_, child)| count_nodes(child))
        .sum::<usize>()
}

/// Paths (`a/b/c`) of every node whose parser is `parser`.
fn paths_with_parser(node: &Value, parser: &str, prefix: &str, out: &mut Vec<String>) {
    for (name, child) in children(node) {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if child["parser"].as_str() == Some(parser) {
            out.push(path.clone());
        }
        paths_with_parser(child, parser, &path, out);
    }
}

/// Paths of nodes that carry their own permission gate below a root literal.
fn nested_gates(node: &Value, prefix: &str, out: &mut Vec<(String, String)>) {
    for (name, child) in children(node) {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if let Some(level) = own_level(child).filter(|_| !prefix.is_empty()) {
            out.push((path.clone(), level.to_string()));
        }
        nested_gates(child, &path, out);
    }
}

#[test]
fn report_has_the_pinned_vanilla_shape() {
    let report = report();
    assert_eq!(report["type"], "root");
    assert_eq!(children(&report).count(), 91);
    assert_eq!(count_nodes(&report), 2092);
}

#[test]
fn root_literal_permission_gates_match_the_report() {
    let report = report();
    for (name, node) in children(&report) {
        let expected = match own_level(node) {
            None => PermissionLevel::All,
            Some("gamemasters") => PermissionLevel::Gamemasters,
            Some("admins") => PermissionLevel::Admins,
            Some("owners") => PermissionLevel::Owners,
            Some(other) => panic!("unexpected level {other} on /{name}"),
        };
        assert_eq!(
            command_required_permission(name),
            expected,
            "/{name} permission gate diverges from the vanilla dump"
        );
    }
}

#[test]
fn nested_permission_gates_match_the_report() {
    let mut gates = Vec::new();
    // The `prefix` guard in `nested_gates` skips root literals themselves.
    nested_gates(&report(), "", &mut gates);
    gates.sort();
    // Java: DataPackCommand `create` LEVEL_OWNERS; DebugCommand `function`
    // LEVEL_ADMINS (root `debug` is ADMINS already); RandomCommand `reset` and
    // the `roll|value <range> <sequence>` branches LEVEL_GAMEMASTERS.
    let expected = [
        ("datapack/create", "owners"),
        ("debug/function", "admins"),
        ("random/reset", "gamemasters"),
        ("random/roll/range/sequence", "gamemasters"),
        ("random/value/range/sequence", "gamemasters"),
    ]
    .map(|(path, level)| (path.to_string(), level.to_string()));
    assert_eq!(gates, expected);

    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "datapack create x y"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "random reset *"),
        Err(CommandError::PermissionDenied)
    );
}

#[test]
fn signed_arguments_are_exactly_the_message_argument_nodes() {
    // Only `MessageArgument implements SignedArgument` in the Java sources, and
    // `SignableCommand.of` signs exactly the nodes whose type is a `SignedArgument`.
    let mut paths = Vec::new();
    paths_with_parser(&report(), "minecraft:message", "", &mut paths);
    paths.sort();
    // One entry per `MessageArgument.message()` call site: BanIpCommands,
    // BanPlayerCommands, KickCommand, EmoteCommands, MsgCommand, SayCommand,
    // TeamMsgCommand. `tell`/`w`/`tm` are redirects, not separate argument nodes.
    assert_eq!(
        paths,
        [
            "ban-ip/target/reason",
            "ban/targets/reason",
            "kick/targets/reason",
            "me/action",
            "msg/targets/message",
            "say/message",
            "teammsg/message",
        ]
    );
}

#[test]
fn root_aliases_are_redirects_in_the_report() {
    let report = report();
    for (alias, target) in [
        ("tell", "msg"),
        ("w", "msg"),
        ("tm", "teammsg"),
        ("tp", "teleport"),
        ("xp", "experience"),
    ] {
        assert_eq!(
            report["children"][alias]["redirect"],
            serde_json::json!([target]),
            "/{alias}"
        );
    }
}

#[test]
fn every_vanilla_root_literal_is_known_to_the_rust_dispatcher_except_pinned_gaps() {
    // Aliases share their target's dispatch arm and usage entry.
    let aliases = [("w", "msg"), ("xp", "experience")];
    let report = report();
    let mut missing: Vec<&str> = children(&report)
        .map(|(name, _)| name.as_str())
        .filter(|name| {
            let lookup = aliases
                .iter()
                .find(|(alias, _)| alias == name)
                .map_or(*name, |(_, target)| *target);
            command_usage(lookup, LevelBasedPermissionSet::OWNER).is_none()
        })
        .collect();
    missing.sort_unstable();
    // TODO(commands-data-test): `/data` (DataCommands) and `/test` (TestCommand,
    // gametest framework) have no Rust dispatcher arm or usage entry yet.
    assert_eq!(missing, ["data", "test"]);
}
