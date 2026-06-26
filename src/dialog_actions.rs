use std::collections::BTreeMap;

use crate::chat_component::ClickEvent;
use crate::command::StringTemplateModel;

const MAX_COMMAND_FUNCTION_LINE_LENGTH: usize = 2_000_000;

pub fn static_action_codec_types() -> &'static [&'static str] {
    &[
        "open_url",
        "run_command",
        "suggest_command",
        "show_dialog",
        "change_page",
        "copy_to_clipboard",
        "custom",
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticDialogAction {
    pub value: ClickEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogActionValueGetter {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDialogTemplate {
    pub raw: String,
    parsed: StringTemplateModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandTemplateDialogAction {
    pub template: ParsedDialogTemplate,
}

impl StaticDialogAction {
    pub fn new(value: ClickEvent) -> Self {
        Self { value }
    }

    pub fn codec_action_type(&self) -> Option<&'static str> {
        self.value
            .allowed_from_server()
            .then(|| self.value.action())
    }

    pub fn create_action(&self) -> Option<ClickEvent> {
        Some(self.value.clone())
    }
}

impl DialogActionValueGetter {
    pub fn of(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_template_substitution(&self) -> String {
        self.value.clone()
    }
}

impl ParsedDialogTemplate {
    pub fn parse(value: impl Into<String>) -> Result<Self, String> {
        let raw = value.into();
        let parsed = StringTemplateModel::from_string(&raw)
            .map_err(|err| format!("Failed to parse template {raw}: {err}"))?;
        Ok(Self { raw, parsed })
    }

    pub fn is_valid_variable_name(variable: &str) -> bool {
        StringTemplateModel::is_valid_variable_name(variable)
    }

    pub fn variables(&self) -> &[String] {
        &self.parsed.variables
    }

    pub fn instantiate(&self, arguments: &BTreeMap<String, String>) -> Result<String, String> {
        let values = self
            .parsed
            .variables
            .iter()
            .map(|key| arguments.get(key).cloned().unwrap_or_default())
            .collect::<Vec<_>>();
        substitute_template(&self.parsed, &values)
    }
}

impl CommandTemplateDialogAction {
    pub fn new(template: ParsedDialogTemplate) -> Self {
        Self { template }
    }

    pub fn create_action(
        &self,
        parameters: &BTreeMap<String, DialogActionValueGetter>,
    ) -> Result<Option<ClickEvent>, String> {
        let substitutions = parameters
            .iter()
            .map(|(key, value)| (key.clone(), value.as_template_substitution()))
            .collect::<BTreeMap<_, _>>();
        let command = self.template.instantiate(&substitutions)?;
        Ok(Some(ClickEvent::RunCommand(command)))
    }
}

fn substitute_template(template: &StringTemplateModel, arguments: &[String]) -> Result<String, String> {
    let mut result = String::new();
    for (index, argument) in arguments.iter().enumerate().take(template.variables.len()) {
        result.push_str(&template.segments[index]);
        result.push_str(argument);
        check_command_line_length(&result)?;
    }
    if template.segments.len() > template.variables.len() {
        if let Some(segment) = template.segments.last() {
            result.push_str(segment);
        }
    }
    check_command_line_length(&result)?;
    Ok(result)
}

fn check_command_line_length(line: &str) -> Result<(), String> {
    let length = java_string_length(line);
    if length > MAX_COMMAND_FUNCTION_LINE_LENGTH {
        let truncated = line.chars().take(512).collect::<String>();
        Err(format!(
            "Command too long: {length} characters, contents: {truncated}..."
        ))
    } else {
        Ok(())
    }
}

fn java_string_length(input: &str) -> usize {
    input.encode_utf16().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_action_wraps_only_server_allowed_click_event_codecs() {
        assert_eq!(
            static_action_codec_types(),
            &[
                "open_url",
                "run_command",
                "suggest_command",
                "show_dialog",
                "change_page",
                "copy_to_clipboard",
                "custom"
            ]
        );

        let wrapped = [
            StaticDialogAction::new(ClickEvent::OpenUrl("https://example.com".to_string())),
            StaticDialogAction::new(ClickEvent::RunCommand("/say hi".to_string())),
            StaticDialogAction::new(ClickEvent::SuggestCommand("/help".to_string())),
            StaticDialogAction::new(ClickEvent::ShowDialog("minecraft:notice".to_string())),
            StaticDialogAction::new(ClickEvent::ChangePage(3)),
            StaticDialogAction::new(ClickEvent::CopyToClipboard("seed".to_string())),
            StaticDialogAction::new(ClickEvent::Custom {
                id: "minecraft:test".to_string(),
                payload: Some("{value:1}".to_string()),
            }),
        ];

        assert_eq!(
            wrapped
                .iter()
                .map(StaticDialogAction::codec_action_type)
                .collect::<Vec<_>>(),
            vec![
                Some("open_url"),
                Some("run_command"),
                Some("suggest_command"),
                Some("show_dialog"),
                Some("change_page"),
                Some("copy_to_clipboard"),
                Some("custom"),
            ]
        );
        for action in wrapped {
            assert_eq!(action.create_action(), Some(action.value));
        }

        let open_file = StaticDialogAction::new(ClickEvent::OpenFile("/tmp/server.log".to_string()));
        assert_eq!(open_file.codec_action_type(), None);
        assert_eq!(
            open_file.create_action(),
            Some(ClickEvent::OpenFile("/tmp/server.log".to_string()))
        );
    }

    #[test]
    fn parsed_dialog_template_scans_validates_and_substitutes_like_java() {
        assert!(ParsedDialogTemplate::is_valid_variable_name("player_1"));
        assert!(ParsedDialogTemplate::is_valid_variable_name(""));
        assert!(!ParsedDialogTemplate::is_valid_variable_name("bad-name"));

        let template = match ParsedDialogTemplate::parse("say $(name) $$ $(missing)") {
            Ok(template) => template,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(template.raw, "say $(name) $$ $(missing)");
        assert_eq!(template.variables(), &["name".to_string(), "missing".to_string()]);

        let mut arguments = BTreeMap::new();
        arguments.insert("name".to_string(), "Alex".to_string());
        assert_eq!(
            template.instantiate(&arguments),
            Ok("say Alex $$ ".to_string())
        );
        let no_variables = parse_template_error("say $name");
        assert!(no_variables.contains("Failed to parse template say $name: No variables in macro"));
        let unterminated = parse_template_error("say $(name");
        assert!(unterminated.contains("Unterminated macro variable"));
        let invalid = parse_template_error("say $(bad-name)");
        assert!(invalid.contains("Invalid macro variable name 'bad-name'"));
    }

    #[test]
    fn command_template_dialog_action_creates_run_command_from_value_getters() {
        let template = match ParsedDialogTemplate::parse("tell $(target) $(message)") {
            Ok(template) => template,
            Err(err) => panic!("{err}"),
        };
        let action = CommandTemplateDialogAction::new(template);
        let mut parameters = BTreeMap::new();
        parameters.insert("target".to_string(), DialogActionValueGetter::of("@a"));
        parameters.insert("message".to_string(), DialogActionValueGetter::of("hello"));

        assert_eq!(
            action.create_action(&parameters),
            Ok(Some(ClickEvent::RunCommand("tell @a hello".to_string())))
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn static_dialog_action_source_matches_java_26_1_2() {
        const STATIC_ACTION: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/action/StaticAction.java");

        for sentinel in [
            "public record StaticAction(ClickEvent value) implements Action",
            "public static final Map<ClickEvent.Action, MapCodec<StaticAction>> WRAPPED_CODECS",
            "new EnumMap<>(ClickEvent.Action.class)",
            "if (action.isAllowedFromServer())",
            "MapCodec<ClickEvent> mapCodec = action.valueCodec();",
            "result.put(action, mapCodec.xmap(StaticAction::new, StaticAction::value));",
            "return Collections.unmodifiableMap(result);",
            "return WRAPPED_CODECS.get(this.value.action());",
            "return Optional.of(this.value);",
        ] {
            assert!(
                STATIC_ACTION.contains(sentinel),
                "StaticAction.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn parsed_dialog_template_source_matches_java_26_1_2() {
        const PARSED_TEMPLATE: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/action/ParsedTemplate.java");
        const STRING_TEMPLATE: &str =
            vibecraft_java_source!("/net/minecraft/commands/functions/StringTemplate.java");
        const COMMAND_FUNCTION: &str =
            vibecraft_java_source!("/net/minecraft/commands/functions/CommandFunction.java");

        for sentinel in [
            "Codec.STRING.comapFlatMap(ParsedTemplate::parse, t -> t.raw)",
            "StringTemplate.isValidVariableName(s) ? DataResult.success(s) : DataResult.error(() -> s + \" is not a valid input name\")",
            "template = StringTemplate.fromString(value);",
            "return DataResult.error(() -> \"Failed to parse template \" + value + \": \" + e.getMessage());",
            "arguments.getOrDefault(k, \"\")",
            "return this.parsed.substitute(values);",
        ] {
            assert!(
                PARSED_TEMPLATE.contains(sentinel),
                "ParsedTemplate.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "if (!Character.isLetterOrDigit(character) && character != '_')",
            "throw new IllegalArgumentException(\"No variables in macro\");",
            "throw new IllegalArgumentException(\"Unterminated macro variable\");",
            "throw new IllegalArgumentException(\"Invalid macro variable name '\" + variable + \"'\");",
            "CommandFunction.checkCommandLineLength(builder);",
        ] {
            assert!(
                STRING_TEMPLATE.contains(sentinel),
                "StringTemplate.java is missing sentinel: {sentinel}"
            );
        }

        assert!(COMMAND_FUNCTION.contains("if (line.length() > 2000000)"));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn command_template_dialog_action_source_matches_java_26_1_2() {
        const COMMAND_TEMPLATE: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/action/CommandTemplate.java");

        for sentinel in [
            "public record CommandTemplate(ParsedTemplate template) implements Action",
            "ParsedTemplate.CODEC.fieldOf(\"template\").forGetter(CommandTemplate::template)",
            "return MAP_CODEC;",
            "String command = this.template.instantiate(Action.ValueGetter.getAsTemplateSubstitutions(parameters));",
            "return Optional.of(new ClickEvent.RunCommand(command));",
        ] {
            assert!(
                COMMAND_TEMPLATE.contains(sentinel),
                "CommandTemplate.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn parse_template_error(input: &str) -> String {
        match ParsedDialogTemplate::parse(input) {
            Ok(template) => panic!("expected template parse error, got {template:?}"),
            Err(err) => err,
        }
    }
}
