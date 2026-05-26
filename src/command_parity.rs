#![allow(dead_code)]

use crate::command::{
    execute_builtin_command, CommandError, CommandResult, LevelBasedPermissionSet,
    ServerCommandState,
};
#[cfg(test)]
use crate::command::ChatCommandKind;
use crate::command_tree::{ArgumentParser, CommandNodeKind, CommandTree};
#[cfg(test)]
use crate::command_tree::ParseError;
#[cfg(test)]
use crate::player_access::NameAndId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandParityExpectation {
    pub input: &'static str,
    pub permission: LevelBasedPermissionSet,
    pub expected: Result<CommandResult, CommandError>,
}

pub fn run_command_parity_cases(
    state: &mut ServerCommandState,
    cases: &[CommandParityExpectation],
) -> Vec<Result<CommandResult, CommandError>> {
    cases
        .iter()
        .map(|case| execute_builtin_command(state, case.permission, case.input))
        .collect()
}

pub fn command_error_identifier(error: &CommandError) -> String {
    format!("{error:?}")
}

pub fn representative_command_tree() -> CommandTree {
    let mut tree = CommandTree::new();
    let gamemode = tree.add_child(0, CommandNodeKind::Literal("gamemode"), 2, false);
    tree.add_child(
        gamemode,
        CommandNodeKind::Argument {
            name: "mode",
            parser: ArgumentParser::Word,
            signed: false,
        },
        2,
        true,
    );
    tree.add_child(0, CommandNodeKind::Literal("seed"), 2, true);
    tree.add_child(0, CommandNodeKind::Literal("list"), 0, true);
    let say = tree.add_child(0, CommandNodeKind::Literal("say"), 2, false);
    tree.add_child(
        say,
        CommandNodeKind::Argument {
            name: "message",
            parser: ArgumentParser::GreedyString,
            signed: true,
        },
        2,
        true,
    );
    tree
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(name: &str) -> NameAndId {
        NameAndId {
            uuid: format!("{name}-uuid"),
            name: name.to_string(),
        }
    }

    #[test]
    fn command_parity_cases_cover_success_counts_and_feedback_keys() {
        let mut state = ServerCommandState {
            online_players: vec![player("Steve"), player("Alex")],
            world_seed: 12_345,
            ..ServerCommandState::default()
        };
        let cases = vec![
            CommandParityExpectation {
                input: "/list",
                permission: LevelBasedPermissionSet::ALL,
                expected: Ok(CommandResult {
                    success_count: 2,
                    feedback_key: "commands.list.players",
                    broadcast_to_admins: false,
                }),
            },
            CommandParityExpectation {
                input: " seed ",
                permission: LevelBasedPermissionSet::GAMEMASTER,
                expected: Ok(CommandResult {
                    success_count: 12_345,
                    feedback_key: "commands.seed.success",
                    broadcast_to_admins: false,
                }),
            },
        ];

        assert_eq!(
            run_command_parity_cases(&mut state, &cases),
            vec![cases[0].expected.clone(), cases[1].expected.clone(),]
        );
    }

    #[test]
    fn command_parity_syntax_and_suggestions_match_permission_filtered_tree() {
        let tree = representative_command_tree();

        assert_eq!(tree.suggestions("", 0), vec!["list".to_string()]);
        assert_eq!(
            tree.suggestions("s", 2),
            vec!["seed".to_string(), "say".to_string()]
        );
        assert_eq!(tree.suggestions("gamemode ", 2), vec!["<mode>".to_string()]);
        assert!(tree.suggestions("gamemode ", 1).is_empty());
        assert_eq!(
            tree.parse("say hello world", 2).unwrap().arguments[0].value,
            "hello world"
        );
        assert_eq!(
            tree.parse("seed extra", 2),
            Err(ParseError::InvalidArgument("argument"))
        );
        assert_eq!(
            tree.parse("gamemode creative", 1),
            Err(ParseError::PermissionDenied)
        );
    }

    #[test]
    fn command_parity_side_effects_record_chat_and_block_changes() {
        let mut state = ServerCommandState::default();

        let say = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "say parity check",
        )
        .unwrap();
        assert_eq!(say.success_count, 1);
        assert_eq!(state.chat_events[0].kind, ChatCommandKind::Say);
        assert_eq!(state.chat_events[0].message, "parity check");

        let setblock = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 1 64 2 stone replace",
        )
        .unwrap();
        assert_eq!(setblock.success_count, 1);
        assert_eq!(state.setblock_events[0].block, "minecraft:stone");
        assert_eq!(state.setblock_events[0].position.x, 1);
        assert_eq!(state.setblock_events[0].position.y, 64);
        assert_eq!(state.setblock_events[0].position.z, 2);
    }

    #[test]
    fn command_parity_error_identifiers_are_stable_for_messages_and_failures() {
        let mut state = ServerCommandState::default();

        assert_eq!(
            command_error_identifier(
                &execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "seed")
                    .unwrap_err()
            ),
            "PermissionDenied"
        );
        assert_eq!(
            command_error_identifier(
                &execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "tell Steve")
                    .unwrap_err()
            ),
            "InvalidSyntax"
        );
        assert_eq!(
            command_error_identifier(
                &execute_builtin_command(
                    &mut state,
                    LevelBasedPermissionSet::ADMIN,
                    "transfer example.com"
                )
                .unwrap_err()
            ),
            "NoPlayers"
        );
    }
}
