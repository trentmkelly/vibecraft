//! `/function` running the queued function frames within the same command
//! (`execute_command_with_functions`, the live command path).

use super::*;
use crate::command::{execute_command_with_functions, MAX_FUNCTION_CHAIN_COMMANDS};

const GM: LevelBasedPermissionSet = LevelBasedPermissionSet::GAMEMASTER;

fn function(id: &str, commands: &[&str]) -> CommandFunctionDefinition {
    CommandFunctionDefinition {
        id: id.to_string(),
        commands: commands.iter().map(|c| (*c).to_string()).collect(),
        macro_parameters: Vec::new(),
    }
}

fn state_with(functions: Vec<CommandFunctionDefinition>) -> ServerCommandState {
    let steve = NameAndId::create_offline("Steve");
    ServerCommandState {
        online_players: vec![steve.clone()],
        command_source_player: Some(steve),
        available_functions: functions,
        ..ServerCommandState::default()
    }
}

fn said(state: &ServerCommandState) -> Vec<String> {
    state
        .chat_events
        .iter()
        .map(|event| event.message.clone())
        .collect()
}

#[test]
fn function_commands_run_in_order_after_the_call_is_queued() {
    let mut state = state_with(vec![function(
        "content:hello",
        &["say one", "say two", "say three"],
    )]);
    let result = execute_command_with_functions(&mut state, GM, "function content:hello");
    assert_eq!(
        result.map(|r| r.feedback_key),
        Ok("commands.function.scheduled.single")
    );
    assert_eq!(said(&state), vec!["one", "two", "three"]);
    assert!(state.queued_functions.is_empty());
}

#[test]
fn nested_function_calls_run_depth_first() {
    let mut state = state_with(vec![
        function(
            "content:outer",
            &["say a", "function content:inner", "say d"],
        ),
        function("content:inner", &["say b", "say c"]),
    ]);
    execute_command_with_functions(&mut state, GM, "function content:outer")
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(said(&state), vec!["a", "b", "c", "d"]);
}

#[test]
fn a_successful_return_ends_only_its_own_function() {
    let mut state = state_with(vec![
        function(
            "content:outer",
            &["function content:inner", "say after inner"],
        ),
        function("content:inner", &["say before", "return 1", "say unreachable"]),
    ]);
    execute_command_with_functions(&mut state, GM, "function content:outer")
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(said(&state), vec!["before", "after inner"]);
}

#[test]
fn a_failing_command_does_not_stop_the_function() {
    let mut state = state_with(vec![function(
        "content:mixed",
        &["not_a_command", "say still running"],
    )]);
    execute_command_with_functions(&mut state, GM, "function content:mixed")
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(said(&state), vec!["still running"]);
}

#[test]
fn a_self_calling_function_is_stopped_by_the_chain_limit() {
    let mut state = state_with(vec![function(
        "content:loop",
        &["say x", "function content:loop"],
    )]);
    execute_command_with_functions(&mut state, GM, "function content:loop")
        .unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(state.chat_events.len(), MAX_FUNCTION_CHAIN_COMMANDS.div_ceil(2));
    assert!(state.queued_functions.is_empty());
}

#[test]
fn commands_other_than_function_calls_are_untouched() {
    let mut state = state_with(Vec::new());
    execute_command_with_functions(&mut state, GM, "say plain").unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(said(&state), vec!["plain"]);
    assert_eq!(
        execute_command_with_functions(&mut state, GM, "function content:missing"),
        Err(CommandError::FunctionNoFunctions)
    );
}
