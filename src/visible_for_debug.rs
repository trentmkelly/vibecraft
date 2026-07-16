//! Marker corresponding to Minecraft's `@VisibleForDebug` annotation.

#![allow(dead_code)]

/// Marker type for APIs exposed solely for diagnostics and debug tooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VisibleForDebug;

#[cfg(test)]
mod tests {
    use super::VisibleForDebug;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn visible_for_debug_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/VisibleForDebug.java");
        assert_eq!(JAVA.lines().count(), 4);
        assert!(JAVA.contains("public @interface VisibleForDebug"));
    }

    #[test]
    fn visible_for_debug_is_an_empty_marker() {
        assert_eq!(VisibleForDebug, VisibleForDebug);
    }
}
