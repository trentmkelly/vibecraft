use super::{
    crash_report_format_block_location_xyz, crash_report_format_location,
    crash_report_format_location_in_level, CrashReport, CrashReportCategoryModel,
    CrashReportDetailValue, CrashReportModel, CrashReportPreloadAction,
    JavaLevelHeightAccessorModel, JavaStackTraceElementModel, JavaThrowableModel,
};
use crate::block_update::BlockPos;
use crate::report_type::REPORT_TYPE_TEST;
use crate::system_report::SystemReportModel;
use std::fs;
use std::time::Duration;

#[test]
fn rendered_crash_report_contains_required_sections() {
    let report = CrashReport {
        title: "boom".to_string(),
        details: vec![
            ("Thread".to_string(), "main".to_string()),
            ("World State".to_string(), "not loaded".to_string()),
        ],
        backtrace: "trace".to_string(),
    };

    let rendered = report.render();
    assert!(rendered.contains("---- VibeCraft Crash Report ----"));
    assert!(rendered.contains("Description: boom"));
    assert!(rendered.contains("-- System Details --"));
    assert!(rendered.contains("World State: not loaded"));
    assert!(rendered.contains("-- Backtrace --"));
}

#[test]
fn writes_vanilla_named_server_crash_report_file() {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vibecraft-crash-report-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);

    let report = CrashReport {
        title: "boom".to_string(),
        details: vec![("Thread".to_string(), "main".to_string())],
        backtrace: "trace".to_string(),
    };

    let path = report.write_to_dir(&dir).unwrap();
    let file_name = path.file_name().unwrap().to_string_lossy();
    assert!(file_name.starts_with("crash-"));
    assert!(file_name.ends_with("-server.txt"));
    assert!(fs::read_to_string(path)
        .unwrap()
        .contains("Description: boom"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn watchdog_crash_report_contains_java_watchdog_context() {
    let world_root = std::env::temp_dir().join("vibecraft-watchdog-report-world");
    let report = CrashReport::from_watchdog_tick(
        42,
        Duration::from_millis(61_500),
        Duration::from_millis(60_000),
        &world_root,
    );

    let rendered = report.render();
    assert!(rendered.contains("Description: Watching Server"));
    assert!(rendered.contains("Thread: Server Watchdog"));
    assert!(rendered.contains("Performance stats: tick=42, elapsed=61500ms, maxTickTime=60000ms"));
    assert!(rendered.contains(&format!("World Root: {}", world_root.display())));
}

#[test]
fn crash_report_category_entry_values_match_java_entry_constructor() {
    let mut category = CrashReportCategoryModel::new("Level");
    category.set_detail("String", "stone");
    category.set_detail("Number", 7);
    category.set_detail_null("Null");
    category.set_detail_error(
        "Failure",
        JavaThrowableModel::new("IllegalStateException", Some("bad state")),
    );
    category.set_detail_error(
        "Missing message",
        JavaThrowableModel::new("NullPointerException", Option::<String>::None),
    );

    assert_eq!(
            category.details(),
            "-- Level --\nDetails:\n\tString: stone\n\tNumber: 7\n\tNull: ~~NULL~~\n\tFailure: ~~ERROR~~ IllegalStateException: bad state\n\tMissing message: ~~ERROR~~ NullPointerException: null"
        );
}

#[test]
fn crash_report_category_detail_callback_matches_java_callable_handling() {
    let mut category = CrashReportCategoryModel::new("Callbacks");
    category.set_detail_callback("Ok", || Ok(CrashReportDetailValue::from("value")));
    category.set_detail_callback("Err", || {
        Err(JavaThrowableModel::new("RuntimeException", Some("boom")))
    });

    assert_eq!(
        category.details(),
        "-- Callbacks --\nDetails:\n\tOk: value\n\tErr: ~~ERROR~~ RuntimeException: boom"
    );
}

#[test]
fn crash_report_category_location_formatting_matches_java_math() {
    let level = JavaLevelHeightAccessorModel {
        min_y: -64,
        max_y: 319,
    };

    assert_eq!(
        crash_report_format_location(1.234, -5.0, 9.876),
        "1.23,-5.00,9.88"
    );
    assert_eq!(
            crash_report_format_location_in_level(level, -1.2, 70.9, 512.0),
            "-1.20,70.90,512.00 - World: (-2,70,512), Section: (at 14,6,0 in -1,4,32; chunk contains blocks -16,-64,512 to -1,319,527), Region: (-1,1; contains chunks -32,32 to -1,63, blocks -512,-64,512 to -1,319,1023)"
        );
    assert_eq!(
            crash_report_format_block_location_xyz(level, 32, -1, -33),
            "World: (32,-1,-33), Section: (at 0,15,15 in 2,-1,-3; chunk contains blocks 32,-64,-48 to 47,319,-33), Region: (0,-1; contains chunks 0,-32 to 31,-1, blocks 0,-64,-512 to 511,319,-1)"
        );
}

#[test]
fn crash_report_category_stacktrace_render_trim_and_validate_match_java() {
    let full = vec![
        JavaStackTraceElementModel::new("java.lang.Thread", "getStackTrace", "Thread.java", 1),
        JavaStackTraceElementModel::new(
            "net.minecraft.CrashReportCategory",
            "fillInStackTrace",
            "CrashReportCategory.java",
            126,
        ),
        JavaStackTraceElementModel::new(
            "net.minecraft.CrashReport",
            "addCategory",
            "CrashReport.java",
            145,
        ),
        JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 10),
        JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 20),
    ];
    let mut category = CrashReportCategoryModel::new("Stack");

    assert_eq!(category.fill_in_stack_trace_from_thread_trace(&full, 0), 2);
    assert_eq!(
        category.stacktrace(),
        &[
            JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 10),
            JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 20),
        ]
    );

    let source = JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 99);
    let next = JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 20);
    assert!(category.validate_stack_trace(Some(source.clone()), Some(next)));
    assert_eq!(category.stacktrace()[0], source);
    assert!(category.details().contains(
        "\nStacktrace:\n\tat game.Source.tick(Source.java:99)\n\tat game.Next.run(Next.java:20)"
    ));

    assert!(!category.validate_stack_trace(
        Some(JavaStackTraceElementModel::new(
            "other.Source",
            "tick",
            "Source.java",
            99
        )),
        Some(JavaStackTraceElementModel::new(
            "game.Next",
            "run",
            "Next.java",
            20
        )),
    ));

    category.trim_stacktrace(1);
    assert_eq!(
        category.stacktrace(),
        &[JavaStackTraceElementModel::new(
            "game.Source",
            "tick",
            "Source.java",
            99
        )]
    );
}

#[test]
fn crash_report_category_block_location_uses_block_pos() {
    let level = JavaLevelHeightAccessorModel {
        min_y: 0,
        max_y: 255,
    };
    assert_eq!(
            super::crash_report_format_block_location(
                level,
                BlockPos {
                    x: 15,
                    y: 64,
                    z: 16,
                },
            ),
            "World: (15,64,16), Section: (at 15,0,0 in 0,4,1; chunk contains blocks 0,0,16 to 15,255,31), Region: (0,0; contains chunks 0,0 to 31,31, blocks 0,0,0 to 511,255,511)"
        );
}

#[test]
fn crash_report_exception_message_replaces_selected_null_messages_with_title() {
    let stack = vec![JavaStackTraceElementModel::new(
        "game.Main",
        "tick",
        "Main.java",
        12,
    )];
    let report = CrashReportModel::new(
        "Ticking entity",
        JavaThrowableModel::new("NullPointerException", Option::<String>::None)
            .with_stack_trace(stack.clone()),
        SystemReportModel::empty(),
    );
    assert_eq!(
        report.get_exception_message(),
        "java.lang.NullPointerException: Ticking entity\n\tat game.Main.tick(Main.java:12)\n"
    );

    let report = CrashReportModel::new(
        "Ticking entity",
        JavaThrowableModel::new("IllegalStateException", Option::<String>::None)
            .with_stack_trace(stack),
        SystemReportModel::empty(),
    );
    assert_eq!(
        report.get_exception_message(),
        "java.lang.IllegalStateException\n\tat game.Main.tick(Main.java:12)\n"
    );
}

#[test]
fn crash_report_details_include_head_categories_and_system_report_like_java() {
    let mut system = SystemReportModel::empty();
    system.set_detail("Minecraft Version", "26.1.2");
    let mut report = CrashReportModel::new(
        "Boom",
        JavaThrowableModel::new("RuntimeException", Some("bad")),
        system,
    );
    let mut category = CrashReportCategoryModel::new("Level");
    category.set_detail("Block", "minecraft:stone");
    category.fill_in_stack_trace_from_thread_trace(
        &[
            JavaStackTraceElementModel::new("java.lang.Thread", "getStackTrace", "Thread.java", 1),
            JavaStackTraceElementModel::new(
                "net.minecraft.CrashReportCategory",
                "fillInStackTrace",
                "CrashReportCategory.java",
                2,
            ),
            JavaStackTraceElementModel::new(
                "net.minecraft.CrashReport",
                "addCategory",
                "CrashReport.java",
                3,
            ),
            JavaStackTraceElementModel::new("game.Level", "tick", "Level.java", 4),
        ],
        0,
    );
    report.details.push(category);

    assert_eq!(
            report.get_details("Server thread"),
            "-- Head --\nThread: Server thread\nStacktrace:\n\tat game.Level.tick(Level.java:4)\n\n-- Level --\nDetails:\n\tBlock: minecraft:stone\nStacktrace:\n\tat game.Level.tick(Level.java:4)\n\n-- System Details --\nDetails:\n\tMinecraft Version: 26.1.2"
        );
}

#[test]
fn crash_report_friendly_report_matches_java_section_order() {
    let mut system = SystemReportModel::empty();
    system.set_detail("Minecraft Version", "26.1.2");
    let mut report = CrashReportModel::new(
        "Loading world",
        JavaThrowableModel::new("RuntimeException", Some("boom")).with_stack_trace(vec![
            JavaStackTraceElementModel::new("game.Main", "load", "Main.java", 8),
        ]),
        system,
    );

    let friendly = report.get_friendly_report_at(
        REPORT_TYPE_TEST,
        &["extra"],
        "Server thread",
        "2026-06-07 12:34:56",
        0,
    );

    assert!(friendly.starts_with(
            "---- Minecraft Test Report ----\n// Don't mind me\n// extra\n\nTime: 2026-06-07 12:34:56\nDescription: Loading world\n\njava.lang.RuntimeException: boom\n\tat game.Main.load(Main.java:8)\n\n\nA detailed walkthrough of the error, its code path and all known details is as follows:\n---------------------------------------------------------------------------------------\n\n-- System Details --\nDetails:\n\tMinecraft Version: 26.1.2"
        ));
}

#[test]
fn crash_report_save_to_file_matches_java_save_once_contract() {
    let mut report = CrashReportModel::new(
        "Save me",
        JavaThrowableModel::new("RuntimeException", Some("boom")),
        SystemReportModel::empty(),
    );
    let dir = std::env::temp_dir().join(format!(
        "vibecraft-java-crash-report-{}",
        std::process::id()
    ));
    let file = dir.join("nested").join("crash.txt");
    let _ = fs::remove_dir_all(&dir);

    assert!(report.save_to_file_at(
        &file,
        REPORT_TYPE_TEST,
        &[],
        "Server thread",
        "2026-06-07 12:34:56",
        0,
    ));
    assert_eq!(report.get_save_file(), Some(file.as_path()));
    assert!(!report.save_to_file_at(
        &file,
        REPORT_TYPE_TEST,
        &[],
        "Server thread",
        "2026-06-07 12:34:56",
        0,
    ));
    assert!(fs::read_to_string(&file)
        .map(|contents| contents.contains("Description: Save me"))
        .unwrap_or(false));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn crash_report_add_category_tracks_uncategorized_stack_like_java() {
    let exception_stack = vec![
        JavaStackTraceElementModel::new("game.Head", "run", "Head.java", 1),
        JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 2),
        JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 3),
    ];
    let current_thread_stack = vec![
        JavaStackTraceElementModel::new("java.lang.Thread", "getStackTrace", "Thread.java", 1),
        JavaStackTraceElementModel::new(
            "net.minecraft.CrashReportCategory",
            "fillInStackTrace",
            "CrashReportCategory.java",
            2,
        ),
        JavaStackTraceElementModel::new(
            "net.minecraft.CrashReport",
            "addCategory",
            "CrashReport.java",
            3,
        ),
        JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 2),
        JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 3),
    ];
    let mut report = CrashReportModel::new(
        "Tick",
        JavaThrowableModel::new("RuntimeException", Some("bad")).with_stack_trace(exception_stack),
        SystemReportModel::empty(),
    );

    let category = report.add_category_from_thread_trace("Ticking block", 0, &current_thread_stack);
    assert_eq!(
        category.stacktrace(),
        &[
            JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 2),
            JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 3),
        ]
    );
    assert_eq!(
        report.uncategorized_stack_trace,
        Some(vec![JavaStackTraceElementModel::new(
            "game.Head",
            "run",
            "Head.java",
            1
        )])
    );
}

#[test]
fn crash_report_for_throwable_unwraps_completion_and_reuses_reported_exception() {
    let cause = JavaThrowableModel::new("IllegalArgumentException", Some("bad"));
    let report = CrashReportModel::for_throwable(
        JavaThrowableModel::completion_exception(cause.clone()),
        "Outer",
        SystemReportModel::empty(),
    );
    assert_eq!(report.get_title(), "Outer");
    assert_eq!(report.get_exception(), &cause);

    let reference = super::CrashReportReferenceModel::new("Existing", cause.clone());
    let report = CrashReportModel::for_throwable(
        JavaThrowableModel::reported_exception(reference),
        "Ignored",
        SystemReportModel::empty(),
    );
    assert_eq!(report.get_title(), "Existing");
    assert_eq!(report.get_exception(), &cause);
}

#[test]
fn crash_report_preload_matches_java_memory_reserve_and_report_build() {
    assert_eq!(
        CrashReportModel::preload_actions(),
        vec![
            CrashReportPreloadAction::AllocateMemoryReserve,
            CrashReportPreloadAction::BuildFriendlyReport {
                title: "Don't panic!",
            },
        ]
    );
}

#[test]
fn default_uncaught_exception_handler_matches_java_single_logger_call() {
    let throwable = JavaThrowableModel::new("IllegalStateException", Some("boom"));
    let exception = super::JavaUncaughtExceptionModel::new("Server thread", throwable.clone());

    assert_eq!(
        super::default_uncaught_exception_handler_actions(exception),
        vec![super::JavaUncaughtExceptionLogAction::WithThrowable {
            message: "Caught previously unhandled exception :",
            throwable,
        }]
    );
}

#[test]
fn default_uncaught_exception_handler_with_name_matches_java_two_logger_calls() {
    let throwable = JavaThrowableModel::new("RuntimeException", Some("bad tick"));
    let exception = super::JavaUncaughtExceptionModel::new("Worker-1", throwable.clone());

    assert_eq!(
        super::default_uncaught_exception_handler_with_name_actions(exception),
        vec![
            super::JavaUncaughtExceptionLogAction::Message {
                message: "Caught previously unhandled exception :",
            },
            super::JavaUncaughtExceptionLogAction::ThreadThrowable {
                thread_name: "Worker-1".to_string(),
                throwable,
            },
        ]
    );
}

#[test]
fn reported_exception_delegates_report_cause_and_message_like_java() {
    let throwable = JavaThrowableModel::new("IllegalArgumentException", Some("bad id"));
    let report = super::CrashReportReferenceModel::new("Loading registry", throwable.clone());
    let reported = super::ReportedExceptionModel::new(report.clone());

    assert_eq!(reported.get_report(), &report);
    assert_eq!(reported.get_cause(), &throwable);
    assert_eq!(reported.get_message(), "Loading registry");
}
