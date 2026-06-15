#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopCommandOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub broadcast_to_admins: bool,
    pub halt_requested: bool,
    pub halt_prevent_crash_report: bool,
}

pub fn execute_stop_command() -> StopCommandOutput {
    StopCommandOutput {
        success_count: 1,
        feedback_key: "commands.stop.stopping",
        broadcast_to_admins: true,
        halt_requested: true,
        halt_prevent_crash_report: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const STOP_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/StopCommand.java");

    #[test]
    fn stop_command_runtime_effects_match_java() {
        assert_eq!(
            execute_stop_command(),
            StopCommandOutput {
                success_count: 1,
                feedback_key: "commands.stop.stopping",
                broadcast_to_admins: true,
                halt_requested: true,
                halt_prevent_crash_report: false,
            }
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn stop_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"stop\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_OWNERS))",
            "sendSuccess(() -> Component.translatable(\"commands.stop.stopping\"), true)",
            "getServer().halt(false)",
            "return 1;",
        ] {
            assert!(
                STOP_COMMAND_JAVA.contains(sentinel),
                "StopCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
