#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestCommandConstants {
    pub nearby_search_radius: i32,
    pub full_search_radius: i32,
    pub verify_grid_axis_size: i32,
    pub verify_batch_size: i32,
    pub default_clear_radius: i32,
    pub max_clear_radius: i32,
    pub test_pos_z_offset_from_player: i32,
    pub default_x_size: i32,
    pub default_y_size: i32,
    pub default_z_size: i32,
    pub max_created_structure_axis_size: i32,
}

pub const TEST_COMMAND_CONSTANTS: TestCommandConstants = TestCommandConstants {
    nearby_search_radius: 15,
    full_search_radius: 250,
    verify_grid_axis_size: 10,
    verify_batch_size: 100,
    default_clear_radius: 250,
    max_clear_radius: 1024,
    test_pos_z_offset_from_player: 3,
    default_x_size: 5,
    default_y_size: 5,
    default_z_size: 5,
    max_created_structure_axis_size: 48,
};

pub const TEST_COMMAND_PERMISSION_LEVEL: &str = "LEVEL_GAMEMASTERS";

pub const TEST_COMMAND_SUBCOMMANDS: &[&str] = &[
    "run",
    "runmultiple",
    "runthese",
    "runclosest",
    "runthat",
    "runfailed",
    "verify",
    "locate",
    "resetclosest",
    "resetthese",
    "resetthat",
    "clearthat",
    "clearthese",
    "clearall",
    "stop",
    "pos",
    "create",
];

pub const TEST_COMMAND_IDE_ONLY_SUBCOMMANDS: &[&str] =
    &["export", "exportclosest", "exportthese", "exportthat"];

pub const TEST_COMMAND_EXCEPTION_KEYS: &[&str] = &[
    "commands.test.clear.error.no_tests",
    "commands.test.reset.error.no_tests",
    "commands.test.error.test_instance_not_found",
    "Could not find any structures to export",
    "commands.test.error.no_test_instances",
    "commands.test.error.no_test_containing_pos",
    "commands.test.error.too_large",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCommandSurface {
    pub root_literal: &'static str,
    pub permission_level: &'static str,
    pub constants: TestCommandConstants,
    pub subcommands: &'static [&'static str],
    pub ide_only_subcommands: &'static [&'static str],
    pub exception_keys: &'static [&'static str],
    pub deferred_live_runtime: &'static str,
}

// TODO(gametest-command-live-runtime): replace this command-surface model with
// Brigadier registration and live GameTest runner/world mutation once command
// execution is wired to the server-backed GameTest runtime.
pub fn test_command_surface() -> TestCommandSurface {
    TestCommandSurface {
        root_literal: "test",
        permission_level: TEST_COMMAND_PERMISSION_LEVEL,
        constants: TEST_COMMAND_CONSTANTS,
        subcommands: TEST_COMMAND_SUBCOMMANDS,
        ide_only_subcommands: TEST_COMMAND_IDE_ONLY_SUBCOMMANDS,
        exception_keys: TEST_COMMAND_EXCEPTION_KEYS,
        deferred_live_runtime: "gametest-command-live-runtime",
    }
}
