# Validation Sources

VibeCraft targets Minecraft Java Edition 26.1.2 behavior. The local Java decompilation is the behavioral oracle when it is present, but checked-in validation must stay independent of copying Mojang source.

Every parity checklist item needs evidence from one or more of these sources:

- Black-box tests: Mineflayer and raw-client scenarios under `harness/mineflayer/` run against VibeCraft and, when a parity claim is made, the official `server.jar`.
- Public protocol references: Mineflayer's prismarine stack and `minecraft-data` package are used as public protocol tables when they support protocol 775; `transcript_scenario_wrapper.mjs` falls back to raw 26.1.2 probes until that support exists.
- Vanilla datapacks and resources: checked-in `vanilla-data/data/minecraft/` files, registry scenarios, datapack scenarios, and resource loaders provide the vanilla resource baseline.
- Generated assets: generated Rust resource/catalog tables under `src/data_*`, `src/block_states/`, `src/generated_reports.rs`, and generated packet coverage plans from `protocol_packet_manifest.mjs` are validation inputs, not hand-written protocol guesses.
- Observed behavior: login sessions, parity snapshots, raw 26.1.2 probes, vanilla-client Xephyr smoke tests, server logs, packet traces, and normalized artifact diffs capture behavior from real clients and official-server runs.

When these sources disagree, prefer observed official-server and unmodified-client behavior, then the Java oracle, then public protocol/resource references. Do not check a checklist row unless the cited evidence covers the row's full behavior surface.
