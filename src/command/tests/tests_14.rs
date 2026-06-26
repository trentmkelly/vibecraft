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

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn waypoint_command_source_matches_java_26_1_2() {
    const WAYPOINT: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/WaypointCommand.java");

    for sentinel in [
        "Commands.literal(\"waypoint\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"list\").executes(c -> listWaypoints((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"modify\")",
        "Commands.argument(\"waypoint\", EntityArgument.entity())",
        "WaypointArgument.getWaypoint(c, \"waypoint\")",
        "Commands.literal(\"color\")",
        "Commands.argument(\"color\", ColorArgument.color())",
        "ColorArgument.getColor(c, \"color\")",
        "Commands.literal(\"hex\")",
        "Commands.argument(\"color\", HexColorArgument.hexColor())",
        "HexColorArgument.getHexColor(c, \"color\")",
        "Commands.literal(\"reset\")",
        "Commands.literal(\"style\")",
        "WaypointStyleAssets.DEFAULT",
        "Commands.literal(\"set\")",
        "Commands.argument(\"style\", IdentifierArgument.id())",
        "ResourceKey.create(WaypointStyleAssets.ROOT_ID, IdentifierArgument.getId(c, \"style\"))",
        "mutateIcon(source, waypoint, icon -> icon.style = style)",
        "mutateIcon(source, waypoint, icon -> icon.color = Optional.of(color.getColor()))",
        "mutateIcon(source, waypoint, icon -> icon.color = Optional.of(color))",
        "mutateIcon(source, waypoint, icon -> icon.color = Optional.empty())",
        "Component.translatable(\"commands.waypoint.modify.style\")",
        "Component.translatable(\"commands.waypoint.modify.color\"",
        "Component.translatable(\"commands.waypoint.modify.color.reset\")",
        "source.sendSuccess(() -> Component.translatable(\"commands.waypoint.list.empty\", dimension), false)",
        "Component.translatable(\"commands.waypoint.list.success\", waypoints.size(), dimension, waypointNames)",
        "return waypoints.size();",
        "return 0;",
        "level.getWaypointManager().untrackWaypoint(waypoint)",
        "iconConsumer.accept(waypoint.waypointIcon())",
        "level.getWaypointManager().trackWaypoint(waypoint)",
    ] {
        assert!(
            WAYPOINT.contains(sentinel),
            "WaypointCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn whitelist_command_source_matches_java_26_1_2() {
    const WHITELIST: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/WhitelistCommand.java");

    for sentinel in [
        "\"whitelist\"",
        ".requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
        "Commands.literal(\"on\").executes(c -> enableWhitelist((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"off\").executes(c -> disableWhitelist((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"list\").executes(c -> showList((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"add\")",
        "Commands.argument(\"targets\", GameProfileArgument.gameProfile())",
        "GameProfileArgument.getGameProfiles(c, \"targets\")",
        "list.getPlayers()",
        ".filter(nameAndId -> !list.getWhiteList().isWhiteListed(nameAndId))",
        "Commands.literal(\"remove\")",
        "SharedSuggestionProvider.suggest(((CommandSourceStack)c.getSource()).getServer().getPlayerList().getWhiteListNames(), p)",
        "Commands.literal(\"reload\").executes(c -> reload((CommandSourceStack)c.getSource()))",
        "source.getServer().getPlayerList().reloadWhiteList();",
        "source.getServer().kickUnlistedPlayers();",
        "return 1;",
        "if (!list.isWhiteListed(target))",
        "list.add(entry);",
        "Component.translatable(\"commands.whitelist.add.success\", Component.literal(target.name()))",
        "throw ERROR_ALREADY_WHITELISTED.create();",
        "if (list.isWhiteListed(target))",
        "list.remove(entry);",
        "Component.translatable(\"commands.whitelist.remove.success\", Component.literal(target.name()))",
        "throw ERROR_NOT_WHITELISTED.create();",
        "if (source.getServer().isUsingWhitelist())",
        "throw ERROR_ALREADY_ENABLED.create();",
        "source.getServer().setUsingWhitelist(true);",
        "Component.translatable(\"commands.whitelist.enabled\")",
        "if (!source.getServer().isUsingWhitelist())",
        "throw ERROR_ALREADY_DISABLED.create();",
        "source.getServer().setUsingWhitelist(false);",
        "Component.translatable(\"commands.whitelist.disabled\")",
        "String[] list = source.getServer().getPlayerList().getWhiteListNames();",
        "Component.translatable(\"commands.whitelist.none\")",
        "Component.translatable(\"commands.whitelist.list\", list.length, String.join(\", \", list))",
        "return list.length;",
    ] {
        assert!(
            WHITELIST.contains(sentinel),
            "WhitelistCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn tick_sprint_restart_records_java_interruption_feedback() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick sprint 10t").unwrap();

    let restarted =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick sprint 5t")
            .unwrap();

    assert_eq!(restarted.success_count, 1);
    assert_eq!(restarted.feedback_key, "commands.tick.status.sprinting");
    assert!(restarted.broadcast_to_admins);
    assert_eq!(
        state.tick_feedback_events,
        vec![TickCommandFeedbackEvent {
            feedback_key: "commands.tick.sprint.stop.success",
            broadcast_to_admins: true,
        }]
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn tick_command_source_matches_java_26_1_2() {
    const TICK_COMMAND: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/TickCommand.java");

    for sentinel in [
        "Commands.literal(\n                                 \"tick\"",
        ".requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
        "Commands.literal(\"query\").executes(c -> tickQuery((CommandSourceStack)c.getSource()))",
        "Commands.argument(\"rate\", FloatArgumentType.floatArg(1.0F, 10000.0F))",
        "Commands.literal(\"step\").executes(c -> step((CommandSourceStack)c.getSource(), 1))",
        "Commands.literal(\"stop\").executes(c -> stopStepping((CommandSourceStack)c.getSource()))",
        "Commands.argument(\"time\", TimeArgument.time(1))",
        "Commands.literal(\"sprint\")",
        "Commands.literal(\"stop\").executes(c -> stopSprinting((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"unfreeze\").executes(c -> setFreeze((CommandSourceStack)c.getSource(), false))",
        "Commands.literal(\"freeze\").executes(c -> setFreeze((CommandSourceStack)c.getSource(), true))",
        "source.sendSuccess(() -> Component.translatable(\"commands.tick.rate.success\", tickRateString), true)",
        "source.sendSuccess(() -> Component.translatable(\"commands.tick.status.sprinting\"), false)",
        "source.sendSuccess(() -> Component.translatable(\"commands.tick.status.frozen\"), false)",
        "source.sendSuccess(() -> Component.translatable(\"commands.tick.status.lagging\"), false)",
        "source.sendSuccess(() -> Component.translatable(\"commands.tick.status.running\"), false)",
        "source.sendSuccess(() -> Component.translatable(\"commands.tick.query.rate.running\", tickRateString, busyTime, milliSecondsPerTickTarget), false)",
        "source.sendSuccess(() -> Component.translatable(\"commands.tick.query.percentiles\", p50, p95, p99, samples.length), false)",
        "boolean interrupted = source.getServer().tickRateManager().requestGameToSprint(time);",
        "Component.translatable(\"commands.tick.sprint.stop.success\"), true",
        "return freeze ? 1 : 0;",
        "boolean success = manager.stepGameIfPaused(advance);",
        "source.sendFailure(Component.translatable(\"commands.tick.step.fail\"));",
        "return 1;",
        "boolean success = manager.stopStepping();",
        "source.sendFailure(Component.translatable(\"commands.tick.step.stop.fail\"));",
        "boolean success = manager.stopSprinting();",
        "source.sendFailure(Component.translatable(\"commands.tick.sprint.stop.fail\"));",
    ] {
        assert!(
            TICK_COMMAND.contains(sentinel),
            "TickCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn advancement_command_source_matches_java_26_1_2() {
    const ADVANCEMENT_COMMAND: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/AdvancementCommands.java");

    for sentinel in [
        "Commands.literal(\"advancement\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"grant\")",
        "Commands.literal(\"revoke\")",
        "\"targets\", EntityArgument.players()",
        "Commands.literal(\"only\")",
        "Commands.literal(\"from\")",
        "Commands.literal(\"until\")",
        "Commands.literal(\"through\")",
        "Commands.literal(\"everything\")",
        "Commands.argument(\"criterion\", StringArgumentType.greedyString())",
        "ERROR_NO_ACTION_PERFORMED.create(",
        "ERROR_CRITERION_NOT_FOUND.create(Advancement.name(holder), criterion)",
        "source.sendSuccess(\n                  () -> Component.translatable(",
        "return count;",
        "player.getAdvancements().flushDirty(player, true);",
        "player.getAdvancements().flushDirty(player, false);",
        "progress.getRemainingCriteria()",
        "progress.getCompletedCriteria()",
        "Mode.ONLY",
        "Mode.FROM",
        "Mode.UNTIL",
        "Mode.THROUGH",
    ] {
        assert!(
            ADVANCEMENT_COMMAND.contains(sentinel),
            "AdvancementCommands.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn ban_ip_command_source_matches_java_26_1_2() {
    const BAN_IP_COMMAND: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/BanIpCommands.java");

    for sentinel in [
        "Commands.literal(\"ban-ip\").requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
        "Commands.argument(\"target\", StringArgumentType.word())",
        "Commands.argument(\"reason\", MessageArgument.message())",
        "InetAddresses.isInetAddress(target)",
        "source.getServer().getPlayerList().getPlayerByName(target)",
        "throw ERROR_INVALID_IP.create();",
        "IpBanList list = source.getServer().getPlayerList().getIpBans();",
        "if (list.isBanned(ip))",
        "throw ERROR_ALREADY_BANNED.create();",
        "source.getServer().getPlayerList().getPlayersWithAddress(ip)",
        "new IpBanListEntry(ip, null, source.getTextName(), null, reason == null ? null : reason.getString())",
        "source.sendSuccess(() -> Component.translatable(\"commands.banip.success\", ip, entry.getReasonMessage()), true)",
        "source.sendSuccess(() -> Component.translatable(\"commands.banip.info\", players.size(), EntitySelector.joinNames(players)), true)",
        "player.connection.disconnect(Component.translatable(\"multiplayer.disconnect.ip_banned\"));",
        "return players.size();",
    ] {
        assert!(
            BAN_IP_COMMAND.contains(sentinel),
            "BanIpCommands.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn ban_player_command_source_matches_java_26_1_2() {
    const BAN_PLAYER_COMMAND: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/BanPlayerCommands.java");

    for sentinel in [
        "Commands.literal(\"ban\").requires(Commands.hasPermission(Commands.LEVEL_ADMINS))",
        "Commands.argument(\"targets\", GameProfileArgument.gameProfile())",
        "Commands.argument(\"reason\", MessageArgument.message())",
        "GameProfileArgument.getGameProfiles(c, \"targets\")",
        "MessageArgument.getMessage(c, \"reason\")",
        "UserBanList list = source.getServer().getPlayerList().getBans();",
        "if (!list.isBanned(player))",
        "new UserBanListEntry(player, null, source.getTextName(), null, reason == null ? null : reason.getString())",
        "source.sendSuccess(() -> Component.translatable(\"commands.ban.success\", Component.literal(player.name()), entry.getReasonMessage()), true)",
        "source.getServer().getPlayerList().getPlayer(player.id())",
        "online.connection.disconnect(Component.translatable(\"multiplayer.disconnect.banned\"));",
        "throw ERROR_ALREADY_BANNED.create();",
        "return count;",
    ] {
        assert!(
            BAN_PLAYER_COMMAND.contains(sentinel),
            "BanPlayerCommands.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn bossbar_command_source_matches_java_26_1_2() {
    const BOSSBAR_COMMAND: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/BossBarCommands.java");

    for sentinel in [
        "\"bossbar\"",
        "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
        "Commands.literal(\"add\")",
        "Commands.literal(\"remove\")",
        "Commands.literal(\"list\")",
        "Commands.literal(\"set\")",
        "Commands.literal(\"get\")",
        "Commands.argument(\"id\", IdentifierArgument.id())",
        "Commands.argument(\"name\", ComponentArgument.textComponent(context))",
        "Commands.argument(\"value\", IntegerArgumentType.integer(0))",
        "Commands.argument(\"max\", IntegerArgumentType.integer(1))",
        "Commands.argument(\"visible\", BoolArgumentType.bool())",
        "Commands.argument(\"targets\", EntityArgument.players())",
        "EntityArgument.getOptionalPlayers(c, \"targets\")",
        "BossEvent.BossBarColor.PINK",
        "BossEvent.BossBarOverlay.NOTCHED_20",
        "throw ERROR_ALREADY_EXISTS.create(id.toString());",
        "throw ERROR_DOESNT_EXIST.create(id.toString());",
        "throw ERROR_NO_PLAYER_CHANGE.create();",
        "throw ERROR_NO_NAME_CHANGE.create();",
        "throw ERROR_NO_COLOR_CHANGE.create();",
        "throw ERROR_NO_STYLE_CHANGE.create();",
        "throw ERROR_NO_VALUE_CHANGE.create();",
        "throw ERROR_NO_MAX_CHANGE.create();",
        "throw ERROR_ALREADY_HIDDEN.create();",
        "throw ERROR_ALREADY_VISIBLE.create();",
        "Component.translatable(\"commands.bossbar.create.success\"",
        "Component.translatable(\"commands.bossbar.remove.success\"",
        "Component.translatable(\"commands.bossbar.list.bars.some\"",
        "Component.translatable(\"commands.bossbar.set.players.success.none\"",
        "\"commands.bossbar.get.players.some\"",
        "bossBar.removeAllPlayers();",
        "events.remove(bossBar);",
        "return events.getEvents().size();",
    ] {
        assert!(
            BOSSBAR_COMMAND.contains(sentinel),
            "BossBarCommands.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn clear_inventory_command_source_matches_java_26_1_2() {
    const CLEAR_COMMAND: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ClearInventoryCommands.java");

    for sentinel in [
        "Commands.literal(\"clear\")",
        "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
        "Collections.singleton(((CommandSourceStack)c.getSource()).getPlayerOrException())",
        "Commands.argument(\"targets\", EntityArgument.players())",
        "EntityArgument.getPlayers(c, \"targets\")",
        "Commands.argument(\"item\", ItemPredicateArgument.itemPredicate(context))",
        "ItemPredicateArgument.getItemPredicate(c, \"item\")",
        "Commands.argument(\"maxCount\", IntegerArgumentType.integer(0))",
        "IntegerArgumentType.getInteger(c, \"maxCount\")",
        "clearInventory(source, players, predicate, -1)",
        "player.getInventory().clearOrCountMatchingItems(predicate, maxCount, player.inventoryMenu.getCraftSlots())",
        "player.containerMenu.broadcastChanges();",
        "player.inventoryMenu.slotsChanged(player.getInventory());",
        "throw ERROR_SINGLE.create(players.iterator().next().getName());",
        "throw ERROR_MULTIPLE.create(players.size());",
        "\"commands.clear.test.single\"",
        "\"commands.clear.test.multiple\"",
        "\"commands.clear.success.single\"",
        "\"commands.clear.success.multiple\"",
        "return count;",
    ] {
        assert!(
            CLEAR_COMMAND.contains(sentinel),
            "ClearInventoryCommands.java is missing sentinel: {sentinel}"
        );
    }
}
