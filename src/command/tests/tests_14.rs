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
