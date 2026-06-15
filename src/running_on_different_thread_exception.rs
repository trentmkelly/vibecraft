#![allow(dead_code)]

/// Java's `RunningOnDifferentThreadException` is a singleton control-flow
/// exception. It deliberately carries no stack trace, even after
/// `fillInStackTrace`, because it is thrown on hot executor paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunningOnDifferentThreadException;

pub static RUNNING_ON_DIFFERENT_THREAD: RunningOnDifferentThreadException =
    RunningOnDifferentThreadException;

impl RunningOnDifferentThreadException {
    pub const fn stack_trace_len(self) -> usize {
        0
    }

    pub fn fill_in_stack_trace(&mut self) -> &mut Self {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/RunningOnDifferentThreadException.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_running_on_different_thread_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("extends RuntimeException"));
        assert!(JAVA_SOURCE.contains(
            "public static final RunningOnDifferentThreadException RUNNING_ON_DIFFERENT_THREAD"
        ));
        assert!(JAVA_SOURCE.contains("private RunningOnDifferentThreadException()"));
        assert!(JAVA_SOURCE.contains("this.setStackTrace(new StackTraceElement[0]);"));
        assert!(JAVA_SOURCE.contains("public synchronized Throwable fillInStackTrace()"));
        assert!(JAVA_SOURCE.contains("return this;"));
    }

    #[test]
    fn server_utility_running_on_different_thread_has_no_stack_trace() {
        assert_eq!(RUNNING_ON_DIFFERENT_THREAD.stack_trace_len(), 0);

        let mut exception = RUNNING_ON_DIFFERENT_THREAD;
        let returned = exception.fill_in_stack_trace() as *mut RunningOnDifferentThreadException;
        let original = &mut exception as *mut RunningOnDifferentThreadException;

        assert_eq!(returned, original);
        assert_eq!(exception.stack_trace_len(), 0);
    }
}
