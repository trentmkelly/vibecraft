use super::*;

#[test]
fn json_rpc_dispatch_covers_declared_methods_and_validates_schemas() {
    let mut state = ManagementServerState::default();
    for method in INCOMING_METHODS {
        let params = match incoming_params_schema(method) {
            Some("KickDto[]") => JsonRpcParams::Kick(Vec::new()),
            Some("PlayerDto[]") => JsonRpcParams::Players(Vec::new()),
            Some("IpBanDto[]") => JsonRpcParams::IpBans(Vec::new()),
            Some("GameRuleDto[]") => JsonRpcParams::GameRules(Vec::new()),
            Some(schema) => panic!("unexpected params schema {schema}"),
            None => JsonRpcParams::None,
        };
        let response = state.handle_request(JsonRpcRequest {
            id: JsonRpcId::String((*method).to_string()),
            method: (*method).to_string(),
            params,
        });
        assert!(
            response.result.is_ok(),
            "{method} should dispatch with schema-valid parameters"
        );
    }

    let invalid_for_no_params = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(12),
        method: "players/get".to_string(),
        params: JsonRpcParams::Kick(Vec::new()),
    });
    assert_eq!(
        invalid_for_no_params.result,
        Err(JsonRpcError::INVALID_PARAMS)
    );

    assert_eq!(
        validate_result_schema("players/get", &JsonRpcResult::Null),
        Err(JsonRpcError::INTERNAL_ERROR)
    );
}

#[test]
fn incoming_rpc_methods_registry_matches_java_bootstrap() {
    assert_incoming_rpc_methods_bootstrap_matches_java_source();
    assert_eq!(
        INCOMING_RPC_METHOD_DEFS.len(),
        INCOMING_RPC_METHODS_JAVA.matches(".register(methodRegistry,").count()
    );

    let mut previous_register = 0;
    for definition in &INCOMING_RPC_METHOD_DEFS[..INCOMING_RPC_METHOD_DEFS.len() - 1] {
        assert_incoming_rpc_method_definition_matches_java(definition, &mut previous_register);
    }

    let discovery = INCOMING_RPC_METHOD_DEFS
        .last()
        .expect("discovery method should be last");
    assert_incoming_rpc_method_definition_matches_java_without_order(discovery);
    assert_eq!(discovery.method, "rpc.discover");
    assert_eq!(discovery.description, "");
    assert_eq!(discovery.param, None);
    assert_eq!(discovery.result, ("result", "Schema.DISCOVERY_SCHEMA"));
    assert!(!discovery.run_on_main_thread);
    assert!(!discovery.discoverable);
    assert!(INCOMING_RPC_METHOD_DEFS[..INCOMING_RPC_METHOD_DEFS.len() - 1]
        .iter()
        .all(|definition| definition.run_on_main_thread && definition.discoverable));
}

fn assert_incoming_rpc_method_definition_matches_java(
    definition: &IncomingRpcMethodDef,
    previous_register: &mut usize,
) {
    assert_incoming_rpc_method_definition_matches_java_without_order(definition);

    let register_sentinel = format!(".register(methodRegistry, \"{}\")", definition.method);
    let register_index = INCOMING_RPC_METHODS_JAVA
        .find(&register_sentinel)
        .unwrap_or_else(|| {
            panic!("IncomingRpcMethods.java missing register sentinel: {register_sentinel}")
        });
    assert!(
        register_index >= *previous_register,
        "IncomingRpcMethods.java register out of order: {register_sentinel}"
    );
    *previous_register = register_index;
}

fn assert_incoming_rpc_method_definition_matches_java_without_order(
    definition: &IncomingRpcMethodDef,
) {
    if !definition.description.is_empty() {
        let description_sentinel = format!(".description(\"{}\")", definition.description);
        assert!(
            INCOMING_RPC_METHODS_JAVA.contains(&description_sentinel),
            "IncomingRpcMethods.java missing description sentinel: {description_sentinel}"
        );
    }

    if let Some((param_name, param_schema)) = definition.param {
        let param_sentinel = format!(".param(\"{param_name}\", {param_schema})");
        assert!(
            INCOMING_RPC_METHODS_JAVA.contains(&param_sentinel),
            "IncomingRpcMethods.java missing param sentinel: {param_sentinel}"
        );
    }

    let response_sentinel = format!(
        ".response(\"{}\", {})",
        definition.result.0, definition.result.1
    );
    assert!(
        INCOMING_RPC_METHODS_JAVA.contains(&response_sentinel),
        "IncomingRpcMethods.java missing response sentinel: {response_sentinel}"
    );

    let register_sentinel = format!(".register(methodRegistry, \"{}\")", definition.method);
    assert!(
        INCOMING_RPC_METHODS_JAVA.contains(&register_sentinel),
        "IncomingRpcMethods.java missing register sentinel: {register_sentinel}"
    );
}

fn assert_incoming_rpc_methods_bootstrap_matches_java_source() {
    for sentinel in [
        "public static IncomingRpcMethod<?, ?> bootstrap(final Registry<IncomingRpcMethod<?, ?>> methodRegistry)",
        "registerAllowListService(methodRegistry);",
        "registerBanlistService(methodRegistry);",
        "registerIpBanlistService(methodRegistry);",
        "registerPlayerService(methodRegistry);",
        "registerOperatorService(methodRegistry);",
        "registerServerStateService(methodRegistry);",
        "registerServerSettingsService(methodRegistry);",
        "registerGameRuleService(methodRegistry);",
        "IncomingRpcMethod.<DiscoveryService.DiscoverResponse>method(apiService -> DiscoveryService.discover(Schema.getSchemaRegistry()))",
        ".undiscoverable()",
        ".notOnMainThread()",
        ".response(\"result\", Schema.DISCOVERY_SCHEMA)",
        ".register(methodRegistry, \"rpc.discover\");",
    ] {
        assert!(
            INCOMING_RPC_METHODS_JAVA.contains(sentinel),
            "IncomingRpcMethods.java missing bootstrap sentinel: {sentinel}"
        );
    }
}

#[test]
fn outgoing_notification_methods_cover_vanilla_set_and_broadcast_to_clients() {
    let notifications = vec![
        OutgoingNotification::ServerStarted,
        OutgoingNotification::ServerStopping,
        OutgoingNotification::ServerSaving,
        OutgoingNotification::ServerSaved,
        OutgoingNotification::ServerActivity("tick".to_string()),
        OutgoingNotification::PlayersJoined(vec![]),
        OutgoingNotification::PlayersLeft(vec![]),
        OutgoingNotification::OperatorsAdded(vec![]),
        OutgoingNotification::OperatorsRemoved(vec![]),
        OutgoingNotification::AllowlistAdded(vec![]),
        OutgoingNotification::AllowlistRemoved(vec![]),
        OutgoingNotification::IpBansAdded(vec![]),
        OutgoingNotification::IpBansRemoved(vec![]),
        OutgoingNotification::BansAdded(vec![]),
        OutgoingNotification::BansRemoved(vec![]),
        OutgoingNotification::GameRulesUpdated(vec![]),
        OutgoingNotification::ServerStatus("running".to_string()),
    ];
    assert_eq!(
        notifications
            .iter()
            .map(OutgoingNotification::method)
            .collect::<Vec<_>>(),
        OUTGOING_METHODS
    );
    assert_eq!(
        OUTGOING_RPC_METHOD_DEFS
            .iter()
            .map(|definition| definition.method)
            .collect::<Vec<_>>(),
        OUTGOING_METHODS
    );
    assert_outgoing_rpc_methods_match_java_source();

    let mut state = ManagementServerState::default();
    state.connect_client("first");
    state.connect_client("second");
    state.broadcast(OutgoingNotification::ServerStarted);
    assert_eq!(
        state.notifications,
        vec![
            QueuedNotification {
                client_id: "first".to_string(),
                method: "server/started".to_string(),
            },
            QueuedNotification {
                client_id: "second".to_string(),
                method: "server/started".to_string(),
            },
        ]
    );
}

#[test]
fn outgoing_rpc_method_builder_and_variants_match_java_contract() {
    assert_outgoing_rpc_method_builder_matches_java_source();

    let notification = OutgoingRpcMethodBuilderModel::notification()
        .description("Server started")
        .build();
    assert_eq!(
        notification.kind,
        OutgoingRpcMethodKind::ParameterlessNotification
    );
    assert!(notification.discoverable);
    assert_eq!(notification.description, "Server started");
    assert_eq!(notification.default_encode_params(), None);
    assert_eq!(notification.default_decode_result(), None);
    assert_eq!(
        notification.encode_params(serde_json::json!({"ignored": true})),
        Ok(None)
    );
    assert_eq!(
        notification.decode_result(serde_json::json!(true)),
        Ok(None)
    );

    let notification_with_params = OutgoingRpcMethodBuilderModel::notification_with_params()
        .param("player", "PlayerDto")
        .build();
    assert_eq!(
        notification_with_params.encode_params(serde_json::json!({"name": "Steve"})),
        Ok(Some(serde_json::json!({"name": "Steve"})))
    );
    assert_eq!(
        notification_with_params.decode_result(serde_json::json!(null)),
        Ok(None)
    );

    let request = OutgoingRpcMethodBuilderModel::request()
        .response("result", "bool")
        .build();
    assert_eq!(
        request.kind,
        OutgoingRpcMethodKind::ParameterlessMethod
    );
    assert_eq!(
        request.encode_params(serde_json::json!(true)),
        Ok(None)
    );
    assert_eq!(
        request.decode_result(serde_json::json!(true)),
        Ok(Some(serde_json::json!(true)))
    );

    let request_with_params = OutgoingRpcMethodBuilderModel::request_with_params()
        .description("Set autosave")
        .param("enable", "bool")
        .response("enabled", "bool")
        .build();
    assert_eq!(
        request_with_params.encode_params(serde_json::json!(false)),
        Ok(Some(serde_json::json!(false)))
    );
    assert_eq!(
        request_with_params.decode_result(serde_json::json!(false)),
        Ok(Some(serde_json::json!(false)))
    );

    let bad_param_method = OutgoingRpcMethodBuilderModel::request_with_params()
        .response("result", "bool")
        .build();
    assert_eq!(
        bad_param_method.encode_params(serde_json::json!(true)),
        Err("Method defined as having no parameters".to_string())
    );
    let bad_result_method = OutgoingRpcMethodBuilderModel::request()
        .build();
    assert_eq!(
        bad_result_method.decode_result(serde_json::json!(true)),
        Err("Method defined as having no result".to_string())
    );
    assert_eq!(
        OutgoingRpcMethodBuilderModel::register_key("server/status"),
        "notification/server/status"
    );
}

fn assert_outgoing_rpc_method_builder_matches_java_source() {
    for sentinel in [
        "String NOTIFICATION_PREFIX = \"notification/\";",
        "default @Nullable JsonElement encodeParams(final Params params)",
        "return null;",
        "default @Nullable Result decodeResult(final JsonElement result)",
        "static OutgoingRpcMethod.OutgoingRpcMethodBuilder<Void, Void> notification()",
        "OutgoingRpcMethod.ParmeterlessNotification::new",
        "static <Params> OutgoingRpcMethod.OutgoingRpcMethodBuilder<Params, Void> notificationWithParams()",
        "OutgoingRpcMethod.Notification::new",
        "static <Result> OutgoingRpcMethod.OutgoingRpcMethodBuilder<Void, Result> request()",
        "OutgoingRpcMethod.ParameterlessMethod::new",
        "static <Params, Result> OutgoingRpcMethod.OutgoingRpcMethodBuilder<Params, Result> requestWithParams()",
        "OutgoingRpcMethod.Method::new",
        "record Attributes(boolean discoverable)",
        "throw new IllegalStateException(\"Method defined as having no parameters\");",
        "throw new IllegalStateException(\"Method defined as having no result\");",
        "public static final OutgoingRpcMethod.Attributes DEFAULT_ATTRIBUTES = new OutgoingRpcMethod.Attributes(true);",
        "private String description = \"\";",
        "this.resultInfo = new ResultInfo<>(resultName, resultSchema);",
        "this.paramInfo = new ParamInfo<>(paramName, paramSchema);",
        "MethodInfo<Params, Result> methodInfo = new MethodInfo<>(this.description, this.paramInfo, this.resultInfo);",
        "return this.method.create(methodInfo, DEFAULT_ATTRIBUTES);",
        "return this.register(Identifier.withDefaultNamespace(\"notification/\" + key));",
        "Registry.registerForHolder(BuiltInRegistries.OUTGOING_RPC_METHOD, id, this.build());",
    ] {
        assert!(
            OUTGOING_RPC_METHOD_JAVA.contains(sentinel),
            "OutgoingRpcMethod.java missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn incoming_rpc_method_builder_and_apply_match_java_contract() {
    assert_incoming_rpc_method_builder_matches_java_source();

    let method = IncomingRpcMethodBuilderModel::method_with_params()
        .description("Kick players")
        .param("players", "KickDto[]")
        .response("kicked", "PlayerDto[]")
        .undiscoverable()
        .not_on_main_thread()
        .build()
        .expect("method with params should build");

    assert_eq!(method.kind, IncomingRpcMethodKind::Method);
    assert_eq!(method.description, "Kick players");
    assert_eq!(method.param, Some(("players".to_string(), "KickDto[]".to_string())));
    assert_eq!(method.result, Some(("kicked".to_string(), "PlayerDto[]".to_string())));
    assert_eq!(
        method.attributes,
        IncomingRpcMethodAttributes {
            run_on_main_thread: false,
            discoverable: false,
        }
    );
    assert_incoming_param_method_apply_matches_java(&method);
    assert_incoming_parameterless_apply_matches_java();
    assert_incoming_builder_errors_match_java();
    assert_eq!(IncomingRpcMethodBuilderModel::register_key("players/kick"), "players/kick");
}

fn assert_incoming_param_method_apply_matches_java(method: &IncomingRpcMethodModel) {
    assert_eq!(
        method.extract_param(Some(&serde_json::json!({"players": [{"name": "Steve"}]}))),
        Ok(serde_json::json!([{"name": "Steve"}]))
    );
    assert_eq!(
        method.extract_param(Some(&serde_json::json!([[{"name": "Alex"}]]))),
        Ok(serde_json::json!([{"name": "Alex"}]))
    );
    assert_eq!(
        method.apply(
            Some(&serde_json::json!({"players": []})),
            serde_json::json!([{"name": "Steve"}])
        ),
        Ok(serde_json::json!([{"name": "Steve"}]))
    );
    assert_eq!(
        method.extract_param(None),
        Err(incoming_invalid_parameter("Expected params as array or named"))
    );
    assert_eq!(
        method.extract_param(Some(&serde_json::json!(true))),
        Err(incoming_invalid_parameter("Expected params as array or named"))
    );
    assert_eq!(
        method.extract_param(Some(&serde_json::json!({}))),
        Err(incoming_invalid_parameter(
            "Params passed by-name, but expected param [players] does not exist"
        ))
    );
    assert_eq!(
        method.extract_param(Some(&serde_json::json!([]))),
        Err(incoming_invalid_parameter(
            "Expected exactly one element in the params array"
        ))
    );
    assert_eq!(
        method.extract_param(Some(&serde_json::json!([1, 2]))),
        Err(incoming_invalid_parameter(
            "Expected exactly one element in the params array"
        ))
    );
}

fn assert_incoming_parameterless_apply_matches_java() {
    let parameterless = IncomingRpcMethodBuilderModel::method_parameterless()
        .response("status", "ServerStatusDto")
        .build()
        .expect("parameterless method should build");
    let supplier = IncomingRpcMethodBuilderModel::method_api_supplier()
        .response("status", "ServerStatusDto")
        .build()
        .expect("api supplier should build as parameterless");

    assert_eq!(parameterless.kind, IncomingRpcMethodKind::ParameterlessMethod);
    assert_eq!(supplier.kind, IncomingRpcMethodKind::ParameterlessMethod);
    assert_eq!(
        parameterless.attributes,
        IncomingRpcMethodAttributes {
            run_on_main_thread: true,
            discoverable: true,
        }
    );
    assert_eq!(parameterless.apply(None, serde_json::json!({"running": true})), Ok(serde_json::json!({"running": true})));
    assert_eq!(
        parameterless.apply(Some(&serde_json::json!([])), serde_json::json!(true)),
        Ok(serde_json::json!(true))
    );
    assert_eq!(
        parameterless.validate_parameterless_params(Some(&serde_json::json!({}))),
        Err(incoming_invalid_parameter("Expected no params, or an empty array"))
    );
    assert_eq!(
        parameterless.validate_parameterless_params(Some(&serde_json::json!([1]))),
        Err(incoming_invalid_parameter("Expected no params, or an empty array"))
    );
}

fn assert_incoming_builder_errors_match_java() {
    assert_eq!(
        IncomingRpcMethodBuilderModel::method_parameterless().build(),
        Err(incoming_illegal_state("No response defined"))
    );
    assert_eq!(
        IncomingRpcMethodBuilderModel::method_with_params()
            .response("result", "bool")
            .build(),
        Err(incoming_illegal_state("No param schema defined"))
    );
    assert_eq!(
        IncomingRpcMethodBuilderModel::missing_function_for_test()
            .response("result", "bool")
            .build(),
        Err(incoming_illegal_state("No method defined"))
    );

    let direct_paramless_with_param = IncomingRpcMethodModel {
        description: String::new(),
        param: Some(("unexpected".to_string(), "bool".to_string())),
        result: Some(("result".to_string(), "bool".to_string())),
        attributes: IncomingRpcMethodAttributes {
            run_on_main_thread: true,
            discoverable: true,
        },
        kind: IncomingRpcMethodKind::ParameterlessMethod,
    };
    assert_eq!(
        direct_paramless_with_param.validate_parameterless_params(None),
        Err(incoming_illegal_argument(
            "Parameterless method unexpectedly has parameter description"
        ))
    );

    let direct_method_without_param = IncomingRpcMethodModel {
        kind: IncomingRpcMethodKind::Method,
        param: None,
        ..direct_paramless_with_param
    };
    assert_eq!(
        direct_method_without_param.extract_param(Some(&serde_json::json!([]))),
        Err(incoming_illegal_argument(
            "Method defined as having parameters without describing them"
        ))
    );

    let direct_method_without_result = IncomingRpcMethodModel {
        description: String::new(),
        param: Some(("param".to_string(), "bool".to_string())),
        result: None,
        attributes: IncomingRpcMethodAttributes {
            run_on_main_thread: true,
            discoverable: true,
        },
        kind: IncomingRpcMethodKind::Method,
    };
    assert_eq!(
        direct_method_without_result.encode_result(serde_json::json!(true)),
        Err(incoming_illegal_state("No result codec defined"))
    );
}

fn assert_incoming_rpc_method_builder_matches_java_source() {
    for sentinel in [
        "static <Result> IncomingRpcMethod.IncomingRpcMethodBuilder<Void, Result> method(final IncomingRpcMethod.ParameterlessRpcMethodFunction<Result> function)",
        "static <Params, Result> IncomingRpcMethod.IncomingRpcMethodBuilder<Params, Result> method(final IncomingRpcMethod.RpcMethodFunction<Params, Result> function)",
        "static <Result> IncomingRpcMethod.IncomingRpcMethodBuilder<Void, Result> method(final Function<MinecraftApi, Result> supplier)",
        "record Attributes(boolean runOnMainThread, boolean discoverable)",
        "private String description = \"\";",
        "private boolean discoverable = true;",
        "private boolean runOnMainThread = true;",
        "this.parameterlessFunction = (apiService, clientInfo) -> supplier.apply(apiService);",
        "this.resultInfo = new ResultInfo<>(resultName, resultSchema.info());",
        "this.paramInfo = new ParamInfo<>(paramName, paramSchema.info());",
        "this.discoverable = false;",
        "this.runOnMainThread = false;",
        "throw new IllegalStateException(\"No response defined\");",
        "throw new IllegalStateException(\"No param schema defined\");",
        "throw new IllegalStateException(\"No method defined\");",
        "return new IncomingRpcMethod.ParameterlessMethod<>(methodInfo, attributes, this.parameterlessFunction);",
        "return new IncomingRpcMethod.Method<>(methodInfo, attributes, this.parameterFunction);",
        "return this.register(methodRegistry, Identifier.withDefaultNamespace(key));",
        "return Registry.register(methodRegistry, id, this.build());",
    ] {
        assert!(
            INCOMING_RPC_METHOD_JAVA.contains(sentinel),
            "IncomingRpcMethod.java missing builder sentinel: {sentinel}"
        );
    }
    assert_incoming_rpc_method_apply_matches_java_source();
}

fn assert_incoming_rpc_method_apply_matches_java_source() {
    for sentinel in [
        "paramsJson != null && (paramsJson.isJsonArray() || paramsJson.isJsonObject())",
        "throw new IllegalArgumentException(\"Method defined as having parameters without describing them\");",
        "String parameterName = this.info.params().get().name();",
        "Params passed by-name, but expected param [%s] does not exist",
        "throw new InvalidParameterJsonRpcException(\"Expected exactly one element in the params array\");",
        "throw new InvalidParameterJsonRpcException(\"Expected params as array or named\");",
        "if (paramsJson == null || paramsJson.isJsonArray() && paramsJson.getAsJsonArray().isEmpty())",
        "throw new IllegalArgumentException(\"Parameterless method unexpectedly has parameter description\");",
        "throw new IllegalStateException(\"No result codec defined\");",
        "throw new InvalidParameterJsonRpcException(\"Expected no params, or an empty array\");",
    ] {
        assert!(
            INCOMING_RPC_METHOD_JAVA.contains(sentinel),
            "IncomingRpcMethod.java missing apply sentinel: {sentinel}"
        );
    }
}

fn incoming_invalid_parameter(message: &str) -> IncomingRpcMethodError {
    IncomingRpcMethodError {
        kind: IncomingRpcMethodErrorKind::InvalidParameter,
        message: message.to_string(),
    }
}

fn incoming_illegal_state(message: &str) -> IncomingRpcMethodError {
    IncomingRpcMethodError {
        kind: IncomingRpcMethodErrorKind::IllegalState,
        message: message.to_string(),
    }
}

fn incoming_illegal_argument(message: &str) -> IncomingRpcMethodError {
    IncomingRpcMethodError {
        kind: IncomingRpcMethodErrorKind::IllegalArgument,
        message: message.to_string(),
    }
}

#[test]
fn json_rpc_notification_service_maps_events_to_outgoing_methods() {
    for sentinel in [
        "public class JsonRpcNotificationService implements NotificationService",
        "private final ManagementServer managementServer;",
        "private final MinecraftApi minecraftApi;",
        "public JsonRpcNotificationService(final MinecraftApi minecraftApi, final ManagementServer managementServer)",
        "this.minecraftApi = minecraftApi;",
        "this.managementServer = managementServer;",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_JOINED, PlayerDto.from(player));",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_LEFT, PlayerDto.from(player));",
        "this.broadcastNotification(OutgoingRpcMethods.SERVER_STARTED);",
        "this.broadcastNotification(OutgoingRpcMethods.SERVER_SHUTTING_DOWN);",
        "this.broadcastNotification(OutgoingRpcMethods.SERVER_SAVE_STARTED);",
        "this.broadcastNotification(OutgoingRpcMethods.SERVER_SAVE_COMPLETED);",
        "this.broadcastNotification(OutgoingRpcMethods.SERVER_ACTIVITY_OCCURRED);",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_OPED, OperatorService.OperatorDto.from(operator));",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_DEOPED, OperatorService.OperatorDto.from(operator));",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_ADDED_TO_ALLOWLIST, PlayerDto.from(player));",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_REMOVED_FROM_ALLOWLIST, PlayerDto.from(player));",
        "this.broadcastNotification(OutgoingRpcMethods.IP_BANNED, IpBanlistService.IpBanDto.from(ban));",
        "this.broadcastNotification(OutgoingRpcMethods.IP_UNBANNED, ip);",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_BANNED, BanlistService.UserBanDto.from(ban));",
        "this.broadcastNotification(OutgoingRpcMethods.PLAYER_UNBANNED, PlayerDto.from(player));",
        "this.broadcastNotification(OutgoingRpcMethods.GAMERULE_CHANGED, GameRulesService.getTypedRule(this.minecraftApi, gameRule, value));",
        "this.broadcastNotification(OutgoingRpcMethods.STATUS_HEARTBEAT, ServerStateService.status(this.minecraftApi));",
        "this.managementServer.forEachConnection(connection -> connection.sendNotification(method));",
        "this.managementServer.forEachConnection(connection -> connection.sendNotification(method, params));",
    ] {
        assert!(
            JSON_RPC_NOTIFICATION_SERVICE_JAVA.contains(sentinel),
            "JsonRpcNotificationService.java missing sentinel: {sentinel}"
        );
    }

    let steve = PlayerDto::from_profile(&player("Steve"));
    let alex = PlayerDto::from_profile(&player("Alex"));
    let mut state = ManagementServerState::default();
    state.connect_client("admin");
    {
        let mut service = JsonRpcNotificationService::new(&mut state);
        service.player_joined(steve.clone());
        service.player_left(steve.clone());
        service.server_started();
        service.server_shutting_down();
        service.server_save_started();
        service.server_save_completed();
        service.server_activity_occured();
        service.player_oped(alex.clone());
        service.player_deoped(alex.clone());
        service.player_added_to_allowlist(alex.clone());
        service.player_removed_from_allowlist(alex.clone());
        service.ip_banned("127.0.0.1".to_string());
        service.ip_unbanned("127.0.0.1".to_string());
        service.player_banned(steve.clone());
        service.player_unbanned(steve);
        service.on_game_rule_changed("doDaylightCycle".to_string());
        service.status_heartbeat("running".to_string());
    }

    assert_eq!(
        state
            .notifications
            .iter()
            .map(|notification| notification.method.as_str())
            .collect::<Vec<_>>(),
        vec![
            "players/joined",
            "players/left",
            "server/started",
            "server/stopping",
            "server/saving",
            "server/saved",
            "server/activity",
            "operators/added",
            "operators/removed",
            "allowlist/added",
            "allowlist/removed",
            "ip_bans/added",
            "ip_bans/removed",
            "bans/added",
            "bans/removed",
            "gamerules/updated",
            "server/status",
        ]
    );
    assert!(state
        .notifications
        .iter()
        .all(|notification| notification.client_id == "admin"));
}

fn assert_outgoing_rpc_methods_match_java_source() {
    for sentinel in [
        "String NOTIFICATION_PREFIX = \"notification/\";",
        "static OutgoingRpcMethod.OutgoingRpcMethodBuilder<Void, Void> notification()",
        "static <Params> OutgoingRpcMethod.OutgoingRpcMethodBuilder<Params, Void> notificationWithParams()",
        "public static final OutgoingRpcMethod.Attributes DEFAULT_ATTRIBUTES = new OutgoingRpcMethod.Attributes(true);",
        "return this.register(Identifier.withDefaultNamespace(\"notification/\" + key));",
    ] {
        assert!(
            OUTGOING_RPC_METHOD_JAVA.contains(sentinel),
            "OutgoingRpcMethod.java missing notification sentinel: {sentinel}"
        );
    }
    assert_eq!(
        OutgoingRpcMethodDef::NOTIFICATION_PREFIX,
        "notification/"
    );

    for definition in OUTGOING_RPC_METHOD_DEFS {
        let register_sentinel = format!(".register(\"{}\")", definition.method);
        let description_sentinel = format!(".description(\"{}\")", definition.description);
        assert!(
            OUTGOING_RPC_METHODS_JAVA.contains(&register_sentinel),
            "OutgoingRpcMethods.java missing register sentinel: {register_sentinel}"
        );
        assert!(
            OUTGOING_RPC_METHODS_JAVA.contains(&description_sentinel),
            "OutgoingRpcMethods.java missing description sentinel: {description_sentinel}"
        );
        if let Some((param_name, _)) = definition.param {
            let param_sentinel = format!(".param(\"{}\", Schema.", param_name);
            assert!(
                OUTGOING_RPC_METHODS_JAVA.contains(&param_sentinel),
                "OutgoingRpcMethods.java missing param sentinel: {param_sentinel}"
            );
        }
        assert_eq!(
            definition.registry_key(),
            format!("notification/{}", definition.method)
        );
        assert!(definition.discoverable);
    }
    assert_eq!(
        OUTGOING_RPC_METHOD_DEFS
            .iter()
            .filter(|definition| definition.param.is_none())
            .map(|definition| definition.method)
            .collect::<Vec<_>>(),
        vec![
            "server/started",
            "server/stopping",
            "server/saving",
            "server/saved",
            "server/activity"
        ]
    );
    assert_eq!(
        OUTGOING_RPC_METHOD_DEFS
            .iter()
            .find(|definition| definition.method == "server/status")
            .and_then(|definition| definition.param),
        Some(("status", "ServerState"))
    );
    assert_eq!(
        OUTGOING_RPC_METHOD_DEFS
            .iter()
            .find(|definition| definition.method == "ip_bans/removed")
            .and_then(|definition| definition.param),
        Some(("player", "string"))
    );
}
