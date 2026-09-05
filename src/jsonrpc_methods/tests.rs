use super::*;

#[test]
fn jsonrpc_runtime_exception_models_match_java_message_constructors() {
    assert_eq!(
        JsonRpcRuntimeException::encode("bad encoding"),
        JsonRpcRuntimeException {
            kind: JsonRpcRuntimeExceptionKind::Encode,
            message: "bad encoding".to_string(),
        }
    );
    assert_eq!(
        JsonRpcRuntimeException::invalid_parameter("bad param").kind,
        JsonRpcRuntimeExceptionKind::InvalidParameter
    );
    assert_eq!(
        JsonRpcRuntimeException::invalid_request("bad request").kind,
        JsonRpcRuntimeExceptionKind::InvalidRequest
    );
    assert_eq!(
        JsonRpcRuntimeException::method_not_found("missing").kind,
        JsonRpcRuntimeExceptionKind::MethodNotFound
    );
}

#[test]
fn remote_rpc_error_exception_preserves_id_and_error_payloads() {
    let error = RemoteRpcErrorException::new("7", "{\"code\":-32603}");
    assert_eq!(error.id(), "7");
    assert_eq!(error.error(), "{\"code\":-32603}");
}

#[test]
fn jsonrpc_method_message_as_component_matches_java_branch_order() {
    assert_eq!(
        JsonRpcMethodMessage::literal("Plain").as_component(),
        Some(Component::literal("Plain"))
    );
    assert_eq!(
        JsonRpcMethodMessage::translatable("chat.type.text", None).as_component(),
        Some(Component::translatable("chat.type.text", Vec::new()))
    );
    assert_eq!(
        JsonRpcMethodMessage::translatable(
            "chat.type.announcement",
            Some(vec!["Server".to_string(), "Restart".to_string()]),
        )
        .as_component(),
        Some(Component::translatable(
            "chat.type.announcement",
            vec![
                ComponentArgument::String("Server".to_string()),
                ComponentArgument::String("Restart".to_string()),
            ],
        ))
    );
    assert_eq!(
        JsonRpcMethodMessage::new(
            Some("ignored".to_string()),
            Some("translation.wins".to_string()),
            None,
        )
        .as_component(),
        Some(Component::translatable("translation.wins", Vec::new()))
    );
    assert_eq!(JsonRpcMethodMessage::new(None, None, None).as_component(), None);
}

#[test]
fn client_info_of_matches_java_factory() {
    assert_eq!(ClientInfo::of(42), ClientInfo { connection_id: 42 });
}

#[test]
fn jsonrpc_logger_prefix_and_varargs_match_java() {
    const JSON_RPC_LOGGER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/JsonRpcLogger.java");
    for sentinel in [
        "private static final String PREFIX = \"RPC Connection #{}: \";",
        "public void log(final ClientInfo clientInfo, final String message, final Object... args)",
        "if (args.length == 0)",
        "LOGGER.info(\"RPC Connection #{}: \" + message, clientInfo.connectionId());",
        "List<Object> list = new ArrayList<>(Arrays.asList(args));",
        "list.addFirst(clientInfo.connectionId());",
        "LOGGER.info(\"RPC Connection #{}: \" + message, list.toArray());",
    ] {
        assert!(
            JSON_RPC_LOGGER_JAVA.contains(sentinel),
            "JsonRpcLogger.java missing sentinel: {sentinel}"
        );
    }

    assert_eq!(JsonRpcLogger::PREFIX, "RPC Connection #{}: ");
    assert_eq!(
        JsonRpcLogger::log_message(ClientInfo::of(7), "connected", &[]),
        (
            "RPC Connection #{}: connected".to_string(),
            vec!["7".to_string()]
        )
    );
    assert_eq!(
        JsonRpcLogger::log_message(ClientInfo::of(9), "sent {}", &["payload"]),
        (
            "RPC Connection #{}: sent {}".to_string(),
            vec!["9".to_string(), "payload".to_string()]
        )
    );
}

#[test]
fn discovery_service_matches_java_openrpc_metadata_and_filtering_order() {
    let schema = JsonRpcSchema::record("Test.CODEC")
        .with_field("field", JsonRpcSchema::of_type("string", "Codec.STRING"));
    let schema_registry = vec![SchemaComponent::new("test", schema)];
    let incoming_visible = method("minecraft:incoming/visible", "incoming visible", true);
    let incoming_hidden = method("minecraft:incoming/hidden", "incoming hidden", false);
    let outgoing_visible = method("minecraft:outgoing/visible", "outgoing visible", true);

    let response = DiscoveryService::discover(
        &schema_registry,
        &[incoming_visible.clone(), incoming_hidden],
        std::slice::from_ref(&outgoing_visible),
    );

    assert_eq!(response.json_rpc_protocol_version, "1.3.2");
    assert_eq!(response.discover_info.title, "Minecraft Server JSON-RPC");
    assert_eq!(response.discover_info.version, "2.0.0");
    assert_eq!(
        response
            .methods
            .iter()
            .map(|method| method.contents.description.as_str())
            .collect::<Vec<_>>(),
        vec!["incoming visible", "outgoing visible"]
    );
    assert_eq!(
        response.components.schemas.get("test").map(|schema| schema.codec.as_str()),
        Some("Test.CODEC")
    );
    assert_eq!(
        response
            .components
            .schemas
            .get("test")
            .and_then(|schema| schema.properties.get("field"))
            .map(|field| field.codec.as_str()),
        Some("Codec.STRING")
    );

    assert_eq!(incoming_visible.info.description, "incoming visible");
    assert_eq!(outgoing_visible.info.description, "outgoing visible");
}

#[test]
fn game_rules_service_matches_java_get_update_and_untyped_validation() {
    let mut game_rules = GameRules::new(false);
    let keep_inventory = game_rule("keep_inventory");
    let max_entity_cramming = game_rule("max_entity_cramming");

    let rules = GameRulesService::get(&game_rules);
    assert!(rules.iter().any(|update| {
        update.game_rule.name == "keep_inventory" && update.value == GameRuleValue::Bool(false)
    }));

    assert_eq!(
        GameRulesService::get_typed_rule(keep_inventory, GameRuleValue::Bool(true)),
        JsonRpcGameRuleUpdate::new(keep_inventory, GameRuleValue::Bool(true))
    );

    let update =
        JsonRpcGameRuleUpdate::from_untyped(keep_inventory, GameRuleType::Bool, GameRuleValue::Bool(true));
    assert_eq!(
        update,
        Ok(JsonRpcGameRuleUpdate::new(
            keep_inventory,
            GameRuleValue::Bool(true)
        ))
    );

    let mismatch = JsonRpcGameRuleUpdate::from_untyped(
        keep_inventory,
        GameRuleType::Int,
        GameRuleValue::Bool(true),
    );
    assert_eq!(
        mismatch,
        Err(JsonRpcRuntimeException::invalid_parameter(
            "Stated type \"integer\" mismatches with actual type \"boolean\" of gamerule \"keep_inventory\""
        ))
    );

    let updated = GameRulesService::update(
        &mut game_rules,
        JsonRpcGameRuleUpdate::new(max_entity_cramming, GameRuleValue::Int(7)),
        ClientInfo::of(9),
    );
    assert_eq!(
        updated,
        Ok(JsonRpcGameRuleUpdate::new(
            max_entity_cramming,
            GameRuleValue::Int(7)
        ))
    );
    assert_eq!(game_rules.get("max_entity_cramming"), Some(GameRuleValue::Int(7)));
}

#[test]
fn minecraft_game_rule_service_covers_typed_values_updates_and_availability() {
    let keep_inventory = game_rule("keep_inventory");
    let max_entity_cramming = game_rule("max_entity_cramming");
    let max_minecart_speed = game_rule("max_minecart_speed");
    let mut game_rules = GameRules::new(false);

    assert_eq!(
        game_rules.get_rule_value(keep_inventory),
        Some(GameRuleValue::Bool(false))
    );
    assert_eq!(
        game_rules.get_rule_value(max_entity_cramming),
        Some(GameRuleValue::Int(24))
    );
    assert_eq!(
        game_rules.get_typed_rule(keep_inventory, GameRuleValue::Bool(true)),
        JsonRpcGameRuleUpdate::new(keep_inventory, GameRuleValue::Bool(true))
    );
    assert_eq!(
        game_rules.update_game_rule(
            JsonRpcGameRuleUpdate::new(max_entity_cramming, GameRuleValue::Int(8)),
            ClientInfo::of(41),
        ),
        Ok(JsonRpcGameRuleUpdate::new(
            max_entity_cramming,
            GameRuleValue::Int(8)
        ))
    );
    assert_eq!(
        game_rules.get_rule_value(max_entity_cramming),
        Some(GameRuleValue::Int(8))
    );
    assert!(!game_rules
        .get_available_game_rules()
        .contains(&max_minecart_speed));
    assert!(GameRules::new(true)
        .get_available_game_rules()
        .contains(&max_minecart_speed));
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn minecraft_game_rule_service_interface_matches_java_contract() {
    const SOURCE: &str = vibecraft_java_source!(
        "/net/minecraft/server/jsonrpc/internalapi/MinecraftGameRuleService.java"
    );
    for sentinel in [
        "import java.util.stream.Stream;",
        "import net.minecraft.server.jsonrpc.methods.ClientInfo;",
        "import net.minecraft.server.jsonrpc.methods.GameRulesService;",
        "import net.minecraft.world.level.gamerules.GameRule;",
        "<T> GameRulesService.GameRuleUpdate<T> updateGameRule(GameRulesService.GameRuleUpdate<T> update, ClientInfo clientInfo);",
        "<T> T getRuleValue(GameRule<T> gameRule);",
        "<T> GameRulesService.GameRuleUpdate<T> getTypedRule(GameRule<T> gameRule, T value);",
        "Stream<GameRule<?>> getAvailableGameRules();",
    ] {
        assert!(
            SOURCE.contains(sentinel),
            "MinecraftGameRuleService.java missing: {sentinel}"
        );
    }
}

#[test]
fn minecraft_game_rule_service_impl_mutates_then_logs_serialized_values() {
    let server = DedicatedServerIdentity(51);
    let keep_inventory = game_rule("keep_inventory");
    let max_entity_cramming = game_rule("max_entity_cramming");
    let mut service = MinecraftGameRuleServiceImplModel::new(server, GameRules::new(false));

    let bool_update =
        JsonRpcGameRuleUpdate::new(keep_inventory, GameRuleValue::Bool(true));
    let int_update =
        JsonRpcGameRuleUpdate::new(max_entity_cramming, GameRuleValue::Int(9));
    assert_eq!(
        service.update_game_rule(bool_update, ClientInfo::of(52)),
        Ok(bool_update)
    );
    assert_eq!(
        service.update_game_rule(int_update, ClientInfo::of(53)),
        Ok(int_update)
    );
    assert_eq!(service.server, server);
    assert_eq!(
        service.get_rule_value(keep_inventory),
        Some(GameRuleValue::Bool(true))
    );
    assert_eq!(
        service.get_rule_value(max_entity_cramming),
        Some(GameRuleValue::Int(9))
    );
    assert_eq!(
        service.log_messages,
        vec![
            (
                ClientInfo::of(52),
                "Game rule 'minecraft:keep_inventory' updated from 'false' to 'true'"
                    .to_string(),
            ),
            (
                ClientInfo::of(53),
                "Game rule 'minecraft:max_entity_cramming' updated from '24' to '9'"
                    .to_string(),
            ),
        ]
    );

    let rejected = JsonRpcGameRuleUpdate::new(keep_inventory, GameRuleValue::Int(1));
    assert_eq!(
        service.update_game_rule(rejected, ClientInfo::of(54)),
        Err(GameRuleError::WrongType)
    );
    assert_eq!(service.log_messages.len(), 2);
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn minecraft_game_rule_service_impl_matches_java_contract() {
    const SOURCE: &str = vibecraft_java_source!(
        "/net/minecraft/server/jsonrpc/internalapi/MinecraftGameRuleServiceImpl.java"
    );
    for sentinel in [
        "private final DedicatedServer server;",
        "private final GameRules gameRules;",
        "private final JsonRpcLogger jsonrpcLogger;",
        "public MinecraftGameRuleServiceImpl(final DedicatedServer server, final JsonRpcLogger jsonrpcLogger)",
        "this.server = server;",
        "this.gameRules = server.getGameRules();",
        "this.jsonrpcLogger = jsonrpcLogger;",
        "GameRule<T> gameRule = update.gameRule();",
        "T oldValue = this.gameRules.get(gameRule);",
        "T newValue = update.value();",
        "this.gameRules.set(gameRule, newValue, this.server);",
        "this.jsonrpcLogger.log(clientInfo, \"Game rule '{}' updated from '{}' to '{}'\", gameRule.id(), gameRule.serialize(oldValue), gameRule.serialize(newValue));",
        "return update;",
        "return new GameRulesService.GameRuleUpdate<>(gameRule, value);",
        "return this.gameRules.availableRules();",
        "return this.gameRules.get(gameRule);",
    ] {
        assert!(
            SOURCE.contains(sentinel),
            "MinecraftGameRuleServiceImpl.java missing: {sentinel}"
        );
    }
}

#[test]
fn server_settings_service_matches_java_boolean_forwarders() {
    let mut settings = JsonRpcServerSettings::default();
    let client_info = ClientInfo::of(11);

    assert!(ServerSettingsService::autosave(&settings));
    assert!(!ServerSettingsService::set_autosave(&mut settings, false, client_info));
    assert!(ServerSettingsService::set_enforce_allowlist(
        &mut settings,
        true,
        client_info
    ));
    assert!(ServerSettingsService::set_using_allowlist(
        &mut settings,
        true,
        client_info
    ));
    assert!(ServerSettingsService::set_allow_flight(
        &mut settings,
        true,
        client_info
    ));
    assert!(ServerSettingsService::set_force_game_mode(
        &mut settings,
        true,
        client_info
    ));
    assert!(ServerSettingsService::set_accept_transfers(
        &mut settings,
        true,
        client_info
    ));
    assert!(ServerSettingsService::set_hides_online_players(
        &mut settings,
        true,
        client_info
    ));
    assert!(!ServerSettingsService::set_replies_to_status(
        &mut settings,
        false,
        client_info
    ));

    assert!(ServerSettingsService::enforce_allowlist(&settings));
    assert!(ServerSettingsService::using_allowlist(&settings));
    assert!(ServerSettingsService::allow_flight(&settings));
    assert!(ServerSettingsService::force_game_mode(&settings));
    assert!(ServerSettingsService::accept_transfers(&settings));
    assert!(ServerSettingsService::hides_online_players(&settings));
    assert!(!ServerSettingsService::replies_to_status(&settings));
}

#[test]
fn server_settings_service_matches_java_numeric_forwarders() {
    let mut settings = JsonRpcServerSettings::default();
    let client_info = ClientInfo::of(11);

    assert_eq!(
        ServerSettingsService::set_max_players(&mut settings, 77, client_info),
        77
    );
    assert_eq!(
        ServerSettingsService::set_pause_when_empty(&mut settings, 5, client_info),
        5
    );
    assert_eq!(
        ServerSettingsService::set_player_idle_timeout(&mut settings, 12, client_info),
        12
    );
    assert_eq!(
        ServerSettingsService::set_spawn_protection(&mut settings, 0, client_info),
        0
    );
    assert_eq!(
        ServerSettingsService::set_view_distance(&mut settings, 14, client_info),
        14
    );
    assert_eq!(
        ServerSettingsService::set_simulation_distance(&mut settings, 9, client_info),
        9
    );
    assert_eq!(
        ServerSettingsService::set_status_heartbeat_interval(&mut settings, 30, client_info),
        30
    );
    assert_eq!(
        ServerSettingsService::set_entity_broadcast_range_percentage(
            &mut settings,
            150,
            client_info,
        ),
        150
    );

    assert_eq!(ServerSettingsService::max_players(&settings), 77);
    assert_eq!(ServerSettingsService::pause_when_empty(&settings), 5);
    assert_eq!(ServerSettingsService::player_idle_timeout(&settings), 12);
    assert_eq!(ServerSettingsService::spawn_protection(&settings), 0);
    assert_eq!(ServerSettingsService::view_distance(&settings), 14);
    assert_eq!(ServerSettingsService::simulation_distance(&settings), 9);
    assert_eq!(ServerSettingsService::status_heartbeat_interval(&settings), 30);
    assert_eq!(
        ServerSettingsService::entity_broadcast_range_percentage(&settings),
        150
    );
}

#[test]
fn server_settings_service_matches_java_enum_and_text_forwarders() {
    let mut settings = JsonRpcServerSettings::default();
    let client_info = ClientInfo::of(11);

    assert_eq!(
        ServerSettingsService::set_difficulty(
            &mut settings,
            JsonRpcDifficulty::Hard,
            client_info,
        ),
        JsonRpcDifficulty::Hard
    );
    assert_eq!(
        ServerSettingsService::set_game_mode(&mut settings, GameMode::Creative, client_info),
        GameMode::Creative
    );
    assert_eq!(
        ServerSettingsService::set_motd(&mut settings, "Managed", client_info),
        "Managed"
    );

    assert_eq!(
        ServerSettingsService::difficulty(&settings),
        JsonRpcDifficulty::Hard
    );
    assert_eq!(ServerSettingsService::game_mode(&settings), GameMode::Creative);
    assert_eq!(ServerSettingsService::motd(&settings), "Managed");
}

#[test]
fn server_settings_service_matches_java_permission_level_conversion() {
    let mut settings = JsonRpcServerSettings::default();
    let client_info = ClientInfo::of(11);

    assert_eq!(
        ServerSettingsService::set_operator_user_permission_level(
            &mut settings,
            PermissionLevel::Admins,
            client_info,
        ),
        PermissionLevel::Admins
    );
    assert_eq!(
        ServerSettingsService::operator_user_permission_level(&settings),
        PermissionLevel::Admins
    );
}

#[test]
fn minecraft_server_settings_service_covers_all_getter_setter_pairs() {
    let mut settings = JsonRpcServerSettings::default();
    let client = ClientInfo::of(101);
    assert!(!settings.set_auto_save(false, client));
    assert_eq!(settings.set_difficulty(JsonRpcDifficulty::Hard, client), JsonRpcDifficulty::Hard);
    assert!(settings.set_enforce_whitelist(true, client));
    assert!(settings.set_using_whitelist(true, client));
    assert_eq!(settings.set_max_players(40, client), 40);
    assert_eq!(settings.set_pause_when_empty_seconds(5, client), 5);
    assert_eq!(settings.set_player_idle_timeout(15, client), 15);
    assert!(settings.set_allow_flight(true, client));
    assert_eq!(settings.set_spawn_protection_radius(3, client), 3);
    assert_eq!(settings.set_motd("Parity".to_string(), client), "Parity");
    assert!(settings.set_force_game_mode(true, client));
    assert_eq!(settings.set_game_mode(GameMode::Creative, client), GameMode::Creative);
    assert_eq!(settings.set_view_distance(16, client), 16);
    assert_eq!(settings.set_simulation_distance(12, client), 12);
    assert!(settings.set_accepts_transfers(true, client));
    assert_eq!(settings.set_status_heartbeat_interval(30, client), 30);
    assert_eq!(
        settings.set_operator_user_permissions(LevelBasedPermissionSet::ADMIN, client),
        LevelBasedPermissionSet::ADMIN
    );
    assert!(settings.set_hides_online_players(true, client));
    assert!(!settings.set_replies_to_status(false, client));
    assert_eq!(settings.set_entity_broadcast_range_percentage(150, client), 150);

    assert_minecraft_server_settings_values(&settings);
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn minecraft_server_settings_service_interface_matches_java_contract() {
    const SOURCE: &str = vibecraft_java_source!(
        "/net/minecraft/server/jsonrpc/internalapi/MinecraftServerSettingsService.java"
    );
    for sentinel in [
        "boolean isAutoSave();", "boolean setAutoSave(boolean enabled, ClientInfo clientInfo);",
        "Difficulty getDifficulty();", "Difficulty setDifficulty(Difficulty difficulty, ClientInfo clientInfo);",
        "boolean isEnforceWhitelist();", "boolean setEnforceWhitelist(boolean enforce, ClientInfo clientInfo);",
        "boolean isUsingWhitelist();", "boolean setUsingWhitelist(boolean use, ClientInfo clientInfo);",
        "int getMaxPlayers();", "int setMaxPlayers(int maxPlayers, ClientInfo clientInfo);",
        "int getPauseWhenEmptySeconds();", "int setPauseWhenEmptySeconds(int emptySeconds, ClientInfo clientInfo);",
        "int getPlayerIdleTimeout();", "int setPlayerIdleTimeout(int idleTime, ClientInfo clientInfo);",
        "boolean allowFlight();", "boolean setAllowFlight(boolean allow, ClientInfo clientInfo);",
        "int getSpawnProtectionRadius();", "int setSpawnProtectionRadius(int spawnProtection, ClientInfo clientInfo);",
        "String getMotd();", "String setMotd(String motd, ClientInfo clientInfo);",
        "boolean forceGameMode();", "boolean setForceGameMode(boolean force, ClientInfo clientInfo);",
        "GameType getGameMode();", "GameType setGameMode(GameType gameMode, ClientInfo clientInfo);",
        "int getViewDistance();", "int setViewDistance(int viewDistance, ClientInfo clientInfo);",
        "int getSimulationDistance();", "int setSimulationDistance(int simulationDistance, ClientInfo clientInfo);",
        "boolean acceptsTransfers();", "boolean setAcceptsTransfers(boolean accept, ClientInfo clientInfo);",
        "int getStatusHeartbeatInterval();", "int setStatusHeartbeatInterval(int statusHeartbeatInterval, ClientInfo clientInfo);",
        "LevelBasedPermissionSet getOperatorUserPermissions();",
        "LevelBasedPermissionSet setOperatorUserPermissions(LevelBasedPermissionSet level, ClientInfo clientInfo);",
        "boolean hidesOnlinePlayers();", "boolean setHidesOnlinePlayers(boolean hide, ClientInfo clientInfo);",
        "boolean repliesToStatus();", "boolean setRepliesToStatus(boolean enable, ClientInfo clientInfo);",
        "int getEntityBroadcastRangePercentage();",
        "int setEntityBroadcastRangePercentage(int percentage, ClientInfo clientInfo);",
    ] {
        assert!(
            SOURCE.contains(sentinel),
            "MinecraftServerSettingsService.java missing: {sentinel}"
        );
    }
}

fn assert_minecraft_server_settings_values(settings: &JsonRpcServerSettings) {
    assert!(!settings.is_auto_save());
    assert_eq!(settings.get_difficulty(), JsonRpcDifficulty::Hard);
    assert!(settings.is_enforce_whitelist());
    assert!(settings.is_using_whitelist());
    assert_eq!(settings.get_max_players(), 40);
    assert_eq!(settings.get_pause_when_empty_seconds(), 5);
    assert_eq!(settings.get_player_idle_timeout(), 15);
    assert!(settings.allow_flight());
    assert_eq!(settings.get_spawn_protection_radius(), 3);
    assert_eq!(settings.get_motd(), "Parity");
    assert!(settings.force_game_mode());
    assert_eq!(settings.get_game_mode(), GameMode::Creative);
    assert_eq!(settings.get_view_distance(), 16);
    assert_eq!(settings.get_simulation_distance(), 12);
    assert!(settings.accepts_transfers());
    assert_eq!(settings.get_status_heartbeat_interval(), 30);
    assert_eq!(
        settings.get_operator_user_permissions(),
        LevelBasedPermissionSet::ADMIN
    );
    assert!(settings.hides_online_players());
    assert!(!settings.replies_to_status());
    assert_eq!(settings.get_entity_broadcast_range_percentage(), 150);
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn jsonrpc_exception_message_sources_match_java_26_1_2() {
    const ENCODE: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/EncodeJsonRpcException.java");
    const INVALID_PARAMETER: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/InvalidParameterJsonRpcException.java");
    const INVALID_REQUEST: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/InvalidRequestJsonRpcException.java");
    const METHOD_NOT_FOUND: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/MethodNotFoundJsonRpcException.java");
    const REMOTE: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/RemoteRpcErrorException.java");
    const MESSAGE: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/Message.java");
    const CLIENT_INFO: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/ClientInfo.java");

    for (name, source, sentinel) in [
        (
            "EncodeJsonRpcException.java",
            ENCODE,
            "public EncodeJsonRpcException(final String message)",
        ),
        (
            "InvalidParameterJsonRpcException.java",
            INVALID_PARAMETER,
            "public InvalidParameterJsonRpcException(final String message)",
        ),
        (
            "InvalidRequestJsonRpcException.java",
            INVALID_REQUEST,
            "public InvalidRequestJsonRpcException(final String message)",
        ),
        (
            "MethodNotFoundJsonRpcException.java",
            METHOD_NOT_FOUND,
            "public MethodNotFoundJsonRpcException(final String message)",
        ),
    ] {
        assert!(source.contains("extends RuntimeException"), "{name} runtime base missing");
        assert!(source.contains(sentinel), "{name} constructor missing");
        assert!(source.contains("super(message);"), "{name} super message missing");
    }

    assert_java_source_contains_all(
        "RemoteRpcErrorException.java",
        REMOTE,
        &[
            "private final JsonElement id;",
            "private final JsonObject error;",
            "public RemoteRpcErrorException(final JsonElement id, final JsonObject error)",
            "return this.error;",
            "return this.id;",
        ],
    );

    assert_java_source_contains_all(
        "Message.java",
        MESSAGE,
        &[
        "public record Message(Optional<String> literal, Optional<String> translatable, Optional<List<String>> translatableParams)",
        "Codec.STRING.optionalFieldOf(\"literal\").forGetter(Message::literal)",
        "Codec.STRING.optionalFieldOf(\"translatable\").forGetter(Message::translatable)",
        "Codec.STRING.listOf().lenientOptionalFieldOf(\"translatableParams\").forGetter(Message::translatableParams)",
        "if (this.translatable.isPresent())",
        "return Optional.of(Component.translatable(translationKey, translationArgs.toArray()));",
        "return Optional.of(Component.translatable(translationKey));",
        "return this.literal.map(Component::literal);",
    ],
    );

    assert_java_source_contains_all(
        "ClientInfo.java",
        CLIENT_INFO,
        &[
            "public record ClientInfo(Integer connectionId)",
            "public static ClientInfo of(final Integer connectionId)",
            "return new ClientInfo(connectionId);",
        ],
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn jsonrpc_service_sources_match_java_26_1_2() {
    const DISCOVERY: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/DiscoveryService.java");
    const GAME_RULES: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/GameRulesService.java");
    const SERVER_SETTINGS: &str =
        vibecraft_java_source!("/net/minecraft/server/jsonrpc/methods/ServerSettingsService.java");

    assert_java_source_contains_all(
        "DiscoveryService.java",
        DISCOVERY,
        &[
        "public static DiscoveryService.DiscoverResponse discover(final List<SchemaComponent<?>> schemaRegistry)",
        "new ArrayList<>(BuiltInRegistries.INCOMING_RPC_METHOD.size() + BuiltInRegistries.OUTGOING_RPC_METHOD.size())",
        "if (e.value().attributes().discoverable())",
        "methods.add(e.value().info().named(e.key().identifier()));",
        "schemas.put(component.name(), component.schema().info());",
        "new DiscoveryService.DiscoverInfo(\"Minecraft Server JSON-RPC\", \"2.0.0\")",
        "return new DiscoveryService.DiscoverResponse(\"1.3.2\", discoverInfo, methods, new DiscoveryService.DiscoverComponents(schemas));",
        "public record DiscoverComponents(Map<String, Schema<?>> schemas)",
        "public record DiscoverInfo(String title, String version)",
        "public record DiscoverResponse(",
        "Codec.STRING.fieldOf(\"openrpc\").forGetter(DiscoveryService.DiscoverResponse::jsonRpcProtocolVersion)",
    ],
    );

    assert_java_source_contains_all(
        "GameRulesService.java",
        GAME_RULES,
        &[
        "public static List<GameRulesService.GameRuleUpdate<?>> get(final MinecraftApi minecraftApi)",
        "minecraftApi.gameRuleService().getAvailableGameRules().forEach(gameRule -> addGameRule(minecraftApi, (GameRule<?>)gameRule, rules));",
        "rules.add(getTypedRule(minecraftApi, gameRule, value));",
        "return minecraftApi.gameRuleService().getTypedRule(gameRule, value);",
        "return minecraftApi.gameRuleService().updateGameRule(update, clientInfo);",
        "public record GameRuleUpdate<T>(GameRule<T> gameRule, T value)",
        "BuiltInRegistries.GAME_RULE",
        "dispatch(\"key\", GameRulesService.GameRuleUpdate::gameRule, GameRulesService.GameRuleUpdate::getValueAndTypeCodec)",
        "StringRepresentable.fromEnum(GameRuleType::values).fieldOf(\"type\")",
        "if (gameRule.gameRuleType() != readType)",
        "throw new InvalidParameterJsonRpcException(",
        "Stated type \\\"\" + readType + \"\\\" mismatches with actual type \\\"\" + gameRule.gameRuleType() + \"\\\" of gamerule \\\"\" + gameRule.id() + \"\\\"\"",
        "return new GameRulesService.GameRuleUpdate<>(gameRule, value);",
    ],
    );

    assert_server_settings_source_matches_java(SERVER_SETTINGS);
}

#[cfg(vibecraft_has_decompiled_sources)]
fn assert_server_settings_source_matches_java(source: &str) {
    assert_java_source_contains_all(
        "ServerSettingsService.java",
        source,
        &[
        "public class ServerSettingsService",
        "return minecraftApi.serverSettingsService().isAutoSave();",
        "return minecraftApi.serverSettingsService().setAutoSave(enabled, clientInfo);",
        "return minecraftApi.serverSettingsService().getDifficulty();",
        "return minecraftApi.serverSettingsService().setDifficulty(difficulty, clientInfo);",
        "return minecraftApi.serverSettingsService().isEnforceWhitelist();",
        "return minecraftApi.serverSettingsService().setEnforceWhitelist(enforce, clientInfo);",
        "return minecraftApi.serverSettingsService().isUsingWhitelist();",
        "return minecraftApi.serverSettingsService().setUsingWhitelist(use, clientInfo);",
        "return minecraftApi.serverSettingsService().getMaxPlayers();",
        "return minecraftApi.serverSettingsService().setMaxPlayers(maxPlayers, clientInfo);",
        "return minecraftApi.serverSettingsService().getPauseWhenEmptySeconds();",
        "return minecraftApi.serverSettingsService().setPauseWhenEmptySeconds(emptySeconds, clientInfo);",
        "return minecraftApi.serverSettingsService().getPlayerIdleTimeout();",
        "return minecraftApi.serverSettingsService().setPlayerIdleTimeout(idleTime, clientInfo);",
        "return minecraftApi.serverSettingsService().allowFlight();",
        "return minecraftApi.serverSettingsService().setAllowFlight(allow, clientInfo);",
        "return minecraftApi.serverSettingsService().getSpawnProtectionRadius();",
        "return minecraftApi.serverSettingsService().setSpawnProtectionRadius(spawnProtection, clientInfo);",
        "return minecraftApi.serverSettingsService().getMotd();",
        "return minecraftApi.serverSettingsService().setMotd(motd, clientInfo);",
        "return minecraftApi.serverSettingsService().forceGameMode();",
        "return minecraftApi.serverSettingsService().setForceGameMode(force, clientInfo);",
        "return minecraftApi.serverSettingsService().getGameMode();",
        "return minecraftApi.serverSettingsService().setGameMode(gameMode, clientInfo);",
        "return minecraftApi.serverSettingsService().getViewDistance();",
        "return minecraftApi.serverSettingsService().setViewDistance(viewDistance, clientInfo);",
        "return minecraftApi.serverSettingsService().getSimulationDistance();",
        "return minecraftApi.serverSettingsService().setSimulationDistance(simulationDistance, clientInfo);",
        "return minecraftApi.serverSettingsService().acceptsTransfers();",
        "return minecraftApi.serverSettingsService().setAcceptsTransfers(accept, clientInfo);",
        "return minecraftApi.serverSettingsService().getStatusHeartbeatInterval();",
        "return minecraftApi.serverSettingsService().setStatusHeartbeatInterval(statusHeartbeatInterval, clientInfo);",
        "return minecraftApi.serverSettingsService().getOperatorUserPermissions().level();",
        "return minecraftApi.serverSettingsService().setOperatorUserPermissions(LevelBasedPermissionSet.forLevel(level), clientInfo).level();",
        "return minecraftApi.serverSettingsService().hidesOnlinePlayers();",
        "return minecraftApi.serverSettingsService().setHidesOnlinePlayers(hide, clientInfo);",
        "return minecraftApi.serverSettingsService().repliesToStatus();",
        "return minecraftApi.serverSettingsService().setRepliesToStatus(enable, clientInfo);",
        "return minecraftApi.serverSettingsService().getEntityBroadcastRangePercentage();",
        "return minecraftApi.serverSettingsService().setEntityBroadcastRangePercentage(percentage, clientInfo);",
    ],
    );
}

#[cfg(vibecraft_has_decompiled_sources)]
fn assert_java_source_contains_all(name: &str, source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "{name} is missing sentinel: {sentinel}"
        );
    }
}

fn method(id: &str, description: &str, discoverable: bool) -> DiscoverableJsonRpcMethod {
    let id = match Identifier::parse(id) {
        Ok(id) => id,
        Err(err) => panic!("test method id should parse: {err}"),
    };
    DiscoverableJsonRpcMethod::new(id, MethodInfo::new(description, None, None), discoverable)
}

fn game_rule(name: &str) -> GameRuleDefinition {
    match vanilla_game_rules()
        .iter()
        .find(|definition| definition.name == name)
    {
        Some(definition) => *definition,
        None => panic!("missing test game rule: {name}"),
    }
}
