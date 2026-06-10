use std::collections::{BTreeMap, HashMap};

use crate::chat_component::{Component, ResolutionContext, TranslationTable};
use crate::command_misc_argument_audits::{ArgumentTypeModel, SignedArgumentModel};
use crate::command_selector::{
    EntityRecord, EntityWithPosition, Selector, SelectorBase, SelectorError, Vec3,
};
use crate::command_signing_context::{CommandSigningContextModel, PlayerChatMessageModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageArgumentModel;

impl ArgumentTypeModel for MessageArgumentModel {
    type Output = MessageModel;
}

impl SignedArgumentModel for MessageArgumentModel {}

impl MessageArgumentModel {
    pub const MAX_LENGTH: usize = 256;

    pub fn message() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
        allow_selectors: bool,
    ) -> Result<MessageModel, MessageArgumentError> {
        MessageModel::parse_text(reader, allow_selectors)
    }

    pub fn examples(&self) -> [&'static str; 4] {
        ["Hello world!", "foo", "@e", "Hello @p :)"]
    }
}

pub fn get_message(
    context: &CommandContextModel,
    name: &str,
) -> Result<Component, MessageArgumentError> {
    context
        .arguments
        .get(name)
        .ok_or(MessageArgumentError::MissingArgument)?
        .resolve_component(context)
}

pub fn resolve_chat_message(
    context: &CommandContextModel,
    name: &str,
) -> Result<ResolvedPlayerChatMessageModel, MessageArgumentError> {
    let message = context
        .arguments
        .get(name)
        .ok_or(MessageArgumentError::MissingArgument)?;
    let formatted = message.resolve_component(context)?;
    if let Some(signed) = context.signing_context.get_argument(name) {
        Ok(resolve_signed_message(context, signed.clone(), formatted))
    } else {
        Ok(resolve_disguised_message(context, &message.text, formatted))
    }
}

fn resolve_signed_message(
    context: &CommandContextModel,
    signed_argument: PlayerChatMessageModel,
    formatted: Component,
) -> ResolvedPlayerChatMessageModel {
    let decorated = context.decorator.decorate(&formatted);
    ResolvedPlayerChatMessageModel {
        source: ResolvedMessageSource::Signed(signed_argument),
        signed_content: None,
        unsigned_content: decorated,
        filtered: context
            .sender_player_uuid
            .as_ref()
            .is_some_and(|uuid| context.filter_signed_messages_from_sender == Some(uuid.clone())),
    }
}

fn resolve_disguised_message(
    context: &CommandContextModel,
    text: &str,
    formatted: Component,
) -> ResolvedPlayerChatMessageModel {
    let decorated = context.decorator.decorate(&formatted);
    ResolvedPlayerChatMessageModel {
        source: ResolvedMessageSource::DisguisedSystem,
        signed_content: Some(text.to_string()),
        unsigned_content: decorated,
        filtered: false,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageModel {
    text: String,
    parts: Vec<MessagePartModel>,
}

impl MessageModel {
    fn parse_text(
        reader: &mut StringReaderModel,
        allow_selectors: bool,
    ) -> Result<Self, MessageArgumentError> {
        if reader.remaining_length() > MessageArgumentModel::MAX_LENGTH {
            return Err(MessageArgumentError::TooLong {
                length: reader.remaining_length(),
                max: MessageArgumentModel::MAX_LENGTH,
            });
        }

        let text = reader.remaining().to_string();
        if !allow_selectors {
            reader.set_cursor(reader.total_length());
            return Ok(Self {
                text,
                parts: Vec::new(),
            });
        }

        let offset = reader.cursor();
        let mut parts = Vec::new();
        while reader.can_read() {
            if reader.peek() != '@' {
                reader.skip();
                continue;
            }
            let start = reader.cursor();
            match parse_selector_at_cursor(reader) {
                Ok(selector) => {
                    parts.push(MessagePartModel {
                        start: start - offset,
                        end: reader.cursor() - offset,
                        selector,
                    });
                }
                Err(MessageArgumentError::MissingSelectorType)
                | Err(MessageArgumentError::UnknownSelectorType) => {
                    reader.set_cursor(start + 1);
                }
                Err(error) => return Err(error),
            }
        }

        Ok(Self { text, parts })
    }

    fn resolve_component(
        &self,
        context: &CommandContextModel,
    ) -> Result<Component, MessageArgumentError> {
        self.to_component(context, context.can_use_selectors)
    }

    pub fn to_component(
        &self,
        context: &CommandContextModel,
        allow_selectors: bool,
    ) -> Result<Component, MessageArgumentError> {
        if self.parts.is_empty() || !allow_selectors {
            return Ok(Component::literal(self.text.clone()));
        }

        let mut result = Component::literal(self.text[..self.parts[0].start].to_string());
        let mut read_to = self.parts[0].start;
        for part in &self.parts {
            if read_to < part.start {
                result = result.append(Component::literal(
                    self.text[read_to..part.start].to_string(),
                ));
            }
            result = result.append(part.to_component(context)?);
            read_to = part.end;
        }
        if read_to < self.text.len() {
            result = result.append(Component::literal(self.text[read_to..].to_string()));
        }
        Ok(result)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn parts(&self) -> &[MessagePartModel] {
        &self.parts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessagePartModel {
    start: usize,
    end: usize,
    selector: Selector,
}

impl MessagePartModel {
    fn to_component(
        &self,
        context: &CommandContextModel,
    ) -> Result<Component, MessageArgumentError> {
        check_selector_permission(context, &self.selector)?;
        let names = self
            .selector
            .select(
                &context.entities,
                context.source_position,
                &context.source_level,
                context.current_entity.as_deref(),
            )
            .into_iter()
            .map(|entity| entity.name)
            .collect::<Vec<_>>();
        Ok(Component::literal(names.join(", ")))
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

fn parse_selector_at_cursor(
    reader: &mut StringReaderModel,
) -> Result<Selector, MessageArgumentError> {
    let token = reader.read_argument_token();
    match token.as_str() {
        "@" => Err(MessageArgumentError::MissingSelectorType),
        "@x" | "@z" | "@q" => Err(MessageArgumentError::UnknownSelectorType),
        _ => Selector::parse(&token).map_err(MessageArgumentError::Selector),
    }
}

fn check_selector_permission(
    context: &CommandContextModel,
    selector: &Selector,
) -> Result<(), MessageArgumentError> {
    if !matches!(selector.base, SelectorBase::Name(_)) && !context.can_use_selectors {
        Err(MessageArgumentError::SelectorsNotAllowed)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandContextModel {
    arguments: HashMap<String, MessageModel>,
    entities: Vec<EntityWithPosition>,
    source_position: Vec3,
    source_level: String,
    current_entity: Option<String>,
    can_use_selectors: bool,
    signing_context: CommandSigningContextModel,
    decorator: ChatDecoratorModel,
    sender_player_uuid: Option<String>,
    filter_signed_messages_from_sender: Option<String>,
}

impl Default for CommandContextModel {
    fn default() -> Self {
        Self {
            arguments: HashMap::new(),
            entities: Vec::new(),
            source_position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            source_level: "overworld".to_string(),
            current_entity: None,
            can_use_selectors: true,
            signing_context: CommandSigningContextModel::anonymous(),
            decorator: ChatDecoratorModel::default(),
            sender_player_uuid: None,
            filter_signed_messages_from_sender: None,
        }
    }
}

impl CommandContextModel {
    pub fn with_argument(mut self, name: impl Into<String>, message: MessageModel) -> Self {
        self.arguments.insert(name.into(), message);
        self
    }

    pub fn with_entities(mut self, entities: Vec<EntityWithPosition>) -> Self {
        self.entities = entities;
        self
    }

    pub fn with_selector_permission(mut self, allowed: bool) -> Self {
        self.can_use_selectors = allowed;
        self
    }

    pub fn with_signing_context(mut self, signing_context: CommandSigningContextModel) -> Self {
        self.signing_context = signing_context;
        self
    }

    pub fn with_decorator(mut self, decorator: ChatDecoratorModel) -> Self {
        self.decorator = decorator;
        self
    }

    pub fn with_sender_player(mut self, uuid: impl Into<String>) -> Self {
        self.sender_player_uuid = Some(uuid.into());
        self
    }

    pub fn with_filter_for_sender(mut self, uuid: impl Into<String>) -> Self {
        self.filter_signed_messages_from_sender = Some(uuid.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ChatDecoratorModel {
    #[default]
    Plain,
    Wrapping {
        prefix: String,
        suffix: String,
    },
}

impl ChatDecoratorModel {
    pub fn wrapping(prefix: impl Into<String>, suffix: impl Into<String>) -> Self {
        Self::Wrapping {
            prefix: prefix.into(),
            suffix: suffix.into(),
        }
    }

    fn decorate(&self, component: &Component) -> Component {
        match self {
            Self::Plain => component.clone(),
            Self::Wrapping { prefix, suffix } => Component::literal(format!(
                "{}{}{}",
                prefix,
                component.render_plain(&TranslationTable::default(), &ResolutionContext::default()),
                suffix
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPlayerChatMessageModel {
    source: ResolvedMessageSource,
    signed_content: Option<String>,
    unsigned_content: Component,
    filtered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedMessageSource {
    Signed(PlayerChatMessageModel),
    DisguisedSystem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageArgumentError {
    TooLong { length: usize, max: usize },
    Selector(SelectorError),
    MissingSelectorType,
    UnknownSelectorType,
    SelectorsNotAllowed,
    MissingArgument,
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

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn total_length(&self) -> usize {
        self.input.len()
    }

    fn remaining_length(&self) -> usize {
        self.input.len() - self.cursor
    }

    fn remaining(&self) -> &str {
        &self.input[self.cursor..]
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn read_argument_token(&mut self) -> String {
        let start = self.cursor;
        if self.peek() == '@' {
            self.cursor += 1;
            if self.cursor < self.input.len() {
                self.cursor += 1;
            }
            if self.cursor < self.input.len() && self.input.as_bytes()[self.cursor] as char == '[' {
                let mut bracket_depth = 0usize;
                while self.cursor < self.input.len() {
                    let ch = self.input.as_bytes()[self.cursor] as char;
                    match ch {
                        '[' => bracket_depth += 1,
                        ']' => {
                            bracket_depth = bracket_depth.saturating_sub(1);
                            self.cursor += 1;
                            if bracket_depth == 0 {
                                break;
                            }
                            continue;
                        }
                        _ => {}
                    }
                    self.cursor += 1;
                }
            }
            return self.input[start..self.cursor].to_string();
        }

        while self.cursor < self.input.len()
            && !(self.input.as_bytes()[self.cursor] as char).is_ascii_whitespace()
        {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    fn parse(
        input: &str,
        allow_selectors: bool,
    ) -> Result<(MessageModel, usize), MessageArgumentError> {
        let mut reader = StringReaderModel::new(input);
        let value = MessageArgumentModel::message().parse(&mut reader, allow_selectors)?;
        Ok((value, reader.cursor()))
    }

    fn entity(name: &str, uuid: &str, player: bool, entity_type: &str) -> EntityWithPosition {
        EntityWithPosition {
            entity: EntityRecord {
                name: name.to_string(),
                uuid: uuid.to_string(),
                player,
                entity_type: entity_type.to_string(),
                level: "overworld".to_string(),
                team: None,
                tags: Vec::new(),
                scores: BTreeMap::new(),
                nbt: BTreeMap::new(),
                predicates: Vec::new(),
                gamemode: None,
                experience_level: 0,
                x_rotation: 0.0,
                y_rotation: 0.0,
                advancements: BTreeMap::new(),
            },
            position: Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
        }
    }

    #[test]
    fn java_factory_examples_and_signed_argument_marker_match_source() {
        let argument = MessageArgumentModel::message();
        assert_eq!(
            argument.examples(),
            ["Hello world!", "foo", "@e", "Hello @p :)"]
        );
        let message = MessageModel {
            text: "signed payload".to_string(),
            parts: Vec::new(),
        };
        assert_eq!(
            crate::command_misc_argument_audits::signed_argument_output(&argument, message.clone()),
            message
        );
    }

    #[test]
    fn java_parse_is_greedy_and_rejects_remaining_length_over_256() {
        let (message, cursor) = parse("Hello world!", true).unwrap();
        assert_eq!(message.text(), "Hello world!");
        assert!(message.parts().is_empty());
        assert_eq!(cursor, "Hello world!".len());

        let long = "a".repeat(257);
        let mut reader = StringReaderModel::new(long);
        assert_eq!(
            MessageArgumentModel::message().parse(&mut reader, true),
            Err(MessageArgumentError::TooLong {
                length: 257,
                max: 256
            })
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_parse_without_selector_permission_consumes_text_without_parts() {
        let (message, cursor) = parse("Hello @p :)", false).unwrap();

        assert_eq!(message.text(), "Hello @p :)");
        assert!(message.parts().is_empty());
        assert_eq!(cursor, "Hello @p :)".len());
    }

    #[test]
    fn java_parse_collects_selector_parts_with_offsets_and_skips_unknown_selector_heads() {
        let (message, cursor) =
            parse("Hello @p and @x then @e[type=minecraft:zombie]", true).unwrap();

        assert_eq!(
            message.text(),
            "Hello @p and @x then @e[type=minecraft:zombie]"
        );
        assert_eq!(message.parts().len(), 2);
        assert_eq!(
            (message.parts()[0].start(), message.parts()[0].end()),
            (6, 8)
        );
        assert_eq!(
            (message.parts()[1].start(), message.parts()[1].end()),
            (21, 46)
        );
        assert_eq!(
            cursor,
            "Hello @p and @x then @e[type=minecraft:zombie]".len()
        );
    }

    #[test]
    fn java_parse_rethrows_selector_errors_other_than_missing_or_unknown_type() {
        let error = parse("bad @a[limit=0]", true).unwrap_err();

        assert_eq!(
            error,
            MessageArgumentError::Selector(SelectorError::InvalidLimit)
        );
    }

    #[test]
    fn java_to_component_interpolates_selector_names_only_when_allowed() {
        let (message, _) = parse("Hello @p and @e[type=minecraft:zombie]!", true).unwrap();
        let context = CommandContextModel::default().with_entities(vec![
            entity("Alex", "u1", true, "minecraft:player"),
            entity("Zombie", "u2", false, "minecraft:zombie"),
        ]);

        assert_eq!(
            message
                .to_component(&context, true)
                .unwrap()
                .render_plain(&TranslationTable::default(), &ResolutionContext::default()),
            "Hello Alex and Zombie!"
        );
        assert_eq!(
            message
                .to_component(&context, false)
                .unwrap()
                .render_plain(&TranslationTable::default(), &ResolutionContext::default()),
            "Hello @p and @e[type=minecraft:zombie]!"
        );
    }

    #[test]
    fn java_get_message_uses_source_selector_permission() {
        let (message, _) = parse("Hello @p", true).unwrap();
        let context = CommandContextModel::default()
            .with_selector_permission(false)
            .with_argument("message", message)
            .with_entities(vec![entity("Alex", "u1", true, "minecraft:player")]);

        assert_eq!(
            get_message(&context, "message")
                .unwrap()
                .render_plain(&TranslationTable::default(), &ResolutionContext::default()),
            "Hello @p"
        );
    }

    #[test]
    fn java_resolve_chat_message_uses_signed_argument_when_present() {
        let (message, _) = parse("Hello @p", true).unwrap();
        let signed = PlayerChatMessageModel::new("client text", "sig");
        let context = CommandContextModel::default()
            .with_argument("message", message)
            .with_entities(vec![entity("Alex", "u1", true, "minecraft:player")])
            .with_signing_context(CommandSigningContextModel::signed([(
                "message",
                signed.clone(),
            )]))
            .with_decorator(ChatDecoratorModel::wrapping("[", "]"))
            .with_sender_player("sender")
            .with_filter_for_sender("sender");

        assert_eq!(
            resolve_chat_message(&context, "message").unwrap(),
            ResolvedPlayerChatMessageModel {
                source: ResolvedMessageSource::Signed(signed),
                signed_content: None,
                unsigned_content: Component::literal("[Hello Alex]"),
                filtered: true,
            }
        );
    }

    #[test]
    fn chat_decorator_plain_matches_java_identity_decorator() {
        const CHAT_DECORATOR_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/ChatDecorator.java"
        );

        for sentinel in [
            "@FunctionalInterface",
            "ChatDecorator PLAIN = (player, plain) -> plain;",
            "Component decorate(@Nullable ServerPlayer player, Component plain);",
        ] {
            assert!(
                CHAT_DECORATOR_JAVA.contains(sentinel),
                "missing ChatDecorator sentinel {sentinel}"
            );
        }

        let component = Component::literal("plain");
        assert_eq!(
            ChatDecoratorModel::default().decorate(&component),
            component
        );
    }

    #[test]
    fn java_resolve_chat_message_falls_back_to_disguised_system_message() {
        let (message, _) = parse("Hello @p", true).unwrap();
        let context = CommandContextModel::default()
            .with_argument("message", message)
            .with_entities(vec![entity("Alex", "u1", true, "minecraft:player")])
            .with_decorator(ChatDecoratorModel::wrapping("<", ">"));

        assert_eq!(
            resolve_chat_message(&context, "message").unwrap(),
            ResolvedPlayerChatMessageModel {
                source: ResolvedMessageSource::DisguisedSystem,
                signed_content: Some("Hello @p".to_string()),
                unsigned_content: Component::literal("<Hello Alex>"),
                filtered: false,
            }
        );
    }
}
