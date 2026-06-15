#![allow(dead_code)]

use crate::network::codec::ComponentJson;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownLinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerLinkType {
    Known(KnownLinkType),
    Custom(ComponentJson),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLinks {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    link_type: ServerLinkType,
    link: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntrustedEntry {
    pub link_type: ServerLinkType,
    pub link: String,
}

impl ServerLinks {
    pub const EMPTY: Self = Self {
        entries: Vec::new(),
    };

    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn find_known_type(&self, link_type: KnownLinkType) -> Option<&Entry> {
        self.entries.iter().find(|entry| {
            matches!(
                &entry.link_type,
                ServerLinkType::Known(known_link_type) if *known_link_type == link_type
            )
        })
    }

    pub fn untrust(&self) -> Vec<UntrustedEntry> {
        self.entries
            .iter()
            .map(|entry| UntrustedEntry {
                link_type: entry.link_type.clone(),
                link: entry.link.clone(),
            })
            .collect()
    }
}

impl Entry {
    pub fn known_type(link_type: KnownLinkType, link: impl Into<String>) -> Self {
        Self {
            link_type: ServerLinkType::Known(link_type),
            link: link.into(),
        }
    }

    pub fn custom(display_name: ComponentJson, link: impl Into<String>) -> Self {
        Self {
            link_type: ServerLinkType::Custom(display_name),
            link: link.into(),
        }
    }

    pub fn link_type(&self) -> &ServerLinkType {
        &self.link_type
    }

    pub fn link(&self) -> &str {
        &self.link
    }

    pub fn display_name(&self) -> ComponentJson {
        match &self.link_type {
            ServerLinkType::Known(link_type) => link_type.display_name(),
            ServerLinkType::Custom(display_name) => display_name.clone(),
        }
    }
}

impl KnownLinkType {
    pub const VALUES: [Self; 10] = [
        Self::BugReport,
        Self::CommunityGuidelines,
        Self::Support,
        Self::Status,
        Self::Feedback,
        Self::Community,
        Self::Website,
        Self::Forums,
        Self::News,
        Self::Announcements,
    ];

    pub fn id(self) -> usize {
        match self {
            Self::BugReport => 0,
            Self::CommunityGuidelines => 1,
            Self::Support => 2,
            Self::Status => 3,
            Self::Feedback => 4,
            Self::Community => 5,
            Self::Website => 6,
            Self::Forums => 7,
            Self::News => 8,
            Self::Announcements => 9,
        }
    }

    pub fn by_id(id: usize) -> Self {
        Self::VALUES.get(id).copied().unwrap_or(Self::BugReport)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::BugReport => "report_bug",
            Self::CommunityGuidelines => "community_guidelines",
            Self::Support => "support",
            Self::Status => "status",
            Self::Feedback => "feedback",
            Self::Community => "community",
            Self::Website => "website",
            Self::Forums => "forums",
            Self::News => "news",
            Self::Announcements => "announcements",
        }
    }

    pub fn display_name(self) -> ComponentJson {
        ComponentJson(format!(
            "{{\"translate\":\"known_server_link.{}\"}}",
            self.name()
        ))
    }

    pub fn create(self, link: impl Into<String>) -> Entry {
        Entry::known_type(self, link)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/ServerLinks.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_server_links_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public record ServerLinks(List<ServerLinks.Entry> entries)"));
        assert!(JAVA_SOURCE.contains("public static final ServerLinks EMPTY = new ServerLinks(List.of());"));
        assert!(JAVA_SOURCE.contains("public boolean isEmpty()"));
        assert!(JAVA_SOURCE.contains("public Optional<ServerLinks.Entry> findKnownType"));
        assert!(JAVA_SOURCE.contains("public List<ServerLinks.UntrustedEntry> untrust()"));
        assert!(JAVA_SOURCE.contains("public static ServerLinks.Entry knownType"));
        assert!(JAVA_SOURCE.contains("public static ServerLinks.Entry custom"));
        assert!(JAVA_SOURCE.contains("public Component displayName()"));
        assert!(JAVA_SOURCE.contains("BUG_REPORT(0, \"report_bug\")"));
        assert!(JAVA_SOURCE.contains("ANNOUNCEMENTS(9, \"announcements\")"));
        assert!(JAVA_SOURCE.contains("ByIdMap.OutOfBoundsStrategy.ZERO"));
        assert!(JAVA_SOURCE.contains("return Component.translatable(\"known_server_link.\" + this.name);"));
        assert!(JAVA_SOURCE.contains("public ServerLinks.Entry create(final URI link)"));
    }

    #[test]
    fn server_utility_server_links_known_type_order_and_fallback_match_java() {
        let ids = KnownLinkType::VALUES
            .iter()
            .map(|link_type| (link_type.id(), link_type.name()))
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                (0, "report_bug"),
                (1, "community_guidelines"),
                (2, "support"),
                (3, "status"),
                (4, "feedback"),
                (5, "community"),
                (6, "website"),
                (7, "forums"),
                (8, "news"),
                (9, "announcements"),
            ]
        );
        assert_eq!(KnownLinkType::by_id(99), KnownLinkType::BugReport);
    }

    #[test]
    fn server_utility_server_links_entry_factories_and_display_names_match_java() {
        let known = KnownLinkType::Support.create("https://example.invalid/support");
        assert_eq!(known.link(), "https://example.invalid/support");
        assert_eq!(known.link_type(), &ServerLinkType::Known(KnownLinkType::Support));
        assert_eq!(
            known.display_name(),
            ComponentJson("{\"translate\":\"known_server_link.support\"}".to_string())
        );

        let custom_name = ComponentJson("{\"text\":\"Rules\"}".to_string());
        let custom = Entry::custom(custom_name.clone(), "https://example.invalid/rules");
        assert_eq!(custom.link_type(), &ServerLinkType::Custom(custom_name.clone()));
        assert_eq!(custom.display_name(), custom_name);
    }

    #[test]
    fn server_utility_server_links_find_known_type_and_untrust_match_java() {
        assert!(ServerLinks::EMPTY.is_empty());

        let links = ServerLinks::new(vec![
            Entry::custom(
                ComponentJson("{\"text\":\"Docs\"}".to_string()),
                "https://example.invalid/docs",
            ),
            Entry::known_type(KnownLinkType::Website, "https://example.invalid"),
            Entry::known_type(KnownLinkType::Support, "https://example.invalid/support"),
        ]);

        assert!(!links.is_empty());
        assert_eq!(
            links.find_known_type(KnownLinkType::Website).map(Entry::link),
            Some("https://example.invalid")
        );
        assert_eq!(links.find_known_type(KnownLinkType::News), None);

        assert_eq!(
            links.untrust(),
            vec![
                UntrustedEntry {
                    link_type: ServerLinkType::Custom(ComponentJson(
                        "{\"text\":\"Docs\"}".to_string()
                    )),
                    link: "https://example.invalid/docs".to_string(),
                },
                UntrustedEntry {
                    link_type: ServerLinkType::Known(KnownLinkType::Website),
                    link: "https://example.invalid".to_string(),
                },
                UntrustedEntry {
                    link_type: ServerLinkType::Known(KnownLinkType::Support),
                    link: "https://example.invalid/support".to_string(),
                },
            ]
        );
    }
}
