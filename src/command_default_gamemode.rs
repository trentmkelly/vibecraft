#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultGameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultGameModeOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub default_game_mode: DefaultGameMode,
    pub enforced_player_count: i32,
    pub broadcast_to_admins: bool,
}

pub fn execute_default_gamemode_command(
    requested_mode: DefaultGameMode,
    forced_mode_after_default_update: Option<DefaultGameMode>,
    online_player_count: i32,
) -> DefaultGameModeOutput {
    let enforced_player_count = if forced_mode_after_default_update.is_some() {
        online_player_count
    } else {
        0
    };

    DefaultGameModeOutput {
        success_count: enforced_player_count,
        feedback_key: "commands.defaultgamemode.success",
        default_game_mode: requested_mode,
        enforced_player_count,
        broadcast_to_admins: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const DEFAULT_GAMEMODE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/DefaultGameModeCommands.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const MINECRAFT_SERVER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/MinecraftServer.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const DEDICATED_SERVER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/dedicated/DedicatedServer.java");

    #[test]
    fn default_gamemode_sets_world_default_and_returns_enforced_player_count() {
        assert_eq!(
            execute_default_gamemode_command(DefaultGameMode::Creative, None, 3),
            DefaultGameModeOutput {
                success_count: 0,
                feedback_key: "commands.defaultgamemode.success",
                default_game_mode: DefaultGameMode::Creative,
                enforced_player_count: 0,
                broadcast_to_admins: true,
            }
        );
        assert_eq!(
            execute_default_gamemode_command(
                DefaultGameMode::Creative,
                Some(DefaultGameMode::Creative),
                3
            ),
            DefaultGameModeOutput {
                success_count: 3,
                feedback_key: "commands.defaultgamemode.success",
                default_game_mode: DefaultGameMode::Creative,
                enforced_player_count: 3,
                broadcast_to_admins: true,
            }
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn default_gamemode_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"defaultgamemode\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
            "Commands.argument(\"gamemode\", GameModeArgument.gameMode())",
            "server.setDefaultGameType(type);",
            "int count = server.enforceGameTypeForPlayers(server.getForcedGameType());",
            "Component.translatable(\"commands.defaultgamemode.success\", type.getLongDisplayName())",
            "return count;",
        ] {
            assert!(
                DEFAULT_GAMEMODE_COMMAND_JAVA.contains(sentinel),
                "DefaultGameModeCommands.java is missing sentinel: {sentinel}"
            );
        }
        for sentinel in [
            "public void setDefaultGameType(final GameType gameType)",
            "this.worldData.setGameType(gameType);",
            "public int enforceGameTypeForPlayers(final @Nullable GameType gameType)",
            "if (gameType == null) {\n         return 0;",
            "if (player.setGameMode(gameType))",
        ] {
            assert!(
                MINECRAFT_SERVER_JAVA.contains(sentinel),
                "MinecraftServer.java is missing default-gamemode sentinel: {sentinel}"
            );
        }
        assert!(
            DEDICATED_SERVER_JAVA
                .contains("return this.forceGameMode() ? this.worldData.getGameType() : null;"),
            "DedicatedServer.java should derive forced game type from force-gamemode"
        );
    }
}
