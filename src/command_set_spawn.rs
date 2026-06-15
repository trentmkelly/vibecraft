#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSpawnOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub dimension: String,
    pub position: (i32, i32, i32),
    pub yaw: f32,
    pub pitch: f32,
    pub forced: bool,
    pub broadcast_to_admins: bool,
}

pub fn wrap_degrees(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

pub fn execute_spawnpoint_command(
    dimension: impl Into<String>,
    target_count: i32,
    position: (i32, i32, i32),
    yaw: f32,
    pitch: f32,
) -> PlayerSpawnOutput {
    PlayerSpawnOutput {
        success_count: target_count,
        feedback_key: if target_count == 1 {
            "commands.spawnpoint.success.single"
        } else {
            "commands.spawnpoint.success.multiple"
        },
        dimension: dimension.into(),
        position,
        yaw: wrap_degrees(yaw),
        pitch: pitch.clamp(-90.0, 90.0),
        forced: true,
        broadcast_to_admins: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const SET_SPAWN_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SetSpawnCommand.java");

    #[test]
    fn spawnpoint_wraps_yaw_clamps_pitch_and_selects_feedback_by_target_count() {
        assert_eq!(
            execute_spawnpoint_command("minecraft:overworld", 1, (1, 70, 2), 270.0, -120.0),
            PlayerSpawnOutput {
                success_count: 1,
                feedback_key: "commands.spawnpoint.success.single",
                dimension: "minecraft:overworld".to_string(),
                position: (1, 70, 2),
                yaw: -90.0,
                pitch: -90.0,
                forced: true,
                broadcast_to_admins: true,
            }
        );
        assert_eq!(
            execute_spawnpoint_command("minecraft:overworld", 2, (1, 70, 2), -181.0, 120.0),
            PlayerSpawnOutput {
                success_count: 2,
                feedback_key: "commands.spawnpoint.success.multiple",
                dimension: "minecraft:overworld".to_string(),
                position: (1, 70, 2),
                yaw: 179.0,
                pitch: 90.0,
                forced: true,
                broadcast_to_admins: true,
            }
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn spawnpoint_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"spawnpoint\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "Collections.singleton(((CommandSourceStack)c.getSource()).getPlayerOrException())",
            "Commands.argument(\"targets\", EntityArgument.players())",
            "EntityArgument.getPlayers(c, \"targets\")",
            "BlockPos.containing(((CommandSourceStack)c.getSource()).getPosition())",
            "Commands.argument(\"pos\", BlockPosArgument.blockPos())",
            "BlockPosArgument.getSpawnablePos(c, \"pos\")",
            "Commands.argument(\"rotation\", RotationArgument.rotation())",
            "RotationArgument.getRotation(c, \"rotation\")",
            "ResourceKey<Level> dimension = source.getLevel().dimension();",
            "float yaw = Mth.wrapDegrees(rotationVector.y);",
            "float pitch = Mth.clamp(rotationVector.x, -90.0F, 90.0F);",
            "target.setRespawnPosition(new ServerPlayer.RespawnConfig(LevelData.RespawnData.of(dimension, pos, yaw, pitch), true), false);",
            "Component.translatable(\n               \"commands.spawnpoint.success.single\"",
            "Component.translatable(\"commands.spawnpoint.success.multiple\"",
            "return targets.size();",
        ] {
            assert!(
                SET_SPAWN_COMMAND_JAVA.contains(sentinel),
                "SetSpawnCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
