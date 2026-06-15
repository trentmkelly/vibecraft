#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopwatchError {
    AlreadyExists,
    DoesNotExist,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopwatchOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub broadcast_to_admins: bool,
}

pub fn create_stopwatch(already_exists: bool) -> Result<StopwatchOutput, StopwatchError> {
    if already_exists {
        return Err(StopwatchError::AlreadyExists);
    }
    Ok(success("commands.stopwatch.create.success", 1))
}

pub fn query_stopwatch(
    exists: bool,
    elapsed_seconds: f64,
    scale: f64,
) -> Result<StopwatchOutput, StopwatchError> {
    if !exists {
        return Err(StopwatchError::DoesNotExist);
    }
    Ok(success(
        "commands.stopwatch.query",
        (elapsed_seconds * scale) as i32,
    ))
}

pub fn restart_stopwatch(exists: bool) -> Result<StopwatchOutput, StopwatchError> {
    if !exists {
        return Err(StopwatchError::DoesNotExist);
    }
    Ok(success("commands.stopwatch.restart.success", 1))
}

pub fn remove_stopwatch(exists: bool) -> Result<StopwatchOutput, StopwatchError> {
    if !exists {
        return Err(StopwatchError::DoesNotExist);
    }
    Ok(success("commands.stopwatch.remove.success", 1))
}

fn success(feedback_key: &'static str, success_count: i32) -> StopwatchOutput {
    StopwatchOutput {
        success_count,
        feedback_key,
        broadcast_to_admins: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const STOPWATCH_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/StopwatchCommand.java");

    #[test]
    fn stopwatch_operations_match_java_success_counts_and_errors() {
        assert_eq!(
            create_stopwatch(false),
            Ok(success("commands.stopwatch.create.success", 1))
        );
        assert_eq!(create_stopwatch(true), Err(StopwatchError::AlreadyExists));
        assert_eq!(
            query_stopwatch(true, 3.25, 10.0),
            Ok(success("commands.stopwatch.query", 32))
        );
        assert_eq!(
            query_stopwatch(false, 3.25, 10.0),
            Err(StopwatchError::DoesNotExist)
        );
        assert_eq!(
            restart_stopwatch(true),
            Ok(success("commands.stopwatch.restart.success", 1))
        );
        assert_eq!(restart_stopwatch(false), Err(StopwatchError::DoesNotExist));
        assert_eq!(
            remove_stopwatch(true),
            Ok(success("commands.stopwatch.remove.success", 1))
        );
        assert_eq!(remove_stopwatch(false), Err(StopwatchError::DoesNotExist));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn stopwatch_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\n                           \"stopwatch\"",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "Commands.literal(\"create\")",
            "Commands.literal(\"query\")",
            "Commands.literal(\"restart\")",
            "Commands.literal(\"remove\")",
            "Commands.argument(\"id\", IdentifierArgument.id())",
            "Commands.argument(\"scale\", DoubleArgumentType.doubleArg())",
            "SUGGEST_STOPWATCHES",
            "new Stopwatch(Stopwatches.currentTime())",
            "if (!stopwatches.add(id, now))",
            "ERROR_ALREADY_EXISTS.create(id)",
            "commands.stopwatch.create.success",
            "Stopwatch stopwatch = stopwatches.get(id);",
            "ERROR_DOES_NOT_EXIST.create(id)",
            "double elapsedSeconds = stopwatch.elapsedSeconds(currentTime);",
            "commands.stopwatch.query",
            "return (int)(elapsedSeconds * scale);",
            "stopwatches.update(id, stopwatch -> new Stopwatch(Stopwatches.currentTime()))",
            "commands.stopwatch.restart.success",
            "if (!stopwatches.remove(id))",
            "commands.stopwatch.remove.success",
        ] {
            assert!(
                STOPWATCH_COMMAND_JAVA.contains(sentinel),
                "StopwatchCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
