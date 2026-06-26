use crate::chat_component::ClickEvent;

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
}
