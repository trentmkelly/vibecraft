# Project Foundation Checklist

Cross-cutting project rules and compatibility commitments moved out of the top-level checklist.

## Migrated From Main Checklist: Ground Rules

- [x] Define the target as Minecraft Java Edition 26.1.2 protocol and server behavior.
- [x] Keep the new implementation independent of Mojang source code licensing constraints.
- [x] Use black-box tests, public protocol references, vanilla datapacks, generated assets, and observed behavior as validation sources.
- [x] Track every intentional deviation from vanilla behavior in a compatibility document.
- [x] Build a repeatable test harness that can compare the rebuilt server against the official `server.jar`.
- [x] Add Mineflayer-based black-box smoke tests that can target either RustCraft or the official `server.jar`.
- [x] Add a Mineflayer fixture that records bot event order, disconnect reasons, kicked messages, and packet-level errors for every black-box scenario.
- [x] Add a Mineflayer offline-mode baseline scenario shared by all black-box tests, including deterministic bot names, expected offline UUIDs, seeded temp worlds, fixed ports, and vanilla comparison output.
- [x] Add a Mineflayer offline-mode login contract test that every black-box suite can reuse to assert server readiness, EULA acceptance, deterministic profile creation, and successful play-state entry before running scenario-specific assertions.
- [x] Add a Mineflayer offline-mode login artifact test that captures `server.properties`, `eula.txt`, `usercache.json`, playerdata paths, bot UUIDs, and normalized vanilla/RustCraft log lines for each baseline run.
- [x] Add Mineflayer parity snapshots for the offline-mode happy path that preserve raw bot events, packet names, server log excerpts, and normalized official-vs-RustCraft diffs as test artifacts.
- [x] Add a Mineflayer offline-mode preflight test that fails fast with actionable artifacts when the server never opens the port, never reaches login success, or disconnects before configuration completes.
- [x] Add a Mineflayer offline-mode version-lock test that verifies the configured Mineflayer/prismarine protocol version matches Minecraft Java Edition 26.1.2 before any black-box scenario runs.
- [x] Add a Mineflayer offline-mode login diagnostics contract requiring every black-box failure to include bot username, expected UUID, server PID, port, last packet, last bot event, and normalized disconnect component.
- [x] Add a Mineflayer offline-mode baseline comparison test that runs the same generated bot profile against vanilla first, stores the accepted event timeline, then reuses it as the RustCraft assertion envelope.
- [x] Add a Mineflayer offline-mode matrix owner test that requires every black-box scenario touching login to declare whether it uses fresh profile, returning profile, duplicate profile, op profile, whitelisted profile, or banned profile fixtures.
- [x] Add a Mineflayer offline-mode vanilla-oracle freshness test that reruns the official `server.jar` baseline when the Mineflayer/prismarine version, protocol version, server jar, or scenario fixture changes.
- [x] Add a Mineflayer offline-mode fixture sanity test that proves each generated bot username maps to the expected UUID before server start, after vanilla login, and after RustCraft login.
- [x] Add a Mineflayer offline-mode artifact diff test that compares vanilla and RustCraft login artifacts with volatile ports, timestamps, temp paths, and random seeds normalized out.
- [x] Treat Mineflayer as the primary automated beacon for regressions whenever its prismarine protocol tables support the target protocol; keep the vanilla 26.1.2 client as the authoritative connectability oracle and use a raw protocol probe to bridge version-support gaps.
- [ ] Preserve exact client compatibility for unmodified 26.1.2 clients.
- [ ] Preserve compatibility with vanilla resource/data packs that rely only on official behavior.
- [x] Decide which implementation language, async runtime, serialization libraries, compression libraries, crypto libraries, and persistence libraries will be used.
- [x] Define project modules matching major vanilla boundaries: bootstrap, registries, network, commands, resources, storage, world, entities, gameplay, operations, and tests.

## Migrated From Main Checklist: Source-Derived Granularity Appendix Note

Use this appendix as an implementation tracking map for the 26.1.2 decompiled tree. The main sections above describe the behavior; this section pins that behavior to concrete source families that should each get design notes, implementation tasks, and parity tests.
