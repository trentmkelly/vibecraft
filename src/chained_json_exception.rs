#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainedJsonException {
    entries: Vec<ChainedJsonExceptionEntry>,
    message: String,
    cause: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrappedExceptionKind {
    FileNotFound,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainedJsonExceptionEntry {
    filename: Option<String>,
    json_keys: Vec<String>,
}

impl ChainedJsonException {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            entries: vec![ChainedJsonExceptionEntry::new()],
            message: message.into(),
            cause: None,
        }
    }

    pub fn with_cause(message: impl Into<String>, cause: impl Into<String>) -> Self {
        Self {
            entries: vec![ChainedJsonExceptionEntry::new()],
            message: message.into(),
            cause: Some(cause.into()),
        }
    }

    pub fn prepend_json_key(&mut self, key: impl Into<String>) {
        self.entries[0].add_json_key(key);
    }

    pub fn set_filename_and_flush(&mut self, filename: impl Into<String>) {
        self.entries[0].filename = Some(filename.into());
        self.entries.insert(0, ChainedJsonExceptionEntry::new());
    }

    pub fn message(&self) -> String {
        let entry = self
            .entries
            .last()
            .map(ToString::to_string)
            .unwrap_or_else(|| "(Unknown file)".to_string());
        format!(
            "Invalid {}: {}",
            entry,
            self.message
        )
    }

    pub fn cause(&self) -> Option<&str> {
        self.cause.as_deref()
    }

    pub fn entries(&self) -> &[ChainedJsonExceptionEntry] {
        &self.entries
    }

    pub fn for_exception(exception: ChainedOrOtherException) -> Self {
        match exception {
            ChainedOrOtherException::Chained(chained) => chained,
            ChainedOrOtherException::Other { kind, message } => {
                let message = match kind {
                    WrappedExceptionKind::FileNotFound => "File not found".to_string(),
                    WrappedExceptionKind::Other => message,
                };
                Self::with_cause(message, format!("{kind:?}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainedOrOtherException {
    Chained(ChainedJsonException),
    Other {
        kind: WrappedExceptionKind,
        message: String,
    },
}

impl ChainedJsonExceptionEntry {
    pub fn new() -> Self {
        Self {
            filename: None,
            json_keys: Vec::new(),
        }
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn json_keys(&self) -> String {
        self.json_keys.join("->")
    }

    fn add_json_key(&mut self, name: impl Into<String>) {
        self.json_keys.insert(0, name.into());
    }
}

impl std::fmt::Display for ChainedJsonExceptionEntry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.filename, self.json_keys.is_empty()) {
            (Some(filename), true) => formatter.write_str(filename),
            (Some(filename), false) => write!(formatter, "{} {}", filename, self.json_keys()),
            (None, true) => formatter.write_str("(Unknown file)"),
            (None, false) => write!(formatter, "(Unknown file) {}", self.json_keys()),
        }
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::{
        ChainedJsonException, ChainedJsonExceptionEntry, ChainedOrOtherException,
        WrappedExceptionKind,
    };

    const CHAINED_JSON_EXCEPTION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/ChainedJsonException.java");

    #[test]
    fn chained_json_exception_matches_java_source_shape() {
        for sentinel in [
            "public class ChainedJsonException extends IOException",
            "private final List<ChainedJsonException.Entry> entries = Lists.newArrayList();",
            "private final String message;",
            "this.entries.add(new ChainedJsonException.Entry());",
            "public void prependJsonKey(final String key)",
            "this.entries.get(0).addJsonKey(key);",
            "public void setFilenameAndFlush(final String filename)",
            "this.entries.get(0).filename = filename;",
            "this.entries.add(0, new ChainedJsonException.Entry());",
            "return \"Invalid \" + this.entries.get(this.entries.size() - 1) + \": \" + this.message;",
            "if (e instanceof FileNotFoundException)",
            "message = \"File not found\";",
            "this.jsonKeys.add(0, name);",
            "return StringUtils.join(this.jsonKeys, \"->\");",
        ] {
            assert!(
                CHAINED_JSON_EXCEPTION_JAVA.contains(sentinel),
                "missing ChainedJsonException.java sentinel {sentinel}"
            );
        }
    }

    #[test]
    fn chained_json_exception_formats_unknown_and_file_paths_like_java() {
        let mut error = ChainedJsonException::new("expected string");
        assert_eq!(error.message(), "Invalid (Unknown file): expected string");

        error.prepend_json_key("name");
        error.prepend_json_key("display");
        assert_eq!(
            error.message(),
            "Invalid (Unknown file) display->name: expected string"
        );

        error.set_filename_and_flush("pack.mcmeta");
        assert_eq!(
            error.message(),
            "Invalid pack.mcmeta display->name: expected string"
        );
        assert_eq!(error.entries().len(), 2);
        assert_eq!(error.entries()[0].to_string(), "(Unknown file)");

        error.prepend_json_key("pack");
        assert_eq!(error.entries()[0].to_string(), "(Unknown file) pack");
        assert_eq!(
            error.message(),
            "Invalid pack.mcmeta display->name: expected string"
        );
    }

    #[test]
    fn chained_json_exception_for_exception_matches_java_special_cases() {
        let chained = ChainedJsonException::new("already chained");
        assert_eq!(
            ChainedJsonException::for_exception(ChainedOrOtherException::Chained(
                chained.clone()
            )),
            chained
        );

        let missing = ChainedJsonException::for_exception(ChainedOrOtherException::Other {
            kind: WrappedExceptionKind::FileNotFound,
            message: "original path text".to_string(),
        });
        assert_eq!(missing.message(), "Invalid (Unknown file): File not found");
        assert_eq!(missing.cause(), Some("FileNotFound"));

        let other = ChainedJsonException::for_exception(ChainedOrOtherException::Other {
            kind: WrappedExceptionKind::Other,
            message: "bad token".to_string(),
        });
        assert_eq!(other.message(), "Invalid (Unknown file): bad token");
        assert_eq!(other.cause(), Some("Other"));
    }

    #[test]
    fn chained_json_exception_entry_accessors_match_java() {
        let entry = ChainedJsonExceptionEntry::new();
        assert_eq!(entry.filename(), None);
        assert_eq!(entry.json_keys(), "");
        assert_eq!(entry.to_string(), "(Unknown file)");
    }
}
