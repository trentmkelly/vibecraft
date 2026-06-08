use std::collections::HashMap;

use crate::chat_component::{
    Component, ComponentArgument as ChatComponentArgument, ComponentContent, NbtSource,
    ResolutionContext, Style, TextColor, TranslationTable,
};
use crate::storage::nbt::{parse_snbt, Tag};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentArgumentModel;

impl ComponentArgumentModel {
    pub fn text_component() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<Component, ComponentParseError> {
        let input = reader.remaining_after_whitespace();
        let consumed = consumed_snbt_argument(input)?;
        let tag = parse_snbt(&input[..consumed]).map_err(|error| {
            ComponentParseError::InvalidComponent(format!("argument.component.invalid: {error}"))
        })?;
        let component = component_from_tag(&tag)?;
        reader.advance_past_whitespace_and(consumed);
        Ok(component)
    }

    pub fn examples(&self) -> [&'static str; 5] {
        [
            "\"hello world\"",
            "'hello world'",
            "\"\"",
            "{text:\"hello world\"}",
            "[\"\"]",
        ]
    }

    pub fn list_suggestions(&self, _remaining: &str) -> Vec<String> {
        Vec::new()
    }
}

pub fn get_raw_component(context: &CommandContextModel, name: &str) -> Option<Component> {
    context.arguments.get(name).cloned()
}

pub fn get_resolved_component(
    context: &CommandContextModel,
    name: &str,
    entity_override: Option<&str>,
) -> Option<Component> {
    let component = get_raw_component(context, name)?;
    let mut resolution = context.resolution.clone();
    if let Some(entity) = entity_override {
        resolution = resolution.with_selector("@s", vec![entity]);
    }
    Some(Component::literal(
        component.render_plain(&context.translations, &resolution),
    ))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, Component>,
    translations: TranslationTable,
    resolution: ResolutionContext,
}

impl CommandContextModel {
    pub fn with_component(mut self, name: impl Into<String>, value: Component) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }

    pub fn with_translation(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.translations = self.translations.with(key, value);
        self
    }

    pub fn with_resolution(mut self, resolution: ResolutionContext) -> Self {
        self.resolution = resolution;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentParseError {
    EmptyInput,
    InvalidComponent(String),
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

    fn remaining_after_whitespace(&self) -> &str {
        self.input[self.cursor..].trim_start()
    }

    fn advance_past_whitespace_and(&mut self, count: usize) {
        while self.cursor < self.input.len()
            && self.input.as_bytes()[self.cursor].is_ascii_whitespace()
        {
            self.cursor += 1;
        }
        self.cursor += count;
    }
}

fn component_from_tag(tag: &Tag) -> Result<Component, ComponentParseError> {
    match tag {
        Tag::String(text) => Ok(Component::literal(text.clone())),
        Tag::List(values) => component_from_non_empty_list(values),
        Tag::Compound(fields) => component_from_compound(fields),
        _ => Err(invalid_component("No matching codec found")),
    }
}

fn component_from_non_empty_list(values: &[Tag]) -> Result<Component, ComponentParseError> {
    let Some((first, rest)) = values.split_first() else {
        return Err(invalid_component("List must have contents"));
    };
    let mut component = component_from_tag(first)?;
    for value in rest {
        component = component.append(component_from_tag(value)?);
    }
    Ok(component)
}

fn component_from_compound(fields: &[(String, Tag)]) -> Result<Component, ComponentParseError> {
    let mut component = if let Some(Tag::String(text)) = field(fields, "text") {
        Component::literal(text.clone())
    } else if let Some(Tag::String(key)) = field(fields, "translate") {
        let args = match field(fields, "with") {
            Some(Tag::List(values)) => values
                .iter()
                .map(component_argument_from_tag)
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => return Err(invalid_component("with must be a list")),
            None => Vec::new(),
        };
        Component::translatable(key.clone(), args)
    } else if let Some(Tag::String(keybind)) = field(fields, "keybind") {
        Component {
            content: ComponentContent::Keybind(keybind.clone()),
            style: Style::empty(),
            siblings: Vec::new(),
        }
    } else if let Some(Tag::String(selector)) = field(fields, "selector") {
        Component {
            content: ComponentContent::Selector {
                selector: selector.clone(),
                separator: None,
            },
            style: Style::empty(),
            siblings: Vec::new(),
        }
    } else if let Some(Tag::String(path)) = field(fields, "nbt") {
        Component {
            content: ComponentContent::Nbt {
                path: path.clone(),
                source: nbt_source_from_fields(fields)?,
                interpret: bool_field(fields, "interpret").unwrap_or(false),
                plain: bool_field(fields, "plain").unwrap_or(false),
                separator: None,
            },
            style: Style::empty(),
            siblings: Vec::new(),
        }
    } else {
        return Err(invalid_component("No matching codec found"));
    };

    apply_style(fields, &mut component)?;
    if let Some(Tag::List(extra)) = field(fields, "extra") {
        for sibling in extra {
            component = component.append(component_from_tag(sibling)?);
        }
    }
    Ok(component)
}

fn component_argument_from_tag(tag: &Tag) -> Result<ChatComponentArgument, ComponentParseError> {
    Ok(match tag {
        Tag::String(value) => ChatComponentArgument::String(value.clone()),
        Tag::Int(value) => ChatComponentArgument::Number(*value),
        Tag::Byte(0) => ChatComponentArgument::Boolean(false),
        Tag::Byte(1) => ChatComponentArgument::Boolean(true),
        Tag::Compound(_) | Tag::List(_) => {
            ChatComponentArgument::Component(Box::new(component_from_tag(tag)?))
        }
        _ => return Err(invalid_component("Unsupported translation argument")),
    })
}

fn apply_style(
    fields: &[(String, Tag)],
    component: &mut Component,
) -> Result<(), ComponentParseError> {
    if let Some(Tag::String(color)) = field(fields, "color") {
        component.style.color =
            Some(TextColor::parse(color).ok_or_else(|| invalid_component("Invalid color"))?);
    }
    component.style.bold = bool_field(fields, "bold");
    component.style.italic = bool_field(fields, "italic");
    component.style.underlined = bool_field(fields, "underlined");
    component.style.strikethrough = bool_field(fields, "strikethrough");
    component.style.obfuscated = bool_field(fields, "obfuscated");
    Ok(())
}

fn nbt_source_from_fields(fields: &[(String, Tag)]) -> Result<NbtSource, ComponentParseError> {
    if let Some(Tag::String(block)) = field(fields, "block") {
        return Ok(NbtSource::Block(block.clone()));
    }
    if let Some(Tag::String(entity)) = field(fields, "entity") {
        return Ok(NbtSource::Entity(entity.clone()));
    }
    if let Some(Tag::String(storage)) = field(fields, "storage") {
        return Ok(NbtSource::Storage(storage.clone()));
    }
    Err(invalid_component("Missing NBT source"))
}

fn bool_field(fields: &[(String, Tag)], name: &str) -> Option<bool> {
    match field(fields, name) {
        Some(Tag::Byte(value)) => Some(*value != 0),
        _ => None,
    }
}

fn field<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    fields
        .iter()
        .find(|(field_name, _value)| field_name == name)
        .map(|(_field_name, value)| value)
}

fn invalid_component(message: impl Into<String>) -> ComponentParseError {
    ComponentParseError::InvalidComponent(format!("argument.component.invalid: {}", message.into()))
}

fn consumed_snbt_argument(input: &str) -> Result<usize, ComponentParseError> {
    let mut chars = input.char_indices();
    let Some((_, first)) = chars.next() else {
        return Err(ComponentParseError::EmptyInput);
    };

    match first {
        '"' | '\'' => consumed_quoted(input, first),
        '{' => consumed_balanced(input, '{', '}'),
        '[' => consumed_balanced(input, '[', ']'),
        _ => Ok(input
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace())
            .map(|(index, _)| index)
            .unwrap_or(input.len())),
    }
}

fn consumed_quoted(input: &str, quote: char) -> Result<usize, ComponentParseError> {
    let mut escaped = false;
    for (index, ch) in input.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Ok(index + ch.len_utf8());
        }
    }
    Err(invalid_component("unterminated SNBT string"))
}

fn consumed_balanced(input: &str, open: char, close: char) -> Result<usize, ComponentParseError> {
    let mut stack = vec![open];
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in input.char_indices().skip(1) {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '{' | '[' => stack.push(ch),
            '}' => {
                if stack.pop() != Some('{') {
                    return Err(invalid_component("mismatched SNBT compound"));
                }
                if stack.is_empty() {
                    return Ok(index + ch.len_utf8());
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return Err(invalid_component("mismatched SNBT list"));
                }
                if stack.is_empty() {
                    return Ok(index + ch.len_utf8());
                }
            }
            _ => {}
        }
    }

    Err(invalid_component(format!(
        "expected SNBT character {close}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(Component, usize), ComponentParseError> {
        let mut reader = StringReaderModel::new(input);
        let value = ComponentArgumentModel::text_component().parse(&mut reader)?;
        Ok((value, reader.cursor()))
    }

    #[test]
    fn java_factory_examples_and_context_getters_match_source() {
        let argument = ComponentArgumentModel::text_component();
        assert_eq!(
            argument.examples(),
            [
                "\"hello world\"",
                "'hello world'",
                "\"\"",
                "{text:\"hello world\"}",
                "[\"\"]"
            ]
        );

        let component = Component::literal("hello");
        let context = CommandContextModel::default().with_component("message", component.clone());

        assert_eq!(get_raw_component(&context, "message"), Some(component));
        assert_eq!(get_raw_component(&context, "missing"), None);
    }

    #[test]
    fn java_parser_based_argument_accepts_documented_component_forms() {
        assert_eq!(
            parse("\"hello world\" tail").unwrap().1,
            "\"hello world\"".len()
        );
        assert_eq!(
            parse("'hello world'").unwrap().0.to_json(),
            "{\"text\":\"hello world\"}"
        );
        assert_eq!(parse("\"\"").unwrap().0.to_json(), "{\"text\":\"\"}");
        assert_eq!(
            parse("{text:\"hello world\"}").unwrap().0.to_json(),
            "{\"text\":\"hello world\"}"
        );
        assert_eq!(parse("[\"\"]").unwrap().0.to_json(), "{\"text\":\"\"}");
    }

    #[test]
    fn java_component_codec_concatenates_non_empty_lists_as_siblings() {
        let (component, cursor) =
            parse("[\"hello\",{text:\" world\",color:\"gold\",bold:true}] trailing").unwrap();

        assert_eq!(
            cursor,
            "[\"hello\",{text:\" world\",color:\"gold\",bold:true}]".len()
        );
        assert_eq!(
            component.render_plain(&TranslationTable::default(), &ResolutionContext::default()),
            "hello world"
        );
        assert_eq!(
            component.to_json(),
            "{\"text\":\"hello\",\"extra\":[{\"text\":\" world\",\"color\":\"gold\",\"bold\":true}]}"
        );
    }

    #[test]
    fn java_component_codec_decodes_translatable_arguments_and_extra() {
        let (component, _) =
            parse("{translate:\"commands.op.success\",with:[\"Alex\"],extra:[\"!\"]}").unwrap();
        let context = CommandContextModel::default()
            .with_component("message", component)
            .with_translation("commands.op.success", "Made %s a server operator");

        assert_eq!(
            get_resolved_component(&context, "message", None)
                .unwrap()
                .to_json(),
            "{\"text\":\"Made Alex a server operator!\"}"
        );
    }

    #[test]
    fn java_resolved_component_uses_resolution_context_and_entity_override() {
        let selector = parse("{selector:\"@a\"}").unwrap().0;
        let self_selector = parse("{selector:\"@s\"}").unwrap().0;
        let context = CommandContextModel::default()
            .with_component("players", selector)
            .with_component("self", self_selector)
            .with_resolution(
                ResolutionContext::default().with_selector("@a", vec!["Steve", "Alex"]),
            );

        assert_eq!(
            get_resolved_component(&context, "players", None)
                .unwrap()
                .to_json(),
            "{\"text\":\"Steve, Alex\"}"
        );
        assert_eq!(
            get_resolved_component(&context, "self", Some("Target"))
                .unwrap()
                .to_json(),
            "{\"text\":\"Target\"}"
        );
    }

    #[test]
    fn java_invalid_component_errors_are_wrapped_and_parser_cursor_is_preserved() {
        let mut reader = StringReaderModel::new("123 trailing");
        let error = ComponentArgumentModel::text_component().parse(&mut reader);

        assert!(matches!(
            error,
            Err(ComponentParseError::InvalidComponent(message))
                if message.starts_with("argument.component.invalid")
        ));
        assert_eq!(reader.cursor(), 0);

        let mut empty = StringReaderModel::new("");
        assert_eq!(
            ComponentArgumentModel::text_component().parse(&mut empty),
            Err(ComponentParseError::EmptyInput)
        );
        assert_eq!(empty.cursor(), 0);
    }

    #[test]
    fn java_suggestions_delegate_to_parser_based_argument() {
        assert_eq!(
            ComponentArgumentModel::text_component().list_suggestions("{te"),
            Vec::<String>::new()
        );
    }
}
