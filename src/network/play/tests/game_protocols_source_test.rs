use super::*;

const GAME_PACKET_TYPES_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/GamePacketTypes.java");
const GAME_PROTOCOLS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/GameProtocols.java");
const SERVER_GAME_PACKET_LISTENER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerGamePacketListener.java");
const SERVER_PACKET_LISTENER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/network/protocol/game/ServerPacketListener.java");

#[test]
fn game_packet_types_and_protocols_match_java_play_registration() {
    assert_java_contains(
        GAME_PACKET_TYPES_JAVA,
        &[
            "private static <T extends Packet<ClientGamePacketListener>> PacketType<T> createClientbound(final String id)",
            "return new PacketType<>(PacketFlow.CLIENTBOUND, Identifier.withDefaultNamespace(id));",
            "private static <T extends Packet<ServerGamePacketListener>> PacketType<T> createServerbound(final String id)",
            "return new PacketType<>(PacketFlow.SERVERBOUND, Identifier.withDefaultNamespace(id));",
            "CLIENTBOUND_DEBUG_BLOCK_VALUE = createClientbound(\"debug/block_value\")",
            "CLIENTBOUND_DEBUG_CHUNK_VALUE = createClientbound(\"debug/chunk_value\")",
            "CLIENTBOUND_DEBUG_ENTITY_VALUE = createClientbound(\"debug/entity_value\")",
            "CLIENTBOUND_DEBUG_EVENT = createClientbound(\"debug/event\")",
        ],
        "GamePacketTypes",
    );
    assert_java_contains(
        GAME_PROTOCOLS_JAVA,
        &[
            "ProtocolInfoBuilder.contextServerboundProtocol(",
            "ConnectionProtocol.PLAY",
            "ServerboundSetCreativeModeSlotPacket.STREAM_CODEC, HAS_INFINITE_MATERIALS",
            "ProtocolInfoBuilder.clientboundProtocol(",
            "builder -> builder.withBundlePacket(GamePacketTypes.CLIENTBOUND_BUNDLE, ClientboundBundlePacket::new, new ClientboundBundleDelimiterPacket())",
            ".addPacket(GamePacketTypes.CLIENTBOUND_DEBUG_BLOCK_VALUE, ClientboundDebugBlockValuePacket.STREAM_CODEC)",
            ".addPacket(GamePacketTypes.CLIENTBOUND_DEBUG_CHUNK_VALUE, ClientboundDebugChunkValuePacket.STREAM_CODEC)",
            ".addPacket(GamePacketTypes.CLIENTBOUND_DEBUG_ENTITY_VALUE, ClientboundDebugEntityValuePacket.STREAM_CODEC)",
            ".addPacket(GamePacketTypes.CLIENTBOUND_DEBUG_EVENT, ClientboundDebugEventPacket.STREAM_CODEC)",
            "public interface Context",
            "boolean hasInfiniteMaterials();",
        ],
        "GameProtocols",
    );

    let registry = PlayProtocolRegistry::new();
    assert_eq!(registry.serverbound().len(), SERVERBOUND_PLAY_PACKET_COUNT_26_1_2);
    assert_eq!(registry.clientbound().len(), CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2);
    assert_eq!(count_add_packets(serverbound_protocol_source()), registry.serverbound().len());
    assert_eq!(
        count_add_packets(clientbound_protocol_source()) + count_bundle_packet(clientbound_protocol_source()),
        registry.clientbound().len()
    );

    for name in registry.clientbound() {
        assert_clientbound_packet_type_name_is_declared(name);
    }
    for name in registry.serverbound() {
        assert_serverbound_packet_type_name_is_declared(name);
    }
}

#[test]
fn server_game_packet_listeners_match_java_protocol_and_handlers() {
    assert_java_contains(
        SERVER_GAME_PACKET_LISTENER_JAVA,
        &[
            "public interface ServerGamePacketListener extends ServerCommonPacketListener, ServerPingPacketListener",
            "default ConnectionProtocol protocol()",
            "return ConnectionProtocol.PLAY;",
        ],
        "ServerGamePacketListener",
    );
    assert_eq!(server_game_handler_signatures().len(), 58);
    assert_eq!(
        SERVER_GAME_PACKET_LISTENER_JAVA.matches("void handle").count(),
        server_game_handler_signatures().len()
    );
    for signature in server_game_handler_signatures() {
        assert!(
            SERVER_GAME_PACKET_LISTENER_JAVA.contains(signature),
            "missing ServerGamePacketListener signature {signature}"
        );
    }

    assert_java_contains(
        SERVER_PACKET_LISTENER_JAVA,
        &[
            "public interface ServerPacketListener extends ServerboundPacketListener",
            "Logger LOGGER = LogUtils.getLogger();",
            "default void onPacketError(final Packet packet, final Exception e) throws ReportedException",
            "LOGGER.error(\"Failed to handle packet {}, suppressing error\", packet, e);",
        ],
        "ServerPacketListener",
    );
}

fn count_add_packets(source: &str) -> usize {
    source.matches(".addPacket(").count()
}

fn count_bundle_packet(source: &str) -> usize {
    source.matches("withBundlePacket(").count()
}

fn serverbound_protocol_source() -> &'static str {
    let start = GAME_PROTOCOLS_JAVA
        .find("SERVERBOUND_TEMPLATE")
        .expect("GameProtocols contains SERVERBOUND_TEMPLATE");
    let end = GAME_PROTOCOLS_JAVA
        .find("public static final SimpleUnboundProtocol")
        .expect("GameProtocols contains CLIENTBOUND_TEMPLATE");
    &GAME_PROTOCOLS_JAVA[start..end]
}

fn clientbound_protocol_source() -> &'static str {
    let start = GAME_PROTOCOLS_JAVA
        .find("CLIENTBOUND_TEMPLATE")
        .expect("GameProtocols contains CLIENTBOUND_TEMPLATE");
    let end = GAME_PROTOCOLS_JAVA
        .find("public interface Context")
        .expect("GameProtocols contains Context");
    &GAME_PROTOCOLS_JAVA[start..end]
}

fn assert_clientbound_packet_type_name_is_declared(name: &str) {
    if is_common_clientbound_play_packet(name) {
        assert!(
            clientbound_protocol_source().contains(common_clientbound_type_name(name)),
            "missing common clientbound protocol registration for {name}"
        );
    } else {
        assert!(
            GAME_PACKET_TYPES_JAVA.contains(&format!("\"{name}\"")),
            "missing GamePacketTypes clientbound packet type {name}"
        );
    }
}

fn assert_serverbound_packet_type_name_is_declared(name: &str) {
    if is_common_serverbound_play_packet(name) {
        assert!(
            serverbound_protocol_source().contains(common_serverbound_type_name(name)),
            "missing common serverbound protocol registration for {name}"
        );
    } else {
        assert!(
            GAME_PACKET_TYPES_JAVA.contains(&format!("\"{name}\"")),
            "missing GamePacketTypes serverbound packet type {name}"
        );
    }
}

fn is_common_clientbound_play_packet(name: &str) -> bool {
    matches!(
        name,
        "cookie_request"
            | "custom_payload"
            | "disconnect"
            | "keep_alive"
            | "ping"
            | "pong_response"
            | "resource_pack_pop"
            | "resource_pack_push"
            | "store_cookie"
            | "transfer"
            | "update_tags"
            | "custom_report_details"
            | "server_links"
            | "clear_dialog"
            | "show_dialog"
    )
}

fn is_common_serverbound_play_packet(name: &str) -> bool {
    matches!(
        name,
        "client_information"
            | "cookie_response"
            | "custom_payload"
            | "keep_alive"
            | "ping_request"
            | "pong"
            | "resource_pack"
            | "custom_click_action"
    )
}

fn common_clientbound_type_name(name: &str) -> &'static str {
    match name {
        "cookie_request" => "CookiePacketTypes.CLIENTBOUND_COOKIE_REQUEST",
        "custom_payload" => "CommonPacketTypes.CLIENTBOUND_CUSTOM_PAYLOAD",
        "disconnect" => "CommonPacketTypes.CLIENTBOUND_DISCONNECT",
        "keep_alive" => "CommonPacketTypes.CLIENTBOUND_KEEP_ALIVE",
        "ping" => "CommonPacketTypes.CLIENTBOUND_PING",
        "pong_response" => "PingPacketTypes.CLIENTBOUND_PONG_RESPONSE",
        "resource_pack_pop" => "CommonPacketTypes.CLIENTBOUND_RESOURCE_PACK_POP",
        "resource_pack_push" => "CommonPacketTypes.CLIENTBOUND_RESOURCE_PACK_PUSH",
        "store_cookie" => "CommonPacketTypes.CLIENTBOUND_STORE_COOKIE",
        "transfer" => "CommonPacketTypes.CLIENTBOUND_TRANSFER",
        "update_tags" => "CommonPacketTypes.CLIENTBOUND_UPDATE_TAGS",
        "custom_report_details" => "CommonPacketTypes.CLIENTBOUND_CUSTOM_REPORT_DETAILS",
        "server_links" => "CommonPacketTypes.CLIENTBOUND_SERVER_LINKS",
        "clear_dialog" => "CommonPacketTypes.CLIENTBOUND_CLEAR_DIALOG",
        "show_dialog" => "CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG",
        _ => unreachable!("not a common clientbound play packet"),
    }
}

fn common_serverbound_type_name(name: &str) -> &'static str {
    match name {
        "client_information" => "CommonPacketTypes.SERVERBOUND_CLIENT_INFORMATION",
        "cookie_response" => "CookiePacketTypes.SERVERBOUND_COOKIE_RESPONSE",
        "custom_payload" => "CommonPacketTypes.SERVERBOUND_CUSTOM_PAYLOAD",
        "keep_alive" => "CommonPacketTypes.SERVERBOUND_KEEP_ALIVE",
        "ping_request" => "PingPacketTypes.SERVERBOUND_PING_REQUEST",
        "pong" => "CommonPacketTypes.SERVERBOUND_PONG",
        "resource_pack" => "CommonPacketTypes.SERVERBOUND_RESOURCE_PACK",
        "custom_click_action" => "CommonPacketTypes.SERVERBOUND_CUSTOM_CLICK_ACTION",
        _ => unreachable!("not a common serverbound play packet"),
    }
}

fn server_game_handler_signatures() -> &'static [&'static str] {
    &[
        "void handleAnimate(ServerboundSwingPacket packet);",
        "void handleChat(ServerboundChatPacket packet);",
        "void handleChatCommand(ServerboundChatCommandPacket packet);",
        "void handleSignedChatCommand(ServerboundChatCommandSignedPacket packet);",
        "void handleChatAck(ServerboundChatAckPacket packet);",
        "void handleClientCommand(ServerboundClientCommandPacket packet);",
        "void handleContainerButtonClick(ServerboundContainerButtonClickPacket packet);",
        "void handleContainerClick(ServerboundContainerClickPacket packet);",
        "void handlePlaceRecipe(final ServerboundPlaceRecipePacket packet);",
        "void handleContainerClose(ServerboundContainerClosePacket packet);",
        "void handleAttack(ServerboundAttackPacket packet);",
        "void handleInteract(ServerboundInteractPacket packet);",
        "void handleSpectateEntity(ServerboundSpectateEntityPacket packet);",
        "void handleMovePlayer(ServerboundMovePlayerPacket packet);",
        "void handlePlayerAbilities(ServerboundPlayerAbilitiesPacket packet);",
        "void handlePlayerAction(ServerboundPlayerActionPacket packet);",
        "void handlePlayerCommand(ServerboundPlayerCommandPacket packet);",
        "void handlePlayerInput(ServerboundPlayerInputPacket packet);",
        "void handleSetCarriedItem(ServerboundSetCarriedItemPacket packet);",
        "void handleSetCreativeModeSlot(ServerboundSetCreativeModeSlotPacket packet);",
        "void handleSignUpdate(ServerboundSignUpdatePacket packet);",
        "void handleUseItemOn(ServerboundUseItemOnPacket packet);",
        "void handleUseItem(ServerboundUseItemPacket packet);",
        "void handleTeleportToEntityPacket(ServerboundTeleportToEntityPacket packet);",
        "void handlePaddleBoat(ServerboundPaddleBoatPacket packet);",
        "void handleMoveVehicle(ServerboundMoveVehiclePacket packet);",
        "void handleAcceptTeleportPacket(ServerboundAcceptTeleportationPacket packet);",
        "void handleAcceptPlayerLoad(ServerboundPlayerLoadedPacket packet);",
        "void handleRecipeBookSeenRecipePacket(ServerboundRecipeBookSeenRecipePacket packet);",
        "void handleBundleItemSelectedPacket(ServerboundSelectBundleItemPacket packet);",
        "void handleRecipeBookChangeSettingsPacket(ServerboundRecipeBookChangeSettingsPacket packet);",
        "void handleSeenAdvancements(ServerboundSeenAdvancementsPacket packet);",
        "void handleCustomCommandSuggestions(ServerboundCommandSuggestionPacket packet);",
        "void handleSetCommandBlock(ServerboundSetCommandBlockPacket packet);",
        "void handleSetCommandMinecart(ServerboundSetCommandMinecartPacket packet);",
        "void handlePickItemFromBlock(ServerboundPickItemFromBlockPacket packet);",
        "void handlePickItemFromEntity(ServerboundPickItemFromEntityPacket packet);",
        "void handleRenameItem(ServerboundRenameItemPacket packet);",
        "void handleSetBeaconPacket(ServerboundSetBeaconPacket packet);",
        "void handleSetGameRule(ServerboundSetGameRulePacket packet);",
        "void handleSetStructureBlock(ServerboundSetStructureBlockPacket packet);",
        "void handleSetTestBlock(ServerboundSetTestBlockPacket packet);",
        "void handleTestInstanceBlockAction(ServerboundTestInstanceBlockActionPacket packet);",
        "void handleSelectTrade(ServerboundSelectTradePacket packet);",
        "void handleEditBook(ServerboundEditBookPacket packet);",
        "void handleEntityTagQuery(ServerboundEntityTagQueryPacket packet);",
        "void handleContainerSlotStateChanged(ServerboundContainerSlotStateChangedPacket packet);",
        "void handleBlockEntityTagQuery(ServerboundBlockEntityTagQueryPacket packet);",
        "void handleSetJigsawBlock(ServerboundSetJigsawBlockPacket packet);",
        "void handleJigsawGenerate(ServerboundJigsawGeneratePacket packet);",
        "void handleChangeDifficulty(ServerboundChangeDifficultyPacket packet);",
        "void handleChangeGameMode(ServerboundChangeGameModePacket packet);",
        "void handleLockDifficulty(ServerboundLockDifficultyPacket packet);",
        "void handleChatSessionUpdate(ServerboundChatSessionUpdatePacket packet);",
        "void handleConfigurationAcknowledged(ServerboundConfigurationAcknowledgedPacket packet);",
        "void handleChunkBatchReceived(ServerboundChunkBatchReceivedPacket packet);",
        "void handleDebugSubscriptionRequest(ServerboundDebugSubscriptionRequestPacket packet);",
        "void handleClientTickEnd(ServerboundClientTickEndPacket packet);",
    ]
}

fn assert_java_contains(source: &str, sentinels: &[&str], class_name: &str) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "missing {class_name} sentinel {sentinel}"
        );
    }
}
