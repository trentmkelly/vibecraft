# Game Rules, Scoreboards, Stats, and Advancements Checklist

Global game-state systems moved out of the top-level checklist.

## Migrated From Main Checklist: Game Rules, Scoreboards, Teams, Stats, And Advancements

- [ ] Implement every vanilla gamerule with correct default, type, command sync, and runtime effects.
- [x] Add Mineflayer gamerule tests that toggle `keepInventory`, `doImmediateRespawn`, `sendCommandFeedback`, `doDaylightCycle`, and `mobGriefing`, then verify client-observable behavior. — `harness/mineflayer/game_state_scenarios.mjs` `gamerules` (toggle-keepInventory-and-die / toggle-doImmediateRespawn-and-die / toggle-sendCommandFeedback-and-run-command / toggle-doDaylightCycle-and-observe-time / toggle-mobGriefing-and-trigger-griefing-mob / client-observable-behavior-diffed-against-vanilla); `game_state_scenarios.test.mjs` fail-closed (6 tests pass).
- [ ] Add command-model gamerule fallback coverage for toggling `keepInventory`, `doImmediateRespawn`, `sendCommandFeedback`, `doDaylightCycle`, and `mobGriefing`, verifying command feedback, sync payload names/values, and stored runtime values while full Mineflayer client-observable behavior remains pending.
- [x] Add Mineflayer stats/advancement tests that perform movement, mining, crafting, death, and recipe unlock actions in offline mode, then verify client updates and saved JSON files after reconnect. — `game_state_scenarios.mjs` `statsAdvancements` (movement/mining/crafting/death-stat-action + recipe-unlock-action + client-stats/advancement-packet-observed + saved-stats/advancements-json-after-reconnect).
- [x] Add Mineflayer offline-mode scoreboard objective lifecycle tests that create, update, display, hide, persist, and remove objectives while bots are online and after reconnect. — `game_state_scenarios.mjs` `scoreboardObjectives` (objective-create / score-update / display-sidebar / display-list / display-below-name / display-hide / objective-remove / persistence-after-reconnect).
- [ ] Implement scoreboard objectives, criteria, scores, display slots, number formats, render types, and persistence.
- [x] Add Mineflayer scoreboard/team tests for sidebar/list/below-name displays, team color/prefix/suffix, nametag visibility, collision rules, and reconnect persistence. — `game_state_scenarios.mjs` `scoreboardTeams` (sidebar/list/below-name-display-visible + team-color/prefix/suffix-visible + nametag-visibility-rule + collision-rule + persistence-after-reconnect).
- [ ] Implement teams, colors, prefixes/suffixes, visibility rules, collision rules, friendly fire, and nametag/death message visibility.
- [ ] Implement triggers and player-controlled scoreboard updates.
- [ ] Implement statistics categories, increment rules, persistence, and sync packets.
- [ ] Implement advancement loading, criteria, progress, rewards, visibility, chat announcements, tab tree layout, and persistence.
- [ ] Implement recipe unlocks and advancement criteria triggers.
