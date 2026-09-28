//! Parity tests for `/scoreboard` against Java `ScoreboardCommand` (26.1.2).

use super::*;

fn run(state: &mut ServerCommandState, line: &str) -> Result<CommandResult, CommandError> {
    execute_builtin_command(state, LevelBasedPermissionSet::GAMEMASTER, line)
}

fn state_with(objectives: &[(&str, &str)]) -> ServerCommandState {
    let mut state = ServerCommandState::default();
    for (name, criteria) in objectives {
        run(
            &mut state,
            &format!("scoreboard objectives add {name} {criteria}"),
        )
        .unwrap();
    }
    state
}

fn value(state: &ServerCommandState, owner: &str, objective: &str) -> i32 {
    scoreboard_score(state, owner, objective).unwrap().value
}

#[test]
fn add_objective_uses_criteria_default_render_type_and_no_auto_update() {
    let state = state_with(&[("hp", "health"), ("k", "dummy")]);
    assert_eq!(state.scoreboard_objectives[0].render_type, "hearts");
    assert_eq!(state.scoreboard_objectives[1].render_type, "integer");
    assert!(!state.scoreboard_objectives[1].display_auto_update);
}

#[test]
fn add_objective_rejects_unknown_criteria_and_non_word_names() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        run(&mut state, "scoreboard objectives add a bogus"),
        Err(CommandError::ScoreboardCriteriaInvalid)
    );
    assert_eq!(
        run(&mut state, "scoreboard objectives add a:b dummy"),
        Err(CommandError::InvalidSyntax)
    );
    assert!(run(&mut state, "scoreboard objectives add a.b+c-d_e dummy").is_ok());
    assert!(run(&mut state, "scoreboard objectives add s dummy").is_ok());
    assert!(run(
        &mut state,
        "scoreboard objectives add j minecraft.custom:minecraft.jump"
    )
    .is_ok());
}

#[test]
fn modify_is_silent_when_nothing_changes() {
    let mut state = state_with(&[("k", "dummy")]);
    for line in [
        "scoreboard objectives modify k displayname k",
        "scoreboard objectives modify k rendertype integer",
        "scoreboard objectives modify k displayautoupdate false",
    ] {
        let result = run(&mut state, line).unwrap();
        assert_eq!(result.feedback_key, NO_COMMAND_FEEDBACK, "{line}");
        assert_eq!(result.success_count, 0);
    }
    let changed = run(
        &mut state,
        "scoreboard objectives modify k displayautoupdate true",
    )
    .unwrap();
    assert_eq!(
        changed.feedback_key,
        "commands.scoreboard.objectives.modify.displayAutoUpdate.enable"
    );
}

#[test]
fn setdisplay_validates_slot_names() {
    let mut state = state_with(&[("k", "dummy")]);
    assert_eq!(
        run(&mut state, "scoreboard objectives setdisplay nowhere k"),
        Err(CommandError::InvalidSyntax)
    );
    assert!(run(
        &mut state,
        "scoreboard objectives setdisplay sidebar.team.red k"
    )
    .is_ok());
    assert!(run(&mut state, "scoreboard objectives setdisplay below_name k").is_ok());
}

#[test]
fn set_add_remove_use_java_results_and_wrapping() {
    let mut state = state_with(&[("k", "dummy")]);
    assert_eq!(
        run(&mut state, "scoreboard players set A,B k 5")
            .unwrap()
            .success_count,
        10
    );
    run(&mut state, "scoreboard players set C k 2147483647").unwrap();
    let added = run(&mut state, "scoreboard players add C k 1").unwrap();
    assert_eq!(value(&state, "C", "k"), i32::MIN);
    assert_eq!(added.success_count, i32::MIN);
    // Multiple targets: the result is the sum of the final scores, not the target count.
    let removed = run(&mut state, "scoreboard players remove A,B k 2").unwrap();
    assert_eq!(removed.success_count, 6);
    assert_eq!(
        removed.feedback_key,
        "commands.scoreboard.players.remove.success.multiple"
    );
}

#[test]
fn read_only_objectives_reject_writes_but_allow_reads() {
    let mut state = state_with(&[("hp", "health")]);
    for line in [
        "scoreboard players set A hp 1",
        "scoreboard players add A hp 1",
        "scoreboard players remove A hp 1",
    ] {
        assert_eq!(
            run(&mut state, line),
            Err(CommandError::ScoreboardObjectiveReadOnly)
        );
    }
    assert!(run(&mut state, "scoreboard players reset A hp").is_ok());
}

#[test]
fn wildcard_targets_expand_to_tracked_holders() {
    let mut state = state_with(&[("k", "dummy")]);
    run(&mut state, "scoreboard players set A k 1").unwrap();
    run(&mut state, "scoreboard players set B k 2").unwrap();
    let result = run(&mut state, "scoreboard players add * k 10").unwrap();
    assert_eq!(result.success_count, 23);
    assert_eq!(
        run(&mut state, "scoreboard players reset *")
            .unwrap()
            .success_count,
        2
    );
    assert!(state.scoreboard_scores.is_empty());
}

#[test]
fn operation_creates_source_scores_and_sums_targets() {
    let mut state = state_with(&[("k", "dummy")]);
    run(&mut state, "scoreboard players set A k 4").unwrap();
    let result = run(&mut state, "scoreboard players operation A k += Nobody k").unwrap();
    assert_eq!(result.success_count, 4);
    assert_eq!(value(&state, "Nobody", "k"), 0);
}

#[test]
fn operation_floors_division_and_modulo_and_swaps() {
    let mut state = state_with(&[("k", "dummy")]);
    run(&mut state, "scoreboard players set A k -7").unwrap();
    run(&mut state, "scoreboard players set B k 2").unwrap();
    run(&mut state, "scoreboard players operation A k /= B k").unwrap();
    assert_eq!(value(&state, "A", "k"), -4);
    run(&mut state, "scoreboard players set A k -7").unwrap();
    run(&mut state, "scoreboard players operation A k %= B k").unwrap();
    assert_eq!(value(&state, "A", "k"), 1);
    run(&mut state, "scoreboard players operation A k >< B k").unwrap();
    assert_eq!((value(&state, "A", "k"), value(&state, "B", "k")), (2, 1));
    run(&mut state, "scoreboard players set B k 0").unwrap();
    assert_eq!(
        run(&mut state, "scoreboard players operation A k /= B k"),
        Err(CommandError::OperationDivideByZero)
    );
    assert_eq!(
        run(&mut state, "scoreboard players operation A k %= B k"),
        Err(CommandError::OperationDivideByZero)
    );
    assert_eq!(
        run(&mut state, "scoreboard players operation A k ?? B k"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn operation_wraps_on_overflow() {
    let mut state = state_with(&[("k", "dummy")]);
    run(&mut state, "scoreboard players set A k 2147483647").unwrap();
    run(&mut state, "scoreboard players set B k 2").unwrap();
    run(&mut state, "scoreboard players operation A k *= B k").unwrap();
    assert_eq!(value(&state, "A", "k"), -2);
}
