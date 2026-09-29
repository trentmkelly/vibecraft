# Client-only vanilla assets (documented deferral)

Scope: `decompiled-server-26.1.2/assets/**`. The dedicated server jar ships only three files
there: `assets/.mcassetsroot`, `assets/minecraft/lang/en_us.json`,
`assets/minecraft/lang/deprecated.json`. The Java evidence (grep of `net/**/*.java` for
`"/assets/` / `assets/minecraft` / `.mcassetsroot`):

| Java location | Reads |
| --- | --- |
| `net/minecraft/locale/Language.java:38` | `/assets/minecraft/lang/en_us.json` |
| `net/minecraft/locale/DeprecatedTranslationsInfo.java:46` | `/assets/minecraft/lang/deprecated.json` |
| `net/minecraft/server/packs/VanillaPackResourcesBuilder.java:37` | `"/" + type.getDirectory() + "/.mcassetsroot"` (pack-root probe per `PackType`) |

No other server code path references an `assets/` resource, so every other category below is
client-only: it is absent from the server jar and never loaded by the dedicated server. VibeCraft
defers them permanently (nothing to port):

- `models`, `textures`, `blockstates`, `items` (client item models), `equipment`, `atlases`
- `font`, `texts` (splash/credits/etc.), `waypoint_style`
- `sounds`, `sounds.json` (the server only uses `SoundEvent` registry ids, not the asset)
- `particles`, `shaders`, `post_effect`, `resourcepacks` (built-in client packs)
- `gpu_warnlist.json`, `regional_compliancies.json`

## Server-read files and their Rust coverage

- `en_us.json` + `deprecated.json`: vendored in `vanilla-data/assets/minecraft/lang/`, embedded in
  `src/language.rs`; `en_us()` applies `DeprecatedTranslationsInfo.applyToMap` exactly as
  `Language.loadDefault()` does (tests `deprecated_json_is_applied_to_the_live_language`,
  `apply_deprecated_matches_java_order_and_missing_rename`).
- `.mcassetsroot`: the JVM classpath probe locating the resource root. Rust embeds the assets at
  compile time (`include_str!`), so no runtime probe exists; the marker is vendored at
  `vanilla-data/assets/.mcassetsroot` to mirror the layout. Documented deferral of the probe.

Guard tests: `src/client_only_assets_tests.rs` (fails if the vendored tree gains an unclassified
file, or, with the optional decompilation, if the Java server references any other asset path).
