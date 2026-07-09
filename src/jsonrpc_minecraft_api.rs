#![allow(dead_code)]

/// Identity used by the internal JSON-RPC models to prove that every service
/// created by `MinecraftApi::of` is bound to the same dedicated server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DedicatedServerIdentity(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationManagerModel {
    pub server: DedicatedServerIdentity,
}

macro_rules! internal_service_model {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name {
            pub server: DedicatedServerIdentity,
            pub logger_id: u64,
        }

        impl $name {
            const fn new(server: DedicatedServerIdentity, logger_id: u64) -> Self {
                Self { server, logger_id }
            }
        }
    };
}

internal_service_model!(MinecraftAllowListServiceModel);
internal_service_model!(MinecraftBanListServiceModel);
internal_service_model!(MinecraftPlayerListServiceModel);
internal_service_model!(MinecraftGameRuleServiceModel);
internal_service_model!(MinecraftOperatorListServiceModel);
internal_service_model!(MinecraftServerSettingsServiceModel);
internal_service_model!(MinecraftServerStateServiceModel);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinecraftSubmittedTask {
    Supplier,
    Runnable,
}

/// Deterministic model of the completed value exposed by Java's
/// `CompletableFuture`. The executor audit owns scheduling behavior; this type
/// preserves the API contract that both submit overloads return their result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftCompletedFuture<V> {
    value: V,
}

impl<V> MinecraftCompletedFuture<V> {
    pub fn into_inner(self) -> V {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftExecutorServiceModel {
    pub server: DedicatedServerIdentity,
    pub submitted_tasks: Vec<MinecraftSubmittedTask>,
}

impl MinecraftExecutorServiceModel {
    pub const fn new(server: DedicatedServerIdentity) -> Self {
        Self {
            server,
            submitted_tasks: Vec::new(),
        }
    }

    fn submit_supplier<V>(
        &mut self,
        supplier: impl FnOnce() -> V,
    ) -> MinecraftCompletedFuture<V> {
        self.submitted_tasks.push(MinecraftSubmittedTask::Supplier);
        MinecraftCompletedFuture { value: supplier() }
    }

    fn submit_runnable(
        &mut self,
        runnable: impl FnOnce(),
    ) -> MinecraftCompletedFuture<()> {
        self.submitted_tasks.push(MinecraftSubmittedTask::Runnable);
        runnable();
        MinecraftCompletedFuture { value: () }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftApiModel {
    notification_manager: NotificationManagerModel,
    allow_list_service: MinecraftAllowListServiceModel,
    ban_list_service: MinecraftBanListServiceModel,
    player_list_service: MinecraftPlayerListServiceModel,
    game_rule_service: MinecraftGameRuleServiceModel,
    operator_list_service: MinecraftOperatorListServiceModel,
    server_settings_service: MinecraftServerSettingsServiceModel,
    server_state_service: MinecraftServerStateServiceModel,
    executor_service: MinecraftExecutorServiceModel,
}

impl MinecraftApiModel {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        notification_manager: NotificationManagerModel,
        allow_list_service: MinecraftAllowListServiceModel,
        ban_list_service: MinecraftBanListServiceModel,
        player_list_service: MinecraftPlayerListServiceModel,
        game_rule_service: MinecraftGameRuleServiceModel,
        operator_list_service: MinecraftOperatorListServiceModel,
        server_settings_service: MinecraftServerSettingsServiceModel,
        server_state_service: MinecraftServerStateServiceModel,
        executor_service: MinecraftExecutorServiceModel,
    ) -> Self {
        Self {
            notification_manager,
            allow_list_service,
            ban_list_service,
            player_list_service,
            game_rule_service,
            operator_list_service,
            server_settings_service,
            server_state_service,
            executor_service,
        }
    }

    pub fn submit<V>(&mut self, supplier: impl FnOnce() -> V) -> MinecraftCompletedFuture<V> {
        self.executor_service.submit_supplier(supplier)
    }

    pub fn submit_runnable(
        &mut self,
        runnable: impl FnOnce(),
    ) -> MinecraftCompletedFuture<()> {
        self.executor_service.submit_runnable(runnable)
    }

    pub const fn allow_list_service(&self) -> &MinecraftAllowListServiceModel {
        &self.allow_list_service
    }

    pub const fn ban_list_service(&self) -> &MinecraftBanListServiceModel {
        &self.ban_list_service
    }

    pub const fn player_list_service(&self) -> &MinecraftPlayerListServiceModel {
        &self.player_list_service
    }

    pub const fn game_rule_service(&self) -> &MinecraftGameRuleServiceModel {
        &self.game_rule_service
    }

    pub const fn operator_list_service(&self) -> &MinecraftOperatorListServiceModel {
        &self.operator_list_service
    }

    pub const fn server_settings_service(&self) -> &MinecraftServerSettingsServiceModel {
        &self.server_settings_service
    }

    pub const fn server_state_service(&self) -> &MinecraftServerStateServiceModel {
        &self.server_state_service
    }

    pub const fn notification_manager(&self) -> &NotificationManagerModel {
        &self.notification_manager
    }

    pub fn executor_service(&self) -> &MinecraftExecutorServiceModel {
        &self.executor_service
    }

    pub fn of(server: DedicatedServerIdentity) -> Self {
        // Java constructs one logger and shares it across all seven logged
        // services. Its identity is observable here so factory wiring cannot
        // silently drift to per-service loggers.
        const JSON_RPC_LOGGER_ID: u64 = 1;
        Self::new(
            NotificationManagerModel { server },
            MinecraftAllowListServiceModel::new(server, JSON_RPC_LOGGER_ID),
            MinecraftBanListServiceModel::new(server, JSON_RPC_LOGGER_ID),
            MinecraftPlayerListServiceModel::new(server, JSON_RPC_LOGGER_ID),
            MinecraftGameRuleServiceModel::new(server, JSON_RPC_LOGGER_ID),
            MinecraftOperatorListServiceModel::new(server, JSON_RPC_LOGGER_ID),
            MinecraftServerSettingsServiceModel::new(server, JSON_RPC_LOGGER_ID),
            MinecraftServerStateServiceModel::new(server, JSON_RPC_LOGGER_ID),
            MinecraftExecutorServiceModel::new(server),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn constructor_accessors_and_submit_overloads_delegate_to_owned_dependencies() {
        let server = DedicatedServerIdentity(42);
        let mut api = MinecraftApiModel::new(
            NotificationManagerModel { server },
            MinecraftAllowListServiceModel::new(server, 7),
            MinecraftBanListServiceModel::new(server, 7),
            MinecraftPlayerListServiceModel::new(server, 7),
            MinecraftGameRuleServiceModel::new(server, 7),
            MinecraftOperatorListServiceModel::new(server, 7),
            MinecraftServerSettingsServiceModel::new(server, 7),
            MinecraftServerStateServiceModel::new(server, 7),
            MinecraftExecutorServiceModel::new(server),
        );

        assert_eq!(api.notification_manager().server, server);
        assert_eq!(api.allow_list_service().logger_id, 7);
        assert_eq!(api.ban_list_service().logger_id, 7);
        assert_eq!(api.player_list_service().logger_id, 7);
        assert_eq!(api.game_rule_service().logger_id, 7);
        assert_eq!(api.operator_list_service().logger_id, 7);
        assert_eq!(api.server_settings_service().logger_id, 7);
        assert_eq!(api.server_state_service().logger_id, 7);

        assert_eq!(api.submit(|| 26).into_inner(), 26);
        let ran = Cell::new(false);
        api.submit_runnable(|| ran.set(true)).into_inner();
        assert!(ran.get());
        assert_eq!(
            api.executor_service().submitted_tasks,
            vec![
                MinecraftSubmittedTask::Supplier,
                MinecraftSubmittedTask::Runnable
            ]
        );
    }

    #[test]
    fn factory_binds_every_dependency_to_one_server_and_one_logger() {
        let server = DedicatedServerIdentity(99);
        let api = MinecraftApiModel::of(server);
        let bindings = [
            *api.allow_list_service(),
            MinecraftAllowListServiceModel::new(
                api.ban_list_service().server,
                api.ban_list_service().logger_id,
            ),
            MinecraftAllowListServiceModel::new(
                api.player_list_service().server,
                api.player_list_service().logger_id,
            ),
            MinecraftAllowListServiceModel::new(
                api.game_rule_service().server,
                api.game_rule_service().logger_id,
            ),
            MinecraftAllowListServiceModel::new(
                api.operator_list_service().server,
                api.operator_list_service().logger_id,
            ),
            MinecraftAllowListServiceModel::new(
                api.server_settings_service().server,
                api.server_settings_service().logger_id,
            ),
            MinecraftAllowListServiceModel::new(
                api.server_state_service().server,
                api.server_state_service().logger_id,
            ),
        ];
        assert!(bindings.iter().all(|binding| binding.server == server));
        assert!(bindings
            .iter()
            .all(|binding| binding.logger_id == bindings[0].logger_id));
        assert_eq!(api.notification_manager().server, server);
        assert_eq!(api.executor_service().server, server);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn minecraft_api_source_matches_java_26_1_2() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/internalapi/MinecraftApi.java"
        );
        for sentinel in [
            "private final NotificationManager notificationManager;",
            "private final MinecraftAllowListService allowListService;",
            "private final MinecraftBanListService banListService;",
            "private final MinecraftPlayerListService minecraftPlayerListService;",
            "private final MinecraftGameRuleService gameRuleService;",
            "private final MinecraftOperatorListService minecraftOperatorListService;",
            "private final MinecraftServerSettingsService minecraftServerSettingsService;",
            "private final MinecraftServerStateService minecraftServerStateService;",
            "private final MinecraftExecutorService executorService;",
            "return this.executorService.submit(supplier);",
            "return this.executorService.submit(runnable);",
            "public MinecraftAllowListService allowListService()",
            "public MinecraftBanListService banListService()",
            "public MinecraftPlayerListService playerListService()",
            "public MinecraftGameRuleService gameRuleService()",
            "public MinecraftOperatorListService operatorListService()",
            "public MinecraftServerSettingsService serverSettingsService()",
            "public MinecraftServerStateService serverStateService()",
            "public NotificationManager notificationManager()",
            "JsonRpcLogger jsonrpcLogger = new JsonRpcLogger();",
            "MinecraftAllowListServiceImpl allowListService = new MinecraftAllowListServiceImpl(server, jsonrpcLogger);",
            "MinecraftBanListServiceImpl banListService = new MinecraftBanListServiceImpl(server, jsonrpcLogger);",
            "MinecraftPlayerListServiceImpl playerListService = new MinecraftPlayerListServiceImpl(server, jsonrpcLogger);",
            "MinecraftGameRuleServiceImpl gameRuleService = new MinecraftGameRuleServiceImpl(server, jsonrpcLogger);",
            "MinecraftOperatorListServiceImpl operatorListService = new MinecraftOperatorListServiceImpl(server, jsonrpcLogger);",
            "MinecraftServerSettingsServiceImpl serverSettingsService = new MinecraftServerSettingsServiceImpl(server, jsonrpcLogger);",
            "MinecraftServerStateServiceImpl serverStateService = new MinecraftServerStateServiceImpl(server, jsonrpcLogger);",
            "MinecraftExecutorService executorService = new MinecraftExecutorServiceImpl(server);",
            "server.notificationManager(),",
        ] {
            assert!(SOURCE.contains(sentinel), "MinecraftApi.java missing: {sentinel}");
        }
    }
}
