#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedPermissionRequirement {
    All,
    Gamemasters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedCommandOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub copy_text: String,
    pub broadcast_to_admins: bool,
}

pub fn seed_permission_requirement(check_permissions: bool) -> SeedPermissionRequirement {
    if check_permissions {
        SeedPermissionRequirement::Gamemasters
    } else {
        SeedPermissionRequirement::All
    }
}

pub fn execute_seed_command(seed: i64) -> SeedCommandOutput {
    SeedCommandOutput {
        success_count: seed as i32,
        feedback_key: "commands.seed.success",
        copy_text: seed.to_string(),
        broadcast_to_admins: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const SEED_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SeedCommand.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const COMMANDS_JAVA: &str = vibecraft_java_source!("/net/minecraft/commands/Commands.java");

    #[test]
    fn seed_permission_requirement_matches_java_check_permissions_parameter() {
        assert_eq!(
            seed_permission_requirement(true),
            SeedPermissionRequirement::Gamemasters
        );
        assert_eq!(
            seed_permission_requirement(false),
            SeedPermissionRequirement::All
        );
    }

    #[test]
    fn seed_command_returns_truncated_int_and_copyable_long_text_like_java() {
        assert_eq!(
            execute_seed_command(8_675_309),
            SeedCommandOutput {
                success_count: 8_675_309,
                feedback_key: "commands.seed.success",
                copy_text: "8675309".to_string(),
                broadcast_to_admins: false,
            }
        );
        assert_eq!(
            execute_seed_command(i32::MAX as i64 + 1),
            SeedCommandOutput {
                success_count: i32::MIN,
                feedback_key: "commands.seed.success",
                copy_text: "2147483648".to_string(),
                broadcast_to_admins: false,
            }
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn seed_command_source_matches_java_26_1_2() {
        for sentinel in [
            "public static void register(final CommandDispatcher<CommandSourceStack> dispatcher, final boolean checkPermissions)",
            "Commands.literal(\"seed\")",
            "Commands.hasPermission(checkPermissions ? Commands.LEVEL_GAMEMASTERS : Commands.LEVEL_ALL)",
            "long seed = ((CommandSourceStack)c.getSource()).getLevel().getSeed();",
            "ComponentUtils.copyOnClickText(String.valueOf(seed))",
            "Component.translatable(\"commands.seed.success\", seedText)",
            "return (int)seed;",
        ] {
            assert!(
                SEED_COMMAND_JAVA.contains(sentinel),
                "SeedCommand.java is missing sentinel: {sentinel}"
            );
        }
        assert!(
            COMMANDS_JAVA.contains(
                "SeedCommand.register(this.dispatcher, commandSelection != Commands.CommandSelection.INTEGRATED);"
            ),
            "Commands.java should pass checkPermissions=false only for integrated command selection"
        );
    }
}
