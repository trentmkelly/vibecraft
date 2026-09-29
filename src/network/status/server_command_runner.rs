//! Console and RCON command execution.
//!
//! Java `DedicatedServer.handleConsoleInputs` pops every queued `ConsoleInput` and calls
//! `Commands.performPrefixedCommand(source, msg)`; `DedicatedServer.runCommand` (invoked by
//! `RconClient` through `ServerInterface`) does the same with the `RconConsoleSource` stack after
//! `prepareForCommand()` and returns `getCommandResponse()`. Both funnel into the one command
//! engine ([`execute_builtin_command`]) exactly like a player's chat command, differing only in
//! the [`CommandOrigin`]:
//!
//! * console: the `MinecraftServer` itself (`CommandSource.NULL`-style, `shouldInformAdmins`
//!   true, permission `LevelBasedPermissionSet.OWNER`); messages go to the log
//!   (`MinecraftServer.sendSystemMessage` -> `LOGGER.info(message.getString())`).
//! * RCON: `RconConsoleSource` (`"Rcon"`, OWNER); every message's plain string is appended to a
//!   `StringBuffer` with no separator and returned to the client.

use super::*;
use crate::command::{CommandError, CommandResult, LevelBasedPermissionSet, PermissionLevel};
use crate::log::log_info;
use super::datapack_live::{apply_data_pack_requests, seed_data_pack_state, DataPackOutcome};
use super::play_session_world_packets::{
    command_feedback_text, function_permission_level_from_properties,
};
use std::sync::atomic::AtomicBool;
use crate::language;
use std::sync::OnceLock;

/// Late-bound handle to the live runner: `main` needs it for the RCON thread before the status
/// runtime (which owns the shared world state) exists.
pub type CommandRunnerSlot = Arc<OnceLock<ServerCommandRunner>>;

/// Who is executing a command (Java `CommandSourceStack.source`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOrigin {
    /// The dedicated server console (`MinecraftServer` as `CommandSource`).
    Console,
    /// `RconConsoleSource`; `broadcast_to_ops` is `shouldRconBroadcast()`
    /// (`broadcast-rcon-to-ops`).
    Rcon { broadcast_to_ops: bool },
}

impl CommandOrigin {
    /// `CommandSource.shouldInformAdmins`: the server console always informs admins.
    fn should_inform_admins(self) -> bool {
        match self {
            CommandOrigin::Console => true,
            CommandOrigin::Rcon { broadcast_to_ops } => broadcast_to_ops,
        }
    }

    /// `CommandSourceStack.getDisplayName` of the origin.
    fn display_name(self) -> &'static str {
        match self {
            CommandOrigin::Console => "Server",
            CommandOrigin::Rcon { .. } => "Rcon",
        }
    }
}

/// Executes console/RCON command lines against the live shared server state.
pub struct ServerCommandRunner {
    properties: ServerProperties,
    world_seed: i64,
    active_logins: ActiveLoginRegistry,
    game_rules: SharedGameRules,
    player_access: Arc<Mutex<PlayerAccess>>,
    /// Java runs every command on the server thread (`executeBlocking`); serialise likewise.
    execution: Mutex<()>,
    halt_requested: AtomicBool,
}

impl ServerCommandRunner {
    pub fn new(
        properties: &ServerProperties,
        world_seed: i64,
        active_logins: ActiveLoginRegistry,
        game_rules: SharedGameRules,
        player_access: Arc<Mutex<PlayerAccess>>,
    ) -> Self {
        Self {
            properties: properties.clone(),
            world_seed,
            active_logins,
            game_rules,
            player_access,
            execution: Mutex::new(()),
            halt_requested: AtomicBool::new(false),
        }
    }

    /// True once a command (`/stop`) asked the server to halt.
    pub fn halt_requested(&self) -> bool {
        self.halt_requested.load(Ordering::SeqCst)
    }

    /// Runs one console line; every message is logged (`LOGGER.info(message.getString())`).
    pub fn run_console(&self, line: &str) {
        // VibeCraft also reloads the on-disk player-access files on a console `reload`;
        // the command itself (the data pack reload) still runs below like in Java.
        if line.eq_ignore_ascii_case("reload") {
            self.reload_player_access();
        }
        for message in self.execute(CommandOrigin::Console, line) {
            log_info(&message);
        }
    }

    /// `DedicatedServer.runCommand`: runs an RCON command and returns the buffered response.
    pub fn run_rcon(&self, line: &str) -> String {
        let broadcast_to_ops = self.properties.broadcast_rcon_to_ops;
        self.execute(CommandOrigin::Rcon { broadcast_to_ops }, line)
            .concat()
    }

    fn reload_player_access(&self) {
        match lock_status_mutex(&self.player_access).reload_lists() {
            Ok(()) => println!("Reloaded player access files"),
            Err(err) => eprintln!("status access reload error: {err}"),
        }
    }

    /// `Commands.performPrefixedCommand` with an OWNER-permission source: returns, in order, the
    /// plain-string messages the source received.
    fn execute(&self, origin: CommandOrigin, line: &str) -> Vec<String> {
        let _serial = lock_status_mutex(&self.execution);
        let mut state = self.command_state();
        seed_data_pack_state(&mut state, line);
        let access_context = AccessContext {
            access: &self.player_access,
            properties: &self.properties,
            sessions: self.active_logins.live_sessions(),
            properties_file: Some(Path::new("server.properties")),
        };
        let access_seed = seed_access_state(&mut state, &self.player_access, access_context.sessions);
        let permissions = LevelBasedPermissionSet::new(PermissionLevel::Owners);
        let result = execute_builtin_command(&mut state, permissions, line);
        apply_command_game_rule_changes(&state, &self.game_rules);
        apply_access_changes(&access_context, &access_seed, &state);
        apply_console_disconnects(access_context.sessions, &state);
        if state.halt_requested {
            self.halt_requested.store(true, Ordering::SeqCst);
        }
        let reload = apply_data_pack_requests(&state, &self.active_logins.world_bus);
        let mut messages = self.messages_for(origin, line, result, &state);
        if reload == DataPackOutcome::Failed {
            // `ReloadCommand.reloadPacks`: `source.sendFailure(commands.reload.failure)`.
            messages.push(language::translate("commands.reload.failure", &[]));
        }
        messages
    }

    fn command_state(&self) -> ServerCommandState {
        let online_players = lock_status_mutex(&self.active_logins.sessions)
            .iter()
            .filter(|(_, session)| session.in_play)
            .map(|(uuid, session)| NameAndId {
                uuid: uuid.clone(),
                name: session.name.clone(),
            })
            .collect();
        let mut state = ServerCommandState {
            online_players,
            max_players: self.properties.max_players,
            world_seed: self.world_seed,
            world_preset: self.properties.level_type.clone(),
            function_permission_level: function_permission_level_from_properties(&self.properties),
            // A dedicated server is always "published" (`/kick` requires it).
            published_server: Some(dedicated_publish_request(&self.properties)),
            ..ServerCommandState::default()
        };
        seed_command_game_rules(&mut state, &self.game_rules);
        state
    }

    /// Converts a command outcome into the messages Java's `CommandSourceStack` delivers to the
    /// source (`sendSuccess` -> `sendSystemMessage`, `sendFailure`, `broadcastToAdmins`).
    fn messages_for(
        &self,
        origin: CommandOrigin,
        line: &str,
        result: Result<CommandResult, CommandError>,
        state: &ServerCommandState,
    ) -> Vec<String> {
        match result {
            Ok(result) => {
                if result.feedback_key.is_empty() {
                    return Vec::new();
                }
                let text = success_message(&result, state);
                if result.broadcast_to_admins && origin.should_inform_admins() {
                    self.broadcast_to_admins(origin, &text);
                }
                vec![text]
            }
            Err(CommandError::GameRuleArgument(error)) => vec![error.message()],
            // `Commands.finishParsing`: the raw message, then the context component
            // (`<input up to cursor>` + `<remaining>` + `<--[HERE]`), the cursor being 0 for an
            // unknown root literal.
            Err(CommandError::PermissionDenied | CommandError::InvalidSyntax) => {
                let input = line.strip_prefix('/').unwrap_or(line);
                vec![
                    language::translate("command.unknown.command", &[]),
                    format!("{input}{}", language::translate("command.context.here", &[])),
                ]
            }
            // TODO(command-error-messages): the model reports typed errors without Brigadier
            // messages/cursors, so only the syntax/permission family is rendered 1:1.
            Err(error) => vec![format!("Command failed: {error:?}")],
        }
    }

    /// `CommandSourceStack.broadcastToAdmins`: `[<source>: <message>]` to online ops (when
    /// `sendCommandFeedback`) and to the log for non-server sources (when `logAdminCommands`).
    fn broadcast_to_admins(&self, origin: CommandOrigin, text: &str) {
        let (send_feedback, log_admin) = {
            let rules = lock_status_mutex(&self.game_rules);
            (rules.bool("send_command_feedback"), rules.bool("log_admin_commands"))
        };
        // TODO(rcon-broadcast-ops): delivering the message to online operators needs the
        // cross-player send path (see the live player registry gap); only the log leg is live.
        let _ = send_feedback;
        if origin != CommandOrigin::Console && log_admin {
            log_info(&language::translate(
                "chat.type.admin",
                &[origin.display_name().to_string(), text.to_string()],
            ));
        }
    }
}

/// Plain-string text of a command's success feedback (`Component.getString`).
fn success_message(result: &CommandResult, state: &ServerCommandState) -> String {
    if !state.feedback_args.is_empty() {
        return language::translate(result.feedback_key, &state.feedback_args);
    }
    let text = command_feedback_text(result, state);
    let fallback = format!("{} ({})", result.feedback_key, result.success_count);
    if text != fallback {
        return text;
    }
    match result.feedback_key.strip_prefix(crate::command::LITERAL_COMMAND_FEEDBACK_PREFIX) {
        Some(literal) => literal.to_string(),
        None => language::translate(result.feedback_key, &[]),
    }
}

/// Drains queued console input (`DedicatedServer.handleConsoleInputs`); returns true once the
/// server was asked to stop.
pub(super) fn handle_console_inputs(
    console_input: &Receiver<ConsoleInput>,
    runner: &ServerCommandRunner,
) -> bool {
    loop {
        match console_input.try_recv() {
            Ok(input) => runner.run_console(input.line()),
            Err(_) => return runner.halt_requested(),
        }
    }
}
