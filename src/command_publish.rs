#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishGameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishError {
    AlreadyPublished { port: u16 },
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishCommandOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub copy_text: String,
    pub port: u16,
    pub allow_commands: bool,
    pub gamemode: Option<PublishGameMode>,
    pub broadcast_to_admins: bool,
}

pub fn execute_publish_command(
    current_server_port: Option<u16>,
    publish_should_fail: bool,
    port: u16,
    allow_commands: bool,
    gamemode: Option<PublishGameMode>,
) -> Result<PublishCommandOutput, PublishError> {
    if let Some(port) = current_server_port {
        return Err(PublishError::AlreadyPublished { port });
    }
    if publish_should_fail {
        return Err(PublishError::Failed);
    }

    Ok(PublishCommandOutput {
        success_count: i32::from(port),
        feedback_key: "commands.publish.started",
        copy_text: port.to_string(),
        port,
        allow_commands,
        gamemode,
        broadcast_to_admins: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const PUBLISH_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/PublishCommand.java");

    #[test]
    fn publish_success_records_port_options_and_copyable_port_text() {
        assert_eq!(
            execute_publish_command(None, false, 24454, false, None),
            Ok(PublishCommandOutput {
                success_count: 24454,
                feedback_key: "commands.publish.started",
                copy_text: "24454".to_string(),
                port: 24454,
                allow_commands: false,
                gamemode: None,
                broadcast_to_admins: true,
            })
        );
        assert_eq!(
            execute_publish_command(
                None,
                false,
                0,
                true,
                Some(PublishGameMode::Creative)
            ),
            Ok(PublishCommandOutput {
                success_count: 0,
                feedback_key: "commands.publish.started",
                copy_text: "0".to_string(),
                port: 0,
                allow_commands: true,
                gamemode: Some(PublishGameMode::Creative),
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    fn publish_errors_match_java_order() {
        assert_eq!(
            execute_publish_command(Some(25565), true, 24454, false, None),
            Err(PublishError::AlreadyPublished { port: 25565 })
        );
        assert_eq!(
            execute_publish_command(None, true, 24454, false, None),
            Err(PublishError::Failed)
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn publish_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"publish\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_OWNERS))",
            "HttpUtil.getAvailablePort()",
            "Commands.argument(\"allowCommands\", BoolArgumentType.bool())",
            "Commands.argument(\"gamemode\", GameModeArgument.gameMode())",
            "Commands.argument(\"port\", IntegerArgumentType.integer(0, 65535))",
            "if (source.getServer().isPublished())",
            "ERROR_ALREADY_PUBLISHED.create(source.getServer().getPort())",
            "if (!source.getServer().publishServer(type, allowCommands, port))",
            "throw ERROR_FAILED.create();",
            "source.sendSuccess(() -> getSuccessMessage(port), true)",
            "ComponentUtils.copyOnClickText(String.valueOf(port))",
            "Component.translatable(\"commands.publish.started\", portText)",
            "return port;",
        ] {
            assert!(
                PUBLISH_COMMAND_JAVA.contains(sentinel),
                "PublishCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
