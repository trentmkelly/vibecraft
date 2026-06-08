#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameTypeModel {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameTypeModel {
    pub const VALUES: [Self; 4] = [
        Self::Survival,
        Self::Creative,
        Self::Adventure,
        Self::Spectator,
    ];

    pub const fn id(self) -> i32 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }

    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Adventure => "adventure",
            Self::Spectator => "spectator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTypePredicateModel {
    types: Vec<GameTypeModel>,
}

impl GameTypePredicateModel {
    pub fn new(types: Vec<GameTypeModel>) -> Self {
        Self { types }
    }

    pub fn any() -> Self {
        Self::of(GameTypeModel::VALUES)
    }

    pub fn survival_like() -> Self {
        Self::of([GameTypeModel::Survival, GameTypeModel::Adventure])
    }

    pub fn of(types: impl IntoIterator<Item = GameTypeModel>) -> Self {
        Self::new(types.into_iter().collect())
    }

    pub fn matches(&self, game_type: GameTypeModel) -> bool {
        self.types.contains(&game_type)
    }

    pub fn types(&self) -> &[GameTypeModel] {
        &self.types
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_contains_all_java_game_type_values_in_enum_order() {
        let predicate = GameTypePredicateModel::any();

        assert_eq!(predicate.types(), &GameTypeModel::VALUES);
        for game_type in GameTypeModel::VALUES {
            assert!(predicate.matches(game_type));
        }
    }

    #[test]
    fn survival_like_contains_only_survival_and_adventure() {
        let predicate = GameTypePredicateModel::survival_like();

        assert_eq!(
            predicate.types(),
            &[GameTypeModel::Survival, GameTypeModel::Adventure]
        );
        assert!(predicate.matches(GameTypeModel::Survival));
        assert!(predicate.matches(GameTypeModel::Adventure));
        assert!(!predicate.matches(GameTypeModel::Creative));
        assert!(!predicate.matches(GameTypeModel::Spectator));
    }

    #[test]
    fn of_preserves_varargs_order_and_matches_by_membership() {
        let predicate = GameTypePredicateModel::of([
            GameTypeModel::Spectator,
            GameTypeModel::Creative,
            GameTypeModel::Spectator,
        ]);

        assert_eq!(
            predicate.types(),
            &[
                GameTypeModel::Spectator,
                GameTypeModel::Creative,
                GameTypeModel::Spectator,
            ]
        );
        assert!(predicate.matches(GameTypeModel::Creative));
        assert!(predicate.matches(GameTypeModel::Spectator));
        assert!(!predicate.matches(GameTypeModel::Survival));
    }

    #[test]
    fn game_type_ids_and_serialized_names_match_java_enum() {
        assert_eq!(
            GameTypeModel::VALUES.map(|game_type| game_type.id()),
            [0, 1, 2, 3]
        );
        assert_eq!(
            GameTypeModel::VALUES.map(|game_type| game_type.serialized_name()),
            ["survival", "creative", "adventure", "spectator"]
        );
    }
}
