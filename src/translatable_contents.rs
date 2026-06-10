use crate::chat_component::{Component, ResolutionContext, Style, TranslationTable};

pub const TRANSLATABLE_CONTENTS_CODEC_FIELDS: [&str; 3] = ["translate", "fallback", "with"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslatableContentsModel {
    key: String,
    fallback: Option<String>,
    args: Vec<TranslatableArgModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslatableArgModel {
    Component(Box<Component>),
    String(String),
    Number(i64),
    Boolean(bool),
    Null,
}

impl TranslatableContentsModel {
    pub fn new(
        key: impl Into<String>,
        fallback: Option<String>,
        args: Vec<TranslatableArgModel>,
    ) -> Self {
        Self {
            key: key.into(),
            fallback,
            args,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }

    pub fn args(&self) -> &[TranslatableArgModel] {
        &self.args
    }

    pub fn codec_fields(&self) -> [&'static str; 3] {
        TRANSLATABLE_CONTENTS_CODEC_FIELDS
    }

    pub fn adjust_args(args: Vec<TranslatableArgModel>) -> Option<Vec<TranslatableArgModel>> {
        (!args.is_empty()).then_some(args)
    }

    pub fn is_allowed_primitive_argument(arg: &TranslatableArgModel) -> bool {
        matches!(
            arg,
            TranslatableArgModel::String(_)
                | TranslatableArgModel::Number(_)
                | TranslatableArgModel::Boolean(_)
        )
    }

    pub fn resolve(&self, _context: &ResolutionContext, _recursion_depth: i32) -> Self {
        self.clone()
    }

    pub fn render_plain(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> String {
        let template = self.template(translations);
        match self.decompose_template(template, translations, context) {
            Ok(parts) => parts.concat(),
            Err(_) => template.to_string(),
        }
    }

    pub fn visit<T>(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
        output: &mut impl FnMut(&str) -> Option<T>,
    ) -> Option<T> {
        let template = self.template(translations);
        let parts = self
            .decompose_template(template, translations, context)
            .unwrap_or_else(|_| vec![template.to_string()]);
        for part in parts {
            if let Some(result) = output(&part) {
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
        current_style: &Style,
    ) -> Option<T> {
        self.visit(translations, context, &mut |part| {
            output(current_style, part)
        })
    }

    fn template<'a>(&'a self, translations: &'a TranslationTable) -> &'a str {
        translations
            .lookup(&self.key)
            .or(self.fallback.as_deref())
            .unwrap_or(&self.key)
    }

    fn decompose_template(
        &self,
        template: &str,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> Result<Vec<String>, ()> {
        let mut parts = Vec::new();
        let mut current = 0;
        let mut replacement_index = 0usize;

        while let Some(relative) = template[current..].find('%') {
            let start = current + relative;
            if start > current {
                parts.push(template[current..start].to_string());
            }

            let after_percent = start + 1;
            if after_percent >= template.len() {
                return Err(());
            }

            let bytes = template.as_bytes();
            if bytes[after_percent] == b'%' {
                parts.push("%".to_string());
                current = after_percent + 1;
                continue;
            }

            let mut scan = after_percent;
            while scan < template.len() && bytes[scan].is_ascii_digit() {
                scan += 1;
            }

            let index = if scan > after_percent {
                if scan >= template.len() || bytes[scan] != b'$' {
                    return Err(());
                }
                let parsed = template[after_percent..scan]
                    .parse::<usize>()
                    .map_err(|_| ())?;
                scan += 1;
                parsed.checked_sub(1).ok_or(())?
            } else {
                let index = replacement_index;
                replacement_index += 1;
                index
            };

            if scan >= template.len() || bytes[scan] != b's' {
                return Err(());
            }
            parts.push(self.argument(index, translations, context)?);
            current = scan + 1;
        }

        if current < template.len() {
            parts.push(template[current..].to_string());
        }
        Ok(parts)
    }

    fn argument(
        &self,
        index: usize,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> Result<String, ()> {
        self.args
            .get(index)
            .map(|arg| arg.render_plain(translations, context))
            .ok_or(())
    }
}

impl std::fmt::Display for TranslatableContentsModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "translation{{key='{}'", self.key)?;
        if let Some(fallback) = &self.fallback {
            write!(formatter, ", fallback='{fallback}'")?;
        }
        write!(
            formatter,
            ", args=[{}]}}",
            self.args
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl TranslatableArgModel {
    pub fn render_plain(
        &self,
        translations: &TranslationTable,
        context: &ResolutionContext,
    ) -> String {
        match self {
            Self::Component(component) => component.render_plain(translations, context),
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Null => "null".to_string(),
        }
    }
}

impl std::fmt::Display for TranslatableArgModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Component(component) => formatter.write_str(&component.get_string()),
            Self::String(value) => formatter.write_str(value),
            Self::Number(value) => write!(formatter, "{value}"),
            Self::Boolean(value) => write!(formatter, "{value}"),
            Self::Null => formatter.write_str("null"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSLATABLE_CONTENTS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/TranslatableContents.java"
    );

    fn arg(value: &str) -> TranslatableArgModel {
        TranslatableArgModel::String(value.to_string())
    }

    #[test]
    fn translatable_contents_java_source_contract_is_tracked() {
        for sentinel in [
            "public class TranslatableContents implements ComponentContents",
            "public static final Object[] NO_ARGS = new Object[0];",
            "Codec.STRING.fieldOf(\"translate\").forGetter(o -> o.key)",
            "Codec.STRING.lenientOptionalFieldOf(\"fallback\")",
            "ARG_CODEC.listOf().optionalFieldOf(\"with\")",
            "private static DataResult<Object> filterAllowedArguments",
            "return object instanceof Number || object instanceof Boolean || object instanceof String;",
            "return args.length == 0 ? Optional.empty() : Optional.of(Arrays.asList(args));",
            "private static final Pattern FORMAT_PATTERN = Pattern.compile(\"%(?:(\\\\d+)\\\\$)?([A-Za-z%]|$)\");",
            "currentLanguage.getOrDefault(this.key, this.fallback)",
            "currentLanguage.getOrDefault(this.key)",
            "this.decomposedParts = ImmutableList.of(FormattedText.of(format));",
            "if (\"%\".equals(formatType) && \"%%\".equals(formatString))",
            "if (!\"s\".equals(formatType))",
            "Integer.parseInt(possiblePositionIndex) - 1",
            "replacementIndex++",
            "return arg == null ? TEXT_NULL : FormattedText.of(arg.toString());",
            "argsCopy[i] = ComponentUtils.resolve(context, component, recursionDepth);",
            "Arrays.equals(this.args, that.args)",
            "return \"translation{key='\"",
        ] {
            assert!(
                TRANSLATABLE_CONTENTS_JAVA.contains(sentinel),
                "missing TranslatableContents Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn translatable_contents_renders_translation_fallback_key_and_percent_tokens() {
        let translations = TranslationTable::default()
            .with("chat.type.text", "<%s> %s")
            .with("example.percent", "Done %% %s");
        let context = ResolutionContext::default();

        let translated = TranslatableContentsModel::new(
            "chat.type.text",
            None,
            vec![
                TranslatableArgModel::Component(Box::new(Component::literal("Steve"))),
                arg("Hello"),
            ],
        );
        assert_eq!(
            translated.render_plain(&translations, &context),
            "<Steve> Hello"
        );

        let fallback = TranslatableContentsModel::new(
            "missing.key",
            Some("Fallback %s".to_string()),
            vec![arg("value")],
        );
        assert_eq!(
            fallback.render_plain(&TranslationTable::default(), &context),
            "Fallback value"
        );

        let key = TranslatableContentsModel::new("missing.key", None, Vec::new());
        assert_eq!(key.render_plain(&translations, &context), "missing.key");

        let percent = TranslatableContentsModel::new("example.percent", None, vec![arg("loaded")]);
        assert_eq!(
            percent.render_plain(&translations, &context),
            "Done % loaded"
        );
    }

    #[test]
    fn translatable_contents_handles_explicit_indices_nulls_and_invalid_format_fallback() {
        let translations = TranslationTable::default()
            .with("swap", "%2$s then %1$s and %2$s")
            .with("null", "value=%s")
            .with("bad", "Score %d")
            .with("bad_index", "%3$s");
        let context = ResolutionContext::default();

        let swap = TranslatableContentsModel::new("swap", None, vec![arg("first"), arg("second")]);
        assert_eq!(
            swap.render_plain(&translations, &context),
            "second then first and second"
        );

        let null = TranslatableContentsModel::new("null", None, vec![TranslatableArgModel::Null]);
        assert_eq!(null.render_plain(&translations, &context), "value=null");

        let bad =
            TranslatableContentsModel::new("bad", None, vec![TranslatableArgModel::Number(4)]);
        assert_eq!(bad.render_plain(&translations, &context), "Score %d");

        let bad_index = TranslatableContentsModel::new("bad_index", None, vec![arg("only")]);
        assert_eq!(bad_index.render_plain(&translations, &context), "%3$s");
    }

    #[test]
    fn translatable_contents_visitors_accessors_and_to_string_match_java_surface() {
        let translations = TranslationTable::default().with("key", "A %s B");
        let context = ResolutionContext::default();
        let style = Style::empty();
        let contents = TranslatableContentsModel::new(
            "key",
            Some("fallback".to_string()),
            vec![TranslatableArgModel::Boolean(true)],
        );

        assert_eq!(contents.codec_fields(), ["translate", "fallback", "with"]);
        assert_eq!(contents.key(), "key");
        assert_eq!(contents.fallback(), Some("fallback"));
        assert_eq!(contents.args(), &[TranslatableArgModel::Boolean(true)]);
        assert_eq!(
            TranslatableContentsModel::adjust_args(Vec::new()),
            None,
            "Java omits empty args from the codec"
        );
        assert!(TranslatableContentsModel::is_allowed_primitive_argument(
            &TranslatableArgModel::String("x".to_string())
        ));
        assert!(!TranslatableContentsModel::is_allowed_primitive_argument(
            &TranslatableArgModel::Null
        ));

        let mut visited = Vec::new();
        assert_eq!(
            contents.visit(&translations, &context, &mut |part| {
                visited.push(part.to_string());
                None::<()>
            }),
            None
        );
        assert_eq!(visited, vec!["A ", "true", " B"]);

        assert_eq!(
            contents.visit_styled(
                &translations,
                &context,
                &mut |visited_style, part| Some((visited_style.clone(), part.to_string())),
                &style,
            ),
            Some((style, "A ".to_string()))
        );
        assert_eq!(
            contents.to_string(),
            "translation{key='key', fallback='fallback', args=[true]}"
        );
        assert_eq!(contents.resolve(&context, 0), contents);
    }
}
