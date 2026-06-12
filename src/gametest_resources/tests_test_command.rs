use super::*;

const TEST_COMMAND_JAVA: &str = include_str!(
    "../../../decompiled-server-26.1.2/net/minecraft/gametest/framework/TestCommand.java"
);

#[test]
fn test_command_matches_java_constants_and_command_surface() {
    assert_eq!(TEST_COMMAND_JAVA.lines().count(), 647);
    for sentinel in [
        "public static final int TEST_NEARBY_SEARCH_RADIUS = 15;",
        "public static final int TEST_FULL_SEARCH_RADIUS = 250;",
        "public static final int VERIFY_TEST_GRID_AXIS_SIZE = 10;",
        "public static final int VERIFY_TEST_BATCH_SIZE = 100;",
        "private static final int DEFAULT_CLEAR_RADIUS = 250;",
        "private static final int MAX_CLEAR_RADIUS = 1024;",
        "private static final int TEST_POS_Z_OFFSET_FROM_PLAYER = 3;",
        "private static final int DEFAULT_X_SIZE = 5;",
        "private static final int DEFAULT_Y_SIZE = 5;",
        "private static final int DEFAULT_Z_SIZE = 5;",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "if (SharedConstants.IS_RUNNING_IN_IDE)",
    ] {
        assert!(
            TEST_COMMAND_JAVA.contains(sentinel),
            "missing TestCommand sentinel {sentinel}"
        );
    }
    assert!(TEST_COMMAND_JAVA.contains("Commands.literal(\n                                                               \"test\""));

    let surface = test_command_surface();
    assert_eq!(surface.root_literal, "test");
    assert_eq!(surface.permission_level, "LEVEL_GAMEMASTERS");
    assert_eq!(surface.constants, TEST_COMMAND_CONSTANTS);
    assert_eq!(surface.subcommands, TEST_COMMAND_SUBCOMMANDS);
    assert_eq!(surface.ide_only_subcommands, TEST_COMMAND_IDE_ONLY_SUBCOMMANDS);
    assert_eq!(surface.deferred_live_runtime, "gametest-command-live-runtime");
}

#[test]
fn test_command_tracks_java_subcommands_exceptions_and_runtime_hooks() {
    for command in TEST_COMMAND_SUBCOMMANDS
        .iter()
        .chain(TEST_COMMAND_IDE_ONLY_SUBCOMMANDS)
    {
        assert!(
            TEST_COMMAND_JAVA.contains(&format!("Commands.literal(\"{command}\")")),
            "missing TestCommand literal {command}"
        );
    }
    for key in TEST_COMMAND_EXCEPTION_KEYS {
        assert!(
            TEST_COMMAND_JAVA.contains(key),
            "missing TestCommand exception key {key}"
        );
    }
    for sentinel in [
        "GameTestTicker.SINGLETON.clear();",
        "runner.addListener(new TestCommand.TestBatchSummaryDisplayer(source));",
        "tracker.addListener(new TestCommand.TestSummaryDisplayer(source, tracker));",
        "tracker.addFailureListener(testInfo -> FailedTestTracker.rememberFailedTest(testInfo.getTestHolder()));",
        "new StructureGridSpawner(testPos, 10, true);",
        "new RetryOptions(1, true)",
        "StructureUtils.createNewEmptyTest(id, testPos, new Vec3i(xSize, ySize, zSize), Rotation.NONE, level);",
        "player.connection.send(new ClientboundGameTestHighlightPosPacket(targetPosAbsolute, targetPosRelative));",
    ] {
        assert!(
            TEST_COMMAND_JAVA.contains(sentinel),
            "missing TestCommand runtime sentinel {sentinel}"
        );
    }
}
