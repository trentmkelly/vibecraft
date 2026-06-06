# Parity Harness Checklist

Automated vanilla-client, Mineflayer, oracle, and regression harness work moved out of the top-level checklist.

## Migrated From Main Checklist: Vanilla Client Oracle Harness Notes

Use the unmodified 26.1.2 client as the authoritative visual/connectability oracle when Mineflayer is not enough.

- [ ] Start a nested X11 display that can be automated without driving the normal desktop: `Xephyr :2 -screen 1280x720 -resizeable -ac`.
- [ ] Capture the nested display with `DISPLAY=:2 import -window root /tmp/codex-screens/xephyr.png`.
- [ ] Send input only to the nested display/window with `DISPLAY=:2 xdotool ...`.
- [ ] Avoid launching `minecraft-launcher` for automation: it uses existing launcher state/single-instance behavior and may open on the real desktop instead of inside Xephyr.
- [ ] Launch the installed vanilla client JVM directly by cloning the argv from the already-running `java ... net.minecraft.client.main.Main` process, changing `--gameDir` to a temp directory such as `/tmp/codex-mc-xephyr`, and omitting `--quickPlayPath` if needed.
- [ ] Keep auth material out of logs/transcripts: read `/proc/<pid>/cmdline` into an argv array and `exec` it directly; do not print the full Java command because it contains `--accessToken`.
- [ ] Use `GLFW_PLATFORM=x11`, `XDG_SESSION_TYPE=x11`, and `DISPLAY=:2` for the direct client process.
- [ ] The first successful setup produced a nested `Minecraft 26.1.2` window on `:2`, visible to `DISPLAY=:2 xwininfo -root -tree`, and screenshots worked with ImageMagick `import`.
- [x] Turn this into a checked-in script that starts Xephyr if needed, launches the direct vanilla client in an isolated game directory, screenshots the root window, and stores artifacts under `artifacts/vanilla-client/`. References: `RustCraft/harness/mineflayer/vanilla_client_xephyr.mjs`, `RustCraft/harness/mineflayer/vanilla_client_xephyr.test.mjs`. — both files present; `vanilla_client_xephyr.test.mjs` passes (4 tests).
- [x] Add a scriptable connection smoke test that uses the vanilla client in Xephyr to enter Multiplayer, select the saved local server, join, wait for terrain, screenshot, and collect `latest.log`/crash reports. References: `RustCraft/harness/mineflayer/vanilla_client_connection_smoke.mjs`, `RustCraft/harness/mineflayer/vanilla_client_connection_smoke.test.mjs`. — both files present; `vanilla_client_connection_smoke.test.mjs` passes (2 tests).

## Migrated From Main Checklist: Testing And Parity Harness

- [ ] Build an integration harness that starts official `server.jar` and the rebuilt server with the same seed/config.
- [x] Build a reusable Mineflayer test runner that starts RustCraft in a temp world, waits for readiness, connects bots, captures logs, and tears down cleanly. — `harness/mineflayer/runner.mjs` (startRustCraft / createTempWorld / waitForPort / writeOfflineServerFiles / stopServer); `runner.test.mjs` passes (8 tests).
- [ ] Build a Mineflayer parity mode that runs the same bot script against RustCraft and the official `server.jar` and diffs observable events.
- [x] Build Mineflayer scenario fixtures for deterministic offline-mode profiles, temp `server.properties`, eula setup, seeded worlds, and per-test log capture. — `harness/mineflayer/fixtures.mjs`; `fixtures.test.mjs` passes (4 tests).
- [x] Build Mineflayer assertion helpers for offline-mode login phases, vanilla UUID derivation, packet/event ordering, kicked-message normalization, and official-vs-RustCraft diff output. — `harness/mineflayer/assertions.mjs` exports `assertLoginPhases` / `assertOfflineUuid` / `assertEventOrder` + `assertPacketOrder` / `normalizeKickedMessage` / `assertParityDiff`+`formatParityDiff`; `assertions.test.mjs` passes (5 tests).
- [x] Build a shared Mineflayer offline-mode login helper that returns a fully observed session object containing bot profile, UUID, event timeline, packet trace, server log slice, temp paths, and cleanup handles. — `harness/mineflayer/login_session.mjs` (`runObservedOfflineLogin` returns profile / uuid / timeline / packetTrace / log slice / temp paths / endpoint / cleanup); `login_session.test.mjs` passes (4 tests).
- [x] Build Mineflayer helpers for scripted offline-mode bot actions: wait for spawn, assert held item, issue command, place/break block, open window, force reconnect, and capture vanilla comparison traces. — `harness/mineflayer/bot_actions.mjs` exports exactly `waitForSpawn` / `assertHeldItem` / `issueCommand` / `placeBlock` / `breakBlock` / `openWindow` / `forceReconnect` / `captureVanillaComparisonTraces`; `bot_actions.test.mjs` passes (6 tests).
- [ ] Build Mineflayer reusable action fixtures for loot and economy tests: deterministic bot inventory setup, tool/enchantment setup, mob/chest placement, villager offer capture, fishing loop control, item entity collection, and post-reconnect artifact snapshots.
- [x] Build a packet recorder/replayer for login, status, configuration, and play-state flows. — `harness/mineflayer/packet_recorder.mjs`; `packet_recorder.test.mjs` passes (6 tests).
- [ ] Build golden packet tests for all protocol states.
- [x] Build Mineflayer packet-flow smoke tests for offline-mode login, configuration completion, keepalive response, chat, command execution, and disconnect reason. — `harness/mineflayer/packet_flow_smoke.mjs`; `packet_flow_smoke.test.mjs` passes (6 tests).
- [x] Build a required Mineflayer offline-mode login gate that runs before every merge touching network, configuration, player management, storage, or tick-loop code. — `harness/mineflayer/login_gate.mjs`; `login_gate.test.mjs` passes (6 tests).
- [x] Build a minimal Mineflayer offline-mode login CI shard that runs before longer parity tests and fails fast on TCP readiness, login timeout, configuration ordering, first spawn, or unexpected disconnects. — `harness/mineflayer/ci_login_shard.mjs`; `ci_login_shard.test.mjs` passes (5 tests).
- [x] Build a Mineflayer offline-mode login debug bundle writer that archives per-failure temp worlds, server logs, bot packet traces, profile UUIDs, and normalized vanilla/RustCraft event diffs. — `harness/mineflayer/debug_bundle.mjs`; `debug_bundle.test.mjs` passes (3 tests).
- [x] Build a Mineflayer offline-mode login bisect mode that repeats the reusable login gate across recent commits or feature flags and reports the first failing protocol/configuration milestone. — `harness/mineflayer/login_bisect.mjs`; `login_bisect.test.mjs` passes (5 tests).
- [x] Build a Mineflayer offline-mode login quarantine report that separates server bugs from Mineflayer/client-version incompatibilities, including captured protocol version, prismarine dependencies, and raw disconnect packets. — `harness/mineflayer/quarantine_report.mjs`; `quarantine_report.test.mjs` passes (5 tests).
- [x] Build a Mineflayer offline-mode login flake detector that repeats the minimal login gate under randomized ports and temp directories, then reports timing variance, intermittent kicks, and leaked server processes. — `harness/mineflayer/flake_detector.mjs`; `flake_detector.test.mjs` passes (4 tests).
- [x] Build a Mineflayer offline-mode login artifact normalizer that redacts temp paths, ports, timestamps, and randomized usernames while preserving UUIDs, packet order, kicked messages, and official-vs-RustCraft diffs. — `harness/mineflayer/artifact_normalizer.mjs`; `artifact_normalizer.test.mjs` passes (4 tests).
- [x] Build a Mineflayer offline-mode fixture linter that rejects scenarios without explicit version, server properties, expected UUIDs, timeout budget, packet capture policy, and vanilla comparison mode. — `harness/mineflayer/fixture_linter.mjs`; `fixture_linter.test.mjs` passes (6 tests).
- [x] Build a Mineflayer offline-mode failure minimizer that can rerun a failing login scenario with one bot, one property file, one temp world, and packet capture still enabled. — `harness/mineflayer/failure_minimizer.mjs`; `failure_minimizer.test.mjs` passes (3 tests).
- [ ] Build worldgen comparison tests for deterministic chunks across many seeds and coordinates.
- [ ] Build command parity tests for syntax, suggestions, success counts, side effects, and error messages.
- [x] Build Mineflayer command tests for `/list`, `/tell`, `/msg`, `/me`, `/help`, `/seed`, `/gamemode`, and permission-denied feedback in offline mode. — `harness/mineflayer/command_scenarios.mjs` covers all 7 commands (/list, /tell, /msg, /me, /help, /seed, /gamemode) + denied feedback; `command_scenarios.test.mjs` passes (24 tests).
- [x] Build Mineflayer chat tests for public chat, private messages, formatting, death/advancement announcements where applicable, and disconnect-on-malformed-message cases. — `harness/mineflayer/chat_scenarios.mjs` (public / private / formatting / death + advancement / malformed-disconnect + system messages); `chat_scenarios.test.mjs` passes (8 tests).
- [ ] Build inventory transaction tests using scripted client packets.
- [x] Build Mineflayer inventory tests for hotbar selection, item pickup/drop, window open/close, slot clicks, held item sync, and carried item correction. — `harness/mineflayer/inventory_scenarios.mjs` (hotbar select / item pickup+drop / window open+close / slot clicks / held-item sync / carried-item correction); `inventory_scenarios.test.mjs` passes (8 tests).
- [ ] Build block behavior tests for placement, break, use, redstone, fluid, and scheduled ticks.
- [ ] Build Mineflayer block interaction tests for digging, placing, using blocks, denied interactions, spawn-protection behavior, and block update visibility.
- [ ] Build entity behavior tests for spawning, AI, pathfinding, combat, drops, save/load, and network metadata.
- [ ] Build Mineflayer entity observation tests for spawn/despawn events, metadata updates, damage animations, item pickup, and simple combat interactions.
- [ ] Build persistence round-trip tests for worlds, chunks, players, advancements, stats, scoreboards, maps, raids, and POIs.
- [ ] Build datapack reload tests with valid and invalid packs.
- [ ] Build fuzz tests for packet decoders, NBT parser, command parser, resource loaders, and save readers.
- [ ] Build performance benchmarks for tick loop, chunk IO, chunk generation, packet throughput, entity ticking, pathfinding, and redstone.
- [ ] Build long-running soak tests with real clients.
- [ ] Build a Mineflayer multi-bot soak test for repeated offline-mode joins, leaves, movement ticks, keepalives, chat, and reconnects.
- [ ] Build compatibility tests with vanilla 26.1.2 client join, survival play, death/respawn, dimension travel, saving, reconnecting, and shutdown.
- [ ] Build Mineflayer gameplay smoke tests for movement, block dig/place, item pickup/drop, inventory clicks, respawn, and reconnect persistence.
- [ ] Build Mineflayer offline-mode regression tests that run after every protocol/login change and fail on login timeout, unexpected kick, registry-order drift, or play-state stall.
- [ ] Build crash recovery tests using killed process and corrupted inputs.
- [ ] Build a release gate requiring no known vanilla parity regressions in core workflows.
