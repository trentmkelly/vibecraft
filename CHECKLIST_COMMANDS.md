# Commands and Functions Checklist

## Relevant Java Source Files

- `decompiled-server-26.1.2/net/minecraft/server/commands/` — all server command implementations
- `decompiled-server-26.1.2/net/minecraft/server/commands/FunctionCommand.java` — `/function`
- `decompiled-server-26.1.2/net/minecraft/server/commands/ScheduleCommand.java` — `/schedule`
- `decompiled-server-26.1.2/net/minecraft/commands/CommandSource.java` — command source interface
- `decompiled-server-26.1.2/net/minecraft/commands/CommandSourceStack.java` — execution context
- `decompiled-server-26.1.2/net/minecraft/commands/ExecutionCommandSource.java` — fork/return context
- `decompiled-server-26.1.2/net/minecraft/world/level/storage/loot/functions/` — loot functions (used by `/loot`)
- `decompiled-server-26.1.2/net/minecraft/world/level/block/entity/CommandBlockEntity.java` — command block
- `decompiled-server-26.1.2/net/minecraft/world/entity/vehicle/CommandBlockMinecart.java` — command block minecart
- `decompiled-server-26.1.2/net/minecraft/commands/functions/` — function system (if present in 26.1.2)
- `RustCraft/src/command.rs` — RustCraft command dispatcher
- `RustCraft/src/command_execution.rs` — RustCraft command execution
- `RustCraft/src/command_parity.rs` — RustCraft command parity tests
- `RustCraft/src/command_feedback.rs` — RustCraft command feedback
- `RustCraft/src/command_tree.rs` — RustCraft command tree

## Command Blocks

- [ ] Implement `CommandBlockEntity` command execution on redstone pulse: permission-level check, `CommandSourceStack` with `CommandBlockEntity` context, output capture into `lastOutput` component
- [ ] Implement command block modes: SEQUENCE (advances chain), AUTO (always active, runs every tick), REDSTONE (triggered by signal)
- [ ] Implement conditional flag: block only executes if previous chain command succeeded (`successCount > 0`)
- [ ] Implement command block NBT: `Command`, `auto`, `conditionMet`, `LastOutput`, `SuccessCount`, `TrackOutput`
- [ ] Implement command block interaction: open GUI via `UseOnContext`, read/write command string via `ClientboundBlockEntityDataPacket` for 26.1.2
- [ ] Implement facing-chain execution: a SEQUENCE/CHAIN command block executes the command block it faces after itself
- [ ] Add parity test: command block chain executes in facing order with conditional flag respected
- [ ] Add parity test: AUTO command block runs every tick when powered, REDSTONE triggers once per leading edge

## Command Block Minecart

- [ ] Implement `CommandBlockMinecart`: per-entity `CommandBlockEntity`-like logic, `delay` counter (4-tick default), activate on powered activator rail
- [ ] Implement command block minecart NBT round-trip: command, last output, track-output flag
- [ ] Add parity test: command block minecart activates on powered activator rail at correct delay

## Functions

- [ ] Implement function loading from `data/*/function/**/*.mcfunction` files: parse each line as a command or comment (`#`)
- [ ] Implement function tags from `data/*/tags/function/*.json`: ordered list of function IDs, `#minecraft:tick` and `#minecraft:load` special tags
- [ ] Implement `#minecraft:tick` invocation: call all tagged functions every server tick
- [ ] Implement `#minecraft:load` invocation: call all tagged functions on world load and datapack reload
- [ ] Implement `/function <id>` command: execute a named function with the invoker's command source stack
- [ ] Implement function-level return values: `/return` command sets the function result used by `/execute` return predicates
- [ ] Implement macro functions (26.1.2): `$` prefix lines in `.mcfunction` accept macro arguments, `$(variable)` substitution; `/function <id> with <entity|block|storage>` syntax
- [ ] Implement function-argument type in Brigadier argument list for `/function` command autocompletion
- [ ] Implement scheduled functions via `/schedule function <id> <time> [append|replace]`: deferred single execution at game-time + delay
- [ ] Implement function-permission-level enforcement: functions run at op-level configured by `function-permission-level` property
- [ ] Implement nested function execution depth limit (vanilla limit: 32 nesting levels) with error on overflow
- [ ] Add parity test: `#minecraft:tick` function called every tick, `#minecraft:load` called on reload
- [ ] Add parity test: macro function variable substitution with entity/storage/block NBT source
- [ ] Add parity test: nested function depth limit error at vanilla boundary

## Command Testing (Mineflayer / Integration)

- [ ] Add Mineflayer command execution tests: teleport a bot with `/tp`, verify bot observes its new position, verify success feedback, verify permission failure for non-op bot
- [ ] Add Mineflayer offline-mode `/list` login-state tests: run `/list` from console and from bot during login, after join, after duplicate replacement, and after disconnect; verify player counts and names match vanilla
- [ ] Add Mineflayer offline-mode `/loot` command tests: `give`, `insert`, `spawn`, and `replace` targets using block, entity, chest, fishing, and custom loot tables; verify inventory/window updates and dropped item entities against official `server.jar`
- [ ] Add Mineflayer command-suggestion tests: compare root command tree, argument suggestions, permission filtering, signed-command metadata, and tab-completion ordering against official `server.jar`
- [ ] Add Mineflayer offline-mode operator command smoke tests for `/op`, `/deop`, `/whitelist`, `/ban`, `/pardon`, `/gamemode`, `/tp`, `/give`, and `/effect`; verify feedback, permission gates, and reconnect-visible state
- [ ] Add Mineflayer offline-mode login-gated command tests: attempt `/list`, `/tell`, `/gamemode`, and `/tp` immediately after join; verify commands only run after vanilla play-state readiness boundary
- [ ] Add Mineflayer offline-mode command permission reload tests: edit `ops.json`, run `/op` and `/deop`, reconnect bots, verify command-tree deltas and denied feedback match vanilla
- [ ] Add Mineflayer offline-mode command-before-ready tests: attempt chat commands during login/configuration/play transition boundaries; verify vanilla-compatible rejection, queuing, or disconnect behavior
- [ ] Add Mineflayer offline-mode command-result consistency tests: run commands from console, op bot, non-op bot, command block, and function context; compare success count, feedback visibility, and player-observed side effects
- [ ] Add Mineflayer chat and command tests: signed/unsigned chat fallback, system messages, command feedback, suggestions, and tab completion in offline mode

## Command Parse Tree Validation

- [ ] Validate command parse tree dump against vanilla `/brigadier dump` or equivalent: all root literals, argument types, redirects, and permission-level gates match
- [ ] Validate signed argument metadata: which commands require signed arguments in online mode matches vanilla
- [ ] Validate tab-completion ordering: alphabetical or vanilla-order for all command literals and subcommands
