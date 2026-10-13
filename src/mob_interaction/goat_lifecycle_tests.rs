use super::*;

#[test]
fn goat_head_lowering_events_are_retained_until_the_next_event() {
    let mut goat = GoatState::new();

    assert!(!goat.is_lowering_head);
    assert!(!goat.handle_entity_event(57));
    assert!(!goat.is_lowering_head);

    assert!(goat.handle_entity_event(58));
    assert!(goat.is_lowering_head);
    assert!(!goat.handle_entity_event(57));
    assert!(
        goat.is_lowering_head,
        "unhandled events leave retained state alone"
    );
    for expected_tick in 1..=GOAT_MAX_LOWER_HEAD_TICK {
        goat.ai_step();
        assert_eq!(goat.lower_head_tick, expected_tick);
    }
    goat.ai_step();
    assert_eq!(goat.lower_head_tick, GOAT_MAX_LOWER_HEAD_TICK);

    assert!(goat.handle_entity_event(59));
    assert!(!goat.is_lowering_head);
    goat.ai_step();
    assert_eq!(goat.lower_head_tick, GOAT_MAX_LOWER_HEAD_TICK - 2);
    for _ in 0..GOAT_MAX_LOWER_HEAD_TICK {
        goat.ai_step();
    }
    assert_eq!(goat.lower_head_tick, 0);
    goat.ai_step();
    assert_eq!(goat.lower_head_tick, 0);
}

#[test]
fn goat_offspring_inherits_selected_screaming_parent_without_fallback_draw() {
    let mut fallback_draws = 0;
    let screaming = goat_baby_is_screaming(true, true, false, true, || {
        fallback_draws += 1;
        1.0
    });

    assert!(screaming);
    assert_eq!(fallback_draws, 0);

    let screaming = goat_baby_is_screaming(false, true, true, false, || {
        fallback_draws += 1;
        1.0
    });
    assert!(screaming);
    assert_eq!(fallback_draws, 0);
}

#[test]
fn goat_offspring_uses_java_fallback_for_non_screaming_or_non_goat_partner() {
    let mut fallback_draws = 0;
    let screaming = goat_baby_is_screaming(false, true, false, false, || {
        fallback_draws += 1;
        GOAT_SCREAMING_OFFSPRING_CHANCE
    });
    assert!(!screaming, "the fallback comparison is strict");
    assert_eq!(fallback_draws, 1);

    let screaming = goat_baby_is_screaming(false, false, true, false, || {
        GOAT_SCREAMING_OFFSPRING_CHANCE - f64::EPSILON
    });
    assert!(screaming, "a non-goat partner still gets the 0.02 fallback");
}

#[test]
fn goat_head_tick_matches_java_integer_wrap_before_clamping() {
    let mut goat = GoatState {
        is_lowering_head: true,
        lower_head_tick: i32::MAX,
        ..GoatState::new()
    };
    goat.ai_step();
    assert_eq!(goat.lower_head_tick, 0);

    goat.is_lowering_head = false;
    goat.lower_head_tick = i32::MIN;
    goat.ai_step();
    assert_eq!(goat.lower_head_tick, GOAT_MAX_LOWER_HEAD_TICK);
}
