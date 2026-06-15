#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectatePlayerMode {
    Spectator,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpectateError {
    SelfTarget,
    NotSpectator,
    CannotSpectate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpectateOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub target: Option<String>,
    pub broadcast_to_admins: bool,
}

pub fn execute_spectate_command(
    player: &str,
    mode: SpectatePlayerMode,
    target: Option<&str>,
    target_tracking_range: Option<i32>,
) -> Result<SpectateOutput, SpectateError> {
    if target.is_some_and(|target| target == player) {
        return Err(SpectateError::SelfTarget);
    }
    if mode != SpectatePlayerMode::Spectator {
        return Err(SpectateError::NotSpectator);
    }
    if target_tracking_range == Some(0) {
        return Err(SpectateError::CannotSpectate);
    }

    Ok(SpectateOutput {
        success_count: 1,
        feedback_key: if target.is_some() {
            "commands.spectate.success.started"
        } else {
            "commands.spectate.success.stopped"
        },
        target: target.map(str::to_string),
        broadcast_to_admins: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const SPECTATE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SpectateCommand.java");

    #[test]
    fn spectate_command_models_java_validation_order_and_feedback() {
        assert_eq!(
            execute_spectate_command("Steve", SpectatePlayerMode::Spectator, Some("Steve"), None),
            Err(SpectateError::SelfTarget)
        );
        assert_eq!(
            execute_spectate_command("Steve", SpectatePlayerMode::Other, Some("Pig"), None),
            Err(SpectateError::NotSpectator)
        );
        assert_eq!(
            execute_spectate_command(
                "Steve",
                SpectatePlayerMode::Spectator,
                Some("Marker"),
                Some(0)
            ),
            Err(SpectateError::CannotSpectate)
        );
        assert_eq!(
            execute_spectate_command("Steve", SpectatePlayerMode::Spectator, Some("Pig"), Some(8)),
            Ok(SpectateOutput {
                success_count: 1,
                feedback_key: "commands.spectate.success.started",
                target: Some("Pig".to_string()),
                broadcast_to_admins: false,
            })
        );
        assert_eq!(
            execute_spectate_command("Steve", SpectatePlayerMode::Spectator, None, None),
            Ok(SpectateOutput {
                success_count: 1,
                feedback_key: "commands.spectate.success.stopped",
                target: None,
                broadcast_to_admins: false,
            })
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn spectate_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"spectate\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "spectate((CommandSourceStack)c.getSource(), null, ((CommandSourceStack)c.getSource()).getPlayerOrException())",
            "Commands.argument(\"target\", EntityArgument.entity())",
            "EntityArgument.getEntity(c, \"target\")",
            "Commands.argument(\"player\", EntityArgument.player())",
            "EntityArgument.getPlayer(c, \"player\")",
            "if (player == target)",
            "throw ERROR_SELF.create();",
            "if (!player.isSpectator())",
            "throw ERROR_NOT_SPECTATOR.create(player.getDisplayName());",
            "target != null && target.getType().clientTrackingRange() == 0",
            "throw ERROR_CANNOT_SPECTATE.create(target.getDisplayName());",
            "player.setCamera(target);",
            "Component.translatable(\"commands.spectate.success.started\", target.getDisplayName())",
            "Component.translatable(\"commands.spectate.success.stopped\")",
            "return 1;",
        ] {
            assert!(
                SPECTATE_COMMAND_JAVA.contains(sentinel),
                "SpectateCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
