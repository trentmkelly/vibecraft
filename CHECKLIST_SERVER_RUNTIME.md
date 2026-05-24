# Server Runtime and Operations Checklist

Bootstrap, process lifecycle, configuration, runtime scheduling, and operator-facing services moved out of the top-level checklist.

## Migrated From Main Checklist: Bootstrap And Process Lifecycle

- [ ] Implement command-line parsing equivalent to `net.minecraft.server.Main`.
- [x] Support `--nogui`. — `CliOptions` accepts both Java `Main`'s `--nogui` option and legacy positional `nogui`, RustCraft is intentionally headless, and `parses_legacy_positional_nogui_like_java_main` covers the positional path.
- [x] Support `--initSettings`. — `CliOptions` parses Java `Main`'s flag and `init_settings_creates_properties_and_eula_before_startup` verifies RustCraft creates `server.properties` plus `eula.txt`, logs the initialized paths, and exits before normal startup.
- [x] Support `--demo`. — `CliOptions` parses Java `Main`'s flag, startup feeds it into `WorldOptions::from_server_inputs`, and `world_options_follow_dedicated_server_and_demo_rules` verifies demo mode selects the vanilla demo seed, structures, and bonus chest behavior.
- [x] Support `--bonusChest`. — `CliOptions` parses Java `Main`'s flag, startup feeds it into `WorldOptions::from_server_inputs`, and `world_options_follow_dedicated_server_and_demo_rules` verifies the non-demo branch enables bonus-chest generation like `worldOptions.withBonusChest(true)`.
- [ ] Support `--forceUpgrade`.
- [ ] Support `--eraseCache`.
- [ ] Support `--recreateRegionFiles`.
- [x] Support `--safeMode`. — `CliOptions` parses Java `Main`'s flag, startup passes it into `PackConfigureOptions`, and `safe_mode_selects_only_vanilla_and_does_not_disable_world_packs` verifies safe mode selects only the vanilla pack.
- [x] Support `--help`. — `CliOptions` parses Java `Main`'s help flag, `main()` prints usage and returns before startup, and `help_flag_parses_and_documents_vanilla_main_options` verifies the Java option surface is documented.
- [x] Support `--universe`. — `CliOptions` parses Java `Main`'s required path argument, `runtime_selection()` feeds it into startup world paths, and `runtime_selection_uses_java_main_world_universe_port_and_server_id_options` verifies CLI override behavior.
- [x] Support `--world`. — `CliOptions` parses Java `Main`'s required world-name argument, `runtime_selection()` falls back to `level-name` when absent, and `runtime_selection_uses_java_main_world_universe_port_and_server_id_options` verifies both paths.
- [x] Support `--port`. — `CliOptions` parses Java `Main`'s integer port argument with `-1` default semantics, `runtime_selection()` falls back to `server-port` when absent, and `runtime_selection_uses_java_main_world_universe_port_and_server_id_options` verifies both paths.
- [x] Support `--serverId`. — `CliOptions` parses Java `Main`'s required string argument, `runtime_selection()` carries it into startup state, and `runtime_selection_uses_java_main_world_universe_port_and_server_id_options` verifies the selected ID.
- [x] Support `--jfrProfile` with an equivalent profiling story or documented no-op. — `CliOptions` accepts Java `Main`'s flag, RustCraft documents it as accepted without profiling support, and `help_documents_jfr_profile_as_noop` verifies the no-op is visible in help output.
- [x] Support `--pidFile`. — `CliOptions` parses the Java `Main` path argument, startup writes the current process ID before EULA refusal can stop normal startup, and `pid_file_is_written_before_eula_refusal` covers the behavior.
- [x] Initialize `server.properties` and `eula.txt` before normal startup. — `run()` loads/saves `server.properties`, creates `eula.txt`, and `missing_eula_refuses_startup_after_generating_files` verifies both files exist before the server refuses normal startup for `eula=false`.
- [x] Refuse startup until `eula=true`. — `run()` returns before world/listener startup when `Eula::load_or_create()` reports `eula=false`, and `missing_eula_refuses_startup_after_generating_files` verifies the refusal log plus absence of a generated world.
- [ ] Write logs matching vanilla lifecycle milestones closely enough for operators and test tooling.
- [x] Install process-level crash and panic handlers. — `main()` installs `crash::install_panic_hook()` before option parsing, the hook writes `crash-reports/crash-*-server.txt`, and the `crash` test filter covers report rendering/writing.
- [ ] Emit crash reports with useful environment, thread, and world state.
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
- [ ] Handle corrupted world metadata with vanilla-compatible refusal paths.
- [ ] Detect incompatible world versions before loading.
- [x] Provide safe-mode datapack loading. — `configure_pack_repository()` mirrors Java `Main`'s safe-mode pack config by selecting only `vanilla`, and `safe_mode_selects_only_vanilla_and_does_not_disable_world_packs` covers the behavior without persisting world-pack disables.
- [ ] Support world data upgrade and region recreation workflow.
- [ ] Implement server watchdog behavior controlled by `max-tick-time`.

## Migrated From Main Checklist: Dedicated Server Configuration

- [ ] Parse and write `server.properties` with vanilla defaults.
- [x] Preserve unknown property keys when rewriting configuration. — `ServerProperties` keeps a raw key/value map through load, typed mutation, and save like Java `Settings` storing its loaded `Properties`, and `preserves_unknown_keys_when_saving` verifies an unknown key survives rewriting.
- [ ] Implement `online-mode`.
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
- [x] Implement `prevent-proxy-connections`. — `ServerProperties` parses Java's default, online authentication forwards the remote address to session service only when enabled like Java, and login rejects mismatched handshake/socket IPs with `multiplayer.disconnect.unverified_username`; `cargo test -q prevent_proxy` covers the behavior.
- [ ] Implement `server-ip`.
- [ ] Implement mutable `allow-flight`.
- [ ] Implement mutable `motd`.
- [ ] Implement `enable-code-of-conduct`.
- [ ] Implement `bug-report-link`.
- [ ] Implement mutable `force-gamemode`.
- [ ] Implement mutable `enforce-whitelist`.
- [ ] Implement mutable `difficulty`.
- [ ] Implement mutable `gamemode`.
- [x] Implement `level-name`. — `ServerProperties` parses Java's default `level-name=world`, `runtime_selection()` uses it when `--world` is absent like Java `Main`, and `runtime_selection_uses_java_main_world_universe_port_and_server_id_options` covers the property fallback.
- [x] Implement `server-port`. — `ServerProperties` parses Java's default `server-port=25565`, startup passes the selected port to the status listener, and `runtime_selection_uses_java_main_world_universe_port_and_server_id_options` verifies the property fallback plus CLI override.
- [ ] Implement management server settings: enabled, host, port, secret, TLS, keystore, password, allowed origins.
- [ ] Implement legacy `announce-player-achievements` migration behavior.
- [x] Implement `enable-query` and `query.port`. — `ServerProperties` parses Java's defaults, startup spawns the GS4 query listener only when `enable-query=true` and binds `query.port`, and the `query_` test filter covers challenge, basic status, full stat, and challenge-expiry behavior.
- [x] Implement `enable-rcon`, `rcon.port`, and `rcon.password`. — `ServerProperties` parses Java's defaults, startup only spawns RCON when `enable-rcon=true`, binds `rcon.port`, and passes `rcon.password` into authentication; `cargo test -q rcon_` covers the listener/auth path.
- [ ] Implement `hardcore`.
- [x] Implement `use-native-transport` or document the equivalent transport decision. — `ServerProperties` accepts Java's default-true `use-native-transport` key, `exposes_typed_vanilla_properties` covers the typed value, and `docs/COMPATIBILITY.md` documents RustCraft's standard `TcpListener`/`TcpStream` transport decision.
- [ ] Implement mutable `spawn-protection`.
- [ ] Implement mutable `op-permission-level`.
- [ ] Implement `function-permission-level`.
- [ ] Implement `max-tick-time`.
- [ ] Implement `max-chained-neighbor-updates`.
- [ ] Implement `rate-limit`.
- [ ] Implement mutable `view-distance`.
- [ ] Implement mutable `simulation-distance`.
- [ ] Implement mutable `max-players`.
- [ ] Add a Mineflayer max-player enforcement test that fills available slots in offline mode and verifies the extra bot receives the vanilla full-server disconnect message.
- [ ] Implement `network-compression-threshold`.
- [x] Implement `broadcast-rcon-to-ops`. — `ServerProperties` parses Java's default, startup passes it into the RCON session, and command feedback mirrors Java's `shouldRconBroadcast()` admin-notification gate; focused RCON and command-feedback tests cover enabled and disabled behavior.
- [ ] Implement `broadcast-console-to-ops`.
- [x] Implement clamped `max-world-size`. — `ServerProperties` parses `max-world-size` and clamps it to Java's `1..=29999984` range from `DedicatedServerProperties`, with `max_world_size_clamps_to_vanilla_property_range` covering both bounds.
- [ ] Implement `sync-chunk-writes`.
- [ ] Implement `region-file-compression` with at least deflate parity.
- [ ] Implement `enable-jmx-monitoring` or equivalent documented metrics export.
- [ ] Implement mutable `enable-status`.
- [ ] Implement mutable `hide-online-players`.
- [ ] Implement mutable `entity-broadcast-range-percentage`.
- [ ] Implement `text-filtering-config` and `text-filtering-version`.
- [ ] Implement server resource pack fields: id, URL, SHA-1, legacy hash, required flag, prompt component.
- [ ] Implement initial datapack enabled/disabled pack lists.
- [ ] Implement mutable `player-idle-timeout`.
- [ ] Implement mutable `status-heartbeat-interval`.
- [ ] Implement mutable `white-list`.
- [ ] Add Mineflayer whitelist tests for offline-mode allow, deny, runtime `/whitelist reload`, and `enforce-whitelist` toggles.
- [ ] Add Mineflayer mutable-property tests that change MOTD, difficulty, gamemode, view-distance, simulation-distance, idle timeout, and whitelist settings at runtime, then verify existing and reconnecting offline-mode bots observe vanilla-compatible state.
- [ ] Add a Mineflayer offline-mode configuration reload test that edits `server.properties`, runs the vanilla-equivalent reload path where supported, reconnects the bot, and verifies which properties do and do not take effect without restart.
- [ ] Add a Mineflayer offline-mode secure-profile toggle test that verifies `enforce-secure-profile=false` never blocks generated offline bots and that `true` matches official `server.jar` behavior for unsigned Mineflayer clients.
- [ ] Implement `enforce-secure-profile`.
- [ ] Implement `log-ips`.
- [ ] Implement mutable `pause-when-empty-seconds`.
- [ ] Implement `level-seed`, `generate-structures`, `generator-settings`, and `level-type`.
- [ ] Implement mutable `accepts-transfers`.

## Migrated From Main Checklist: Core Runtime Model

- [ ] Implement a main server thread with a deterministic 20 TPS tick loop.
- [ ] Implement tick budget accounting and drift handling.
- [ ] Implement pause-when-empty behavior.
- [ ] Implement scheduled task execution on the server thread.
- [ ] Implement async task pools for IO, chunk generation, resource reloads, and profile/session services.
- [ ] Implement thread-safety rules equivalent to vanilla server access restrictions.
- [ ] Implement tick-rate management, freeze/step/sprint behavior, and tick command hooks.
- [ ] Implement profiling hooks equivalent to debug/perf/JFR command output where practical.
- [ ] Implement bandwidth, tick time, packet, and debug sample collection.
- [ ] Implement crash-safe autosave cadence.
- [ ] Implement forced save, save-off, and save-on semantics.
- [ ] Add Mineflayer tick-loop stability tests that keep an offline-mode bot connected through pause-when-empty transitions, autosave, `/save-off`, `/save-on`, and `/save-all`, verifying keepalives and visible state do not stall.

## Migrated From Main Checklist: Source-Derived Granularity Appendix - Operational File And API Coverage

- [ ] Implement `eula.txt`, `server.properties`, `ops.json`, `whitelist.json`, `banned-players.json`, `banned-ips.json`, `usercache.json`, `session.lock`, `level.dat`, `level.dat_old`, region files, entity region files, POI files, playerdata, advancements, stats, icon, crash reports, logs, debug output, and generated reports.
- [ ] Implement RCON authentication, command execution, response fragmentation, broadcast behavior, and failure modes.
- [ ] Implement query protocol basic and full stat responses with plugin list behavior equivalent to vanilla.
- [ ] Implement JSON-RPC management methods, schemas, notifications, player DTOs, reference utilities, pending request tracking, origin checks, TLS settings, and shutdown behavior.
- [ ] Implement chase server/client debug feature or document it as intentionally unsupported with no impact on vanilla clients.
- [ ] Implement game test framework hooks enough for parity test execution or document a replacement harness.
- [ ] Implement generated data/report tooling used to compare registries, tags, commands, packs, and worldgen definitions.
