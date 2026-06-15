#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugPathSourceKind {
    Mob,
    Player,
    NonLiving,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugPathError {
    NotMob,
    NoPath,
    NotComplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugPathOutput {
    pub success_count: i32,
    pub feedback_marker: &'static str,
    pub target: (i32, i32, i32),
    pub broadcast_to_admins: bool,
}

pub fn execute_debug_path_command(
    source_kind: DebugPathSourceKind,
    target: (i32, i32, i32),
    path_exists: bool,
    path_can_reach: bool,
) -> Result<DebugPathOutput, DebugPathError> {
    if source_kind != DebugPathSourceKind::Mob {
        return Err(DebugPathError::NotMob);
    }
    if !path_exists {
        return Err(DebugPathError::NoPath);
    }
    if !path_can_reach {
        return Err(DebugPathError::NotComplete);
    }

    Ok(DebugPathOutput {
        success_count: 1,
        feedback_marker: crate::command::DEBUG_PATH_SUCCESS_FEEDBACK,
        target,
        broadcast_to_admins: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const DEBUG_PATH_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/DebugPathCommand.java");

    #[test]
    fn debug_path_requires_mob_path_and_complete_reachability() {
        assert_eq!(
            execute_debug_path_command(DebugPathSourceKind::Missing, (1, 64, 1), true, true),
            Err(DebugPathError::NotMob)
        );
        assert_eq!(
            execute_debug_path_command(DebugPathSourceKind::Player, (1, 64, 1), true, true),
            Err(DebugPathError::NotMob)
        );
        assert_eq!(
            execute_debug_path_command(DebugPathSourceKind::Mob, (1, 64, 1), false, true),
            Err(DebugPathError::NoPath)
        );
        assert_eq!(
            execute_debug_path_command(DebugPathSourceKind::Mob, (1, 64, 1), true, false),
            Err(DebugPathError::NotComplete)
        );
        assert_eq!(
            execute_debug_path_command(DebugPathSourceKind::Mob, (1, 64, 1), true, true),
            Ok(DebugPathOutput {
                success_count: 1,
                feedback_marker: crate::command::DEBUG_PATH_SUCCESS_FEEDBACK,
                target: (1, 64, 1),
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn debug_path_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"debugpath\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "Commands.argument(\"to\", BlockPosArgument.blockPos())",
            "BlockPosArgument.getLoadedBlockPos(c, \"to\")",
            "if (!(source.getEntity() instanceof Mob mob))",
            "new GroundPathNavigation(mob, source.getLevel())",
            "Path path = pathNavigation.createPath(target, 0);",
            "if (path == null)",
            "throw ERROR_NO_PATH.create();",
            "if (!path.canReach())",
            "throw ERROR_NOT_COMPLETE.create();",
            "source.sendSuccess(() -> Component.literal(\"Made path\"), true);",
            "return 1;",
        ] {
            assert!(
                DEBUG_PATH_COMMAND_JAVA.contains(sentinel),
                "DebugPathCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
