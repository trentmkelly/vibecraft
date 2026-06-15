#![allow(dead_code)]

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
