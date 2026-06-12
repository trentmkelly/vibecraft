use super::*;
use crate::network::codec::cursor;
use std::collections::BTreeSet;

const SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ServerboundDebugSubscriptionRequestPacket.java"
);
const DEBUG_SUBSCRIPTIONS_JAVA: &str =
    include_str!("../../../../../decompiled-server-26.1.2/net/minecraft/util/debug/DebugSubscriptions.java");

#[test]
fn serverbound_debug_subscription_request_packet_matches_java_codec() {
    for sentinel in [
        "ByteBufCodecs.registry(Registries.DEBUG_SUBSCRIPTION)",
        "ByteBufCodecs.collection(ReferenceOpenHashSet::new)",
        "SET_STREAM_CODEC.map(",
        "ServerboundDebugSubscriptionRequestPacket::new",
        "ServerboundDebugSubscriptionRequestPacket::subscriptions",
        "return GamePacketTypes.SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST;",
        "listener.handleDebugSubscriptionRequest(this);",
    ] {
        assert!(
            SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_JAVA.contains(sentinel),
            "missing ServerboundDebugSubscriptionRequestPacket sentinel {sentinel}"
        );
    }
    for sentinel in [
        "registerSimple(\"dedicated_server_tick_time\")",
        "registerWithValue(\"bees\", DebugBeeInfo.STREAM_CODEC)",
        "registerWithValue(\"brains\", DebugBrainDump.STREAM_CODEC)",
        "registerWithValue(\"breezes\", DebugBreezeInfo.STREAM_CODEC)",
        "registerWithValue(\"goal_selectors\", DebugGoalInfo.STREAM_CODEC)",
        "registerWithValue(\"entity_paths\", DebugPathInfo.STREAM_CODEC)",
        "\"entity_block_intersections\"",
        "registerWithValue(\"bee_hives\", DebugHiveInfo.STREAM_CODEC)",
        "registerWithValue(\"pois\", DebugPoiInfo.STREAM_CODEC)",
        "\"redstone_wire_orientations\"",
        "registerWithValue(\"village_sections\", Unit.STREAM_CODEC)",
        "registerWithValue(\"raids\", BlockPos.STREAM_CODEC.apply(ByteBufCodecs.list()))",
        "\"structures\"",
        "\"game_event_listeners\"",
        "registerTemporaryValue(\"neighbor_updates\", BlockPos.STREAM_CODEC, 200)",
        "registerTemporaryValue(\"game_events\", DebugGameEventInfo.STREAM_CODEC, 60)",
    ] {
        assert!(
            DEBUG_SUBSCRIPTIONS_JAVA.contains(sentinel),
            "missing DebugSubscriptions sentinel {sentinel}"
        );
    }

    assert_eq!(SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID, 23);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID),
        Some("debug_subscription_request")
    );

    let packet = ServerboundDebugSubscriptionRequestPacket {
        subscriptions: BTreeSet::from([0, 4, 15]),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![3, 0, 4, 15]);
    assert_eq!(
        ServerboundDebugSubscriptionRequestPacket::read(&mut cursor(payload.clone())).unwrap(),
        packet
    );

    assert_eq!(
        ServerboundDebugSubscriptionRequestPacket::read(&mut cursor(vec![3, 4, 4, 0]))
            .unwrap()
            .subscriptions,
        BTreeSet::from([0, 4]),
        "Java reads into ReferenceOpenHashSet, so duplicate subscriptions collapse"
    );

    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_debug_subscription_request, Some(packet));
}

#[test]
fn serverbound_debug_subscription_request_packet_rejects_malformed_payloads() {
    assert!(ServerboundDebugSubscriptionRequestPacket::read(&mut cursor(Vec::new())).is_err());
    assert!(ServerboundDebugSubscriptionRequestPacket::read(&mut cursor(vec![1])).is_err());
    assert!(ServerboundDebugSubscriptionRequestPacket::read(&mut cursor(vec![1, 16])).is_err());
    let mut too_many = vec![17];
    too_many.extend([0; 17]);
    assert!(ServerboundDebugSubscriptionRequestPacket::read(&mut cursor(too_many)).is_err());

    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID,
            vec![1, 16]
        )),
        DispatchOutcome::Disconnect(reason)
            if reason.contains("bad debug subscription request packet")
    ));

    let mut session = PlaySession::new(1, 0);
    assert!(matches!(
        session.handle_decoded(super::decoded(
            SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST_PACKET_ID,
            vec![1, 0, 0]
        )),
        DispatchOutcome::Disconnect(reason)
            if reason.contains("bad debug subscription request packet")
    ));
}
