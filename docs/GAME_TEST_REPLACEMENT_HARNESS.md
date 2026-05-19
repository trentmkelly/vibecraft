# Game Test Replacement Harness

RustCraft does not expose vanilla's in-game GameTest runner as the primary parity
mechanism. The replacement harness is the versioned black-box test surface under
`harness/mineflayer/`, plus the Rust-side parity models in `src/parity_harness.rs`.

This intentionally tests through the same external surfaces used by real clients:
status, handshake, login, configuration, play packets, world files, and command
side effects. That gives coverage for behavior that vanilla GameTest cannot see
directly, such as protocol ordering, offline-mode profile files, reconnect
state, server properties, and client crash regressions.

## Vanilla GameTest Responsibility Mapping

- Structure setup and scripted actions: `harness/mineflayer/action_fixtures.mjs`,
  `first_tick_actions.mjs`, and the scenario modules for blocks, entities,
  inventory, commands, worldgen, weather, time/sleep, and status effects.
- Command execution and assertions: `command_scenarios.mjs`,
  `command_parity_scenarios.mjs`, and `command_block_function_scenarios.mjs`.
- Client-observable pass/fail: raw 26.1.2 probes and Mineflayer gates record
  accepted packets, disconnects, spawned entities, chunks, inventory state,
  chat/feedback, and persisted files.
- Official-server oracle comparisons: `runner.mjs`, `vanilla_compatibility.mjs`,
  `configuration_transcript_oracle.mjs`, `vanilla_worldgen_oracle.mjs`, and
  `vanilla_region_reader.mjs`.
- Regression gating: `release_gate.mjs`, `ci_login_shard.mjs`,
  `configuration_registry_readiness_gate.mjs`, and
  `worldgen_acceptance_gates.mjs`.

## Required Evidence For Checklist Credit

A parity item that would normally be a vanilla GameTest should include at least
one of:

- a Rust unit/model test tied to decompiled Java behavior;
- a Mineflayer scenario test with an offline-mode bot;
- a raw 26.1.2 protocol test when Mineflayer does not yet support the target
  packet surface;
- a fixture/oracle report that compares RustCraft and official `server.jar`.

The checklist should remain open for any subsystem that lacks one of those
evidence paths, even if a helper or model exists.
