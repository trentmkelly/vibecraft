use super::*;

const CLIENTBOUND_REMOVE_ENTITIES_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRemoveEntitiesPacket.java"
);
const CLIENTBOUND_REMOVE_MOB_EFFECT_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRemoveMobEffectPacket.java"
);
const CLIENTBOUND_RESET_SCORE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundResetScorePacket.java"
);
const CLIENTBOUND_ROTATE_HEAD_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundRotateHeadPacket.java"
);
const CLIENTBOUND_SECTION_BLOCKS_UPDATE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSectionBlocksUpdatePacket.java"
);
const CLIENTBOUND_SET_ACTION_BAR_TEXT_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetActionBarTextPacket.java"
);
const CLIENTBOUND_SET_CURSOR_ITEM_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetCursorItemPacket.java"
);
const CLIENTBOUND_SET_DISPLAY_OBJECTIVE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetDisplayObjectivePacket.java"
);
const CLIENTBOUND_SET_ENTITY_LINK_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEntityLinkPacket.java"
);
const CLIENTBOUND_SET_ENTITY_MOTION_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEntityMotionPacket.java"
);
const CLIENTBOUND_SET_EQUIPMENT_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetEquipmentPacket.java"
);
const CLIENTBOUND_SET_EXPERIENCE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetExperiencePacket.java"
);
const CLIENTBOUND_SET_HEALTH_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetHealthPacket.java"
);
const CLIENTBOUND_SET_HELD_SLOT_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundSetHeldSlotPacket.java"
);

#[test]
fn entity_inventory_clientbound_packet_codecs_match_java_sources() {
    assert_remove_entities();
    assert_remove_mob_effect();
    assert_reset_score();
    assert_rotate_head();
    assert_section_blocks_update();
    assert_set_action_bar_text();
    assert_set_cursor_item();
    assert_set_display_objective();
    assert_set_entity_link();
    assert_set_entity_motion();
    assert_set_equipment();
    assert_set_experience();
    assert_set_health();
    assert_set_held_slot();
}

fn assert_remove_entities() {
    assert_java_contains(
        CLIENTBOUND_REMOVE_ENTITIES_JAVA,
        &[
            "this.entityIds = input.readIntIdList();",
            "output.writeIntIdList(this.entityIds);",
            "return GamePacketTypes.CLIENTBOUND_REMOVE_ENTITIES;",
            "listener.handleRemoveEntities(this);",
        ],
    );
    assert_registry(CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, "remove_entities");

    let mut payload = Vec::new();
    ClientboundRemoveEntitiesPacket {
        entity_ids: vec![1, 300],
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![2, 1, 0xac, 0x02]);
}

fn assert_remove_mob_effect() {
    assert_java_contains(
        CLIENTBOUND_REMOVE_MOB_EFFECT_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "MobEffect.STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_REMOVE_MOB_EFFECT;",
            "listener.handleRemoveMobEffect(this);",
        ],
    );
    assert_registry(CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, "remove_mob_effect");

    let mut payload = Vec::new();
    ClientboundRemoveMobEffectPacket {
        entity_id: 300,
        effect_id: 7,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 7]);
}

fn assert_reset_score() {
    assert_java_contains(
        CLIENTBOUND_RESET_SCORE_JAVA,
        &[
            "this(input.readUtf(), input.readNullable(FriendlyByteBuf::readUtf));",
            "output.writeUtf(this.owner);",
            "output.writeNullable(this.objectiveName, FriendlyByteBuf::writeUtf);",
            "listener.handleResetScore(this);",
        ],
    );
    assert_registry(CLIENTBOUND_RESET_SCORE_PACKET_ID, "reset_score");

    let mut payload = Vec::new();
    ClientboundResetScorePacket {
        owner: "Alex".to_string(),
        objective_name: Some("kills".to_string()),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        [vec![4], b"Alex".to_vec(), vec![1, 5], b"kills".to_vec()].concat()
    );
}

fn assert_rotate_head() {
    assert_java_contains(
        CLIENTBOUND_ROTATE_HEAD_JAVA,
        &[
            "this.entityId = input.readVarInt();",
            "this.yHeadRot = input.readByte();",
            "output.writeByte(this.yHeadRot);",
            "listener.handleRotateMob(this);",
        ],
    );
    assert_registry(CLIENTBOUND_ROTATE_HEAD_PACKET_ID, "rotate_head");

    let packet = ClientboundRotateHeadPacket::new(300, 180.0);
    assert_eq!(packet.y_head_rot, 128);
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 0x80]);
}

fn assert_section_blocks_update() {
    assert_java_contains(
        CLIENTBOUND_SECTION_BLOCKS_UPDATE_JAVA,
        &[
            "this.sectionPos = SectionPos.STREAM_CODEC.decode(input);",
            "int count = input.readVarInt();",
            "long packedChange = input.readVarLong();",
            "output.writeVarLong((long)Block.getId(this.states[i]) << 12 | this.positions[i]);",
            "listener.handleChunkBlocksUpdate(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID,
        "section_blocks_update",
    );

    let mut payload = Vec::new();
    ClientboundSectionBlocksUpdatePacket {
        section_pos: SectionPos { x: 1, y: -2, z: 3 },
        updates: vec![SectionBlockUpdate {
            packed_pos: 0x0abc,
            block_state_id: 118,
        }],
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..8], &0x0000_0400_003f_fffe_i64.to_be_bytes());
    assert_eq!(&payload[8..], &[1, 0xbc, 0xd5, 0x1d]);
}

fn assert_set_action_bar_text() {
    assert_java_contains(
        CLIENTBOUND_SET_ACTION_BAR_TEXT_JAVA,
        &[
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_SET_ACTION_BAR_TEXT;",
            "listener.setActionBarText(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID, "set_action_bar_text");

    let mut payload = Vec::new();
    ClientboundSetActionBarTextPacket {
        text: Tag::Compound(vec![(
            "text".to_string(),
            Tag::String("Action".to_string()),
        )]),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..4], &[10, 8, 0, 4]);
    assert!(payload.ends_with(&[0]));
}

fn assert_set_cursor_item() {
    assert_java_contains(
        CLIENTBOUND_SET_CURSOR_ITEM_JAVA,
        &[
            "ItemStack.OPTIONAL_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_SET_CURSOR_ITEM;",
            "listener.handleSetCursorItem(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID, "set_cursor_item");

    let stack = RawItemStack {
        count: 3,
        item_id: Some(42),
        components: RawDataComponentPatch {
            added: vec![(7, vec![0xaa, 0xbb])],
            removed: vec![9],
        },
    };
    let mut payload = Vec::new();
    ClientboundSetCursorItemPacket { item_stack: stack }
        .write(&mut payload)
        .unwrap();
    assert_eq!(payload, vec![3, 42, 1, 1, 7, 0xaa, 0xbb, 9]);
}

fn assert_set_display_objective() {
    assert_java_contains(
        CLIENTBOUND_SET_DISPLAY_OBJECTIVE_JAVA,
        &[
            "this.slot = input.readById(DisplaySlot.BY_ID);",
            "this.objectiveName = input.readUtf();",
            "output.writeById(DisplaySlot::id, this.slot);",
            "listener.handleSetDisplayObjective(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID,
        "set_display_objective",
    );

    let mut payload = Vec::new();
    ClientboundSetDisplayObjectivePacket {
        slot: 18,
        objective_name: String::new(),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![18, 0]);
}

fn assert_set_entity_link() {
    assert_java_contains(
        CLIENTBOUND_SET_ENTITY_LINK_JAVA,
        &[
            "this.sourceId = input.readInt();",
            "this.destId = input.readInt();",
            "output.writeInt(this.sourceId);",
            "output.writeInt(this.destId);",
            "listener.handleEntityLinkPacket(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID, "set_entity_link");

    let mut payload = Vec::new();
    ClientboundSetEntityLinkPacket::new(300, None)
        .write(&mut payload)
        .unwrap();
    assert_eq!(
        payload,
        [300_i32.to_be_bytes(), 0_i32.to_be_bytes()].concat()
    );
}

fn assert_set_entity_motion() {
    assert_java_contains(
        CLIENTBOUND_SET_ENTITY_MOTION_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "Vec3.LP_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_SET_ENTITY_MOTION;",
            "listener.handleSetEntityMotion(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, "set_entity_motion");

    let mut payload = Vec::new();
    ClientboundSetEntityMotionPacket::new(
        300,
        Vec3 {
            x: 0.125,
            y: -0.25,
            z: 0.5,
        },
    )
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..2], &[0xac, 0x02]);
    assert_eq!(&payload[2..], &[0xf9, 0x3f, 0xbf, 0xfe, 0xbf, 0xfe]);
}

fn assert_set_equipment() {
    assert_java_contains(
        CLIENTBOUND_SET_EQUIPMENT_JAVA,
        &[
            "this.entity = input.readVarInt();",
            "EquipmentSlot slot = EquipmentSlot.VALUES.get(slotId & 127);",
            "int slotId = slotType.ordinal();",
            "ItemStack.OPTIONAL_STREAM_CODEC.encode(output, (ItemStack)e.getSecond());",
            "listener.handleSetEquipment(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_EQUIPMENT_PACKET_ID, "set_equipment");

    let packet = ClientboundSetEquipmentPacket {
        entity: 300,
        slots: vec![
            EquipmentEntry {
                slot: EquipmentSlotKind::MainHand,
                item_stack: RawItemStack::empty(),
            },
            EquipmentEntry {
                slot: EquipmentSlotKind::Saddle,
                item_stack: RawItemStack::empty(),
            },
        ],
    };
    assert_eq!(packet.encoded_slot_bytes(), vec![0x80, 0x07]);
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 0x80, 0, 0x07, 0]);
}

fn assert_set_experience() {
    assert_java_contains(
        CLIENTBOUND_SET_EXPERIENCE_JAVA,
        &[
            "this.experienceProgress = input.readFloat();",
            "this.experienceLevel = input.readVarInt();",
            "output.writeFloat(this.experienceProgress);",
            "listener.handleSetExperience(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_EXPERIENCE_PACKET_ID, "set_experience");

    let packet = ClientboundSetExperiencePacket {
        experience_progress: 0.5,
        experience_level: 30,
        total_experience: 300,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(&payload[..4], &0.5_f32.to_be_bytes());
    assert_eq!(&payload[4..], &[30, 0xac, 0x02]);
    assert_eq!(
        ClientboundSetExperiencePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_set_health() {
    assert_java_contains(
        CLIENTBOUND_SET_HEALTH_JAVA,
        &[
            "this.health = input.readFloat();",
            "this.food = input.readVarInt();",
            "output.writeFloat(this.saturation);",
            "listener.handleSetHealth(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_HEALTH_PACKET_ID, "set_health");

    let packet = ClientboundSetHealthPacket {
        health: 20.0,
        food: 18,
        saturation: 4.5,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(&payload[..4], &20.0_f32.to_be_bytes());
    assert_eq!(payload[4], 18);
    assert_eq!(&payload[5..], &4.5_f32.to_be_bytes());
    assert_eq!(
        ClientboundSetHealthPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_set_held_slot() {
    assert_java_contains(
        CLIENTBOUND_SET_HELD_SLOT_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "return GamePacketTypes.CLIENTBOUND_SET_HELD_SLOT;",
            "listener.handleSetHeldSlot(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_HELD_SLOT_PACKET_ID, "set_held_slot");

    let packet = ClientboundSetHeldSlotPacket { slot: 300 };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundSetHeldSlotPacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_registry(packet_id: i32, name: &str) {
    let registry = PlayProtocolRegistry::new();
    assert_eq!(registry.clientbound_name(packet_id), Some(name));
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }
}
