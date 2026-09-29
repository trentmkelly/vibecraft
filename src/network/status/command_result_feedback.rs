//! Reports a live command's outcome to the executing player (`CommandSourceStack.sendSuccess`
//! / `sendFailure`).

use super::*;
use crate::command::{CommandError, CommandResult};

/// Sends the feedback for one executed command.
///
/// Java `ServerPlayer.commandSource.acceptsSuccess` is `GameRules.SEND_COMMAND_FEEDBACK`, so
/// success messages are dropped when it is off; `acceptsFailure` is always true.
pub(super) fn write_command_result_feedback(
    stream: &mut ClientStream,
    compression: CompressionState,
    result: Result<CommandResult, CommandError>,
    command_state: &ServerCommandState,
    game_rules: &SharedGameRules,
) -> io::Result<()> {
    match result {
        Ok(_) if !lock_status_mutex(game_rules).bool("send_command_feedback") => Ok(()),
        // Commands that report nothing (`/say`, `/msg`, ...) and `/kick`, whose per-player
        // feedback is delivered by `CommandEffects`.
        Ok(result)
            if result.feedback_key == crate::command::NO_COMMAND_FEEDBACK
                || result.feedback_key == KICK_SUCCESS_KEY =>
        {
            Ok(())
        }
        // Translatable feedback that carries arguments (e.g. `commands.gamerule.set`).
        Ok(result) if !command_state.feedback_args.is_empty() => write_system_chat_translatable(
            stream,
            compression,
            result.feedback_key,
            &command_state.feedback_args,
        ),
        Ok(result) => write_system_chat_text(
            stream,
            compression,
            &command_feedback_text(&result, command_state),
            false,
        ),
        // Brigadier `CommandSyntaxException` message.
        Err(CommandError::GameRuleArgument(error)) => {
            write_system_chat_text(stream, compression, &error.message(), false)
        }
        Err(error) => match error.translation() {
            // `CommandSourceStack.sendFailure(Component.translatable(key, args...))`.
            Some((key, args)) => write_system_chat_translatable(stream, compression, key, &args),
            None => write_system_chat_text(
                stream,
                compression,
                &format!("Command failed: {error:?}"),
                false,
            ),
        },
    }
}
