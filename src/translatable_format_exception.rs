use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslatableContentsDebug {
    pub key: String,
    pub fallback: Option<String>,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct TranslatableFormatException {
    message: String,
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl TranslatableFormatException {
    pub fn parsing(component: &TranslatableContentsDebug, message: impl AsRef<str>) -> Self {
        Self {
            message: format!("Error parsing: {component}: {}", message.as_ref()),
            cause: None,
        }
    }

    pub fn invalid_index(component: &TranslatableContentsDebug, index: i32) -> Self {
        Self {
            message: format!("Invalid index {index} requested for {component}"),
            cause: None,
        }
    }

    pub fn while_parsing(
        component: &TranslatableContentsDebug,
        cause: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: format!("Error while parsing: {component}"),
            cause: Some(Box::new(cause)),
        }
    }
}

impl fmt::Display for TranslatableContentsDebug {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "translation{{key='{}'", self.key)?;
        if let Some(fallback) = &self.fallback {
            write!(formatter, ", fallback='{fallback}'")?;
        }
        write!(formatter, ", args=[{}]}}", self.args.join(", "))
    }
}

impl fmt::Display for TranslatableFormatException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for TranslatableFormatException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Cause;

    impl fmt::Display for Cause {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("bad percent")
        }
    }

    impl Error for Cause {}

    fn component() -> TranslatableContentsDebug {
        TranslatableContentsDebug {
            key: "chat.type.text".to_string(),
            fallback: Some("<%s> %s".to_string()),
            args: vec!["Steve".to_string(), "Hello".to_string()],
        }
    }

    #[test]
    fn translatable_format_exception_formats_parse_message_like_java() {
        let error = TranslatableFormatException::parsing(&component(), "Unsupported format: '%d'");

        assert_eq!(
            error.to_string(),
            "Error parsing: translation{key='chat.type.text', fallback='<%s> %s', args=[Steve, Hello]}: Unsupported format: '%d'"
        );
        assert!(error.source().is_none());
    }

    #[test]
    fn translatable_format_exception_formats_invalid_index_like_java() {
        let error = TranslatableFormatException::invalid_index(&component(), 2);

        assert_eq!(
            error.to_string(),
            "Invalid index 2 requested for translation{key='chat.type.text', fallback='<%s> %s', args=[Steve, Hello]}"
        );
    }

    #[test]
    fn translatable_format_exception_formats_cause_constructor_like_java() {
        let error = TranslatableFormatException::while_parsing(&component(), Cause);

        assert_eq!(
            error.to_string(),
            "Error while parsing: translation{key='chat.type.text', fallback='<%s> %s', args=[Steve, Hello]}"
        );
        assert_eq!(
            error.source().map(ToString::to_string),
            Some("bad percent".to_string())
        );
    }

    #[test]
    fn translatable_contents_debug_omits_missing_fallback_like_java_to_string() {
        let component = TranslatableContentsDebug {
            key: "key".to_string(),
            fallback: None,
            args: Vec::new(),
        };

        assert_eq!(component.to_string(), "translation{key='key', args=[]}");
    }
}
