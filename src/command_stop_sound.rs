#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopSoundSource {
    Master,
    Music,
    Record,
    Weather,
    Block,
    Hostile,
    Neutral,
    Player,
    Ambient,
    Voice,
    Ui,
}

impl StopSoundSource {
    pub fn name(self) -> &'static str {
        match self {
            Self::Master => "master",
            Self::Music => "music",
            Self::Record => "record",
            Self::Weather => "weather",
            Self::Block => "block",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Player => "player",
            Self::Ambient => "ambient",
            Self::Voice => "voice",
            Self::Ui => "ui",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopSoundOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub source: Option<StopSoundSource>,
    pub sound: Option<String>,
    pub broadcast_to_admins: bool,
}

pub fn execute_stop_sound_command(
    target_count: i32,
    source: Option<StopSoundSource>,
    sound: Option<&str>,
) -> StopSoundOutput {
    StopSoundOutput {
        success_count: target_count,
        feedback_key: match (source, sound) {
            (Some(_), Some(_)) => "commands.stopsound.success.source.sound",
            (Some(_), None) => "commands.stopsound.success.source.any",
            (None, Some(_)) => "commands.stopsound.success.sourceless.sound",
            (None, None) => "commands.stopsound.success.sourceless.any",
        },
        source,
        sound: sound.map(str::to_string),
        broadcast_to_admins: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const STOP_SOUND_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/StopSoundCommand.java");

    #[test]
    fn stop_sound_feedback_matches_source_and_sound_filters() {
        assert_eq!(
            execute_stop_sound_command(2, Some(StopSoundSource::Player), Some("minecraft:music.menu")),
            StopSoundOutput {
                success_count: 2,
                feedback_key: "commands.stopsound.success.source.sound",
                source: Some(StopSoundSource::Player),
                sound: Some("minecraft:music.menu".to_string()),
                broadcast_to_admins: true,
            }
        );
        assert_eq!(
            execute_stop_sound_command(1, Some(StopSoundSource::Music), None).feedback_key,
            "commands.stopsound.success.source.any"
        );
        assert_eq!(
            execute_stop_sound_command(1, None, Some("minecraft:music.menu")).feedback_key,
            "commands.stopsound.success.sourceless.sound"
        );
        assert_eq!(
            execute_stop_sound_command(1, None, None).feedback_key,
            "commands.stopsound.success.sourceless.any"
        );
    }

    #[test]
    fn stop_sound_sources_are_exact_java_names() {
        assert_eq!(StopSoundSource::Master.name(), "master");
        assert_eq!(StopSoundSource::Music.name(), "music");
        assert_eq!(StopSoundSource::Record.name(), "record");
        assert_eq!(StopSoundSource::Weather.name(), "weather");
        assert_eq!(StopSoundSource::Block.name(), "block");
        assert_eq!(StopSoundSource::Hostile.name(), "hostile");
        assert_eq!(StopSoundSource::Neutral.name(), "neutral");
        assert_eq!(StopSoundSource::Player.name(), "player");
        assert_eq!(StopSoundSource::Ambient.name(), "ambient");
        assert_eq!(StopSoundSource::Voice.name(), "voice");
        assert_eq!(StopSoundSource::Ui.name(), "ui");
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn stop_sound_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"stopsound\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "Commands.argument(\n               \"targets\", EntityArgument.players()",
            ".executes(c -> stopSound((CommandSourceStack)c.getSource(), EntityArgument.getPlayers(c, \"targets\"), null, null))",
            "Commands.literal(\"*\")",
            "Commands.argument(\"sound\", IdentifierArgument.id())",
            "SuggestionProviders.AVAILABLE_SOUNDS",
            "for (SoundSource source : SoundSource.values())",
            "Commands.literal(source.getName())",
            "new ClientboundStopSoundPacket(sound, soundSource)",
            "player.connection.send(packet);",
            "commands.stopsound.success.source.sound",
            "commands.stopsound.success.source.any",
            "commands.stopsound.success.sourceless.sound",
            "commands.stopsound.success.sourceless.any",
            "return targets.size();",
        ] {
            assert!(
                STOP_SOUND_COMMAND_JAVA.contains(sentinel),
                "StopSoundCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
