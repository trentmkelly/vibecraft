# Authentication, Security, and Chat Trust Checklist

Authentication, secure profile, signed chat, and abuse-reporting metadata parity work moved out of the top-level checklist.

## Migrated From Main Checklist: Authentication, Security, And Chat Trust

- [ ] Implement Yggdrasil session server authentication for online-mode.
- [ ] Implement profile lookup/cache with expiration behavior.
- [ ] Add a Mineflayer offline-mode identity test that confirms username, UUID derivation, whitelist checks, bans, and operator lookup behavior.
- [ ] Add raw 26.1.2 offline-mode identity/access fallback coverage for username, offline UUID derivation, usercache writes, whitelist rejection, whitelist allow, op whitelist bypass, player ban, and IP ban while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer/raw 26.1.2 offline-mode deterministic UUID test that logs in multiple generated names, verifies `OfflinePlayer:<name>` UUID derivation against vanilla, then repeats across restart and case-variant joins.
- [ ] Add a Mineflayer offline-mode same-name replacement test that joins one bot, connects another with the same username, and verifies vanilla-compatible kick order, entity cleanup, tab-list replacement, and playerdata ownership.
- [ ] Add raw 26.1.2 same-name active-session replacement fallback coverage that holds one offline profile in play, connects a second session with the same UUID, verifies the replacement reaches play, verifies the original socket is closed, and verifies same-name retry succeeds after cleanup while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode identity-normalization test that compares original casing, UUID derivation input, stored profile names, command selector matching, and log output for case-variant generated names.
- [ ] Add raw 26.1.2 identity-normalization fallback coverage that joins case-variant generated offline names, verifies original casing and vanilla `OfflinePlayer:<name>` UUID derivation are preserved in play tab-list identity, `usercache.json`, and UUID-owned playerdata files while Mineflayer command selector/log-output validation remains blocked on target-protocol play support.
- [ ] Add a Mineflayer offline-mode profile collision test that joins names differing only by case and verifies vanilla-compatible UUIDs, display names, duplicate-session handling, and persisted file ownership.
- [ ] Add raw 26.1.2 case-variant profile-collision fallback coverage that joins generated offline names differing only by case, verifies distinct vanilla offline UUID derivation, preserves display names in login/usercache, and confirms distinct UUID-owned playerdata files while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode profile-file test that logs in with fresh, returning, renamed-case, banned, whitelisted, and op bot profiles and verifies `usercache.json`, `ops.json`, `whitelist.json`, and ban-list matching use vanilla UUID/name semantics.
- [ ] Add raw 26.1.2 profile-file fallback coverage for fresh and case-variant offline profiles, including distinct case-sensitive UUIDs, persisted `usercache.json` names/UUIDs, and vanilla-style `expiresOn` fields while Mineflayer lacks target-protocol play support.
- [ ] Add focused raw 26.1.2 profile-file matrix coverage for fresh, returning, renamed-case, banned, whitelisted, and op offline profiles, verifying `usercache.json`, `ops.json`, `whitelist.json`, ban-list matching, and UUID-owned playerdata files use vanilla offline UUID/name semantics while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer/raw 26.1.2 offline-mode generated-profile fixture test that creates disposable usernames, verifies deterministic UUIDs before connect, and confirms server-side profile files use those exact UUIDs after login.
- [ ] Add a Mineflayer offline-mode ban test covering profile ban, IP ban, pardon, and reconnect behavior with vanilla-compatible messages.
- [ ] Add raw 26.1.2 ban/pardon reconnect fallback coverage that rejects banned profiles and IPs, removes each ban from the vanilla access file, restarts the server, and verifies the same offline profile can reconnect while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode operator-permission test that joins as non-op and op profiles, runs permission-gated commands, reloads `ops.json`, and verifies tab completion plus feedback visibility.
- [ ] Add command-model/raw fallback operator-permission coverage for non-op denial, admin/op command visibility, op/deop feedback keys, `ops.json` whitelist bypass, and deop reload/kick side effects while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode usercache test that joins generated profiles, restarts the server, and verifies cached names, UUIDs, expiration behavior, and lookup side effects match vanilla.
- [ ] Add raw 26.1.2 usercache fallback coverage for generated profile joins across restart plus missing, empty, malformed, duplicate, and stale `usercache.json` repair with vanilla-style `expiresOn` handling while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode usercache-corruption test that starts with missing, empty, malformed, duplicate, and stale `usercache.json` entries, then verifies login repair and disconnect behavior match vanilla.
- [ ] Add focused raw 26.1.2 usercache-corruption fallback coverage that starts isolated servers with missing, empty, malformed, stale, and duplicate `usercache.json`, joins generated offline profiles, and verifies repaired UUID/name/expiresOn entries plus stale bogus UUID removal while Mineflayer lacks target-protocol play support.
- [ ] Add a Mineflayer offline-mode auth-file hot-edit test that edits `ops.json`, `whitelist.json`, `banned-players.json`, and `banned-ips.json` while bots are online, runs the vanilla reload path, and verifies current and reconnecting bots observe the same effects as official `server.jar`.
- [ ] Add raw 26.1.2 auth-file hot-edit fallback coverage that edits `banned-players.json` while the server process is online, runs console `reload`/`whitelist reload`, and verifies reconnecting offline profiles observe the new ban and subsequent pardon while Mineflayer lacks target-protocol play support.
- [x] Implement banned profile and banned IP checks — `PlayerAccess` loads vanilla `banned-players.json`/`banned-ips.json`, exposes UUID/IP checks, persists the files, and the login gate now matches Java `PlayerList.canPlayerLogin` precedence (profile ban before whitelist before IP ban); covered by `player_access` load/save/check tests, command ban/pardon tests, and `login_access_gate_matches_java_ban_whitelist_and_op_order`.
- [x] Implement whitelist checks — `PlayerAccess` loads/persists `whitelist.json`, command handling mutates the whitelist model, and `login_access_disconnect_reason` enforces `enforce-whitelist` with Java's op bypass behavior; covered by whitelist command tests, player-access load/save tests, and login-gate precedence coverage.
- [x] Implement operator permission lookup — `PlayerAccess` loads/persists `ops.json`, exposes op levels and op bypass checks used by login/spawn-protection/permission surfaces, and command handling updates operator state; covered by operator command tests, player-access load/save/hot-edit tests, and login-gate op-bypass coverage.
- [ ] Implement secure chat chain validation.
- [ ] Implement signed message body, link, signature, cache, and last-seen validation.
- [ ] Implement unsigned, modified, filtered, and deleted chat behavior.
- [ ] Implement command signing and signed argument tracking.
- [ ] Implement text filtering integration and fallbacks.
- [ ] Implement player reporting-relevant metadata where clients expect it.
- [ ] Implement prevent-proxy-connections behavior.
- [ ] Implement IP logging controls.
- [ ] Implement management server secret and TLS behavior.
