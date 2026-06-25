use super::*;

const CLIENTBOUND_PLAYER_CHAT_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundPlayerChatPacket.java");
const SIGNED_MESSAGE_BODY_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/chat/SignedMessageBody.java");
const LAST_SEEN_MESSAGES_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/chat/LastSeenMessages.java");
const MESSAGE_SIGNATURE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/chat/MessageSignature.java");
const FILTER_MASK_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/FilterMask.java");
const CHAT_TYPE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/chat/ChatType.java");
const BYTE_BUF_CODECS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/codec/ByteBufCodecs.java");
const FRIENDLY_BYTE_BUF_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/FriendlyByteBuf.java");

#[test]
fn clientbound_player_chat_packet_matches_java_codec_sources() {
    assert_java_contains(
        CLIENTBOUND_PLAYER_CHAT_JAVA,
        &[
            "input.readVarInt()",
            "input.readUUID()",
            "input.readNullable(MessageSignature::read)",
            "new SignedMessageBody.Packed(input)",
            "FriendlyByteBuf.readNullable(input, ComponentSerialization.TRUSTED_STREAM_CODEC)",
            "FilterMask.read(input)",
            "ChatType.Bound.STREAM_CODEC.decode(input)",
            "output.writeVarInt(this.globalIndex);",
            "output.writeUUID(this.sender);",
            "output.writeNullable(this.signature, MessageSignature::write);",
            "this.body.write(output);",
            "FriendlyByteBuf.writeNullable(output, this.unsignedContent, ComponentSerialization.TRUSTED_STREAM_CODEC);",
            "FilterMask.write(output, this.filterMask);",
            "ChatType.Bound.STREAM_CODEC.encode(output, this.chatType);",
            "return GamePacketTypes.CLIENTBOUND_PLAYER_CHAT;",
            "listener.handlePlayerChat(this);",
            "public boolean isSkippable()",
        ],
        "ClientboundPlayerChatPacket",
    );
    assert_java_contains(
        SIGNED_MESSAGE_BODY_JAVA,
        &[
            "this(input.readUtf(256), input.readInstant(), input.readLong(), new LastSeenMessages.Packed(input));",
            "output.writeUtf(this.content, 256);",
            "output.writeInstant(this.timeStamp);",
            "output.writeLong(this.salt);",
            "this.lastSeen.write(output);",
        ],
        "SignedMessageBody.Packed",
    );
    assert_java_contains(
        LAST_SEEN_MESSAGES_JAVA,
        &[
            "public static final int LAST_SEEN_MESSAGES_MAX_LENGTH = 20;",
            "input.readCollection(FriendlyByteBuf.limitValue(ArrayList::new, 20), MessageSignature.Packed::read)",
            "output.writeCollection(this.entries, MessageSignature.Packed::write);",
        ],
        "LastSeenMessages.Packed",
    );
    assert_java_contains(
        MESSAGE_SIGNATURE_JAVA,
        &[
            "public static final int BYTES = 256;",
            "input.readBytes(bytes);",
            "output.writeBytes(signature.bytes);",
            "int id = input.readVarInt() - 1;",
            "output.writeVarInt(packed.id() + 1);",
        ],
        "MessageSignature",
    );
    assert_java_contains(
        FILTER_MASK_JAVA,
        &[
            "FilterMask.Type type = input.readEnum(FilterMask.Type.class);",
            "case PASS_THROUGH -> PASS_THROUGH;",
            "case FULLY_FILTERED -> FULLY_FILTERED;",
            "case PARTIALLY_FILTERED -> new FilterMask(input.readBitSet(), FilterMask.Type.PARTIALLY_FILTERED);",
            "output.writeEnum(mask.type);",
            "output.writeBitSet(mask.mask);",
        ],
        "FilterMask",
    );
    assert_java_contains(
        CHAT_TYPE_JAVA,
        &[
            "public static final StreamCodec<RegistryFriendlyByteBuf, Holder<ChatType>> STREAM_CODEC = ByteBufCodecs.holder(Registries.CHAT_TYPE, DIRECT_STREAM_CODEC);",
            "ChatType.STREAM_CODEC",
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "ComponentSerialization.TRUSTED_OPTIONAL_STREAM_CODEC",
        ],
        "ChatType.Bound",
    );
    assert_java_contains(
        BYTE_BUF_CODECS_JAVA,
        &[
            "id == 0 ? Holder.direct(directCodec.decode(input))",
            "VarInt.write(output, id + 1);",
        ],
        "ByteBufCodecs.holder",
    );
    assert_java_contains(
        FRIENDLY_BYTE_BUF_JAVA,
        &[
            "return Instant.ofEpochMilli(this.readLong());",
            "this.writeLong(value.toEpochMilli());",
            "return clazz.getEnumConstants()[this.readVarInt()];",
            "return this.writeVarInt(value.ordinal());",
            "return BitSet.valueOf(this.readLongArray());",
            "this.writeLongArray(bitSet.toLongArray());",
        ],
        "FriendlyByteBuf helpers",
    );

    assert_eq!(CLIENTBOUND_PLAYER_CHAT_PACKET_ID, 65);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_PLAYER_CHAT_PACKET_ID),
        Some("player_chat")
    );

    let mut payload = Vec::new();
    player_chat_packet().write(&mut payload).unwrap();
    assert_eq!(payload, expected_player_chat_payload());
}

fn player_chat_packet() -> ClientboundPlayerChatPacket {
    ClientboundPlayerChatPacket {
        global_index: 300,
        sender: Uuid([1; 16]),
        index: 2,
        signature: Some(vec![8; MessageSignature::BYTES]),
        body: SignedMessageBodyPacked {
            content: "hello".to_string(),
            timestamp_epoch_millis: 1_700_000_000_123,
            salt: -42,
            last_seen: vec![
                MessageSignaturePackedData::Id(0),
                MessageSignaturePackedData::Full(vec![9; MessageSignature::BYTES]),
            ],
        },
        unsigned_content_payload: Some(text_component_bytes("Unsigned")),
        filter_mask: FilterMaskData::PartiallyFiltered(vec![0b101]),
        chat_type: BoundChatTypeData {
            chat_type_id: 0,
            name_payload: text_component_bytes("Steve"),
            target_name_payload: Some(text_component_bytes("Alex")),
        },
    }
}

fn expected_player_chat_payload() -> Vec<u8> {
    [
        vec![0xac, 0x02],
        vec![1; 16],
        vec![2, 1],
        vec![8; MessageSignature::BYTES],
        vec![5],
        b"hello".to_vec(),
        1_700_000_000_123_i64.to_be_bytes().to_vec(),
        (-42_i64).to_be_bytes().to_vec(),
        vec![2, 1, 0],
        vec![9; MessageSignature::BYTES],
        vec![1],
        text_component_bytes("Unsigned"),
        vec![2, 1],
        0b101_u64.to_be_bytes().to_vec(),
        vec![1],
        text_component_bytes("Steve"),
        vec![1],
        text_component_bytes("Alex"),
    ]
    .concat()
}

fn text_component_bytes(text: &str) -> Vec<u8> {
    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        (text.len() as u16).to_be_bytes().to_vec(),
        text.as_bytes().to_vec(),
        vec![0],
    ]
    .concat()
}

fn assert_java_contains(source: &str, sentinels: &[&str], class_name: &str) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing {class_name} sentinel {sentinel}"
        );
    }
}
