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
