use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveArgumentModel;

impl ObjectiveArgumentModel {
    pub fn objective() -> Self {
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
            CommandSourceModel::CommandSourceStack(source) => {
                suggest(source.server.scoreboard.objective_names(), builder);
                builder.clone().build()
            }
            CommandSourceModel::SharedSuggestionProvider(provider) => {
                provider.custom_suggestions.clone()
            }
            CommandSourceModel::Other => Vec::new(),
        }
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["foo", "*", "012"]
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
    CommandSourceStack(CommandSourceStackModel),
    SharedSuggestionProvider(SharedSuggestionProviderModel),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSourceStackModel {
    server: ServerModel,
}

impl CommandSourceStackModel {
    pub fn new(server: ServerModel) -> Self {
        Self { server }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSuggestionProviderModel {
    custom_suggestions: Vec<SuggestionModel>,
}

impl SharedSuggestionProviderModel {
    pub fn new(custom_suggestions: Vec<SuggestionModel>) -> Self {
        Self { custom_suggestions }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerModel {
    scoreboard: ScoreboardModel,
}

impl ServerModel {
    pub fn new(scoreboard: ScoreboardModel) -> Self {
        Self { scoreboard }
    }
}

pub fn get_objective(
    context: &CommandContextModel,
    name: &str,
) -> Result<ObjectiveModel, ObjectiveArgumentError> {
    let id = argument_string(context, name)?;
    let scoreboard = source_scoreboard(context)?;
    scoreboard
        .get_objective(&id)
        .ok_or(ObjectiveArgumentError::ObjectiveNotFound { name: id })
}

pub fn get_writable_objective(
    context: &CommandContextModel,
    name: &str,
) -> Result<ObjectiveModel, ObjectiveArgumentError> {
    let objective = get_objective(context, name)?;
    if objective.criteria().is_read_only() {
        Err(ObjectiveArgumentError::ObjectiveReadOnly {
            name: objective.name().to_string(),
        })
    } else {
        Ok(objective)
    }
}

fn argument_string(
    context: &CommandContextModel,
    name: &str,
) -> Result<String, ObjectiveArgumentError> {
    context
        .arguments
        .get(name)
        .cloned()
        .ok_or_else(|| ObjectiveArgumentError::MissingArgument {
            name: name.to_string(),
        })
}

fn source_scoreboard(
    context: &CommandContextModel,
) -> Result<&ScoreboardModel, ObjectiveArgumentError> {
    let CommandSourceModel::CommandSourceStack(source) = &context.source else {
        return Err(ObjectiveArgumentError::MissingScoreboardSource);
    };
    Ok(&source.server.scoreboard)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScoreboardModel {
    objectives_by_name: HashMap<String, ObjectiveModel>,
    objective_names: Vec<String>,
}

impl ScoreboardModel {
    pub fn add_objective(
        mut self,
        name: impl Into<String>,
        criteria: ObjectiveCriteriaModel,
    ) -> Self {
        let name = name.into();
        if !self.objectives_by_name.contains_key(&name) {
            self.objective_names.push(name.clone());
        }
        self.objectives_by_name
            .insert(name.clone(), ObjectiveModel::new(name, criteria));
        self
    }

    pub fn get_objective(&self, name: &str) -> Option<ObjectiveModel> {
        self.objectives_by_name.get(name).cloned()
    }

    pub fn objective_names(&self) -> Vec<String> {
        self.objective_names.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveModel {
    name: String,
    criteria: ObjectiveCriteriaModel,
    display_name: String,
}

impl ObjectiveModel {
    pub fn new(name: impl Into<String>, criteria: ObjectiveCriteriaModel) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            criteria,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn criteria(&self) -> ObjectiveCriteriaModel {
        self.criteria
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectiveCriteriaModel {
    Dummy,
    Trigger,
    Health,
    Food,
}

impl ObjectiveCriteriaModel {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dummy => "dummy",
            Self::Trigger => "trigger",
            Self::Health => "health",
            Self::Food => "food",
        }
    }

    pub fn is_read_only(self) -> bool {
        matches!(self, Self::Health | Self::Food)
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
pub enum ObjectiveArgumentError {
    MissingArgument { name: String },
    MissingScoreboardSource,
    ObjectiveNotFound { name: String },
    ObjectiveReadOnly { name: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command_context(scoreboard: ScoreboardModel) -> CommandContextModel {
        CommandContextModel::new(CommandSourceModel::CommandSourceStack(
            CommandSourceStackModel::new(ServerModel::new(scoreboard)),
        ))
    }

    fn suggestion_values(suggestions: Vec<SuggestionModel>) -> Vec<String> {
        suggestions
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect()
    }

    #[test]
    fn java_factory_examples_and_unquoted_parse_match_source() {
        assert_eq!(
            ObjectiveArgumentModel::objective().examples(),
            ["foo", "*", "012"]
        );

        let mut reader = StringReaderModel::new("kills.total trailing");
        let parsed = ObjectiveArgumentModel::objective().parse(&mut reader);
        assert_eq!(parsed, "kills.total");
        assert_eq!(reader.cursor(), "kills.total".len());

        let mut empty = StringReaderModel::new("");
        assert_eq!(ObjectiveArgumentModel::objective().parse(&mut empty), "");
        assert_eq!(empty.cursor(), 0);
    }

    #[test]
    fn java_get_objective_uses_context_string_and_server_scoreboard() {
        let scoreboard = ScoreboardModel::default()
            .add_objective("kills", ObjectiveCriteriaModel::Dummy)
            .add_objective("health", ObjectiveCriteriaModel::Health);
        let context = command_context(scoreboard).with_string("target", "kills");

        let objective = get_objective(&context, "target").unwrap();
        assert_eq!(objective.name(), "kills");
        assert_eq!(objective.display_name(), "kills");
        assert_eq!(objective.criteria().name(), "dummy");
        assert!(!objective.criteria().is_read_only());
    }

    #[test]
    fn java_criteria_read_only_flags_cover_custom_and_read_only_criteria() {
        assert_eq!(ObjectiveCriteriaModel::Dummy.name(), "dummy");
        assert_eq!(ObjectiveCriteriaModel::Trigger.name(), "trigger");
        assert_eq!(ObjectiveCriteriaModel::Health.name(), "health");
        assert_eq!(ObjectiveCriteriaModel::Food.name(), "food");
        assert!(!ObjectiveCriteriaModel::Dummy.is_read_only());
        assert!(!ObjectiveCriteriaModel::Trigger.is_read_only());
        assert!(ObjectiveCriteriaModel::Health.is_read_only());
        assert!(ObjectiveCriteriaModel::Food.is_read_only());
    }

    #[test]
    fn java_get_objective_reports_requested_missing_name() {
        let scoreboard =
            ScoreboardModel::default().add_objective("kills", ObjectiveCriteriaModel::Dummy);
        let context = command_context(scoreboard).with_string("target", "missing");

        assert_eq!(
            get_objective(&context, "target"),
            Err(ObjectiveArgumentError::ObjectiveNotFound {
                name: "missing".to_string()
            })
        );
    }

    #[test]
    fn java_get_writable_objective_rejects_read_only_criteria_by_objective_name() {
        let scoreboard = ScoreboardModel::default()
            .add_objective("healthbar", ObjectiveCriteriaModel::Health)
            .add_objective("triggerable", ObjectiveCriteriaModel::Trigger);

        let read_only_context =
            command_context(scoreboard.clone()).with_string("target", "healthbar");
        assert_eq!(
            get_writable_objective(&read_only_context, "target"),
            Err(ObjectiveArgumentError::ObjectiveReadOnly {
                name: "healthbar".to_string()
            })
        );

        let writable_context = command_context(scoreboard).with_string("target", "triggerable");
        assert_eq!(
            get_writable_objective(&writable_context, "target")
                .unwrap()
                .name(),
            "triggerable"
        );
    }

    #[test]
    fn java_missing_argument_and_wrong_source_are_distinct_model_errors() {
        let scoreboard =
            ScoreboardModel::default().add_objective("kills", ObjectiveCriteriaModel::Dummy);
        let context = command_context(scoreboard);
        assert_eq!(
            get_objective(&context, "target"),
            Err(ObjectiveArgumentError::MissingArgument {
                name: "target".to_string()
            })
        );

        let other_source =
            CommandContextModel::new(CommandSourceModel::Other).with_string("target", "kills");
        assert_eq!(
            get_objective(&other_source, "target"),
            Err(ObjectiveArgumentError::MissingScoreboardSource)
        );
    }

    #[test]
    fn java_suggestions_use_scoreboard_names_for_command_source_stack() {
        let scoreboard = ScoreboardModel::default()
            .add_objective("kills", ObjectiveCriteriaModel::Dummy)
            .add_objective("deaths", ObjectiveCriteriaModel::Dummy)
            .add_objective("player_kills", ObjectiveCriteriaModel::Dummy);
        let context = command_context(scoreboard);
        let mut builder = SuggestionsBuilderModel::new("kills");

        assert_eq!(
            suggestion_values(
                ObjectiveArgumentModel::objective().list_suggestions(&context, &mut builder)
            ),
            vec!["kills".to_string(), "player_kills".to_string()]
        );
    }

    #[test]
    fn java_suggestions_fall_back_to_custom_provider_or_empty() {
        let custom = vec![SuggestionModel {
            value: "custom-objective".to_string(),
            tooltip: None,
        }];
        let provider_context =
            CommandContextModel::new(CommandSourceModel::SharedSuggestionProvider(
                SharedSuggestionProviderModel::new(custom.clone()),
            ));
        let mut builder = SuggestionsBuilderModel::new("");
        assert_eq!(
            ObjectiveArgumentModel::objective().list_suggestions(&provider_context, &mut builder),
            custom
        );

        let other_context = CommandContextModel::new(CommandSourceModel::Other);
        let mut other_builder = SuggestionsBuilderModel::new("");
        assert!(ObjectiveArgumentModel::objective()
            .list_suggestions(&other_context, &mut other_builder)
            .is_empty());
    }
}
