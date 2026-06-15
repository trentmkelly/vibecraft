//! Mirrors net.minecraft.network.chat.ComponentUtils.java.

use crate::chat_component::common_components::CommonComponents;
use crate::chat_component::component_serialization::decode_json_str;
use crate::chat_component::{
    ClickEvent, Component, ComponentArgument, ComponentContent, HoverEvent, ResolutionContext,
    Style, TextColor, TranslationTable,
};
use crate::chat_component::resolution_context::ResolutionContextLimitBehavior;

pub const DEFAULT_SEPARATOR_TEXT: &str = ", ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrigadierMessageModel {
    string: String,
}

impl BrigadierMessageModel {
    pub fn new(string: impl Into<String>) -> Self {
        Self {
            string: string.into(),
        }
    }

    pub fn get_string(&self) -> &str {
        &self.string
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageModel {
    Component(Box<Component>),
    Plain(BrigadierMessageModel),
}

pub fn default_separator() -> Component {
    Component::literal(DEFAULT_SEPARATOR_TEXT).styled(Style::empty().with_color(gray()))
}

pub fn default_no_style_separator() -> Component {
    Component::literal(DEFAULT_SEPARATOR_TEXT)
}

pub fn merge_styles_component(component: &Component, style: &Style) -> Component {
    if *style == Style::empty() {
        return component.clone();
    }
    let inner = component.get_style();
    if *inner == Style::empty() {
        return component.copy_component().styled(style.clone());
    }
    if inner == style {
        return component.clone();
    }
    component.copy_component().styled(inner.apply_to(style))
}

pub fn resolve_optional(
    context: &ResolutionContext,
    component: Option<&Component>,
    recursion_depth: i32,
) -> Option<Component> {
    component.map(|component| resolve(context, component, recursion_depth))
}

pub fn resolve(context: &ResolutionContext, component: &Component, recursion_depth: i32) -> Component {
    if recursion_depth > context.depth_limit {
        return match context.depth_limit_behavior {
            ResolutionContextLimitBehavior::DiscardRemaining => CommonComponents::ellipsis(),
            ResolutionContextLimitBehavior::StopProcessingAndCopyRemaining => component.copy_component(),
        };
    }

    let mut result = resolve_content(context, component.get_contents(), recursion_depth + 1);
    for sibling in component.get_siblings() {
        result = result.append(resolve(context, sibling, recursion_depth + 1));
    }
    result.styled(resolve_style(context, component.get_style(), recursion_depth))
}

pub fn format_string_list(values: &[String]) -> Component {
    let mut sorted = values.to_vec();
    sorted.sort();
    let formatted = sorted
        .iter()
        .map(|value| Component::literal(value.clone()).styled(Style::empty().with_color(green())))
        .collect::<Vec<_>>();
    format_component_list(&formatted, &default_separator())
}

pub fn format_and_sort_list<T: Ord + Clone>(
    values: &[T],
    formatter: impl Fn(&T) -> Component,
) -> Component {
    if values.is_empty() {
        return Component::empty();
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    if sorted.len() == 1 {
        return formatter(&sorted[0]);
    }
    let formatted = sorted.iter().map(formatter).collect::<Vec<_>>();
    format_component_list(&formatted, &default_separator())
}

pub fn format_list<T>(values: &[T], formatter: impl Fn(&T) -> Component) -> Component {
    format_list_with_separator(values, &default_separator(), formatter)
}

pub fn format_list_optional_separator<T>(
    values: &[T],
    separator: Option<&Component>,
    formatter: impl Fn(&T) -> Component,
) -> Component {
    format_list_with_separator(values, separator.unwrap_or(&default_separator()), formatter)
}

pub fn format_component_list(values: &[Component], separator: &Component) -> Component {
    format_list_with_separator(values, separator, Clone::clone)
}

pub fn format_list_with_separator<T>(
    values: &[T],
    separator: &Component,
    formatter: impl Fn(&T) -> Component,
) -> Component {
    if values.is_empty() {
        return Component::empty();
    }
    if values.len() == 1 {
        return formatter(&values[0]).copy_component();
    }

    let mut result = Component::empty();
    let mut first = true;
    for value in values {
        if !first {
            result = result.append(separator.clone());
        }
        result = result.append(formatter(value));
        first = false;
    }
    result
}

pub fn wrap_in_square_brackets(inner: Component) -> Component {
    Component::translatable(
        "chat.square_brackets",
        vec![ComponentArgument::Component(Box::new(inner))],
    )
}

pub fn from_message(message: MessageModel) -> Component {
    match message {
        MessageModel::Component(component) => *component,
        MessageModel::Plain(message) => Component::literal(message.get_string()),
    }
}

pub fn is_translation_resolvable(component: Option<&Component>, translations: &TranslationTable) -> bool {
    match component.map(Component::get_contents) {
        Some(ComponentContent::Translatable { key, fallback, .. }) => {
            fallback.is_some() || translations.lookup(key).is_some()
        }
        _ => true,
    }
}

pub fn copy_on_click_text(text: &str) -> Component {
    let mut style = Style::empty()
        .with_color(green())
        .with_click_event(ClickEvent::CopyToClipboard(text.to_string()))
        .with_hover_event(HoverEvent::Text(Box::new(Component::translatable(
            "chat.copy.click",
            Vec::new(),
        ))));
    style.insertion = Some(text.to_string());
    wrap_in_square_brackets(Component::literal(text).styled(style))
}

fn resolve_content(
    context: &ResolutionContext,
    content: &ComponentContent,
    recursion_depth: i32,
) -> Component {
    match content {
        ComponentContent::Literal(text) => Component::literal(text.clone()),
        ComponentContent::Translatable { key, fallback, args } => Component::translatable(
            key.clone(),
            args.iter()
                .map(|arg| resolve_translation_arg(context, arg, recursion_depth))
                .collect(),
        )
        .with_optional_fallback(fallback.clone()),
        ComponentContent::Selector { selector, separator } => {
            if context.source.is_none() {
                return Component::empty();
            }
            let values = context.selector(selector).unwrap_or_default();
            let separator = resolve_optional(context, separator.as_deref(), recursion_depth)
                .unwrap_or_else(default_separator);
            let components = values.into_iter().map(Component::literal).collect::<Vec<_>>();
            format_component_list(&components, &separator)
        }
        ComponentContent::Score { name, objective } => {
            if context.source.is_none() {
                return Component::empty();
            }
            let score_name = if name == "*" {
                context.default_scoreboard_entity.as_deref().unwrap_or(name)
            } else {
                name
            };
            context
                .score(score_name, objective)
                .map(|value| Component::literal(value.to_string()))
                .unwrap_or_else(Component::empty)
        }
        ComponentContent::Keybind(key) => Component::keybind_component(key.clone()),
        ComponentContent::Nbt {
            path,
            source,
            interpret,
            separator,
            ..
        } => {
            if context.source.is_none() {
                return Component::empty();
            }
            let values = context.nbt(source, path);
            let separator = resolve_optional(context, separator.as_deref(), recursion_depth)
                .unwrap_or_else(default_no_style_separator);
            let components = values
                .iter()
                .map(|value| {
                    if *interpret {
                        decode_json_str(value).unwrap_or_else(|_| Component::literal(value.clone()))
                    } else {
                        Component::literal(value.clone())
                    }
                })
                .collect::<Vec<_>>();
            format_component_list(&components, &separator)
        }
        ComponentContent::Object { info, fallback } => {
            let resolved_fallback = resolve_optional(context, fallback.as_deref(), recursion_depth);
            match context.validate(info) {
                Some(validated) => Component::object_component(validated.clone(), resolved_fallback),
                None => resolved_fallback.unwrap_or_else(|| Component::literal(info.default_fallback())),
            }
        }
    }
}

fn resolve_translation_arg(
    context: &ResolutionContext,
    arg: &ComponentArgument,
    recursion_depth: i32,
) -> ComponentArgument {
    match arg {
        ComponentArgument::Component(component) => {
            ComponentArgument::Component(Box::new(resolve(context, component, recursion_depth)))
        }
        other => other.clone(),
    }
}

fn resolve_style(context: &ResolutionContext, style: &Style, recursion_depth: i32) -> Style {
    let mut resolved = style.clone();
    if let Some(HoverEvent::Text(text)) = &style.hover_event {
        resolved.hover_event = Some(HoverEvent::Text(Box::new(resolve(
            context,
            text,
            recursion_depth + 1,
        ))));
    }
    resolved
}

fn green() -> TextColor {
    TextColor::parse("green").unwrap_or_else(|| TextColor::from_rgb(0x55ff55))
}

fn gray() -> TextColor {
    TextColor::parse("gray").unwrap_or_else(|| TextColor::from_rgb(0xaaaaaa))
}

trait ComponentFallbackExt {
    fn with_optional_fallback(self, fallback: Option<String>) -> Self;
}

impl ComponentFallbackExt for Component {
    fn with_optional_fallback(self, fallback: Option<String>) -> Self {
        match fallback {
            Some(fallback) => self.with_fallback(fallback),
            None => self,
        }
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::resolution_context::{
        ObjectInfoValidatorModel, ResolutionContextLimitBehavior, ResolutionSourceModel,
    };
    use crate::chat_component::{NbtSource, ObjectContent};

    const COMPONENT_UTILS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/chat/ComponentUtils.java");

    fn source() -> ResolutionSourceModel {
        ResolutionSourceModel {
            entity: Some("Steve".to_string()),
        }
    }

    #[test]
    fn component_utils_java_source_contract_is_tracked() {
        for sentinel in [
            "public static final String DEFAULT_SEPARATOR_TEXT = \", \";",
            "public static final Component DEFAULT_SEPARATOR = Component.literal(\", \").withStyle(ChatFormatting.GRAY);",
            "public static final Component DEFAULT_NO_STYLE_SEPARATOR = Component.literal(\", \");",
            "public static MutableComponent mergeStyles(final MutableComponent component, final Style style)",
            "return inner.equals(style) ? component : component.setStyle(inner.applyTo(style));",
            "public static Optional<MutableComponent> resolve(final ResolutionContext context, final Optional<Component> component, final int recursionDepth)",
            "if (recursionDepth > context.depthLimit())",
            "case DISCARD_REMAINING -> CommonComponents.ELLIPSIS.copy();",
            "case STOP_PROCESSING_AND_COPY_REMAINING -> component.copy();",
            "MutableComponent result = component.getContents().resolve(context, recursionDepth + 1);",
            "result.append(resolve(context, sibling, recursionDepth + 1));",
            "return result.withStyle(resolveStyle(context, component.getStyle(), recursionDepth));",
            "if (style.getHoverEvent() instanceof HoverEvent.ShowText(Component text))",
            "return formatAndSortList(values, v -> Component.literal(v).withStyle(ChatFormatting.GREEN));",
            "sorted.sort(Comparable::compareTo);",
            "return formatList(values, (Component)DataFixUtils.orElse(separator, DEFAULT_SEPARATOR), formatter);",
            "return Component.empty();",
            "return formatter.apply((T)values.iterator().next()).copy();",
            "result.append(separator);",
            "public static MutableComponent wrapInSquareBrackets(final Component inner)",
            "return Component.translatable(\"chat.square_brackets\", inner);",
            "public static Component fromMessage(final Message message)",
            "public static boolean isTranslationResolvable(final @Nullable Component component)",
            "fallback != null || Language.getInstance().has(key)",
            "public static MutableComponent copyOnClickText(final String text)",
            "new ClickEvent.CopyToClipboard(text)",
            "new HoverEvent.ShowText(Component.translatable(\"chat.copy.click\"))",
            ".withInsertion(text)",
        ] {
            assert!(
                COMPONENT_UTILS_JAVA.contains(sentinel),
                "missing ComponentUtils Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn merge_styles_matches_java_empty_equal_and_apply_paths() {
        let base = Component::literal("base");
        let green_style = Style::empty().with_color(green());
        assert_eq!(merge_styles_component(&base, &Style::empty()), base);
        assert_eq!(
            merge_styles_component(&base, &green_style).style.color,
            Some(green())
        );

        let bold = Component::literal("bold").styled(Style::empty().with_bold(true));
        assert_eq!(merge_styles_component(&bold, bold.get_style()), bold);
        let merged = merge_styles_component(&bold, &green_style);
        assert_eq!(merged.style.bold, Some(true));
        assert_eq!(merged.style.color, Some(green()));
    }

    #[test]
    fn resolve_respects_depth_limit_siblings_dynamic_contents_and_hover_text() {
        let context = ResolutionContext::builder()
            .with_source(source())
            .set_depth_limit(1)
            .set_depth_limit_behavior(ResolutionContextLimitBehavior::DiscardRemaining)
            .build()
            .with_selector("@a", vec!["Alex", "Steve"])
            .with_score("Steve", "kills", 7)
            .with_keybind("key.jump", "Space")
            .with_nbt(
                NbtSource::Block("0 64 0".to_string()),
                "Items[0].id",
                vec![r#""minecraft:stone""#],
            );

        let component = Component::selector_component("@a", Some(Component::literal(" | ")))
            .append(Component::score_component("*", "kills"))
            .append(Component::keybind_component("key.jump"))
            .append(Component::nbt_component(
                "Items[0].id",
                NbtSource::Block("0 64 0".to_string()),
                true,
                false,
                None,
            ));
        let resolved = resolve(&context, &component, 0);
        assert_eq!(resolved.get_string(), "Alex | Steve7key.jumpminecraft:stone");

        let copy_context = ResolutionContext::builder()
            .with_source(source())
            .set_depth_limit(0)
            .set_depth_limit_behavior(ResolutionContextLimitBehavior::StopProcessingAndCopyRemaining)
            .build();
        assert_eq!(
            resolve(&copy_context, &Component::literal("root").append(Component::literal(" child")), 1)
                .get_string(),
            "root child"
        );

        let hover = Component::literal("hover")
            .styled(Style::empty().with_hover_event(HoverEvent::Text(Box::new(
                Component::selector_component("@a", None),
            ))));
        let hover_context = ResolutionContext::create(source()).with_selector("@a", vec!["Alex"]);
        let resolved_hover = resolve(&hover_context, &hover, 0);
        assert_eq!(
            resolved_hover.style.hover_event,
            Some(HoverEvent::Text(Box::new(Component::literal("Alex"))))
        );
    }

    #[test]
    fn format_list_variants_match_java_empty_single_sorted_and_separator_paths() {
        assert_eq!(format_string_list(&[]).get_string(), "");
        assert_eq!(format_string_list(&["z".to_string()]).get_string(), "z");
        let sorted = format_string_list(&["z".to_string(), "a".to_string()]);
        assert_eq!(sorted.get_string(), "a, z");
        assert_eq!(sorted.get_siblings()[0].style.color, Some(green()));

        let values = vec![Component::literal("A"), Component::literal("B")];
        assert_eq!(
            format_component_list(&values, &Component::literal(" | ")).get_string(),
            "A | B"
        );
        assert_eq!(
            format_list_optional_separator(&values, None, Clone::clone).get_string(),
            "A, B"
        );
    }

    #[test]
    fn wrappers_messages_translation_resolvability_and_copy_text_match_java() {
        let wrapped = wrap_in_square_brackets(Component::literal("x"));
        assert_eq!(
            wrapped,
            Component::translatable(
                "chat.square_brackets",
                vec![ComponentArgument::Component(Box::new(Component::literal("x")))]
            )
        );
        assert_eq!(
            from_message(MessageModel::Plain(BrigadierMessageModel::new("plain"))),
            Component::literal("plain")
        );
        let component = Component::literal("component");
        assert_eq!(
            from_message(MessageModel::Component(Box::new(component.clone()))),
            component
        );

        let translations = TranslationTable::default().with("known.key", "Known");
        assert!(is_translation_resolvable(
            Some(&Component::translatable("known.key", Vec::new())),
            &translations,
        ));
        assert!(is_translation_resolvable(
            Some(&Component::translatable("missing.key", Vec::new()).with_fallback("Fallback")),
            &TranslationTable::default(),
        ));
        assert!(!is_translation_resolvable(
            Some(&Component::translatable("missing.key", Vec::new())),
            &TranslationTable::default(),
        ));
        assert!(is_translation_resolvable(None, &TranslationTable::default()));

        let copy = copy_on_click_text("/seed");
        let ComponentContent::Translatable { key, args, .. } = copy.get_contents() else {
            panic!("copy text should be wrapped in square brackets translation");
        };
        assert_eq!(key, "chat.square_brackets");
        assert_eq!(
            copy.render_plain(
                &TranslationTable::default().with("chat.square_brackets", "[%s]"),
                &ResolutionContext::default(),
            ),
            "[/seed]"
        );
        let ComponentArgument::Component(inner) = &args[0] else {
            panic!("copy text wrapper should carry the clickable component as first arg");
        };
        assert_eq!(inner.style.color, Some(green()));
        assert_eq!(
            inner.style.click_event,
            Some(ClickEvent::CopyToClipboard("/seed".to_string()))
        );
        assert_eq!(
            inner.style.hover_event,
            Some(HoverEvent::Text(Box::new(Component::translatable(
                "chat.copy.click",
                Vec::new()
            ))))
        );
        assert_eq!(inner.style.insertion.as_deref(), Some("/seed"));
    }

    #[test]
    fn object_resolution_validates_and_uses_resolved_fallback_like_java() {
        let object = Component::object_component(
            ObjectContent::PlayerSprite {
                profile: "Alex".to_string(),
                hat: true,
            },
            Some(Component::selector_component("@s", None)),
        );
        let deny = ResolutionContext::builder()
            .with_source(source())
            .with_object_info_validator(ObjectInfoValidatorModel::DenyAll)
            .build()
            .with_selector("@s", vec!["Steve"]);
        assert_eq!(resolve(&deny, &object, 0), Component::literal("Steve"));

        let allow = ResolutionContext::builder()
            .with_source(source())
            .with_object_info_validator(ObjectInfoValidatorModel::AllowAll)
            .build()
            .with_selector("@s", vec!["Steve"]);
        let resolved = resolve(&allow, &object, 0);
        assert_eq!(resolved.get_string(), "Steve");
        assert!(matches!(
            resolved.get_contents(),
            ComponentContent::Object { .. }
        ));
    }
}
