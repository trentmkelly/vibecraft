use crate::advancement_system::{AdvancementDefinition, AdvancementDisplay, AdvancementFrame};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfoModel {
    pub icon: Identifier,
    pub title: String,
    pub description: String,
    pub background: Option<Identifier>,
    pub frame: AdvancementFrame,
    pub show_toast: bool,
    pub announce_chat: bool,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
}

impl DisplayInfoModel {
    pub fn new(fields: DisplayInfoFields) -> Self {
        Self {
            icon: fields.icon,
            title: fields.title,
            description: fields.description,
            background: fields.background,
            frame: fields.frame,
            show_toast: fields.show_toast,
            announce_chat: fields.announce_chat,
            hidden: fields.hidden,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn from_advancement_display(display: &AdvancementDisplay) -> Self {
        Self {
            icon: display.icon.clone(),
            title: display.title.clone(),
            description: display.description.clone(),
            background: display.background.clone(),
            frame: display.frame,
            show_toast: display.show_toast,
            announce_chat: display.announce_chat,
            hidden: display.hidden,
            x: display.x,
            y: display.y,
        }
    }

    pub fn set_location(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn network_flags(&self) -> i32 {
        let mut flags = 0;
        if self.background.is_some() {
            flags |= 1;
        }
        if self.show_toast {
            flags |= 2;
        }
        if self.hidden {
            flags |= 4;
        }
        flags
    }

    pub fn from_network_payload(payload: DisplayInfoNetworkPayload) -> Self {
        let mut info = Self::new(DisplayInfoFields {
            icon: payload.icon,
            title: payload.title,
            description: payload.description,
            background: payload.background,
            frame: payload.frame,
            show_toast: (payload.flags & 2) != 0,
            announce_chat: false,
            hidden: (payload.flags & 4) != 0,
        });
        info.set_location(payload.x, payload.y);
        info
    }

    pub fn to_network_payload(&self) -> DisplayInfoNetworkPayload {
        DisplayInfoNetworkPayload {
            title: self.title.clone(),
            description: self.description.clone(),
            icon: self.icon.clone(),
            frame: self.frame,
            flags: self.network_flags(),
            background: self.background.clone(),
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfoFields {
    pub icon: Identifier,
    pub title: String,
    pub description: String,
    pub background: Option<Identifier>,
    pub frame: AdvancementFrame,
    pub show_toast: bool,
    pub announce_chat: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfoNetworkPayload {
    pub title: String,
    pub description: String,
    pub icon: Identifier,
    pub frame: AdvancementFrame,
    pub flags: i32,
    pub background: Option<Identifier>,
    pub x: f32,
    pub y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    #[test]
    fn display_info_json_codec_defaults_and_required_fields_match_java() {
        let parsed = AdvancementDefinition::from_json(
            "minecraft:test/display",
            r#"{
                "criteria":{"tick":{"trigger":"minecraft:tick"}},
                "display":{
                    "icon":{"id":"minecraft:grass_block"},
                    "title":{"translate":"title.key"},
                    "description":{"text":"Description"}
                }
            }"#,
        )
        .unwrap();
        let display = DisplayInfoModel::from_advancement_display(parsed.display.as_ref().unwrap());
        assert_eq!(display.icon, id("minecraft:grass_block"));
        assert_eq!(display.title, "title.key");
        assert_eq!(display.description, "Description");
        assert_eq!(display.background, None);
        assert_eq!(display.frame, AdvancementFrame::Task);
        assert!(display.show_toast);
        assert!(display.announce_chat);
        assert!(!display.hidden);
        assert_eq!((display.x, display.y), (0.0, 0.0));

        assert!(AdvancementDefinition::from_json(
            "minecraft:test/no_icon",
            r#"{
                "criteria":{"tick":{"trigger":"minecraft:tick"}},
                "display":{"title":"Title","description":"Description"}
            }"#,
        )
        .unwrap_err()
        .contains("missing icon"));
    }

    #[test]
    fn display_info_json_explicit_fields_match_java_codec() {
        let parsed = AdvancementDefinition::from_json(
            "minecraft:test/display",
            r#"{
                "criteria":{"tick":{"trigger":"minecraft:tick"}},
                "display":{
                    "icon":{"id":"minecraft:diamond"},
                    "title":"Title",
                    "description":"Description",
                    "background":"minecraft:gui/advancements/backgrounds/stone",
                    "frame":"challenge",
                    "show_toast":false,
                    "announce_to_chat":false,
                    "hidden":true
                }
            }"#,
        )
        .unwrap();
        let display = DisplayInfoModel::from_advancement_display(parsed.display.as_ref().unwrap());
        assert_eq!(display.icon, id("minecraft:diamond"));
        assert_eq!(
            display.background,
            Some(id("minecraft:gui/advancements/backgrounds/stone"))
        );
        assert_eq!(display.frame, AdvancementFrame::Challenge);
        assert!(!display.show_toast);
        assert!(!display.announce_chat);
        assert!(display.hidden);
        assert_eq!(display.network_flags(), 5);
    }

    #[test]
    fn display_info_network_flags_location_and_announce_chat_match_java() {
        let mut display = DisplayInfoModel::new(DisplayInfoFields {
            icon: id("minecraft:stone"),
            title: "Title".to_string(),
            description: "Description".to_string(),
            background: Some(id("minecraft:gui/advancements/backgrounds/stone")),
            frame: AdvancementFrame::Goal,
            show_toast: true,
            announce_chat: true,
            hidden: true,
        });
        display.set_location(1.5, -2.0);
        let payload = display.to_network_payload();
        assert_eq!(payload.flags, 7);
        assert_eq!((payload.x, payload.y), (1.5, -2.0));

        let decoded = DisplayInfoModel::from_network_payload(payload);
        assert_eq!(decoded.frame, AdvancementFrame::Goal);
        assert_eq!(
            decoded.background,
            Some(id("minecraft:gui/advancements/backgrounds/stone"))
        );
        assert!(decoded.show_toast);
        assert!(!decoded.announce_chat);
        assert!(decoded.hidden);
        assert_eq!((decoded.x, decoded.y), (1.5, -2.0));
    }
}
