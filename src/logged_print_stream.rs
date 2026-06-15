#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintlnArgument<'a> {
    String(Option<&'a str>),
    Object(Option<&'a str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggedPrintStream {
    name: String,
}

impl LoggedPrintStream {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn auto_flush(&self) -> bool {
        false
    }

    pub const fn charset(&self) -> &'static str {
        "UTF-8"
    }

    pub fn println(&self, argument: PrintlnArgument<'_>) -> String {
        match argument {
            PrintlnArgument::String(value) => self.log_line(value),
            PrintlnArgument::Object(value) => self.log_line(Some(value.unwrap_or("null"))),
        }
    }

    pub fn log_line(&self, out: Option<&str>) -> String {
        format!("[{}]: {}", self.name, slf4j_argument(out))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackTraceElement {
    pub file_name: String,
    pub line_number: i32,
}

impl StackTraceElement {
    pub fn new(file_name: impl Into<String>, line_number: i32) -> Self {
        Self {
            file_name: file_name.into(),
            line_number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugLoggedPrintStream {
    base: LoggedPrintStream,
}

impl DebugLoggedPrintStream {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: LoggedPrintStream::new(name),
        }
    }

    pub fn name(&self) -> &str {
        self.base.name()
    }

    pub const fn auto_flush(&self) -> bool {
        false
    }

    pub const fn charset(&self) -> &'static str {
        "UTF-8"
    }

    pub fn log_line_with_stack_trace(
        &self,
        out: Option<&str>,
        stack_trace: &[StackTraceElement],
    ) -> Option<String> {
        let index = debug_stack_trace_index(stack_trace.len());
        let stack_trace_element = stack_trace.get(index)?;
        Some(format!(
            "[{}]@.({}:{}): {}",
            self.name(),
            stack_trace_element.file_name,
            stack_trace_element.line_number,
            slf4j_argument(out)
        ))
    }
}

pub const fn debug_stack_trace_index(stack_trace_len: usize) -> usize {
    if stack_trace_len < 3 {
        stack_trace_len
    } else {
        3
    }
}

fn slf4j_argument(value: Option<&str>) -> &str {
    value.unwrap_or("null")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const LOGGED_PRINT_STREAM_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/LoggedPrintStream.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const DEBUG_LOGGED_PRINT_STREAM_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/DebugLoggedPrintStream.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_logged_print_stream_matches_java_source_shape() {
        assert!(LOGGED_PRINT_STREAM_JAVA.contains("extends PrintStream"));
        assert!(LOGGED_PRINT_STREAM_JAVA.contains("protected final String name;"));
        assert!(LOGGED_PRINT_STREAM_JAVA.contains("super(out, false, StandardCharsets.UTF_8);"));
        assert!(LOGGED_PRINT_STREAM_JAVA.contains("this.name = name;"));
        assert!(LOGGED_PRINT_STREAM_JAVA
            .contains("public void println(final @Nullable String string)"));
        assert!(LOGGED_PRINT_STREAM_JAVA
            .contains("public void println(final @Nullable Object object)"));
        assert!(LOGGED_PRINT_STREAM_JAVA.contains("this.logLine(String.valueOf(object));"));
        assert!(LOGGED_PRINT_STREAM_JAVA.contains("LOGGER.info(\"[{}]: {}\", this.name, out);"));
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_debug_logged_print_stream_matches_java_source_shape() {
        assert!(DEBUG_LOGGED_PRINT_STREAM_JAVA.contains("extends LoggedPrintStream"));
        assert!(DEBUG_LOGGED_PRINT_STREAM_JAVA.contains("super(name, out);"));
        assert!(
            DEBUG_LOGGED_PRINT_STREAM_JAVA.contains("StackTraceElement[] stackTrace = Thread.currentThread().getStackTrace();")
        );
        assert!(DEBUG_LOGGED_PRINT_STREAM_JAVA
            .contains("stackTrace[Math.min(3, stackTrace.length)]"));
        assert!(DEBUG_LOGGED_PRINT_STREAM_JAVA
            .contains("LOGGER.info(\"[{}]@.({}:{}): {}\""));
    }

    #[test]
    fn server_utility_logged_print_stream_routes_println_to_base_log_line() {
        let stream = LoggedPrintStream::new("STDOUT");

        assert_eq!(stream.name(), "STDOUT");
        assert!(!stream.auto_flush());
        assert_eq!(stream.charset(), "UTF-8");
        assert_eq!(
            stream.println(PrintlnArgument::String(Some("hello"))),
            "[STDOUT]: hello"
        );
        assert_eq!(stream.println(PrintlnArgument::String(None)), "[STDOUT]: null");
        assert_eq!(stream.println(PrintlnArgument::Object(None)), "[STDOUT]: null");
    }

    #[test]
    fn server_utility_debug_logged_print_stream_uses_java_stack_frame_index() {
        let stream = DebugLoggedPrintStream::new("STDERR");
        let stack_trace = [
            StackTraceElement::new("Thread.java", 1),
            StackTraceElement::new("DebugLoggedPrintStream.java", 18),
            StackTraceElement::new("PrintStream.java", 42),
            StackTraceElement::new("Main.java", 100),
            StackTraceElement::new("Launcher.java", 7),
        ];

        assert_eq!(debug_stack_trace_index(stack_trace.len()), 3);
        assert_eq!(
            stream
                .log_line_with_stack_trace(Some("boom"), &stack_trace)
                .as_deref(),
            Some("[STDERR]@.(Main.java:100): boom")
        );
        assert_eq!(
            stream.log_line_with_stack_trace(None, &stack_trace).as_deref(),
            Some("[STDERR]@.(Main.java:100): null")
        );
    }
}
