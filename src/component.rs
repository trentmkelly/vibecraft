use crate::chat_component::{
    Component, ComponentArgument, ComponentContent, NbtSource, ObjectContent, ResolutionContext,
    Style, TranslationTable,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableComponentSurface {
    component: Component,
    visual_order_text: String,
    decomposed_language: Option<String>,
}

impl Component {
    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn get_contents(&self) -> &ComponentContent {
        &self.content
    }

    pub fn get_siblings(&self) -> &[Component] {
        &self.siblings
    }

    pub fn get_string_limit(&self, limit: usize) -> String {
        self.get_string().chars().take(limit).collect()
    }

    pub fn try_collapse_to_string(&self) -> Option<&str> {
        match &self.content {
            ComponentContent::Literal(text)
                if self.siblings.is_empty() && self.style == Style::empty() =>
            {
                Some(text)
            }
            _ => None,
        }
    }

    pub fn plain_copy(&self) -> Self {
        Self {
            content: self.content.clone(),
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    pub fn copy_component(&self) -> Self {
        self.clone()
    }

    pub fn visit_plain<T>(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
        output: &mut impl FnMut(&str) -> Option<T>,
    ) -> Option<T> {
        let self_text = self.content_only().render_plain(translations, context);
        if !self_text.is_empty() {
            if let Some(result) = output(&self_text) {
                return Some(result);
            }
        }
        for sibling in &self.siblings {
            if let Some(result) = sibling.visit_plain(translations, context, output) {
                return Some(result);
            }
        }
        None
    }

    pub fn visit_styled<T>(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
        output: &mut impl FnMut(&Style, &str) -> Option<T>,
        parent_style: &Style,
    ) -> Option<T> {
        let self_style = self.style.apply_to(parent_style);
        let self_text = self.content_only().render_plain(translations, context);
        if !self_text.is_empty() {
            if let Some(result) = output(&self_style, &self_text) {
                return Some(result);
            }
        }
        for sibling in &self.siblings {
            if let Some(result) = sibling.visit_styled(translations, context, output, &self_style) {
                return Some(result);
            }
        }
        None
    }

    pub fn to_flat_list(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
        root_style: &Style,
    ) -> Vec<Component> {
        let mut result = Vec::new();
        self.visit_styled(
            translations,
            context,
            &mut |style, contents| {
                result.push(Component::literal(contents.to_string()).styled(style.clone()));
                None::<()>
            },
            root_style,
        );
        result
    }

    pub fn contains_component(
        &self,
        other: &Component,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> bool {
        if self == other {
            return true;
        }
        let flat = self.to_flat_list(translations, context, &Style::empty());
        let other_flat = other.to_flat_list(translations, context, &self.style);
        !other_flat.is_empty()
            && flat
                .windows(other_flat.len())
                .any(|window| window == other_flat.as_slice())
    }

    pub fn null_to_empty(text: Option<&str>) -> Self {
        text.map(Component::literal)
            .unwrap_or_else(Component::empty)
    }

    pub fn translatable_escape(key: impl Into<String>, args: Vec<TranslationArg>) -> Self {
        Self::translatable(
            key,
            args.into_iter()
                .map(TranslationArg::into_component_argument)
                .collect(),
        )
    }

    pub fn keybind_component(name: impl Into<String>) -> Self {
        Self {
            content: ComponentContent::Keybind(name.into()),
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    pub fn nbt_component(
        path: impl Into<String>,
        source: NbtSource,
        interpret: bool,
        plain: bool,
        separator: Option<Component>,
    ) -> Self {
        Self {
            content: ComponentContent::Nbt {
                path: path.into(),
                source,
                interpret,
                plain,
                separator: separator.map(Box::new),
            },
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    pub fn score_component(name: impl Into<String>, objective: impl Into<String>) -> Self {
        Self {
            content: ComponentContent::Score {
                name: name.into(),
                objective: objective.into(),
            },
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    pub fn selector_component(selector: impl Into<String>, separator: Option<Component>) -> Self {
        Self {
            content: ComponentContent::Selector {
                selector: selector.into(),
                separator: separator.map(Box::new),
            },
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    pub fn object_component(info: ObjectContent, _fallback: Option<Component>) -> Self {
        Self {
            content: ComponentContent::Object(info),
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }

    fn content_only(&self) -> Self {
        Self {
            content: self.content.clone(),
            style: Style::empty(),
            siblings: Vec::new(),
        }
    }
}

impl MutableComponentSurface {
    pub fn create(contents: ComponentContent) -> Self {
        Self {
            component: Component {
                content: contents,
                style: Style::empty(),
                siblings: Vec::new(),
            },
            visual_order_text: String::new(),
            decomposed_language: None,
        }
    }

    pub fn from_component(component: Component) -> Self {
        Self {
            component,
            visual_order_text: String::new(),
            decomposed_language: None,
        }
    }

    pub fn component(&self) -> &Component {
        &self.component
    }

    pub fn get_contents(&self) -> &ComponentContent {
        &self.component.content
    }

    pub fn get_siblings(&self) -> &[Component] {
        &self.component.siblings
    }

    pub fn get_style(&self) -> &Style {
        &self.component.style
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.component.style = style;
        self
    }

    pub fn append_text(&mut self, text: impl Into<String>) -> &mut Self {
        let text = text.into();
        if !text.is_empty() {
            self.append(Component::literal(text));
        }
        self
    }

    pub fn append(&mut self, component: Component) -> &mut Self {
        self.component.siblings.push(component);
        self
    }

    pub fn with_style_patch(&mut self, patch: &Style) -> &mut Self {
        let style = patch.apply_to(&self.component.style);
        self.set_style(style)
    }

    pub fn with_style_update(&mut self, updater: impl FnOnce(Style) -> Style) -> &mut Self {
        let style = updater(self.component.style.clone());
        self.set_style(style)
    }

    pub fn with_color(&mut self, color: u32) -> &mut Self {
        self.set_style(
            self.component
                .style
                .clone()
                .with_color(crate::chat_component::TextColor::from_rgb(color)),
        )
    }

    pub fn get_visual_order_text(
        &mut self,
        language: impl Into<String>,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> &str {
        let language = language.into();
        if self.decomposed_language.as_deref() != Some(language.as_str()) {
            self.visual_order_text = self.component.render_plain(translations, context);
            self.decomposed_language = Some(language);
        }
        &self.visual_order_text
    }
}

impl std::fmt::Display for MutableComponentSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}",
            component_contents_label(&self.component.content)
        )?;
        let has_style = self.component.style != Style::empty();
        let has_siblings = !self.component.siblings.is_empty();
        if has_style || has_siblings {
            formatter.write_str("[")?;
            if has_style {
                write!(formatter, "style={:?}", self.component.style)?;
            }
            if has_style && has_siblings {
                formatter.write_str(", ")?;
            }
            if has_siblings {
                write!(formatter, "siblings={:?}", self.component.siblings)?;
            }
            formatter.write_str("]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationArg {
    Component(Box<Component>),
    String(String),
    Number(i32),
    Long(i64),
    Boolean(bool),
    Null,
    Other(String),
}

impl TranslationArg {
    fn into_component_argument(self) -> ComponentArgument {
        match self {
            Self::Component(component) => ComponentArgument::Component(component),
            Self::String(value) | Self::Other(value) => ComponentArgument::String(value),
            Self::Number(value) => ComponentArgument::Number(value),
            Self::Long(value) => ComponentArgument::Long(value),
            Self::Boolean(value) => ComponentArgument::Boolean(value),
            Self::Null => ComponentArgument::Null,
        }
    }
}

pub fn translation_arg(value: impl Into<String>) -> Component {
    Component::literal(value.into())
}

fn component_contents_label(content: &ComponentContent) -> String {
    match content {
        ComponentContent::Literal(text) => format!("literal{{{text}}}"),
        ComponentContent::Translatable {
            key,
            fallback,
            args,
        } => {
            let fallback = fallback
                .as_ref()
                .map(|fallback| format!(", fallback='{fallback}'"))
                .unwrap_or_default();
            format!("translation{{key='{key}'{fallback}, args={args:?}}}")
        }
        ComponentContent::Selector { selector, .. } => format!("pattern{{{selector}}}"),
        ComponentContent::Score { name, objective } => {
            format!("score{{name='{name}', objective='{objective}'}}")
        }
        ComponentContent::Keybind(key) => format!("keybind{{{key}}}"),
        ComponentContent::Nbt { path, .. } => format!("nbt{{{path}}}"),
        ComponentContent::Object(object) => format!("object{{{object:?}}}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::{FontDescription, TextColor};

    const COMPONENT_JAVA: &str =
        include_str!("../../decompiled-server-26.1.2/net/minecraft/network/chat/Component.java");
    const MUTABLE_COMPONENT_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/MutableComponent.java"
    );

    #[test]
    fn component_java_source_contract_is_tracked() {
        for sentinel in [
            "public interface Component extends Message, FormattedText",
            "Style getStyle();",
            "ComponentContents getContents();",
            "default String getString(final int limit)",
            "List<Component> getSiblings();",
            "default @Nullable String tryCollapseToString()",
            "default MutableComponent plainCopy()",
            "default MutableComponent copy()",
            "FormattedCharSequence getVisualOrderText();",
            "Style selfStyle = this.getStyle().applyTo(parentStyle);",
            "Optional<T> selfResult = this.getContents().visit(output, selfStyle);",
            "default List<Component> toFlatList(final Style rootStyle)",
            "return Collections.indexOfSubList(flat, otherFlat) != -1;",
            "static Component nullToEmpty(final @Nullable String text)",
            "static MutableComponent literal(final String text)",
            "static MutableComponent translatableEscape(final String key, final Object... args)",
            "static MutableComponent nbt(",
            "static MutableComponent object(final ObjectInfo info, final Component fallback)",
            "static Component translationArg(final UUID uuid)",
        ] {
            assert!(
                COMPONENT_JAVA.contains(sentinel),
                "missing Component Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn mutable_component_java_source_contract_is_tracked() {
        for sentinel in [
            "public final class MutableComponent implements Component",
            "private final ComponentContents contents;",
            "private final List<Component> siblings;",
            "private Style style;",
            "private FormattedCharSequence visualOrderText = FormattedCharSequence.EMPTY;",
            "public static MutableComponent create(final ComponentContents contents)",
            "public MutableComponent setStyle(final Style style)",
            "public MutableComponent append(final String text)",
            "return text.isEmpty() ? this : this.append(Component.literal(text));",
            "public MutableComponent append(final Component component)",
            "public MutableComponent withStyle(final UnaryOperator<Style> updater)",
            "public MutableComponent withStyle(final Style patch)",
            "public MutableComponent withColor(final int color)",
            "public FormattedCharSequence getVisualOrderText()",
            "currentLanguage.getVisualOrder(this)",
            "this.contents.equals(that.contents) && this.style.equals(that.style) && this.siblings.equals(that.siblings)",
            "return 31 * result + this.siblings.hashCode();",
            "StringBuilder result = new StringBuilder(this.contents.toString());",
        ] {
            assert!(
                MUTABLE_COMPONENT_JAVA.contains(sentinel),
                "missing MutableComponent Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn component_accessors_copy_collapse_limit_and_factories_match_java_surface() {
        let styled = Component::literal("root")
            .styled(Style::empty().with_bold(true))
            .append(Component::literal(" child"));

        assert_eq!(styled.get_style().bold, Some(true));
        assert_eq!(styled.get_siblings().len(), 1);
        assert_eq!(styled.get_string_limit(6), "root c");
        assert_eq!(
            Component::literal("plain").try_collapse_to_string(),
            Some("plain")
        );
        assert_eq!(styled.try_collapse_to_string(), None);

        let plain_copy = styled.plain_copy();
        assert_eq!(plain_copy.get_string(), "root");
        assert_eq!(plain_copy.get_style(), &Style::empty());
        assert!(plain_copy.get_siblings().is_empty());
        assert_eq!(styled.copy_component(), styled);

        assert_eq!(Component::null_to_empty(None).get_string(), "");
        assert_eq!(Component::null_to_empty(Some("x")).get_string(), "x");
        assert_eq!(
            Component::keybind_component("key.jump").get_string(),
            "key.jump"
        );
        assert_eq!(
            Component::score_component("Steve", "kills").get_contents(),
            &ComponentContent::Score {
                name: "Steve".to_string(),
                objective: "kills".to_string(),
            }
        );
        assert_eq!(
            Component::selector_component("@a", Some(Component::literal(" | "))).get_string(),
            "@a"
        );
        assert_eq!(
            Component::object_component(
                ObjectContent::AtlasSprite {
                    atlas: "minecraft:blocks".to_string(),
                    sprite: "minecraft:stone".to_string(),
                },
                None,
            )
            .get_string(),
            "[stone]"
        );
        assert_eq!(
            translation_arg("minecraft:stone").get_string(),
            "minecraft:stone"
        );
    }

    #[test]
    fn component_visitors_flat_list_contains_and_translation_escape_match_java() {
        let translations = TranslationTable::default().with("chat.type.text", "<%s> %s");
        let context = ResolutionContext::default();
        let root_style = Style::empty().with_color(TextColor::from_rgb(0x00aa00));
        let component = Component::literal("A")
            .styled(Style::empty().with_bold(true))
            .append(Component::literal("B"))
            .append(Component::literal("C").styled(Style::empty().with_italic(true)));

        let mut plain_parts = Vec::new();
        assert_eq!(
            component.visit_plain(&translations, &context, &mut |text| {
                plain_parts.push(text.to_string());
                None::<()>
            }),
            None
        );
        assert_eq!(plain_parts, vec!["A", "B", "C"]);

        let flat = component.to_flat_list(&translations, &context, &root_style);
        assert_eq!(
            flat.iter().map(Component::get_string).collect::<Vec<_>>(),
            vec!["A", "B", "C"]
        );
        assert_eq!(flat[0].style.bold, Some(true));
        assert_eq!(flat[0].style.color, root_style.color);
        assert_eq!(flat[2].style.italic, Some(true));
        assert!(component.contains_component(
            &Component::literal("B")
                .append(Component::literal("C").styled(Style::empty().with_italic(true))),
            &translations,
            &context,
        ));

        let escaped = Component::translatable_escape(
            "chat.type.text",
            vec![
                TranslationArg::Other("custom-object".to_string()),
                TranslationArg::Component(Box::new(Component::literal("hello"))),
            ],
        );
        assert_eq!(
            escaped.render_plain(&translations, &context),
            "<custom-object> hello"
        );
    }

    #[test]
    fn mutable_component_mutation_visual_order_and_display_match_java_surface() {
        let mut mutable =
            MutableComponentSurface::create(ComponentContent::Literal("Root".to_string()));
        assert_eq!(
            mutable.get_contents(),
            &ComponentContent::Literal("Root".to_string())
        );
        assert!(mutable.get_siblings().is_empty());

        mutable
            .append_text("")
            .append_text(" child")
            .append(Component::literal(" tail"));
        assert_eq!(
            mutable
                .get_siblings()
                .iter()
                .map(Component::get_string)
                .collect::<Vec<_>>(),
            vec![" child", " tail"]
        );

        mutable
            .set_style(Style::empty().with_italic(true))
            .with_style_patch(&Style::empty().with_bold(true))
            .with_style_update(|style| style.with_font(FontDescription::default_resource()))
            .with_color(0xff00aa);
        assert_eq!(mutable.get_style().italic, Some(true));
        assert_eq!(mutable.get_style().bold, Some(true));
        assert_eq!(
            mutable.get_style().font,
            Some(FontDescription::default_resource())
        );
        assert_eq!(
            mutable.get_style().color,
            Some(TextColor::from_rgb(0xff00aa))
        );

        assert_eq!(
            mutable.get_visual_order_text(
                "en_us",
                &TranslationTable::default(),
                &ResolutionContext::default(),
            ),
            "Root child tail"
        );
        assert!(mutable.to_string().contains("literal{Root}[style="));
        assert!(mutable.to_string().contains("siblings="));

        let same = mutable.clone();
        assert_eq!(mutable, same);
    }
}
