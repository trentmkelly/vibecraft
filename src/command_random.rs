#[cfg(test)]
mod tests {
    #[cfg(vibecraft_has_decompiled_sources)]
    const RANDOM_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/RandomCommand.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn random_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"random\")",
            "drawRandomValueTree(\"value\", false)",
            "drawRandomValueTree(\"roll\", true)",
            "Commands.literal(\"reset\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
            "Commands.literal(\"*\").executes(c -> resetAllSequences((CommandSourceStack)c.getSource()))",
            "Commands.argument(\"seed\", IntegerArgumentType.integer())",
            "Commands.argument(\"includeWorldSeed\", BoolArgumentType.bool())",
            "Commands.argument(\"includeSequenceId\", BoolArgumentType.bool())",
            "Commands.argument(\"sequence\", IdentifierArgument.id())",
            ".suggests(RandomCommand::suggestRandomSequence)",
            "Commands.argument(\"range\", RangeArgument.intRange())",
            ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
            "randomSample((CommandSourceStack)c.getSource(), RangeArgument.Ints.getRange(c, \"range\"), null, announce)",
            "IdentifierArgument.getId(c, \"sequence\")",
            "int min = range.min().orElse(Integer.MIN_VALUE);",
            "int max = range.max().orElse(Integer.MAX_VALUE);",
            "long span = (long)max - min;",
            "if (span == 0L)",
            "throw ERROR_RANGE_TOO_SMALL.create();",
            "if (span >= 2147483647L)",
            "throw ERROR_RANGE_TOO_LARGE.create();",
            "Mth.randomBetweenInclusive(random, min, max)",
            "broadcastSystemMessage(Component.translatable(\"commands.random.roll\"",
            "source.sendSuccess(() -> Component.translatable(\"commands.random.sample.success\"",
            "return value;",
            "getRandomSequences().reset(sequence, level.getSeed());",
            "getRandomSequences().reset(sequence, level.getSeed(), salt, includeWorldSeed, includeSequenceId);",
            "randomSequences.setSeedDefaults(salt, includeWorldSeed, includeSequenceId);",
            "int count = randomSequences.clear();",
            "return count;",
            "return 1;",
        ] {
            assert!(
                RANDOM_COMMAND_JAVA.contains(sentinel),
                "RandomCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
