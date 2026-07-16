#![allow(dead_code)]

use crate::chat_component::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn id(self) -> i32 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Peaceful => "peaceful",
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    /// Java `ByIdMap.continuous(..., WRAP)` semantics, including negative and
    /// out-of-range ids wrapping around the four enum values.
    pub fn by_id(id: i32) -> Self {
        match id.rem_euclid(4) {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            _ => Self::Hard,
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "peaceful" => Some(Self::Peaceful),
            "easy" => Some(Self::Easy),
            "normal" => Some(Self::Normal),
            "hard" => Some(Self::Hard),
            _ => None,
        }
    }

    pub fn display_name(self) -> Component {
        Component::translatable(format!("options.difficulty.{}", self.serialized_name()), Vec::new())
    }

    pub fn info(self) -> Component {
        Component::translatable(
            format!("options.difficulty.{}.info", self.serialized_name()),
            Vec::new(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DifficultyError {
    AlreadySame { difficulty: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifficultyOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub difficulty: Difficulty,
    pub broadcast_to_admins: bool,
}

pub fn query_difficulty(current: Difficulty) -> DifficultyOutput {
    DifficultyOutput {
        success_count: current.id(),
        feedback_key: "commands.difficulty.query",
        difficulty: current,
        broadcast_to_admins: false,
    }
}

pub fn set_difficulty(
    current: Difficulty,
    requested: Difficulty,
) -> Result<DifficultyOutput, DifficultyError> {
    if current == requested {
        return Err(DifficultyError::AlreadySame {
            difficulty: requested.serialized_name(),
        });
    }

    Ok(DifficultyOutput {
        success_count: 0,
        feedback_key: "commands.difficulty.success",
        difficulty: requested,
        broadcast_to_admins: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const DIFFICULTY_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/DifficultyCommand.java");

    #[test]
    fn difficulty_query_returns_java_ids_without_broadcast() {
        for (difficulty, id) in [
            (Difficulty::Peaceful, 0),
            (Difficulty::Easy, 1),
            (Difficulty::Normal, 2),
            (Difficulty::Hard, 3),
        ] {
            assert_eq!(
                query_difficulty(difficulty),
                DifficultyOutput {
                    success_count: id,
                    feedback_key: "commands.difficulty.query",
                    difficulty,
                    broadcast_to_admins: false,
                }
            );
        }
    }

    #[test]
    fn difficulty_set_rejects_noop_and_broadcasts_changed_value() {
        assert_eq!(
            set_difficulty(Difficulty::Easy, Difficulty::Easy),
            Err(DifficultyError::AlreadySame { difficulty: "easy" })
        );
        assert_eq!(
            set_difficulty(Difficulty::Easy, Difficulty::Hard),
            Ok(DifficultyOutput {
                success_count: 0,
                feedback_key: "commands.difficulty.success",
                difficulty: Difficulty::Hard,
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    fn difficulty_lookup_and_components_match_java_enum_contract() {
        assert_eq!(Difficulty::by_id(-1), Difficulty::Hard);
        assert_eq!(Difficulty::by_id(4), Difficulty::Peaceful);
        assert_eq!(Difficulty::by_id(6), Difficulty::Normal);
        assert_eq!(Difficulty::by_name("normal"), Some(Difficulty::Normal));
        assert_eq!(Difficulty::by_name("NORMAL"), None);
        assert_eq!(
            Difficulty::Hard.display_name().get_string(),
            "options.difficulty.hard"
        );
        assert_eq!(
            Difficulty::Peaceful.info().get_string(),
            "options.difficulty.peaceful.info"
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn difficulty_source_matches_java_26_1_2() {
        const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/world/Difficulty.java");
        for fragment in [
            "PEACEFUL(0, \"peaceful\")",
            "EASY(1, \"easy\")",
            "NORMAL(2, \"normal\")",
            "HARD(3, \"hard\")",
            "ByIdMap.continuous(Difficulty::getId, values(), ByIdMap.OutOfBoundsStrategy.WRAP)",
            "public Component getDisplayName()",
            "Component.translatable(\"options.difficulty.\" + this.key)",
            "public Component getInfo()",
            "Component.translatable(\"options.difficulty.\" + this.key + \".info\")",
            "public static Difficulty byId(final int id)",
            "public static @Nullable Difficulty byName(final String name)",
            "public String getSerializedName()",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn difficulty_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"difficulty\")",
            "for (Difficulty difficulty : Difficulty.values())",
            "Commands.literal(difficulty.getSerializedName())",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "Difficulty difficultyx = ((CommandSourceStack)c.getSource()).getLevel().getDifficulty();",
            "Component.translatable(\"commands.difficulty.query\", difficultyx.getDisplayName())",
            "return difficultyx.getId();",
            "if (server.getWorldData().getDifficulty() == difficulty)",
            "ERROR_ALREADY_SAME_DIFFICULTY.create(difficulty.getSerializedName())",
            "server.setDifficulty(difficulty, true);",
            "Component.translatable(\"commands.difficulty.success\", difficulty.getDisplayName())",
            "return 0;",
        ] {
            assert!(
                DIFFICULTY_COMMAND_JAVA.contains(sentinel),
                "DifficultyCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
