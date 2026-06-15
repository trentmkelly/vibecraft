use super::*;

const MULTIPLE_TEST_TRACKER_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/MultipleTestTracker.java");

fn test_info(id: &str, required: bool) -> GameTestInfoStateModel {
    GameTestInfoStateModel::new(
        id,
        required,
        10,
        0,
        RotationModel::None,
        RotationModel::None,
        "noRetries",
    )
}

#[test]
fn multiple_test_tracker_matches_java_source_shape() {
    assert_eq!(MULTIPLE_TEST_TRACKER_JAVA.lines().count(), 117);
    for sentinel in [
        "private static final char NOT_STARTED_TEST_CHAR = ' ';",
        "private static final char ONGOING_TEST_CHAR = '_';",
        "private static final char SUCCESSFUL_TEST_CHAR = '+';",
        "private static final char FAILED_OPTIONAL_TEST_CHAR = 'x';",
        "private static final char FAILED_REQUIRED_TEST_CHAR = 'X';",
        "private final Collection<GameTestInfo> tests = Lists.newArrayList();",
        "private final Collection<GameTestListener> listeners = Lists.newArrayList();",
        "this.tests.addAll(tests);",
        "this.listeners.forEach(testInfo::addListener);",
        "this.tests.forEach(testInfo -> testInfo.addListener(listener));",
        "listener.accept(testInfo);",
        ".filter(GameTestInfo::hasFailed).filter(GameTestInfo::isRequired).count()",
        ".filter(GameTestInfo::hasFailed).filter(GameTestInfo::isOptional).count()",
        "return this.getDoneCount() == this.getTotalCount();",
        "if (!test.hasStarted())",
        "buf.append((char)(test.isRequired() ? 'X' : 'x'));",
        "return this.getProgressBar();",
        "this.tests.remove(testInfo);",
    ] {
        assert!(
            MULTIPLE_TEST_TRACKER_JAVA.contains(sentinel),
            "missing MultipleTestTracker sentinel {sentinel}"
        );
    }
}

#[test]
fn multiple_test_tracker_listener_attachment_matches_java() {
    let mut tracker = MultipleTestTrackerModel::new(vec![test_info("minecraft:one", true)]);
    tracker.add_listener();
    tracker.add_failure_listener();

    assert_eq!(tracker.listener_count, 2);
    assert_eq!(tracker.failure_listener_count, 1);
    assert_eq!(tracker.tests[0].listener_count, 2);

    tracker.add_test_to_track(test_info("minecraft:two", true));
    assert_eq!(tracker.tests[1].listener_count, 2);

    tracker.notify_test_failed("minecraft:two");
    assert_eq!(
        tracker.failure_notifications,
        vec!["minecraft:two".to_string()]
    );
}

#[test]
fn multiple_test_tracker_counts_failed_done_and_total_tests_like_java() {
    let mut required_failed = test_info("minecraft:required_failed", true);
    required_failed.started = true;
    required_failed.fail("required failure");
    required_failed.finish();

    let mut optional_failed = test_info("minecraft:optional_failed", false);
    optional_failed.started = true;
    optional_failed.fail("optional failure");
    optional_failed.finish();

    let mut passed = test_info("minecraft:passed", true);
    passed.started = true;
    passed.succeed();

    let running = {
        let mut info = test_info("minecraft:running", true);
        info.started = true;
        info
    };

    let tracker =
        MultipleTestTrackerModel::new(vec![required_failed, optional_failed, passed, running]);

    assert_eq!(tracker.get_failed_required_count(), 1);
    assert_eq!(tracker.get_failed_optional_count(), 1);
    assert_eq!(tracker.get_done_count(), 3);
    assert_eq!(tracker.get_total_count(), 4);
    assert!(tracker.has_failed_required());
    assert!(tracker.has_failed_optional());
    assert!(!tracker.is_done());
    assert_eq!(
        tracker.get_failed_required(),
        vec!["minecraft:required_failed".to_string()]
    );
    assert_eq!(
        tracker.get_failed_optional(),
        vec!["minecraft:optional_failed".to_string()]
    );
}

#[test]
fn multiple_test_tracker_progress_bar_and_remove_match_java_ordering() {
    let not_started = test_info("minecraft:not_started", true);

    let mut running = test_info("minecraft:running", true);
    running.started = true;

    let mut passed = test_info("minecraft:passed", true);
    passed.started = true;
    passed.succeed();

    let mut failed_required = test_info("minecraft:failed_required", true);
    failed_required.started = true;
    failed_required.fail("required failure");

    let mut failed_optional = test_info("minecraft:failed_optional", false);
    failed_optional.started = true;
    failed_optional.fail("optional failure");

    let mut failed_before_start = test_info("minecraft:failed_before_start", true);
    failed_before_start.fail("not started takes precedence");

    let mut tracker = MultipleTestTrackerModel::new(vec![
        not_started,
        running,
        passed,
        failed_required,
        failed_optional,
        failed_before_start,
    ]);

    assert_eq!(tracker.get_progress_bar(), "[ _+Xx ]");
    assert_eq!(tracker.to_string(), "[ _+Xx ]");

    tracker.remove("minecraft:running");
    assert_eq!(tracker.get_total_count(), 5);
    assert_eq!(tracker.get_progress_bar(), "[ +Xx ]");
}
