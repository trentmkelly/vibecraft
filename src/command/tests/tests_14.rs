use super::*;

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn chat_message_command_sources_match_java_26_1_2() {
    const MSG: &str = vibecraft_java_source!("/net/minecraft/server/commands/MsgCommand.java");
    const TEAMMSG: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/TeamMsgCommand.java");
    const SAY: &str = vibecraft_java_source!("/net/minecraft/server/commands/SayCommand.java");
    const TELLRAW: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/TellRawCommand.java");

    for sentinel in [
        "Commands.literal(\"msg\")",
        "Commands.argument(\"targets\", EntityArgument.players())",
        "Commands.argument(\"message\", MessageArgument.message())",
        "MessageArgument.resolveChatMessage(c, \"message\", message -> sendMessage",
        "return players.size();",
        "Commands.literal(\"tell\").redirect(msg)",
        "Commands.literal(\"w\").redirect(msg)",
        "ChatType.MSG_COMMAND_INCOMING",
        "ChatType.MSG_COMMAND_OUTGOING",
        "source.sendChatMessage(tracked, false, outgoingChatType)",
        "player.sendChatMessage(tracked, filtered, incomingChatType)",
        "PlayerList.CHAT_FILTERED_FULL",
    ] {
        assert!(
            MSG.contains(sentinel),
            "MsgCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "Commands.literal(\"teammsg\")",
        "Commands.argument(\"message\", MessageArgument.message())",
        "Entity entity = source.getEntityOrException();",
        "PlayerTeam team = entity.getTeam();",
        "throw ERROR_NOT_ON_TEAM.create();",
        "receiver == entity || receiver.getTeam() == team",
        "MessageArgument.resolveChatMessage(c, \"message\", message -> sendMessage",
        "return receivers.size();",
        "Commands.literal(\"tm\").redirect(msg)",
        "ChatType.TEAM_MSG_COMMAND_INCOMING",
        "ChatType.TEAM_MSG_COMMAND_OUTGOING",
        "teamPlayer == entity ? outgoingChatType : incomingChatType",
    ] {
        assert!(
            TEAMMSG.contains(sentinel),
            "TeamMsgCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "Commands.literal(\"say\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"message\", MessageArgument.message())",
        "MessageArgument.resolveChatMessage(c, \"message\", message ->",
        "playerList.broadcastChatMessage(message, source, ChatType.bind(ChatType.SAY_COMMAND, source));",
        "return 1;",
    ] {
        assert!(
            SAY.contains(sentinel),
            "SayCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "Commands.literal(\"tellraw\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"targets\", EntityArgument.players())",
        "Commands.argument(\"message\", ComponentArgument.textComponent(context))",
        "player.sendSystemMessage(ComponentArgument.getResolvedComponent(c, \"message\", player));",
        "result++;",
        "return result;",
    ] {
        assert!(
            TELLRAW.contains(sentinel),
            "TellRawCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn rotate_command_defaults_facing_entity_anchor_to_feet() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "rotate pig facing entity cow",
    )
    .unwrap();

    assert_eq!(
        state.rotation_requests[0].mode,
        RotationMode::FacingEntity {
            entity: EntityRef {
                id: "cow".to_string(),
                display_name: "cow".to_string(),
            },
            anchor: EntityAnchor::Feet,
        }
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn rotate_command_source_matches_java_26_1_2() {
    const ROTATE: &str = vibecraft_java_source!("/net/minecraft/server/commands/RotateCommand.java");

    for sentinel in [
        "Commands.literal(\"rotate\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"target\", EntityArgument.entity())",
        "Commands.argument(\"rotation\", RotationArgument.rotation())",
        "RotationArgument.getRotation(c, \"rotation\")",
        "Commands.literal(\"facing\")",
        "Commands.literal(\"entity\")",
        "Commands.argument(\"facingEntity\", EntityArgument.entity())",
        "EntityAnchorArgument.Anchor.FEET",
        "Commands.argument(\"facingAnchor\", EntityAnchorArgument.anchor())",
        "EntityAnchorArgument.getAnchor(c, \"facingAnchor\")",
        "Commands.argument(\"facingLocation\", Vec3Argument.vec3())",
        "new LookAt.LookAtPosition(Vec3Argument.getVec3(c, \"facingLocation\"))",
        "rotation.getRotation(source)",
        "rotation.isYRelative() ? rot.y - entity.getYRot() : rot.y",
        "rotation.isXRelative() ? rot.x - entity.getXRot() : rot.x",
        "entity.forceSetRotation(relativeOrAbsoluteYRot, rotation.isYRelative(), relativeOrAbsoluteXRot, rotation.isXRelative())",
        "facing.perform(source, entity)",
        "Component.translatable(\"commands.rotate.success\", entity.getDisplayName())",
        "return 1;",
    ] {
        assert!(
            ROTATE.contains(sentinel),
            "RotateCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn schedule_command_source_matches_java_26_1_2() {
    const SCHEDULE: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ScheduleCommand.java");

    for sentinel in [
        "Commands.literal(\"schedule\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"function\")",
        "Commands.argument(\"function\", FunctionArgument.functions())",
        ".suggests(FunctionCommand.SUGGEST_FUNCTION)",
        "Commands.argument(\"time\", TimeArgument.time())",
        "FunctionArgument.getFunctionOrTag(c, \"function\")",
        "Commands.literal(\"append\")",
        "Commands.literal(\"replace\")",
        "Commands.literal(\"clear\")",
        "Commands.argument(\"function\", StringArgumentType.greedyString())",
        ".suggests(SUGGEST_SCHEDULE)",
        "if (time == 0)",
        "throw ERROR_SAME_TICK.create();",
        "long tickTime = source.getLevel().getGameTime() + time;",
        "function.get() instanceof MacroFunction",
        "throw ERROR_MACRO.create();",
        "if (replace)",
        "queue.remove(scheduleId);",
        "queue.schedule(scheduleId, tickTime, new FunctionCallback(callbackId));",
        "String scheduleId = \"#\" + callbackId;",
        "queue.schedule(scheduleId, tickTime, new FunctionTagCallback(callbackId));",
        "commands.schedule.created.function",
        "commands.schedule.created.tag",
        "return Math.floorMod(tickTime, Integer.MAX_VALUE);",
        "int count = source.getServer().getScheduledEvents().remove(id);",
        "throw ERROR_CANT_REMOVE.create(id);",
        "commands.schedule.cleared.success",
        "return count;",
    ] {
        assert!(
            SCHEDULE.contains(sentinel),
            "ScheduleCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn time_command_source_matches_java_26_1_2() {
    const TIME: &str = vibecraft_java_source!("/net/minecraft/server/commands/TimeCommand.java");

    for sentinel in [
        "Commands.literal(\"time\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "addClockNodes(context, baseCommand, c -> getDefaultClock((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"query\").then(Commands.literal(\"gametime\")",
        "Commands.literal(\"of\")",
        "Commands.argument(\"clock\", ResourceArgument.resource(context, Registries.WORLD_CLOCK))",
        "Commands.literal(\"set\")",
        "Commands.argument(\"time\", TimeArgument.time())",
        "Commands.argument(\"timemarker\", IdentifierArgument.id())",
        "suggestTimeMarkers((CommandSourceStack)c.getSource(), p, clockGetter.getClock(c))",
        "ClockTimeMarkers.ROOT_ID",
        "Commands.literal(\"add\")",
        "Commands.argument(\"time\", TimeArgument.time(Integer.MIN_VALUE))",
        "Commands.literal(\"pause\")",
        "Commands.literal(\"resume\")",
        "Commands.literal(\"rate\")",
        "FloatArgumentType.floatArg(1.0E-5F, 1000.0F)",
        "Commands.literal(\"query\")",
        "Commands.literal(\"time\")",
        "Commands.argument(\"timeline\", ResourceArgument.resource(context, Registries.TIMELINE))",
        "Commands.literal(\"repetition\")",
        "ERROR_NO_DEFAULT_CLOCK.create(dimensionType.getRegisteredName())",
        "ERROR_NO_TIME_MARKER_FOUND.create(clock.getRegisteredName(), timeMarkerId)",
        "ERROR_WRONG_TIMELINE_FOR_CLOCK.create(clock.getRegisteredName(), timeline.getRegisteredName())",
        "source.getLevel().getGameTime()",
        "Component.translatable(\"commands.time.query.gametime\", gameTime)",
        "Component.translatable(\"commands.time.query.absolute\", clock.getRegisteredName(), totalTicks)",
        "Component.translatable(\"commands.time.query.timeline\", timeline.getRegisteredName(), currentTicks)",
        "Component.translatable(\"commands.time.query.timeline.repetitions\", timeline.getRegisteredName(), repetitions)",
        "clockManager.setTotalTicks(clock, totalTicks)",
        "clockManager.addTicks(clock, time)",
        "clockManager.moveToTimeMarker(clock, timeMarkerId)",
        "Component.translatable(\"commands.time.set.absolute\", clock.getRegisteredName(), totalTicks)",
        "Component.translatable(\"commands.time.set.time_marker\", clock.getRegisteredName(), timeMarkerId.identifier().toString())",
        "setPaused(clock, paused)",
        "setRate(clock, rate)",
        "return Math.toIntExact(ticks % 2147483647L);",
    ] {
        assert!(
            TIME.contains(sentinel),
            "TimeCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn title_command_source_matches_java_26_1_2() {
    const TITLE: &str = vibecraft_java_source!("/net/minecraft/server/commands/TitleCommand.java");

    for sentinel in [
        "Commands.literal(\"title\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "\"targets\", EntityArgument.players()",
        "Commands.literal(\"clear\")",
        "clearTitle((CommandSourceStack)c.getSource(), EntityArgument.getPlayers(c, \"targets\"))",
        "Commands.literal(\"reset\")",
        "resetTitle((CommandSourceStack)c.getSource(), EntityArgument.getPlayers(c, \"targets\"))",
        "Commands.literal(\"title\")",
        "Commands.literal(\"subtitle\")",
        "Commands.literal(\"actionbar\")",
        "Commands.argument(\"title\", ComponentArgument.textComponent(context))",
        "ComponentArgument.getRawComponent(c, \"title\")",
        "ClientboundSetTitleTextPacket::new",
        "ClientboundSetSubtitleTextPacket::new",
        "ClientboundSetActionBarTextPacket::new",
        "Commands.literal(\"times\")",
        "Commands.argument(\"fadeIn\", TimeArgument.time())",
        "Commands.argument(\"stay\", TimeArgument.time())",
        "Commands.argument(\"fadeOut\", TimeArgument.time())",
        "new ClientboundClearTitlesPacket(false)",
        "new ClientboundClearTitlesPacket(true)",
        "new ClientboundSetTitlesAnimationPacket(fadeIn, stay, fadeOut)",
        "player.connection.send(packet)",
        "ComponentUtils.resolve(ResolutionContext.builder().withSource(source).withEntityOverride(player).build(), title)",
        "source.sendSuccess(() -> Component.translatable(\"commands.title.cleared.single\", targets.iterator().next().getDisplayName()), true)",
        "source.sendSuccess(() -> Component.translatable(\"commands.title.reset.multiple\", targets.size()), true)",
        "source.sendSuccess(() -> Component.translatable(\"commands.title.show.\" + type + \".single\", targets.iterator().next().getDisplayName()), true)",
        "source.sendSuccess(() -> Component.translatable(\"commands.title.times.multiple\", targets.size()), true)",
        "return targets.size();",
    ] {
        assert!(
            TITLE.contains(sentinel),
            "TitleCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn version_command_source_matches_java_26_1_2() {
    const VERSION: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/VersionCommand.java");

    for sentinel in [
        "Commands.literal(\"version\")",
        ".requires(Commands.hasPermission(checkPermissions ? Commands.LEVEL_GAMEMASTERS : Commands.LEVEL_ALL))",
        "source.sendSystemMessage(HEADER)",
        "dumpVersion(source::sendSystemMessage)",
        "return 1;",
        "private static final Component HEADER = Component.translatable(\"commands.version.header\")",
        "private static final Component STABLE = Component.translatable(\"commands.version.stable.yes\")",
        "private static final Component UNSTABLE = Component.translatable(\"commands.version.stable.no\")",
        "WorldVersion version = SharedConstants.getCurrentVersion();",
        "Component.translatable(\"commands.version.id\", version.id())",
        "Component.translatable(\"commands.version.name\", version.name())",
        "Component.translatable(\"commands.version.data\", version.dataVersion().version())",
        "Component.translatable(\"commands.version.series\", version.dataVersion().series())",
        "Component.translatable(\"commands.version.protocol\", version.protocolVersion(), \"0x\" + Integer.toHexString(version.protocolVersion()))",
        "Component.translatable(\"commands.version.build_time\", Component.translationArg(version.buildTime()))",
        "Component.translatable(\"commands.version.pack.resource\", version.packVersion(PackType.CLIENT_RESOURCES).toString())",
        "Component.translatable(\"commands.version.pack.data\", version.packVersion(PackType.SERVER_DATA).toString())",
        "output.accept(version.stable() ? STABLE : UNSTABLE);",
    ] {
        assert!(
            VERSION.contains(sentinel),
            "VersionCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn weather_command_source_matches_java_26_1_2() {
    const WEATHER: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/WeatherCommand.java");

    for sentinel in [
        "private static final int DEFAULT_TIME = -1;",
        "Commands.literal(\"weather\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"clear\").executes(c -> setClear((CommandSourceStack)c.getSource(), -1))",
        "Commands.literal(\"rain\").executes(c -> setRain((CommandSourceStack)c.getSource(), -1))",
        "Commands.literal(\"thunder\").executes(c -> setThunder((CommandSourceStack)c.getSource(), -1))",
        "Commands.argument(\"duration\", TimeArgument.time(1))",
        "IntegerArgumentType.getInteger(c, \"duration\")",
        "input == -1 ? defaultDistribution.sample(source.getLevel().getRandom()) : input",
        "source.getServer().setWeatherParameters(getDuration(source, duration, ServerLevel.RAIN_DELAY), 0, false, false)",
        "source.getServer().setWeatherParameters(0, getDuration(source, duration, ServerLevel.RAIN_DURATION), true, false)",
        "source.getServer().setWeatherParameters(0, getDuration(source, duration, ServerLevel.THUNDER_DURATION), true, true)",
        "Component.translatable(\"commands.weather.set.clear\")",
        "Component.translatable(\"commands.weather.set.rain\")",
        "Component.translatable(\"commands.weather.set.thunder\")",
        "return duration;",
    ] {
        assert!(
            WEATHER.contains(sentinel),
            "WeatherCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn warden_spawn_tracker_command_source_matches_java_26_1_2() {
    const WARDEN: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/WardenSpawnTrackerCommand.java");

    for sentinel in [
        "Commands.literal(\"warden_spawn_tracker\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"clear\")",
        "ImmutableList.of(((CommandSourceStack)c.getSource()).getPlayerOrException())",
        "resetTracker((CommandSourceStack)c.getSource(),",
        "Commands.literal(\"set\")",
        "Commands.argument(\"warning_level\", IntegerArgumentType.integer(0, 4))",
        "IntegerArgumentType.getInteger(c, \"warning_level\")",
        "wardenSpawnTracker.setWarningLevel(warningLevel)",
        "player.getWardenSpawnTracker().ifPresent(WardenSpawnTracker::reset)",
        "Component.translatable(\"commands.warden_spawn_tracker.set.success.single\", players.iterator().next().getDisplayName())",
        "Component.translatable(\"commands.warden_spawn_tracker.set.success.multiple\", players.size())",
        "Component.translatable(\"commands.warden_spawn_tracker.clear.success.single\", players.iterator().next().getDisplayName())",
        "Component.translatable(\"commands.warden_spawn_tracker.clear.success.multiple\", players.size())",
        "return players.size();",
    ] {
        assert!(
            WARDEN.contains(sentinel),
            "WardenSpawnTrackerCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn worldborder_command_source_matches_java_26_1_2() {
    const WORLDBORDER: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/WorldBorderCommand.java");

    for sentinel in [
        "\"worldborder\"",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"add\")",
        "Commands.argument(\"distance\", DoubleArgumentType.doubleArg(-5.999997E7F, 5.999997E7F))",
        "+ DoubleArgumentType.getDouble(c, \"distance\")",
        "getWorldBorder().getLerpTime()",
        "+ IntegerArgumentType.getInteger(c, \"time\")",
        "Commands.literal(\"set\")",
        "Commands.argument(\"time\", TimeArgument.time(0))",
        "Commands.literal(\"center\")",
        "Commands.argument(\"pos\", Vec2Argument.vec2())",
        "Commands.literal(\"damage\")",
        "Commands.literal(\"amount\")",
        "Commands.argument(\"damagePerBlock\", FloatArgumentType.floatArg(0.0F))",
        "Commands.literal(\"buffer\")",
        "Commands.argument(\"distance\", FloatArgumentType.floatArg(0.0F))",
        "Commands.literal(\"get\")",
        "Commands.literal(\"warning\")",
        "Commands.argument(\"distance\", IntegerArgumentType.integer(0))",
        "if (border.getSafeZone() == distance)",
        "throw ERROR_SAME_DAMAGE_BUFFER.create();",
        "if (border.getDamagePerBlock() == damagePerBlock)",
        "throw ERROR_SAME_DAMAGE_AMOUNT.create();",
        "if (border.getWarningTime() == ticks)",
        "throw ERROR_SAME_WARNING_TIME.create();",
        "if (border.getWarningBlocks() == distance)",
        "throw ERROR_SAME_WARNING_DISTANCE.create();",
        "Mth.floor(size + 0.5)",
        "if (border.getCenterX() == center.x && border.getCenterZ() == center.y)",
        "throw ERROR_SAME_CENTER.create();",
        "Math.abs(center.x) > 2.9999984E7",
        "throw ERROR_TOO_FAR_OUT.create();",
        "if (current == distance)",
        "throw ERROR_SAME_SIZE.create();",
        "if (distance < 1.0)",
        "throw ERROR_TOO_SMALL.create();",
        "if (distance > 5.999997E7F)",
        "throw ERROR_TOO_BIG.create();",
        "border.lerpSizeBetween(current, distance, ticks, level.getGameTime())",
        "border.setSize(distance)",
        "commands.worldborder.set.grow",
        "commands.worldborder.set.shrink",
        "commands.worldborder.set.immediate",
        "return (int)(distance - current);",
        "return String.format(Locale.ROOT, \"%.2f\", ticks / 20.0);",
    ] {
        assert!(
            WORLDBORDER.contains(sentinel),
            "WorldBorderCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn transfer_command_source_matches_java_26_1_2() {
    const TRANSFER: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/TransferCommand.java");

    for sentinel in [
        "Commands.literal(\"transfer\").requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
        "Commands.argument(\"hostname\", StringArgumentType.string())",
        "StringArgumentType.getString(c, \"hostname\")",
        "25565",
        "List.of(((CommandSourceStack)c.getSource()).getPlayerOrException())",
        "Commands.argument(\"port\", IntegerArgumentType.integer(1, 65535))",
        "IntegerArgumentType.getInteger(c, \"port\")",
        "Commands.argument(\"players\", EntityArgument.players())",
        "EntityArgument.getPlayers(c, \"players\")",
        "if (players.isEmpty())",
        "throw ERROR_NO_PLAYERS.create();",
        "player.connection.send(new ClientboundTransferPacket(hostname, port))",
        "Component.translatable(\"commands.transfer.success.single\", players.iterator().next().getDisplayName(), hostname, port)",
        "Component.translatable(\"commands.transfer.success.multiple\", players.size(), hostname, port)",
        "return players.size();",
    ] {
        assert!(
            TRANSFER.contains(sentinel),
            "TransferCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn trigger_command_source_matches_java_26_1_2() {
    const TRIGGER: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/TriggerCommand.java");
    const SCORE_ACCESS: &str = vibecraft_java_source!("/net/minecraft/world/scores/ScoreAccess.java");

    for sentinel in [
        "Commands.literal(\"trigger\")",
        "Commands.argument(\"objective\", ObjectiveArgument.objective())",
        ".suggests((c, p) -> suggestObjectives((CommandSourceStack)c.getSource(), p))",
        "((CommandSourceStack)c.getSource()).getPlayerOrException()",
        "ObjectiveArgument.getObjective(c, \"objective\")",
        "Commands.literal(\"add\")",
        "Commands.argument(\"value\", IntegerArgumentType.integer())",
        "IntegerArgumentType.getInteger(c, \"value\")",
        "Commands.literal(\"set\")",
        "objective.getCriteria() == ObjectiveCriteria.TRIGGER",
        "scoreInfo != null && !scoreInfo.isLocked()",
        "result.add(objective.getName())",
        "if (objective.getCriteria() != ObjectiveCriteria.TRIGGER)",
        "throw ERROR_INVALID_OBJECTIVE.create();",
        "score.lock();",
        "throw ERROR_NOT_PRIMED.create();",
        "int newValue = score.add(amount);",
        "int newValue = score.add(1);",
        "score.set(amount);",
        "Component.translatable(\"commands.trigger.simple.success\", objective.getFormattedDisplayName())",
        "Component.translatable(\"commands.trigger.add.success\", objective.getFormattedDisplayName(), amount)",
        "Component.translatable(\"commands.trigger.set.success\", objective.getFormattedDisplayName(), amount)",
        "return newValue;",
        "return amount;",
    ] {
        assert!(
            TRIGGER.contains(sentinel),
            "TriggerCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "default int add(final int count)",
        "int newValue = this.get() + count;",
        "this.set(newValue);",
        "return newValue;",
    ] {
        assert!(
            SCORE_ACCESS.contains(sentinel),
            "ScoreAccess.java is missing trigger arithmetic sentinel: {sentinel}"
        );
    }
}
