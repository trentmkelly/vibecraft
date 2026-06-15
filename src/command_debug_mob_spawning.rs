#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugMobSpawningOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub category: &'static str,
    pub position: (i32, i32, i32),
    pub broadcast_to_admins: bool,
}

pub fn execute_debug_mob_spawning_command(
    category: &'static str,
    position: (i32, i32, i32),
) -> DebugMobSpawningOutput {
    DebugMobSpawningOutput {
        success_count: 1,
        feedback_key: crate::command::NO_COMMAND_FEEDBACK,
        category,
        position,
        broadcast_to_admins: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const DEBUG_MOB_SPAWNING_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/DebugMobSpawningCommand.java");

    #[test]
    fn debug_mob_spawning_records_category_position_and_no_feedback() {
        assert_eq!(
            execute_debug_mob_spawning_command("monster", (0, 64, 0)),
            DebugMobSpawningOutput {
                success_count: 1,
                feedback_key: crate::command::NO_COMMAND_FEEDBACK,
                category: "monster",
                position: (0, 64, 0),
                broadcast_to_admins: false,
            }
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn debug_mob_spawning_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"debugmobspawning\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "for (MobCategory mobCategory : MobCategory.values())",
            "Commands.literal(mobCategory.getName())",
            "Commands.argument(\"at\", BlockPosArgument.blockPos())",
            "BlockPosArgument.getLoadedBlockPos(c, \"at\")",
            "NaturalSpawner.spawnCategoryForPosition(mobCategory, source.getLevel(), at);",
            "return 1;",
        ] {
            assert!(
                DEBUG_MOB_SPAWNING_COMMAND_JAVA.contains(sentinel),
                "DebugMobSpawningCommand.java is missing sentinel: {sentinel}"
            );
        }
        assert!(
            !DEBUG_MOB_SPAWNING_COMMAND_JAVA.contains("sendSuccess"),
            "DebugMobSpawningCommand.java should not send command feedback"
        );
    }
}
