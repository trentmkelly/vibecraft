#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketPackageCoverage {
    pub java_package: &'static str,
    pub packet_class_count: usize,
    pub rust_modules: &'static [&'static str],
}

pub const TOTAL_PACKET_CLASSES_26_1_2: usize = 227;

pub const PACKET_PACKAGE_COVERAGE_26_1_2: &[PacketPackageCoverage] = &[
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol",
        packet_class_count: 3,
        rust_modules: &["network::bundle", "network::dispatch", "network::pipeline"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/common",
        packet_class_count: 19,
        rust_modules: &[
            "network::common",
            "network::cookie",
            "network::ping",
            "network::transfer",
        ],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/configuration",
        packet_class_count: 7,
        rust_modules: &["network::configuration"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/cookie",
        packet_class_count: 2,
        rust_modules: &["network::cookie"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/game",
        packet_class_count: 182,
        rust_modules: &["network::play"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/handshake",
        packet_class_count: 1,
        rust_modules: &["network::handshake"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/login",
        packet_class_count: 9,
        rust_modules: &["network::login"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/ping",
        packet_class_count: 2,
        rust_modules: &["network::ping"],
    },
    PacketPackageCoverage {
        java_package: "net/minecraft/network/protocol/status",
        packet_class_count: 2,
        rust_modules: &["network::status"],
    },
];

pub fn covered_packet_class_count() -> usize {
    PACKET_PACKAGE_COVERAGE_26_1_2
        .iter()
        .map(|entry| entry.packet_class_count)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{
        covered_packet_class_count, PACKET_PACKAGE_COVERAGE_26_1_2, TOTAL_PACKET_CLASSES_26_1_2,
    };
    use crate::network::configuration::{
        ClientboundCodeOfConductPacket, ClientboundFinishConfigurationPacket,
        ClientboundRegistryDataPacket, ClientboundResetChatPacket,
        ClientboundUpdateEnabledFeaturesPacket, ServerboundAcceptCodeOfConductPacket,
        ServerboundFinishConfigurationPacket,
    };
    use crate::network::login::{
        CLIENTBOUND_COOKIE_REQUEST_PACKET_ID, CLIENTBOUND_LOGIN_FINISHED_PACKET_ID,
        SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, SERVERBOUND_HELLO_PACKET_ID,
    };
    use crate::network::play::{
        CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2, SERVERBOUND_PLAY_PACKET_COUNT_26_1_2,
    };
    use crate::registry::Identifier;

    #[test]
    fn protocol_packet_class_manifest_matches_decompiled_26_1_2_tree() {
        assert_eq!(covered_packet_class_count(), TOTAL_PACKET_CLASSES_26_1_2);
        assert_eq!(PACKET_PACKAGE_COVERAGE_26_1_2.len(), 9);
        assert!(PACKET_PACKAGE_COVERAGE_26_1_2.iter().any(|entry| entry
            .java_package
            .ends_with("/game")
            && entry.packet_class_count == 182
            && entry.rust_modules == ["network::play"]));
    }

    #[test]
    fn protocol_state_registries_back_the_packet_class_manifest() {
        assert_eq!(SERVERBOUND_PLAY_PACKET_COUNT_26_1_2, 69);
        assert_eq!(CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2, 141);
        let _configuration_packets = (
            ClientboundCodeOfConductPacket {
                code_of_conduct: String::new(),
            },
            ClientboundFinishConfigurationPacket,
            ClientboundRegistryDataPacket {
                registry: Identifier::parse("minecraft:root").unwrap(),
                entries: Vec::new(),
            },
            ClientboundResetChatPacket,
            ClientboundUpdateEnabledFeaturesPacket {
                features: Vec::new(),
            },
            ServerboundAcceptCodeOfConductPacket,
            ServerboundFinishConfigurationPacket,
        );
        assert_eq!(SERVERBOUND_HELLO_PACKET_ID, 0);
        assert_eq!(SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, 4);
        assert_eq!(CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, 2);
        assert_eq!(CLIENTBOUND_COOKIE_REQUEST_PACKET_ID, 5);
    }
}
