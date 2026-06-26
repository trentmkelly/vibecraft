#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveToggleCommand {
    Off,
    On,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveToggleError {
    AlreadyOff,
    AlreadyOn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveToggleOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub autosave_enabled: bool,
    pub broadcast_to_admins: bool,
}

pub fn execute_save_toggle(
    command: SaveToggleCommand,
    autosave_enabled: bool,
) -> Result<SaveToggleOutput, SaveToggleError> {
    match command {
        SaveToggleCommand::Off if !autosave_enabled => Err(SaveToggleError::AlreadyOff),
        SaveToggleCommand::Off => Ok(SaveToggleOutput {
            success_count: 1,
            feedback_key: "commands.save.disabled",
            autosave_enabled: false,
            broadcast_to_admins: true,
        }),
        SaveToggleCommand::On if autosave_enabled => Err(SaveToggleError::AlreadyOn),
        SaveToggleCommand::On => Ok(SaveToggleOutput {
            success_count: 1,
            feedback_key: "commands.save.enabled",
            autosave_enabled: true,
            broadcast_to_admins: true,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const SAVE_ALL_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SaveAllCommand.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const SAVE_OFF_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SaveOffCommand.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const SAVE_ON_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SaveOnCommand.java");

    #[test]
    fn save_off_matches_java_state_error_and_feedback() {
        assert_eq!(
            execute_save_toggle(SaveToggleCommand::Off, false),
            Err(SaveToggleError::AlreadyOff)
        );
        assert_eq!(
            execute_save_toggle(SaveToggleCommand::Off, true),
            Ok(SaveToggleOutput {
                success_count: 1,
                feedback_key: "commands.save.disabled",
                autosave_enabled: false,
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    fn save_on_matches_java_state_error_and_feedback() {
        assert_eq!(
            execute_save_toggle(SaveToggleCommand::On, true),
            Err(SaveToggleError::AlreadyOn)
        );
        assert_eq!(
            execute_save_toggle(SaveToggleCommand::On, false),
            Ok(SaveToggleOutput {
                success_count: 1,
                feedback_key: "commands.save.enabled",
                autosave_enabled: true,
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn save_all_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"save-all\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_OWNERS))",
            ".executes(c -> saveAll((CommandSourceStack)c.getSource(), false))",
            "Commands.literal(\"flush\").executes(c -> saveAll((CommandSourceStack)c.getSource(), true))",
            "source.sendSuccess(() -> Component.translatable(\"commands.save.saving\"), false);",
            "boolean success = server.saveEverything(true, flush, true);",
            "throw ERROR_FAILED.create();",
            "source.sendSuccess(() -> Component.translatable(\"commands.save.success\"), true);",
            "return 1;",
        ] {
            assert!(
                SAVE_ALL_COMMAND_JAVA.contains(sentinel),
                "SaveAllCommand.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn save_off_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"save-off\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_OWNERS))",
            "boolean success = source.getServer().setAutoSave(false);",
            "throw ERROR_ALREADY_OFF.create();",
            "Component.translatable(\"commands.save.disabled\")",
            "return 1;",
        ] {
            assert!(
                SAVE_OFF_COMMAND_JAVA.contains(sentinel),
                "SaveOffCommand.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn save_on_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"save-on\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_OWNERS))",
            "boolean success = source.getServer().setAutoSave(true);",
            "throw ERROR_ALREADY_ON.create();",
            "Component.translatable(\"commands.save.enabled\")",
            "return 1;",
        ] {
            assert!(
                SAVE_ON_COMMAND_JAVA.contains(sentinel),
                "SaveOnCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
