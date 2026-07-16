//! Modification confidence reporting matching Minecraft's `ModCheck`.

#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    ProbablyNot,
    VeryLikely,
    Definitely,
}

impl Confidence {
    pub const fn description(self) -> &'static str {
        match self {
            Self::ProbablyNot => "Probably not.",
            Self::VeryLikely => "Very likely;",
            Self::Definitely => "Definitely;",
        }
    }

    pub const fn should_report_as_modified(self) -> bool {
        !matches!(self, Self::ProbablyNot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModCheck {
    pub confidence: Confidence,
    pub description: String,
}

impl ModCheck {
    pub fn identify(
        expected_brand: &str,
        actual_brand: impl FnOnce() -> String,
        component: &str,
        signature_present: bool,
    ) -> Self {
        let mod_brand = actual_brand();
        if expected_brand != mod_brand {
            Self {
                confidence: Confidence::Definitely,
                description: format!("{component} brand changed to '{mod_brand}'"),
            }
        } else if !signature_present {
            Self {
                confidence: Confidence::VeryLikely,
                description: format!("{component} jar signature invalidated"),
            }
        } else {
            Self {
                confidence: Confidence::ProbablyNot,
                description: format!("{component} jar signature and brand is untouched"),
            }
        }
    }

    pub const fn should_report_as_modified(&self) -> bool {
        self.confidence.should_report_as_modified()
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            confidence: self.confidence.max(other.confidence),
            description: format!("{}; {}", self.description, other.description),
        }
    }

    pub fn full_description(&self) -> String {
        format!("{} {}", self.confidence.description(), self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::{Confidence, ModCheck};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn mod_check_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/ModCheck.java");
        assert_eq!(JAVA.lines().count(), 45);
        for fragment in [
            "public record ModCheck(ModCheck.Confidence confidence, String description)",
            "component + \" brand changed to '\" + mod + \"'\"",
            "jar signature invalidated",
            "jar signature and brand is untouched",
            "public boolean shouldReportAsModified()",
            "this.description + \"; \" + other.description",
            "public String fullDescription()",
            "PROBABLY_NOT(\"Probably not.\", false)",
            "VERY_LIKELY(\"Very likely;\", true)",
            "DEFINITELY(\"Definitely;\", true)",
        ] {
            assert!(JAVA.contains(fragment), "missing ModCheck source fragment: {fragment}");
        }
    }

    #[test]
    fn mod_check_identifies_merges_and_formats_confidence_like_vanilla() {
        let untouched = ModCheck::identify("vanilla", || "vanilla".to_string(), "server", true);
        assert_eq!(untouched.confidence, Confidence::ProbablyNot);
        assert!(!untouched.should_report_as_modified());
        assert_eq!(untouched.full_description(), "Probably not. server jar signature and brand is untouched");

        let unsigned = ModCheck::identify("vanilla", || "vanilla".to_string(), "server", false);
        assert_eq!(unsigned.confidence, Confidence::VeryLikely);
        assert!(unsigned.should_report_as_modified());

        let branded = ModCheck::identify("vanilla", || "fabric".to_string(), "server", true);
        assert_eq!(branded.confidence, Confidence::Definitely);
        assert_eq!(branded.description, "server brand changed to 'fabric'");

        let merged = unsigned.merge(&branded);
        assert_eq!(merged.confidence, Confidence::Definitely);
        assert_eq!(merged.description, "server jar signature invalidated; server brand changed to 'fabric'");
        assert_eq!(merged.full_description(), "Definitely; server jar signature invalidated; server brand changed to 'fabric'");
    }
}
