//! Shared Mojang links and Realms extension URL construction.

#![allow(dead_code)]

use crate::network::codec::Uuid;

pub const GDPR: &str = "https://aka.ms/MinecraftGDPR";
pub const EULA: &str = "https://aka.ms/MinecraftEULA";
pub const PRIVACY_STATEMENT: &str = "http://go.microsoft.com/fwlink/?LinkId=521839";
pub const ATTRIBUTION: &str = "https://aka.ms/MinecraftJavaAttribution";
pub const LICENSES: &str = "https://aka.ms/MinecraftJavaLicenses";
pub const BUY_MINECRAFT_JAVA: &str = "https://aka.ms/BuyMinecraftJava";
pub const ACCOUNT_SETTINGS: &str = "https://aka.ms/JavaAccountSettings";
pub const SNAPSHOT_FEEDBACK: &str = "https://aka.ms/snapshotfeedback?ref=game";
pub const RELEASE_FEEDBACK: &str = "https://aka.ms/javafeedback?ref=game";
pub const SNAPSHOT_BUGS_FEEDBACK: &str = "https://aka.ms/snapshotbugs?ref=game";
pub const GENERAL_HELP: &str = "https://aka.ms/Minecraft-Support";
pub const ACCESSIBILITY_HELP: &str = "https://aka.ms/MinecraftJavaAccessibility";
pub const REPORTING_HELP: &str = "https://aka.ms/aboutjavareporting";
pub const SUSPENSION_HELP: &str = "https://aka.ms/mcjavamoderation";
pub const BLOCKING_HELP: &str = "https://aka.ms/javablocking";
pub const SYMLINK_HELP: &str = "https://aka.ms/MinecraftSymLinks";
pub const START_REALMS_TRIAL: &str = "https://aka.ms/startjavarealmstrial";
pub const BUY_REALMS: &str = "https://aka.ms/BuyJavaRealms";
pub const REALMS_TERMS: &str = "https://aka.ms/MinecraftRealmsTerms";
pub const REALMS_CONTENT_CREATION: &str = "https://aka.ms/MinecraftRealmsContentCreator";
pub const EXTEND_REALMS_LINK: &str = "https://aka.ms/ExtendJavaRealms";
pub const INTENTIONAL_GAME_DESIGN_BUG_ID: &str = "MCPE-28723";
pub const INTENTIONAL_GAME_DESIGN_BUG: &str = "https://bugs.mojang.com/browse/MCPE-28723";

pub fn extend_realms(subscription_id: Option<&str>, profile_id: Uuid, trial: bool) -> String {
    let Some(subscription_id) = subscription_id else {
        return EXTEND_REALMS_LINK.to_string();
    };
    format!(
        "{}&ref={}",
        extend_realms_for_profile(Some(subscription_id), profile_id),
        if trial { "expiredTrial" } else { "expiredRealm" }
    )
}

pub fn extend_realms_for_profile(subscription_id: Option<&str>, profile_id: Uuid) -> String {
    let Some(subscription_id) = subscription_id else {
        return EXTEND_REALMS_LINK.to_string();
    };
    format!(
        "{EXTEND_REALMS_LINK}?subscriptionId={subscription_id}&profileId={}",
        undashed_uuid(profile_id)
    )
}

fn undashed_uuid(uuid: Uuid) -> String {
    uuid.0.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn common_links_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/CommonLinks.java");
        assert_eq!(JAVA.lines().count(), 44);
        for fragment in [
            "public static final URI GDPR = URI.create(\"https://aka.ms/MinecraftGDPR\")",
            "public static final URI EULA = URI.create(\"https://aka.ms/MinecraftEULA\")",
            "public static final String EXTEND_REALMS_LINK = \"https://aka.ms/ExtendJavaRealms\"",
            "public static String extendRealms(final @Nullable String subscriptionId, final UUID profileId, final boolean trial)",
            "&ref=\" + (trial ? \"expiredTrial\" : \"expiredRealm\")",
            "UndashedUuid.toString(profileId)",
        ] {
            assert!(JAVA.contains(fragment), "missing CommonLinks source fragment: {fragment}");
        }
    }

    #[test]
    fn common_links_match_java_url_branches_and_uuid_format() {
        let profile = Uuid([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        assert_eq!(extend_realms(None, profile, true), EXTEND_REALMS_LINK);
        assert_eq!(
            extend_realms_for_profile(None, profile),
            EXTEND_REALMS_LINK
        );
        let base = "https://aka.ms/ExtendJavaRealms?subscriptionId=sub-42&profileId=000102030405060708090a0b0c0d0e0f";
        assert_eq!(extend_realms_for_profile(Some("sub-42"), profile), base);
        assert_eq!(
            extend_realms(Some("sub-42"), profile, true),
            format!("{base}&ref=expiredTrial")
        );
        assert_eq!(
            extend_realms(Some("sub-42"), profile, false),
            format!("{base}&ref=expiredRealm")
        );
        assert_eq!(GDPR, "https://aka.ms/MinecraftGDPR");
        assert_eq!(INTENTIONAL_GAME_DESIGN_BUG_ID, "MCPE-28723");
    }
}
