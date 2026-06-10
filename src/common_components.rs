#![allow(dead_code)]

use crate::chat_component::{Component, ComponentArgument};

pub struct CommonComponents;

impl CommonComponents {
    pub fn empty() -> Component {
        Component::empty()
    }

    pub fn option_on() -> Component {
        Component::translatable("options.on", Vec::new())
    }

    pub fn option_off() -> Component {
        Component::translatable("options.off", Vec::new())
    }

    pub fn gui_done() -> Component {
        Component::translatable("gui.done", Vec::new())
    }

    pub fn gui_cancel() -> Component {
        Component::translatable("gui.cancel", Vec::new())
    }

    pub fn gui_yes() -> Component {
        Component::translatable("gui.yes", Vec::new())
    }

    pub fn gui_no() -> Component {
        Component::translatable("gui.no", Vec::new())
    }

    pub fn gui_ok() -> Component {
        Component::translatable("gui.ok", Vec::new())
    }

    pub fn gui_proceed() -> Component {
        Component::translatable("gui.proceed", Vec::new())
    }

    pub fn gui_continue() -> Component {
        Component::translatable("gui.continue", Vec::new())
    }

    pub fn gui_back() -> Component {
        Component::translatable("gui.back", Vec::new())
    }

    pub fn gui_to_title() -> Component {
        Component::translatable("gui.toTitle", Vec::new())
    }

    pub fn gui_acknowledge() -> Component {
        Component::translatable("gui.acknowledge", Vec::new())
    }

    pub fn gui_open_in_browser() -> Component {
        Component::translatable("chat.link.open", Vec::new())
    }

    pub fn gui_copy_to_clipboard() -> Component {
        Component::translatable("chat.copy", Vec::new())
    }

    pub fn gui_copy_link_to_clipboard() -> Component {
        Component::translatable("gui.copy_link_to_clipboard", Vec::new())
    }

    pub fn gui_disconnect() -> Component {
        Component::translatable("menu.disconnect", Vec::new())
    }

    pub fn gui_return_to_menu() -> Component {
        Component::translatable("menu.returnToMenu", Vec::new())
    }

    pub fn transfer_connect_failed() -> Component {
        Component::translatable("connect.failed.transfer", Vec::new())
    }

    pub fn connect_failed() -> Component {
        Component::translatable("connect.failed", Vec::new())
    }

    pub fn new_line() -> Component {
        Component::literal("\n")
    }

    pub fn narration_separator() -> Component {
        Component::literal(". ")
    }

    pub fn ellipsis() -> Component {
        Component::literal("...")
    }

    pub fn space() -> Component {
        Component::literal(" ")
    }

    pub fn days(value: i64) -> Component {
        Component::translatable("gui.days", vec![ComponentArgument::Long(value)])
    }

    pub fn hours(value: i64) -> Component {
        Component::translatable("gui.hours", vec![ComponentArgument::Long(value)])
    }

    pub fn minutes(value: i64) -> Component {
        Component::translatable("gui.minutes", vec![ComponentArgument::Long(value)])
    }

    pub fn option_status(value: bool) -> Component {
        if value {
            Self::option_on()
        } else {
            Self::option_off()
        }
    }

    pub fn disconnect_button_label(is_local_server: bool) -> Component {
        if is_local_server {
            Self::gui_return_to_menu()
        } else {
            Self::gui_disconnect()
        }
    }

    pub fn option_status_named(name: Component, value: bool) -> Component {
        Component::translatable(
            if value {
                "options.on.composed"
            } else {
                "options.off.composed"
            },
            vec![ComponentArgument::Component(Box::new(name))],
        )
    }

    pub fn option_name_value(name: Component, value: Component) -> Component {
        Component::translatable(
            "options.generic_value",
            vec![
                ComponentArgument::Component(Box::new(name)),
                ComponentArgument::Component(Box::new(value)),
            ],
        )
    }

    pub fn join_for_narration(components: &[Component]) -> Component {
        let mut result = Component::empty();
        for (index, component) in components.iter().enumerate() {
            result = result.append(component.clone());
            if index != components.len() - 1 {
                result = result.append(Self::narration_separator());
            }
        }
        result
    }

    pub fn join_lines(lines: &[Component]) -> Component {
        format_component_list(lines, Self::new_line())
    }
}

fn format_component_list(components: &[Component], separator: Component) -> Component {
    if components.is_empty() {
        return Component::empty();
    }

    if components.len() == 1 {
        return components[0].clone();
    }

    let mut result = Component::empty();
    let mut first = true;
    for component in components {
        if !first {
            result = result.append(separator.clone());
        }
        result = result.append(component.clone());
        first = false;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::{ResolutionContext, TranslationTable};

    #[test]
    fn common_components_match_java_constants_and_helpers() {
        const COMMON_COMPONENTS_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/chat/CommonComponents.java"
        );

        for sentinel in [
            "public static final Component EMPTY = Component.empty();",
            "public static final Component OPTION_ON = Component.translatable(\"options.on\");",
            "public static final Component OPTION_OFF = Component.translatable(\"options.off\");",
            "public static final Component GUI_DONE = Component.translatable(\"gui.done\");",
            "public static final Component GUI_CANCEL = Component.translatable(\"gui.cancel\");",
            "public static final Component GUI_YES = Component.translatable(\"gui.yes\");",
            "public static final Component GUI_NO = Component.translatable(\"gui.no\");",
            "public static final Component GUI_OK = Component.translatable(\"gui.ok\");",
            "public static final Component GUI_PROCEED = Component.translatable(\"gui.proceed\");",
            "public static final Component GUI_CONTINUE = Component.translatable(\"gui.continue\");",
            "public static final Component GUI_BACK = Component.translatable(\"gui.back\");",
            "public static final Component GUI_TO_TITLE = Component.translatable(\"gui.toTitle\");",
            "public static final Component GUI_ACKNOWLEDGE = Component.translatable(\"gui.acknowledge\");",
            "public static final Component GUI_OPEN_IN_BROWSER = Component.translatable(\"chat.link.open\");",
            "public static final Component GUI_COPY_TO_CLIPBOARD = Component.translatable(\"chat.copy\");",
            "public static final Component GUI_COPY_LINK_TO_CLIPBOARD = Component.translatable(\"gui.copy_link_to_clipboard\");",
            "public static final Component GUI_DISCONNECT = Component.translatable(\"menu.disconnect\");",
            "public static final Component GUI_RETURN_TO_MENU = Component.translatable(\"menu.returnToMenu\");",
            "public static final Component TRANSFER_CONNECT_FAILED = Component.translatable(\"connect.failed.transfer\");",
            "public static final Component CONNECT_FAILED = Component.translatable(\"connect.failed\");",
            "public static final Component NEW_LINE = Component.literal(\"\\n\");",
            "public static final Component NARRATION_SEPARATOR = Component.literal(\". \");",
            "public static final Component ELLIPSIS = Component.literal(\"...\");",
            "public static final Component SPACE = space();",
            "return Component.literal(\" \");",
            "return Component.translatable(\"gui.days\", value);",
            "return value ? OPTION_ON : OPTION_OFF;",
            "return isLocalServer ? GUI_RETURN_TO_MENU : GUI_DISCONNECT;",
            "return Component.translatable(value ? \"options.on.composed\" : \"options.off.composed\", name);",
            "return Component.translatable(\"options.generic_value\", name, value);",
            "MutableComponent result = Component.empty();",
            "result.append(NARRATION_SEPARATOR);",
            "return ComponentUtils.formatList(lines, NEW_LINE);",
        ] {
            assert!(
                COMMON_COMPONENTS_JAVA.contains(sentinel),
                "missing CommonComponents sentinel {sentinel}"
            );
        }

        let translations = TranslationTable::default();
        let context = ResolutionContext::default();

        assert_eq!(
            CommonComponents::space().render_plain(&translations, &context),
            " "
        );
        assert_eq!(
            CommonComponents::ellipsis().render_plain(&translations, &context),
            "..."
        );
        assert_eq!(
            CommonComponents::option_status(true).to_json(),
            "{\"translate\":\"options.on\"}"
        );
        assert_eq!(
            CommonComponents::option_status(false).to_json(),
            "{\"translate\":\"options.off\"}"
        );
        for (component, json) in [
            (CommonComponents::gui_done(), "{\"translate\":\"gui.done\"}"),
            (
                CommonComponents::gui_cancel(),
                "{\"translate\":\"gui.cancel\"}",
            ),
            (CommonComponents::gui_yes(), "{\"translate\":\"gui.yes\"}"),
            (CommonComponents::gui_no(), "{\"translate\":\"gui.no\"}"),
            (CommonComponents::gui_ok(), "{\"translate\":\"gui.ok\"}"),
            (
                CommonComponents::gui_proceed(),
                "{\"translate\":\"gui.proceed\"}",
            ),
            (
                CommonComponents::gui_continue(),
                "{\"translate\":\"gui.continue\"}",
            ),
            (CommonComponents::gui_back(), "{\"translate\":\"gui.back\"}"),
            (
                CommonComponents::gui_to_title(),
                "{\"translate\":\"gui.toTitle\"}",
            ),
            (
                CommonComponents::gui_acknowledge(),
                "{\"translate\":\"gui.acknowledge\"}",
            ),
            (
                CommonComponents::gui_open_in_browser(),
                "{\"translate\":\"chat.link.open\"}",
            ),
            (
                CommonComponents::gui_copy_to_clipboard(),
                "{\"translate\":\"chat.copy\"}",
            ),
            (
                CommonComponents::gui_copy_link_to_clipboard(),
                "{\"translate\":\"gui.copy_link_to_clipboard\"}",
            ),
            (
                CommonComponents::transfer_connect_failed(),
                "{\"translate\":\"connect.failed.transfer\"}",
            ),
            (
                CommonComponents::connect_failed(),
                "{\"translate\":\"connect.failed\"}",
            ),
        ] {
            assert_eq!(component.to_json(), json);
        }
        assert_eq!(
            CommonComponents::disconnect_button_label(true).to_json(),
            "{\"translate\":\"menu.returnToMenu\"}"
        );
        assert_eq!(
            CommonComponents::disconnect_button_label(false).to_json(),
            "{\"translate\":\"menu.disconnect\"}"
        );
        assert_eq!(
            CommonComponents::days(9_000_000_000).to_json(),
            "{\"translate\":\"gui.days\",\"with\":[9000000000]}"
        );

        let joined = CommonComponents::join_for_narration(&[
            Component::literal("First"),
            Component::literal("Second"),
            Component::literal("Third"),
        ]);
        assert_eq!(
            joined.render_plain(&translations, &context),
            "First. Second. Third"
        );
        assert_eq!(
            joined.to_json(),
            "{\"text\":\"\",\"extra\":[{\"text\":\"First\"},{\"text\":\". \"},{\"text\":\"Second\"},{\"text\":\". \"},{\"text\":\"Third\"}]}"
        );

        let lines =
            CommonComponents::join_lines(&[Component::literal("A"), Component::literal("B")]);
        assert_eq!(lines.render_plain(&translations, &context), "A\nB");
        assert_eq!(
            lines.to_json(),
            "{\"text\":\"\",\"extra\":[{\"text\":\"A\"},{\"text\":\"\\n\"},{\"text\":\"B\"}]}"
        );
        assert_eq!(
            CommonComponents::join_lines(&[Component::literal("Solo")]).to_json(),
            "{\"text\":\"Solo\"}"
        );

        assert_eq!(
            CommonComponents::option_status_named(Component::literal("Music"), true).to_json(),
            "{\"translate\":\"options.on.composed\",\"with\":[{\"text\":\"Music\"}]}"
        );
        assert_eq!(
            CommonComponents::option_name_value(
                Component::literal("Volume"),
                Component::literal("80%")
            )
            .to_json(),
            "{\"translate\":\"options.generic_value\",\"with\":[{\"text\":\"Volume\"},{\"text\":\"80%\"}]}"
        );
    }
}
