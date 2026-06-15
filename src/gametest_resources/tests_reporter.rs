use super::*;

const GLOBAL_TEST_REPORTER_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/GlobalTestReporter.java");
const JUNIT_LIKE_TEST_REPORTER_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/JUnitLikeTestReporter.java");
const LOG_TEST_REPORTER_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/LogTestReporter.java");
const REPORT_GAME_LISTENER_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/ReportGameListener.java");
const RETRY_OPTIONS_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/RetryOptions.java");
const TEST_REPORTER_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/TestReporter.java");

fn required_failure() -> ReporterTestInfoModel {
    ReporterTestInfoModel::new(
        "minecraft:required",
        "minecraft:structure",
        2500,
        true,
        BlockPosModel::new(1, 2, 3),
        Some("boom".to_string()),
    )
}

fn optional_failure() -> ReporterTestInfoModel {
    ReporterTestInfoModel::new(
        "minecraft:optional",
        "minecraft:structure",
        500,
        false,
        BlockPosModel::new(4, 5, 6),
        Some("soft boom".to_string()),
    )
}

#[test]
fn test_reporter_interface_and_global_delegate_match_java_shape() {
    assert_eq!(TEST_REPORTER_JAVA.lines().count(), 10);
    assert_eq!(GLOBAL_TEST_REPORTER_JAVA.lines().count(), 21);
    for sentinel in [
        "void onTestFailed(GameTestInfo testInfo);",
        "void onTestSuccess(GameTestInfo testInfo);",
        "default void finish()",
        "private static TestReporter DELEGATE = new LogTestReporter();",
        "DELEGATE = testReporter;",
        "DELEGATE.onTestFailed(testInfo);",
        "DELEGATE.onTestSuccess(testInfo);",
        "DELEGATE.finish();",
    ] {
        assert!(
            TEST_REPORTER_JAVA.contains(sentinel) || GLOBAL_TEST_REPORTER_JAVA.contains(sentinel),
            "missing reporter/global sentinel {sentinel}"
        );
    }
}

#[test]
fn log_and_junit_reporters_match_java_source_shape() {
    assert_eq!(LOG_TEST_REPORTER_JAVA.lines().count(), 23);
    assert_eq!(JUNIT_LIKE_TEST_REPORTER_JAVA.lines().count(), 79);
    for sentinel in [
        "LOGGER.error(\"{} failed at {}! {}\"",
        "LOGGER.warn(\"(optional) {} failed at {}. {}\"",
        "public void onTestSuccess(final GameTestInfo testInfo)",
        "private Element createTestCase(final GameTestInfo testInfo, final String name)",
        "testCase.setAttribute(\"classname\", testInfo.getStructure().toString());",
        "testCase.setAttribute(\"time\", String.valueOf(testInfo.getRunTime() / 1000.0));",
        "this.document.createElement(testInfo.isRequired() ? \"failure\" : \"skipped\")",
        "result.setAttribute(\"message\", \"(\" + testInfo.getTestBlockPos().toShortString() + \") \" + message);",
        "this.stopwatch.elapsed(TimeUnit.MILLISECONDS) / 1000.0",
        "this.save(this.destination);",
    ] {
        assert!(
            LOG_TEST_REPORTER_JAVA.contains(sentinel)
                || JUNIT_LIKE_TEST_REPORTER_JAVA.contains(sentinel),
            "missing concrete reporter sentinel {sentinel}"
        );
    }
}

#[test]
fn global_reporter_defaults_to_log_and_replace_with_delegates_all_callbacks() {
    let failed = required_failure();
    let passed = ReporterTestInfoModel::new(
        "minecraft:passed",
        "minecraft:structure",
        100,
        true,
        BlockPosModel::new(0, 0, 0),
        None,
    );
    let mut global = GlobalTestReporterModel::default();

    global.on_test_failed(&failed);
    assert_eq!(
        global.delegate,
        TestReporterDelegateModel::Log(LogTestReporterModel {
            logs: vec!["ERROR minecraft:required failed at 1, 2, 3! boom".to_string()],
        })
    );

    global.replace_with(TestReporterDelegateModel::Recording(
        RecordingTestReporterModel::default(),
    ));
    global.on_test_failed(&failed);
    global.on_test_success(&passed);
    global.finish(1234);

    assert_eq!(
        global.delegate,
        TestReporterDelegateModel::Recording(RecordingTestReporterModel {
            calls: vec![
                TestReporterCall::Failed("minecraft:required".to_string()),
                TestReporterCall::Success("minecraft:passed".to_string()),
                TestReporterCall::Finish,
            ],
        })
    );
}

#[test]
fn log_reporter_formats_required_and_optional_failures_and_ignores_success() {
    let mut reporter = LogTestReporterModel::default();
    reporter.on_test_failed(&required_failure());
    reporter.on_test_failed(&optional_failure());
    reporter.on_test_success(&required_failure());
    reporter.finish();

    assert_eq!(
        reporter.logs,
        vec![
            "ERROR minecraft:required failed at 1, 2, 3! boom",
            "WARN (optional) minecraft:optional failed at 4, 5, 6. soft boom",
        ]
    );
}

#[test]
fn junit_reporter_records_cases_failure_skipped_success_and_finish_save_target() {
    let mut reporter = JUnitLikeTestReporterModel::new("report.xml");
    let passed = ReporterTestInfoModel::new(
        "minecraft:passed",
        "minecraft:structure",
        1250,
        true,
        BlockPosModel::new(7, 8, 9),
        None,
    );

    reporter.on_test_failed(&required_failure());
    reporter.on_test_failed(&optional_failure());
    reporter.on_test_success(&passed);
    reporter.finish(3456);

    assert!(reporter.timestamp_set);
    assert_eq!(reporter.suite_time_seconds, Some(3.456));
    assert_eq!(reporter.saved_destination, Some("report.xml".to_string()));
    assert_eq!(
        reporter.test_cases,
        vec![
            JUnitTestCaseModel {
                name: "minecraft:required".to_string(),
                classname: "minecraft:structure".to_string(),
                time_seconds: 2.5,
                result: Some(JUnitTestResultModel::Failure {
                    message: "(1, 2, 3) boom".to_string(),
                }),
            },
            JUnitTestCaseModel {
                name: "minecraft:optional".to_string(),
                classname: "minecraft:structure".to_string(),
                time_seconds: 0.5,
                result: Some(JUnitTestResultModel::Skipped {
                    message: "(4, 5, 6) soft boom".to_string(),
                }),
            },
            JUnitTestCaseModel {
                name: "minecraft:passed".to_string(),
                classname: "minecraft:structure".to_string(),
                time_seconds: 1.25,
                result: None,
            },
        ]
    );
}

#[test]
fn retry_options_match_java_source_shape_and_attempt_logic() {
    assert_eq!(RETRY_OPTIONS_JAVA.lines().count(), 23);
    for sentinel in [
        "public record RetryOptions(int numberOfTries, boolean haltOnFailure)",
        "private static final RetryOptions NO_RETRIES = new RetryOptions(1, true);",
        "return this.numberOfTries < 1;",
        "boolean hasFailures = attempts != successes;",
        "boolean hasMoreAttempts = this.unlimitedTries() || attempts < this.numberOfTries;",
        "return hasMoreAttempts && (!hasFailures || !this.haltOnFailure);",
        "return this.numberOfTries != 1;",
    ] {
        assert!(
            RETRY_OPTIONS_JAVA.contains(sentinel),
            "missing RetryOptions sentinel {sentinel}"
        );
    }

    assert_eq!(RetryOptionsModel::no_retries().number_of_tries, 1);
    assert!(RetryOptionsModel {
        number_of_tries: 0,
        halt_on_failure: true,
    }
    .unlimited_tries());
    assert!(!RetryOptionsModel::no_retries().has_retries());
    assert!(RetryOptionsModel {
        number_of_tries: 3,
        halt_on_failure: true,
    }
    .has_retries());
    assert!(RetryOptionsModel {
        number_of_tries: 3,
        halt_on_failure: true,
    }
    .has_tries_left(2, 2));
    assert!(!RetryOptionsModel {
        number_of_tries: 3,
        halt_on_failure: true,
    }
    .has_tries_left(2, 1));
    assert!(RetryOptionsModel {
        number_of_tries: 3,
        halt_on_failure: false,
    }
    .has_tries_left(2, 1));
    assert!(!RetryOptionsModel {
        number_of_tries: 3,
        halt_on_failure: false,
    }
    .has_tries_left(3, 2));
}

#[test]
fn report_game_listener_matches_java_source_shape() {
    assert_eq!(REPORT_GAME_LISTENER_JAVA.lines().count(), 136);
    for sentinel in [
        "private int attempts = 0;",
        "private int successes = 0;",
        "this.attempts++;",
        "String.format(Locale.ROOT, \"[Run: %4d, Ok: %4d, Fail: %4d\"",
        "retryOptions.numberOfTries() - this.attempts",
        "runner.rerunTest(testInfo);",
        "this.successes++;",
        "reportPassed(testInfo, testInfo.id() + \" passed! (\" + testInfo.getRunTime() + \"ms / \" + testInfo.getTick() + \"gameticks)\");",
        "Flaky test \" + testInfo + \" succeeded, attempt: \"",
        "reportFailure(testInfo, new ExhaustedAttemptsException(this.attempts, this.successes, testInfo));",
        "copy.addListener(this);",
        "blockEntity.setSuccess()",
        "GlobalTestReporter.onTestSuccess(testInfo);",
        "blockEntity.setErrorMessage(description)",
        "markError(assertError.getAbsolutePos(), assertError.getMessageToShowAtBlock())",
        "GlobalTestReporter.onTestFailed(testInfo);",
    ] {
        assert!(
            REPORT_GAME_LISTENER_JAVA.contains(sentinel),
            "missing ReportGameListener sentinel {sentinel}"
        );
    }
}

#[test]
fn report_game_listener_reports_plain_success_and_failure_to_world_and_global_reporter() {
    let mut listener = ReportGameListenerModel::default();
    let passed = ReportGameListenerTestInfoModel::new("minecraft:plain_pass").with_runtime(123, 7);
    listener.test_structure_loaded();
    listener.test_passed(&passed);

    assert_eq!(listener.attempts, 1);
    assert_eq!(listener.successes, 1);
    assert_eq!(
        listener.block_entity_events,
        vec![ReportGameListenerBlockEntityEvent::Success]
    );
    assert_eq!(
        listener.chats,
        vec![ReportGameListenerChatEvent {
            format: ReportGameListenerChatFormat::Green,
            text: "minecraft:plain_pass passed! (123ms / 7gameticks)".to_string(),
        }]
    );
    assert_eq!(
        listener.global_calls,
        vec![TestReporterCall::Success(
            "minecraft:plain_pass".to_string()
        )]
    );

    let mut listener = ReportGameListenerModel::default();
    let failed = ReportGameListenerTestInfoModel::new("minecraft:plain_fail")
        .with_error(ReportGameListenerErrorModel::plain("boom"));
    listener.test_structure_loaded();
    listener.test_failed(&failed);

    assert_eq!(
        listener.block_entity_events,
        vec![ReportGameListenerBlockEntityEvent::ErrorMessage(
            "boom".to_string()
        )]
    );
    assert_eq!(
        listener.chats,
        vec![ReportGameListenerChatEvent {
            format: ReportGameListenerChatFormat::Red,
            text: "minecraft:plain_fail failed! boom".to_string(),
        }]
    );
    assert_eq!(
        listener.global_calls,
        vec![TestReporterCall::Failed("minecraft:plain_fail".to_string())]
    );
}

#[test]
fn report_game_listener_retry_status_and_rerun_match_java() {
    let mut listener = ReportGameListenerModel::default();
    let retrying = ReportGameListenerTestInfoModel::new("minecraft:retry")
        .with_runtime(50, 2)
        .with_retry_options(RetryOptionsModel {
            number_of_tries: 3,
            halt_on_failure: false,
        });

    listener.test_structure_loaded();
    listener.test_failed(
        &retrying
            .clone()
            .with_error(ReportGameListenerErrorModel::plain("first failure")),
    );

    assert_eq!(
        listener.chats,
        vec![
            ReportGameListenerChatEvent {
                format: ReportGameListenerChatFormat::Red,
                text: "minecraft:retry failed! first failure".to_string(),
            },
            ReportGameListenerChatEvent {
                format: ReportGameListenerChatFormat::Red,
                text: "[Run:    1, Ok:    0, Fail:    1, Left:    2]        minecraft:retry failed! 50ms"
                    .to_string(),
            },
        ]
    );
    assert_eq!(listener.reruns, vec!["minecraft:retry".to_string()]);

    listener.test_structure_loaded();
    listener.test_passed(&retrying);

    assert_eq!(listener.successes, 1);
    assert_eq!(
        listener.reruns,
        vec!["minecraft:retry".to_string(), "minecraft:retry".to_string()]
    );
    assert_eq!(
        listener.global_calls,
        vec![
            TestReporterCall::Failed("minecraft:retry".to_string()),
            TestReporterCall::Success("minecraft:retry".to_string()),
        ]
    );
    assert_eq!(
        listener.chats.last().map(|event| event.text.as_str()),
        Some("[Run:    2, Ok:    1, Fail:    1, Left:    1]        minecraft:retry passed! 50ms")
    );
}

#[test]
fn report_game_listener_flaky_success_rerun_and_exhaustion_match_java() {
    let mut listener = ReportGameListenerModel::default();
    let flaky = ReportGameListenerTestInfoModel::new("minecraft:flaky").flaky(3, 2);

    listener.test_structure_loaded();
    listener.test_passed(&flaky);
    assert_eq!(
        listener.chats,
        vec![ReportGameListenerChatEvent {
            format: ReportGameListenerChatFormat::Green,
            text: "Flaky test minecraft:flaky succeeded, attempt: 1 successes: 1".to_string(),
        }]
    );
    assert_eq!(listener.reruns, vec!["minecraft:flaky".to_string()]);

    listener.test_structure_loaded();
    listener.test_passed(&flaky);
    assert_eq!(
        listener.chats.last(),
        Some(&ReportGameListenerChatEvent {
            format: ReportGameListenerChatFormat::Green,
            text: "minecraft:flaky passed 2 times of 2 attempts.".to_string(),
        })
    );
    assert_eq!(
        listener.global_calls.last(),
        Some(&TestReporterCall::Success("minecraft:flaky".to_string()))
    );

    let mut listener = ReportGameListenerModel::default();
    let failed_flaky = ReportGameListenerTestInfoModel::new("minecraft:flaky_fail")
        .flaky(3, 2)
        .with_error(ReportGameListenerErrorModel::plain("last failure"));
    listener.test_structure_loaded();
    listener.test_failed(&failed_flaky);
    assert_eq!(listener.reruns, vec!["minecraft:flaky_fail".to_string()]);

    listener.test_structure_loaded();
    listener.test_failed(&failed_flaky);
    assert_eq!(
        listener.block_entity_events,
        vec![ReportGameListenerBlockEntityEvent::ErrorMessage(
            "Not enough successes: 0 out of 2 attempts. Required successes: 2. max attempts: 3."
                .to_string()
        )]
    );
    assert_eq!(
        listener.global_calls,
        vec![TestReporterCall::Failed("minecraft:flaky_fail".to_string())]
    );
}

#[test]
fn report_game_listener_assertion_descriptions_positions_and_rerun_attachment_match_java() {
    let mut listener = ReportGameListenerModel::default();
    let positional_failure = ReportGameListenerTestInfoModel::new("minecraft:assert_pos")
        .optional()
        .with_error(ReportGameListenerErrorModel::positional_assertion(
            "wrong block",
            "test.error.position",
            BlockPosModel::new(9, 10, 11),
            "show this at block",
        ));

    listener.test_structure_loaded();
    listener.test_failed(&positional_failure);

    assert_eq!(
        listener.block_entity_events,
        vec![
            ReportGameListenerBlockEntityEvent::ErrorMessage("test.error.position".to_string()),
            ReportGameListenerBlockEntityEvent::MarkError {
                absolute_pos: BlockPosModel::new(9, 10, 11),
                message: "show this at block".to_string(),
            },
        ]
    );
    assert_eq!(
        listener.chats,
        vec![ReportGameListenerChatEvent {
            format: ReportGameListenerChatFormat::Yellow,
            text: "(optional) minecraft:assert_pos failed! wrong block".to_string(),
        }]
    );

    listener.test_added_for_rerun(&ReportGameListenerTestInfoModel::new("minecraft:copy"));
    assert_eq!(
        listener.attached_rerun_ids,
        vec!["minecraft:copy".to_string()]
    );
}
