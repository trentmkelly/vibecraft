#![allow(dead_code)]

use crate::chat_component::{Component, ComponentArgument, HoverEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverEventAction {
    Text,
    Item,
    Entity,
}

impl HoverEventAction {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Text => "show_text",
            Self::Item => "show_item",
            Self::Entity => "show_entity",
        }
    }

    pub fn is_allowed_from_server(self) -> bool {
        true
    }
}

impl HoverEvent {
    pub fn action_kind(&self) -> HoverEventAction {
        match self {
            Self::Text(_) => HoverEventAction::Text,
            Self::Item { .. } => HoverEventAction::Item,
            Self::Entity { .. } => HoverEventAction::Entity,
        }
    }

    pub fn is_allowed_from_server(&self) -> bool {
        self.action_kind().is_allowed_from_server()
    }

    pub fn entity_tooltip_lines(&self, entity_description: Component) -> Option<Vec<Component>> {
        let Self::Entity { uuid, name, .. } = self else {
            return None;
        };

        let mut lines = Vec::new();
        if let Some(name) = name {
            lines.push((**name).clone());
        }
        lines.push(Component::translatable(
            "gui.entity_tooltip.type",
            vec![ComponentArgument::Component(Box::new(entity_description))],
        ));
        lines.push(Component::literal(uuid.clone()));
        Some(lines)
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    #[test]
    fn hover_event_matches_java_actions_payloads_and_tooltip_lines() {
        const HOVER_EVENT_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/HoverEvent.java"
        );
        const ITEM_STACK_TEMPLATE_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/world/item/ItemStackTemplate.java"
        );

        for sentinel in [
            "Codec<HoverEvent> CODEC = HoverEvent.Action.CODEC.dispatch(\"action\", HoverEvent::action, action -> action.codec);",
            "SHOW_TEXT(\"show_text\", true, HoverEvent.ShowText.CODEC)",
            "SHOW_ITEM(\"show_item\", true, HoverEvent.ShowItem.CODEC)",
            "SHOW_ENTITY(\"show_entity\", true, HoverEvent.ShowEntity.CODEC)",
            "public boolean isAllowedFromServer()",
            "ComponentSerialization.CODEC.fieldOf(\"value\").forGetter(HoverEvent.ShowText::value)",
            "BuiltInRegistries.ENTITY_TYPE.byNameCodec().fieldOf(\"id\").forGetter(o -> o.type)",
            "UUIDUtil.LENIENT_CODEC.fieldOf(\"uuid\").forGetter(o -> o.uuid)",
            "ComponentSerialization.CODEC.optionalFieldOf(\"name\").forGetter(o -> o.name)",
            "this.linesCache.add(Component.translatable(\"gui.entity_tooltip.type\", this.type.getDescription()));",
            "this.linesCache.add(Component.literal(this.uuid.toString()));",
            "ItemStackTemplate.MAP_CODEC.xmap(HoverEvent.ShowItem::new, HoverEvent.ShowItem::item)",
        ] {
            assert!(
                HOVER_EVENT_JAVA.contains(sentinel),
                "missing HoverEvent sentinel {sentinel}"
            );
        }

        for sentinel in [
            "Item.CODEC.fieldOf(\"id\").forGetter(ItemStackTemplate::item)",
            "ExtraCodecs.intRange(1, 99).optionalFieldOf(\"count\", 1).forGetter(ItemStackTemplate::count)",
            "DataComponentPatch.CODEC.optionalFieldOf(\"components\", DataComponentPatch.EMPTY).forGetter(ItemStackTemplate::components)",
        ] {
            assert!(
                ITEM_STACK_TEMPLATE_JAVA.contains(sentinel),
                "missing ItemStackTemplate sentinel {sentinel}"
            );
        }

        let text = HoverEvent::Text(Box::new(Component::literal("hello")));
        assert_eq!(text.action(), "show_text");
        assert_eq!(text.action_kind(), HoverEventAction::Text);
        assert!(text.is_allowed_from_server());
        assert_eq!(
            text.to_json(),
            "{\"action\":\"show_text\",\"value\":{\"text\":\"hello\"}}"
        );

        let item_default = HoverEvent::Item {
            item: "minecraft:stone".to_string(),
            count: 1,
            components: None,
        };
        assert_eq!(item_default.action_kind(), HoverEventAction::Item);
        assert_eq!(
            item_default.to_json(),
            "{\"action\":\"show_item\",\"id\":\"minecraft:stone\"}"
        );

        let item = HoverEvent::Item {
            item: "minecraft:diamond".to_string(),
            count: 3,
            components: Some("{\"minecraft:custom_name\":{\"text\":\"Gem\"}}".to_string()),
        };
        assert_eq!(
            item.to_json(),
            "{\"action\":\"show_item\",\"id\":\"minecraft:diamond\",\"count\":3,\"components\":{\"minecraft:custom_name\":{\"text\":\"Gem\"}}}"
        );

        let entity = HoverEvent::Entity {
            entity_type: "minecraft:zombie".to_string(),
            uuid: "00000000-0000-0000-0000-000000000001".to_string(),
            name: Some(Box::new(Component::literal("Zombie"))),
        };
        assert_eq!(entity.action_kind(), HoverEventAction::Entity);
        assert_eq!(
            entity.to_json(),
            "{\"action\":\"show_entity\",\"id\":\"minecraft:zombie\",\"uuid\":\"00000000-0000-0000-0000-000000000001\",\"name\":{\"text\":\"Zombie\"}}"
        );

        let Some(lines) = entity.entity_tooltip_lines(Component::translatable(
            "entity.minecraft.zombie",
            Vec::new(),
        )) else {
            panic!("entity hover event should provide tooltip lines");
        };
        assert_eq!(lines[0].to_json(), "{\"text\":\"Zombie\"}");
        assert_eq!(
            lines[1].to_json(),
            "{\"translate\":\"gui.entity_tooltip.type\",\"with\":[{\"translate\":\"entity.minecraft.zombie\"}]}"
        );
        assert_eq!(
            lines[2].to_json(),
            "{\"text\":\"00000000-0000-0000-0000-000000000001\"}"
        );

        assert!(HoverEventAction::Text.is_allowed_from_server());
        assert!(HoverEventAction::Item.is_allowed_from_server());
        assert!(HoverEventAction::Entity.is_allowed_from_server());
        assert_eq!(HoverEventAction::Entity.serialized_name(), "show_entity");
    }
}
