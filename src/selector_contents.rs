use crate::chat_component::{Component, ResolutionContext, Style, TranslationTable};

pub const SELECTOR_CONTENTS_CODEC_FIELDS: [&str; 2] = ["selector", "separator"];
pub const DEFAULT_SELECTOR_SEPARATOR: &str = ", ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorContentsModel {
    selector: String,
    separator: Option<Component>,
}

impl SelectorContentsModel {
    pub fn new(selector: impl Into<String>, separator: Option<Component>) -> Self {
        Self {
            selector: selector.into(),
            separator,
        }
    }

    pub fn selector(&self) -> &str {
        &self.selector
    }

    pub fn separator(&self) -> Option<&Component> {
        self.separator.as_ref()
    }

    pub fn codec_fields(&self) -> [&'static str; 2] {
        SELECTOR_CONTENTS_CODEC_FIELDS
    }

    pub fn resolve(&self, context: &ResolutionContext, _recursion_depth: i32) -> Component {
        if context.source.is_none() {
            return Component::empty();
        }
        let values = context.selector(&self.selector).unwrap_or_default();
        let separator = self
            .separator
            .as_ref()
            .map(|component| component.render_plain(&TranslationTable::default(), context))
            .unwrap_or_else(|| DEFAULT_SELECTOR_SEPARATOR.to_string());
        format_component_list(&values, &separator)
    }

    pub fn visit<T>(&self, output: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        output(&self.selector)
    }

    pub fn visit_styled<T>(
        &self,
        output: &mut impl FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        output(current_style, &self.selector)
    }
}

impl std::fmt::Display for SelectorContentsModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "pattern{{{}}}", self.selector)
    }
}

fn format_component_list(values: &[String], separator: &str) -> Component {
    match values {
        [] => Component::empty(),
        [single] => Component::literal(single.clone()),
        [first, rest @ ..] => {
            let mut result = Component::literal(first.clone());
            for value in rest {
                result = result
                    .append(Component::literal(separator.to_string()))
                    .append(Component::literal(value.clone()));
            }
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::resolution_context::ResolutionSourceModel;

    const SELECTOR_CONTENTS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/SelectorContents.java"
    );
    const COMPONENT_UTILS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/ComponentUtils.java"
    );

    fn source(entity: Option<&str>) -> ResolutionSourceModel {
        ResolutionSourceModel {
            entity: entity.map(ToOwned::to_owned),
        }
    }

    fn context_with_source() -> ResolutionContext {
        ResolutionContext::create(source(None))
    }

    #[test]
    fn selector_contents_java_source_contract_is_tracked() {
        for sentinel in [
            "public record SelectorContents(CompilableString<EntitySelector> selector, Optional<Component> separator) implements ComponentContents",
            "EntitySelector.COMPILABLE_CODEC.fieldOf(\"selector\").forGetter(SelectorContents::selector)",
            "ComponentSerialization.CODEC.optionalFieldOf(\"separator\").forGetter(SelectorContents::separator)",
            "return MAP_CODEC;",
            "if (source == null)",
            "Optional<? extends Component> resolvedSeparator = ComponentUtils.resolve(context, this.separator, recursionDepth);",
            "return ComponentUtils.formatList(this.selector.compiled().findEntities(source), resolvedSeparator, Entity::getDisplayName);",
            "return output.accept(currentStyle, this.selector.source());",
            "return output.accept(this.selector.source());",
            "return \"pattern{\" + this.selector + \"}\";",
        ] {
            assert!(
                SELECTOR_CONTENTS_JAVA.contains(sentinel),
                "missing SelectorContents Java sentinel: {sentinel}"
            );
        }
        for sentinel in [
            "return formatList(values, (Component)DataFixUtils.orElse(separator, DEFAULT_SEPARATOR), formatter);",
            "if (values.isEmpty())",
            "return Component.empty();",
            "if (values.size() == 1)",
            "return formatter.apply((T)values.iterator().next()).copy();",
            "result.append(separator);",
            "result.append(formatter.apply(value));",
        ] {
            assert!(
                COMPONENT_UTILS_JAVA.contains(sentinel),
                "missing ComponentUtils formatList sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn selector_contents_resolves_entity_list_with_default_and_custom_separator() {
        let selector = SelectorContentsModel::new("@a", None);
        let context = context_with_source().with_selector("@a", vec!["Alex", "Steve"]);
        assert_eq!(selector.resolve(&context, 0).get_string(), "Alex, Steve");

        let custom = SelectorContentsModel::new("@a", Some(Component::literal(" | ")));
        assert_eq!(custom.resolve(&context, 0).get_string(), "Alex | Steve");

        let single = context_with_source().with_selector("@a", vec!["Alex"]);
        assert_eq!(selector.resolve(&single, 0).get_string(), "Alex");

        let empty = context_with_source().with_selector("@a", Vec::new());
        assert_eq!(selector.resolve(&empty, 0).get_string(), "");
    }

    #[test]
    fn selector_contents_source_null_and_missing_selector_resolve_empty() {
        let selector = SelectorContentsModel::new("@s", None);
        assert_eq!(
            selector
                .resolve(&ResolutionContext::default(), 0)
                .get_string(),
            ""
        );
        assert_eq!(selector.resolve(&context_with_source(), 0).get_string(), "");
    }

    #[test]
    fn selector_contents_visitors_accessors_codec_and_to_string_match_java_surface() {
        let style = Style::empty();
        let contents = SelectorContentsModel::new("@e[type=pig]", Some(Component::literal("; ")));

        assert_eq!(contents.codec_fields(), ["selector", "separator"]);
        assert_eq!(contents.selector(), "@e[type=pig]");
        assert_eq!(
            contents.separator().map(Component::get_string).as_deref(),
            Some("; ")
        );
        assert_eq!(
            contents.visit(&mut |selector| Some(selector.to_string())),
            Some("@e[type=pig]".to_string())
        );
        assert_eq!(
            contents.visit_styled(
                &mut |visited_style, selector| Some((visited_style.clone(), selector.to_string())),
                &style,
            ),
            Some((style, "@e[type=pig]".to_string()))
        );
        assert_eq!(contents.to_string(), "pattern{@e[type=pig]}");
    }
}
