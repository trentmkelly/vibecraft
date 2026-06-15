#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct SetWorldSpawnOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub dimension: String,
    pub position: (i32, i32, i32),
    pub yaw: f32,
    pub pitch: f32,
    pub broadcast_to_admins: bool,
}

pub fn execute_set_world_spawn_command(
    dimension: impl Into<String>,
    position: (i32, i32, i32),
    yaw: f32,
    pitch: f32,
) -> SetWorldSpawnOutput {
    SetWorldSpawnOutput {
        success_count: 1,
        feedback_key: "commands.setworldspawn.success",
        dimension: dimension.into(),
        position,
        yaw,
        pitch,
        broadcast_to_admins: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const SET_WORLD_SPAWN_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SetWorldSpawnCommand.java");

    #[test]
    fn set_world_spawn_preserves_raw_rotation_and_broadcasts_success() {
        assert_eq!(
            execute_set_world_spawn_command("minecraft:the_nether", (1, 70, 2), 270.0, -120.0),
            SetWorldSpawnOutput {
                success_count: 1,
                feedback_key: "commands.setworldspawn.success",
                dimension: "minecraft:the_nether".to_string(),
                position: (1, 70, 2),
                yaw: 270.0,
                pitch: -120.0,
                broadcast_to_admins: true,
            }
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn set_world_spawn_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"setworldspawn\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "BlockPos.containing(((CommandSourceStack)c.getSource()).getPosition())",
            "Commands.argument(\"pos\", BlockPosArgument.blockPos())",
            "BlockPosArgument.getSpawnablePos(c, \"pos\")",
            "Commands.argument(\"rotation\", RotationArgument.rotation())",
            "RotationArgument.getRotation(c, \"rotation\")",
            "Vec2 rotationVector = rotation.getRotation(source);",
            "float yaw = rotationVector.y;",
            "float pitch = rotationVector.x;",
            "LevelData.RespawnData.of(level.dimension(), pos, yaw, pitch)",
            "level.setRespawnData(respawnData);",
            "Component.translatable(\n            \"commands.setworldspawn.success\"",
            "respawnData.yaw()",
            "respawnData.pitch()",
            "level.dimension().identifier().toString()",
            "return 1;",
        ] {
            assert!(
                SET_WORLD_SPAWN_COMMAND_JAVA.contains(sentinel),
                "SetWorldSpawnCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
