use super::*;

const CLIENTBOUND_SET_OBJECTIVE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetObjectivePacket.java");
const CLIENTBOUND_SET_PASSENGERS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetPassengersPacket.java");
const CLIENTBOUND_SET_PLAYER_INVENTORY_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetPlayerInventoryPacket.java");
const CLIENTBOUND_SET_PLAYER_TEAM_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetPlayerTeamPacket.java");
const CLIENTBOUND_SET_SCORE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetScorePacket.java");
const CLIENTBOUND_SET_SIMULATION_DISTANCE_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetSimulationDistancePacket.java");
const CLIENTBOUND_SET_SUBTITLE_TEXT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetSubtitleTextPacket.java");
const CLIENTBOUND_SET_TIME_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetTimePacket.java");
const CLIENTBOUND_SET_TITLE_TEXT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetTitleTextPacket.java");
const CLIENTBOUND_SET_TITLES_ANIMATION_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSetTitlesAnimationPacket.java");
const CLIENTBOUND_STOP_SOUND_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundStopSoundPacket.java");
const CLIENTBOUND_SYSTEM_CHAT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundSystemChatPacket.java");
const CLIENTBOUND_TAB_LIST_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundTabListPacket.java");
const CLIENTBOUND_TAKE_ITEM_ENTITY_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundTakeItemEntityPacket.java");
const CLIENTBOUND_TELEPORT_ENTITY_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundTeleportEntityPacket.java");
const CLIENTBOUND_UPDATE_ATTRIBUTES_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundUpdateAttributesPacket.java");
const CLIENTBOUND_UPDATE_MOB_EFFECT_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundUpdateMobEffectPacket.java");

#[test]
fn scoreboard_display_clientbound_packet_codecs_match_java_sources() {
    assert_set_objective();
    assert_set_passengers();
    assert_set_player_inventory();
    assert_set_player_team();
    assert_set_score();
    assert_set_simulation_distance();
    assert_set_text_packets();
    assert_set_time();
    assert_set_titles_animation();
    assert_stop_sound();
    assert_system_chat();
    assert_tab_list();
    assert_take_item_entity();
    assert_teleport_entity();
    assert_update_attributes();
    assert_update_mob_effect();
}

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

fn text_component_prefix(text: &str) -> Vec<u8> {
    [
        vec![10, 8, 0, 4],
        b"text".to_vec(),
        vec![0, text.len() as u8],
        text.as_bytes().to_vec(),
    ]
    .concat()
}

fn assert_set_objective() {
    assert_java_contains(
        CLIENTBOUND_SET_OBJECTIVE_JAVA,
        &[
            "this.objectiveName = input.readUtf();",
            "this.method = input.readByte();",
            "ComponentSerialization.TRUSTED_STREAM_CODEC.decode(input);",
            "output.writeEnum(this.renderType);",
            "listener.handleAddObjective(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_OBJECTIVE_PACKET_ID, "set_objective");

    let mut payload = Vec::new();
    ClientboundSetObjectivePacket {
        objective_name: "kills".to_string(),
        method: ObjectiveMethod::Remove,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, [vec![5], b"kills".to_vec(), vec![1]].concat());
}

fn assert_set_passengers() {
    assert_java_contains(
        CLIENTBOUND_SET_PASSENGERS_JAVA,
        &[
            "this.vehicle = input.readVarInt();",
            "this.passengers = input.readVarIntArray();",
            "output.writeVarIntArray(this.passengers);",
            "listener.handleSetEntityPassengersPacket(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_PASSENGERS_PACKET_ID, "set_passengers");

    let mut payload = Vec::new();
    ClientboundSetPassengersPacket {
        vehicle: 300,
        passengers: vec![0, 301],
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 2, 0, 0xad, 0x02]);
}

fn assert_set_player_inventory() {
    assert_java_contains(
        CLIENTBOUND_SET_PLAYER_INVENTORY_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "ItemStack.OPTIONAL_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_SET_PLAYER_INVENTORY;",
            "listener.handleSetPlayerInventory(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
        "set_player_inventory",
    );

    let mut payload = Vec::new();
    ClientboundSetPlayerInventoryPacket {
        slot: 40,
        contents: RawItemStack::empty(),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![40, 0]);
}

fn assert_set_player_team() {
    assert_java_contains(
        CLIENTBOUND_SET_PLAYER_TEAM_JAVA,
        &[
            "this.name = input.readUtf();",
            "this.method = input.readByte();",
            "this.parameters = Optional.of(new ClientboundSetPlayerTeamPacket.Parameters(input));",
            "output.writeCollection(this.players, FriendlyByteBuf::writeUtf);",
            "listener.handleSetPlayerTeamPacket(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID, "set_player_team");

    let mut payload = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::Remove,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, [vec![3], b"red".to_vec(), vec![1]].concat());
}

fn assert_set_score() {
    assert_java_contains(
        CLIENTBOUND_SET_SCORE_JAVA,
        &[
            "ByteBufCodecs.STRING_UTF8",
            "ByteBufCodecs.VAR_INT",
            "ComponentSerialization.TRUSTED_OPTIONAL_STREAM_CODEC",
            "NumberFormatTypes.OPTIONAL_STREAM_CODEC",
            "listener.handleSetScore(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_SCORE_PACKET_ID, "set_score");

    let mut payload = Vec::new();
    ClientboundSetScorePacket {
        owner: "Steve".to_string(),
        objective_name: "deaths".to_string(),
        score: 0,
        display: None,
        number_format: None,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        [
            vec![5],
            b"Steve".to_vec(),
            vec![6],
            b"deaths".to_vec(),
            vec![0, 0, 0]
        ]
        .concat()
    );
}

fn assert_set_simulation_distance() {
    assert_java_contains(
        CLIENTBOUND_SET_SIMULATION_DISTANCE_JAVA,
        &[
            "this(input.readVarInt());",
            "output.writeVarInt(this.simulationDistance);",
            "return GamePacketTypes.CLIENTBOUND_SET_SIMULATION_DISTANCE;",
            "listener.handleSetSimulationDistance(this);",
        ],
    );
    assert_registry(111, "set_simulation_distance");

    let packet = ClientboundSetSimulationDistancePacket {
        simulation_distance: 300,
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(payload, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundSetSimulationDistancePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_set_text_packets() {
    assert_java_contains(
        CLIENTBOUND_SET_TITLE_TEXT_JAVA,
        &[
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_SET_TITLE_TEXT;",
            "listener.setTitleText(this);",
        ],
    );
    assert_java_contains(
        CLIENTBOUND_SET_SUBTITLE_TEXT_JAVA,
        &[
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_SET_SUBTITLE_TEXT;",
            "listener.setSubtitleText(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID, "set_title_text");
    assert_registry(CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID, "set_subtitle_text");

    let mut title = Vec::new();
    ClientboundSetTitleTextPacket {
        text: text_component("Title"),
    }
    .write(&mut title)
    .unwrap();
    assert!(title.starts_with(&text_component_prefix("Title")));

    let mut subtitle = Vec::new();
    ClientboundSetSubtitleTextPacket {
        text: text_component("Subtitle"),
    }
    .write(&mut subtitle)
    .unwrap();
    assert!(subtitle.starts_with(&text_component_prefix("Subtitle")));
}

fn assert_set_time() {
    assert_java_contains(
        CLIENTBOUND_SET_TIME_JAVA,
        &[
            "ByteBufCodecs.LONG",
            "ByteBufCodecs.map(HashMap::new, WorldClock.STREAM_CODEC, ClockNetworkState.STREAM_CODEC)",
            "return GamePacketTypes.CLIENTBOUND_SET_TIME;",
            "listener.handleSetTime(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SET_TIME_PACKET_ID, "set_time");

    let packet = ClientboundSetTimePacket {
        game_time: 900_000,
        clock_updates: BTreeMap::new(),
    };
    let mut payload = Vec::new();
    packet.write(&mut payload).unwrap();
    assert_eq!(
        payload,
        [900_000_i64.to_be_bytes().to_vec(), vec![0]].concat()
    );
    assert_eq!(
        ClientboundSetTimePacket::read(&mut cursor(payload)).unwrap(),
        packet
    );
}

fn assert_set_titles_animation() {
    assert_java_contains(
        CLIENTBOUND_SET_TITLES_ANIMATION_JAVA,
        &[
            "this.fadeIn = input.readInt();",
            "this.stay = input.readInt();",
            "this.fadeOut = input.readInt();",
            "listener.setTitlesAnimation(this);",
        ],
    );
    assert_registry(
        CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID,
        "set_titles_animation",
    );

    let mut payload = Vec::new();
    ClientboundSetTitlesAnimationPacket {
        fade_in: 10,
        stay: 20,
        fade_out: -1,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        [
            10_i32.to_be_bytes(),
            20_i32.to_be_bytes(),
            (-1_i32).to_be_bytes()
        ]
        .concat()
    );
}

fn assert_stop_sound() {
    assert_java_contains(
        CLIENTBOUND_STOP_SOUND_JAVA,
        &[
            "int flags = input.readByte();",
            "this.source = input.readEnum(SoundSource.class);",
            "output.writeByte(3);",
            "listener.handleStopSoundEvent(this);",
        ],
    );
    assert_registry(CLIENTBOUND_STOP_SOUND_PACKET_ID, "stop_sound");

    let mut payload = Vec::new();
    ClientboundStopSoundPacket {
        source: Some(SoundSource::Blocks),
        name: Some(Identifier::parse("minecraft:block.note_block.harp").unwrap()),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..2], &[3, 4]);
    assert_eq!(
        payload[2],
        "minecraft:block.note_block.harp".len() as u8
    );
}

fn assert_system_chat() {
    assert_java_contains(
        CLIENTBOUND_SYSTEM_CHAT_JAVA,
        &[
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "ByteBufCodecs.BOOL",
            "return GamePacketTypes.CLIENTBOUND_SYSTEM_CHAT;",
            "listener.handleSystemChat(this);",
        ],
    );
    assert_registry(CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, "system_chat");

    let mut payload = Vec::new();
    ClientboundSystemChatPacket {
        content: text_component("Hello"),
        overlay: true,
    }
    .write(&mut payload)
    .unwrap();
    assert!(payload.starts_with(&text_component_prefix("Hello")));
    assert_eq!(payload.last(), Some(&1));
}

fn assert_tab_list() {
    assert_java_contains(
        CLIENTBOUND_TAB_LIST_JAVA,
        &[
            "ComponentSerialization.TRUSTED_STREAM_CODEC",
            "return GamePacketTypes.CLIENTBOUND_TAB_LIST;",
            "listener.handleTabListCustomisation(this);",
        ],
    );
    assert_registry(CLIENTBOUND_TAB_LIST_PACKET_ID, "tab_list");

    let mut payload = Vec::new();
    ClientboundTabListPacket {
        header: text_component("Header"),
        footer: text_component("Footer"),
    }
    .write(&mut payload)
    .unwrap();
    assert!(payload.starts_with(&text_component_prefix("Header")));
    assert!(payload.ends_with(&[0]));
}

fn assert_take_item_entity() {
    assert_java_contains(
        CLIENTBOUND_TAKE_ITEM_ENTITY_JAVA,
        &[
            "this.itemId = input.readVarInt();",
            "this.playerId = input.readVarInt();",
            "this.amount = input.readVarInt();",
            "listener.handleTakeItemEntity(this);",
        ],
    );
    assert_registry(CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID, "take_item_entity");

    let mut payload = Vec::new();
    ClientboundTakeItemEntityPacket {
        item_entity_id: 300,
        collector_entity_id: 301,
        amount: 5,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0xac, 0x02, 0xad, 0x02, 5]);
}

fn assert_teleport_entity() {
    assert_java_contains(
        CLIENTBOUND_TELEPORT_ENTITY_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "PositionMoveRotation.STREAM_CODEC",
            "Relative.SET_STREAM_CODEC",
            "ByteBufCodecs.BOOL",
            "listener.handleTeleportEntity(this);",
        ],
    );
    assert_registry(CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID, "teleport_entity");

    let mut payload = Vec::new();
    ClientboundTeleportEntityPacket {
        id: 300,
        position: Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        movement: Vec3 {
            x: 0.1,
            y: -0.2,
            z: 0.3,
        },
        y_rot: 90.0,
        x_rot: -30.5,
        relative_flags: 0x1ff,
        on_ground: false,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..2], &[0xac, 0x02]);
    assert_eq!(payload.len(), 63);
    assert_eq!(payload.last(), Some(&0));
}

fn assert_update_attributes() {
    assert_java_contains(
        CLIENTBOUND_UPDATE_ATTRIBUTES_JAVA,
        &[
            "ByteBufCodecs.VAR_INT",
            "Attribute.STREAM_CODEC",
            "AttributeModifier.Operation.STREAM_CODEC",
            "listener.handleUpdateAttributes(this);",
        ],
    );
    assert_registry(CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID, "update_attributes");

    let mut payload = Vec::new();
    ClientboundUpdateAttributesPacket {
        entity_id: 300,
        attributes: vec![AttributeSnapshot {
            attribute_id: 4,
            base: 20.0,
            modifiers: Vec::new(),
        }],
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..4], &[0xac, 0x02, 1, 4]);
    assert_eq!(&payload[4..12], &20.0_f64.to_be_bytes());
    assert_eq!(payload[12], 0);
}

fn assert_update_mob_effect() {
    assert_java_contains(
        CLIENTBOUND_UPDATE_MOB_EFFECT_JAVA,
        &[
            "this.entityId = input.readVarInt();",
            "this.effect = MobEffect.STREAM_CODEC.decode(input);",
            "output.writeVarInt(this.effectDurationTicks);",
            "listener.handleUpdateMobEffect(this);",
        ],
    );
    assert_registry(CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID, "update_mob_effect");

    let mut payload = Vec::new();
    ClientboundUpdateMobEffectPacket {
        entity_id: 129,
        effect_id: 5,
        amplifier: 2,
        duration_ticks: 600,
        flags: MobEffectFlags::from_parts(true, false, true, true),
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(payload, vec![0x81, 0x01, 5, 2, 0xd8, 0x04, 0x0d]);
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
