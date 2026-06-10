#![allow(dead_code)]

use crate::chat_component::{Component, ComponentArgument, Style, TextColor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatType {
    pub chat: ChatTypeDecoration,
    pub narration: ChatTypeDecoration,
}

impl ChatType {
    pub const CHAT: &'static str = "chat";
    pub const SAY_COMMAND: &'static str = "say_command";
    pub const MSG_COMMAND_INCOMING: &'static str = "msg_command_incoming";
    pub const MSG_COMMAND_OUTGOING: &'static str = "msg_command_outgoing";
    pub const TEAM_MSG_COMMAND_INCOMING: &'static str = "team_msg_command_incoming";
    pub const TEAM_MSG_COMMAND_OUTGOING: &'static str = "team_msg_command_outgoing";
    pub const EMOTE_COMMAND: &'static str = "emote_command";

    pub fn new(chat: ChatTypeDecoration, narration: ChatTypeDecoration) -> Self {
        Self { chat, narration }
    }

    pub fn default_chat_decoration() -> ChatTypeDecoration {
        ChatTypeDecoration::with_sender("chat.type.text")
    }

    pub fn bootstrap_registration_order() -> Vec<(&'static str, ChatType)> {
        vec![
            (
                Self::CHAT,
                Self::new(
                    Self::default_chat_decoration(),
                    ChatTypeDecoration::with_sender("chat.type.text.narrate"),
                ),
            ),
            (
                Self::SAY_COMMAND,
                Self::new(
                    ChatTypeDecoration::with_sender("chat.type.announcement"),
                    ChatTypeDecoration::with_sender("chat.type.text.narrate"),
                ),
            ),
            (
                Self::MSG_COMMAND_INCOMING,
                Self::new(
                    ChatTypeDecoration::incoming_direct_message(
                        "commands.message.display.incoming",
                    ),
                    ChatTypeDecoration::with_sender("chat.type.text.narrate"),
                ),
            ),
            (
                Self::MSG_COMMAND_OUTGOING,
                Self::new(
                    ChatTypeDecoration::outgoing_direct_message(
                        "commands.message.display.outgoing",
                    ),
                    ChatTypeDecoration::with_sender("chat.type.text.narrate"),
                ),
            ),
            (
                Self::TEAM_MSG_COMMAND_INCOMING,
                Self::new(
                    ChatTypeDecoration::team_message("chat.type.team.text"),
                    ChatTypeDecoration::with_sender("chat.type.text.narrate"),
                ),
            ),
            (
                Self::TEAM_MSG_COMMAND_OUTGOING,
                Self::new(
                    ChatTypeDecoration::team_message("chat.type.team.sent"),
                    ChatTypeDecoration::with_sender("chat.type.text.narrate"),
                ),
            ),
            (
                Self::EMOTE_COMMAND,
                Self::new(
                    ChatTypeDecoration::with_sender("chat.type.emote"),
                    ChatTypeDecoration::with_sender("chat.type.emote"),
                ),
            ),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTypeDecoration {
    pub translation_key: String,
    pub parameters: Vec<ChatTypeDecorationParameter>,
    pub style: Style,
}

impl ChatTypeDecoration {
    pub fn new(
        translation_key: impl Into<String>,
        parameters: Vec<ChatTypeDecorationParameter>,
        style: Style,
    ) -> Self {
        Self {
            translation_key: translation_key.into(),
            parameters,
            style,
        }
    }

    pub fn with_sender(translation_key: impl Into<String>) -> Self {
        Self::new(
            translation_key,
            vec![
                ChatTypeDecorationParameter::Sender,
                ChatTypeDecorationParameter::Content,
            ],
            Style::empty(),
        )
    }

    pub fn incoming_direct_message(translation_key: impl Into<String>) -> Self {
        Self::new(
            translation_key,
            vec![
                ChatTypeDecorationParameter::Sender,
                ChatTypeDecorationParameter::Content,
            ],
            gray_italic_style(),
        )
    }

    pub fn outgoing_direct_message(translation_key: impl Into<String>) -> Self {
        Self::new(
            translation_key,
            vec![
                ChatTypeDecorationParameter::Target,
                ChatTypeDecorationParameter::Content,
            ],
            gray_italic_style(),
        )
    }

    pub fn team_message(translation_key: impl Into<String>) -> Self {
        Self::new(
            translation_key,
            vec![
                ChatTypeDecorationParameter::Target,
                ChatTypeDecorationParameter::Sender,
                ChatTypeDecorationParameter::Content,
            ],
            Style::empty(),
        )
    }

    pub fn decorate(&self, content: Component, chat_type: &BoundChatType) -> Component {
        let parameters = self
            .parameters
            .iter()
            .map(|parameter| {
                ComponentArgument::Component(Box::new(parameter.select(&content, chat_type)))
            })
            .collect();
        Component::translatable(self.translation_key.clone(), parameters).styled(self.style.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatTypeDecorationParameter {
    Sender,
    Target,
    Content,
}

impl ChatTypeDecorationParameter {
    pub fn id(self) -> i32 {
        match self {
            Self::Sender => 0,
            Self::Target => 1,
            Self::Content => 2,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Target,
            2 => Self::Content,
            _ => Self::Sender,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Sender => "sender",
            Self::Target => "target",
            Self::Content => "content",
        }
    }

    pub fn select(self, content: &Component, chat_type: &BoundChatType) -> Component {
        match self {
            Self::Sender => chat_type.name.clone(),
            Self::Target => chat_type
                .target_name
                .clone()
                .unwrap_or_else(Component::empty),
            Self::Content => content.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundChatType {
    pub chat_type: ChatType,
    pub name: Component,
    pub target_name: Option<Component>,
}

impl BoundChatType {
    pub fn new(chat_type: ChatType, name: Component) -> Self {
        Self {
            chat_type,
            name,
            target_name: None,
        }
    }

    pub fn with_target_name(mut self, target_name: Component) -> Self {
        self.target_name = Some(target_name);
        self
    }

    pub fn decorate(&self, content: Component) -> Component {
        self.chat_type.chat.decorate(content, self)
    }

    pub fn decorate_narration(&self, content: Component) -> Component {
        self.chat_type.narration.decorate(content, self)
    }
}

fn gray_italic_style() -> Style {
    Style::empty()
        .with_color(TextColor::parse("gray").unwrap_or_else(|| TextColor::from_rgb(0xAAAAAA)))
        .with_italic(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::status::{chat_type_nbt, ChatTypeDecorationStyle, CHAT_TYPES};
    use crate::storage::nbt::Tag;

    #[test]
    fn chat_type_and_decoration_match_java_bootstrap_and_parameters() {
        const CHAT_TYPE_JAVA: &str =
            include_str!("../../decompiled-server-26.1.2/net/minecraft/network/chat/ChatType.java");
        const CHAT_TYPE_DECORATION_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/ChatTypeDecoration.java"
        );

        for sentinel in [
            "public static final ChatTypeDecoration DEFAULT_CHAT_DECORATION = ChatTypeDecoration.withSender(\"chat.type.text\");",
            "public static final ResourceKey<ChatType> CHAT = create(\"chat\");",
            "public static final ResourceKey<ChatType> SAY_COMMAND = create(\"say_command\");",
            "public static final ResourceKey<ChatType> MSG_COMMAND_INCOMING = create(\"msg_command_incoming\");",
            "public static final ResourceKey<ChatType> MSG_COMMAND_OUTGOING = create(\"msg_command_outgoing\");",
            "public static final ResourceKey<ChatType> TEAM_MSG_COMMAND_INCOMING = create(\"team_msg_command_incoming\");",
            "public static final ResourceKey<ChatType> TEAM_MSG_COMMAND_OUTGOING = create(\"team_msg_command_outgoing\");",
            "public static final ResourceKey<ChatType> EMOTE_COMMAND = create(\"emote_command\");",
            "context.register(CHAT, new ChatType(DEFAULT_CHAT_DECORATION, ChatTypeDecoration.withSender(\"chat.type.text.narrate\")));",
            "return this.chatType.value().chat().decorate(content, this);",
            "return this.chatType.value().narration().decorate(content, this);",
        ] {
            assert!(
                CHAT_TYPE_JAVA.contains(sentinel),
                "missing ChatType sentinel {sentinel}"
            );
        }

        for sentinel in [
            "return new ChatTypeDecoration(translationKey, List.of(ChatTypeDecoration.Parameter.SENDER, ChatTypeDecoration.Parameter.CONTENT), Style.EMPTY);",
            "Style style = Style.EMPTY.withColor(ChatFormatting.GRAY).withItalic(true);",
            "return new ChatTypeDecoration(translationKey, List.of(ChatTypeDecoration.Parameter.TARGET, ChatTypeDecoration.Parameter.CONTENT), style);",
            "translationKey, List.of(ChatTypeDecoration.Parameter.TARGET, ChatTypeDecoration.Parameter.SENDER, ChatTypeDecoration.Parameter.CONTENT), Style.EMPTY",
            "SENDER(0, \"sender\", (content, chatType) -> chatType.name())",
            "TARGET(1, \"target\", (content, chatType) -> chatType.targetName().orElse(CommonComponents.EMPTY))",
            "CONTENT(2, \"content\", (content, chatType) -> content)",
            "ByIdMap.continuous(p -> p.id, values(), ByIdMap.OutOfBoundsStrategy.ZERO)",
        ] {
            assert!(
                CHAT_TYPE_DECORATION_JAVA.contains(sentinel),
                "missing ChatTypeDecoration sentinel {sentinel}"
            );
        }

        assert_eq!(
            ChatType::bootstrap_registration_order()
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            vec![
                "chat",
                "say_command",
                "msg_command_incoming",
                "msg_command_outgoing",
                "team_msg_command_incoming",
                "team_msg_command_outgoing",
                "emote_command",
            ]
        );

        let incoming =
            ChatTypeDecoration::incoming_direct_message("commands.message.display.incoming");
        assert_eq!(
            incoming.parameters,
            vec![
                ChatTypeDecorationParameter::Sender,
                ChatTypeDecorationParameter::Content,
            ]
        );
        assert_eq!(
            incoming
                .style
                .color
                .map(|color| color.serialize())
                .as_deref(),
            Some("gray")
        );
        assert_eq!(incoming.style.italic, Some(true));

        let outgoing =
            ChatTypeDecoration::outgoing_direct_message("commands.message.display.outgoing");
        let bound = BoundChatType::new(
            ChatType::new(
                outgoing.clone(),
                ChatTypeDecoration::with_sender("narration.key"),
            ),
            Component::literal("Alex"),
        )
        .with_target_name(Component::literal("Steve"));
        assert_eq!(
            bound.decorate(Component::literal("hello")).to_json(),
            "{\"translate\":\"commands.message.display.outgoing\",\"with\":[{\"text\":\"Steve\"},{\"text\":\"hello\"}],\"color\":\"gray\",\"italic\":true}"
        );
        assert_eq!(
            bound
                .decorate_narration(Component::literal("hello"))
                .to_json(),
            "{\"translate\":\"narration.key\",\"with\":[{\"text\":\"Alex\"},{\"text\":\"hello\"}]}"
        );

        let unbounded = BoundChatType::new(
            ChatType::new(
                ChatTypeDecoration::team_message("team"),
                ChatTypeDecoration::with_sender("narration"),
            ),
            Component::literal("Sender"),
        );
        assert_eq!(
            ChatTypeDecorationParameter::Target
                .select(&Component::literal("content"), &unbounded)
                .to_json(),
            "{\"text\":\"\"}"
        );

        assert_eq!(ChatTypeDecorationParameter::Sender.id(), 0);
        assert_eq!(ChatTypeDecorationParameter::Target.id(), 1);
        assert_eq!(ChatTypeDecorationParameter::Content.id(), 2);
        assert_eq!(
            ChatTypeDecorationParameter::from_id(-1),
            ChatTypeDecorationParameter::Sender
        );
        assert_eq!(
            ChatTypeDecorationParameter::from_id(99),
            ChatTypeDecorationParameter::Sender
        );
        assert_eq!(
            ChatTypeDecorationParameter::Target.serialized_name(),
            "target"
        );
    }

    #[test]
    fn chat_type_registry_nbt_matches_java_direct_message_styles() {
        let incoming = CHAT_TYPES
            .iter()
            .find(|chat_type| chat_type.id == "msg_command_incoming")
            .expect("vanilla incoming chat type");
        assert_eq!(incoming.chat_style, ChatTypeDecorationStyle::GrayItalic);
        assert_eq!(incoming.narration_style, ChatTypeDecorationStyle::Empty);

        let tag = chat_type_nbt(incoming);
        let Tag::Compound(root) = tag else {
            panic!("chat type NBT root should be a compound");
        };
        let Some(Tag::Compound(chat)) = field_value(&root, "chat") else {
            panic!("chat decoration should be a compound");
        };
        let Some(Tag::Compound(style)) = field_value(chat, "style") else {
            panic!("incoming direct message chat decoration should carry style");
        };
        assert_eq!(
            field_value(style, "color"),
            Some(&Tag::String("gray".to_string()))
        );
        assert_eq!(field_value(style, "italic"), Some(&Tag::Byte(1)));

        let Some(Tag::Compound(narration)) = field_value(&root, "narration") else {
            panic!("narration decoration should be a compound");
        };
        assert!(field_value(narration, "style").is_none());
    }

    fn field_value<'a>(fields: &'a [(String, Tag)], field: &str) -> Option<&'a Tag> {
        fields
            .iter()
            .find_map(|(name, value)| (name == field).then_some(value))
    }
}
