use crate::chat_component::chat_type::BoundChatType;
use crate::chat_component::filter_mask::FilterMask;
use crate::chat_component::Component;

pub const SYSTEM_SENDER_UUID: &str = "00000000-0000-0000-0000-000000000000";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutgoingChatMessageModel {
    Disguised(Component),
    Player(OutgoingPlayerChatMessageModel),
}

impl OutgoingChatMessageModel {
    pub fn create(message: OutgoingPlayerChatMessageModel) -> Self {
        if message.is_system() {
            Self::Disguised(message.decorated_content())
        } else {
            Self::Player(message)
        }
    }

    pub fn content(&self) -> Component {
        match self {
            Self::Disguised(content) => content.clone(),
            Self::Player(message) => message.decorated_content(),
        }
    }

    pub fn send_to_player(
        &self,
        filtered: bool,
        chat_type: BoundChatType,
    ) -> OutgoingChatDelivery {
        match self {
            Self::Disguised(content) => OutgoingChatDelivery::Disguised {
                content: content.clone(),
                chat_type,
            },
            Self::Player(message) => {
                let filtered_message = message.filter_for_recipient(filtered);
                if filtered_message.is_fully_filtered() {
                    OutgoingChatDelivery::SuppressedFullyFiltered
                } else {
                    OutgoingChatDelivery::Player {
                        message: filtered_message,
                        chat_type,
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingPlayerChatMessageModel {
    sender: String,
    signed_content: String,
    unsigned_content: Option<Component>,
    filter_mask: FilterMask,
}

impl OutgoingPlayerChatMessageModel {
    pub fn unsigned(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            signed_content: content.into(),
            unsigned_content: None,
            filter_mask: FilterMask::pass_through(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::unsigned(SYSTEM_SENDER_UUID, content)
    }

    pub fn with_unsigned_content(mut self, content: Component) -> Self {
        self.unsigned_content = (content != Component::literal(self.signed_content.clone()))
            .then_some(content);
        self
    }

    pub fn with_filter_mask(mut self, filter_mask: FilterMask) -> Self {
        self.filter_mask = filter_mask;
        self
    }

    pub fn sender(&self) -> &str {
        &self.sender
    }

    pub fn signed_content(&self) -> &str {
        &self.signed_content
    }

    pub fn unsigned_content(&self) -> Option<&Component> {
        self.unsigned_content.as_ref()
    }

    pub fn filter_mask(&self) -> &FilterMask {
        &self.filter_mask
    }

    pub fn decorated_content(&self) -> Component {
        self.unsigned_content
            .clone()
            .unwrap_or_else(|| Component::literal(self.signed_content.clone()))
    }

    pub fn filter_for_recipient(&self, filtered: bool) -> Self {
        if filtered {
            self.clone()
        } else {
            self.clone().with_filter_mask(FilterMask::pass_through())
        }
    }

    pub fn is_system(&self) -> bool {
        self.sender == SYSTEM_SENDER_UUID
    }

    pub fn is_fully_filtered(&self) -> bool {
        self.filter_mask.is_fully_filtered()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutgoingChatDelivery {
    Disguised {
        content: Component,
        chat_type: BoundChatType,
    },
    Player {
        message: OutgoingPlayerChatMessageModel,
        chat_type: BoundChatType,
    },
    SuppressedFullyFiltered,
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::chat_type::{ChatType, ChatTypeDecoration};

    fn bound_chat_type() -> BoundChatType {
        BoundChatType::new(
            ChatType::new(
                ChatTypeDecoration::with_sender("chat.type.text"),
                ChatTypeDecoration::with_sender("chat.type.text.narrate"),
            ),
            Component::literal("Alex"),
        )
    }

    #[test]
    fn outgoing_chat_message_java_contract_is_tracked() {
        const OUTGOING_CHAT_MESSAGE_JAVA: &str =
            vibecraft_java_source!("/net/minecraft/network/chat/OutgoingChatMessage.java");
        const PLAYER_CHAT_MESSAGE_JAVA: &str =
            vibecraft_java_source!("/net/minecraft/network/chat/PlayerChatMessage.java");

        for sentinel in [
            "return message.isSystem() ? new OutgoingChatMessage.Disguised(message.decoratedContent()) : new OutgoingChatMessage.Player(message);",
            "player.connection.sendDisguisedChatMessage(this.content, chatType);",
            "PlayerChatMessage filteredMessage = this.message.filter(filtered);",
            "if (!filteredMessage.isFullyFiltered()) {",
            "player.connection.sendPlayerChatMessage(filteredMessage, chatType);",
        ] {
            assert!(
                OUTGOING_CHAT_MESSAGE_JAVA.contains(sentinel),
                "missing OutgoingChatMessage sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "private static final UUID SYSTEM_SENDER = Util.NIL_UUID;",
            "return unsigned(SYSTEM_SENDER, content);",
            "return Objects.requireNonNullElseGet(this.unsignedContent, () -> Component.literal(this.signedContent()));",
            "return this.filter(filtered ? this.filterMask : FilterMask.PASS_THROUGH);",
            "return this.filterMask.isFullyFiltered();",
        ] {
            assert!(
                PLAYER_CHAT_MESSAGE_JAVA.contains(sentinel),
                "missing PlayerChatMessage outgoing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn create_uses_disguised_delivery_for_system_messages_only() {
        let system = OutgoingPlayerChatMessageModel::system("server text")
            .with_unsigned_content(Component::literal("decorated"));
        let player = OutgoingPlayerChatMessageModel::unsigned(
            "00000000-0000-0000-0000-000000000001",
            "hello",
        );

        assert_eq!(
            OutgoingChatMessageModel::create(system),
            OutgoingChatMessageModel::Disguised(Component::literal("decorated"))
        );
        assert!(matches!(
            OutgoingChatMessageModel::create(player),
            OutgoingChatMessageModel::Player(_)
        ));
    }

    #[test]
    fn disguised_messages_ignore_filter_flag_and_send_disguised_chat() {
        let chat_type = bound_chat_type();
        let message = OutgoingChatMessageModel::Disguised(Component::literal("system"));

        assert_eq!(message.content(), Component::literal("system"));
        assert_eq!(
            message.send_to_player(true, chat_type.clone()),
            OutgoingChatDelivery::Disguised {
                content: Component::literal("system"),
                chat_type: chat_type.clone(),
            }
        );
        assert_eq!(
            message.send_to_player(false, chat_type.clone()),
            OutgoingChatDelivery::Disguised {
                content: Component::literal("system"),
                chat_type,
            }
        );
    }

    #[test]
    fn player_messages_decorate_filter_and_suppress_like_java() {
        let chat_type = bound_chat_type();
        let mut partial = FilterMask::partially_filtered(5);
        partial.set_filtered(1);
        let message = OutgoingPlayerChatMessageModel::unsigned(
            "00000000-0000-0000-0000-000000000001",
            "hello",
        )
        .with_unsigned_content(Component::literal("decorated"))
        .with_filter_mask(partial.clone());
        let outgoing = OutgoingChatMessageModel::Player(message.clone());

        assert_eq!(outgoing.content(), Component::literal("decorated"));
        assert_eq!(
            outgoing.send_to_player(false, chat_type.clone()),
            OutgoingChatDelivery::Player {
                message: message
                    .clone()
                    .with_filter_mask(FilterMask::pass_through()),
                chat_type: chat_type.clone(),
            }
        );
        assert_eq!(
            outgoing.send_to_player(true, chat_type.clone()),
            OutgoingChatDelivery::Player {
                message: message.clone(),
                chat_type: chat_type.clone(),
            }
        );

        let fully_filtered = OutgoingChatMessageModel::Player(
            message.with_filter_mask(FilterMask::fully_filtered()),
        );
        assert_eq!(
            fully_filtered.send_to_player(true, chat_type),
            OutgoingChatDelivery::SuppressedFullyFiltered
        );
    }
}
