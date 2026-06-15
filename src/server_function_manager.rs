#![allow(dead_code)]

use crate::command::CommandFunctionDefinition;
use crate::registry::Identifier;
use crate::server_function_library::ServerFunctionLibrary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLoopSenderModel {
    pub permission: FunctionPermissionLevel,
    pub suppressed_output: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionPermissionLevel {
    Gamemaster,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerFunctionManagerEvent {
    ProfilerPush(String),
    ExecuteFunction {
        id: Identifier,
        sender: GameLoopSenderModel,
    },
    ProfilerPop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerFunctionManager {
    library: ServerFunctionLibrary,
    ticking: Vec<CommandFunctionDefinition>,
    post_reload: bool,
    events: Vec<ServerFunctionManagerEvent>,
}

impl ServerFunctionManager {
    pub fn new(library: ServerFunctionLibrary) -> Self {
        let mut manager = Self {
            library: ServerFunctionLibrary::default(),
            ticking: Vec::new(),
            post_reload: false,
            events: Vec::new(),
        };
        manager.post_reload(library);
        manager
    }

    pub fn tick(&mut self, runs_normally: bool) {
        if !runs_normally {
            return;
        }

        if self.post_reload {
            self.post_reload = false;
            let functions = self
                .library
                .get_tag(&Self::load_function_tag())
                .to_vec();
            self.execute_tag_functions(&functions, &Self::load_function_tag());
        }

        let ticking = self.ticking.clone();
        self.execute_tag_functions(&ticking, &Self::tick_function_tag());
    }

    fn execute_tag_functions(&mut self, functions: &[CommandFunctionDefinition], tag: &Identifier) {
        self.events
            .push(ServerFunctionManagerEvent::ProfilerPush(tag.to_string()));
        for function in functions {
            self.execute(function, self.get_game_loop_sender());
        }
        self.events.push(ServerFunctionManagerEvent::ProfilerPop);
    }

    pub fn execute(&mut self, function: &CommandFunctionDefinition, sender: GameLoopSenderModel) {
        let id = Identifier::parse(&function.id).unwrap_or_else(|err| {
            unreachable!("command function ids are validated during load: {err}");
        });
        self.events.push(ServerFunctionManagerEvent::ProfilerPush(format!(
            "function {id}"
        )));
        self.events
            .push(ServerFunctionManagerEvent::ExecuteFunction { id, sender });
        self.events.push(ServerFunctionManagerEvent::ProfilerPop);
    }

    pub fn replace_library(&mut self, library: ServerFunctionLibrary) {
        self.post_reload(library);
    }

    fn post_reload(&mut self, library: ServerFunctionLibrary) {
        self.ticking = library.get_tag(&Self::tick_function_tag()).to_vec();
        self.library = library;
        self.post_reload = true;
    }

    pub fn get_game_loop_sender(&self) -> GameLoopSenderModel {
        GameLoopSenderModel {
            permission: FunctionPermissionLevel::Gamemaster,
            suppressed_output: true,
        }
    }

    pub fn get(&self, id: &Identifier) -> Option<&CommandFunctionDefinition> {
        self.library.get_function(id)
    }

    pub fn get_tag(&self, id: &Identifier) -> &[CommandFunctionDefinition] {
        self.library.get_tag(id)
    }

    pub fn get_function_names(&self) -> impl Iterator<Item = &Identifier> {
        self.library.get_functions().keys()
    }

    pub fn get_tag_names(&self) -> impl Iterator<Item = &Identifier> {
        self.library.get_available_tags()
    }

    pub fn events(&self) -> &[ServerFunctionManagerEvent] {
        &self.events
    }

    pub fn take_events(&mut self) -> Vec<ServerFunctionManagerEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn is_post_reload(&self) -> bool {
        self.post_reload
    }

    fn tick_function_tag() -> Identifier {
        match Identifier::parse("minecraft:tick") {
            Ok(id) => id,
            Err(err) => unreachable!("static identifier is valid: {err}"),
        }
    }

    fn load_function_tag() -> Identifier {
        match Identifier::parse("minecraft:load") {
            Ok(id) => id,
            Err(err) => unreachable!("static identifier is valid: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/ServerFunctionManager.java");

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn library_with_tick_and_load() -> ServerFunctionLibrary {
        ServerFunctionLibrary::reload_from_resources(
            [
                ("data/minecraft/function/tick.mcfunction", "say tick"),
                ("data/minecraft/function/load.mcfunction", "say load"),
            ],
            [
                (
                    "data/minecraft/tags/function/tick.json",
                    r#"{"values":["minecraft:tick"]}"#,
                ),
                (
                    "data/minecraft/tags/function/load.json",
                    r#"{"values":["minecraft:load"]}"#,
                ),
            ],
        )
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_function_manager_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("private static final Identifier TICK_FUNCTION_TAG = Identifier.withDefaultNamespace(\"tick\");"));
        assert!(JAVA_SOURCE.contains("private static final Identifier LOAD_FUNCTION_TAG = Identifier.withDefaultNamespace(\"load\");"));
        assert!(JAVA_SOURCE.contains("private List<CommandFunction<CommandSourceStack>> ticking = ImmutableList.of();"));
        assert!(JAVA_SOURCE.contains("private boolean postReload;"));
        assert!(JAVA_SOURCE.contains("public void tick()"));
        assert!(JAVA_SOURCE.contains("if (this.server.tickRateManager().runsNormally())"));
        assert!(JAVA_SOURCE.contains("this.executeTagFunctions(functions, LOAD_FUNCTION_TAG);"));
        assert!(JAVA_SOURCE.contains("this.executeTagFunctions(this.ticking, TICK_FUNCTION_TAG);"));
        assert!(JAVA_SOURCE.contains("public void execute(final CommandFunction<CommandSourceStack> functionIn"));
        assert!(JAVA_SOURCE.contains("Commands.executeCommandInContext(sender"));
        assert!(JAVA_SOURCE.contains("public void replaceLibrary(final ServerFunctionLibrary library)"));
        assert!(JAVA_SOURCE.contains("this.ticking = List.copyOf(library.getTag(TICK_FUNCTION_TAG));"));
        assert!(JAVA_SOURCE.contains("withPermission(LevelBasedPermissionSet.GAMEMASTER).withSuppressedOutput()"));
        assert!(JAVA_SOURCE.contains("public Optional<CommandFunction<CommandSourceStack>> get(final Identifier id)"));
        assert!(JAVA_SOURCE.contains("public Iterable<Identifier> getFunctionNames()"));
        assert!(JAVA_SOURCE.contains("public Iterable<Identifier> getTagNames()"));
    }

    #[test]
    fn server_function_manager_runs_load_once_then_tick_when_game_runs_normally() {
        let mut manager = ServerFunctionManager::new(library_with_tick_and_load());

        manager.tick(false);
        assert!(manager.events().is_empty());
        assert!(manager.is_post_reload());

        manager.tick(true);
        let first = manager.take_events();
        assert_eq!(
            first
                .iter()
                .filter_map(|event| match event {
                    ServerFunctionManagerEvent::ExecuteFunction { id, .. } => {
                        Some(id.to_string())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec!["minecraft:load", "minecraft:tick"]
        );
        assert!(!manager.is_post_reload());

        manager.tick(true);
        assert_eq!(
            manager
                .events()
                .iter()
                .filter_map(|event| match event {
                    ServerFunctionManagerEvent::ExecuteFunction { id, .. } => {
                        Some(id.to_string())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec!["minecraft:tick"]
        );
    }

    #[test]
    fn server_function_manager_replace_library_refreshes_ticking_and_loads_again() {
        let mut manager = ServerFunctionManager::new(library_with_tick_and_load());
        manager.tick(true);
        manager.take_events();

        let replacement = ServerFunctionLibrary::reload_from_resources(
            [
                ("data/custom/function/tick.mcfunction", "say custom tick"),
                ("data/custom/function/load.mcfunction", "say custom load"),
            ],
            [
                (
                    "data/minecraft/tags/function/tick.json",
                    r#"{"values":["custom:tick"]}"#,
                ),
                (
                    "data/minecraft/tags/function/load.json",
                    r#"{"values":["custom:load"]}"#,
                ),
            ],
        );
        manager.replace_library(replacement);
        assert!(manager.is_post_reload());

        manager.tick(true);
        assert_eq!(
            manager
                .events()
                .iter()
                .filter_map(|event| match event {
                    ServerFunctionManagerEvent::ExecuteFunction { id, .. } => {
                        Some(id.to_string())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec!["custom:load", "custom:tick"]
        );
    }

    #[test]
    fn server_function_manager_delegates_lookup_names_and_sender_shape() {
        let manager = ServerFunctionManager::new(library_with_tick_and_load());

        assert!(manager.get(&id("minecraft:tick")).is_some());
        assert_eq!(manager.get_tag(&id("minecraft:tick")).len(), 1);
        assert_eq!(
            manager
                .get_function_names()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["minecraft:load", "minecraft:tick"]
        );
        assert_eq!(
            manager
                .get_tag_names()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["minecraft:load", "minecraft:tick"]
        );
        assert_eq!(
            manager.get_game_loop_sender(),
            GameLoopSenderModel {
                permission: FunctionPermissionLevel::Gamemaster,
                suppressed_output: true,
            }
        );
    }
}
