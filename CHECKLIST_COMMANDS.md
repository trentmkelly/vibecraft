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

- [x] Implement `CommandBlockEntity` command execution on redstone pulse: permission-level check, `CommandSourceStack` with `CommandBlockEntity` context, output capture into `lastOutput` component — Java `CommandBlock.tick()`/`BaseCommandBlock.performCommand()` run command blocks from a gamemaster-level block source and update success/output state; `CommandBlockEntity::execute_from_context` models redstone-gated execution with command-block source position/level, permission success count, same-tick suppression, and tracked output capture; covered by `command_block_execution_uses_block_source_and_captures_output`
- [x] Implement command block modes: SEQUENCE (advances chain), AUTO (always active, runs every tick), REDSTONE (triggered by signal) — Java `CommandBlockEntity.Mode` and `CommandBlock.tick()` distinguish redstone, repeating/auto, and chain/sequence behavior; `command_block_tick()` and `execute_command_block_chain()` cover those mode gates in `command_block_tick_mode_and_edge_detection_match_vanilla_modes`, `auto_command_block_runs_each_tick_and_redstone_runs_on_leading_edge`, and `command_block_chain_executes_facing_order_and_respects_conditional_flag`
- [x] Implement conditional flag: block only executes if previous chain command succeeded (`successCount > 0`) — Java `CommandBlockEntity.markConditionMet()` reads the previous command block success count; Rust propagates `previous_success` through `mark_condition_met()`, `command_block_tick()`, and `execute_command_block_chain()`; covered by `command_block_chain_executes_facing_order_and_respects_conditional_flag`
- [x] Implement command block NBT: `Command`, `auto`, `conditionMet`, `LastOutput`, `SuccessCount`, `TrackOutput` — Java `CommandBlockEntity.saveAdditional/loadAdditional` preserve command block fields and `BaseCommandBlock` output flags; `CommandBlockEntity::save_additional()`/`load_additional()` cover those tags, plus `UpdateLastExecution`, `powered`, and tracking-disabled output suppression; covered by `command_block_entity_persists_base_fields_and_models_execution_gate`
- [x] Implement command block interaction: open GUI via `UseOnContext`, read/write command string via `ClientboundBlockEntityDataPacket` for 26.1.2 — Java `CommandBlock.useWithoutItem()` gates editor access on gamemaster blocks and `ServerboundSetCommandBlockPacket` carries command/mode/flag edits; Rust command blocks expose permission-gated editor update packets and apply client command/mode/flag edits, complementing the 26.1.2 set-command-block packet codec; covered by `command_block_editor_packet_and_client_update_require_permission` and `small_play_packets_round_trip_vanilla_codecs`
- [x] Implement facing-chain execution: a SEQUENCE/CHAIN command block executes the command block it faces after itself — Java `CommandBlock.executeChain()` walks facing chain command blocks until a non-sequence block or command-chain limit; `execute_command_block_chain()` walks command blocks by facing direction, executes contiguous sequence blocks, and propagates prior success into conditional gates; covered by `command_block_chain_executes_facing_order_and_respects_conditional_flag`
- [x] Add parity test: command block chain executes in facing order with conditional flag respected — `command_block_chain_executes_facing_order_and_respects_conditional_flag` covers ordered east-facing chains and conditional failure after a zero-success predecessor
- [x] Add parity test: AUTO command block runs every tick when powered, REDSTONE triggers once per leading edge — `auto_command_block_runs_each_tick_and_redstone_runs_on_leading_edge` covers repeated AUTO execution and REDSTONE rising-edge gating

## Command Block Minecart

- [x] Implement `CommandBlockMinecart`: per-entity `CommandBlockEntity`-like logic, `delay` counter (4-tick default), activate on powered activator rail — Java `MinecartCommandBlock.activateMinecart()` only runs on powered activator rails after `tickCount - lastActivated >= 4`, while Rust `CommandBlockMinecartState` keeps per-entity command/success/output state and enforces the same 4-tick activation gate; covered by `chest_spawner_and_command_minecarts_keep_special_server_hooks`
- [x] Implement command block minecart NBT round-trip: command, last output, track-output flag — Java `MinecartCommandBlock` delegates save/load to `BaseCommandBlock` for `Command`, `SuccessCount`, `TrackOutput`, and tracked `LastOutput`; Rust `CommandBlockMinecartState::save_additional()`/`load_additional()` round-trip those fields and suppress last output when tracking is disabled; covered by `chest_spawner_and_command_minecarts_keep_special_server_hooks`
- [x] Add parity test: command block minecart activates on powered activator rail at correct delay — `chest_spawner_and_command_minecarts_keep_special_server_hooks` covers the Java 4-tick boundary, unpowered no-op behavior, interaction permission gate, Searge output path, and NBT round-trip

## Functions

- [ ] Implement function loading from `data/*/function/**/*.mcfunction` files: parse each line as a command or comment (`#`)
- [ ] Implement function tags from `data/*/tags/function/*.json`: ordered list of function IDs, `#minecraft:tick` and `#minecraft:load` special tags
- [ ] Implement `#minecraft:tick` invocation: call all tagged functions every server tick
- [ ] Implement `#minecraft:load` invocation: call all tagged functions on world load and datapack reload
- [ ] Implement `/function <id>` command: execute a named function with the invoker's command source stack
- [x] Implement function-level return values: `/return` command sets the function result used by `/execute` return predicates: Java `ReturnCommand` gamemaster-gates integer return values, `return fail`, and `return run <command>` forwarding, while `return_command()` records success/failure/forwarded-command return events and `command_execution::ExecutionContextModel` discards same-depth continuations and propagates callbacks for `/execute` return-style result handling; covered by `return_command_requires_gamemaster_and_records_success_or_failure`, `return_command_records_forwarded_command_and_rejects_invalid_syntax`, and `return_value_discards_same_depth_continuations`
- [ ] Implement macro functions (26.1.2): `$` prefix lines in `.mcfunction` accept macro arguments, `$(variable)` substitution; `/function <id> with <entity|block|storage>` syntax — function loading already records `$` macro lines and `$(variable)` arguments; `/function <id> with entity|block|storage` now resolves modeled NBT sources into the existing macro instantiation path
- [x] Implement macro function instantiation for `/function <id> {compound}`: SNBT compound parsing, missing-argument errors, vanilla-style numeric/string argument stringification, and `$(variable)` substitution before queueing commands — Java `MacroFunction.instantiate()` requires a compound argument, stringifies primitive tags with root-locale decimal formatting, and substitutes macro entries before queueing through `FunctionCommand`; `instantiate_command_function()` and `function_command()` mirror that path, covered by `macro_function_instantiation_substitutes_compound_arguments_like_vanilla` and `function_command_queues_single_function_tags_and_arguments`
- [x] Implement function-argument type in Brigadier argument list for `/function` command autocompletion: Java `FunctionArgument.functions()` parses resource IDs and `#`-prefixed function tags, while `FunctionCommand.SUGGEST_FUNCTION` suggests function IDs plus `#tag` IDs; `command_tree::ArgumentParser::Function` validates both forms, `vanilla_like_tree()` exposes `/function <name>`, and `CommandTree::function_suggestions()` suggests known function IDs plus `#tag` IDs; covered by `function_argument_suggestions_use_function_and_tag_ids` plus the existing command-parity suggestion suite
- [ ] Implement scheduled functions via `/schedule function <id> <time> [append|replace]`: deferred single execution at game-time + delay
- [ ] Implement function-permission-level enforcement: functions run at op-level configured by `function-permission-level` property
- [ ] Implement function execution quota/fork limiting using `maxCommandChainLength` / `max_command_sequence_length`, `maxCommandForkCount` / `max_command_forks`, and vanilla queue overflow behavior
- [x] Add parity test: `#minecraft:tick` function called every tick, `#minecraft:load` called on reload — Java `ServerFunctionManager.tick()` runs load-tag functions once when `postReload` is set, then tick-tag functions every normal tick, and `replaceLibrary()` sets `postReload` after datapack reload; covered by `server_function_tick_queues_load_once_then_tick_when_running`
- [x] Add parity test: macro function variable substitution with entity/storage/block NBT source — Java `FunctionCommand` wires `with` through `DataCommands.SOURCE_PROVIDERS` and compound-tag macro instantiation; `function_with_entity_block_and_storage_sources_instantiates_macros` covers entity, block, storage, and missing-source failure with substituted macro commands.
- [x] Add parity test: command function execution stops at the vanilla game-rule sequence/fork limits and queue-overflow boundary — Java `ExecutionContext` stops when command quota reaches zero, rejects fork batches at `>= max_command_forks`, and trips queue overflow only when the pre-enqueue queue size is already over the max depth; covered by `execution_context_runs_commands_with_quota_and_result_callbacks`, `function_continuations_fork_with_limit_and_fallthrough_when_returning_empty`, and `execution_context_matches_vanilla_fork_limit_and_queue_overflow_boundary`

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
- [ ] Implement `/advancement`.
- [ ] Implement `/attribute`.
- [ ] Implement `/ban-ip`, `/banlist`, `/ban`, `/pardon`, `/pardon-ip`.
- [ ] Implement `/bossbar`.
- [ ] Implement `/chase`.
- [ ] Implement `/clear`.
- [ ] Implement `/clone`.
- [ ] Implement `/damage`.
- [ ] Implement `/datapack`.
- [ ] Implement `/deop` and `/op`.
- [ ] Implement `/debug`, `/debugconfig`, `/debugmobspawning`, `/debugpath`.
- [ ] Implement `/defaultgamemode`, `/difficulty`, `/gamemode`, `/gamerule`.
- [ ] Implement `/dialog`.
- [ ] Implement `/effect`.
- [ ] Implement `/emote`.
- [ ] Implement `/enchant`.
- [ ] Implement `/execute`.
- [ ] Implement `/experience`.
- [ ] Implement `/fetchprofile`.
- [ ] Implement `/fill` and `/fillbiome`.
- [ ] Implement `/forceload`.
- [ ] Implement `/function` and function scheduling.
- [ ] Implement `/give`.
- [ ] Implement `/help`.
- [ ] Implement `/item`.
- [ ] Implement `/jfr`.
- [ ] Implement `/kick`, `/kill`, `/list`.
- [ ] Implement `/locate`.
- [ ] Implement `/loot`.
- [ ] Implement `/msg`, `/teammsg`, `/say`, `/tellraw`.
- [ ] Implement `/particle`.
- [ ] Implement `/perf`.
- [ ] Implement `/place`.
- [ ] Implement `/playsound` and `/stopsound`.
- [ ] Implement `/publish`.
- [ ] Implement `/raid`.
- [ ] Implement `/random`.
- [ ] Implement `/recipe`.
- [ ] Implement `/reload`.
- [x] Implement `/return` — `return_command()` covers Java's gamemaster-gated `return <value>`, `return fail`, and `return run <command>` surfaces, records return/discard events for the execution model, and is covered by `return_command_requires_gamemaster_and_records_success_or_failure`, `return_command_records_forwarded_command_and_rejects_invalid_syntax`, and `return_value_discards_same_depth_continuations`.
- [ ] Implement `/ride`.
- [ ] Implement `/rotate`.
- [ ] Implement `/save-all`, `/save-off`, `/save-on`.
- [ ] Implement `/schedule`.
- [ ] Implement `/scoreboard`.
- [x] Implement `/seed` — dedicated-server command path uses gamemaster permission, reports `commands.seed.success` without admin broadcast, and returns Java's `(int) seed` result including long-to-int wrapping; covered by `seed_command_reports_level_seed_with_gamemaster_permission`.
- [ ] Implement `/serverpack`.
- [ ] Implement `/setblock`.
- [x] Implement `/setidletimeout` — admin-gated command accepts Java `IntegerArgumentType.integer(0)` minute values, rejects negative and above-`i32` inputs, updates the modeled player idle timeout, emits the enabled/disabled feedback keys, and returns the minute value; covered by `set_idle_timeout_command_updates_minutes_and_feedback`.
- [ ] Implement `/spawnpoint`, `/setworldspawn`.
- [ ] Implement `/spawn_armor_trims`.
- [ ] Implement `/spectate`.
- [ ] Implement `/spreadplayers`.
- [x] Implement `/stop` — owner-gated command dispatch records the server halt request, emits `commands.stop.stopping`, and returns success count 1, matching Java `StopCommand`; covered by `stop_command_requests_halt_and_requires_owner`.
- [ ] Implement `/stopwatch`.
- [ ] Implement `/summon`.
- [ ] Implement `/swing`.
- [ ] Implement `/tag`.
- [ ] Implement `/team`.
- [ ] Implement `/teleport`.
- [ ] Add Mineflayer command execution tests for teleporting a bot, validating its observed position correction, success feedback, and permission failures.
- [ ] Add Mineflayer offline-mode `/list` login-state tests that run `/list` from console and bot during login, after join, after duplicate replacement, and after disconnect to verify player counts and names match vanilla.
- [x] Add command-model `/list` login-state fallback coverage for empty, joined, duplicate-replacement, multi-player, and post-disconnect online counts while raw play command execution is not wired to `/list` yet — Java `ListPlayersCommand` returns `PlayerList.getPlayers().size()` for both `/list` and `/list uuids`, emits `commands.list.players`, and does not broadcast to admins; `list_command_tracks_login_replacement_and_disconnect_counts` covers empty, joined, duplicate-replacement, multi-player, UUID-list, and post-disconnect modeled online counts.
- [ ] Add Mineflayer offline-mode `/loot` command tests for `give`, `insert`, `spawn`, and `replace` targets using block, entity, chest, fishing, and custom loot tables, verifying inventory/window updates and dropped item entities against vanilla.
- [ ] Add command-model `/loot` fallback coverage for `give`, `insert`, `spawn`, and `replace` targets across custom loot table, block mine, entity kill, and fishing sources, including inventory/container slot updates and dropped-item event records while live Mineflayer inventory/entity visibility remains pending.
- [ ] Add Mineflayer command-suggestion tests that compare root command tree, argument suggestions, permission filtering, signed-command metadata, and tab-completion ordering against official `server.jar`.
- [x] Add raw 26.1.2 command-suggestion fallback coverage that sends an immediate play-state `/list` suggestion request, verifies the `list` suggestion response, and keeps the connection alive through the next keepalive while Mineflayer lacks target-protocol play support. — Java `ServerGamePacketListenerImpl.handleCustomCommandSuggestions` strips an optional leading slash, parses against the player command source, caps suggestions at 1000, and sends `ClientboundCommandSuggestionsPacket` with the request id; `raw_command_suggestion_response_keeps_stream_open_for_keepalive` sends raw `/li`, verifies packet id 15, request id 42, replacement start/length `1/2`, the `list` suggestion with no tooltip, and then decodes a following keepalive from the same play stream.
- [x] Add command-model command-suggestion fallback coverage for permission-filtered root command visibility and stable tab-completion ordering across all/moderator/gamemaster/admin/owner tiers while full Mineflayer-vs-vanilla signed metadata comparison remains pending — Java command roots are gated by each literal's `requires(Commands.hasPermission(...))` predicate before being sent/suggested to a player; `command_visibility_surface_filters_by_permission_tier_in_stable_order` covers modeled visible usages for all, moderator, gamemaster, admin, and owner tiers, including stable ordering boundaries and hidden/visible gated roots.
- [ ] Add Mineflayer offline-mode operator command smoke tests for `/op`, `/deop`, `/whitelist`, `/ban`, `/pardon`, `/gamemode`, `/tp`, `/give`, and `/effect`, verifying feedback, permission gates, and reconnect-visible state.
- [x] Add command-model operator smoke fallback coverage for `/op`, `/deop`, `/whitelist`, `/ban`, `/pardon`, `/gamemode`, `/tp`, `/give`, and `/effect`, including permission denial, feedback keys, disconnect side effects, and in-memory state changes while live bot command execution remains incomplete. — Java gates `/op`, `/deop`, `/whitelist`, `/ban`, and `/pardon` at admin level, gates `/gamemode`, `/tp`, `/give`, and `/effect` at gamemaster level, emits command-specific success feedback, and disconnects online banned players; `operator_command_smoke_matrix_covers_permissions_feedback_and_state_changes` covers denial, feedback keys, operator/whitelist/ban/pardon state, ban disconnect records, gamemode mutation, teleport position records, given inventory items, and active effects.
- [ ] Add Mineflayer offline-mode login-gated command tests that attempt `/list`, `/tell`, `/gamemode`, and `/tp` immediately after join and verify commands only run after the vanilla play-state readiness boundary.
- [x] Add play-session command-readiness fallback coverage that rejects chat command, signed chat command, chat, and command-suggestion packets while waiting for `player_loaded`, then accepts command suggestions after the loaded transition while live Mineflayer command execution remains incomplete. — Java 26.1.2 registers `ServerboundPlayerLoadedPacket` and `ServerGamePacketListenerImpl.handleAcceptPlayerLoad` calls `markClientLoaded()`; Rust’s play-session fallback models that readiness boundary by rejecting chat/command/suggestion packets in `WaitingForPlayerLoaded` and transitioning to `Playing` on `player_loaded`, with `command_like_packets_wait_for_player_loaded_boundary` covering all four rejected packet IDs plus a post-load command-suggestion accept.
- [ ] Add Mineflayer offline-mode command permission reload tests that edit `ops.json`, run `/op` and `/deop`, reconnect bots, and verify command tree deltas plus denied feedback match vanilla.
- [x] Add player-access reload fallback coverage proving hot-edited `ops.json`, `whitelist.json`, and `banned-players.json` are reflected by the same `PlayerAccess::load_from_dir` path used by console `reload`/`whitelist reload`, while live Mineflayer command-tree delta coverage remains pending. — Java `DedicatedPlayerList` loads `banned-players.json`, `ops.json`, and `whitelist.json`, and `WhitelistCommand.reload` calls `PlayerList.reloadWhiteList()` before kicking unlisted players; Rust console `reload`/`whitelist reload` replaces the shared `PlayerAccess` with `PlayerAccess::load_from_dir`, and `reload_from_dir_reflects_hot_edited_operator_whitelist_and_ban_files` proves hot-edited op level, whitelist membership, and player bans are reflected on reload.
- [ ] Add Mineflayer offline-mode command-before-ready tests that attempt chat commands during login/configuration/play transition boundaries and verify vanilla-compatible rejection, queuing, or disconnect behavior.
- [x] Add raw 26.1.2 command-before-ready fallback coverage that sends command-suggestion and chat-shaped packets during login/configuration before play readiness and verifies vanilla-compatible rejection/close behavior while Mineflayer lacks target-protocol play support. — Java 26.1.2 `LoginProtocols` accepts only `ServerboundHelloPacket` before the login switch, `ConfigurationProtocols` registers configuration-specific packet IDs before play, and `ServerGamePacketListenerImpl.handleAcceptPlayerLoad` marks the play client loaded; Rust now factors the login-hello read for direct raw testing, with `raw_play_command_packets_reject_before_login_hello`, `raw_play_command_packets_reject_during_configuration_wait`, and `command_like_packets_wait_for_player_loaded_boundary` covering play-shaped command suggestion/chat packets before login, during configuration, and before `player_loaded`.
- [ ] Add Mineflayer offline-mode command-result consistency tests that run commands from console, op bot, non-op bot, command block, and function context, then compare success count, feedback visibility, and player-observed side effects.
- [x] Add command-model command-result fallback coverage for non-op denial, op self-target side effects, repeated no-op success counts, console explicit-target execution, feedback keys, and admin broadcast flags while live bot command execution remains incomplete — Java `GameModeCommand` requires gamemaster permission, returns the number of players whose mode changed, emits self/other success feedback only for changes, and broadcasts admin-visible feedback; `command_results_keep_source_permissions_feedback_and_side_effects_consistent` covers modeled denial, self-target mutation, repeated no-op `0`, explicit console target execution, feedback keys, and admin broadcast flags.
- [ ] Implement `/tick`.
- [ ] Implement `/time`.
- [ ] Implement `/title`.
- [ ] Implement `/transfer`.
- [ ] Implement `/trigger`.
- [x] Implement `/version` — Java `VersionCommand` emits `commands.version.header`, dumps `SharedConstants.getCurrentVersion()` fields, and returns success count `1`; Rust `VersionInfo::CURRENT_26_1_2` models the 26.1.2 id, data/protocol, pack versions, build time, and stable flag, and `/version` now returns Java's success count; covered by `version_command_reports_26_1_2_metadata`.
- [ ] Implement `/warden_spawn_tracker`.
- [ ] Implement `/waypoint`.
- [ ] Implement `/weather`.
- [ ] Implement `/whitelist`.
- [ ] Implement `/worldborder`.
- [ ] Implement command blocks, command block minecarts, functions, tags, macro/function arguments, and scheduled functions. — detailed command-block, command-block-minecart, function/tag, macro-argument, and scheduled-function rows above are now implemented and covered by focused parity tests
- [ ] Validate command parse trees and results against vanilla command dumps and scripted execution tests.
- [ ] Add Mineflayer chat and command tests covering signed/unsigned chat fallback, system messages, command feedback, suggestions, and tab completion in offline mode.
- [x] Add command-model chat/command fallback coverage for public `/say`, `/me`, private `/tell`, `/teammsg`, `/tellraw`, feedback keys, permission denial, and target routing while full Mineflayer signed/unsigned/system-message coverage remains pending. — Java `SayCommand` and `TellRawCommand` require gamemaster permission, while `EmoteCommands`, `MsgCommand`, and `TeamMsgCommand` are player-accessible; Java returns message target counts and routes private/team/raw messages to selected receivers. `chat_command_model_covers_public_private_team_raw_and_permission_feedback` covers permission denial, feedback keys, success counts, public/private/team/raw target selection, and `/tellraw` payload preservation, with `chat_broadcast_routes_player_filtered_private_team_and_tellraw_messages` covering presentation routing.
