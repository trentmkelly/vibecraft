use super::*;

const VALID_SECRET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCD";

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
fn origins_parse_exact_wildcard_and_empty_policy() {
    let exact = AllowedOrigins::parse("https://a.example, https://b.example");
    assert!(exact.accepts(None));
    assert!(exact.accepts(Some("https://b.example")));
    assert!(!exact.accepts(Some("https://c.example")));
    assert!(AllowedOrigins::parse("*").accepts(Some("https://anything.example")));
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
            motd: "RustCraft".to_string(),
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
            motd: "RustCraft".to_string(),
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
}

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
