# Server Runtime and Operations Checklist

Bootstrap, process lifecycle, configuration, runtime scheduling, and operator-facing services moved out of the top-level checklist.

## Migrated From Main Checklist: Bootstrap And Process Lifecycle

- [x] Implement command-line parsing equivalent to `net.minecraft.server.Main`.
- [x] Support `--nogui`.
- [x] Support `--initSettings`.
- [x] Support `--demo`.
- [x] Support `--bonusChest`.
- [x] Support `--forceUpgrade`.
- [x] Support `--eraseCache`.
- [x] Support `--recreateRegionFiles`.
- [x] Support `--safeMode`.
- [x] Support `--help`.
- [x] Support `--universe`.
- [x] Support `--world`.
- [x] Support `--port`.
- [x] Support `--serverId`.
- [x] Support `--jfrProfile` with an equivalent profiling story or documented no-op.
- [x] Support `--pidFile`.
- [x] Initialize `server.properties` and `eula.txt` before normal startup.
- [x] Refuse startup until `eula=true`.
- [ ] Write logs matching vanilla lifecycle milestones closely enough for operators and test tooling.
- [x] Install process-level crash and panic handlers.
- [x] Emit crash reports with useful environment, thread, and world state.
- [ ] Implement graceful shutdown from console, signal, stop command, and JVM-style shutdown hook equivalents.
- [ ] Save all worlds, players, scoreboards, advancements, raids, maps, and server state during shutdown.
- [ ] Add a Mineflayer shutdown/restart test that joins in offline mode, mutates visible player state, stops the server cleanly, restarts, and verifies reconnect plus persisted position, inventory, and stats.
- [ ] Add a Mineflayer crash-after-login recovery test that joins in offline mode, waits for the first save boundary, terminates the process, restarts, and verifies vanilla-compatible reconnect behavior and no corrupted player/world files.
- [ ] Add a Mineflayer startup-race test that begins connecting during bootstrap in offline mode and verifies the bot is either held until readiness or rejected with the same timing/message as official `server.jar`.
- [ ] Add a Mineflayer first-login bootstrap test that starts from only `eula=true`, joins in offline mode, and verifies generated config/world/player artifacts appear before the bot reaches play state in vanilla order.
- [ ] Add a Mineflayer offline-mode failed-start cleanup test that attempts login when startup is refused by missing EULA, bad world metadata, or invalid properties, then verifies no partial player files or stale bot sessions remain.
- [ ] Add a Mineflayer offline-mode lifecycle artifact test that records startup logs, readiness signal, first accepted login tick, shutdown reason, and post-exit file flushes for vanilla and RustCraft.
- [ ] Add a Mineflayer offline-mode interrupted-bootstrap test that kills the server while a bot is mid-login and verifies restart does not retain half-created players, sockets, or world locks.
- [ ] Add a Mineflayer offline-mode EULA refusal test that attempts login before `eula=true`, verifies vanilla-compatible connection refusal/log messages, then accepts EULA and verifies the same bot can join without stale state.
- [x] Handle corrupted world metadata with vanilla-compatible refusal paths.
- [x] Detect incompatible world versions before loading.
- [x] Provide safe-mode datapack loading.
- [ ] Support world data upgrade and region recreation workflow.
- [x] Implement server watchdog behavior controlled by `max-tick-time`.

## Migrated From Main Checklist: Dedicated Server Configuration

- [x] Parse and write `server.properties` with vanilla defaults.
- [x] Preserve unknown property keys when rewriting configuration.
- [x] Implement `online-mode`.
- [ ] Add a Mineflayer offline-mode login test using a generated bot profile and default `server.properties`.
- [ ] Add a Mineflayer offline-mode login test matrix covering `online-mode=false`, `enforce-secure-profile=false`, default port selection, and generated world directory setup.
- [ ] Add a Mineflayer offline-mode login test matrix covering `max-players`, `enable-status`, `hide-online-players`, `network-compression-threshold`, `player-idle-timeout`, and `white-list` interactions during first join.
- [ ] Add a Mineflayer offline-mode login test matrix for `level-name`, `level-seed`, `gamemode`, `difficulty`, `hardcore`, `force-gamemode`, `allow-flight`, and `spawn-protection`, verifying the first joined bot observes vanilla-compatible initial state.
- [ ] Add a Mineflayer offline-mode single-property login bisect test that toggles one `server.properties` key per run and reports the first key that changes login, configuration, spawn, or disconnect behavior.
- [ ] Add a Mineflayer offline-mode generated-properties test that starts with deleted or partial `server.properties`, accepts the generated defaults, then verifies the first bot can join without hand-edited configuration.
- [ ] Add a Mineflayer offline-mode property-minimization test that removes optional keys one at a time and verifies vanilla-compatible defaulting, warnings, and first-login behavior.
- [ ] Add a Mineflayer offline-mode negative configuration test covering accidental `online-mode=true`, secure-profile enforcement, invalid `server-ip`, occupied ports, and malformed properties with vanilla-compatible refusal or kick messages.
- [ ] Add a Mineflayer offline-mode login test that starts from an empty working directory and verifies `eula.txt`, `server.properties`, world folders, and first successful bot join are produced in vanilla-compatible order.
- [ ] Add a Mineflayer offline-mode login property-roundtrip test that rewrites `server.properties`, restarts, reconnects the same generated bot, and verifies vanilla-compatible UUID/profile reuse and observed config changes.
- [ ] Add a Mineflayer offline-mode bind-address test that verifies bots can connect through `localhost`, explicit `server-ip`, randomized ports, and rejected addresses with vanilla-compatible socket or kick behavior.
- [ ] Add a Mineflayer offline-mode compression-property login test that verifies first join succeeds with `network-compression-threshold=-1`, `0`, small positive values, and the vanilla default while preserving packet ordering.
- [x] Implement `prevent-proxy-connections`.
- [x] Implement `server-ip`.
- [x] Implement mutable `allow-flight`.
- [x] Implement mutable `motd`.
- [x] Implement `enable-code-of-conduct`.
- [x] Implement `bug-report-link`.
- [x] Implement mutable `force-gamemode`.
- [x] Implement mutable `enforce-whitelist`.
- [x] Implement mutable `difficulty`.
- [x] Implement mutable `gamemode`.
- [x] Implement `level-name`.
- [x] Implement `server-port`.
- [x] Implement management server settings: enabled, host, port, secret, TLS, keystore, password, allowed origins.
- [ ] Implement legacy `announce-player-achievements` migration behavior.
- [x] Implement `enable-query` and `query.port`.
- [x] Implement `enable-rcon`, `rcon.port`, and `rcon.password`.
- [x] Implement `hardcore`.
- [x] Implement `use-native-transport` or document the equivalent transport decision.
- [x] Implement mutable `spawn-protection`.
- [x] Implement mutable `op-permission-level`.
- [x] Implement `function-permission-level`.
- [x] Implement `max-tick-time`.
- [x] Implement `max-chained-neighbor-updates`.
- [x] Implement `rate-limit`.
- [x] Implement mutable `view-distance`.
- [x] Implement mutable `simulation-distance`.
- [x] Implement mutable `max-players`.
- [ ] Add a Mineflayer max-player enforcement test that fills available slots in offline mode and verifies the extra bot receives the vanilla full-server disconnect message.
- [x] Implement `network-compression-threshold`.
- [x] Implement `broadcast-rcon-to-ops`.
- [x] Implement `broadcast-console-to-ops`.
- [x] Implement clamped `max-world-size`.
- [ ] Implement `sync-chunk-writes`.
- [x] Implement `region-file-compression` with at least deflate parity.
- [ ] Implement `enable-jmx-monitoring` or equivalent documented metrics export.
- [x] Implement mutable `enable-status`.
- [x] Implement mutable `hide-online-players`.
- [x] Implement mutable `entity-broadcast-range-percentage`.
- [x] Implement `text-filtering-config` and `text-filtering-version`.
- [x] Implement server resource pack fields: id, URL, SHA-1, legacy hash, required flag, prompt component.
- [x] Implement initial datapack enabled/disabled pack lists.
- [x] Implement mutable `player-idle-timeout`.
- [x] Implement mutable `status-heartbeat-interval`.
- [x] Implement mutable `white-list`.
- [ ] Add Mineflayer whitelist tests for offline-mode allow, deny, runtime `/whitelist reload`, and `enforce-whitelist` toggles.
- [ ] Add Mineflayer mutable-property tests that change MOTD, difficulty, gamemode, view-distance, simulation-distance, idle timeout, and whitelist settings at runtime, then verify existing and reconnecting offline-mode bots observe vanilla-compatible state.
- [ ] Add a Mineflayer offline-mode configuration reload test that edits `server.properties`, runs the vanilla-equivalent reload path where supported, reconnects the bot, and verifies which properties do and do not take effect without restart.
- [ ] Add a Mineflayer offline-mode secure-profile toggle test that verifies `enforce-secure-profile=false` never blocks generated offline bots and that `true` matches official `server.jar` behavior for unsigned Mineflayer clients.
- [x] Implement `enforce-secure-profile`.
- [x] Implement `log-ips`.
- [x] Implement mutable `pause-when-empty-seconds`.
- [ ] Implement `level-seed`, `generate-structures`, `generator-settings`, and `level-type`.
- [x] Implement mutable `accepts-transfers`.

## Migrated From Main Checklist: Core Runtime Model

- [x] Implement a main server thread with a deterministic 20 TPS tick loop.
- [x] Implement tick budget accounting and drift handling.
- [x] Implement pause-when-empty behavior.
- [x] Implement scheduled task execution on the server thread.
- [x] Implement async task pools for IO, chunk generation, resource reloads, and profile/session services.
- [x] Implement thread-safety rules equivalent to vanilla server access restrictions.
- [x] Implement tick-rate management, freeze/step/sprint behavior, and tick command hooks.
- [x] Implement profiling hooks equivalent to debug/perf/JFR command output where practical.
- [x] Implement bandwidth, tick time, packet, and debug sample collection.
- [x] Implement crash-safe autosave cadence.
- [x] Implement forced save, save-off, and save-on semantics.
- [ ] Add Mineflayer tick-loop stability tests that keep an offline-mode bot connected through pause-when-empty transitions, autosave, `/save-off`, `/save-on`, and `/save-all`, verifying keepalives and visible state do not stall.

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Operational File And API Coverage

- [ ] Implement `eula.txt`, `server.properties`, `ops.json`, `whitelist.json`, `banned-players.json`, `banned-ips.json`, `usercache.json`, `session.lock`, `level.dat`, `level.dat_old`, region files, entity region files, POI files, playerdata, advancements, stats, icon, crash reports, logs, debug output, and generated reports.
- [x] Implement RCON authentication, command execution, response fragmentation, broadcast behavior, and failure modes.
- [x] Implement query protocol basic and full stat responses with plugin list behavior equivalent to vanilla.
- [x] Implement JSON-RPC management methods, schemas, notifications, player DTOs, reference utilities, pending request tracking, origin checks, TLS settings, and shutdown behavior.
- [x] Implement chase server/client debug feature or document it as intentionally unsupported with no impact on vanilla clients.
- [ ] Implement game test framework hooks enough for parity test execution or document a replacement harness.
- [ ] Implement generated data/report tooling used to compare registries, tags, commands, packs, and worldgen definitions.
