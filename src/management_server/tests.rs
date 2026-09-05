use super::*;

const VALID_SECRET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCD";
const CONNECTION_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/jsonrpc/Connection.java");
const MANAGEMENT_SERVER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/ManagementServer.java");
const JSON_RPC_ERRORS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/JsonRPCErrors.java");
const JSON_RPC_UTILS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/JsonRPCUtils.java");
const OUTGOING_RPC_METHODS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/OutgoingRpcMethods.java");
const OUTGOING_RPC_METHOD_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/OutgoingRpcMethod.java");
const INCOMING_RPC_METHOD_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/IncomingRpcMethod.java");
const INCOMING_RPC_METHODS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/IncomingRpcMethods.java");
const PENDING_RPC_REQUEST_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/PendingRpcRequest.java");
const JSON_RPC_NOTIFICATION_SERVICE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/jsonrpc/JsonRpcNotificationService.java");

fn config() -> ManagementServerConfig {
    ManagementServerConfig {
        enabled: true,
        host: "localhost".to_string(),
        port: 25585,
        allowed_origins: AllowedOrigins::parse("https://admin.example"),
        security: ManagementSecurityConfig {
            secret_key: VALID_SECRET.to_string(),
            tls_enabled: false,
            tls_keystore: None,
            tls_keystore_password: None,
        },
    }
}

fn player(name: &str) -> NameAndId {
    NameAndId::create_offline(name)
}

#[test]
fn startup_refuses_invalid_secret_and_builds_plain_or_tls_listener() {
    let disabled = ManagementServerConfig {
        enabled: false,
        ..config()
    };
    assert_eq!(
        startup_plan(&disabled, None, None),
        ManagementStartupPlan::Disabled
    );

    let invalid_secret = ManagementServerConfig {
        security: ManagementSecurityConfig {
            secret_key: "bad".to_string(),
            ..config().security
        },
        ..config()
    };
    assert_eq!(
        startup_plan(&invalid_secret, None, None),
        ManagementStartupPlan::Refused(ManagementSecurityDecision::DisabledInvalidSecret)
    );

    assert!(matches!(
        startup_plan(&config(), None, None),
        ManagementStartupPlan::Listen { tls: None, .. }
    ));

    let tls = ManagementServerConfig {
        security: ManagementSecurityConfig {
            tls_enabled: true,
            tls_keystore: Some("management.p12".to_string()),
            tls_keystore_password: Some("server-pass".to_string()),
            ..config().security
        },
        ..config()
    };
    assert_eq!(
        startup_plan(&tls, Some("env-pass"), None),
        ManagementStartupPlan::Listen {
            host: "localhost".to_string(),
            port: 25585,
            tls: Some(TlsEndpoint {
                keystore: "management.p12".to_string(),
                password: "env-pass".to_string(),
            }),
            allowed_origins: AllowedOrigins::parse("https://admin.example"),
        }
    );
}

#[test]
fn management_server_lifecycle_and_pipeline_match_java_contract() {
    assert_management_server_java_source_shape();

    let mut server = ManagementServerRuntimeModel::new("127.0.0.1", 0, "bearer-auth");
    assert_eq!(server.get_port(), 0);
    assert!(!server.nio_event_loop_group_closed);

    server.start_without_tls(25_585);
    assert_eq!(server.get_port(), 25_585);
    assert_eq!(
        server.pipelines,
        vec![ManagementServerPipeline {
            tls: false,
            handlers: vec![
                "HttpServerCodec",
                "HttpObjectAggregator(65536)",
                "AuthenticationHandler",
                "WebSocketServerProtocolHandler(/)",
                "WebSocketToJsonCodec",
                "JsonToWebSocketEncoder",
                "Connection",
            ],
            tcp_no_delay_attempted: true,
        }]
    );
    assert_eq!(
        server.log_messages,
        vec!["Json-RPC Management connection listening on 127.0.0.1:25585".to_string()]
    );

    let mut connection = JsonRpcConnectionModel::new(1);
    connection.send_request("notification/server/status", &[], 10);
    server.on_connected(connection.clone());
    server.on_connected(connection);
    assert_eq!(server.connections.len(), 1);
    server.tick(5_011);
    assert_eq!(
        server.connections[0].completed_requests.get(&1),
        Some(&Err(
            "RPC method notification/server/status timed out waiting for response".to_string()
        ))
    );
    server.on_disconnected(1);
    assert!(server.connections.is_empty());

    server.start_with_tls(25_586);
    assert_eq!(
        server
            .pipelines
            .last()
            .map(|pipeline| pipeline.handlers.first().copied()),
        Some(Some("SslHandler"))
    );
    server.on_connected(JsonRpcConnectionModel::new(2));
    server.stop(false);
    assert_eq!(server.get_port(), 0);
    assert!(server.connections.is_empty());
    assert!(!server.nio_event_loop_group_closed);

    server.start_without_tls(25_587);
    server.stop(true);
    assert!(server.nio_event_loop_group_closed);
}

fn assert_management_server_java_source_shape() {
    for sentinel in [
        "private final HostAndPort hostAndPort;",
        "private final AuthenticationHandler authenticationHandler;",
        "private @Nullable Channel serverChannel;",
        "private final NioEventLoopGroup nioEventLoopGroup;",
        "private final Set<Connection> connections = Sets.newIdentityHashSet();",
        "setNameFormat(\"Management server IO #%d\").setDaemon(true).build()",
        "this.connections.add(connection);",
        "this.connections.remove(connection);",
        "public void startWithoutTls(final MinecraftApi minecraftApi)",
        "this.start(minecraftApi, null);",
        "public void startWithTls(final MinecraftApi minecraftApi, final SslContext sslContext)",
        "this.start(minecraftApi, sslContext);",
        "final JsonRpcLogger jsonrpcLogger = new JsonRpcLogger();",
        "new ServerBootstrap().handler(new LoggingHandler(LogLevel.DEBUG))",
        ".channel(NioServerSocketChannel.class)",
        "channel.config().setOption(ChannelOption.TCP_NODELAY, true);",
        "pipeline.addLast(new ChannelHandler[]{sslContext.newHandler(channel.alloc())});",
        "pipeline.addLast(new ChannelHandler[]{new HttpServerCodec()})",
        ".addLast(new ChannelHandler[]{new HttpObjectAggregator(65536)})",
        ".addLast(new ChannelHandler[]{ManagementServer.this.authenticationHandler})",
        ".addLast(new ChannelHandler[]{new WebSocketServerProtocolHandler(\"/\")})",
        ".addLast(new ChannelHandler[]{new WebSocketToJsonCodec()})",
        ".addLast(new ChannelHandler[]{new JsonToWebSocketEncoder()})",
        ".addLast(new ChannelHandler[]{new Connection(channel, ManagementServer.this, minecraftApi, jsonrpcLogger)});",
        ".group(this.nioEventLoopGroup)",
        ".localAddress(this.hostAndPort.getHost(), this.hostAndPort.getPort()))",
        "this.serverChannel = channel.channel();",
        "channel.syncUninterruptibly();",
        "Json-RPC Management connection listening on {}:{}",
        "this.serverChannel.close().sync();",
        "this.serverChannel = null;",
        "this.connections.clear();",
        "this.nioEventLoopGroup.shutdownGracefully().sync();",
        "this.forEachConnection(Connection::tick);",
        "return this.serverChannel != null ? ((InetSocketAddress)this.serverChannel.localAddress()).getPort() : this.hostAndPort.getPort();",
        "this.connections.forEach(action);",
    ] {
        assert!(
            MANAGEMENT_SERVER_JAVA.contains(sentinel),
            "ManagementServer.java missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn origins_parse_exact_wildcard_and_empty_policy() {
    let exact = AllowedOrigins::parse("https://a.example, https://b.example");
    assert!(exact.accepts(None));
    assert!(exact.accepts(Some("https://b.example")));
    assert!(!exact.accepts(Some("https://c.example")));
    assert!(AllowedOrigins::parse("*").accepts(Some("*")));
    assert!(!AllowedOrigins::parse("*").accepts(Some("https://anything.example")));
    assert!(!AllowedOrigins::parse("").accepts(Some("https://a.example")));
}

#[test]
fn authorization_checks_origin_before_constant_time_secret() {
    let cfg = config();
    assert_eq!(
        authorize_request(&cfg, Some("https://evil.example"), Some(VALID_SECRET)),
        ManagementAccessDecision::RejectedOrigin
    );
    assert_eq!(
        authorize_request(&cfg, Some("https://admin.example"), Some("wrong")),
        ManagementAccessDecision::Unauthorized
    );
    assert_eq!(
        authorize_request(&cfg, Some("https://admin.example"), Some(VALID_SECRET)),
        ManagementAccessDecision::Accepted
    );
}

#[test]
fn player_dto_resolves_by_id_before_name_fallback() {
    let steve = player("Steve");
    let alex = player("Alex");
    let dto = PlayerDto {
        id: Some(alex.uuid.clone()),
        name: Some("Steve".to_string()),
        latency: 0,
        game_mode: GameMode::Survival,
    };
    assert_eq!(dto.resolve(&[steve.clone(), alex.clone()]), Some(&alex));

    let by_name = PlayerDto {
        id: None,
        name: Some("steve".to_string()),
        latency: 0,
        game_mode: GameMode::Survival,
    };
    assert_eq!(
        by_name.resolve(&[steve.clone(), alex.clone()]),
        Some(&steve)
    );
}

#[test]
fn player_dto_json_includes_identity_latency_and_game_mode() {
    let steve = player("Steve");
    let dto = PlayerDto::with_state(&steve, 47, GameMode::Creative);
    assert_eq!(
        dto.to_json(),
        format!(
            "{{\"id\":\"{}\",\"name\":\"Steve\",\"latency\":47,\"gameMode\":\"creative\"}}",
            steve.uuid
        )
    );
}

#[test]
fn discovery_exposes_management_methods_notifications_and_schema_refs() {
    let discovery = DiscoveryDocument::vanilla();
    assert!(discovery
        .incoming_methods
        .iter()
        .any(|method| method.method == "players/kick"
            && method.params_schema.as_deref() == Some("KickDto[]")
            && method.result_schema.as_deref() == Some("PlayerDto[]")));
    assert!(discovery
        .outgoing_notifications
        .iter()
        .any(|method| method.method == "server/status"
            && method.params_schema.as_deref() == Some("ServerStatusDto")));
    assert!(discovery
        .incoming_methods
        .iter()
        .any(|method| method.method == "server/metrics"
            && method.result_schema.as_deref() == Some("ServerMetricsDto")));
    assert!(discovery
        .schemas
        .iter()
        .any(|schema| schema.name == "PlayerDto"
            && schema.fields == ["id", "name", "latency", "gameMode"]));
    assert!(discovery
        .schemas
        .iter()
        .any(|schema| schema.name == "ServerMetricsDto"
            && schema.fields.contains(&"tickDurationNanos".to_string())));
}

#[test]
fn jmx_property_uses_management_metrics_export_as_rust_equivalent() {
    let mut properties = ServerProperties::load_or_default(std::path::Path::new(
        "definitely-missing-test-server.properties",
    ))
    .unwrap();
    let management_plan = ManagementStartupPlan::Listen {
        host: "localhost".to_string(),
        port: 25585,
        tls: None,
        allowed_origins: AllowedOrigins::Empty,
    };

    assert_eq!(
        jmx_monitoring_plan(&properties, &management_plan),
        JmxMonitoringPlan::Disabled
    );

    properties.set("enable-jmx-monitoring", "true");
    assert_eq!(
        jmx_monitoring_plan(&properties, &management_plan),
        JmxMonitoringPlan::EquivalentMetricsExport {
            endpoint: "server/metrics",
            transport: "json-rpc-management",
        }
    );
    assert_eq!(
        jmx_monitoring_plan(&properties, &ManagementStartupPlan::Disabled),
        JmxMonitoringPlan::RefusedManagementDisabled
    );
}

#[test]
fn player_get_and_kick_match_vanilla_resolution_and_default_message() {
    let steve = player("Steve");
    let alex = player("Alex");
    let mut state = ManagementServerState {
        online_players: vec![steve.clone(), alex.clone()],
        ..Default::default()
    };
    state.connect_client("admin");

    let get = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(1),
        method: "players/get".to_string(),
        params: JsonRpcParams::None,
    });
    assert_eq!(
        get.result,
        Ok(JsonRpcResult::Players(vec![
            PlayerDto::from_profile(&steve),
            PlayerDto::from_profile(&alex)
        ]))
    );

    let kick = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::String("kick".to_string()),
        method: "players/kick".to_string(),
        params: JsonRpcParams::Kick(vec![
            KickDto {
                player: PlayerDto {
                    id: Some(steve.uuid.clone()),
                    name: None,
                    latency: 0,
                    game_mode: GameMode::Survival,
                },
                message: None,
            },
            KickDto {
                player: PlayerDto {
                    id: None,
                    name: Some("Alex".to_string()),
                    latency: 0,
                    game_mode: GameMode::Survival,
                },
                message: Some("Go away".to_string()),
            },
        ]),
    });

    assert_eq!(
        kick.result,
        Ok(JsonRpcResult::Players(vec![
            PlayerDto::from_profile(&steve),
            PlayerDto::from_profile(&alex)
        ]))
    );
    assert!(state.online_players.is_empty());
    assert_eq!(state.disconnected_players[0].message, DEFAULT_KICK_MESSAGE);
    assert_eq!(state.disconnected_players[1].message, "Go away");
    assert_eq!(
        state.notifications,
        vec![QueuedNotification {
            client_id: "admin".to_string(),
            method: "players/left".to_string(),
        }]
    );
}

#[test]
fn management_player_lists_ip_bans_and_gamerules_mutate_state_and_notify() {
    let steve = player("Steve");
    let alex = player("Alex");
    let mut state = ManagementServerState::default();
    state.connect_client("admin");

    let add_ops = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(1),
        method: "operators/add".to_string(),
        params: JsonRpcParams::Players(vec![PlayerDto::from_profile(&steve)]),
    });
    assert_eq!(add_ops.result, Ok(JsonRpcResult::Null));
    assert_eq!(state.operators, vec![steve.clone()]);
    assert_eq!(state.notifications[0].method, "operators/added");

    let remove_ops = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(2),
        method: "operators/remove".to_string(),
        params: JsonRpcParams::Players(vec![PlayerDto::from_profile(&steve)]),
    });
    assert_eq!(remove_ops.result, Ok(JsonRpcResult::Null));
    assert!(state.operators.is_empty());
    assert_eq!(state.notifications[1].method, "operators/removed");

    let add_allowlist = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(3),
        method: "allowlist/add".to_string(),
        params: JsonRpcParams::Players(vec![PlayerDto::from_profile(&alex)]),
    });
    assert_eq!(add_allowlist.result, Ok(JsonRpcResult::Null));
    assert_eq!(state.allowlist, vec![alex.clone()]);
    assert_eq!(state.notifications[2].method, "allowlist/added");

    let add_ban = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(4),
        method: "bans/add".to_string(),
        params: JsonRpcParams::Players(vec![PlayerDto::from_profile(&alex)]),
    });
    assert_eq!(add_ban.result, Ok(JsonRpcResult::Null));
    assert_eq!(state.banned_players, vec![alex.clone()]);
    assert_eq!(state.notifications[3].method, "bans/added");

    let ip_ban = IpBanDto {
        ip: "203.0.113.9".to_string(),
        reason: Some("test".to_string()),
        expires: None,
    };
    let add_ip_ban = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(5),
        method: "ip_bans/add".to_string(),
        params: JsonRpcParams::IpBans(vec![ip_ban.clone()]),
    });
    assert_eq!(add_ip_ban.result, Ok(JsonRpcResult::Null));
    assert_eq!(state.ip_bans, vec![ip_ban.clone()]);
    assert_eq!(state.notifications[4].method, "ip_bans/added");

    let remove_ip_ban = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(6),
        method: "ip_bans/remove".to_string(),
        params: JsonRpcParams::IpBans(vec![IpBanDto {
            ip: ip_ban.ip,
            reason: None,
            expires: None,
        }]),
    });
    assert_eq!(remove_ip_ban.result, Ok(JsonRpcResult::Null));
    assert!(state.ip_bans.is_empty());
    assert_eq!(state.notifications[5].method, "ip_bans/removed");

    let update_rules = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(7),
        method: "gamerules/update".to_string(),
        params: JsonRpcParams::GameRules(vec![GameRuleDto {
            key: "doDaylightCycle".to_string(),
            value: "false".to_string(),
        }]),
    });
    assert_eq!(update_rules.result, Ok(JsonRpcResult::Null));
    assert_eq!(
        state.game_rules,
        vec![GameRuleDto {
            key: "doDaylightCycle".to_string(),
            value: "false".to_string(),
        }]
    );
    assert_eq!(state.notifications[6].method, "gamerules/updated");

    let get_rules = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(8),
        method: "gamerules/get".to_string(),
        params: JsonRpcParams::None,
    });
    assert_eq!(
        get_rules.result,
        Ok(JsonRpcResult::GameRules(vec![GameRuleDto {
            key: "doDaylightCycle".to_string(),
            value: "false".to_string(),
        }]))
    );
}

#[test]
fn management_settings_state_and_metrics_return_typed_snapshots() {
    let steve = player("Steve");
    let mut state = ManagementServerState {
        online_players: vec![steve],
        settings: ServerSettingsDto {
            online_mode: false,
            max_players: 42,
            motd: "VibeCraft".to_string(),
        },
        metrics: ServerMetricsDto {
            tick: 99,
            tick_duration_nanos: 40_000_000,
            over_budget_nanos: 0,
            bytes_in: 123,
            bytes_out: 456,
            packets_in: 7,
            packets_out: 8,
        },
        ..Default::default()
    };

    let settings = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::String("settings".to_string()),
        method: "server/settings".to_string(),
        params: JsonRpcParams::None,
    });
    assert_eq!(
        settings.result,
        Ok(JsonRpcResult::ServerSettings(ServerSettingsDto {
            online_mode: false,
            max_players: 42,
            motd: "VibeCraft".to_string(),
        }))
    );

    let status = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::String("state".to_string()),
        method: "server/state".to_string(),
        params: JsonRpcParams::None,
    });
    assert_eq!(
        status.result,
        Ok(JsonRpcResult::ServerStatus(ServerStatusDto {
            running: true,
            player_count: 1,
        }))
    );

    let metrics = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::String("metrics".to_string()),
        method: "server/metrics".to_string(),
        params: JsonRpcParams::None,
    });
    assert_eq!(
        metrics.result,
        Ok(JsonRpcResult::ServerMetrics(ServerMetricsDto {
            tick: 99,
            tick_duration_nanos: 40_000_000,
            over_budget_nanos: 0,
            bytes_in: 123,
            bytes_out: 456,
            packets_in: 7,
            packets_out: 8,
        }))
    );
}

#[test]
fn request_tracking_correlates_responses_and_cleans_up_on_disconnect() {
    let steve = player("Steve");
    let mut state = ManagementServerState {
        online_players: vec![steve],
        ..Default::default()
    };
    state.connect_client("admin");

    let get = JsonRpcRequest {
        id: JsonRpcId::String("req-1".to_string()),
        method: "players/get".to_string(),
        params: JsonRpcParams::None,
    };
    let response = state.handle_client_request("admin", get);
    assert_eq!(response.id, JsonRpcId::String("req-1".to_string()));
    assert_eq!(state.pending_request_count(), 0);

    let slow = JsonRpcRequest {
        id: JsonRpcId::Number(2),
        method: "server/metrics".to_string(),
        params: JsonRpcParams::None,
    };
    state.begin_request("admin", &slow);
    assert_eq!(state.pending_request_count(), 1);
    state.disconnect_client("admin");
    assert_eq!(state.pending_request_count(), 0);
}

#[test]
fn json_rpc_connection_protocol_flow_matches_java_contract() {
    assert_connection_java_source_shape();

    let mut connection = JsonRpcConnectionModel::new(42);
    connection.channel_active("/127.0.0.1:25585");
    connection.channel_inactive("/127.0.0.1:25585");
    assert_eq!(
        connection.events,
        vec![
            JsonRpcConnectionEvent::Connected {
                client_info: 42,
                remote_address: "/127.0.0.1:25585".to_string(),
            },
            JsonRpcConnectionEvent::Disconnected {
                client_info: 42,
                remote_address: "/127.0.0.1:25585".to_string(),
            },
        ]
    );

    connection.send_notification("server/status", &["{\"running\":true}"]);
    let id = connection.send_request("notification/server/status", &[], 1_000);
    assert_eq!(id, 1);
    assert_eq!(
        connection.writes,
        vec![
            JsonRpcConnectionWrite::Single(
                "{\"jsonrpc\":\"2.0\",\"method\":\"server/status\",\"params\":[{\"running\":true}]}".to_string()
            ),
            JsonRpcConnectionWrite::Single(
                "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"notification/server/status\"}"
                    .to_string()
            ),
        ]
    );
    assert_eq!(connection.pending_requests[&1].timeout_time, 6_000);

    connection.read_json(&serde_json::json!({"id": 1, "result": {"ok": true}}));
    assert!(!connection.pending_requests.contains_key(&1));
    assert_eq!(
        connection.completed_requests.get(&1),
        Some(&Ok("{\"ok\":true}".to_string()))
    );

    let error_id = connection.send_request("notification/server/status", &[], 2_000);
    connection.read_json(&serde_json::json!({
        "id": error_id,
        "error": {"code": -32000, "message": "remote failure"}
    }));
    assert!(!connection.pending_requests.contains_key(&error_id));
    assert!(connection.completed_requests[&error_id]
        .as_ref()
        .is_err_and(|message| message.contains("Remote RPC error")));

    let timeout_id = connection.send_request("notification/server/status", &[], 5);
    connection.tick(5_005);
    assert!(connection.pending_requests.contains_key(&timeout_id));
    connection.tick(5_006);
    assert!(connection.completed_requests[&timeout_id]
        .as_ref()
        .is_err_and(|message| {
            message == "RPC method notification/server/status timed out waiting for response"
        }));
}

#[test]
fn json_rpc_connection_handles_incoming_objects_and_batches_like_java() {
    let mut connection = JsonRpcConnectionModel::new(7);

    let invalid_id_json = serde_json::json!({"id": {}, "method": "players", "params": []});
    let invalid_id = connection.handle_json_object(json_object(&invalid_id_json));
    assert_eq!(
        invalid_id,
        Some("{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32600,\"message\":\"Invalid Request\",\"data\":\"Invalid request id - only String, Number and NULL supported\"}}".to_string())
    );

    let notification_json = serde_json::json!({"method": "players", "params": []});
    let notification = connection.handle_json_object(json_object(&notification_json));
    assert_eq!(notification, None);

    let request_json = serde_json::json!({"id": "req", "method": "players", "params": []});
    let request = connection.handle_json_object(json_object(&request_json));
    assert_eq!(
        request,
        Some("{\"jsonrpc\":\"2.0\",\"id\":\"req\",\"result\":null}".to_string())
    );

    let bad_method_json = serde_json::json!({"id": 2, "method": "bad method"});
    let bad_method = connection.handle_json_object(json_object(&bad_method_json));
    assert_eq!(
        bad_method,
        Some("{\"jsonrpc\":\"2.0\",\"id\":2,\"error\":{\"code\":-32600,\"message\":\"Invalid Request\",\"data\":\"Failed to parse method value: bad method\"}}".to_string())
    );

    let missing_json = serde_json::json!({"id": 3, "method": "missing/method"});
    let missing = connection.handle_json_object(json_object(&missing_json));
    assert_eq!(
        missing,
        Some("{\"jsonrpc\":\"2.0\",\"id\":3,\"error\":{\"code\":-32601,\"message\":\"Method not found\",\"data\":\"Method not found: missing/method\"}}".to_string())
    );

    connection.read_json(&serde_json::json!([
        {"id": 4, "method": "players", "params": []},
        {"method": "players", "params": []}
    ]));
    assert_eq!(
        connection.writes.last(),
        Some(&JsonRpcConnectionWrite::Batch(vec![
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"result\":null}".to_string()
        ]))
    );

    connection.read_json(&serde_json::json!(true));
    assert_eq!(
        connection.writes.last(),
        Some(&JsonRpcConnectionWrite::Single(
            "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32600,\"message\":\"Invalid Request\"}}".to_string()
        ))
    );

    connection.exception_caught("bad json", true);
    assert_eq!(
        connection.writes.last(),
        Some(&JsonRpcConnectionWrite::Single(
            "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32700,\"message\":\"Parse error\",\"data\":\"bad json\"}}".to_string()
        ))
    );
    connection.exception_caught("closed", false);
    assert!(connection.events.contains(&JsonRpcConnectionEvent::Closed));
}

fn assert_connection_java_source_shape() {
    for sentinel in [
        "private static final AtomicInteger CONNECTION_ID_COUNTER = new AtomicInteger(0);",
        "private final AtomicInteger transactionId = new AtomicInteger();",
        "private final Int2ObjectMap<PendingRpcRequest<?>> pendingRequests",
        "new ReadTimeoutException(",
        "\" timed out waiting for response\"",
        "this.managementServer.onConnected(this);",
        "this.managementServer.onDisconnected(this);",
        "cause.getCause() instanceof JsonParseException",
        "JsonRPCErrors.PARSE_ERROR.createWithUnknownId(cause.getMessage())",
        "this.channel.close().awaitUninterruptibly();",
        "jsonElement.isJsonObject()",
        "jsonElement.isJsonArray()",
        "this.handleBatchRequest(jsonElement.getAsJsonArray().asList())",
        "JsonRPCErrors.INVALID_REQUEST.createWithUnknownId(null)",
        "batchRequests.stream().map(batchEntry -> this.handleJsonObject(batchEntry.getAsJsonObject())).filter(Objects::nonNull).forEach(batchResponses::add);",
        "this.sendRequest(method, null, false);",
        "this.sendRequest(method, params, false);",
        "this.sendRequest(method, null, true);",
        "this.sendRequest(method, params, true);",
        "List<JsonElement> jsonParams = params != null ? List.of(Objects.requireNonNull(method.value().encodeParams(params))) : List.of();",
        "int id = this.transactionId.incrementAndGet();",
        "time + 5000L",
        "JsonRPCUtils.createRequest(id, method.key().identifier(), jsonParams)",
        "JsonRPCUtils.createRequest(null, method.key().identifier(), jsonParams)",
    ] {
        assert!(
            CONNECTION_JAVA.contains(sentinel),
            "Connection.java missing sentinel: {sentinel}"
        );
    }
    assert_connection_java_handle_object_shape();
}

fn json_object(value: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
    if let Some(object) = value.as_object() {
        object
    } else {
        panic!("JSON fixture should be an object")
    }
}

fn assert_connection_java_handle_object_shape() {
    for sentinel in [
        "String method = JsonRPCUtils.getMethodName(jsonObject);",
        "JsonElement result = JsonRPCUtils.getResult(jsonObject);",
        "JsonObject error = JsonRPCUtils.getError(jsonObject);",
        "method != null && result == null && error == null",
        "Invalid request id - only String, Number and NULL supported",
        "method == null && result != null && error == null && id != null",
        "this.handleRequestResponse(id.getAsInt(), result);",
        "Received respose {} with id {} we did not request",
        "method == null && result == null && error != null",
        "JsonRPCErrors.INVALID_REQUEST.createWithoutData(Objects.requireNonNullElse(id, JsonNull.INSTANCE))",
        "Identifier identifier = Identifier.tryParse(method);",
        "throw new InvalidRequestJsonRpcException(\"Failed to parse method value: \" + method);",
        "throw new MethodNotFoundJsonRpcException(\"Method not found: \" + method);",
        "incomingRpcMethod.get().attributes().runOnMainThread()",
        "this.minecraftApi.<JsonElement>submit(() -> incomingRpcMethod.get().apply(this.minecraftApi, params, this.clientInfo)).join();",
        "this.pendingRequests.remove(id)",
        "request.accept(result);",
        "request.resultFuture().completeExceptionally(new RemoteRpcErrorException(id, error));",
        "LOGGER.error(\"Received error (id: {}): {}\", id, error);",
    ] {
        assert!(
            CONNECTION_JAVA.contains(sentinel),
            "Connection.java missing handle-object sentinel: {sentinel}"
        );
    }
}

#[test]
fn pending_rpc_request_accept_and_timeout_match_java_record() {
    for sentinel in [
        "public record PendingRpcRequest<Result>(",
        "Holder.Reference<? extends OutgoingRpcMethod<?, ? extends Result>> method",
        "CompletableFuture<Result> resultFuture",
        "long timeoutTime",
        "public void accept(final JsonElement response)",
        "Result result = (Result)this.method.value().decodeResult(response);",
        "this.resultFuture.complete(Objects.requireNonNull(result));",
        "this.resultFuture.completeExceptionally(e);",
        "return currentTime > this.timeoutTime;",
    ] {
        assert!(
            PENDING_RPC_REQUEST_JAVA.contains(sentinel),
            "PendingRpcRequest.java missing sentinel: {sentinel}"
        );
    }

    let response = serde_json::json!({"ok": true});
    let mut pending = PendingRpcRequest::new("notification/server/status", 100);
    assert!(!pending.timed_out(100));
    assert!(pending.timed_out(101));
    pending.accept(&response, |value| {
        Ok(Some(value["ok"].as_bool().unwrap().to_string()))
    });
    assert_eq!(pending.result, Some(Ok("true".to_string())));

    let mut decode_error = PendingRpcRequest::new("notification/server/status", 5);
    decode_error.accept(&response, |_| Err("decode failed".to_string()));
    assert_eq!(
        decode_error.result,
        Some(Err("decode failed".to_string()))
    );

    let mut null_result = PendingRpcRequest::new("notification/server/status", 5);
    null_result.accept(&response, |_| Ok(None));
    assert_eq!(
        null_result.result,
        Some(Err("decoded result was null".to_string()))
    );
}

#[test]
fn shutdown_rejects_in_flight_requests_and_announces_stopping() {
    let mut state = ManagementServerState::default();
    state.connect_client("first");
    state.connect_client("second");
    state.begin_request(
        "first",
        &JsonRpcRequest {
            id: JsonRpcId::Number(11),
            method: "players/get".to_string(),
            params: JsonRpcParams::None,
        },
    );
    state.begin_request(
        "second",
        &JsonRpcRequest {
            id: JsonRpcId::String("slow".to_string()),
            method: "players/kick".to_string(),
            params: JsonRpcParams::None,
        },
    );

    let rejected = state.shutdown();
    assert_eq!(
        rejected,
        vec![
            JsonRpcResponse {
                id: JsonRpcId::Number(11),
                result: Err(JsonRpcError::INTERNAL_ERROR),
            },
            JsonRpcResponse {
                id: JsonRpcId::String("slow".to_string()),
                result: Err(JsonRpcError::INTERNAL_ERROR),
            },
        ]
    );
    assert_eq!(state.pending_request_count(), 0);
    assert_eq!(
        state.notifications,
        vec![
            QueuedNotification {
                client_id: "first".to_string(),
                method: "server/stopping".to_string(),
            },
            QueuedNotification {
                client_id: "second".to_string(),
                method: "server/stopping".to_string(),
            },
        ]
    );
}

#[test]
fn json_rpc_errors_and_envelopes_use_standard_codes() {
    for sentinel in [
        "PARSE_ERROR(-32700, \"Parse error\")",
        "INVALID_REQUEST(-32600, \"Invalid Request\")",
        "METHOD_NOT_FOUND(-32601, \"Method not found\")",
        "INVALID_PARAMS(-32602, \"Invalid params\")",
        "INTERNAL_ERROR(-32603, \"Internal error\")",
        "public JsonObject createWithUnknownId(final @Nullable String data)",
        "return JsonRPCUtils.createError(JsonNull.INSTANCE, this.message, this.errorCode, data);",
        "public JsonObject createWithoutData(final JsonElement id)",
        "return JsonRPCUtils.createError(id, this.message, this.errorCode, null);",
        "public JsonObject create(final JsonElement id, final String data)",
        "return JsonRPCUtils.createError(id, this.message, this.errorCode, data);",
    ] {
        assert!(
            JSON_RPC_ERRORS_JAVA.contains(sentinel),
            "JsonRPCErrors.java missing sentinel: {sentinel}"
        );
    }
    for sentinel in [
        "public static JsonObject createError(final JsonElement id, final String message, final int errorCode, final @Nullable String data)",
        "errorResponse.addProperty(\"jsonrpc\", \"2.0\");",
        "errorResponse.add(\"id\", id);",
        "error.addProperty(\"code\", errorCode);",
        "error.addProperty(\"message\", message);",
        "if (data != null && !data.isBlank())",
        "error.addProperty(\"data\", data);",
        "errorResponse.add(\"error\", error);",
    ] {
        assert!(
            JSON_RPC_UTILS_JAVA.contains(sentinel),
            "JsonRPCUtils.java missing error sentinel: {sentinel}"
        );
    }

    assert_eq!(JsonRpcError::PARSE_ERROR.code, -32700);
    assert_eq!(JsonRpcError::INVALID_REQUEST.message, "Invalid Request");
    assert_eq!(JsonRpcError::METHOD_NOT_FOUND.code, -32601);
    assert_eq!(JsonRpcError::INVALID_PARAMS.code, -32602);
    assert_eq!(JsonRpcError::INTERNAL_ERROR.message, "Internal error");

    let mut state = ManagementServerState::default();
    let unknown = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Number(99),
        method: "missing/method".to_string(),
        params: JsonRpcParams::None,
    });
    assert_eq!(unknown.result, Err(JsonRpcError::METHOD_NOT_FOUND));
    assert_eq!(
        unknown.to_json(),
        "{\"jsonrpc\":\"2.0\",\"id\":99,\"error\":{\"code\":-32601,\"message\":\"Method not found\"}}"
    );

    let invalid_params = state.handle_request(JsonRpcRequest {
        id: JsonRpcId::Null,
        method: "players/kick".to_string(),
        params: JsonRpcParams::None,
    });
    assert_eq!(invalid_params.result, Err(JsonRpcError::INVALID_PARAMS));

    assert_eq!(
        JsonRpcError::PARSE_ERROR
            .create_with_unknown_id(Some("bad json"))
            .to_json(),
        "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32700,\"message\":\"Parse error\",\"data\":\"bad json\"}}"
    );
    assert_eq!(
        JsonRpcError::INVALID_REQUEST
            .create_with_unknown_id(Some("   "))
            .to_json(),
        "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32600,\"message\":\"Invalid Request\"}}"
    );
    assert_eq!(
        JsonRpcError::INTERNAL_ERROR
            .create_without_data(JsonRpcId::String("abc".to_string()))
            .to_json(),
        "{\"jsonrpc\":\"2.0\",\"id\":\"abc\",\"error\":{\"code\":-32603,\"message\":\"Internal error\"}}"
    );
    assert_eq!(
        JsonRpcError::INVALID_PARAMS
            .create(JsonRpcId::Number(7), "expected object")
            .to_json(),
        "{\"jsonrpc\":\"2.0\",\"id\":7,\"error\":{\"code\":-32602,\"message\":\"Invalid params\",\"data\":\"expected object\"}}"
    );
}

#[test]
fn json_rpc_utils_match_java_envelope_helpers_and_accessors() {
    for sentinel in [
        "public static final String JSON_RPC_VERSION = \"2.0\";",
        "public static final String OPEN_RPC_VERSION = \"1.3.2\";",
        "public static JsonObject createSuccessResult(final JsonElement id, final JsonElement result)",
        "response.addProperty(\"jsonrpc\", \"2.0\");",
        "response.add(\"id\", id);",
        "response.add(\"result\", result);",
        "public static JsonObject createRequest(final @Nullable Integer id, final Identifier method, final List<JsonElement> params)",
        "if (id != null)",
        "request.addProperty(\"method\", method.toString());",
        "if (!params.isEmpty())",
        "request.add(\"params\", jsonArray);",
        "public static @Nullable JsonElement getRequestId(final JsonObject jsonObject)",
        "return jsonObject.get(\"id\");",
        "public static @Nullable String getMethodName(final JsonObject jsonObject)",
        "return GsonHelper.getAsString(jsonObject, \"method\", null);",
        "public static @Nullable JsonElement getParams(final JsonObject jsonObject)",
        "return jsonObject.get(\"params\");",
        "public static @Nullable JsonElement getResult(final JsonObject jsonObject)",
        "return jsonObject.get(\"result\");",
        "public static @Nullable JsonObject getError(final JsonObject jsonObject)",
        "return GsonHelper.getAsJsonObject(jsonObject, \"error\", null);",
    ] {
        assert!(
            JSON_RPC_UTILS_JAVA.contains(sentinel),
            "JsonRPCUtils.java missing sentinel: {sentinel}"
        );
    }

    assert_eq!(JSON_RPC_VERSION, "2.0");
    assert_eq!(OPEN_RPC_VERSION, "1.3.2");
    assert_eq!(
        JsonRpcUtils::create_success_result(JsonRpcId::Number(4), "{\"ok\":true}"),
        "{\"jsonrpc\":\"2.0\",\"id\":4,\"result\":{\"ok\":true}}"
    );
    assert_eq!(
        JsonRpcUtils::create_request(None, "server/status", &[]),
        "{\"jsonrpc\":\"2.0\",\"method\":\"server/status\"}"
    );
    assert_eq!(
        JsonRpcUtils::create_request(Some(8), "players/kick", &["{\"name\":\"Steve\"}", "true"]),
        "{\"jsonrpc\":\"2.0\",\"id\":8,\"method\":\"players/kick\",\"params\":[{\"name\":\"Steve\"},true]}"
    );
    assert_eq!(
        JsonRpcUtils::create_error(JsonRpcId::Null, "Invalid Request", -32600, Some("")),
        "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32600,\"message\":\"Invalid Request\"}}"
    );
    assert_eq!(
        JsonRpcUtils::create_error(
            JsonRpcId::String("req".to_string()),
            "Invalid params",
            -32602,
            Some("missing player")
        ),
        "{\"jsonrpc\":\"2.0\",\"id\":\"req\",\"error\":{\"code\":-32602,\"message\":\"Invalid params\",\"data\":\"missing player\"}}"
    );

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "players/get",
        "params": [{"name": "Steve"}]
    });
    assert_eq!(
        JsonRpcUtils::get_request_id(&request),
        request.get("id")
    );
    assert_eq!(JsonRpcUtils::get_method_name(&request), Some("players/get"));
    assert_eq!(JsonRpcUtils::get_params(&request), request.get("params"));
    assert_eq!(JsonRpcUtils::get_result(&request), None);
    assert_eq!(JsonRpcUtils::get_error(&request), None);

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 9,
        "result": {"running": true},
        "error": {"code": -32603}
    });
    assert_eq!(JsonRpcUtils::get_result(&response), response.get("result"));
    assert!(JsonRpcUtils::get_error(&response)
        .is_some_and(|error| error.contains_key("code")));
    assert_eq!(JsonRpcUtils::get_method_name(&response), None);
}

mod dispatch_and_notifications;
