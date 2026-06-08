use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamArgumentModel;

impl TeamArgumentModel {
    pub fn team() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> String {
        reader.read_unquoted_string()
    }

    pub fn list_suggestions(
        &self,
        context: &CommandContextModel,
        builder: &mut SuggestionsBuilderModel,
    ) -> Vec<SuggestionModel> {
        match &context.source {
            CommandSourceModel::SharedSuggestionProvider(provider) => {
                suggest(provider.all_teams.clone(), builder);
                builder.clone().build()
            }
            CommandSourceModel::Other => Vec::new(),
        }
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["foo", "123"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, String>,
    source: CommandSourceModel,
}

impl CommandContextModel {
    pub fn new(source: CommandSourceModel) -> Self {
        Self {
            arguments: HashMap::new(),
            source,
        }
    }

    pub fn with_string(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments.insert(name.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSourceModel {
    SharedSuggestionProvider(SharedSuggestionProviderModel),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSuggestionProviderModel {
    all_teams: Vec<String>,
    scoreboard: ScoreboardModel,
}

impl SharedSuggestionProviderModel {
    pub fn new(all_teams: impl Into<Vec<String>>, scoreboard: ScoreboardModel) -> Self {
        Self {
            all_teams: all_teams.into(),
            scoreboard,
        }
    }
}

pub fn get_team(
    context: &CommandContextModel,
    name: &str,
) -> Result<PlayerTeamModel, TeamArgumentError> {
    let id =
        context
            .arguments
            .get(name)
            .cloned()
            .ok_or_else(|| TeamArgumentError::MissingArgument {
                name: name.to_string(),
            })?;
    let CommandSourceModel::SharedSuggestionProvider(provider) = &context.source else {
        return Err(TeamArgumentError::MissingScoreboardSource);
    };
    provider
        .scoreboard
        .get_player_team(&id)
        .ok_or(TeamArgumentError::TeamNotFound { name: id })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScoreboardModel {
    teams_by_name: HashMap<String, PlayerTeamModel>,
}

impl ScoreboardModel {
    pub fn add_player_team(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.teams_by_name
            .entry(name.clone())
            .or_insert_with(|| PlayerTeamModel::new(name));
        self
    }

    pub fn get_player_team(&self, name: &str) -> Option<PlayerTeamModel> {
        self.teams_by_name.get(name).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerTeamModel {
    name: String,
    display_name: String,
}

impl PlayerTeamModel {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReaderModel {
    input: String,
    cursor: usize,
}

impl StringReaderModel {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && !self.peek().is_ascii_whitespace() {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamArgumentError {
    MissingArgument { name: String },
    MissingScoreboardSource,
    TeamNotFound { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestion_values(suggestions: Vec<SuggestionModel>) -> Vec<String> {
        suggestions
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect()
    }

    fn source_with_teams(names: &[&str]) -> CommandSourceModel {
        let scoreboard = names
            .iter()
            .fold(ScoreboardModel::default(), |scoreboard, name| {
                scoreboard.add_player_team(*name)
            });
        CommandSourceModel::SharedSuggestionProvider(SharedSuggestionProviderModel::new(
            names
                .iter()
                .map(|name| (*name).to_string())
                .collect::<Vec<_>>(),
            scoreboard,
        ))
    }

    #[test]
    fn java_factory_and_examples_match_source() {
        assert_eq!(TeamArgumentModel::team().examples(), ["foo", "123"]);
    }

    #[test]
    fn java_parse_returns_raw_unquoted_string_and_cursor() {
        let mut reader = StringReaderModel::new("alpha.team trailing");
        let parsed = TeamArgumentModel::team().parse(&mut reader);

        assert_eq!(parsed, "alpha.team");
        assert_eq!(reader.cursor(), "alpha.team".len());
    }

    #[test]
    fn java_parse_empty_input_returns_empty_string() {
        let mut reader = StringReaderModel::new("");
        let parsed = TeamArgumentModel::team().parse(&mut reader);

        assert_eq!(parsed, "");
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_get_team_returns_scoreboard_team_for_context_argument() {
        let context = CommandContextModel::new(source_with_teams(&["red", "blue"]))
            .with_string("team", "blue");

        let team = get_team(&context, "team").unwrap();
        assert_eq!(team.name(), "blue");
        assert_eq!(team.display_name(), "blue");
    }

    #[test]
    fn java_get_team_is_case_sensitive_and_errors_with_requested_id() {
        let context =
            CommandContextModel::new(source_with_teams(&["Red"])).with_string("team", "red");

        assert_eq!(
            get_team(&context, "team"),
            Err(TeamArgumentError::TeamNotFound {
                name: "red".to_string()
            })
        );
    }

    #[test]
    fn java_get_team_errors_when_argument_is_missing() {
        let context = CommandContextModel::new(source_with_teams(&["red"]));

        assert_eq!(
            get_team(&context, "team"),
            Err(TeamArgumentError::MissingArgument {
                name: "team".to_string()
            })
        );
    }

    #[test]
    fn java_suggestions_require_shared_suggestion_provider_source() {
        let argument = TeamArgumentModel::team();

        let context = CommandContextModel::new(source_with_teams(&["red", "blue", "green"]));
        let mut builder = SuggestionsBuilderModel::new("r");
        assert_eq!(
            suggestion_values(argument.list_suggestions(&context, &mut builder)),
            vec!["red".to_string()]
        );

        let other_context = CommandContextModel::new(CommandSourceModel::Other);
        let mut other_builder = SuggestionsBuilderModel::new("");
        assert!(argument
            .list_suggestions(&other_context, &mut other_builder)
            .is_empty());
    }
}
