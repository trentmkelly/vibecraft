#![allow(dead_code)]

#[cfg(test)]
mod tests {
    #[cfg(vibecraft_has_decompiled_sources)]
    const SUMMON_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SummonCommand.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const ENTITY_TYPE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/entity/EntityType.java");

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn summon_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"summon\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "ResourceArgument.resource(context, Registries.ENTITY_TYPE)",
            "SuggestionProviders.SUMMONABLE_ENTITIES",
            "ResourceArgument.getSummonableEntityType(c, \"entity\")",
            "Vec3Argument.vec3()",
            "CompoundTagArgument.compoundTag()",
            "new CompoundTag()",
            "BlockPos.containing(pos)",
            "Level.isInSpawnableBounds(blockPos)",
            "source.getLevel().getDifficulty() == Difficulty.PEACEFUL",
            "!type.value().isAllowedInPeaceful()",
            "entityTag.putString(\"id\", type.key().identifier().toString())",
            "EntityType.loadEntityRecursive(entityTag, level, EntitySpawnReason.COMMAND",
            "e.snapTo(pos.x, pos.y, pos.z, e.getYRot(), e.getXRot())",
            "if (entity == null)",
            "if (finalize && entity instanceof Mob mob)",
            "mob.finalizeSpawn(source.getLevel(), source.getLevel().getCurrentDifficultyAt(entity.blockPosition()), EntitySpawnReason.COMMAND, null)",
            "level.tryAddFreshEntityWithPassengers(entity)",
            "commands.summon.success",
        ] {
            assert!(
                SUMMON_COMMAND_JAVA.contains(sentinel),
                "SummonCommand.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn summon_entity_type_flags_match_java_defaults() {
        for sentinel in [
            "private boolean summon = true;",
            "private boolean allowedInPeaceful = true;",
            "public EntityType.Builder<T> noSummon()",
            "this.summon = false;",
            "public EntityType.Builder<T> notInPeaceful()",
            "this.allowedInPeaceful = false;",
            "public boolean canSummon()",
            "return this.summon;",
            "public boolean isAllowedInPeaceful()",
            "return this.allowedInPeaceful;",
            "EntityType.Builder.<Player>createNothing(MobCategory.MISC)",
            ".noSummon()",
            "EntityType.Builder.<FishingHook>of(FishingHook::new, MobCategory.MISC)",
            "EntityType.Builder.<Zombie>of(Zombie::new, MobCategory.MONSTER)",
            ".notInPeaceful()",
            "\"hoglin\", EntityType.Builder.of(Hoglin::new, MobCategory.MONSTER)",
        ] {
            assert!(
                ENTITY_TYPE_JAVA.contains(sentinel),
                "EntityType.java is missing summon sentinel: {sentinel}"
            );
        }
    }
}
