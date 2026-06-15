#![allow(dead_code)]

#[cfg(test)]
mod tests {
    #[cfg(vibecraft_has_decompiled_sources)]
    const RETURN_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ReturnCommand.java");

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn return_command_source_matches_java_26_1_2() {
        for sentinel in [
            "LiteralArgumentBuilder.literal(\"return\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "RequiredArgumentBuilder.argument(\"value\", IntegerArgumentType.integer())",
            "new ReturnCommand.ReturnValueCustomExecutor()",
            "LiteralArgumentBuilder.literal(\"fail\")",
            "new ReturnCommand.ReturnFailCustomExecutor()",
            "LiteralArgumentBuilder.literal(\"run\")",
            "new ReturnCommand.ReturnFromCommandCustomModifier()",
            "sender.callback().onFailure()",
            "frame.returnFailure()",
            "frame.discard()",
            "currentSources.isEmpty()",
            "modifiers.isReturn()",
            "FallthroughTask.instance()",
            "output.currentFrame().discard()",
            "modifiers.setReturn()",
            "IntegerArgumentType.getInteger(currentStep.getTopContext(), \"value\")",
            "sender.callback().onSuccess(returnValue)",
            "frame.returnSuccess(returnValue)",
        ] {
            assert!(
                RETURN_COMMAND_JAVA.contains(sentinel),
                "ReturnCommand.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn return_command_does_not_emit_command_feedback() {
        assert!(
            !RETURN_COMMAND_JAVA.contains("sendSuccess"),
            "ReturnCommand.java should use callbacks/frame control instead of sendSuccess"
        );
        assert!(
            !RETURN_COMMAND_JAVA.contains("Component.translatable"),
            "ReturnCommand.java should not define feedback components"
        );
    }
}
