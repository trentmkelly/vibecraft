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
