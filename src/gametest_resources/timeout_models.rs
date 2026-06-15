use super::{GameTestExceptionBase, GameTestExceptionModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestTimeoutExceptionModel {
    pub base: GameTestExceptionBase,
    pub message: String,
}

impl GameTestTimeoutExceptionModel {
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            base: GameTestExceptionBase::new(message.clone()),
            message,
        }
    }

    pub fn runtime_message(&self) -> &str {
        self.base.runtime_message()
    }
}

impl GameTestExceptionModel for GameTestTimeoutExceptionModel {
    type Description = String;

    fn base_exception(&self) -> &GameTestExceptionBase {
        &self.base
    }

    fn get_description(&self) -> Self::Description {
        self.message.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownGameTestExceptionModel {
    pub base: GameTestExceptionBase,
    pub reason_message: String,
}

impl UnknownGameTestExceptionModel {
    pub fn new(reason_message: impl Into<String>) -> Self {
        let reason_message = reason_message.into();
        Self {
            base: GameTestExceptionBase::new(reason_message.clone()),
            reason_message,
        }
    }

    pub fn runtime_message(&self) -> &str {
        self.base.runtime_message()
    }
}

impl GameTestExceptionModel for UnknownGameTestExceptionModel {
    type Description = (&'static str, String);

    fn base_exception(&self) -> &GameTestExceptionBase {
        &self.base
    }

    fn get_description(&self) -> Self::Description {
        ("test.error.unknown", self.reason_message.clone())
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const UNKNOWN_GAME_TEST_EXCEPTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/UnknownGameTestException.java");
    const FRAMEWORK_PACKAGE_INFO_JAVA: &str = vibecraft_java_source!("/net/minecraft/gametest/framework/package-info.java");
    const GAMETEST_PACKAGE_INFO_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/gametest/package-info.java");

    #[test]
    fn unknown_gametest_exception_matches_java_source_shape() {
        assert_eq!(UNKNOWN_GAME_TEST_EXCEPTION_JAVA.lines().count(), 17);
        for sentinel in [
            "public class UnknownGameTestException extends GameTestException",
            "private final Throwable reason;",
            "public UnknownGameTestException(final Throwable reason)",
            "super(reason.getMessage());",
            "this.reason = reason;",
            "Component.translatable(\"test.error.unknown\", this.reason.getMessage())",
        ] {
            assert!(
                UNKNOWN_GAME_TEST_EXCEPTION_JAVA.contains(sentinel),
                "missing UnknownGameTestException sentinel {sentinel}"
            );
        }
    }

    #[test]
    fn unknown_gametest_exception_runtime_message_and_description_match_java() {
        let exception = UnknownGameTestExceptionModel::new("unexpected failure");

        assert_eq!(exception.runtime_message(), "unexpected failure");
        assert_eq!(
            exception.base_exception().runtime_message(),
            "unexpected failure"
        );
        assert_eq!(
            exception.get_description(),
            ("test.error.unknown", "unexpected failure".to_string())
        );
    }

    #[test]
    fn gametest_package_info_files_are_null_marked_like_java() {
        for (name, source) in [
            ("framework", FRAMEWORK_PACKAGE_INFO_JAVA),
            ("gametest", GAMETEST_PACKAGE_INFO_JAVA),
        ] {
            assert_eq!(source.lines().count(), 4, "{name} package-info line count");
            assert!(
                source.contains("@NullMarked"),
                "{name} package-info missing NullMarked annotation"
            );
            assert!(
                source.contains("import org.jspecify.annotations.NullMarked;"),
                "{name} package-info missing NullMarked import"
            );
        }
        assert!(FRAMEWORK_PACKAGE_INFO_JAVA.contains("package net.minecraft.gametest.framework;"));
        assert!(GAMETEST_PACKAGE_INFO_JAVA.contains("package net.minecraft.gametest;"));
    }
}
