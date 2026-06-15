#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetIdleTimeoutError {
    InvalidMinutes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetIdleTimeoutOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub idle_timeout_minutes: u32,
    pub broadcast_to_admins: bool,
}

pub fn execute_set_idle_timeout(minutes: i32) -> Result<SetIdleTimeoutOutput, SetIdleTimeoutError> {
    if minutes < 0 {
        return Err(SetIdleTimeoutError::InvalidMinutes);
    }

    Ok(SetIdleTimeoutOutput {
        success_count: minutes,
        feedback_key: if minutes > 0 {
            "commands.setidletimeout.success"
        } else {
            "commands.setidletimeout.success.disabled"
        },
        idle_timeout_minutes: minutes as u32,
        broadcast_to_admins: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const SET_PLAYER_IDLE_TIMEOUT_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SetPlayerIdleTimeoutCommand.java");

    #[test]
    fn idle_timeout_rejects_negative_values_like_integer_argument_min_zero() {
        assert_eq!(
            execute_set_idle_timeout(-1),
            Err(SetIdleTimeoutError::InvalidMinutes)
        );
    }

    #[test]
    fn idle_timeout_zero_disables_timeout_with_java_feedback() {
        assert_eq!(
            execute_set_idle_timeout(0),
            Ok(SetIdleTimeoutOutput {
                success_count: 0,
                feedback_key: "commands.setidletimeout.success.disabled",
                idle_timeout_minutes: 0,
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    fn idle_timeout_positive_sets_minutes_and_returns_time() {
        assert_eq!(
            execute_set_idle_timeout(5),
            Ok(SetIdleTimeoutOutput {
                success_count: 5,
                feedback_key: "commands.setidletimeout.success",
                idle_timeout_minutes: 5,
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn set_player_idle_timeout_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"setidletimeout\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
            "Commands.argument(\"minutes\", IntegerArgumentType.integer(0))",
            "setIdleTimeout((CommandSourceStack)c.getSource(), IntegerArgumentType.getInteger(c, \"minutes\"))",
            "source.getServer().setPlayerIdleTimeout(time);",
            "if (time > 0)",
            "Component.translatable(\"commands.setidletimeout.success\", time)",
            "Component.translatable(\"commands.setidletimeout.success.disabled\")",
            "return time;",
        ] {
            assert!(
                SET_PLAYER_IDLE_TIMEOUT_COMMAND_JAVA.contains(sentinel),
                "SetPlayerIdleTimeoutCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
