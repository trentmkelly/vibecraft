#[cfg(test)]
mod tests {
    #[cfg(vibecraft_has_decompiled_sources)]
    const RANDOM_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/RandomCommand.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn random_command_source_matches_java_26_1_2() {
        assert!(RANDOM_COMMAND_JAVA.contains("drawRandomValueTree(\"value\", false)"));
        assert!(RANDOM_COMMAND_JAVA.contains("drawRandomValueTree(\"roll\", true)"));
        assert!(RANDOM_COMMAND_JAVA.contains(".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))"));
        assert!(RANDOM_COMMAND_JAVA.contains("randomSample((CommandSourceStack)c.getSource(), RangeArgument.Ints.getRange(c, \"range\"), null, announce)"));
        assert!(RANDOM_COMMAND_JAVA.contains("throw ERROR_RANGE_TOO_SMALL.create();"));
        assert!(RANDOM_COMMAND_JAVA.contains("throw ERROR_RANGE_TOO_LARGE.create();"));
        assert!(RANDOM_COMMAND_JAVA.contains("broadcastSystemMessage(Component.translatable(\"commands.random.roll\""));
        assert!(RANDOM_COMMAND_JAVA.contains("source.sendSuccess(() -> Component.translatable(\"commands.random.sample.success\""));
        assert!(RANDOM_COMMAND_JAVA.contains("getRandomSequences().reset(sequence, level.getSeed());"));
        assert!(RANDOM_COMMAND_JAVA.contains("getRandomSequences().reset(sequence, level.getSeed(), salt, includeWorldSeed, includeSequenceId);"));
        assert!(RANDOM_COMMAND_JAVA.contains("randomSequences.setSeedDefaults(salt, includeWorldSeed, includeSequenceId);"));
        assert!(RANDOM_COMMAND_JAVA.contains("randomSequences.clear();"));
    }
}
