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
- [x] Implement command block modes: SEQUENCE (advances chain), AUTO (always active, runs every tick), REDSTONE (triggered by signal)
- [x] Implement conditional flag: block only executes if previous chain command succeeded (`successCount > 0`)
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

- [x] Implement function loading from `data/*/function/**/*.mcfunction` files: parse each line as a command or comment (`#`)
- [x] Implement function tags from `data/*/tags/function/*.json`: ordered list of function IDs, `#minecraft:tick` and `#minecraft:load` special tags
- [x] Implement `#minecraft:tick` invocation: call all tagged functions every server tick
- [x] Implement `#minecraft:load` invocation: call all tagged functions on world load and datapack reload
- [x] Implement `/function <id>` command: execute a named function with the invoker's command source stack
- [ ] Implement function-level return values: `/return` command sets the function result used by `/execute` return predicates
- [ ] Implement macro functions (26.1.2): `$` prefix lines in `.mcfunction` accept macro arguments, `$(variable)` substitution; `/function <id> with <entity|block|storage>` syntax
- [ ] Implement function-argument type in Brigadier argument list for `/function` command autocompletion
- [x] Implement scheduled functions via `/schedule function <id> <time> [append|replace]`: deferred single execution at game-time + delay
- [x] Implement function-permission-level enforcement: functions run at op-level configured by `function-permission-level` property
- [ ] Implement function execution quota/fork limiting using `maxCommandChainLength` / `max_command_sequence_length`, `maxCommandForkCount` / `max_command_forks`, and vanilla queue overflow behavior
- [ ] Add parity test: `#minecraft:tick` function called every tick, `#minecraft:load` called on reload
- [ ] Add parity test: macro function variable substitution with entity/storage/block NBT source
- [ ] Add parity test: command function execution stops at the vanilla game-rule sequence/fork limits and queue-overflow boundary

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

## Migrated From Main Checklist: Commands And Functions

- [ ] Implement Brigadier-compatible command tree, parsing, suggestions, redirects, forks, permissions, and signed arguments.
- [ ] Implement command source stack, execution context, result propagation, `/execute` semantics, return values, and function continuation.
- [ ] Implement selector parsing, entity predicates, score predicates, NBT predicates, range predicates, sort/order, limits, and current entity context.
- [ ] Implement all server command classes found under `net/minecraft/server/commands`.
- [x] Implement `/advancement`.
- [x] Implement `/attribute`.
- [x] Implement `/ban-ip`, `/banlist`, `/ban`, `/pardon`, `/pardon-ip`.
- [x] Implement `/bossbar`.
- [x] Implement `/chase`.
- [x] Implement `/clear`.
- [x] Implement `/clone`.
- [x] Implement `/damage`.
- [x] Implement `/datapack`.
- [x] Implement `/deop` and `/op`.
- [x] Implement `/debug`, `/debugconfig`, `/debugmobspawning`, `/debugpath`.
- [x] Implement `/defaultgamemode`, `/difficulty`, `/gamemode`, `/gamerule`.
- [x] Implement `/dialog`.
- [x] Implement `/effect`.
- [x] Implement `/emote`.
- [x] Implement `/enchant`.
- [x] Implement `/execute`.
- [x] Implement `/experience`.
- [x] Implement `/fetchprofile`.
- [x] Implement `/fill` and `/fillbiome`.
- [x] Implement `/forceload`.
- [x] Implement `/function` and function scheduling.
- [x] Implement `/give`.
- [x] Implement `/help`.
- [x] Implement `/item`.
- [x] Implement `/jfr`.
- [x] Implement `/kick`, `/kill`, `/list`.
- [x] Implement `/locate`.
- [x] Implement `/loot`.
- [x] Implement `/msg`, `/teammsg`, `/say`, `/tellraw`.
- [x] Implement `/particle`.
- [x] Implement `/perf`.
- [x] Implement `/place`.
- [x] Implement `/playsound` and `/stopsound`.
- [x] Implement `/publish`.
- [x] Implement `/raid`.
- [x] Implement `/random`.
- [x] Implement `/recipe`.
- [x] Implement `/reload`.
- [x] Implement `/return`.
- [x] Implement `/ride`.
- [x] Implement `/rotate`.
- [x] Implement `/save-all`, `/save-off`, `/save-on`.
- [x] Implement `/schedule`.
- [x] Implement `/scoreboard`.
- [x] Implement `/seed`.
- [x] Implement `/serverpack`.
- [x] Implement `/setblock`.
- [x] Implement `/setidletimeout`.
- [x] Implement `/spawnpoint`, `/setworldspawn`.
- [x] Implement `/spawn_armor_trims`.
- [x] Implement `/spectate`.
- [x] Implement `/spreadplayers`.
- [x] Implement `/stop`.
- [x] Implement `/stopwatch`.
- [x] Implement `/summon`.
- [x] Implement `/swing`.
- [x] Implement `/tag`.
- [x] Implement `/team`.
- [x] Implement `/teleport`.
- [ ] Add Mineflayer command execution tests for teleporting a bot, validating its observed position correction, success feedback, and permission failures.
- [ ] Add Mineflayer offline-mode `/list` login-state tests that run `/list` from console and bot during login, after join, after duplicate replacement, and after disconnect to verify player counts and names match vanilla.
- [x] Add command-model `/list` login-state fallback coverage for empty, joined, duplicate-replacement, multi-player, and post-disconnect online counts while raw play command execution is not wired to `/list` yet.
- [ ] Add Mineflayer offline-mode `/loot` command tests for `give`, `insert`, `spawn`, and `replace` targets using block, entity, chest, fishing, and custom loot tables, verifying inventory/window updates and dropped item entities against vanilla.
- [x] Add command-model `/loot` fallback coverage for `give`, `insert`, `spawn`, and `replace` targets across custom loot table, block mine, entity kill, and fishing sources, including inventory/container slot updates and dropped-item event records while live Mineflayer inventory/entity visibility remains pending.
- [ ] Add Mineflayer command-suggestion tests that compare root command tree, argument suggestions, permission filtering, signed-command metadata, and tab-completion ordering against official `server.jar`.
- [x] Add raw 26.1.2 command-suggestion fallback coverage that sends an immediate play-state `/list` suggestion request, verifies the `list` suggestion response, and keeps the connection alive through the next keepalive while Mineflayer lacks target-protocol play support.
- [x] Add command-model command-suggestion fallback coverage for permission-filtered root command visibility and stable tab-completion ordering across all/moderator/gamemaster/admin/owner tiers while full Mineflayer-vs-vanilla signed metadata comparison remains pending.
- [ ] Add Mineflayer offline-mode operator command smoke tests for `/op`, `/deop`, `/whitelist`, `/ban`, `/pardon`, `/gamemode`, `/tp`, `/give`, and `/effect`, verifying feedback, permission gates, and reconnect-visible state.
- [x] Add command-model operator smoke fallback coverage for `/op`, `/deop`, `/whitelist`, `/ban`, `/pardon`, `/gamemode`, `/tp`, `/give`, and `/effect`, including permission denial, feedback keys, disconnect side effects, and in-memory state changes while live bot command execution remains incomplete.
- [ ] Add Mineflayer offline-mode login-gated command tests that attempt `/list`, `/tell`, `/gamemode`, and `/tp` immediately after join and verify commands only run after the vanilla play-state readiness boundary.
- [x] Add play-session command-readiness fallback coverage that rejects chat command, signed chat command, chat, and command-suggestion packets while waiting for `player_loaded`, then accepts command suggestions after the loaded transition while live Mineflayer command execution remains incomplete.
- [ ] Add Mineflayer offline-mode command permission reload tests that edit `ops.json`, run `/op` and `/deop`, reconnect bots, and verify command tree deltas plus denied feedback match vanilla.
- [x] Add player-access reload fallback coverage proving hot-edited `ops.json`, `whitelist.json`, and `banned-players.json` are reflected by the same `PlayerAccess::load_from_dir` path used by console `reload`/`whitelist reload`, while live Mineflayer command-tree delta coverage remains pending.
- [ ] Add Mineflayer offline-mode command-before-ready tests that attempt chat commands during login/configuration/play transition boundaries and verify vanilla-compatible rejection, queuing, or disconnect behavior.
- [x] Add raw 26.1.2 command-before-ready fallback coverage that sends command-suggestion and chat-shaped packets during login/configuration before play readiness and verifies vanilla-compatible rejection/close behavior while Mineflayer lacks target-protocol play support.
- [ ] Add Mineflayer offline-mode command-result consistency tests that run commands from console, op bot, non-op bot, command block, and function context, then compare success count, feedback visibility, and player-observed side effects.
- [x] Add command-model command-result fallback coverage for non-op denial, op self-target side effects, repeated no-op success counts, console explicit-target execution, feedback keys, and admin broadcast flags while live bot command execution remains incomplete.
- [x] Implement `/tick`.
- [x] Implement `/time`.
- [x] Implement `/title`.
- [x] Implement `/transfer`.
- [x] Implement `/trigger`.
- [x] Implement `/version`.
- [x] Implement `/warden_spawn_tracker`.
- [x] Implement `/waypoint`.
- [x] Implement `/weather`.
- [x] Implement `/whitelist`.
- [x] Implement `/worldborder`.
- [ ] Implement command blocks, command block minecarts, functions, tags, macro/function arguments, and scheduled functions.
- [ ] Validate command parse trees and results against vanilla command dumps and scripted execution tests.
- [ ] Add Mineflayer chat and command tests covering signed/unsigned chat fallback, system messages, command feedback, suggestions, and tab completion in offline mode.
- [x] Add command-model chat/command fallback coverage for public `/say`, `/me`, private `/tell`, `/teammsg`, `/tellraw`, feedback keys, permission denial, and target routing while full Mineflayer signed/unsigned/system-message coverage remains pending.
