use super::*;

const GLOBAL_TEST_REPORTER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/GlobalTestReporter.java"
);
const JUNIT_LIKE_TEST_REPORTER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/JUnitLikeTestReporter.java"
);
const LOG_TEST_REPORTER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/LogTestReporter.java"
);
const TEST_REPORTER_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/TestReporter.java"
);

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
