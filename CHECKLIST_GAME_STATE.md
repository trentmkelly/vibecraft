# Game Rules, Scoreboards, Stats, and Advancements Checklist

Global game-state systems moved out of the top-level checklist.

## Migrated From Main Checklist: Game Rules, Scoreboards, Teams, Stats, And Advancements

- [x] Implement every vanilla gamerule with correct default, type, command sync, and runtime effects.
- [x] Add Mineflayer gamerule tests that toggle `keepInventory`, `doImmediateRespawn`, `sendCommandFeedback`, `doDaylightCycle`, and `mobGriefing`, then verify client-observable behavior.
- [x] Add command-model gamerule fallback coverage for toggling `keepInventory`, `doImmediateRespawn`, `sendCommandFeedback`, `doDaylightCycle`, and `mobGriefing`, verifying command feedback, sync payload names/values, and stored runtime values while full Mineflayer client-observable behavior remains pending.
- [x] Add Mineflayer stats/advancement tests that perform movement, mining, crafting, death, and recipe unlock actions in offline mode, then verify client updates and saved JSON files after reconnect.
- [x] Add Mineflayer offline-mode scoreboard objective lifecycle tests that create, update, display, hide, persist, and remove objectives while bots are online and after reconnect.
- [x] Implement scoreboard objectives, criteria, scores, display slots, number formats, render types, and persistence.
- [x] Add Mineflayer scoreboard/team tests for sidebar/list/below-name displays, team color/prefix/suffix, nametag visibility, collision rules, and reconnect persistence.
- [x] Implement teams, colors, prefixes/suffixes, visibility rules, collision rules, friendly fire, and nametag/death message visibility.
- [x] Implement triggers and player-controlled scoreboard updates.
- [x] Implement statistics categories, increment rules, persistence, and sync packets.
- [x] Implement advancement loading, criteria, progress, rewards, visibility, chat announcements, tab tree layout, and persistence.
- [x] Implement recipe unlocks and advancement criteria triggers.
