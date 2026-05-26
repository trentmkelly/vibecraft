use super::*;

#[test]
fn delete_chat_packet_writes_java_packed_message_signature() {
    assert_eq!(CLIENTBOUND_DELETE_CHAT_PACKET_ID, 31);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_DELETE_CHAT_PACKET_ID),
        Some("delete_chat")
    );

    let mut cached = Vec::new();
    ClientboundDeleteChatPacket {
        message_signature: PackedMessageSignature::CacheId(127),
    }
    .write(&mut cached)
    .unwrap();
    assert_eq!(cached, vec![0x80, 0x01]);

    let mut full = Vec::new();
    ClientboundDeleteChatPacket {
        message_signature: PackedMessageSignature::Full(Box::new(MessageSignature([9; 256]))),
    }
    .write(&mut full)
    .unwrap();
    assert_eq!(full.len(), 257);
    assert_eq!(full[0], 0);
    assert_eq!(&full[1..], &[9; 256]);
}
