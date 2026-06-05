use super::super::*;
use super::*;

#[test]
fn serverbound_container_slot_state_changed_packet_matches_java_codec() {
    assert_eq!(SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED_PACKET_ID, 20);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED_PACKET_ID),
        Some("container_slot_state_changed")
    );

    // Java wire order: slotId (VarInt), containerId (ContainerId = VarInt), newState (bool).
    let packet = ServerboundContainerSlotStateChangedPacket {
        slot_id: 4,
        container_id: 128,
        new_state: true,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![4, 0x80, 0x01, 1]);
    assert_eq!(
        ServerboundContainerSlotStateChangedPacket::read(&mut std::io::Cursor::new(
            payload.clone()
        ))
        .unwrap(),
        packet
    );
}

#[test]
fn serverbound_container_slot_state_changed_packet_dispatches_into_session() {
    let mut session = PlaySession::new(1, 0);
    let packet = ServerboundContainerSlotStateChangedPacket {
        slot_id: 2,
        container_id: 7,
        new_state: false,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED_PACKET_ID,
            payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_container_slot_state_changed, Some(packet));
}
