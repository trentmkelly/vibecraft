use super::*;
use crate::network::common::{
    ClientboundCustomPayloadPacket, CustomPayload, MAX_CLIENTBOUND_CUSTOM_PAYLOAD_SIZE,
};
use std::io::{self, Cursor};

#[test]
fn clientbound_custom_payload_packet_matches_java_brand_codec() {
    assert_eq!(CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID, 24);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID),
        Some("custom_payload")
    );

    let packet = ClientboundCustomPayloadPacket {
        payload: CustomPayload::Brand("rustcraft".to_string()),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        payload,
        [
            vec![15],
            b"minecraft:brand".to_vec(),
            vec![9],
            b"rustcraft".to_vec(),
        ]
        .concat()
    );
    assert_eq!(
        ClientboundCustomPayloadPacket::read(&mut Cursor::new(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_custom_payload_packet_allows_unknown_payloads_up_to_java_limit() {
    let packet = ClientboundCustomPayloadPacket {
        payload: CustomPayload::Unknown {
            channel: Identifier::parse("rustcraft:debug").unwrap(),
            payload: vec![0x5a; MAX_CLIENTBOUND_CUSTOM_PAYLOAD_SIZE],
        },
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();

    assert_eq!(
        ClientboundCustomPayloadPacket::read(&mut Cursor::new(payload)).unwrap(),
        packet
    );
}

#[test]
fn clientbound_custom_payload_packet_rejects_unknown_payloads_past_java_limit() {
    let packet = ClientboundCustomPayloadPacket {
        payload: CustomPayload::Unknown {
            channel: Identifier::parse("rustcraft:debug").unwrap(),
            payload: vec![0; MAX_CLIENTBOUND_CUSTOM_PAYLOAD_SIZE + 1],
        },
    };

    let err = packet.write(&mut Vec::new()).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
}
